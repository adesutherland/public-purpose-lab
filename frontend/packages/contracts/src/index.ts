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
