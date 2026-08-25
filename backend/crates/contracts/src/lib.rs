//! Shared Rust representations of the transport-neutral common contracts.
//!
//! JSON Schema under `contracts/common` remains canonical. These types provide
//! the implemented Rust consumption surface and are checked against its
//! examples. They do not authenticate an envelope or select a transport.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const C001_VERSION: &str = "1.0.0";
pub const C002_VERSION: &str = "1.0.0";
pub const C003_VERSION: &str = "1.0.0";
pub const C004_VERSION: &str = "1.0.0";
pub const C005_VERSION: &str = "1.0.0";
pub const C006_VERSION: &str = "1.0.0";

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
        CommandOutcome, ComponentCapabilityManifest, ContractCompatibilityDescriptor,
        InteractionEnvelope, PrincipalType,
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
}
