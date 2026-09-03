//! Shared Rust representations of the transport-neutral framework contracts.
//!
//! JSON Schema under `contracts/` remains canonical. These types provide the
//! implemented Rust consumption surface and are checked against canonical
//! examples. They do not authenticate an envelope or select a transport.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const C001_VERSION: &str = "1.0.0";
pub const C002_VERSION: &str = "1.0.0";
pub const C003_VERSION: &str = "1.0.0";
pub const C004_VERSION: &str = "1.0.0";
pub const C005_VERSION: &str = "1.0.0";
pub const C006_VERSION: &str = "1.0.0";
pub const I001_VERSION: &str = "1.0.0";
pub const I002_VERSION: &str = "1.0.0";
pub const I003_VERSION: &str = "1.0.0";
pub const I004_VERSION: &str = "1.0.0";
pub const I005_VERSION: &str = "1.0.0";
pub const AZ001_VERSION: &str = "1.0.0";
pub const D001_VERSION: &str = "1.0.0";
pub const D002_VERSION: &str = "1.0.0";
pub const D003_VERSION: &str = "1.0.0";
pub const D004_VERSION: &str = "1.0.0";
pub const P001_VERSION: &str = "1.0.0";
pub const P002_VERSION: &str = "1.0.0";
pub const P003_VERSION: &str = "1.0.0";
pub const P004_VERSION: &str = "1.0.0";
pub const A001_VERSION: &str = "0.1.0";
pub const O001_VERSION: &str = "0.1.0";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceAcquisitionMode {
    Upload,
    Paste,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SourceIntakePayload {
    pub acquisition_mode: SourceAcquisitionMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_name: Option<String>,
    pub media_type: String,
    pub size_bytes: u64,
    pub content: String,
    pub title: String,
    pub owner: String,
    pub rights: String,
    pub provenance: String,
    pub classification: String,
}

/// Gate C working A-001 command. The browser supplies source fields to the
/// Workbench backend; the backend adds the authenticated actor and scenario
/// binding before this message enters the component channel.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SourceIntakeCommand {
    pub contract_id: String,
    pub contract_version: String,
    pub message_type: String,
    pub command_id: String,
    pub action: String,
    pub environment_id: String,
    pub demonstration_session_id: String,
    pub engagement_id: String,
    pub actor_id: String,
    pub actor_role: String,
    pub authority_reference: String,
    pub purpose: String,
    pub correlation_id: String,
    pub causation_id: String,
    pub idempotency_key: String,
    pub issued_at: String,
    pub source: SourceIntakePayload,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SourceVersionSummary {
    pub source_id: String,
    pub source_version_id: String,
    pub version: u64,
    pub status: String,
    pub digest_algorithm: String,
    pub digest_value: String,
    pub acquisition_mode: SourceAcquisitionMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_name: Option<String>,
    pub media_type: String,
    pub size_bytes: u64,
    pub title: String,
    pub owner: String,
    pub rights: String,
    pub provenance: String,
    pub classification: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceIntakeStatus {
    Quarantined,
    Refused,
    Duplicate,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SourceIntakeOutcome {
    pub contract_id: String,
    pub contract_version: String,
    pub message_type: String,
    pub outcome_id: String,
    pub command_id: String,
    pub status: SourceIntakeStatus,
    pub code: String,
    pub environment_id: String,
    pub demonstration_session_id: String,
    pub engagement_id: String,
    pub actor_id: String,
    pub correlation_id: String,
    pub recorded_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_version: Option<SourceVersionSummary>,
    pub event_types: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SourceIntakeQuery {
    pub contract_id: String,
    pub contract_version: String,
    pub message_type: String,
    pub query_id: String,
    pub command_id: String,
    pub environment_id: String,
    pub requested_at: String,
}

/// Gate A operational command. O-001 remains an in-development contract until
/// the component mesh has supplied implementation evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OperationalCommand {
    pub contract_id: String,
    pub contract_version: String,
    pub command_id: String,
    pub command_name: String,
    pub target_component: String,
    pub issuer_component: String,
    pub environment_id: String,
    pub purpose: String,
    pub correlation_id: String,
    pub causation_id: String,
    pub idempotency_key: String,
    pub issued_at: String,
}

/// Privacy-minimised operational fact used for readiness and command outcomes.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OperationalEvent {
    pub contract_id: String,
    pub contract_version: String,
    pub event_id: String,
    pub event_type: String,
    pub component_id: String,
    pub component_name: String,
    pub instance_id: String,
    pub workload_identity: String,
    pub environment_id: String,
    pub status: String,
    pub capability: String,
    pub source_revision: String,
    pub image_digest: String,
    pub occurred_at: String,
    pub information_profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_reference: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioSurfaceSlot {
    pub slot_id: String,
    pub role: String,
    pub required: bool,
    pub supported_views: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioStep {
    pub step_id: String,
    pub kind: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioStage {
    pub stage_id: String,
    pub title: String,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioLogicalTimeDefinition {
    pub mode: String,
    pub initial_instant: String,
    pub maximum_advance_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioControls {
    pub logical_time: ScenarioLogicalTimeDefinition,
    pub reset_targets: Vec<String>,
    pub fault_profiles: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioPackage {
    pub contract_id: String,
    pub contract_version: String,
    pub package_id: String,
    pub package_version: String,
    pub title: String,
    pub purpose: String,
    pub publisher: String,
    pub released_at: String,
    pub information_profile: String,
    pub permitted_profiles: Vec<String>,
    pub surface_slots: Vec<ScenarioSurfaceSlot>,
    pub stages: Vec<ScenarioStage>,
    pub controls: ScenarioControls,
    pub limitations: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScenarioLifecycleAction {
    Create,
    Prepare,
    Start,
    Pause,
    Resume,
    Complete,
    Stop,
    Reset,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScenarioState {
    Preparing,
    Ready,
    Running,
    Paused,
    Completed,
    Stopped,
    Failed,
    Superseded,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioLifecycleCommand {
    pub contract_id: String,
    pub contract_version: String,
    pub operation_id: String,
    pub session_id: String,
    pub package_id: String,
    pub package_version: String,
    pub action: ScenarioLifecycleAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_state: Option<ScenarioState>,
    pub expected_revision: u64,
    pub requested_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioControlCommand {
    pub contract_id: String,
    pub contract_version: String,
    pub operation_id: String,
    pub session_id: String,
    pub kind: String,
    pub operation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_instant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advance_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_milliseconds: Option<u64>,
    pub expected_revision: u64,
    pub requested_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScenarioCheckpointEvaluation {
    pub contract_id: String,
    pub contract_version: String,
    pub evaluation_id: String,
    pub session_id: String,
    pub claim_class: String,
    pub claim_id: String,
    pub result: String,
    pub source_contract: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<String>,
    pub observed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub evidence_references: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PresentationViewCapability {
    pub view_id: String,
    pub view_version: String,
    pub context_keys: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PresentationCapabilityManifest {
    pub contract_id: String,
    pub contract_version: String,
    pub manifest_id: String,
    pub manifest_version: String,
    pub application_id: String,
    pub application_release: String,
    pub surface_roles: Vec<String>,
    pub views: Vec<PresentationViewCapability>,
    pub information_profiles: Vec<String>,
    pub released_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PresentationRegistration {
    pub contract_id: String,
    pub contract_version: String,
    pub registration_id: String,
    pub session_id: String,
    pub surface_slot: String,
    pub surface_role: String,
    pub manifest_id: String,
    pub manifest_version: String,
    pub supported_views: Vec<String>,
    pub binding_mode: String,
    pub registration_revision: u64,
    pub connection_generation: u64,
    pub lease_expires_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PresentationCue {
    pub contract_id: String,
    pub contract_version: String,
    pub cue_id: String,
    pub cue_digest: String,
    pub idempotency_key: String,
    pub session_id: String,
    pub session_revision: u64,
    pub surface_slot: String,
    pub registration_id: String,
    pub registration_revision: u64,
    pub connection_generation: u64,
    pub semantic_view: String,
    pub view_version: String,
    pub context: std::collections::BTreeMap<String, String>,
    pub issued_at: String,
    pub expires_at: String,
    pub stage_id: String,
    pub step_id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PresentationOutcomeResult {
    Applied,
    Refused,
    Unsupported,
    Expired,
    Duplicate,
    Superseded,
    Failed,
    Uncertain,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PresentationCueOutcome {
    pub contract_id: String,
    pub contract_version: String,
    pub outcome_id: String,
    pub cue_id: String,
    pub cue_digest: String,
    pub session_id: String,
    pub session_revision: u64,
    pub surface_slot: String,
    pub registration_id: String,
    pub registration_revision: u64,
    pub connection_generation: u64,
    pub semantic_view: String,
    pub result: PresentationOutcomeResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_code: Option<String>,
    pub concluded_at: String,
    pub business_completion_claimed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageKind {
    BusinessCommand,
    BusinessEvent,
    DemonstrationControl,
    TrustCommand,
    PresentationCue,
    OperationalSignal,
    AnalyticalProjection,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrincipalType {
    ExternalHuman,
    SyntheticHuman,
    Workload,
    Operator,
    ServiceOwner,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PrincipalReference {
    pub principal_type: PrincipalType,
    pub principal_id: String,
    pub environment_id: String,
    pub issuer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_domain: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Purpose {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorityConstraints {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub engagement_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demonstration_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_components: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub information_levels: Vec<InformationLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorityContext {
    pub contract_id: String,
    pub contract_version: String,
    pub environment_id: String,
    pub requester: PrincipalReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<PrincipalReference>,
    pub roles: Vec<String>,
    pub delegated_authority: Vec<String>,
    pub purpose: Purpose,
    pub constraints: AuthorityConstraints,
    pub policy_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_reference: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InformationLevel {
    Public,
    Synthetic,
    Internal,
    RestrictedSecurity,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InformationClassification {
    pub level: InformationLevel,
    pub categories: Vec<String>,
    pub handling: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SecurityMetadata {
    pub authentication_context_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity_reference: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TraceContext {
    pub trace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct InteractionEnvelope {
    pub contract_id: String,
    pub contract_version: String,
    pub message_id: String,
    pub message_type: String,
    pub message_kind: MessageKind,
    pub source_component: String,
    pub target_component: String,
    pub audience: String,
    pub issued_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurred_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub correlation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub authority: AuthorityContext,
    pub classification: InformationClassification,
    pub security: SecurityMetadata,
    pub trace: TraceContext,
    pub payload: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceKind {
    SourceVersion,
    ProcessingRecord,
    AuditRecord,
    DecisionRecord,
    RuleExecution,
    ModelExecution,
    ReleasedArtifact,
    ConformanceResult,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EvidenceDigest {
    pub algorithm: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EvidenceReference {
    pub contract_id: String,
    pub contract_version: String,
    pub evidence_id: String,
    pub evidence_kind: EvidenceKind,
    pub owner_component: String,
    pub reference: String,
    pub created_at: String,
    pub classification: InformationLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<EvidenceDigest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_version: Option<String>,
    pub retention_class: String,
    pub access_policy_ref: String,
    #[serde(default)]
    pub predecessor_evidence_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutcomeStatus {
    Accepted,
    Refused,
    Expired,
    Duplicate,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CommandOutcome {
    pub contract_id: String,
    pub contract_version: String,
    pub outcome_id: String,
    pub command_message_id: String,
    pub status: OutcomeStatus,
    pub code: String,
    pub summary: String,
    pub retryable: bool,
    pub completed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_outcome_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_owner: Option<String>,
    pub evidence: Vec<EvidenceReference>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthenticationStrength {
    SingleFactor,
    MultiFactor,
    PhishingResistant,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ExternalHumanIdentityContext {
    pub contract_id: String,
    pub contract_version: String,
    pub context_id: String,
    pub environment_id: String,
    pub issuer: String,
    pub subject_id: String,
    pub principal_id: String,
    pub roles: Vec<String>,
    pub audience: String,
    pub authentication_strength: AuthenticationStrength,
    pub mapping_version: String,
    pub issued_at: String,
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_reference: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkloadIdentityContext {
    pub contract_id: String,
    pub contract_version: String,
    pub context_id: String,
    pub environment_id: String,
    pub issuer: String,
    pub workload_id: String,
    pub audiences: Vec<String>,
    pub contract_actions: Vec<String>,
    pub attestation_reference: String,
    pub policy_version: String,
    pub issued_at: String,
    pub expires_at: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnvironmentClass {
    LocalScratch,
    PortableIsolated,
    HostedShared,
    ProductionLike,
    Production,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InformationProfile {
    SyntheticOnly,
    NonSyntheticAuthorised,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustProfile {
    LocalSynthetic,
    Managed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyCustodyClass {
    LocalFile,
    ManagedService,
    HardwareBacked,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecoveryProfile {
    RebuildNewTrustDomain,
    ProtectedSameEnvironment,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustStatus {
    Ready,
    NotReady,
    Revoked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SyntheticTrustBootstrapRecord {
    pub contract_id: String,
    pub contract_version: String,
    pub record_id: String,
    pub environment_id: String,
    pub environment_class: EnvironmentClass,
    pub information_profile: InformationProfile,
    pub trust_profile: TrustProfile,
    pub trust_domain: String,
    pub trust_epoch: u64,
    pub signer_id: String,
    pub public_key_fingerprint: String,
    pub key_custody_class: KeyCustodyClass,
    pub recovery_profile: RecoveryProfile,
    pub status: TrustStatus,
    pub compatible: bool,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DemonstrationSignInGrantClaims {
    pub grant_id: String,
    pub establishment_operation_id: String,
    pub environment_id: String,
    pub trust_domain: String,
    pub trust_epoch: u64,
    pub actor_id: String,
    pub application_id: String,
    pub audience: String,
    pub surface_id: String,
    pub demonstration_session_id: String,
    pub roles: Vec<String>,
    pub purpose: String,
    pub synthetic_realm: String,
    pub decision_reference: String,
    pub issued_at: String,
    pub not_before: String,
    pub expires_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GrantSignature {
    pub profile: String,
    pub algorithm: String,
    pub signer_id: String,
    pub public_key_fingerprint: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DemonstrationSignInGrant {
    pub contract_id: String,
    pub contract_version: String,
    pub claims: DemonstrationSignInGrantClaims,
    pub signature: GrantSignature,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyntheticSessionStatus {
    Established,
    Refused,
    Expired,
    ReplayDetected,
    Failed,
    Terminated,
    Revoked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SyntheticSessionOutcome {
    pub contract_id: String,
    pub contract_version: String,
    pub outcome_id: String,
    pub grant_id: String,
    pub establishment_operation_id: String,
    pub environment_id: String,
    pub application_id: String,
    pub surface_id: String,
    pub demonstration_session_id: String,
    pub actor_id: String,
    pub roles: Vec<String>,
    pub synthetic_realm: String,
    pub status: SyntheticSessionStatus,
    pub occurred_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum_valid_until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    pub decision_reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_outcome_id: Option<String>,
    #[serde(default)]
    pub evidence_references: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssertionType {
    Relationship,
    Consent,
    Restriction,
    Organisation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssertionStatus {
    Active,
    Revoked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthoritativeAssertion {
    pub source_id: String,
    pub assertion_type: AssertionType,
    pub subject_id: String,
    pub resource_id: String,
    pub purpose_codes: Vec<String>,
    pub status: AssertionStatus,
    pub effective_at: String,
    pub expires_at: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorisationDecisionRequest {
    pub contract_id: String,
    pub contract_version: String,
    pub kind: String,
    pub request_id: String,
    pub environment_id: String,
    pub requester: PrincipalReference,
    pub actor: PrincipalReference,
    pub action: String,
    pub resource: String,
    pub purpose: String,
    pub requested_roles: Vec<String>,
    pub assertions: Vec<AuthoritativeAssertion>,
    pub policy_version: String,
    pub requested_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorisationObligation {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorisationDecisionStatus {
    Permit,
    Deny,
    NotApplicable,
    Indeterminate,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorisationDecision {
    pub contract_id: String,
    pub contract_version: String,
    pub kind: String,
    pub decision_id: String,
    pub request_id: String,
    pub status: AuthorisationDecisionStatus,
    pub reason_code: String,
    pub obligations: Vec<AuthorisationObligation>,
    pub policy_version: String,
    pub decided_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
    #[serde(default)]
    pub evidence_references: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentMaturity {
    RepositorySkeleton,
    InDevelopment,
    Demonstrated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentProfile {
    LocalPrivate,
    PortableDemonstration,
    HostedDemonstrator,
    DevelopmentAssurance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SupportedContract {
    pub contract_id: String,
    pub versions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SemanticCapability {
    pub capability_id: String,
    pub description: String,
    pub contracts: Vec<SupportedContract>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ComponentCapabilityManifest {
    pub contract_id: String,
    pub contract_version: String,
    pub component_id: String,
    pub component_name: String,
    pub release_version: String,
    pub maturity: ComponentMaturity,
    pub generated_at: String,
    pub supported_profiles: Vec<DeploymentProfile>,
    pub capabilities: Vec<SemanticCapability>,
    pub readiness_dependencies: Vec<String>,
    pub evidence: Vec<EvidenceReference>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContractStatus {
    WorkingDraft,
    Agreed,
    Implemented,
    Deprecated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CompatibilityMode {
    None,
    Backward,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContractCompatibilityDescriptor {
    pub contract_id: String,
    pub contract_version: String,
    pub described_contract_id: String,
    pub described_contract_name: String,
    pub described_version: String,
    pub status: ContractStatus,
    pub schema_id: String,
    pub compatibility_mode: CompatibilityMode,
    pub effective_from: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    pub examples: Vec<String>,
    pub producer_expectations: Vec<String>,
    pub consumer_expectations: Vec<String>,
    pub evidence: Vec<EvidenceReference>,
}

#[cfg(test)]
mod tests {
    use super::{
        AuthorisationDecision, CommandOutcome, ComponentCapabilityManifest,
        ContractCompatibilityDescriptor, DemonstrationSignInGrant, ExternalHumanIdentityContext,
        InteractionEnvelope, PresentationCapabilityManifest, PresentationCue,
        PresentationCueOutcome, PresentationRegistration, PrincipalType,
        ScenarioCheckpointEvaluation, ScenarioControlCommand, ScenarioLifecycleCommand,
        ScenarioPackage, SourceIntakeCommand, SourceIntakeOutcome, SourceIntakeQuery,
        SyntheticSessionOutcome, SyntheticTrustBootstrapRecord, WorkloadIdentityContext,
    };

    #[test]
    fn canonical_command_example_deserialises() {
        let envelope: InteractionEnvelope = serde_json::from_str(include_str!(
            "../../../../contracts/common/examples/c-001-m1-conformance-command.json"
        ))
        .expect("canonical command must match Rust types");

        assert_eq!(envelope.contract_id, "C-001");
        assert_eq!(
            envelope.authority.requester.principal_type,
            PrincipalType::Workload
        );
        assert!(envelope.idempotency_key.is_some());
    }

    #[test]
    fn canonical_outcome_and_descriptors_deserialise() {
        let _: CommandOutcome = serde_json::from_str(include_str!(
            "../../../../contracts/common/examples/c-003-accepted-outcome.json"
        ))
        .expect("canonical outcome must match Rust types");
        let _: ComponentCapabilityManifest = serde_json::from_str(include_str!(
            "../../../../contracts/common/examples/c-005-int-01-manifest.json"
        ))
        .expect("canonical manifest must match Rust types");
        let _: ContractCompatibilityDescriptor = serde_json::from_str(include_str!(
            "../../../../contracts/common/descriptors/c-001.json"
        ))
        .expect("compatibility descriptor must match Rust types");
    }

    #[test]
    fn canonical_m2_examples_deserialise() {
        let _: ExternalHumanIdentityContext = serde_json::from_str(include_str!(
            "../../../../contracts/identity/examples/i-001-external-human-context.json"
        ))
        .expect("I-001 example must match Rust types");
        let _: WorkloadIdentityContext = serde_json::from_str(include_str!(
            "../../../../contracts/identity/examples/i-002-workload-context.json"
        ))
        .expect("I-002 example must match Rust types");
        let _: SyntheticTrustBootstrapRecord = serde_json::from_str(include_str!(
            "../../../../contracts/identity/examples/i-003-local-synthetic-ready.json"
        ))
        .expect("I-003 example must match Rust types");
        let _: DemonstrationSignInGrant = serde_json::from_str(include_str!(
            "../../../../contracts/identity/examples/i-004-demonstration-sign-in-grant.json"
        ))
        .expect("I-004 example must match Rust types");
        let _: SyntheticSessionOutcome = serde_json::from_str(include_str!(
            "../../../../contracts/identity/examples/i-005-session-established.json"
        ))
        .expect("I-005 example must match Rust types");
        let _: AuthorisationDecision = serde_json::from_str(include_str!(
            "../../../../contracts/authorisation/examples/az-001-permit-decision.json"
        ))
        .expect("AZ-001 example must match Rust types");
    }

    #[test]
    fn canonical_m3_examples_deserialise() {
        let _: ScenarioPackage = serde_json::from_str(include_str!(
            "../../../../scenarios/presentation-control-assurance/scenario.json"
        ))
        .expect("D-001 example must match Rust types");
        let _: ScenarioLifecycleCommand = serde_json::from_str(include_str!(
            "../../../../contracts/demonstration/examples/d-002-create-session.json"
        ))
        .expect("D-002 example must match Rust types");
        let _: ScenarioControlCommand = serde_json::from_str(include_str!(
            "../../../../contracts/demonstration/examples/d-003-advance-time.json"
        ))
        .expect("D-003 example must match Rust types");
        let _: ScenarioCheckpointEvaluation = serde_json::from_str(include_str!(
            "../../../../contracts/demonstration/examples/d-004-presentation-checkpoint.json"
        ))
        .expect("D-004 example must match Rust types");
        let _: PresentationCapabilityManifest = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-001-assurance-surface.json"
        ))
        .expect("P-001 example must match Rust types");
        let _: PresentationRegistration = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-002-audience-registration.json"
        ))
        .expect("P-002 example must match Rust types");
        let _: PresentationCue = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-003-welcome-cue.json"
        ))
        .expect("P-003 example must match Rust types");
        let outcome: PresentationCueOutcome = serde_json::from_str(include_str!(
            "../../../../contracts/presentation/examples/p-004-welcome-applied.json"
        ))
        .expect("P-004 example must match Rust types");
        assert!(!outcome.business_completion_claimed);
    }

    #[test]
    fn canonical_gate_c_source_examples_deserialise() {
        let _: SourceIntakeCommand = serde_json::from_str(include_str!(
            "../../../../contracts/source/examples/a-001-paste-command.json"
        ))
        .expect("A-001 command example must match Rust types");
        let _: SourceIntakeOutcome = serde_json::from_str(include_str!(
            "../../../../contracts/source/examples/a-001-quarantined-outcome.json"
        ))
        .expect("A-001 outcome example must match Rust types");
        let _: SourceIntakeQuery = serde_json::from_str(include_str!(
            "../../../../contracts/source/examples/a-001-outcome-query.json"
        ))
        .expect("A-001 query example must match Rust types");
    }
}
