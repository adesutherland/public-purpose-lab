//! Managed-trust grant authority with an external non-exportable signer.
//!
//! The authority owns policy, idempotency and issuance evidence. A platform
//! adapter signs only the canonical bytes and returns a raw Ed25519 signature.

use std::path::PathBuf;

use ed25519_dalek::{Signature, VerifyingKey};
use ppl_aut_01::{PolicyAdapter, PolicyConfig};
use ppl_contracts::{
    AZ001_VERSION, AuthorisationDecisionRequest, AuthorisationDecisionStatus,
    DemonstrationSignInGrant, DemonstrationSignInGrantClaims, GrantSignature, I004_VERSION,
    KeyCustodyClass, PrincipalReference, PrincipalType, TrustProfile,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use crate::{
    DemonstrationConfiguration, GrantRequest, IamError, SyntheticTrustBundle, canonical_claims,
    enforce_obligations, format_time, hex, random_id, required_obligations, sha256_hex,
    validate_actor_request, validate_configuration,
};

const SIGNATURE_PROFILE: &str = "ppl-i004-ed25519-v1";
const GRANT_LIFETIME: Duration = Duration::minutes(2);

#[derive(Clone, Debug)]
pub struct ManagedGrantAuthority {
    database_path: PathBuf,
    trust: SyntheticTrustBundle,
    configuration: DemonstrationConfiguration,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ManagedGrantPlan {
    pub issuance_operation_id: String,
    pub request_digest: String,
    pub claims: DemonstrationSignInGrantClaims,
    pub decision_reference: String,
    pub obligations: Vec<String>,
}

impl ManagedGrantAuthority {
    /// Opens a managed issuer using public trust, protected configuration and a
    /// durable issuance store. No private key is accepted by this component.
    ///
    /// # Errors
    /// Refuses non-managed trust, local-file custody, configuration mismatch or
    /// unavailable state.
    pub fn open(
        database_path: impl Into<PathBuf>,
        trust: SyntheticTrustBundle,
        configuration: DemonstrationConfiguration,
    ) -> Result<Self, IamError> {
        if trust.record.trust_profile != TrustProfile::Managed
            || trust.record.key_custody_class == KeyCustodyClass::LocalFile
            || !trust.record.compatible
        {
            return Err(IamError::NotReady);
        }
        validate_configuration(&configuration, &trust.record.environment_id)?;
        validate_public_key(&trust)?;
        let authority = Self {
            database_path: database_path.into(),
            trust,
            configuration,
        };
        authority.prepare_store()?;
        Ok(authority)
    }

    /// Applies workload, actor and policy checks and durably reconciles one
    /// issuance operation before any external signing call.
    ///
    /// # Errors
    /// Refuses insufficient authority, actor/purpose mismatch, failed policy,
    /// operation conflict or unavailable protected state.
    pub fn prepare(
        &self,
        issuance_operation_id: &str,
        request: &GrantRequest,
        now: OffsetDateTime,
    ) -> Result<ManagedGrantPlan, IamError> {
        if issuance_operation_id.is_empty() {
            return Err(IamError::RequestRefused("issuance-operation-required"));
        }
        let request_bytes = serde_json::to_vec(request).map_err(|_| IamError::Serialization)?;
        let request_digest = sha256_hex(&request_bytes);
        if let Some((stored_digest, plan_json)) = self
            .connection()?
            .query_row(
                "SELECT request_digest,plan_json FROM managed_grant_issuance
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
            return serde_json::from_str(&plan_json).map_err(|_| IamError::StateInconsistent);
        }

        let (actor_id, decision_reference, obligations) = self.authorise_grant(request, now)?;

        let mut roles = request.roles.clone();
        roles.sort();
        let plan = ManagedGrantPlan {
            issuance_operation_id: issuance_operation_id.to_owned(),
            request_digest: request_digest.clone(),
            claims: DemonstrationSignInGrantClaims {
                grant_id: random_id("grant", 16)?,
                establishment_operation_id: random_id("establish", 16)?,
                environment_id: self.trust.record.environment_id.clone(),
                trust_domain: self.trust.record.trust_domain.clone(),
                trust_epoch: self.trust.record.trust_epoch,
                actor_id,
                application_id: request.application_id.clone(),
                audience: request.audience.clone(),
                surface_id: request.surface_id.clone(),
                demonstration_session_id: request.demonstration_session_id.clone(),
                roles,
                purpose: request.purpose.clone(),
                synthetic_realm: request.synthetic_realm.clone(),
                decision_reference: decision_reference.clone(),
                issued_at: format_time(now)?,
                not_before: format_time(now)?,
                expires_at: format_time(now + GRANT_LIFETIME)?,
            },
            decision_reference,
            obligations,
        };
        self.connection()?
            .execute(
                "INSERT INTO managed_grant_issuance(
                   issuance_operation_id,request_digest,plan_json,grant_json,prepared_at,completed_at
                 ) VALUES(?1,?2,?3,NULL,?4,NULL)",
                params![
                    issuance_operation_id,
                    request_digest,
                    serde_json::to_string(&plan).map_err(|_| IamError::Serialization)?,
                    format_time(now)?
                ],
            )
            .map_err(|_| IamError::StateUnavailable)?;
        Ok(plan)
    }

    fn authorise_grant(
        &self,
        request: &GrantRequest,
        now: OffsetDateTime,
    ) -> Result<(String, String, Vec<String>), IamError> {
        let workload = self
            .configuration
            .workloads
            .iter()
            .find(|candidate| candidate.workload_id == request.workload_id && candidate.enabled)
            .ok_or(IamError::RequestRefused("workload-not-authorised"))?;
        if !workload
            .audiences
            .iter()
            .any(|value| value == "iam-01-service")
            || !workload
                .contract_actions
                .iter()
                .any(|value| value == "I-004:request-grant")
        {
            return Err(IamError::RequestRefused("workload-authority-insufficient"));
        }
        let actor = self
            .configuration
            .actors
            .iter()
            .find(|candidate| candidate.actor_id == request.actor_id && candidate.enabled)
            .ok_or(IamError::RequestRefused("synthetic-actor-disabled"))?;
        validate_actor_request(actor, request)?;

        let principal = |principal_type, principal_id: String| PrincipalReference {
            principal_type,
            principal_id,
            environment_id: self.trust.record.environment_id.clone(),
            issuer: self.trust.record.signer_id.clone(),
            trust_domain: Some(self.trust.record.trust_domain.clone()),
        };
        let authorisation_request = AuthorisationDecisionRequest {
            contract_id: "AZ-001".to_owned(),
            contract_version: AZ001_VERSION.to_owned(),
            kind: "decision-request".to_owned(),
            request_id: random_id("authorisation-request", 12)?,
            environment_id: self.trust.record.environment_id.clone(),
            requester: principal(PrincipalType::Workload, workload.workload_id.clone()),
            actor: principal(PrincipalType::SyntheticHuman, actor.actor_id.clone()),
            action: "issue-synthetic-grant".to_owned(),
            resource: request.application_id.clone(),
            purpose: request.purpose.clone(),
            requested_roles: request.roles.clone(),
            assertions: self.configuration.assertions.clone(),
            policy_version: self.configuration.policy_version.clone(),
            requested_at: format_time(now)?,
        };
        let obligations = required_obligations(&self.configuration, &request.synthetic_realm)?;
        let allowed_resources = self
            .configuration
            .actors
            .iter()
            .flat_map(|registered| registered.applications.iter().cloned())
            .collect();
        let adapter = PolicyAdapter::new(PolicyConfig {
            environment_id: self.trust.record.environment_id.clone(),
            policy_version: self.configuration.policy_version.clone(),
            allowed_action: "issue-synthetic-grant".to_owned(),
            allowed_resources,
            relationship_source: self.configuration.relationship_source.clone(),
            consent_source: self.configuration.consent_source.clone(),
            obligations,
            dependency_available: true,
        });
        let decision = adapter.evaluate(&authorisation_request, now);
        if decision.status != AuthorisationDecisionStatus::Permit {
            return Err(IamError::AuthorisationRefused(decision.status));
        }
        enforce_obligations(&decision.obligations, &request.synthetic_realm)?;
        Ok((
            actor.actor_id.clone(),
            decision.decision_id,
            decision
                .obligations
                .iter()
                .map(|obligation| obligation.code.clone())
                .collect(),
        ))
    }

    #[must_use]
    pub fn canonical_payload(plan: &ManagedGrantPlan) -> Vec<u8> {
        canonical_claims(&plan.claims)
    }

    /// Verifies a platform signature against the pinned public key and records
    /// the completed grant idempotently.
    ///
    /// # Errors
    /// Refuses wrong signatures/operations or unavailable protected state.
    pub fn complete(
        &self,
        plan: &ManagedGrantPlan,
        signature: &[u8],
        now: OffsetDateTime,
    ) -> Result<DemonstrationSignInGrant, IamError> {
        let (stored_plan_json, stored_grant_json) = self
            .connection()?
            .query_row(
                "SELECT plan_json,grant_json FROM managed_grant_issuance
                 WHERE issuance_operation_id=?1 AND request_digest=?2",
                params![plan.issuance_operation_id, plan.request_digest],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()
            .map_err(|_| IamError::StateUnavailable)?
            .ok_or(IamError::StateInconsistent)?;
        let stored_plan: ManagedGrantPlan =
            serde_json::from_str(&stored_plan_json).map_err(|_| IamError::StateInconsistent)?;
        if stored_plan != *plan {
            return Err(IamError::RequestRefused("issuance-plan-conflict"));
        }
        if let Some(grant_json) = stored_grant_json {
            return serde_json::from_str(&grant_json).map_err(|_| IamError::StateInconsistent);
        }
        let key = validating_key(&self.trust)?;
        let parsed = Signature::try_from(signature).map_err(|_| IamError::CryptographyFailure)?;
        key.verify_strict(&canonical_claims(&plan.claims), &parsed)
            .map_err(|_| IamError::CryptographyFailure)?;
        let grant = DemonstrationSignInGrant {
            contract_id: "I-004".to_owned(),
            contract_version: I004_VERSION.to_owned(),
            claims: plan.claims.clone(),
            signature: GrantSignature {
                profile: SIGNATURE_PROFILE.to_owned(),
                algorithm: "Ed25519".to_owned(),
                signer_id: self.trust.record.signer_id.clone(),
                public_key_fingerprint: self.trust.record.public_key_fingerprint.clone(),
                value: hex(signature),
            },
        };
        let changed = self
            .connection()?
            .execute(
                "UPDATE managed_grant_issuance SET grant_json=?1,completed_at=?2
                 WHERE issuance_operation_id=?3 AND request_digest=?4 AND grant_json IS NULL",
                params![
                    serde_json::to_string(&grant).map_err(|_| IamError::Serialization)?,
                    format_time(now)?,
                    plan.issuance_operation_id,
                    plan.request_digest
                ],
            )
            .map_err(|_| IamError::StateUnavailable)?;
        if changed == 1 {
            Ok(grant)
        } else {
            Err(IamError::StateInconsistent)
        }
    }

    fn prepare_store(&self) -> Result<(), IamError> {
        if let Some(parent) = self.database_path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| IamError::StateUnavailable)?;
        }
        self.connection()?
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS managed_grant_issuance (
                   issuance_operation_id TEXT PRIMARY KEY,
                   request_digest TEXT NOT NULL,
                   plan_json TEXT NOT NULL,
                   grant_json TEXT,
                   prepared_at TEXT NOT NULL,
                   completed_at TEXT
                 );",
            )
            .map_err(|_| IamError::StateUnavailable)
    }

    fn connection(&self) -> Result<Connection, IamError> {
        let connection =
            Connection::open(&self.database_path).map_err(|_| IamError::StateUnavailable)?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| IamError::StateUnavailable)?;
        connection
            .pragma_update(None, "synchronous", "FULL")
            .map_err(|_| IamError::StateUnavailable)?;
        Ok(connection)
    }
}

fn validate_public_key(trust: &SyntheticTrustBundle) -> Result<(), IamError> {
    let key = validating_key(trust)?;
    let fingerprint = format!("sha256:{}", sha256_hex(key.as_bytes()));
    if fingerprint == trust.record.public_key_fingerprint {
        Ok(())
    } else {
        Err(IamError::CryptographyFailure)
    }
}

fn validating_key(trust: &SyntheticTrustBundle) -> Result<VerifyingKey, IamError> {
    let bytes = decode_hex(&trust.public_key).ok_or(IamError::CryptographyFailure)?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| IamError::CryptographyFailure)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| IamError::CryptographyFailure)
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16)?;
            let low = char::from(pair[1]).to_digit(16)?;
            u8::try_from((high << 4) | low).ok()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};
    use ppl_contracts::{
        AssertionStatus, AssertionType, AuthoritativeAssertion, EnvironmentClass,
        InformationProfile, KeyCustodyClass, RecoveryProfile, SyntheticTrustBootstrapRecord,
        TrustProfile, TrustStatus,
    };
    use tempfile::tempdir;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::*;
    use crate::{SyntheticActorRegistration, WorkloadRegistration};

    fn at(value: &str) -> OffsetDateTime {
        OffsetDateTime::parse(value, &Rfc3339).expect("test timestamp")
    }

    fn managed_fixture() -> (SigningKey, SyntheticTrustBundle, DemonstrationConfiguration) {
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let public_key = signing_key.verifying_key().to_bytes();
        let environment_id = "environment-managed-test";
        let now = at("2030-01-01T09:00:00Z");
        let trust = SyntheticTrustBundle {
            record: SyntheticTrustBootstrapRecord {
                contract_id: "I-003".to_owned(),
                contract_version: "1.0.0".to_owned(),
                record_id: "trust-record-managed-test".to_owned(),
                environment_id: environment_id.to_owned(),
                environment_class: EnvironmentClass::HostedShared,
                information_profile: InformationProfile::SyntheticOnly,
                trust_profile: TrustProfile::Managed,
                trust_domain: "urn:public-purpose-lab:managed-test".to_owned(),
                trust_epoch: 1,
                signer_id: "gcp-kms:projects/test-project/locations/global/keyRings/ppl/cryptoKeys/synthetic/cryptoKeyVersions/1".to_owned(),
                public_key_fingerprint: format!("sha256:{}", sha256_hex(&public_key)),
                key_custody_class: KeyCustodyClass::ManagedService,
                recovery_profile: RecoveryProfile::ProtectedSameEnvironment,
                status: TrustStatus::Ready,
                compatible: true,
                created_at: "2030-01-01T08:00:00Z".to_owned(),
                reason_code: None,
            },
            public_key: hex(&public_key),
        };
        let assertion = |source: &str, assertion_type| AuthoritativeAssertion {
            source_id: source.to_owned(),
            assertion_type,
            subject_id: "synthetic-audience-user".to_owned(),
            resource_id: "presentation-gateway".to_owned(),
            purpose_codes: vec!["demonstration".to_owned()],
            status: AssertionStatus::Active,
            effective_at: format_time(now - Duration::minutes(1)).expect("effective time"),
            expires_at: format_time(now + Duration::hours(1)).expect("expiry time"),
            version: "fixture-v1".to_owned(),
        };
        let configuration = DemonstrationConfiguration {
            environment_id: environment_id.to_owned(),
            policy_version: "managed-policy-v1".to_owned(),
            relationship_source: "synthetic-relationships".to_owned(),
            consent_source: "synthetic-consents".to_owned(),
            supported_obligations: vec!["mark-synthetic".to_owned(), "restrict-realm".to_owned()],
            workloads: vec![WorkloadRegistration {
                workload_id: "scenario-director".to_owned(),
                audiences: vec!["iam-01-service".to_owned()],
                contract_actions: vec!["I-004:request-grant".to_owned()],
                enabled: true,
            }],
            actors: vec![SyntheticActorRegistration {
                actor_id: "synthetic-audience-user".to_owned(),
                display_name: "Synthetic Audience User".to_owned(),
                roles: vec!["portal-viewer".to_owned()],
                applications: vec!["presentation-gateway".to_owned()],
                purposes: vec!["demonstration".to_owned()],
                synthetic_realm: "environment-fixture".to_owned(),
                enabled: true,
            }],
            assertions: vec![
                assertion("synthetic-relationships", AssertionType::Relationship),
                assertion("synthetic-consents", AssertionType::Consent),
            ],
        };
        (signing_key, trust, configuration)
    }

    fn request(session: &str) -> GrantRequest {
        GrantRequest {
            workload_id: "scenario-director".to_owned(),
            actor_id: "synthetic-audience-user".to_owned(),
            application_id: "presentation-gateway".to_owned(),
            audience: "presentation-gateway".to_owned(),
            surface_id: "audience-display".to_owned(),
            demonstration_session_id: session.to_owned(),
            roles: vec!["portal-viewer".to_owned()],
            purpose: "demonstration".to_owned(),
            synthetic_realm: "environment-fixture".to_owned(),
        }
    }

    #[test]
    fn externally_signed_grant_is_verified_and_reconciled_after_restart() {
        let directory = tempdir().expect("temporary directory");
        let database = directory.path().join("managed-issuer.sqlite");
        let now = at("2030-01-01T09:00:00Z");
        let (signer, trust, configuration) = managed_fixture();
        let authority =
            ManagedGrantAuthority::open(&database, trust.clone(), configuration.clone())
                .expect("managed authority");
        let plan = authority
            .prepare("issuance-1", &request("scenario-1"), now)
            .expect("prepared plan");
        let signature = signer.sign(&ManagedGrantAuthority::canonical_payload(&plan));
        let grant = authority
            .complete(&plan, &signature.to_bytes(), now)
            .expect("completed grant");
        assert_eq!(grant.claims.environment_id, trust.record.environment_id);
        assert_eq!(grant.signature.signer_id, trust.record.signer_id);

        drop(authority);
        let reopened = ManagedGrantAuthority::open(database, trust, configuration)
            .expect("reopened managed authority");
        let duplicate_plan = reopened
            .prepare(
                "issuance-1",
                &request("scenario-1"),
                now + Duration::seconds(1),
            )
            .expect("reconciled plan");
        let duplicate = reopened
            .complete(&duplicate_plan, &[0_u8; 64], now + Duration::seconds(1))
            .expect("reconciled completed grant");
        assert_eq!(duplicate, grant);
    }

    #[test]
    fn refuses_wrong_signature_operation_conflict_and_local_trust() {
        let directory = tempdir().expect("temporary directory");
        let now = at("2030-01-01T09:00:00Z");
        let (_, trust, configuration) = managed_fixture();
        let authority = ManagedGrantAuthority::open(
            directory.path().join("managed-issuer.sqlite"),
            trust.clone(),
            configuration.clone(),
        )
        .expect("managed authority");
        let plan = authority
            .prepare("issuance-1", &request("scenario-1"), now)
            .expect("prepared plan");
        let mut altered_plan = plan.clone();
        altered_plan.claims.demonstration_session_id = "scenario-substituted".to_owned();
        assert_eq!(
            authority.complete(&altered_plan, &[0_u8; 64], now),
            Err(IamError::RequestRefused("issuance-plan-conflict"))
        );
        assert_eq!(
            authority.complete(&plan, &[0_u8; 64], now),
            Err(IamError::CryptographyFailure)
        );
        assert_eq!(
            authority.prepare("issuance-1", &request("scenario-2"), now),
            Err(IamError::RequestRefused("issuance-operation-conflict"))
        );

        let mut local_trust = trust;
        local_trust.record.trust_profile = TrustProfile::LocalSynthetic;
        local_trust.record.key_custody_class = KeyCustodyClass::LocalFile;
        assert!(matches!(
            ManagedGrantAuthority::open(
                directory.path().join("local-root.sqlite"),
                local_trust,
                configuration
            ),
            Err(IamError::NotReady)
        ));
    }
}
