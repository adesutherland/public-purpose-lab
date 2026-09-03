import { describe, expect, it } from "vitest";
import {
  commonContractVersions,
  gateCContractVersions,
  isSyntheticSessionOutcome,
  isCommandOutcome,
  m2ContractVersions,
  type CommandOutcome,
  type ComponentCapabilityManifest,
  type InteractionEnvelope,
} from "./index.ts";

describe("common contract consumption", () => {
  it("publishes the complete M1 version set", () => {
    expect(Object.keys(commonContractVersions)).toEqual([
      "C-001",
      "C-002",
      "C-003",
      "C-004",
      "C-005",
      "C-006",
    ]);
  });

  it("represents an assurance envelope without treating it as authentication", () => {
    const envelope: InteractionEnvelope<{ readonly probe: string }> = {
      contractId: "C-001",
      contractVersion: "1.0.0",
      messageId: "message-typescript-test-001",
      messageType: "ppl.interaction.conformance-probe.command",
      messageKind: "demonstration-control",
      sourceComponent: "CTL-01",
      targetComponent: "INT-01",
      audience: "INT-01",
      issuedAt: "2030-08-25T12:00:00Z",
      expiresAt: "2030-08-25T12:05:00Z",
      correlationId: "correlation-typescript-test-001",
      idempotencyKey: "idempotency-typescript-test-001",
      authority: {
        contractId: "C-002",
        contractVersion: "1.0.0",
        environmentId: "env-local-001",
        requester: {
          principalType: "workload",
          principalId: "workload-framework-host",
          environmentId: "env-local-001",
          issuer: "issuer-local-assurance",
        },
        roles: ["interaction-assurance"],
        delegatedAuthority: ["C-001:submit"],
        purpose: { code: "assurance.conformance" },
        constraints: {
          targetComponents: ["INT-01"],
          informationLevels: ["synthetic"],
        },
        policyVersion: "0.1.0",
      },
      classification: {
        level: "synthetic",
        categories: ["conformance-evidence"],
        handling: ["environment-bound"],
      },
      security: { authenticationContextRef: "assurance-reference-only" },
      trace: { traceId: "trace-typescript-test-001" },
      payload: { probe: "typescript-consumption" },
    };

    expect(envelope.authority.requester.principalType).toBe("workload");
    expect(envelope.security.authenticationContextRef).toContain("reference");
  });

  it("narrows safe command outcomes", () => {
    const outcome: CommandOutcome = {
      contractId: "C-003",
      contractVersion: "1.0.0",
      outcomeId: "outcome-typescript-test-001",
      commandMessageId: "message-typescript-test-001",
      status: "refused",
      code: "authority_insufficient",
      summary: "The authority context did not permit the operation.",
      retryable: false,
      completedAt: "2030-08-25T12:00:01Z",
      evidence: [],
    };

    expect(isCommandOutcome(outcome)).toBe(true);
    expect(isCommandOutcome({ ...outcome, status: "successful" })).toBe(false);
  });

  it("keeps in-development distinct from demonstrated", () => {
    const manifest = {
      maturity: "in-development",
    } satisfies Pick<ComponentCapabilityManifest, "maturity">;

    expect(manifest.maturity).not.toBe("demonstrated");
  });

  it("publishes the complete M2 identity and authorisation version set", () => {
    expect(Object.keys(m2ContractVersions)).toEqual([
      "I-001",
      "I-002",
      "I-003",
      "I-004",
      "I-005",
      "AZ-001",
    ]);
  });

  it("publishes the bounded Gate C source and processing versions", () => {
    expect(gateCContractVersions).toEqual({
      "A-001": "0.1.0",
      "A-002": "0.1.0",
      "K-001": "0.1.0",
    });
  });

  it("narrows synthetic session outcomes without treating an unknown state as established", () => {
    const outcome = {
      contractId: "I-005",
      contractVersion: "1.0.0",
      outcomeId: "session-outcome-typescript-001",
      status: "established",
    };
    expect(isSyntheticSessionOutcome(outcome)).toBe(true);
    expect(isSyntheticSessionOutcome({ ...outcome, status: "logged-in" })).toBe(
      false,
    );
  });
});
