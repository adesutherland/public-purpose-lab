//! Single-host M2 reference implementation of IAM-01.
//!
//! The implementation qualifies only an isolated local-synthetic profile. It
//! proves environment-specific trust, bounded workload contexts, signed
//! short-lived grants, fail-closed policy decisions, replay-safe synthetic
//! session state and rebuild recovery. It has no network or browser login
//! endpoint and is not a managed or production identity service.

use std::{
    collections::HashSet,
    fmt,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use ppl_aut_01::{PolicyAdapter, PolicyConfig};
use ppl_contracts::{
    AZ001_VERSION, AuthorisationDecisionRequest, AuthorisationDecisionStatus,
    AuthorisationObligation, AuthoritativeAssertion, DemonstrationSignInGrant,
    DemonstrationSignInGrantClaims, EnvironmentClass, GrantSignature, I002_VERSION, I003_VERSION,
    I004_VERSION, I005_VERSION, InformationProfile, KeyCustodyClass, PrincipalReference,
    PrincipalType, RecoveryProfile, SyntheticSessionOutcome, SyntheticSessionStatus,
    SyntheticTrustBootstrapRecord, TrustProfile, TrustStatus, WorkloadIdentityContext,
};
use ppl_core::{ComponentDescriptor, Maturity};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

pub mod application_grants;
pub mod application_sessions;
pub mod managed_issuer;

const IAM_CONTRACTS: &[&str] = &["I-001", "I-002", "I-003", "I-004", "I-005", "AZ-001"];
const STATE_DIRECTORY: &str = "iam-01";
const ENVIRONMENT_FILE: &str = "environment.json";
const PRIVATE_KEY_FILE: &str = "local-signing-key.bin";
const CONFIGURATION_FILE: &str = "demonstration-configuration.json";
const JOURNAL_FILE: &str = "iam-security-journal.jsonl";
const LOCK_FILE: &str = "iam-security-journal.lock";
const SIGNATURE_PROFILE: &str = "ppl-i004-ed25519-v1";
const GRANT_LIFETIME: Duration = Duration::minutes(2);
const WORKLOAD_CONTEXT_LIFETIME: Duration = Duration::minutes(5);
const SESSION_LIFETIME: Duration = Duration::minutes(30);

#[must_use]
pub const fn descriptor() -> ComponentDescriptor {
    ComponentDescriptor {
        id: "IAM-01",
        name: "Identity, Trust and Synthetic Session Broker",
        maturity: Maturity::InDevelopment,
        contracts: IAM_CONTRACTS,
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdentityState {
    Ready,
    NotReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeclaredEnvironment {
    pub environment_class: EnvironmentClass,
    pub information_profile: InformationProfile,
    pub required_trust_profile: TrustProfile,
}

impl DeclaredEnvironment {
    #[must_use]
    pub const fn local_synthetic() -> Self {
        Self {
            environment_class: EnvironmentClass::LocalScratch,
            information_profile: InformationProfile::SyntheticOnly,
            required_trust_profile: TrustProfile::LocalSynthetic,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IamHealthReport {
    pub software_status: &'static str,
    pub identity_state: IdentityState,
    pub maturity: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<String>,
    pub environment_class: EnvironmentClass,
    pub information_profile: InformationProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_trust_profile: Option<TrustProfile>,
    pub required_trust_profile: TrustProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_epoch: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_custody_class: Option<KeyCustodyClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_profile: Option<RecoveryProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prominent_warning: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<&'static str>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkloadRegistration {
    pub workload_id: String,
    pub audiences: Vec<String>,
    pub contract_actions: Vec<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SyntheticActorRegistration {
    pub actor_id: String,
    pub display_name: String,
    pub roles: Vec<String>,
    pub applications: Vec<String>,
    pub purposes: Vec<String>,
    pub synthetic_realm: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DemonstrationConfiguration {
    pub environment_id: String,
    pub policy_version: String,
    pub relationship_source: String,
    pub consent_source: String,
    pub supported_obligations: Vec<String>,
    pub workloads: Vec<WorkloadRegistration>,
    pub actors: Vec<SyntheticActorRegistration>,
    pub assertions: Vec<AuthoritativeAssertion>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GrantRequest {
    pub workload_id: String,
    pub actor_id: String,
    pub application_id: String,
    pub audience: String,
    pub surface_id: String,
    pub demonstration_session_id: String,
    pub roles: Vec<String>,
    pub purpose: String,
    pub synthetic_realm: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IamError {
    StateUnavailable,
    StateInconsistent,
    ConfigurationInvalid,
    NotReady,
    RequestRefused(&'static str),
    AuthorisationRefused(AuthorisationDecisionStatus),
    RandomUnavailable,
    CryptographyFailure,
    Serialization,
}

impl fmt::Display for IamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::StateUnavailable => "IAM state is unavailable",
            Self::StateInconsistent => "IAM state is inconsistent",
            Self::ConfigurationInvalid => "IAM configuration is invalid",
            Self::NotReady => "IAM identity path is not ready",
            Self::RequestRefused(code) => code,
            Self::AuthorisationRefused(_) => "authorisation failed closed",
            Self::RandomUnavailable => "operating-system randomness is unavailable",
            Self::CryptographyFailure => "cryptographic validation failed",
            Self::Serialization => "IAM data could not be represented safely",
        })
    }
}

impl std::error::Error for IamError {}

#[derive(Clone, Debug)]
pub struct IamRuntime {
    state_root: PathBuf,
    declared_environment: DeclaredEnvironment,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct EnvironmentState {
    record: SyntheticTrustBootstrapRecord,
    public_key: String,
}

/// Public, environment-bound material distributed to application validators.
/// It contains no signing key or reusable workload credential.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SyntheticTrustBundle {
    pub record: SyntheticTrustBootstrapRecord,
    pub public_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
enum JournalRecord {
    Bootstrap {
        recorded_at: String,
        trust_domain: String,
        signer_fingerprint: String,
    },
    Configuration {
        recorded_at: String,
        configuration_digest: String,
        policy_version: String,
    },
    GrantIssued {
        recorded_at: String,
        grant_digest: String,
        operation_digest: String,
        actor_digest: String,
        application_id: String,
        decision_reference: String,
        obligations: Vec<String>,
    },
    SessionEstablished {
        recorded_at: String,
        grant_digest: String,
        operation_digest: String,
        outcome_id: String,
        session_reference: String,
        maximum_valid_until: String,
        actor_id: String,
        application_id: String,
        surface_id: String,
        demonstration_session_id: String,
        roles: Vec<String>,
        synthetic_realm: String,
        decision_reference: String,
    },
    SessionTerminal {
        recorded_at: String,
        outcome_id: String,
        session_reference: String,
        status: SyntheticSessionStatus,
        reason_code: String,
    },
    TrustRevoked {
        recorded_at: String,
        reason_code: String,
    },
}

impl IamRuntime {
    #[must_use]
    pub fn new(state_dir: impl Into<PathBuf>) -> Self {
        Self::with_declared_environment(state_dir, DeclaredEnvironment::local_synthetic())
    }

    /// Exports only the public trust record and verifying key.
    ///
    /// # Errors
    /// Refuses incomplete, revoked or inconsistent trust state.
    pub fn trust_bundle(&self) -> Result<SyntheticTrustBundle, IamError> {
        self.require_ready()?;
        let environment = self.load_environment()?;
        Ok(SyntheticTrustBundle {
            record: environment.record,
            public_key: environment.public_key,
        })
    }

    #[must_use]
    pub fn with_declared_environment(
        state_dir: impl Into<PathBuf>,
        declared_environment: DeclaredEnvironment,
    ) -> Self {
        Self {
            state_root: state_dir.into().join(STATE_DIRECTORY),
            declared_environment,
        }
    }

    /// Creates or returns one local-synthetic trust domain.
    ///
    /// # Errors
    ///
    /// Fails closed if state exists only partially, permissions or randomness
    /// are unavailable, or the stored state is inconsistent.
    pub fn bootstrap_local(
        &self,
        now: OffsetDateTime,
    ) -> Result<SyntheticTrustBootstrapRecord, IamError> {
        self.bootstrap_local_with_environment(None, now)
    }

    /// Creates or returns local-synthetic trust for an environment identifier
    /// generated by the enclosing environment setup.
    ///
    /// # Errors
    /// Refuses malformed identifiers, mismatch with existing state or any
    /// partial/incompatible trust state.
    pub fn bootstrap_local_for_environment(
        &self,
        environment_id: &str,
        now: OffsetDateTime,
    ) -> Result<SyntheticTrustBootstrapRecord, IamError> {
        if environment_id.len() < 8
            || environment_id.len() > 128
            || !environment_id
                .chars()
                .all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | ':' | '_'))
        {
            return Err(IamError::ConfigurationInvalid);
        }
        self.bootstrap_local_with_environment(Some(environment_id), now)
    }

    fn bootstrap_local_with_environment(
        &self,
        environment_id: Option<&str>,
        now: OffsetDateTime,
    ) -> Result<SyntheticTrustBootstrapRecord, IamError> {
        self.prepare_directory()?;
        let lock = self.open_lock()?;
        lock.lock().map_err(|_| IamError::StateUnavailable)?;
        let result = self.bootstrap_local_locked(environment_id, now);
        let unlock = lock.unlock().map_err(|_| IamError::StateUnavailable);
        match (result, unlock) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    fn bootstrap_local_locked(
        &self,
        requested_environment_id: Option<&str>,
        now: OffsetDateTime,
    ) -> Result<SyntheticTrustBootstrapRecord, IamError> {
        if required_profile(self.declared_environment) != TrustProfile::LocalSynthetic
            || self.declared_environment.information_profile != InformationProfile::SyntheticOnly
            || !matches!(
                self.declared_environment.environment_class,
                EnvironmentClass::LocalScratch | EnvironmentClass::PortableIsolated
            )
        {
            return Err(IamError::NotReady);
        }
        let environment_path = self.state_root.join(ENVIRONMENT_FILE);
        let key_path = self.state_root.join(PRIVATE_KEY_FILE);
        if environment_path.exists() {
            let environment = self.load_environment()?;
            if requested_environment_id
                .is_some_and(|value| value != environment.record.environment_id)
            {
                return Err(IamError::StateInconsistent);
            }
            drop(self.load_local_signer(&environment)?);
            self.load_journal()?;
            return Ok(environment.record);
        }
        if key_path.exists()
            || self.state_root.join(CONFIGURATION_FILE).exists()
            || self.state_root.join(JOURNAL_FILE).exists()
        {
            return Err(IamError::StateInconsistent);
        }

        let environment_id = requested_environment_id
            .map_or_else(|| random_id("env", 16), |value| Ok(value.to_owned()))?;
        let trust_domain = random_id("trust", 16)?;
        let mut secret = [0_u8; 32];
        getrandom::fill(&mut secret).map_err(|_| IamError::RandomUnavailable)?;
        let signing_key = SigningKey::from_bytes(&secret);
        let public_key = signing_key.verifying_key().to_bytes();
        let fingerprint = format!("sha256:{}", sha256_hex(&public_key));
        let signer_id = format!("signer-{}", &fingerprint[7..23]);
        write_private_key(&key_path, &secret)?;

        let record = SyntheticTrustBootstrapRecord {
            contract_id: "I-003".to_owned(),
            contract_version: I003_VERSION.to_owned(),
            record_id: random_id("trust-bootstrap", 12)?,
            environment_id,
            environment_class: EnvironmentClass::LocalScratch,
            information_profile: InformationProfile::SyntheticOnly,
            trust_profile: TrustProfile::LocalSynthetic,
            trust_domain: trust_domain.clone(),
            trust_epoch: 1,
            signer_id,
            public_key_fingerprint: fingerprint.clone(),
            key_custody_class: KeyCustodyClass::LocalFile,
            recovery_profile: RecoveryProfile::RebuildNewTrustDomain,
            status: TrustStatus::Ready,
            compatible: true,
            created_at: format_time(now)?,
            reason_code: None,
        };
        let environment = EnvironmentState {
            record: record.clone(),
            public_key: hex(&public_key),
        };
        write_new_json(&environment_path, &environment)?;
        self.append_record(&JournalRecord::Bootstrap {
            recorded_at: format_time(now)?,
            trust_domain,
            signer_fingerprint: fingerprint,
        })?;
        Ok(record)
    }

    /// Installs one synthetic demonstration configuration.
    ///
    /// # Errors
    ///
    /// Refuses replacement, mismatched environments, duplicate registrations,
    /// unsupported obligations or malformed authority configuration.
    pub fn configure(
        &self,
        configuration: &DemonstrationConfiguration,
        now: OffsetDateTime,
    ) -> Result<(), IamError> {
        let lock = self.open_lock()?;
        lock.lock().map_err(|_| IamError::StateUnavailable)?;
        let result = self.configure_locked(configuration, now);
        let unlock = lock.unlock().map_err(|_| IamError::StateUnavailable);
        match (result, unlock) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    fn configure_locked(
        &self,
        configuration: &DemonstrationConfiguration,
        now: OffsetDateTime,
    ) -> Result<(), IamError> {
        let environment = self.load_environment()?;
        drop(self.load_local_signer(&environment)?);
        self.load_journal()?;
        validate_configuration(configuration, &environment.record.environment_id)?;
        let path = self.state_root.join(CONFIGURATION_FILE);
        if path.exists() {
            let existing: DemonstrationConfiguration = read_json(&path)?;
            return if existing == *configuration {
                Ok(())
            } else {
                Err(IamError::ConfigurationInvalid)
            };
        }
        write_new_json(&path, configuration)?;
        let bytes = serde_json::to_vec(configuration).map_err(|_| IamError::Serialization)?;
        self.append_record(&JournalRecord::Configuration {
            recorded_at: format_time(now)?,
            configuration_digest: sha256_hex(&bytes),
            policy_version: configuration.policy_version.clone(),
        })
    }

    #[must_use]
    pub fn health(&self, declared: DeclaredEnvironment) -> IamHealthReport {
        self.health_result(declared)
            .unwrap_or_else(|error| not_ready_report(declared, error_reason(&error)))
    }

    fn health_result(&self, declared: DeclaredEnvironment) -> Result<IamHealthReport, IamError> {
        let environment = self.load_environment()?;
        drop(self.load_local_signer(&environment)?);
        let records = self.load_journal()?;
        if records
            .iter()
            .any(|record| matches!(record, JournalRecord::TrustRevoked { .. }))
        {
            return Err(IamError::NotReady);
        }
        let _: DemonstrationConfiguration = read_json(&self.state_root.join(CONFIGURATION_FILE))?;
        let compatible = profile_compatible(&environment.record, declared);
        Ok(IamHealthReport {
            software_status: "healthy",
            identity_state: if compatible {
                IdentityState::Ready
            } else {
                IdentityState::NotReady
            },
            maturity: "in-development",
            environment_id: Some(environment.record.environment_id.clone()),
            environment_class: declared.environment_class,
            information_profile: declared.information_profile,
            active_trust_profile: Some(environment.record.trust_profile),
            required_trust_profile: required_profile(declared),
            trust_domain: Some(environment.record.trust_domain.clone()),
            trust_epoch: Some(environment.record.trust_epoch),
            signer_fingerprint: Some(environment.record.public_key_fingerprint.clone()),
            key_custody_class: Some(environment.record.key_custody_class),
            recovery_profile: Some(environment.record.recovery_profile),
            prominent_warning: Some("LOCAL-SYNTHETIC TRUST - ISOLATED SCRATCH USE ONLY"),
            reason_code: (!compatible).then_some("trust-profile-incompatible"),
        })
    }

    /// Produces a bounded context for a configured local-process workload.
    ///
    /// # Errors
    ///
    /// Refuses unknown, disabled or insufficiently scoped workloads and any
    /// environment whose identity state is not ready.
    pub fn workload_context(
        &self,
        workload_id: &str,
        now: OffsetDateTime,
    ) -> Result<WorkloadIdentityContext, IamError> {
        self.require_ready()?;
        let environment = self.load_environment()?;
        let configuration = self.load_configuration()?;
        let workload = configuration
            .workloads
            .iter()
            .find(|candidate| candidate.workload_id == workload_id && candidate.enabled)
            .ok_or(IamError::RequestRefused("workload-not-authorised"))?;
        Ok(WorkloadIdentityContext {
            contract_id: "I-002".to_owned(),
            contract_version: I002_VERSION.to_owned(),
            context_id: random_id("workload-context", 12)?,
            environment_id: environment.record.environment_id.clone(),
            issuer: environment.record.signer_id.clone(),
            workload_id: workload.workload_id.clone(),
            audiences: workload.audiences.clone(),
            contract_actions: workload.contract_actions.clone(),
            attestation_reference: format!(
                "local-process-attestation-{}",
                digest_prefix(&workload.workload_id)
            ),
            policy_version: configuration.policy_version,
            issued_at: format_time(now)?,
            expires_at: format_time(now + WORKLOAD_CONTEXT_LIFETIME)?,
        })
    }

    /// Issues one signed, short-lived, application-bound grant.
    ///
    /// # Errors
    ///
    /// Refuses invalid workload/actor scope, failed policy decisions,
    /// unsupported obligations or unavailable protected state.
    pub fn issue_grant(
        &self,
        request: &GrantRequest,
        now: OffsetDateTime,
    ) -> Result<DemonstrationSignInGrant, IamError> {
        let lock = self.open_lock()?;
        lock.lock().map_err(|_| IamError::StateUnavailable)?;
        let result = self.issue_grant_locked(request, now);
        let unlock = lock.unlock().map_err(|_| IamError::StateUnavailable);
        match (result, unlock) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn issue_grant_locked(
        &self,
        request: &GrantRequest,
        now: OffsetDateTime,
    ) -> Result<DemonstrationSignInGrant, IamError> {
        self.require_ready_locked()?;
        let environment = self.load_environment()?;
        let configuration = self.load_configuration()?;
        let signer = self.load_local_signer(&environment)?;
        let workload = configuration
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
        let actor = configuration
            .actors
            .iter()
            .find(|candidate| candidate.actor_id == request.actor_id && candidate.enabled)
            .ok_or(IamError::RequestRefused("synthetic-actor-disabled"))?;
        validate_actor_request(actor, request)?;

        let principal = |principal_type, principal_id: String| PrincipalReference {
            principal_type,
            principal_id,
            environment_id: environment.record.environment_id.clone(),
            issuer: environment.record.signer_id.clone(),
            trust_domain: Some(environment.record.trust_domain.clone()),
        };
        let authorisation_request = AuthorisationDecisionRequest {
            contract_id: "AZ-001".to_owned(),
            contract_version: AZ001_VERSION.to_owned(),
            kind: "decision-request".to_owned(),
            request_id: random_id("authorisation-request", 12)?,
            environment_id: environment.record.environment_id.clone(),
            requester: principal(PrincipalType::Workload, workload.workload_id.clone()),
            actor: principal(PrincipalType::SyntheticHuman, actor.actor_id.clone()),
            action: "issue-synthetic-grant".to_owned(),
            resource: request.application_id.clone(),
            purpose: request.purpose.clone(),
            requested_roles: request.roles.clone(),
            assertions: configuration.assertions.clone(),
            policy_version: configuration.policy_version.clone(),
            requested_at: format_time(now)?,
        };
        let obligations = required_obligations(&configuration, &request.synthetic_realm)?;
        let allowed_resources = configuration
            .actors
            .iter()
            .flat_map(|registered| registered.applications.iter().cloned())
            .collect();
        let adapter = PolicyAdapter::new(PolicyConfig {
            environment_id: environment.record.environment_id.clone(),
            policy_version: configuration.policy_version.clone(),
            allowed_action: "issue-synthetic-grant".to_owned(),
            allowed_resources,
            relationship_source: configuration.relationship_source.clone(),
            consent_source: configuration.consent_source.clone(),
            obligations,
            dependency_available: true,
        });
        let decision = adapter.evaluate(&authorisation_request, now);
        if decision.status != AuthorisationDecisionStatus::Permit {
            return Err(IamError::AuthorisationRefused(decision.status));
        }
        enforce_obligations(&decision.obligations, &request.synthetic_realm)?;

        let mut roles = request.roles.clone();
        roles.sort();
        let claims = DemonstrationSignInGrantClaims {
            grant_id: random_id("grant", 16)?,
            establishment_operation_id: random_id("establish", 16)?,
            environment_id: environment.record.environment_id.clone(),
            trust_domain: environment.record.trust_domain.clone(),
            trust_epoch: environment.record.trust_epoch,
            actor_id: actor.actor_id.clone(),
            application_id: request.application_id.clone(),
            audience: request.audience.clone(),
            surface_id: request.surface_id.clone(),
            demonstration_session_id: request.demonstration_session_id.clone(),
            roles,
            purpose: request.purpose.clone(),
            synthetic_realm: request.synthetic_realm.clone(),
            decision_reference: decision.decision_id.clone(),
            issued_at: format_time(now)?,
            not_before: format_time(now)?,
            expires_at: format_time(now + GRANT_LIFETIME)?,
        };
        let signature = signer.sign(&canonical_claims(&claims));
        let grant = DemonstrationSignInGrant {
            contract_id: "I-004".to_owned(),
            contract_version: I004_VERSION.to_owned(),
            claims,
            signature,
        };
        let grant_digest = grant_digest(&grant)?;
        self.append_record(&JournalRecord::GrantIssued {
            recorded_at: format_time(now)?,
            grant_digest,
            operation_digest: sha256_hex(grant.claims.establishment_operation_id.as_bytes()),
            actor_digest: sha256_hex(grant.claims.actor_id.as_bytes()),
            application_id: grant.claims.application_id.clone(),
            decision_reference: decision.decision_id,
            obligations: decision
                .obligations
                .iter()
                .map(|obligation| obligation.code.clone())
                .collect(),
        })?;
        Ok(grant)
    }

    /// Validates and establishes at most one backend-only synthetic session.
    ///
    /// # Errors
    ///
    /// Returns an infrastructure error only when protected state cannot be
    /// trusted. Invalid grants produce a safe refused `I-005` outcome.
    pub fn establish_session(
        &self,
        grant: &DemonstrationSignInGrant,
        now: OffsetDateTime,
    ) -> Result<SyntheticSessionOutcome, IamError> {
        let lock = self.open_lock()?;
        lock.lock().map_err(|_| IamError::StateUnavailable)?;
        let result = self.establish_session_locked(grant, now);
        let unlock = lock.unlock().map_err(|_| IamError::StateUnavailable);
        match (result, unlock) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn establish_session_locked(
        &self,
        grant: &DemonstrationSignInGrant,
        now: OffsetDateTime,
    ) -> Result<SyntheticSessionOutcome, IamError> {
        let environment = self.load_environment()?;
        let configuration = self.load_configuration()?;
        let records = self.load_journal()?;
        if !profile_compatible(&environment.record, self.declared_environment) {
            return refused_outcome(grant, "trust-profile-incompatible", now);
        }
        if records
            .iter()
            .any(|record| matches!(record, JournalRecord::TrustRevoked { .. }))
        {
            return refused_outcome(grant, "trust-or-signer-revoked", now);
        }
        if let Err(code) = verify_grant(grant, &environment, now) {
            return refused_outcome(grant, code, now);
        }
        let Some(actor) = configuration
            .actors
            .iter()
            .find(|actor| actor.actor_id == grant.claims.actor_id && actor.enabled)
        else {
            return refused_outcome(grant, "synthetic-actor-disabled", now);
        };
        if !actor.applications.contains(&grant.claims.application_id)
            || !grant
                .claims
                .roles
                .iter()
                .all(|role| actor.roles.contains(role))
            || actor.synthetic_realm != grant.claims.synthetic_realm
        {
            return refused_outcome(grant, "actor-role-or-realm-refused", now);
        }

        let digest = grant_digest(grant)?;
        let operation_digest = sha256_hex(grant.claims.establishment_operation_id.as_bytes());
        if let Some(existing) = records.iter().find_map(|record| match record {
            JournalRecord::SessionEstablished {
                grant_digest,
                operation_digest: recorded_operation,
                outcome_id,
                session_reference,
                maximum_valid_until,
                ..
            } if recorded_operation == &operation_digest => Some((
                grant_digest,
                outcome_id,
                session_reference,
                maximum_valid_until,
            )),
            _ => None,
        }) {
            if existing.0 != &digest {
                return refused_outcome(grant, "establishment-operation-conflict", now);
            }
            return established_outcome(
                grant,
                existing.1.clone(),
                existing.2.clone(),
                existing.3.clone(),
                now,
                Some(existing.1.clone()),
            );
        }
        let issued = records.iter().any(|record| match record {
            JournalRecord::GrantIssued {
                grant_digest,
                decision_reference,
                obligations,
                ..
            } => {
                grant_digest == &digest
                    && decision_reference == &grant.claims.decision_reference
                    && obligations.iter().any(|value| value == "mark-synthetic")
                    && obligations.iter().any(|value| value == "restrict-realm")
            }
            _ => false,
        });
        if !issued {
            return refused_outcome(grant, "grant-issuance-not-established", now);
        }

        let mut random = [0_u8; 32];
        getrandom::fill(&mut random).map_err(|_| IamError::RandomUnavailable)?;
        let session_reference = format!("session-{}", sha256_hex(&random));
        let grant_expiry =
            parse_time(&grant.claims.expires_at).map_err(|_| IamError::CryptographyFailure)?;
        let maximum_valid_until = format_time(grant_expiry.min(now + SESSION_LIFETIME))?;
        let outcome_id = random_id("session-outcome", 12)?;
        self.append_record(&JournalRecord::SessionEstablished {
            recorded_at: format_time(now)?,
            grant_digest: digest,
            operation_digest,
            outcome_id: outcome_id.clone(),
            session_reference: session_reference.clone(),
            maximum_valid_until: maximum_valid_until.clone(),
            actor_id: grant.claims.actor_id.clone(),
            application_id: grant.claims.application_id.clone(),
            surface_id: grant.claims.surface_id.clone(),
            demonstration_session_id: grant.claims.demonstration_session_id.clone(),
            roles: grant.claims.roles.clone(),
            synthetic_realm: grant.claims.synthetic_realm.clone(),
            decision_reference: grant.claims.decision_reference.clone(),
        })?;
        established_outcome(
            grant,
            outcome_id,
            session_reference,
            maximum_valid_until,
            now,
            None,
        )
    }

    /// Terminates a safe session reference idempotently.
    ///
    /// # Errors
    ///
    /// Refuses unknown sessions or unavailable/corrupt security state.
    pub fn terminate_session(
        &self,
        session_reference: &str,
        reason_code: &str,
        now: OffsetDateTime,
    ) -> Result<SyntheticSessionOutcome, IamError> {
        let lock = self.open_lock()?;
        lock.lock().map_err(|_| IamError::StateUnavailable)?;
        let result = self.terminate_session_locked(session_reference, reason_code, now);
        let unlock = lock.unlock().map_err(|_| IamError::StateUnavailable);
        match (result, unlock) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    fn terminate_session_locked(
        &self,
        session_reference: &str,
        reason_code: &str,
        now: OffsetDateTime,
    ) -> Result<SyntheticSessionOutcome, IamError> {
        if !valid_reason_code(reason_code) {
            return Err(IamError::RequestRefused("termination-reason-invalid"));
        }
        let environment = self.load_environment()?;
        let records = self.load_journal()?;
        let established = records.iter().find_map(|record| match record {
            JournalRecord::SessionEstablished {
                outcome_id,
                session_reference: candidate,
                actor_id,
                application_id,
                surface_id,
                demonstration_session_id,
                roles,
                synthetic_realm,
                decision_reference,
                ..
            } if candidate == session_reference => Some((
                outcome_id,
                actor_id,
                application_id,
                surface_id,
                demonstration_session_id,
                roles,
                synthetic_realm,
                decision_reference,
            )),
            _ => None,
        });
        let Some(established) = established else {
            return Err(IamError::RequestRefused("session-not-found"));
        };
        let terminal = records.iter().rev().find_map(|record| match record {
            JournalRecord::SessionTerminal {
                outcome_id,
                session_reference: candidate,
                status,
                reason_code,
                recorded_at,
            } if candidate == session_reference => {
                Some((outcome_id, *status, reason_code, recorded_at))
            }
            _ => None,
        });
        if let Some(terminal) = terminal {
            return Ok(terminal_outcome(
                &environment.record.environment_id,
                session_reference,
                established,
                terminal.0.clone(),
                terminal.1,
                terminal.2.clone(),
                terminal.3.clone(),
            ));
        }
        let outcome_id = random_id("session-outcome", 12)?;
        let recorded_at = format_time(now)?;
        self.append_record(&JournalRecord::SessionTerminal {
            recorded_at: recorded_at.clone(),
            outcome_id: outcome_id.clone(),
            session_reference: session_reference.to_owned(),
            status: SyntheticSessionStatus::Terminated,
            reason_code: reason_code.to_owned(),
        })?;
        Ok(terminal_outcome(
            &environment.record.environment_id,
            session_reference,
            established,
            outcome_id,
            SyntheticSessionStatus::Terminated,
            reason_code.to_owned(),
            recorded_at,
        ))
    }

    /// Revokes the local trust domain. Re-enabling it is not supported.
    ///
    /// # Errors
    ///
    /// Refuses invalid reasons or unavailable/corrupt state.
    pub fn revoke_trust(&self, reason_code: &str, now: OffsetDateTime) -> Result<(), IamError> {
        if !valid_reason_code(reason_code) {
            return Err(IamError::RequestRefused("revocation-reason-invalid"));
        }
        let lock = self.open_lock()?;
        lock.lock().map_err(|_| IamError::StateUnavailable)?;
        let result = (|| {
            self.load_environment()?;
            let records = self.load_journal()?;
            if records
                .iter()
                .any(|record| matches!(record, JournalRecord::TrustRevoked { .. }))
            {
                return Ok(());
            }
            self.append_record(&JournalRecord::TrustRevoked {
                recorded_at: format_time(now)?,
                reason_code: reason_code.to_owned(),
            })
        })();
        let unlock = lock.unlock().map_err(|_| IamError::StateUnavailable);
        match (result, unlock) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    fn require_ready(&self) -> Result<(), IamError> {
        if self.health(self.declared_environment).identity_state == IdentityState::Ready {
            Ok(())
        } else {
            Err(IamError::NotReady)
        }
    }

    fn require_ready_locked(&self) -> Result<(), IamError> {
        let environment = self.load_environment()?;
        if !profile_compatible(&environment.record, self.declared_environment) {
            return Err(IamError::NotReady);
        }
        drop(self.load_local_signer(&environment)?);
        self.load_configuration()?;
        let records = self.load_journal()?;
        if records
            .iter()
            .any(|record| matches!(record, JournalRecord::TrustRevoked { .. }))
        {
            return Err(IamError::NotReady);
        }
        Ok(())
    }

    fn prepare_directory(&self) -> Result<(), IamError> {
        fs::create_dir_all(&self.state_root).map_err(|_| IamError::StateUnavailable)?;
        restrict_directory(&self.state_root)
    }

    fn open_lock(&self) -> Result<File, IamError> {
        self.prepare_directory()?;
        open_restricted(&self.state_root.join(LOCK_FILE), true, false)
    }

    fn load_environment(&self) -> Result<EnvironmentState, IamError> {
        read_json(&self.state_root.join(ENVIRONMENT_FILE))
    }

    fn load_configuration(&self) -> Result<DemonstrationConfiguration, IamError> {
        read_json(&self.state_root.join(CONFIGURATION_FILE))
    }

    fn load_local_signer(&self, environment: &EnvironmentState) -> Result<LocalSigner, IamError> {
        let mut file = open_restricted(&self.state_root.join(PRIVATE_KEY_FILE), false, false)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|_| IamError::StateUnavailable)?;
        let secret: [u8; 32] = bytes.try_into().map_err(|_| IamError::StateInconsistent)?;
        let signing_key = SigningKey::from_bytes(&secret);
        let public_key = signing_key.verifying_key().to_bytes();
        if hex(&public_key) != environment.public_key
            || format!("sha256:{}", sha256_hex(&public_key))
                != environment.record.public_key_fingerprint
        {
            return Err(IamError::StateInconsistent);
        }
        Ok(LocalSigner {
            signing_key,
            signer_id: environment.record.signer_id.clone(),
            public_key_fingerprint: environment.record.public_key_fingerprint.clone(),
        })
    }

    fn load_journal(&self) -> Result<Vec<JournalRecord>, IamError> {
        let path = self.state_root.join(JOURNAL_FILE);
        if !path.exists() {
            return Err(IamError::StateInconsistent);
        }
        let file = open_restricted(&path, false, false)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|_| IamError::StateUnavailable)?;
            if line.trim().is_empty() {
                return Err(IamError::StateInconsistent);
            }
            records.push(serde_json::from_str(&line).map_err(|_| IamError::StateInconsistent)?);
        }
        if records.is_empty() || !matches!(records.first(), Some(JournalRecord::Bootstrap { .. })) {
            return Err(IamError::StateInconsistent);
        }
        Ok(records)
    }

    fn append_record(&self, record: &JournalRecord) -> Result<(), IamError> {
        let bytes = serde_json::to_vec(record).map_err(|_| IamError::Serialization)?;
        let mut file = open_restricted(&self.state_root.join(JOURNAL_FILE), true, false)?;
        file.write_all(&bytes)
            .and_then(|()| file.write_all(b"\n"))
            .and_then(|()| file.sync_data())
            .map_err(|_| IamError::StateUnavailable)
    }
}

struct LocalSigner {
    signing_key: SigningKey,
    signer_id: String,
    public_key_fingerprint: String,
}

impl LocalSigner {
    fn sign(&self, message: &[u8]) -> GrantSignature {
        let signature = self.signing_key.sign(message);
        GrantSignature {
            profile: SIGNATURE_PROFILE.to_owned(),
            algorithm: "Ed25519".to_owned(),
            signer_id: self.signer_id.clone(),
            public_key_fingerprint: self.public_key_fingerprint.clone(),
            value: hex(&signature.to_bytes()),
        }
    }
}

pub(crate) fn validate_configuration(
    configuration: &DemonstrationConfiguration,
    environment_id: &str,
) -> Result<(), IamError> {
    if configuration.environment_id != environment_id
        || configuration.policy_version.is_empty()
        || configuration.relationship_source.is_empty()
        || configuration.consent_source.is_empty()
        || !configuration
            .supported_obligations
            .iter()
            .any(|value| value == "mark-synthetic")
        || !configuration
            .supported_obligations
            .iter()
            .any(|value| value == "restrict-realm")
        || configuration.workloads.is_empty()
        || configuration.actors.is_empty()
    {
        return Err(IamError::ConfigurationInvalid);
    }
    let workloads: HashSet<_> = configuration
        .workloads
        .iter()
        .map(|workload| workload.workload_id.as_str())
        .collect();
    let actors: HashSet<_> = configuration
        .actors
        .iter()
        .map(|actor| actor.actor_id.as_str())
        .collect();
    if workloads.len() != configuration.workloads.len()
        || actors.len() != configuration.actors.len()
        || configuration.workloads.iter().any(|workload| {
            workload.workload_id.is_empty()
                || workload.audiences.is_empty()
                || workload.contract_actions.is_empty()
        })
        || configuration.actors.iter().any(|actor| {
            actor.actor_id.is_empty()
                || actor.roles.is_empty()
                || actor.applications.is_empty()
                || actor.purposes.is_empty()
                || actor.synthetic_realm.is_empty()
        })
    {
        return Err(IamError::ConfigurationInvalid);
    }
    Ok(())
}

pub(crate) fn validate_actor_request(
    actor: &SyntheticActorRegistration,
    request: &GrantRequest,
) -> Result<(), IamError> {
    if request.roles.is_empty()
        || !request.roles.iter().all(|role| actor.roles.contains(role))
        || !actor.applications.contains(&request.application_id)
        || !actor.purposes.contains(&request.purpose)
        || actor.synthetic_realm != request.synthetic_realm
        || request.audience.is_empty()
        || request.surface_id.is_empty()
        || request.demonstration_session_id.is_empty()
    {
        return Err(IamError::RequestRefused("actor-scope-refused"));
    }
    Ok(())
}

pub(crate) fn required_obligations(
    configuration: &DemonstrationConfiguration,
    realm: &str,
) -> Result<Vec<AuthorisationObligation>, IamError> {
    if !configuration
        .supported_obligations
        .iter()
        .any(|value| value == "mark-synthetic")
        || !configuration
            .supported_obligations
            .iter()
            .any(|value| value == "restrict-realm")
    {
        return Err(IamError::ConfigurationInvalid);
    }
    Ok(vec![
        AuthorisationObligation {
            code: "mark-synthetic".to_owned(),
            value: None,
        },
        AuthorisationObligation {
            code: "restrict-realm".to_owned(),
            value: Some(realm.to_owned()),
        },
    ])
}

pub(crate) fn enforce_obligations(
    obligations: &[AuthorisationObligation],
    realm: &str,
) -> Result<(), IamError> {
    let marked = obligations
        .iter()
        .any(|obligation| obligation.code == "mark-synthetic");
    let restricted = obligations.iter().any(|obligation| {
        obligation.code == "restrict-realm" && obligation.value.as_deref() == Some(realm)
    });
    if marked && restricted {
        Ok(())
    } else {
        Err(IamError::RequestRefused("required-obligation-unmet"))
    }
}

fn verify_grant(
    grant: &DemonstrationSignInGrant,
    environment: &EnvironmentState,
    now: OffsetDateTime,
) -> Result<(), &'static str> {
    if grant.contract_id != "I-004"
        || grant.contract_version != I004_VERSION
        || grant.signature.profile != SIGNATURE_PROFILE
        || grant.signature.algorithm != "Ed25519"
        || grant.signature.signer_id != environment.record.signer_id
        || grant.signature.public_key_fingerprint != environment.record.public_key_fingerprint
    {
        return Err("grant-signature-profile-invalid");
    }
    if grant.claims.environment_id != environment.record.environment_id
        || grant.claims.trust_domain != environment.record.trust_domain
        || grant.claims.trust_epoch != environment.record.trust_epoch
    {
        return Err("grant-environment-mismatch");
    }
    let public_bytes: [u8; 32] = decode_hex(&environment.public_key)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or("grant-public-key-invalid")?;
    let verifying_key =
        VerifyingKey::from_bytes(&public_bytes).map_err(|_| "grant-public-key-invalid")?;
    let signature_bytes = decode_hex(&grant.signature.value).ok_or("grant-signature-invalid")?;
    let signature =
        Signature::try_from(signature_bytes.as_slice()).map_err(|_| "grant-signature-invalid")?;
    verifying_key
        .verify_strict(&canonical_claims(&grant.claims), &signature)
        .map_err(|_| "grant-signature-invalid")?;
    let not_before = parse_time(&grant.claims.not_before).map_err(|_| "grant-time-invalid")?;
    let expires_at = parse_time(&grant.claims.expires_at).map_err(|_| "grant-time-invalid")?;
    let issued_at = parse_time(&grant.claims.issued_at).map_err(|_| "grant-time-invalid")?;
    if issued_at > not_before || now < not_before || now >= expires_at || expires_at <= issued_at {
        return Err("grant-expired-or-premature");
    }
    Ok(())
}

pub(crate) fn canonical_claims(claims: &DemonstrationSignInGrantClaims) -> Vec<u8> {
    let mut roles = claims.roles.clone();
    roles.sort();
    let fields = [
        SIGNATURE_PROFILE.to_owned(),
        I004_VERSION.to_owned(),
        claims.grant_id.clone(),
        claims.establishment_operation_id.clone(),
        claims.environment_id.clone(),
        claims.trust_domain.clone(),
        claims.trust_epoch.to_string(),
        claims.actor_id.clone(),
        claims.application_id.clone(),
        claims.audience.clone(),
        claims.surface_id.clone(),
        claims.demonstration_session_id.clone(),
        roles.join("\u{001f}"),
        claims.purpose.clone(),
        claims.synthetic_realm.clone(),
        claims.decision_reference.clone(),
        claims.issued_at.clone(),
        claims.not_before.clone(),
        claims.expires_at.clone(),
    ];
    let mut encoded = Vec::new();
    for field in fields {
        let length = u32::try_from(field.len()).unwrap_or(u32::MAX);
        encoded.extend_from_slice(&length.to_be_bytes());
        encoded.extend_from_slice(field.as_bytes());
    }
    encoded
}

fn established_outcome(
    grant: &DemonstrationSignInGrant,
    outcome_id: String,
    session_reference: String,
    maximum_valid_until: String,
    now: OffsetDateTime,
    original_outcome_id: Option<String>,
) -> Result<SyntheticSessionOutcome, IamError> {
    Ok(SyntheticSessionOutcome {
        contract_id: "I-005".to_owned(),
        contract_version: I005_VERSION.to_owned(),
        outcome_id,
        grant_id: grant.claims.grant_id.clone(),
        establishment_operation_id: grant.claims.establishment_operation_id.clone(),
        environment_id: grant.claims.environment_id.clone(),
        application_id: grant.claims.application_id.clone(),
        surface_id: grant.claims.surface_id.clone(),
        demonstration_session_id: grant.claims.demonstration_session_id.clone(),
        actor_id: grant.claims.actor_id.clone(),
        roles: grant.claims.roles.clone(),
        synthetic_realm: grant.claims.synthetic_realm.clone(),
        status: SyntheticSessionStatus::Established,
        occurred_at: format_time(now)?,
        maximum_valid_until: Some(maximum_valid_until),
        session_reference: Some(session_reference),
        reason_code: None,
        decision_reference: grant.claims.decision_reference.clone(),
        original_outcome_id,
        evidence_references: vec![format!(
            "evidence-session-{}",
            digest_prefix(&grant.claims.establishment_operation_id)
        )],
    })
}

fn refused_outcome(
    grant: &DemonstrationSignInGrant,
    reason: &'static str,
    now: OffsetDateTime,
) -> Result<SyntheticSessionOutcome, IamError> {
    let grant_ref = digest_prefix(&grant.claims.grant_id);
    let operation_ref = digest_prefix(&grant.claims.establishment_operation_id);
    Ok(SyntheticSessionOutcome {
        contract_id: "I-005".to_owned(),
        contract_version: I005_VERSION.to_owned(),
        outcome_id: random_id("session-outcome", 12)?,
        grant_id: format!("grant-ref-{grant_ref}"),
        establishment_operation_id: format!("operation-ref-{operation_ref}"),
        environment_id: format!(
            "environment-ref-{}",
            digest_prefix(&grant.claims.environment_id)
        ),
        application_id: format!(
            "application-ref-{}",
            digest_prefix(&grant.claims.application_id)
        ),
        surface_id: format!("surface-ref-{}", digest_prefix(&grant.claims.surface_id)),
        demonstration_session_id: format!(
            "demonstration-ref-{}",
            digest_prefix(&grant.claims.demonstration_session_id)
        ),
        actor_id: format!("actor-ref-{}", digest_prefix(&grant.claims.actor_id)),
        roles: vec!["refused".to_owned()],
        synthetic_realm: format!("realm-ref-{}", digest_prefix(&grant.claims.synthetic_realm)),
        status: if reason == "grant-expired-or-premature" {
            SyntheticSessionStatus::Expired
        } else {
            SyntheticSessionStatus::Refused
        },
        occurred_at: format_time(now)?,
        maximum_valid_until: None,
        session_reference: None,
        reason_code: Some(reason.to_owned()),
        decision_reference: format!(
            "decision-ref-{}",
            digest_prefix(&grant.claims.decision_reference)
        ),
        original_outcome_id: None,
        evidence_references: vec![format!("evidence-refusal-{operation_ref}")],
    })
}

type EstablishedRecord<'a> = (
    &'a String,
    &'a String,
    &'a String,
    &'a String,
    &'a String,
    &'a Vec<String>,
    &'a String,
    &'a String,
);

#[allow(clippy::too_many_arguments)]
fn terminal_outcome(
    environment_id: &str,
    session_reference: &str,
    established: EstablishedRecord<'_>,
    outcome_id: String,
    status: SyntheticSessionStatus,
    reason_code: String,
    occurred_at: String,
) -> SyntheticSessionOutcome {
    SyntheticSessionOutcome {
        contract_id: "I-005".to_owned(),
        contract_version: I005_VERSION.to_owned(),
        outcome_id,
        grant_id: format!("grant-ref-{}", digest_prefix(established.0)),
        establishment_operation_id: format!("operation-ref-{}", digest_prefix(established.0)),
        environment_id: environment_id.to_owned(),
        application_id: established.2.clone(),
        surface_id: established.3.clone(),
        demonstration_session_id: established.4.clone(),
        actor_id: established.1.clone(),
        roles: established.5.clone(),
        synthetic_realm: established.6.clone(),
        status,
        occurred_at,
        maximum_valid_until: None,
        session_reference: Some(session_reference.to_owned()),
        reason_code: Some(reason_code),
        decision_reference: established.7.clone(),
        original_outcome_id: Some(established.0.clone()),
        evidence_references: vec![format!(
            "evidence-session-terminal-{}",
            digest_prefix(session_reference)
        )],
    }
}

fn profile_compatible(
    record: &SyntheticTrustBootstrapRecord,
    declared: DeclaredEnvironment,
) -> bool {
    record.status == TrustStatus::Ready
        && record.compatible
        && record.trust_profile == required_profile(declared)
        && match declared.environment_class {
            EnvironmentClass::LocalScratch | EnvironmentClass::PortableIsolated => true,
            EnvironmentClass::HostedShared
            | EnvironmentClass::ProductionLike
            | EnvironmentClass::Production => record.trust_profile == TrustProfile::Managed,
        }
        && (declared.information_profile == InformationProfile::SyntheticOnly
            || record.trust_profile == TrustProfile::Managed)
}

const fn required_profile(declared: DeclaredEnvironment) -> TrustProfile {
    match (
        declared.environment_class,
        declared.information_profile,
        declared.required_trust_profile,
    ) {
        (
            EnvironmentClass::HostedShared
            | EnvironmentClass::ProductionLike
            | EnvironmentClass::Production,
            _,
            _,
        )
        | (_, InformationProfile::NonSyntheticAuthorised, _)
        | (_, _, TrustProfile::Managed) => TrustProfile::Managed,
        _ => TrustProfile::LocalSynthetic,
    }
}

fn not_ready_report(declared: DeclaredEnvironment, reason: &'static str) -> IamHealthReport {
    IamHealthReport {
        software_status: "healthy",
        identity_state: IdentityState::NotReady,
        maturity: "in-development",
        environment_id: None,
        environment_class: declared.environment_class,
        information_profile: declared.information_profile,
        active_trust_profile: None,
        required_trust_profile: required_profile(declared),
        trust_domain: None,
        trust_epoch: None,
        signer_fingerprint: None,
        key_custody_class: None,
        recovery_profile: None,
        prominent_warning: None,
        reason_code: Some(reason),
    }
}

const fn error_reason(error: &IamError) -> &'static str {
    match error {
        IamError::StateUnavailable => "identity-state-unavailable",
        IamError::StateInconsistent => "identity-state-inconsistent",
        IamError::ConfigurationInvalid => "identity-configuration-invalid",
        IamError::NotReady => "identity-not-ready",
        IamError::RequestRefused(_) => "identity-request-refused",
        IamError::AuthorisationRefused(_) => "authorisation-refused",
        IamError::RandomUnavailable => "random-source-unavailable",
        IamError::CryptographyFailure => "cryptography-failure",
        IamError::Serialization => "identity-serialization-failure",
    }
}

fn parse_time(value: &str) -> Result<OffsetDateTime, time::error::Parse> {
    OffsetDateTime::parse(value, &Rfc3339)
}

pub(crate) fn format_time(value: OffsetDateTime) -> Result<String, IamError> {
    value.format(&Rfc3339).map_err(|_| IamError::Serialization)
}

pub(crate) fn random_id(prefix: &str, length: usize) -> Result<String, IamError> {
    let mut bytes = vec![0_u8; length];
    getrandom::fill(&mut bytes).map_err(|_| IamError::RandomUnavailable)?;
    Ok(format!("{prefix}-{}", hex(&bytes)))
}

fn grant_digest(grant: &DemonstrationSignInGrant) -> Result<String, IamError> {
    let bytes = serde_json::to_vec(grant).map_err(|_| IamError::Serialization)?;
    Ok(sha256_hex(&bytes))
}

fn digest_prefix(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex(&digest[..8])
}

pub(crate) fn sha256_hex(value: &[u8]) -> String {
    hex(&Sha256::digest(value))
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    pairs
        .iter()
        .map(|pair| {
            let high = hex_value(pair[0])?;
            let low = hex_value(pair[1])?;
            Some((high << 4) | low)
        })
        .collect()
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

fn valid_reason_code(value: &str) -> bool {
    (3..=96).contains(&value.len())
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase() || (index > 0 && (byte.is_ascii_digit() || byte == b'-'))
        })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, IamError> {
    let file = open_restricted(path, false, false)?;
    serde_json::from_reader(file).map_err(|_| IamError::StateInconsistent)
}

fn write_new_json<T: Serialize>(path: &Path, value: &T) -> Result<(), IamError> {
    let bytes = serde_json::to_vec(value).map_err(|_| IamError::Serialization)?;
    let mut file = open_restricted(path, true, true)?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| IamError::StateUnavailable)
}

fn write_private_key(path: &Path, secret: &[u8; 32]) -> Result<(), IamError> {
    let mut file = open_restricted(path, true, true)?;
    file.write_all(secret)
        .and_then(|()| file.sync_all())
        .map_err(|_| IamError::StateUnavailable)
}

fn open_restricted(path: &Path, write: bool, create_new: bool) -> Result<File, IamError> {
    let mut options = OpenOptions::new();
    options.read(true).write(write);
    if create_new {
        options.create_new(true);
    } else if write {
        options.create(true).append(true);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path).map_err(|_| IamError::StateUnavailable)?;
    set_file_permissions(path, 0o600)?;
    Ok(file)
}

#[cfg(unix)]
fn restrict_directory(path: &Path) -> Result<(), IamError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| IamError::StateUnavailable)
}

#[cfg(not(unix))]
fn restrict_directory(_path: &Path) -> Result<(), IamError> {
    Ok(())
}

#[cfg(unix)]
fn set_file_permissions(path: &Path, mode: u32) -> Result<(), IamError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|_| IamError::StateUnavailable)
}

#[cfg(not(unix))]
fn set_file_permissions(_path: &Path, _mode: u32) -> Result<(), IamError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use ppl_contracts::{
        AssertionStatus, AssertionType, AuthorisationDecisionStatus, AuthoritativeAssertion,
        EnvironmentClass, InformationProfile, SyntheticSessionStatus, TrustProfile,
    };
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::{
        DeclaredEnvironment, DemonstrationConfiguration, GrantRequest, IamError, IamRuntime,
        IdentityState, SyntheticActorRegistration, WorkloadRegistration, descriptor,
        enforce_obligations,
    };

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "public-purpose-lab-iam-test-{}",
                super::random_id("run", 8).expect("random test path")
            ));
            fs::create_dir(&path).expect("create test directory");
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove owned test directory");
        }
    }

    fn at(value: &str) -> OffsetDateTime {
        OffsetDateTime::parse(value, &Rfc3339).expect("test timestamp")
    }

    fn configuration(environment_id: &str) -> DemonstrationConfiguration {
        let assertion =
            |assertion_type, source: &str, actor: &str, app: &str| AuthoritativeAssertion {
                source_id: source.to_owned(),
                assertion_type,
                subject_id: actor.to_owned(),
                resource_id: app.to_owned(),
                purpose_codes: vec!["demonstrate-discovery".to_owned()],
                status: AssertionStatus::Active,
                effective_at: "2030-08-26T09:00:00Z".to_owned(),
                expires_at: "2030-08-26T11:00:00Z".to_owned(),
                version: "1.0.0".to_owned(),
            };
        DemonstrationConfiguration {
            environment_id: environment_id.to_owned(),
            policy_version: "1.0.0".to_owned(),
            relationship_source: "source-relationships".to_owned(),
            consent_source: "source-consents".to_owned(),
            supported_obligations: vec!["mark-synthetic".to_owned(), "restrict-realm".to_owned()],
            workloads: vec![WorkloadRegistration {
                workload_id: "workload-director".to_owned(),
                audiences: vec!["iam-01-service".to_owned()],
                contract_actions: vec!["I-004:request-grant".to_owned()],
                enabled: true,
            }],
            actors: vec![
                SyntheticActorRegistration {
                    actor_id: "synthetic-reviewer".to_owned(),
                    display_name: "Alex Reviewer".to_owned(),
                    roles: vec!["reviewer".to_owned()],
                    applications: vec!["workbench-app".to_owned()],
                    purposes: vec!["demonstrate-discovery".to_owned()],
                    synthetic_realm: "realm-synthetic-demo".to_owned(),
                    enabled: true,
                },
                SyntheticActorRegistration {
                    actor_id: "synthetic-coordinator".to_owned(),
                    display_name: "Sam Coordinator".to_owned(),
                    roles: vec!["coordinator".to_owned()],
                    applications: vec!["workflow-app".to_owned()],
                    purposes: vec!["demonstrate-discovery".to_owned()],
                    synthetic_realm: "realm-synthetic-demo".to_owned(),
                    enabled: true,
                },
            ],
            assertions: vec![
                assertion(
                    AssertionType::Relationship,
                    "source-relationships",
                    "synthetic-reviewer",
                    "workbench-app",
                ),
                assertion(
                    AssertionType::Consent,
                    "source-consents",
                    "synthetic-reviewer",
                    "workbench-app",
                ),
                assertion(
                    AssertionType::Relationship,
                    "source-relationships",
                    "synthetic-coordinator",
                    "workflow-app",
                ),
                assertion(
                    AssertionType::Consent,
                    "source-consents",
                    "synthetic-coordinator",
                    "workflow-app",
                ),
            ],
        }
    }

    fn request(actor: &str, role: &str, application: &str, surface: &str) -> GrantRequest {
        GrantRequest {
            workload_id: "workload-director".to_owned(),
            actor_id: actor.to_owned(),
            application_id: application.to_owned(),
            audience: format!("{application}-backend"),
            surface_id: surface.to_owned(),
            demonstration_session_id: "demonstration-session-001".to_owned(),
            roles: vec![role.to_owned()],
            purpose: "demonstrate-discovery".to_owned(),
            synthetic_realm: "realm-synthetic-demo".to_owned(),
        }
    }

    fn configured_runtime(directory: &TestDirectory) -> IamRuntime {
        let runtime = IamRuntime::new(&directory.0);
        let record = runtime
            .bootstrap_local(at("2030-08-26T10:00:00Z"))
            .expect("bootstrap local trust");
        runtime
            .configure(
                &configuration(&record.environment_id),
                at("2030-08-26T10:00:00Z"),
            )
            .expect("configure demonstration");
        runtime
    }

    #[test]
    fn descriptor_names_implemented_m2_contracts() {
        assert_eq!(
            descriptor().contracts,
            &["I-001", "I-002", "I-003", "I-004", "I-005", "AZ-001"]
        );
        assert_eq!(descriptor().maturity.to_string(), "in-development");
    }

    #[test]
    fn independent_environments_have_unrelated_roots_and_refuse_each_other() {
        let first_dir = TestDirectory::new();
        let second_dir = TestDirectory::new();
        let first = configured_runtime(&first_dir);
        let second = configured_runtime(&second_dir);
        let first_health = first.health(DeclaredEnvironment::local_synthetic());
        let second_health = second.health(DeclaredEnvironment::local_synthetic());
        assert_ne!(first_health.environment_id, second_health.environment_id);
        assert_ne!(first_health.trust_domain, second_health.trust_domain);
        assert_ne!(
            first_health.signer_fingerprint,
            second_health.signer_fingerprint
        );

        let grant = first
            .issue_grant(
                &request(
                    "synthetic-reviewer",
                    "reviewer",
                    "workbench-app",
                    "surface-workbench-001",
                ),
                at("2030-08-26T10:00:10Z"),
            )
            .expect("issue first environment grant");
        let outcome = second
            .establish_session(&grant, at("2030-08-26T10:00:20Z"))
            .expect("safe refusal outcome");
        assert_eq!(outcome.status, SyntheticSessionStatus::Refused);
        assert_eq!(
            outcome.reason_code.as_deref(),
            Some("grant-signature-profile-invalid")
        );
    }

    #[test]
    fn local_profile_is_visible_and_cannot_ready_a_hosted_environment() {
        let directory = TestDirectory::new();
        let runtime = configured_runtime(&directory);
        let local = runtime.health(DeclaredEnvironment::local_synthetic());
        assert_eq!(local.identity_state, IdentityState::Ready);
        assert_eq!(
            local.active_trust_profile,
            Some(TrustProfile::LocalSynthetic)
        );
        assert!(local.prominent_warning.is_some());

        let hosted = runtime.health(DeclaredEnvironment {
            environment_class: EnvironmentClass::HostedShared,
            information_profile: InformationProfile::SyntheticOnly,
            required_trust_profile: TrustProfile::Managed,
        });
        assert_eq!(hosted.identity_state, IdentityState::NotReady);
        assert_eq!(hosted.required_trust_profile, TrustProfile::Managed);
        assert_eq!(hosted.reason_code, Some("trust-profile-incompatible"));
    }

    #[test]
    fn hosted_declaration_cannot_bootstrap_or_use_local_trust() {
        let hosted = DeclaredEnvironment {
            environment_class: EnvironmentClass::HostedShared,
            information_profile: InformationProfile::SyntheticOnly,
            required_trust_profile: TrustProfile::Managed,
        };
        let empty_directory = TestDirectory::new();
        let hosted_runtime = IamRuntime::with_declared_environment(&empty_directory.0, hosted);
        assert_eq!(
            hosted_runtime.bootstrap_local(at("2030-08-26T10:00:00Z")),
            Err(IamError::NotReady)
        );

        let configured_directory = TestDirectory::new();
        let local_runtime = configured_runtime(&configured_directory);
        let grant = local_runtime
            .issue_grant(
                &request(
                    "synthetic-reviewer",
                    "reviewer",
                    "workbench-app",
                    "surface-workbench-001",
                ),
                at("2030-08-26T10:00:10Z"),
            )
            .expect("local grant");
        let hosted_runtime = IamRuntime::with_declared_environment(&configured_directory.0, hosted);
        assert_eq!(
            hosted_runtime.issue_grant(
                &request(
                    "synthetic-reviewer",
                    "reviewer",
                    "workbench-app",
                    "surface-workbench-001",
                ),
                at("2030-08-26T10:00:20Z"),
            ),
            Err(IamError::NotReady)
        );
        let outcome = hosted_runtime
            .establish_session(&grant, at("2030-08-26T10:00:20Z"))
            .expect("safe refusal");
        assert_eq!(outcome.status, SyntheticSessionStatus::Refused);
        assert_eq!(
            outcome.reason_code.as_deref(),
            Some("trust-profile-incompatible")
        );
    }

    #[test]
    fn duplicate_and_restart_reconcile_one_session() {
        let directory = TestDirectory::new();
        let runtime = configured_runtime(&directory);
        let grant = runtime
            .issue_grant(
                &request(
                    "synthetic-reviewer",
                    "reviewer",
                    "workbench-app",
                    "surface-workbench-001",
                ),
                at("2030-08-26T10:00:10Z"),
            )
            .expect("issue grant");
        let first = runtime
            .establish_session(&grant, at("2030-08-26T10:00:20Z"))
            .expect("establish session");
        let duplicate = IamRuntime::new(&directory.0)
            .establish_session(&grant, at("2030-08-26T10:00:30Z"))
            .expect("reconcile after restart");
        assert_eq!(first.session_reference, duplicate.session_reference);
        assert_eq!(duplicate.original_outcome_id, Some(first.outcome_id));
    }

    #[test]
    fn one_scenario_supports_distinct_actors_and_applications() {
        let directory = TestDirectory::new();
        let runtime = configured_runtime(&directory);
        let reviewer = runtime
            .issue_grant(
                &request(
                    "synthetic-reviewer",
                    "reviewer",
                    "workbench-app",
                    "surface-workbench-001",
                ),
                at("2030-08-26T10:00:10Z"),
            )
            .expect("reviewer grant");
        let coordinator = runtime
            .issue_grant(
                &request(
                    "synthetic-coordinator",
                    "coordinator",
                    "workflow-app",
                    "surface-workflow-001",
                ),
                at("2030-08-26T10:00:11Z"),
            )
            .expect("coordinator grant");
        let first = runtime
            .establish_session(&reviewer, at("2030-08-26T10:00:20Z"))
            .expect("reviewer session");
        let second = runtime
            .establish_session(&coordinator, at("2030-08-26T10:00:21Z"))
            .expect("coordinator session");
        assert_ne!(first.actor_id, second.actor_id);
        assert_ne!(first.application_id, second.application_id);
        assert_ne!(first.session_reference, second.session_reference);
    }

    #[test]
    fn workload_and_synthetic_actor_cannot_substitute() {
        let directory = TestDirectory::new();
        let runtime = configured_runtime(&directory);
        let mut invalid = request(
            "synthetic-reviewer",
            "reviewer",
            "workbench-app",
            "surface-workbench-001",
        );
        invalid.workload_id = "synthetic-reviewer".to_owned();
        assert_eq!(
            runtime.issue_grant(&invalid, at("2030-08-26T10:00:10Z")),
            Err(IamError::RequestRefused("workload-not-authorised"))
        );
    }

    #[test]
    fn required_obligations_cannot_be_discarded() {
        assert_eq!(
            enforce_obligations(&[], "realm-synthetic-demo"),
            Err(IamError::RequestRefused("required-obligation-unmet"))
        );
    }

    #[test]
    fn stale_relationship_fails_closed() {
        let directory = TestDirectory::new();
        let runtime = IamRuntime::new(&directory.0);
        let record = runtime
            .bootstrap_local(at("2030-08-26T10:00:00Z"))
            .expect("bootstrap");
        let mut config = configuration(&record.environment_id);
        config.assertions[0].expires_at = "2030-08-26T09:30:00Z".to_owned();
        runtime
            .configure(&config, at("2030-08-26T10:00:00Z"))
            .expect("configure stale test");
        assert_eq!(
            runtime.issue_grant(
                &request(
                    "synthetic-reviewer",
                    "reviewer",
                    "workbench-app",
                    "surface-workbench-001"
                ),
                at("2030-08-26T10:00:10Z"),
            ),
            Err(IamError::AuthorisationRefused(
                AuthorisationDecisionStatus::Indeterminate
            ))
        );
    }

    #[test]
    fn termination_revocation_and_rebuild_are_monotonic() {
        let directory = TestDirectory::new();
        let runtime = configured_runtime(&directory);
        let original_trust = runtime
            .health(DeclaredEnvironment::local_synthetic())
            .trust_domain;
        let grant = runtime
            .issue_grant(
                &request(
                    "synthetic-reviewer",
                    "reviewer",
                    "workbench-app",
                    "surface-workbench-001",
                ),
                at("2030-08-26T10:00:10Z"),
            )
            .expect("issue grant");
        let reference = runtime
            .establish_session(&grant, at("2030-08-26T10:00:20Z"))
            .expect("establish")
            .session_reference
            .expect("safe reference");
        let terminated = runtime
            .terminate_session(&reference, "scenario-stopped", at("2030-08-26T10:01:00Z"))
            .expect("terminate");
        let duplicate = runtime
            .terminate_session(&reference, "scenario-stopped", at("2030-08-26T10:01:10Z"))
            .expect("duplicate terminate");
        assert_eq!(terminated.outcome_id, duplicate.outcome_id);
        runtime
            .revoke_trust("operator-security-response", at("2030-08-26T10:02:00Z"))
            .expect("revoke");
        assert_eq!(
            runtime
                .health(DeclaredEnvironment::local_synthetic())
                .identity_state,
            IdentityState::NotReady
        );

        let rebuilt_dir = TestDirectory::new();
        let rebuilt = configured_runtime(&rebuilt_dir);
        assert_ne!(
            original_trust,
            rebuilt
                .health(DeclaredEnvironment::local_synthetic())
                .trust_domain
        );
    }

    #[test]
    fn corrupt_state_fails_closed() {
        let directory = TestDirectory::new();
        let runtime = configured_runtime(&directory);
        fs::write(
            directory.0.join("iam-01/iam-security-journal.jsonl"),
            b"not-json\n",
        )
        .expect("corrupt owned test journal");
        assert_eq!(
            runtime
                .health(DeclaredEnvironment::local_synthetic())
                .identity_state,
            IdentityState::NotReady
        );
    }

    #[cfg(unix)]
    #[test]
    fn local_private_key_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let directory = TestDirectory::new();
        configured_runtime(&directory);
        let mode = fs::metadata(directory.0.join("iam-01/local-signing-key.bin"))
            .expect("private key metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
}
