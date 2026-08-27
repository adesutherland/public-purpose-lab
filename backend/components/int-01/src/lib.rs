//! M1 reference implementation of the INT-01 interaction boundary.
//!
//! The implementation is deliberately local and transport-neutral. It proves
//! contract checks, idempotency, restart reconciliation, safe outcomes and a
//! privacy-minimised append journal. It does not authenticate an envelope and
//! must not be exposed as an external listener.

use std::{
    fmt,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use ppl_contracts::{
    AuthorityContext, C003_VERSION, C004_VERSION, CommandOutcome, ComponentCapabilityManifest,
    ComponentMaturity, DeploymentProfile, EvidenceKind, EvidenceReference,
    InformationClassification, InformationLevel, InteractionEnvelope, MessageKind, OutcomeStatus,
    PrincipalType, SemanticCapability, SupportedContract,
};
use ppl_core::{ComponentDescriptor, Maturity};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

pub mod nats;

const INTERACTION_CONTRACTS: &[&str] = &["C-001", "C-002", "C-003", "C-004", "C-005", "C-006"];
const JOURNAL_VERSION: u32 = 1;
const JOURNAL_FILE: &str = "interaction-journal.jsonl";
const LOCK_FILE: &str = "interaction-journal.lock";
const EXPECTED_MESSAGE_TYPE: &str = "ppl.interaction.conformance-probe.command";
const CLOCK_TOLERANCE: Duration = Duration::minutes(5);

#[must_use]
pub const fn descriptor() -> ComponentDescriptor {
    ComponentDescriptor {
        id: "INT-01",
        name: "Interaction Infrastructure and Contract Registry",
        maturity: Maturity::InDevelopment,
        contracts: INTERACTION_CONTRACTS,
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub state_dir: PathBuf,
    pub environment_id: String,
}

#[derive(Clone, Debug)]
pub struct InteractionRuntime {
    config: RuntimeConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    StateUnavailable,
    StateCorrupt,
    Serialization,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::StateUnavailable => "interaction state is unavailable",
            Self::StateCorrupt => "interaction state is inconsistent",
            Self::Serialization => "interaction data could not be represented safely",
        })
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InteractionState {
    Ready,
    NotReady,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub software_status: &'static str,
    pub interaction_state: InteractionState,
    pub maturity: &'static str,
    pub environment_id: String,
    pub journal_records: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<&'static str>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct JournalRecord {
    journal_version: u32,
    environment_id: String,
    recorded_at: String,
    message_id: String,
    correlation_id: String,
    contract_id: String,
    contract_version: String,
    idempotency_digest: String,
    command_fingerprint: String,
    principal_type: PrincipalType,
    principal_digest: String,
    decision_code: String,
    operation_applied: bool,
    outcome: CommandOutcome,
}

#[derive(Clone, Debug)]
struct Decision {
    status: OutcomeStatus,
    code: &'static str,
    summary: &'static str,
    retryable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticOperation<'a> {
    contract_id: &'a str,
    contract_version: &'a str,
    message_type: &'a str,
    message_kind: &'a MessageKind,
    source_component: &'a str,
    target_component: &'a str,
    audience: &'a str,
    issued_at: &'a str,
    expires_at: &'a Option<String>,
    authority: &'a AuthorityContext,
    classification: &'a InformationClassification,
    payload: &'a Value,
}

impl InteractionRuntime {
    #[must_use]
    pub fn new(config: RuntimeConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub fn capability_manifest(&self, generated_at: &str) -> ComponentCapabilityManifest {
        ComponentCapabilityManifest {
            contract_id: "C-005".to_owned(),
            contract_version: "1.0.0".to_owned(),
            component_id: "INT-01".to_owned(),
            component_name: "Interaction infrastructure and contract registry".to_owned(),
            release_version: env!("CARGO_PKG_VERSION").to_owned(),
            maturity: ComponentMaturity::InDevelopment,
            generated_at: generated_at.to_owned(),
            supported_profiles: vec![DeploymentProfile::DevelopmentAssurance],
            capabilities: vec![SemanticCapability {
                capability_id: "interaction.conformance-probe".to_owned(),
                description: "Validate and durably reconcile one local M1 assurance command"
                    .to_owned(),
                contracts: vec![
                    supported_contract("C-001"),
                    supported_contract("C-003"),
                    supported_contract("C-004"),
                ],
            }],
            readiness_dependencies: vec![
                "interaction-state.writable".to_owned(),
                "interaction-state.consistent".to_owned(),
            ],
            evidence: Vec::new(),
        }
    }

    #[must_use]
    pub fn health(&self) -> HealthReport {
        match self.with_locked_records() {
            Ok(records) => HealthReport {
                software_status: "healthy",
                interaction_state: InteractionState::Ready,
                maturity: "in-development",
                environment_id: self.config.environment_id.clone(),
                journal_records: records.len(),
                reason_code: None,
            },
            Err(RuntimeError::StateCorrupt) => HealthReport {
                software_status: "healthy",
                interaction_state: InteractionState::NotReady,
                maturity: "in-development",
                environment_id: self.config.environment_id.clone(),
                journal_records: 0,
                reason_code: Some("interaction_state_inconsistent"),
            },
            Err(_) => HealthReport {
                software_status: "healthy",
                interaction_state: InteractionState::NotReady,
                maturity: "in-development",
                environment_id: self.config.environment_id.clone(),
                journal_records: 0,
                reason_code: Some("interaction_state_unavailable"),
            },
        }
    }

    /// Processes and durably reconciles one local assurance envelope.
    ///
    /// # Errors
    ///
    /// Returns a safe runtime error when delivery state is unavailable,
    /// inconsistent or cannot be represented without accepting the operation.
    pub fn process(
        &self,
        envelope: &InteractionEnvelope,
        now: OffsetDateTime,
    ) -> Result<CommandOutcome, RuntimeError> {
        self.prepare_state()?;
        let lock = self.open_lock()?;
        lock.lock().map_err(|_| RuntimeError::StateUnavailable)?;

        let result = self.process_locked(envelope, now);
        let unlock_result = lock.unlock().map_err(|_| RuntimeError::StateUnavailable);
        match (result, unlock_result) {
            (Ok(outcome), Ok(())) => Ok(outcome),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    fn process_locked(
        &self,
        envelope: &InteractionEnvelope,
        now: OffsetDateTime,
    ) -> Result<CommandOutcome, RuntimeError> {
        let records = self.load_records()?;
        let fingerprint = typed_fingerprint(envelope)?;
        let idempotency_scope = format!(
            "{}|{}|{}|{}",
            envelope.target_component,
            envelope.contract_id,
            envelope.contract_version,
            envelope
                .idempotency_key
                .as_deref()
                .unwrap_or(&envelope.message_id)
        );
        let idempotency_digest = sha256_hex(idempotency_scope.as_bytes());
        let sequence = records.len() + 1;
        let completed_at = format_time(now)?;

        let (decision, original_outcome_id) = if let Some(previous) =
            records.iter().rev().find(|record| {
                record.environment_id == self.config.environment_id
                    && record.idempotency_digest == idempotency_digest
            }) {
            if previous.command_fingerprint == fingerprint {
                if previous.outcome.retryable
                    && matches!(
                        previous.outcome.status,
                        OutcomeStatus::Refused | OutcomeStatus::Failed
                    )
                {
                    (self.validate(envelope, now), None)
                } else {
                    (
                        Decision {
                            status: OutcomeStatus::Duplicate,
                            code: "duplicate_delivery",
                            summary: "The idempotency operation was already decided; no second operation was applied.",
                            retryable: false,
                        },
                        Some(previous.outcome.outcome_id.clone()),
                    )
                }
            } else {
                (
                    Decision {
                        status: OutcomeStatus::Refused,
                        code: "idempotency_conflict",
                        summary: "The idempotency key was previously bound to different semantic content.",
                        retryable: false,
                    },
                    None,
                )
            }
        } else {
            (self.validate(envelope, now), None)
        };

        let outcome_id = format!("outcome-{}-{sequence:08}", &fingerprint[..16]);
        let evidence = journal_evidence(&outcome_id, &completed_at);
        let outcome = CommandOutcome {
            contract_id: "C-003".to_owned(),
            contract_version: C003_VERSION.to_owned(),
            outcome_id,
            command_message_id: envelope.message_id.clone(),
            status: decision.status.clone(),
            code: decision.code.to_owned(),
            summary: decision.summary.to_owned(),
            retryable: decision.retryable,
            completed_at: completed_at.clone(),
            original_outcome_id,
            recovery_owner: None,
            evidence: vec![evidence],
        };

        let record = JournalRecord {
            journal_version: JOURNAL_VERSION,
            environment_id: self.config.environment_id.clone(),
            recorded_at: completed_at,
            message_id: envelope.message_id.clone(),
            correlation_id: envelope.correlation_id.clone(),
            contract_id: envelope.contract_id.clone(),
            contract_version: envelope.contract_version.clone(),
            idempotency_digest,
            command_fingerprint: fingerprint,
            principal_type: envelope.authority.requester.principal_type.clone(),
            principal_digest: sha256_hex(
                format!(
                    "{}|{}|{}",
                    envelope.authority.requester.environment_id,
                    envelope.authority.requester.issuer,
                    envelope.authority.requester.principal_id
                )
                .as_bytes(),
            ),
            decision_code: decision.code.to_owned(),
            operation_applied: decision.status == OutcomeStatus::Accepted,
            outcome: outcome.clone(),
        };
        self.append_record(&record)?;

        Ok(outcome)
    }

    // Keeping the ordered refusal rules together makes the fail-closed decision
    // path reviewable as one unit; extraction must preserve this exact order.
    #[allow(clippy::too_many_lines)]
    fn validate(&self, envelope: &InteractionEnvelope, now: OffsetDateTime) -> Decision {
        if envelope.contract_id != "C-001" || envelope.contract_version != "1.0.0" {
            return refused(
                "contract_version_unsupported",
                "The interaction contract identifier or version is not supported.",
            );
        }
        if envelope.message_kind != MessageKind::DemonstrationControl
            || envelope.message_type != EXPECTED_MESSAGE_TYPE
        {
            return refused(
                "capability_unsupported",
                "The M1 reference runtime supports only its conformance probe.",
            );
        }
        if envelope.target_component != "INT-01" || envelope.audience != "INT-01" {
            return refused(
                "target_or_audience_invalid",
                "The interaction target or audience does not match this component.",
            );
        }
        if envelope.authority.environment_id != self.config.environment_id
            || envelope.authority.requester.environment_id != self.config.environment_id
        {
            return refused(
                "environment_mismatch",
                "The authority context is not valid in this environment.",
            );
        }
        if envelope.authority.requester.principal_type != PrincipalType::Workload {
            return refused(
                "principal_type_invalid",
                "The assurance adapter requires a distinct workload requester.",
            );
        }
        if envelope.authority.actor.as_ref().is_some_and(|actor| {
            !matches!(
                actor.principal_type,
                PrincipalType::ExternalHuman | PrincipalType::SyntheticHuman
            ) || actor.environment_id != self.config.environment_id
        }) {
            return refused(
                "actor_context_invalid",
                "The initiating actor is not a valid human or synthetic-human context.",
            );
        }
        if !envelope
            .authority
            .roles
            .iter()
            .any(|role| role == "interaction-assurance")
            || !envelope
                .authority
                .delegated_authority
                .iter()
                .any(|authority| authority == "C-001:submit")
            || envelope.authority.purpose.code != "assurance.conformance"
        {
            return refused(
                "authority_insufficient",
                "The authority context does not permit this assurance operation.",
            );
        }
        if !envelope
            .authority
            .constraints
            .target_components
            .iter()
            .any(|target| target == "INT-01")
            || !envelope
                .authority
                .constraints
                .information_levels
                .contains(&envelope.classification.level)
        {
            return refused(
                "authority_constraint_mismatch",
                "The target or information class exceeds the authority constraints.",
            );
        }
        if envelope.classification.level == InformationLevel::RestrictedSecurity {
            return refused(
                "restricted_security_payload",
                "Restricted security information is prohibited on the general interaction path.",
            );
        }
        if contains_prohibited_field(&envelope.payload) {
            return refused(
                "prohibited_sensitive_field",
                "The payload contains a field prohibited from the general interaction path.",
            );
        }
        if envelope.idempotency_key.is_none() {
            return refused(
                "idempotency_key_required",
                "A command requires an idempotency key.",
            );
        }
        if envelope
            .payload
            .get("probe")
            .and_then(Value::as_str)
            .is_none_or(|probe| probe.is_empty() || probe.len() > 128)
        {
            return refused(
                "payload_invalid",
                "The conformance probe payload is missing or invalid.",
            );
        }

        let Ok(issued_at) = parse_time(&envelope.issued_at) else {
            return refused("time_invalid", "The issued time is not valid RFC 3339.");
        };
        let Some(expires_at_text) = envelope.expires_at.as_deref() else {
            return refused(
                "expiry_required",
                "An expiring command requires an expiry time.",
            );
        };
        let Ok(expires_at) = parse_time(expires_at_text) else {
            return refused("time_invalid", "The expiry time is not valid RFC 3339.");
        };
        if expires_at <= issued_at {
            return refused(
                "time_window_invalid",
                "The expiry time must follow the issued time.",
            );
        }
        if issued_at > now + CLOCK_TOLERANCE {
            return Decision {
                status: OutcomeStatus::Refused,
                code: "not_yet_valid",
                summary: "The command is not yet valid under the configured clock tolerance.",
                retryable: true,
            };
        }
        if now >= expires_at {
            return Decision {
                status: OutcomeStatus::Expired,
                code: "command_expired",
                summary: "The command expired before it could be accepted.",
                retryable: false,
            };
        }
        if let Some(authority_expiry) = envelope.authority.constraints.expires_at.as_deref() {
            let Ok(authority_expires_at) = parse_time(authority_expiry) else {
                return refused(
                    "authority_time_invalid",
                    "The authority constraint expiry is not valid RFC 3339.",
                );
            };
            if now >= authority_expires_at {
                return Decision {
                    status: OutcomeStatus::Expired,
                    code: "authority_expired",
                    summary: "The authority context expired before the command was accepted.",
                    retryable: false,
                };
            }
        }

        Decision {
            status: OutcomeStatus::Accepted,
            code: "conformance_probe_recorded",
            summary: "The M1 conformance probe was durably recorded.",
            retryable: false,
        }
    }

    fn with_locked_records(&self) -> Result<Vec<JournalRecord>, RuntimeError> {
        self.prepare_state()?;
        let lock = self.open_lock()?;
        lock.lock_shared()
            .map_err(|_| RuntimeError::StateUnavailable)?;
        let result = self.load_records();
        let unlock_result = lock.unlock().map_err(|_| RuntimeError::StateUnavailable);
        match (result, unlock_result) {
            (Ok(records), Ok(())) => Ok(records),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }

    fn prepare_state(&self) -> Result<(), RuntimeError> {
        fs::create_dir_all(&self.config.state_dir).map_err(|_| RuntimeError::StateUnavailable)?;
        restrict_directory(&self.config.state_dir)?;
        Ok(())
    }

    fn open_lock(&self) -> Result<File, RuntimeError> {
        secure_open_options()
            .create(true)
            .read(true)
            .write(true)
            .open(self.config.state_dir.join(LOCK_FILE))
            .map_err(|_| RuntimeError::StateUnavailable)
    }

    fn load_records(&self) -> Result<Vec<JournalRecord>, RuntimeError> {
        let path = self.config.state_dir.join(JOURNAL_FILE);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(path).map_err(|_| RuntimeError::StateUnavailable)?;
        let mut records = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|_| RuntimeError::StateCorrupt)?;
            if line.trim().is_empty() {
                return Err(RuntimeError::StateCorrupt);
            }
            let record: JournalRecord =
                serde_json::from_str(&line).map_err(|_| RuntimeError::StateCorrupt)?;
            if record.journal_version != JOURNAL_VERSION
                || record.environment_id != self.config.environment_id
            {
                return Err(RuntimeError::StateCorrupt);
            }
            records.push(record);
        }
        Ok(records)
    }

    fn append_record(&self, record: &JournalRecord) -> Result<(), RuntimeError> {
        let mut file = secure_open_options()
            .create(true)
            .append(true)
            .open(self.config.state_dir.join(JOURNAL_FILE))
            .map_err(|_| RuntimeError::StateUnavailable)?;
        serde_json::to_writer(&mut file, record).map_err(|_| RuntimeError::Serialization)?;
        file.write_all(b"\n")
            .map_err(|_| RuntimeError::StateUnavailable)?;
        file.sync_data().map_err(|_| RuntimeError::StateUnavailable)
    }
}

fn supported_contract(id: &str) -> SupportedContract {
    SupportedContract {
        contract_id: id.to_owned(),
        versions: vec!["1.0.0".to_owned()],
    }
}

fn journal_evidence(outcome_id: &str, completed_at: &str) -> EvidenceReference {
    EvidenceReference {
        contract_id: "C-004".to_owned(),
        contract_version: C004_VERSION.to_owned(),
        evidence_id: format!("evidence-{outcome_id}"),
        evidence_kind: EvidenceKind::AuditRecord,
        owner_component: "INT-01".to_owned(),
        reference: format!("urn:public-purpose-lab:evidence:interaction-journal.{outcome_id}"),
        created_at: completed_at.to_owned(),
        classification: InformationLevel::Internal,
        media_type: Some("application/json".to_owned()),
        digest: None,
        source_version: None,
        retention_class: "m1-assurance-evidence".to_owned(),
        access_policy_ref: "access-policy-support-001".to_owned(),
        predecessor_evidence_ids: Vec::new(),
    }
}

const fn refused(code: &'static str, summary: &'static str) -> Decision {
    Decision {
        status: OutcomeStatus::Refused,
        code,
        summary,
        retryable: false,
    }
}

fn contains_prohibited_field(value: &Value) -> bool {
    const PROHIBITED: &[&str] = &[
        "password",
        "accesstoken",
        "refreshtoken",
        "bearertoken",
        "cookie",
        "privatekey",
        "sessionsecret",
        "rawgrant",
        "signedgrant",
        "credential",
    ];

    match value {
        Value::Object(object) => object.iter().any(|(key, child)| {
            let normalised: String = key
                .chars()
                .filter(char::is_ascii_alphanumeric)
                .flat_map(char::to_lowercase)
                .collect();
            PROHIBITED.contains(&normalised.as_str()) || contains_prohibited_field(child)
        }),
        Value::Array(items) => items.iter().any(contains_prohibited_field),
        _ => false,
    }
}

fn typed_fingerprint(envelope: &InteractionEnvelope) -> Result<String, RuntimeError> {
    let operation = SemanticOperation {
        contract_id: &envelope.contract_id,
        contract_version: &envelope.contract_version,
        message_type: &envelope.message_type,
        message_kind: &envelope.message_kind,
        source_component: &envelope.source_component,
        target_component: &envelope.target_component,
        audience: &envelope.audience,
        issued_at: &envelope.issued_at,
        expires_at: &envelope.expires_at,
        authority: &envelope.authority,
        classification: &envelope.classification,
        payload: &envelope.payload,
    };
    let bytes = serde_json::to_vec(&operation).map_err(|_| RuntimeError::Serialization)?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Parses a contract timestamp using the common RFC 3339 profile.
///
/// # Errors
///
/// Returns the parser error when the timestamp is not valid RFC 3339.
pub fn parse_time(value: &str) -> Result<OffsetDateTime, time::error::Parse> {
    OffsetDateTime::parse(value, &Rfc3339)
}

/// Returns the current UTC time in the common RFC 3339 representation.
///
/// # Errors
///
/// Returns a serialization error if the platform time cannot be represented.
pub fn now_utc() -> Result<String, RuntimeError> {
    format_time(OffsetDateTime::now_utc())
}

fn format_time(value: OffsetDateTime) -> Result<String, RuntimeError> {
    value
        .format(&Rfc3339)
        .map_err(|_| RuntimeError::Serialization)
}

#[cfg(unix)]
fn restrict_directory(path: &Path) -> Result<(), RuntimeError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| RuntimeError::StateUnavailable)
}

#[cfg(not(unix))]
fn restrict_directory(_path: &Path) -> Result<(), RuntimeError> {
    Ok(())
}

fn secure_open_options() -> OpenOptions {
    let options = OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        let mut options = options;
        options.mode(0o600);
        options
    }
    #[cfg(not(unix))]
    {
        options
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{
            Arc, Barrier,
            atomic::{AtomicU64, Ordering},
        },
        thread,
    };

    use ppl_contracts::{InteractionEnvelope, OutcomeStatus};
    use serde_json::json;

    use super::{InteractionRuntime, InteractionState, RuntimeConfig, RuntimeError, parse_time};

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    fn test_runtime() -> InteractionRuntime {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let state_dir = std::env::temp_dir().join(format!(
            "public-purpose-lab-int-01-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&state_dir);
        InteractionRuntime::new(RuntimeConfig {
            state_dir,
            environment_id: "env-local-001".to_owned(),
        })
    }

    fn command() -> InteractionEnvelope {
        serde_json::from_str(include_str!(
            "../../../../contracts/common/examples/c-001-m1-conformance-command.json"
        ))
        .expect("canonical command must deserialize")
    }

    fn now() -> time::OffsetDateTime {
        parse_time("2030-08-25T12:01:00Z").expect("test time must parse")
    }

    #[test]
    fn duplicate_is_not_applied_after_restart() {
        let runtime = test_runtime();
        let first = runtime.process(&command(), now()).expect("first command");
        assert_eq!(first.status, OutcomeStatus::Accepted);

        let restarted = InteractionRuntime::new(RuntimeConfig {
            state_dir: runtime.config.state_dir.clone(),
            environment_id: runtime.config.environment_id.clone(),
        });
        let duplicate = restarted
            .process(&command(), now())
            .expect("duplicate command");
        assert_eq!(duplicate.status, OutcomeStatus::Duplicate);
        assert_eq!(duplicate.original_outcome_id, Some(first.outcome_id));

        let records = restarted.load_records().expect("journal must load");
        assert_eq!(
            records
                .iter()
                .filter(|record| record.operation_applied)
                .count(),
            1
        );
    }

    #[test]
    fn conflicting_idempotency_content_is_refused() {
        let runtime = test_runtime();
        runtime.process(&command(), now()).expect("first command");
        let mut changed = command();
        changed.payload = json!({ "probe": "changed-content" });

        let outcome = runtime.process(&changed, now()).expect("conflict outcome");
        assert_eq!(outcome.status, OutcomeStatus::Refused);
        assert_eq!(outcome.code, "idempotency_conflict");
    }

    #[test]
    fn delivery_metadata_can_change_without_reapplying_the_operation() {
        let runtime = test_runtime();
        let first = runtime.process(&command(), now()).expect("first command");
        let mut retried = command();
        retried.message_id = "message-m1-conformance-retry-002".to_owned();
        retried.correlation_id = "correlation-m1-conformance-retry-002".to_owned();
        retried.trace.trace_id = "trace-m1-conformance-retry-002".to_owned();
        retried.security.authentication_context_ref = "auth-context-assurance-002".to_owned();

        let duplicate = runtime
            .process(&retried, now())
            .expect("metadata-only retry outcome");
        assert_eq!(duplicate.status, OutcomeStatus::Duplicate);
        assert_eq!(duplicate.original_outcome_id, Some(first.outcome_id));
    }

    #[test]
    fn concurrent_delivery_applies_one_operation() {
        let runtime = test_runtime();
        let barrier = Arc::new(Barrier::new(8));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let runtime = runtime.clone();
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    runtime
                        .process(&command(), now())
                        .expect("concurrent outcome")
                })
            })
            .collect();
        let outcomes: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().expect("thread must finish"))
            .collect();

        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| outcome.status == OutcomeStatus::Accepted)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| outcome.status == OutcomeStatus::Duplicate)
                .count(),
            7
        );
    }

    #[test]
    fn sensitive_payload_is_refused_and_not_retained() {
        let runtime = test_runtime();
        let mut sensitive = command();
        sensitive.payload = json!({
            "probe": "m1-common-interaction",
            "password": "example-secret-must-not-survive"
        });
        let outcome = runtime
            .process(&sensitive, now())
            .expect("safe refusal outcome");
        assert_eq!(outcome.status, OutcomeStatus::Refused);
        assert_eq!(outcome.code, "prohibited_sensitive_field");

        let journal = fs::read_to_string(runtime.config.state_dir.join(super::JOURNAL_FILE))
            .expect("journal must exist");
        assert!(!journal.contains("password"));
        assert!(!journal.contains("example-secret-must-not-survive"));
        assert!(!journal.contains("idempotency-m1-conformance-001"));
        assert!(!journal.contains("workload-framework-host"));
        assert!(!journal.contains("issuer-local-assurance"));
        assert!(!journal.contains("auth-context-assurance-001"));
    }

    #[test]
    fn corrupt_journal_makes_readiness_false() {
        let runtime = test_runtime();
        runtime.prepare_state().expect("state directory");
        fs::write(
            runtime.config.state_dir.join(super::JOURNAL_FILE),
            "{incomplete",
        )
        .expect("corrupt fixture");

        let health = runtime.health();
        assert_eq!(health.interaction_state, InteractionState::NotReady);
        assert_eq!(health.reason_code, Some("interaction_state_inconsistent"));
    }

    #[test]
    fn unavailable_state_fails_closed() {
        let runtime = test_runtime();
        fs::write(&runtime.config.state_dir, "not-a-directory")
            .expect("blocking file must be created");

        assert!(matches!(
            runtime.process(&command(), now()),
            Err(RuntimeError::StateUnavailable)
        ));
        let health = runtime.health();
        assert_eq!(health.interaction_state, InteractionState::NotReady);
        assert_eq!(health.reason_code, Some("interaction_state_unavailable"));
    }

    #[test]
    fn expired_command_is_not_applied() {
        let runtime = test_runtime();
        let mut expired = command();
        expired.expires_at = Some("2030-08-25T12:00:30Z".to_owned());

        let outcome = runtime.process(&expired, now()).expect("expiry outcome");
        assert_eq!(outcome.status, OutcomeStatus::Expired);
        assert_eq!(outcome.code, "command_expired");
    }

    #[test]
    fn retryable_early_delivery_is_applied_once_in_its_valid_window() {
        let runtime = test_runtime();
        let early = parse_time("2030-08-25T11:50:00Z").expect("early test time");
        let first = runtime
            .process(&command(), early)
            .expect("early refusal outcome");
        assert_eq!(first.status, OutcomeStatus::Refused);
        assert!(first.retryable);

        let accepted = runtime
            .process(&command(), now())
            .expect("in-window accepted outcome");
        assert_eq!(accepted.status, OutcomeStatus::Accepted);
        let duplicate = runtime
            .process(&command(), now())
            .expect("post-acceptance duplicate outcome");
        assert_eq!(duplicate.status, OutcomeStatus::Duplicate);

        let records = runtime.load_records().expect("journal must load");
        assert_eq!(
            records
                .iter()
                .filter(|record| record.operation_applied)
                .count(),
            1
        );
    }

    #[test]
    fn unsupported_contract_version_is_refused() {
        let runtime = test_runtime();
        let mut unsupported = command();
        unsupported.contract_version = "2.0.0".to_owned();

        let outcome = runtime
            .process(&unsupported, now())
            .expect("unsupported-version outcome");
        assert_eq!(outcome.status, OutcomeStatus::Refused);
        assert_eq!(outcome.code, "contract_version_unsupported");
    }

    #[test]
    fn another_environment_cannot_use_the_authority_context() {
        let runtime = test_runtime();
        let mut wrong_environment = command();
        wrong_environment.authority.environment_id = "env-other-001".to_owned();
        wrong_environment.authority.requester.environment_id = "env-other-001".to_owned();

        let outcome = runtime
            .process(&wrong_environment, now())
            .expect("environment-refusal outcome");
        assert_eq!(outcome.status, OutcomeStatus::Refused);
        assert_eq!(outcome.code, "environment_mismatch");
    }
}
