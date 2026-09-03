export const commonContractVersions = {
  "C-001": "1.0.0",
  "C-002": "1.0.0",
  "C-003": "1.0.0",
  "C-004": "1.0.0",
  "C-005": "1.0.0",
  "C-006": "1.0.0",
} as const;

export const m2ContractVersions = {
  "I-001": "1.0.0",
  "I-002": "1.0.0",
  "I-003": "1.0.0",
  "I-004": "1.0.0",
  "I-005": "1.0.0",
  "AZ-001": "1.0.0",
} as const;

export const m3ContractVersions = {
  "D-001": "1.0.0",
  "D-002": "1.0.0",
  "D-003": "1.0.0",
  "D-004": "1.0.0",
  "P-001": "1.0.0",
  "P-002": "1.0.0",
  "P-003": "1.0.0",
  "P-004": "1.0.0",
} as const;

export const gateCContractVersions = {
  "A-001": "0.1.0",
  "A-002": "0.1.0",
  "K-001": "0.1.0",
} as const;

export type SourceAcquisitionMode = "upload" | "paste";

export interface SourceVersionSummary {
  readonly sourceId: string;
  readonly sourceVersionId: string;
  readonly version: number;
  readonly status: "quarantined";
  readonly digestAlgorithm: "sha-256";
  readonly digestValue: string;
  readonly acquisitionMode: SourceAcquisitionMode;
  readonly originalName?: string;
  readonly mediaType: "text/plain" | "text/markdown";
  readonly sizeBytes: number;
  readonly title: string;
  readonly owner: string;
  readonly rights: string;
  readonly provenance: string;
  readonly classification: "synthetic";
}

export interface SourceIntakeOutcome {
  readonly contractId: "A-001";
  readonly contractVersion: "0.1.0";
  readonly messageType: "source-intake.outcome";
  readonly outcomeId: string;
  readonly commandId: string;
  readonly status: "quarantined" | "refused" | "duplicate";
  readonly code: string;
  readonly environmentId: string;
  readonly demonstrationSessionId: string;
  readonly engagementId: string;
  readonly actorId: string;
  readonly correlationId: string;
  readonly recordedAt: string;
  readonly sourceVersion?: SourceVersionSummary;
  readonly eventTypes: readonly string[];
}

export interface SourceValidationCheck {
  readonly checkId: string;
  readonly status: "passed" | "failed";
  readonly reasonCode?: string;
}

export interface SourceValidationSummary {
  readonly status: "validated" | "refused";
  readonly validatedAt: string;
  readonly digestVerified: boolean;
  readonly reasonCode?: string;
  readonly checks: readonly SourceValidationCheck[];
}

export interface SourceStagingSummary {
  readonly status: "staged" | "refused";
  readonly actorId: string;
  readonly purpose: "governed-source-staging";
  readonly policyDecisionReference: string;
  readonly reasonCode: string;
  readonly decidedAt: string;
}

export interface SourceLifecycleStatus {
  readonly contractId: "A-002";
  readonly contractVersion: "0.1.0";
  readonly messageType: "source-lifecycle.status";
  readonly statusId: string;
  readonly environmentId: string;
  readonly demonstrationSessionId: string;
  readonly engagementId: string;
  readonly sourceVersionId: string;
  readonly lifecycleStatus:
    "validated" | "validation-refused" | "staged" | "staging-refused";
  readonly validation: SourceValidationSummary;
  readonly staging?: SourceStagingSummary;
  readonly recordedAt: string;
}

export interface SourceStageOutcome {
  readonly contractId: "A-002";
  readonly contractVersion: "0.1.0";
  readonly messageType: "staged-source-release.outcome";
  readonly outcomeId: string;
  readonly commandId: string;
  readonly status: "staged" | "refused" | "duplicate";
  readonly code: string;
  readonly sourceStatus: SourceLifecycleStatus;
  readonly eventTypes: readonly string[];
}

export type ProcessingLifecycleState =
  "accepted" | "processing" | "completed" | "failed";

export interface ProcessingLifecycleStage {
  readonly state: ProcessingLifecycleState;
  readonly occurredAt: string;
  readonly reasonCode?: string;
}

export interface BoundedProcessingResult {
  readonly digestVerified: boolean;
  readonly byteCount: number;
  readonly lineCount: number;
  readonly sectionCount: number;
  readonly safePreview: string;
  readonly previewTruncated: boolean;
}

export interface ProcessingLifecycleStatus {
  readonly contractId: "K-001";
  readonly contractVersion: "0.1.0";
  readonly messageType: "processing-lifecycle.status";
  readonly statusId: string;
  readonly processingId: string;
  readonly environmentId: string;
  readonly demonstrationSessionId: string;
  readonly engagementId: string;
  readonly sourceVersionId: string;
  readonly correlationId: string;
  readonly causationId: string;
  readonly lifecycleStatus: ProcessingLifecycleState;
  readonly stages: readonly ProcessingLifecycleStage[];
  readonly result?: BoundedProcessingResult;
  readonly reasonCode?: string;
  readonly terminalCount: number;
  readonly recordedAt: string;
}

export interface PresentationProcessingProgress {
  readonly processingId: string;
  readonly sourceVersionId: string;
  readonly componentId: "KNO-01";
  readonly lifecycleStatus: ProcessingLifecycleState;
  readonly latestOutcome: string;
  readonly acceptedAt?: string;
  readonly startedAt?: string;
  readonly completedAt?: string;
  readonly byteCount?: number;
  readonly lineCount?: number;
  readonly sectionCount?: number;
  readonly limitation: string;
}

export type ScenarioLifecycleAction =
  | "create"
  | "prepare"
  | "start"
  | "pause"
  | "resume"
  | "complete"
  | "stop"
  | "reset";

export interface ScenarioPackage {
  readonly contractId: "D-001";
  readonly contractVersion: "1.0.0";
  readonly packageId: string;
  readonly packageVersion: string;
  readonly title: string;
  readonly purpose: string;
  readonly publisher: string;
  readonly releasedAt: string;
  readonly informationProfile: "synthetic-only";
  readonly permittedProfiles: readonly string[];
  readonly surfaceSlots: readonly {
    readonly slotId: string;
    readonly role: "audience-display" | "reviewer-workbench";
    readonly required: boolean;
    readonly supportedViews: readonly string[];
  }[];
  readonly stages: readonly {
    readonly stageId: string;
    readonly title: string;
    readonly steps: readonly {
      readonly stepId: string;
      readonly kind:
        "lifecycle" | "presentation-cue" | "checkpoint" | "control";
      readonly description: string;
      readonly target?: string;
      readonly semanticView?: string;
      readonly operation?: string;
    }[];
  }[];
  readonly controls: {
    readonly logicalTime: {
      readonly mode: "manual-step";
      readonly initialInstant: string;
      readonly maximumAdvanceSeconds: number;
    };
    readonly resetTargets: readonly string[];
    readonly faultProfiles: readonly string[];
  };
  readonly limitations: readonly string[];
}

export type ScenarioState =
  | "preparing"
  | "ready"
  | "running"
  | "paused"
  | "completed"
  | "stopped"
  | "failed"
  | "superseded";

export interface ScenarioLifecycleCommand {
  readonly contractId: "D-002";
  readonly contractVersion: "1.0.0";
  readonly operationId: string;
  readonly sessionId: string;
  readonly packageId: string;
  readonly packageVersion: string;
  readonly action: ScenarioLifecycleAction;
  readonly expectedState?: ScenarioState;
  readonly expectedRevision: number;
  readonly requestedAt: string;
  readonly reason?: string;
}

export interface ScenarioControlCommand {
  readonly contractId: "D-003";
  readonly contractVersion: "1.0.0";
  readonly operationId: string;
  readonly sessionId: string;
  readonly kind: "logical-time" | "reset" | "fault";
  readonly operation: "set" | "advance" | "execute" | "activate" | "clear";
  readonly target?: string;
  readonly logicalInstant?: string;
  readonly advanceSeconds?: number;
  readonly delayMilliseconds?: number;
  readonly expectedRevision: number;
  readonly requestedAt: string;
}

export interface ScenarioCheckpointEvaluation {
  readonly contractId: "D-004";
  readonly contractVersion: "1.0.0";
  readonly evaluationId: string;
  readonly sessionId: string;
  readonly claimClass:
    | "software-health"
    | "scenario-readiness"
    | "presentation-progress"
    | "business-completion";
  readonly claimId: string;
  readonly result:
    "satisfied" | "not-satisfied" | "pending" | "not-evaluable" | "uncertain";
  readonly sourceContract: string;
  readonly sourceReference?: string;
  readonly observedAt: string;
  readonly reason?: string;
  readonly evidenceReferences: readonly string[];
}

export interface PresentationCapabilityManifest {
  readonly contractId: "P-001";
  readonly contractVersion: "1.0.0";
  readonly manifestId: string;
  readonly manifestVersion: string;
  readonly applicationId: string;
  readonly applicationRelease: string;
  readonly surfaceRoles: readonly ("audience-display" | "reviewer-workbench")[];
  readonly views: readonly {
    readonly viewId: string;
    readonly viewVersion: string;
    readonly contextKeys: readonly string[];
  }[];
  readonly informationProfiles: readonly "synthetic-only"[];
  readonly releasedAt: string;
}

export interface PresentationRegistration {
  readonly contractId: "P-002";
  readonly contractVersion: "1.0.0";
  readonly registrationId: string;
  readonly sessionId: string;
  readonly surfaceSlot: string;
  readonly surfaceRole: "audience-display" | "reviewer-workbench";
  readonly manifestId: string;
  readonly manifestVersion: string;
  readonly supportedViews: readonly string[];
  readonly bindingMode:
    | "pre-session"
    | "external-session"
    | "synthetic-session"
    | "development-assurance";
  readonly registrationRevision: number;
  readonly connectionGeneration: number;
  readonly leaseExpiresAt: string;
}

export interface PresentationCue {
  readonly contractId: "P-003";
  readonly contractVersion: "1.0.0";
  readonly cueId: string;
  readonly cueDigest: string;
  readonly idempotencyKey: string;
  readonly sessionId: string;
  readonly sessionRevision: number;
  readonly surfaceSlot: string;
  readonly registrationId: string;
  readonly registrationRevision: number;
  readonly connectionGeneration: number;
  readonly semanticView: string;
  readonly viewVersion: string;
  readonly context: Readonly<Record<string, string>>;
  readonly issuedAt: string;
  readonly expiresAt: string;
  readonly stageId: string;
  readonly stepId: string;
}

export type PresentationOutcomeResult =
  | "applied"
  | "refused"
  | "unsupported"
  | "expired"
  | "duplicate"
  | "superseded"
  | "failed"
  | "uncertain";

export interface PresentationCueOutcome {
  readonly contractId: "P-004";
  readonly contractVersion: "1.0.0";
  readonly outcomeId: string;
  readonly cueId: string;
  readonly cueDigest: string;
  readonly sessionId: string;
  readonly sessionRevision: number;
  readonly surfaceSlot: string;
  readonly registrationId: string;
  readonly registrationRevision: number;
  readonly connectionGeneration: number;
  readonly semanticView: string;
  readonly result: PresentationOutcomeResult;
  readonly reason?: string;
  readonly diagnosticCode?: string;
  readonly concludedAt: string;
  readonly businessCompletionClaimed: false;
}

export type PrincipalType =
  | "external-human"
  | "synthetic-human"
  | "workload"
  | "operator"
  | "service-owner";

export type InformationLevel =
  "public" | "synthetic" | "internal" | "restricted-security";

export type MessageKind =
  | "business-command"
  | "business-event"
  | "demonstration-control"
  | "trust-command"
  | "presentation-cue"
  | "operational-signal"
  | "analytical-projection";

export interface PrincipalReference {
  readonly principalType: PrincipalType;
  readonly principalId: string;
  readonly environmentId: string;
  readonly issuer: string;
  readonly trustDomain?: string;
}

export interface AuthorityContext {
  readonly contractId: "C-002";
  readonly contractVersion: "1.0.0";
  readonly environmentId: string;
  readonly requester: PrincipalReference;
  readonly actor?: PrincipalReference;
  readonly roles: readonly string[];
  readonly delegatedAuthority: readonly string[];
  readonly purpose: {
    readonly code: string;
    readonly description?: string;
  };
  readonly constraints: {
    readonly engagementIds?: readonly string[];
    readonly demonstrationSessionId?: string;
    readonly targetComponents?: readonly string[];
    readonly informationLevels?: readonly InformationLevel[];
    readonly expiresAt?: string;
  };
  readonly policyVersion: string;
  readonly decisionReference?: string;
}

export interface InteractionEnvelope<Payload extends object = object> {
  readonly contractId: "C-001";
  readonly contractVersion: "1.0.0";
  readonly messageId: string;
  readonly messageType: string;
  readonly messageKind: MessageKind;
  readonly sourceComponent: string;
  readonly targetComponent: string;
  readonly audience: string;
  readonly issuedAt: string;
  readonly occurredAt?: string;
  readonly expiresAt?: string;
  readonly correlationId: string;
  readonly causationId?: string;
  readonly idempotencyKey?: string;
  readonly authority: AuthorityContext;
  readonly classification: {
    readonly level: InformationLevel;
    readonly categories: readonly string[];
    readonly handling: readonly string[];
  };
  readonly security: {
    readonly authenticationContextRef: string;
    readonly integrityReference?: string;
  };
  readonly trace: {
    readonly traceId: string;
    readonly parentSpanId?: string;
  };
  readonly payload: Payload;
}

export type EvidenceKind =
  | "source-version"
  | "processing-record"
  | "audit-record"
  | "decision-record"
  | "rule-execution"
  | "model-execution"
  | "released-artifact"
  | "conformance-result";

export interface EvidenceReference {
  readonly contractId: "C-004";
  readonly contractVersion: "1.0.0";
  readonly evidenceId: string;
  readonly evidenceKind: EvidenceKind;
  readonly ownerComponent: string;
  readonly reference: string;
  readonly createdAt: string;
  readonly classification: InformationLevel;
  readonly mediaType?: string;
  readonly digest?: {
    readonly algorithm: "sha-256";
    readonly value: string;
  };
  readonly sourceVersion?: string;
  readonly retentionClass: string;
  readonly accessPolicyRef: string;
  readonly predecessorEvidenceIds: readonly string[];
}

export type OutcomeStatus =
  "accepted" | "refused" | "expired" | "duplicate" | "failed";

export interface CommandOutcome {
  readonly contractId: "C-003";
  readonly contractVersion: "1.0.0";
  readonly outcomeId: string;
  readonly commandMessageId: string;
  readonly status: OutcomeStatus;
  readonly code: string;
  readonly summary: string;
  readonly retryable: boolean;
  readonly completedAt: string;
  readonly originalOutcomeId?: string;
  readonly recoveryOwner?: string;
  readonly evidence: readonly EvidenceReference[];
}

export type TrustProfile = "local-synthetic" | "managed";
export type EnvironmentClass =
  | "local-scratch"
  | "portable-isolated"
  | "hosted-shared"
  | "production-like"
  | "production";

export interface WorkloadIdentityContext {
  readonly contractId: "I-002";
  readonly contractVersion: "1.0.0";
  readonly contextId: string;
  readonly environmentId: string;
  readonly issuer: string;
  readonly workloadId: string;
  readonly audiences: readonly string[];
  readonly contractActions: readonly string[];
  readonly attestationReference: string;
  readonly policyVersion: string;
  readonly issuedAt: string;
  readonly expiresAt: string;
}

export interface SyntheticTrustBootstrapRecord {
  readonly contractId: "I-003";
  readonly contractVersion: "1.0.0";
  readonly recordId: string;
  readonly environmentId: string;
  readonly environmentClass: EnvironmentClass;
  readonly informationProfile: "synthetic-only" | "non-synthetic-authorised";
  readonly trustProfile: TrustProfile;
  readonly trustDomain: string;
  readonly trustEpoch: number;
  readonly signerId: string;
  readonly publicKeyFingerprint: string;
  readonly keyCustodyClass:
    "local-file" | "managed-service" | "hardware-backed";
  readonly recoveryProfile:
    "rebuild-new-trust-domain" | "protected-same-environment";
  readonly status: "ready" | "not-ready" | "revoked";
  readonly compatible: boolean;
  readonly createdAt: string;
  readonly reasonCode?: string;
}

export interface DemonstrationSignInGrant {
  readonly contractId: "I-004";
  readonly contractVersion: "1.0.0";
  readonly claims: {
    readonly grantId: string;
    readonly establishmentOperationId: string;
    readonly environmentId: string;
    readonly trustDomain: string;
    readonly trustEpoch: number;
    readonly actorId: string;
    readonly applicationId: string;
    readonly audience: string;
    readonly surfaceId: string;
    readonly demonstrationSessionId: string;
    readonly roles: readonly string[];
    readonly purpose: string;
    readonly syntheticRealm: string;
    readonly decisionReference: string;
    readonly issuedAt: string;
    readonly notBefore: string;
    readonly expiresAt: string;
  };
  readonly signature: {
    readonly profile: "ppl-i004-ed25519-v1";
    readonly algorithm: "Ed25519";
    readonly signerId: string;
    readonly publicKeyFingerprint: string;
    readonly value: string;
  };
}

export type SyntheticSessionStatus =
  | "established"
  | "refused"
  | "expired"
  | "replay-detected"
  | "failed"
  | "terminated"
  | "revoked";

export interface SyntheticSessionOutcome {
  readonly contractId: "I-005";
  readonly contractVersion: "1.0.0";
  readonly outcomeId: string;
  readonly grantId: string;
  readonly establishmentOperationId: string;
  readonly environmentId: string;
  readonly applicationId: string;
  readonly surfaceId: string;
  readonly demonstrationSessionId: string;
  readonly actorId: string;
  readonly roles: readonly string[];
  readonly syntheticRealm: string;
  readonly status: SyntheticSessionStatus;
  readonly occurredAt: string;
  readonly maximumValidUntil?: string;
  readonly sessionReference?: string;
  readonly reasonCode?: string;
  readonly decisionReference: string;
  readonly originalOutcomeId?: string;
  readonly evidenceReferences: readonly string[];
}

export type AuthorisationDecisionStatus =
  "permit" | "deny" | "not-applicable" | "indeterminate";

export interface AuthorisationDecision {
  readonly contractId: "AZ-001";
  readonly contractVersion: "1.0.0";
  readonly kind: "decision";
  readonly decisionId: string;
  readonly requestId: string;
  readonly status: AuthorisationDecisionStatus;
  readonly reasonCode: string;
  readonly obligations: readonly {
    readonly code: string;
    readonly value?: string;
  }[];
  readonly policyVersion: string;
  readonly decidedAt: string;
  readonly validUntil?: string;
  readonly evidenceReferences: readonly string[];
}

export function isSyntheticSessionOutcome(
  value: unknown,
): value is SyntheticSessionOutcome {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Partial<SyntheticSessionOutcome>;
  return (
    candidate.contractId === "I-005" &&
    candidate.contractVersion === "1.0.0" &&
    typeof candidate.outcomeId === "string" &&
    [
      "established",
      "refused",
      "expired",
      "replay-detected",
      "failed",
      "terminated",
      "revoked",
    ].includes(candidate.status ?? "")
  );
}

export type ComponentMaturity =
  "repository-skeleton" | "in-development" | "demonstrated";

export interface ComponentCapabilityManifest {
  readonly contractId: "C-005";
  readonly contractVersion: "1.0.0";
  readonly componentId: string;
  readonly componentName: string;
  readonly releaseVersion: string;
  readonly maturity: ComponentMaturity;
  readonly generatedAt: string;
  readonly supportedProfiles: readonly (
    | "local-private"
    | "portable-demonstration"
    | "hosted-demonstrator"
    | "development-assurance"
  )[];
  readonly capabilities: readonly {
    readonly capabilityId: string;
    readonly description: string;
    readonly contracts: readonly {
      readonly contractId: string;
      readonly versions: readonly string[];
    }[];
  }[];
  readonly readinessDependencies: readonly string[];
  readonly evidence: readonly EvidenceReference[];
}

export function isCommandOutcome(value: unknown): value is CommandOutcome {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Partial<CommandOutcome>;
  return (
    candidate.contractId === "C-003" &&
    candidate.contractVersion === "1.0.0" &&
    typeof candidate.outcomeId === "string" &&
    typeof candidate.commandMessageId === "string" &&
    ["accepted", "refused", "expired", "duplicate", "failed"].includes(
      candidate.status ?? "",
    ) &&
    typeof candidate.code === "string" &&
    typeof candidate.retryable === "boolean" &&
    Array.isArray(candidate.evidence)
  );
}
