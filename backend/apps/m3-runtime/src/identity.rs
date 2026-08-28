use std::{
    fs,
    path::{Path, PathBuf},
};

use ppl_contracts::{
    AssertionStatus, AssertionType, AuthenticationStrength, AuthoritativeAssertion,
    DemonstrationSignInGrant, ExternalHumanIdentityContext, SyntheticSessionOutcome,
};
use ppl_iam_01::{
    DemonstrationConfiguration, GrantRequest, IamError, IamRuntime, SyntheticActorRegistration,
    SyntheticTrustBundle, WorkloadRegistration,
    managed_issuer::{ManagedGrantAuthority, ManagedGrantPlan},
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::{AppError, kms::KmsEd25519Signer};

#[derive(Clone)]
pub struct ManagedIdentityRuntimeConfig {
    pub configuration_path: PathBuf,
    pub kms_key_version: String,
    pub project_id: String,
}

pub const LOCAL_MAPPING_VERSION: &str = "m3-4-local-identity-v1";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SyntheticGrantRequestEvent {
    pub contract_id: String,
    pub contract_version: String,
    pub request_id: String,
    pub requested_at: String,
    pub request: GrantRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SyntheticGrantDeliveryEvent {
    pub contract_id: String,
    pub contract_version: String,
    pub request_id: String,
    pub delivered_at: String,
    pub grant: DemonstrationSignInGrant,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SyntheticIdentityOutcomeEvent {
    pub contract_id: String,
    pub contract_version: String,
    pub request_id: String,
    pub status: String,
    pub code: String,
    pub occurred_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synthetic_session: Option<SyntheticSessionOutcome>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SyntheticTerminationEvent {
    pub contract_id: String,
    pub contract_version: String,
    pub operation_id: String,
    pub demonstration_session_id: String,
    pub reason: String,
    pub requested_at: String,
}

pub struct LocalIdentityBroker {
    runtime: IamRuntime,
    trust: SyntheticTrustBundle,
    issuance_state_path: PathBuf,
}

impl LocalIdentityBroker {
    pub fn open(
        state_root: &Path,
        public_bundle_path: &Path,
        environment_id: &str,
        now: OffsetDateTime,
    ) -> Result<Self, IamError> {
        let runtime = IamRuntime::new(state_root);
        let record = runtime.bootstrap_local_for_environment(environment_id, now)?;
        let configuration_time = OffsetDateTime::parse(&record.created_at, &Rfc3339)
            .map_err(|_| IamError::StateInconsistent)?;
        runtime.configure(
            &configuration(&record.environment_id, configuration_time)?,
            configuration_time,
        )?;
        let trust = runtime.trust_bundle()?;
        publish_public_bundle(public_bundle_path, &trust)?;
        let issuance_state_path = state_root.join("local-issuance.sqlite");
        prepare_local_issuance_store(&issuance_state_path)?;
        Ok(Self {
            runtime,
            trust,
            issuance_state_path,
        })
    }

    pub fn issue_grant(
        &self,
        issuance_operation_id: &str,
        request: &GrantRequest,
        now: OffsetDateTime,
    ) -> Result<DemonstrationSignInGrant, IamError> {
        let request_digest = digest_json(request)?;
        let connection =
            Connection::open(&self.issuance_state_path).map_err(|_| IamError::StateUnavailable)?;
        if let Some((stored_digest, grant_json)) = connection
            .query_row(
                "SELECT request_digest,grant_json FROM local_grant_issuance
                 WHERE issuance_operation_id=?1",
                params![issuance_operation_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|_| IamError::StateUnavailable)?
        {
            if stored_digest != request_digest {
                return Err(IamError::RequestRefused("issuance-operation-conflict"));
            }
            return serde_json::from_str(&grant_json).map_err(|_| IamError::StateInconsistent);
        }
        let grant = self.runtime.issue_grant(request, now)?;
        connection
            .execute(
                "INSERT INTO local_grant_issuance VALUES(?1,?2,?3,?4)",
                params![
                    issuance_operation_id,
                    request_digest,
                    serde_json::to_string(&grant).map_err(|_| IamError::Serialization)?,
                    format_time(now)?
                ],
            )
            .map_err(|_| IamError::StateUnavailable)?;
        Ok(grant)
    }

    pub fn trust(&self) -> &SyntheticTrustBundle {
        &self.trust
    }
}

pub struct ManagedIdentityBroker {
    authority: ManagedGrantAuthority,
    signer: KmsEd25519Signer,
    trust: SyntheticTrustBundle,
}

impl ManagedIdentityBroker {
    pub async fn open(
        state_path: &Path,
        trust_bundle_path: &Path,
        configuration_path: &Path,
        environment_id: &str,
        kms_key_version: String,
        project_id: &str,
    ) -> Result<Self, AppError> {
        let trust = load_trust_bundle(trust_bundle_path)?;
        if trust.record.environment_id != environment_id
            || trust.record.signer_id != format!("gcp-kms:{kms_key_version}")
        {
            return Err(AppError::configuration("managed-trust-binding-mismatch"));
        }
        let configuration: DemonstrationConfiguration = serde_json::from_slice(
            &fs::read(configuration_path)
                .map_err(|_| AppError::configuration("identity-configuration-unavailable"))?,
        )
        .map_err(|_| AppError::configuration("identity-configuration-invalid"))?;
        let authority = ManagedGrantAuthority::open(state_path, trust.clone(), configuration)?;
        let signer = KmsEd25519Signer::new(
            kms_key_version,
            project_id,
            &trust.public_key,
            &trust.record.public_key_fingerprint,
        )
        .await?;
        Ok(Self {
            authority,
            signer,
            trust,
        })
    }

    pub async fn issue_grant(
        &self,
        issuance_operation_id: &str,
        request: &GrantRequest,
        now: OffsetDateTime,
    ) -> Result<DemonstrationSignInGrant, AppError> {
        let plan: ManagedGrantPlan = self
            .authority
            .prepare(issuance_operation_id, request, now)?;
        let signature = self
            .signer
            .sign(&ManagedGrantAuthority::canonical_payload(&plan))
            .await?;
        Ok(self.authority.complete(&plan, &signature, now)?)
    }

    pub fn trust(&self) -> &SyntheticTrustBundle {
        &self.trust
    }
}

pub enum IdentityBroker {
    Local(Box<LocalIdentityBroker>),
    Managed(Box<ManagedIdentityBroker>),
}

impl IdentityBroker {
    pub fn trust(&self) -> &SyntheticTrustBundle {
        match self {
            Self::Local(broker) => broker.trust(),
            Self::Managed(broker) => broker.trust(),
        }
    }

    pub async fn issue_grant(
        &self,
        issuance_operation_id: &str,
        request: &GrantRequest,
        now: OffsetDateTime,
    ) -> Result<DemonstrationSignInGrant, AppError> {
        match self {
            Self::Local(broker) => Ok(broker.issue_grant(issuance_operation_id, request, now)?),
            Self::Managed(broker) => {
                broker
                    .issue_grant(issuance_operation_id, request, now)
                    .await
            }
        }
    }
}

pub fn local_external_identity(
    environment_id: &str,
    audience: &str,
    principal_id: &str,
    role: &str,
    now: OffsetDateTime,
) -> Result<ExternalHumanIdentityContext, IamError> {
    Ok(ExternalHumanIdentityContext {
        contract_id: "I-001".to_owned(),
        contract_version: "1.0.0".to_owned(),
        context_id: format!("external-context-{}", Uuid::new_v4()),
        environment_id: environment_id.to_owned(),
        issuer: format!("urn:public-purpose-lab:test-identity:{environment_id}"),
        subject_id: format!("test-subject-{principal_id}"),
        principal_id: principal_id.to_owned(),
        roles: vec![role.to_owned()],
        audience: audience.to_owned(),
        authentication_strength: AuthenticationStrength::SingleFactor,
        mapping_version: LOCAL_MAPPING_VERSION.to_owned(),
        issued_at: format_time(now)?,
        expires_at: format_time(now + Duration::minutes(30))?,
        decision_reference: Some(format!("local-test-mapping-{role}")),
    })
}

pub fn load_trust_bundle(path: &Path) -> Result<SyntheticTrustBundle, IamError> {
    let bytes = fs::read(path).map_err(|_| IamError::StateUnavailable)?;
    serde_json::from_slice(&bytes).map_err(|_| IamError::StateInconsistent)
}

fn configuration(
    environment_id: &str,
    now: OffsetDateTime,
) -> Result<DemonstrationConfiguration, IamError> {
    let assertion = |source_id: &str, assertion_type: AssertionType, actor_id: &str| {
        Ok(AuthoritativeAssertion {
            source_id: source_id.to_owned(),
            assertion_type,
            subject_id: actor_id.to_owned(),
            resource_id: "presentation-gateway".to_owned(),
            purpose_codes: vec!["demonstrate-presentation".to_owned()],
            status: AssertionStatus::Active,
            effective_at: format_time(now - Duration::minutes(1))?,
            expires_at: format_time(now + Duration::days(7))?,
            version: "m3-4-synthetic-fixture-v1".to_owned(),
        })
    };
    Ok(DemonstrationConfiguration {
        environment_id: environment_id.to_owned(),
        policy_version: "m3-4-synthetic-policy-v1".to_owned(),
        relationship_source: "m3-4-synthetic-relationships".to_owned(),
        consent_source: "m3-4-synthetic-consents".to_owned(),
        supported_obligations: vec!["mark-synthetic".to_owned(), "restrict-realm".to_owned()],
        workloads: vec![WorkloadRegistration {
            workload_id: "scenario-director".to_owned(),
            audiences: vec!["iam-01-service".to_owned()],
            contract_actions: vec!["I-004:request-grant".to_owned()],
            enabled: true,
        }],
        actors: vec![
            SyntheticActorRegistration {
                actor_id: "synthetic-audience-user".to_owned(),
                display_name: "Alex Synthetic Viewer".to_owned(),
                roles: vec!["portal-viewer".to_owned()],
                applications: vec!["presentation-gateway".to_owned()],
                purposes: vec!["demonstrate-presentation".to_owned()],
                synthetic_realm: format!("synthetic-realm-{environment_id}"),
                enabled: true,
            },
            SyntheticActorRegistration {
                actor_id: "synthetic-reviewer".to_owned(),
                display_name: "Sam Synthetic Reviewer".to_owned(),
                roles: vec!["workbench-reviewer".to_owned()],
                applications: vec!["presentation-gateway".to_owned()],
                purposes: vec!["demonstrate-presentation".to_owned()],
                synthetic_realm: format!("synthetic-realm-{environment_id}"),
                enabled: true,
            },
        ],
        assertions: vec![
            assertion(
                "m3-4-synthetic-relationships",
                AssertionType::Relationship,
                "synthetic-audience-user",
            )?,
            assertion(
                "m3-4-synthetic-consents",
                AssertionType::Consent,
                "synthetic-audience-user",
            )?,
            assertion(
                "m3-4-synthetic-relationships",
                AssertionType::Relationship,
                "synthetic-reviewer",
            )?,
            assertion(
                "m3-4-synthetic-consents",
                AssertionType::Consent,
                "synthetic-reviewer",
            )?,
        ],
    })
}

fn publish_public_bundle(path: &Path, bundle: &SyntheticTrustBundle) -> Result<(), IamError> {
    let parent = path.parent().ok_or(IamError::StateUnavailable)?;
    fs::create_dir_all(parent).map_err(|_| IamError::StateUnavailable)?;
    let temporary = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(bundle).map_err(|_| IamError::Serialization)?;
    fs::write(&temporary, bytes).map_err(|_| IamError::StateUnavailable)?;
    fs::rename(temporary, path).map_err(|_| IamError::StateUnavailable)
}

fn prepare_local_issuance_store(path: &Path) -> Result<(), IamError> {
    let connection = Connection::open(path).map_err(|_| IamError::StateUnavailable)?;
    connection
        .execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=FULL;
             CREATE TABLE IF NOT EXISTS local_grant_issuance (
               issuance_operation_id TEXT PRIMARY KEY,
               request_digest TEXT NOT NULL,
               grant_json TEXT NOT NULL,
               issued_at TEXT NOT NULL
             );",
        )
        .map_err(|_| IamError::StateUnavailable)
}

fn digest_json<T: Serialize>(value: &T) -> Result<String, IamError> {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(value).map_err(|_| IamError::Serialization)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn format_time(value: OffsetDateTime) -> Result<String, IamError> {
    value.format(&Rfc3339).map_err(|_| IamError::Serialization)
}
