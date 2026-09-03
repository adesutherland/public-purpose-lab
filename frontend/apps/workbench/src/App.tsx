import { useEffect, useState } from "react";
import type {
  PresentationCue,
  PresentationCueOutcome,
  PresentationRegistration,
  SourceIntakeOutcome,
  SourceLifecycleStatus,
  SourceStageOutcome,
} from "@public-purpose-lab/contracts";
import { SurfaceShell, surfaceById } from "@public-purpose-lab/ui";

type WorkbenchView = "wb-engagement" | "wb-source-intake" | "wb-source-status";

interface SessionContext {
  readonly externalPrincipalId: string;
  readonly syntheticStatus: "established" | "not-established";
  readonly syntheticActorId?: string;
  readonly syntheticRoles?: readonly string[];
  readonly maximumValidUntil?: string;
  readonly environmentId: string;
  readonly trustProfile: string;
  readonly registration?: PresentationRegistration;
}

async function postJson<T>(path: string, body: object): Promise<T> {
  const csrf = csrfToken();
  const response = await fetch(path, {
    method: "POST",
    credentials: "same-origin",
    headers: {
      "Content-Type": "application/json",
      ...(csrf ? { "X-PPL-CSRF": csrf } : {}),
    },
    body: JSON.stringify(body),
  });
  const result = (await response.json()) as T & { code?: string };
  if (!response.ok) throw new Error(result.code ?? `HTTP ${response.status}`);
  return result;
}

async function getJson<T>(path: string): Promise<T> {
  const response = await fetch(path, { credentials: "same-origin" });
  const result = (await response.json()) as T & { code?: string };
  if (!response.ok) throw new Error(result.code ?? `HTTP ${response.status}`);
  return result;
}

function csrfToken(): string | undefined {
  return document.cookie
    .split(";")
    .map((value) => value.trim())
    .find((value) => value.startsWith("PPL_CSRF="))
    ?.slice("PPL_CSRF=".length);
}

function outcomeFor(
  cue: PresentationCue,
  result: PresentationCueOutcome["result"],
  reason?: string,
): PresentationCueOutcome {
  return {
    contractId: "P-004",
    contractVersion: "1.0.0",
    outcomeId: `outcome:${crypto.randomUUID()}`,
    cueId: cue.cueId,
    cueDigest: cue.cueDigest,
    sessionId: cue.sessionId,
    sessionRevision: cue.sessionRevision,
    surfaceSlot: cue.surfaceSlot,
    registrationId: cue.registrationId,
    registrationRevision: cue.registrationRevision,
    connectionGeneration: cue.connectionGeneration,
    semanticView: cue.semanticView,
    result,
    reason,
    concludedAt: new Date().toISOString(),
    businessCompletionClaimed: false,
  };
}

export function App() {
  const [connected, setConnected] = useState(false);
  const [loginMode, setLoginMode] = useState<"local-test" | "google-oidc">(
    "local-test",
  );
  const [sessionId, setSessionId] = useState("");
  const [registration, setRegistration] = useState<PresentationRegistration>();
  const [context, setContext] = useState<SessionContext>();
  const [activeView, setActiveView] = useState<WorkbenchView>();
  const [engagementReference, setEngagementReference] = useState(
    "engagement:harbour-support-review",
  );
  const [sourceMode, setSourceMode] = useState<"upload" | "paste">("paste");
  const [sourceText, setSourceText] = useState("");
  const [sourceName, setSourceName] = useState<string>();
  const [sourceMediaType, setSourceMediaType] = useState("text/plain");
  const [sourceTitle, setSourceTitle] = useState("Harbour support policy");
  const [sourceOwner, setSourceOwner] = useState("Harbour Community Support");
  const [sourceRights, setSourceRights] = useState(
    "Synthetic demonstration fixture",
  );
  const [sourceProvenance, setSourceProvenance] = useState(
    "Created for the Gate C demonstration",
  );
  const [syntheticConfirmed, setSyntheticConfirmed] = useState(false);
  const [sourceOutcome, setSourceOutcome] = useState<SourceIntakeOutcome>();
  const [sourceLifecycle, setSourceLifecycle] =
    useState<SourceLifecycleStatus>();
  const [message, setMessage] = useState(
    "Connect the Workbench, register it to the demonstration, then ask the Director to assign synthetic-reviewer.",
  );
  const [error, setError] = useState(false);

  const refreshContext = async () => {
    const next = await getJson<SessionContext>("/api/v1/session-context");
    setConnected(true);
    setContext(next);
    if (next.registration?.surfaceSlot === "reviewer-workbench") {
      setRegistration(next.registration);
      setSessionId(next.registration.sessionId);
    }
    return next;
  };

  useEffect(() => {
    void getJson<{ mode: "local-test" | "google-oidc" }>("/api/v1/login-mode")
      .then((result) => setLoginMode(result.mode))
      .catch(() => undefined);
    void refreshContext().catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!connected || !registration) return undefined;
    const interval = window.setInterval(() => {
      void refreshContext()
        .then((next) => {
          if (
            context?.syntheticStatus === "established" &&
            next.syntheticStatus !== "established"
          ) {
            setActiveView(undefined);
            setMessage(
              "Synthetic reviewer session terminated; the external Workbench operator remains signed in.",
            );
          }
        })
        .catch(() => undefined);
    }, 2000);
    return () => window.clearInterval(interval);
  }, [connected, context?.syntheticStatus, registration]);

  useEffect(() => {
    if (
      !registration ||
      context?.syntheticStatus !== "established" ||
      !context.syntheticActorId
    ) {
      return undefined;
    }
    const source = new EventSource("/api/v1/cues", { withCredentials: true });
    const receive = (event: MessageEvent<string>) => {
      const cue = JSON.parse(event.data) as PresentationCue;
      if (
        cue.sessionId !== registration.sessionId ||
        cue.surfaceSlot !== registration.surfaceSlot ||
        cue.registrationId !== registration.registrationId ||
        cue.connectionGeneration !== registration.connectionGeneration
      ) {
        return;
      }
      void (async () => {
        const supported = [
          "wb-engagement",
          "wb-source-intake",
          "wb-source-status",
        ].includes(cue.semanticView);
        const expired = new Date(cue.expiresAt).getTime() <= Date.now();
        const result = !supported
          ? "unsupported"
          : expired
            ? "expired"
            : "applied";
        await postJson(
          "/api/v1/outcomes",
          outcomeFor(
            cue,
            result,
            !supported
              ? "semantic-view-unsupported"
              : expired
                ? "operational-expiry-passed"
                : undefined,
          ),
        );
        if (result === "applied") {
          setActiveView(cue.semanticView as WorkbenchView);
          setEngagementReference(
            cue.context.syntheticReference ??
              "engagement:harbour-support-review",
          );
          setMessage(
            `${cue.semanticView} resolved by the Workbench. This records presentation progress only.`,
          );
        } else {
          setError(true);
          setMessage(`${cue.semanticView} visibly refused: ${result}.`);
        }
      })().catch((caught) => {
        setError(true);
        setMessage(
          caught instanceof Error ? caught.message : "View resolution failed",
        );
      });
    };
    source.addEventListener("presentation-cue", receive as EventListener);
    source.onerror = () =>
      setMessage(
        "Semantic cue channel reconnecting; no business result inferred.",
      );
    return () => source.close();
  }, [context?.syntheticActorId, context?.syntheticStatus, registration]);

  const perform = async (operation: () => Promise<void>) => {
    try {
      setError(false);
      await operation();
    } catch (caught) {
      setError(true);
      setMessage(
        caught instanceof Error ? caught.message : "Operation failed safely",
      );
    }
  };

  const syntheticEstablished = context?.syntheticStatus === "established";

  return (
    <SurfaceShell
      surface={surfaceById("UX-02")}
      maturityLabel="Gate C · source validation and staging in development"
      notice="Synthetic demonstration only. This Gate C slice can quarantine and visibly validate text, then request reviewer-controlled staging through AUT-01. It does not yet index, retrieve or answer from that source."
    >
      <article className="ppl-card ppl-live-card">
        <p className="ppl-card-label">Reviewer Workbench binding</p>
        <label className="ppl-field">
          Demonstration Session ID
          <input
            value={sessionId}
            onChange={(event) => setSessionId(event.target.value)}
            placeholder="session:…"
          />
        </label>
        <div className="ppl-button-row">
          <button
            className="ppl-button"
            type="button"
            disabled={connected}
            onClick={() =>
              void perform(async () => {
                if (loginMode === "google-oidc") {
                  window.location.assign("/auth/google/start");
                  return;
                }
                await postJson("/api/v1/development-session", {});
                await refreshContext();
                setMessage("Local test Workbench operator authenticated.");
              })
            }
          >
            {loginMode === "google-oidc"
              ? "Sign in with Google"
              : "Connect test Workbench operator"}
          </button>
          <button
            className="ppl-button"
            type="button"
            disabled={!connected || !sessionId || Boolean(registration)}
            onClick={() =>
              void perform(async () => {
                const value = await postJson<PresentationRegistration>(
                  "/api/v1/registrations",
                  {
                    sessionId,
                    surfaceSlot: "reviewer-workbench",
                    surfaceRole: "reviewer-workbench",
                  },
                );
                setRegistration(value);
                setMessage(
                  "Workbench registered. Ask the Director to assign synthetic-reviewer.",
                );
              })
            }
          >
            Register Workbench
          </button>
          <button
            className="ppl-button ppl-button-secondary"
            type="button"
            disabled={!registration}
            onClick={() =>
              void perform(async () => {
                const next = await refreshContext();
                setMessage(
                  next.syntheticStatus === "established"
                    ? `Synthetic reviewer ${next.syntheticActorId} is established.`
                    : "Synthetic reviewer is not established yet.",
                );
              })
            }
          >
            Refresh identity
          </button>
        </div>

        <div className="ppl-status-grid ppl-runtime-message">
          <span>
            Actor
            <br />
            <strong>
              {context?.syntheticActorId ?? "no synthetic actor assigned"}
            </strong>
          </span>
          <span>
            Role
            <br />
            <strong>{context?.syntheticRoles?.join(", ") ?? "—"}</strong>
          </span>
          <span>
            Trust
            <br />
            <strong>{context?.trustProfile ?? "—"}</strong>
          </span>
          <span>
            Valid until
            <br />
            <strong>{context?.maximumValidUntil ?? "—"}</strong>
          </span>
        </div>
        <p className="ppl-mono">
          {context?.environmentId ?? "environment pending"} ·{" "}
          {registration?.sessionId ?? "session pending"}
        </p>
      </article>

      <article className="ppl-card ppl-live-card">
        <p className="ppl-card-label">Ordinary accessible navigation</p>
        <div className="ppl-button-row" role="navigation">
          <button
            className="ppl-button ppl-button-secondary"
            type="button"
            disabled={!syntheticEstablished}
            onClick={() => {
              setActiveView("wb-engagement");
              setMessage(
                "Manual navigation opened WB-ENGAGEMENT; no business event was emitted.",
              );
            }}
          >
            Engagement
          </button>
          <button
            className="ppl-button ppl-button-secondary"
            type="button"
            disabled={!syntheticEstablished}
            onClick={() => {
              setActiveView("wb-source-intake");
              setMessage(
                "Manual navigation opened WB-SOURCE-INTAKE; no business event was emitted.",
              );
            }}
          >
            Source intake
          </button>
          <button
            className="ppl-button ppl-button-secondary"
            type="button"
            disabled={!syntheticEstablished || !sourceOutcome}
            onClick={() => setActiveView("wb-source-status")}
          >
            Source status
          </button>
        </div>

        {activeView === "wb-engagement" && (
          <section className="ppl-semantic-view" aria-live="polite">
            <p className="ppl-eyebrow">WB-ENGAGEMENT</p>
            <h2>Harbour support policy review</h2>
            <p className="ppl-purpose">
              Review how a synthetic community-support organisation can govern
              policy sources before producing evidence-bounded guidance.
            </p>
            <div className="ppl-status-grid ppl-runtime-message">
              <span>
                Reference
                <br />
                <strong className="ppl-mono">{engagementReference}</strong>
              </span>
              <span>
                Authority
                <br />
                <strong>Human reviewer retains decision authority</strong>
              </span>
              <span>
                Classification
                <br />
                <strong>Synthetic only</strong>
              </span>
              <span>
                Status
                <br />
                <strong>Context available; no engagement created</strong>
              </span>
            </div>
          </section>
        )}

        {activeView === "wb-source-intake" && (
          <section className="ppl-semantic-view" aria-live="polite">
            <p className="ppl-eyebrow">WB-SOURCE-INTAKE</p>
            <h2>Add a source for governed review</h2>
            <p>
              Context{" "}
              <strong className="ppl-mono">{engagementReference}</strong>
            </p>
            <div className="ppl-controls">
              <label className="ppl-field">
                Upload a small text or Markdown file
                <input
                  type="file"
                  accept=".txt,.md,text/plain,text/markdown"
                  onChange={(event) => {
                    const file = event.target.files?.[0];
                    if (!file) return;
                    void perform(async () => {
                      const text = await file.text();
                      setSourceMode("upload");
                      setSourceText(text);
                      setSourceName(file.name);
                      setSourceMediaType(
                        file.type === "text/markdown"
                          ? "text/markdown"
                          : "text/plain",
                      );
                      if (!sourceTitle.trim()) setSourceTitle(file.name);
                      setMessage(
                        `${file.name} loaded locally for review; it has not yet been submitted.`,
                      );
                    });
                  }}
                />
              </label>
              <label className="ppl-field">
                Link a source · later Gate C slice
                <input
                  type="url"
                  placeholder="Remote retrieval is deliberately unavailable"
                  disabled
                />
              </label>
              <label className="ppl-field">
                Or paste synthetic text
                <textarea
                  rows={6}
                  value={sourceText}
                  placeholder="Paste synthetic policy text…"
                  onChange={(event) => {
                    setSourceMode("paste");
                    setSourceName(undefined);
                    setSourceMediaType("text/plain");
                    setSourceText(event.target.value);
                  }}
                />
              </label>
              <label className="ppl-field">
                Title
                <input
                  value={sourceTitle}
                  onChange={(event) => setSourceTitle(event.target.value)}
                />
              </label>
              <label className="ppl-field">
                Source owner
                <input
                  value={sourceOwner}
                  onChange={(event) => setSourceOwner(event.target.value)}
                />
              </label>
              <label className="ppl-field">
                Rights
                <input
                  value={sourceRights}
                  onChange={(event) => setSourceRights(event.target.value)}
                />
              </label>
              <label className="ppl-field">
                Provenance
                <textarea
                  rows={3}
                  value={sourceProvenance}
                  onChange={(event) => setSourceProvenance(event.target.value)}
                />
              </label>
            </div>
            <div className="ppl-runtime-message">
              <strong>Submission preview</strong>
              <br />
              {sourceName ?? "Pasted text"} · {sourceMediaType} ·{" "}
              {new Blob([sourceText]).size} bytes
              <br />
              {sourceText.slice(0, 180) || "No source text loaded."}
              {sourceText.length > 180 ? "…" : ""}
            </div>
            <label className="ppl-field">
              <span>
                <input
                  type="checkbox"
                  checked={syntheticConfirmed}
                  onChange={(event) =>
                    setSyntheticConfirmed(event.target.checked)
                  }
                />{" "}
                I confirm this source contains synthetic demonstration data
                only.
              </span>
            </label>
            <div className="ppl-button-row">
              <button
                className="ppl-button"
                type="button"
                disabled={
                  !syntheticEstablished ||
                  !syntheticConfirmed ||
                  !sourceText.trim() ||
                  !sourceTitle.trim() ||
                  !sourceOwner.trim() ||
                  !sourceRights.trim() ||
                  !sourceProvenance.trim()
                }
                onClick={() =>
                  void perform(async () => {
                    const submissionId = crypto.randomUUID();
                    const outcome = await postJson<SourceIntakeOutcome>(
                      "/api/v1/source-intake",
                      {
                        submissionId,
                        idempotencyKey: `source-intake:${submissionId}`,
                        source: {
                          acquisitionMode: sourceMode,
                          ...(sourceName ? { originalName: sourceName } : {}),
                          mediaType: sourceMediaType,
                          sizeBytes: new Blob([sourceText]).size,
                          content: sourceText,
                          title: sourceTitle,
                          owner: sourceOwner,
                          rights: sourceRights,
                          provenance: sourceProvenance,
                          classification: "synthetic",
                        },
                      },
                    );
                    setSourceOutcome(outcome);
                    const sourceVersionId =
                      outcome.sourceVersion?.sourceVersionId;
                    setSourceLifecycle(
                      sourceVersionId
                        ? await getJson<SourceLifecycleStatus>(
                            `/api/v1/source-status/${encodeURIComponent(sourceVersionId)}`,
                          )
                        : undefined,
                    );
                    setActiveView("wb-source-status");
                    setMessage(
                      `${outcome.sourceVersion?.sourceVersionId ?? outcome.commandId} recorded as ${outcome.status}.`,
                    );
                  })
                }
              >
                Submit to quarantine
              </button>
            </div>
            <p className="ppl-runtime-message">
              Submission creates a quarantined, immutable first version and
              metadata-only lifecycle events. It does not approve or process the
              source.
            </p>
          </section>
        )}

        {activeView === "wb-source-status" && sourceOutcome && (
          <section className="ppl-semantic-view" aria-live="polite">
            <p className="ppl-eyebrow">WB-SOURCE-STATUS</p>
            <h2>Source validation and controlled staging</h2>
            <p className="ppl-purpose">
              The source remains isolated from knowledge processing until its
              deterministic checks pass and this reviewer requests an AUT-01
              policy decision. Human authority is unchanged.
            </p>
            <div className="ppl-status-grid ppl-runtime-message">
              <span>
                Intake status
                <br />
                <strong>{sourceOutcome.status}</strong>
              </span>
              <span>
                Lifecycle status
                <br />
                <strong>
                  {sourceLifecycle?.lifecycleStatus ?? "unavailable"}
                </strong>
              </span>
              <span>
                Version
                <br />
                <strong>{sourceOutcome.sourceVersion?.version ?? "—"}</strong>
              </span>
              <span>
                Source version
                <br />
                <strong className="ppl-mono">
                  {sourceOutcome.sourceVersion?.sourceVersionId ?? "—"}
                </strong>
              </span>
              <span>
                Actor
                <br />
                <strong>{sourceOutcome.actorId}</strong>
              </span>
              <span>
                Digest
                <br />
                <strong className="ppl-mono">
                  {sourceOutcome.sourceVersion?.digestValue ?? "—"}
                </strong>
              </span>
              <span>
                Correlation
                <br />
                <strong className="ppl-mono">
                  {sourceOutcome.correlationId}
                </strong>
              </span>
            </div>
            {sourceLifecycle && (
              <div className="ppl-runtime-message">
                <strong>Bounded validation checks</strong>
                <ul>
                  {sourceLifecycle.validation.checks.map((check) => (
                    <li key={check.checkId}>
                      {check.checkId}: <strong>{check.status}</strong>
                      {check.reasonCode ? ` · ${check.reasonCode}` : ""}
                    </li>
                  ))}
                </ul>
                Digest verified:{" "}
                {String(sourceLifecycle.validation.digestVerified)}
              </div>
            )}
            {sourceLifecycle?.staging && (
              <div className="ppl-runtime-message">
                <strong>Staging decision</strong>
                <br />
                {sourceLifecycle.staging.status} by{" "}
                {sourceLifecycle.staging.actorId}
                {" · "}
                {sourceLifecycle.staging.reasonCode}
                <br />
                Policy decision:{" "}
                {sourceLifecycle.staging.policyDecisionReference}
              </div>
            )}
            <div className="ppl-button-row">
              <button
                className="ppl-button"
                type="button"
                disabled={sourceLifecycle?.lifecycleStatus !== "validated"}
                onClick={() =>
                  void perform(async () => {
                    if (!sourceLifecycle) return;
                    const requestId = crypto.randomUUID();
                    const outcome = await postJson<SourceStageOutcome>(
                      "/api/v1/source-stage",
                      {
                        requestId,
                        idempotencyKey: `source-stage:${requestId}`,
                        sourceVersionId: sourceLifecycle.sourceVersionId,
                      },
                    );
                    setSourceLifecycle(outcome.sourceStatus);
                    setMessage(
                      `${outcome.sourceStatus.sourceVersionId} ${outcome.status} through ${outcome.sourceStatus.staging?.policyDecisionReference ?? "a refused policy decision"}.`,
                    );
                  })
                }
              >
                Release validated source to staging
              </button>
            </div>
            <p className="ppl-runtime-message">
              Events: {sourceOutcome.eventTypes.join(", ")}. Source content is
              intentionally absent from lifecycle, policy and event views.
            </p>
          </section>
        )}

        {!activeView && (
          <p className="ppl-runtime-message">
            Waiting for an admitted semantic view or ordinary user navigation.
          </p>
        )}
        <div
          className="ppl-runtime-message"
          data-status={error ? "error" : "ok"}
          role="status"
        >
          {message}
        </div>
      </article>
    </SurfaceShell>
  );
}
