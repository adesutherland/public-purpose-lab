import { useEffect, useState } from "react";
import type {
  PresentationCue,
  PresentationCueOutcome,
  PresentationRegistration,
} from "@public-purpose-lab/contracts";
import { SurfaceShell, surfaceById } from "@public-purpose-lab/ui";

async function postJson<T>(path: string, body: object): Promise<T> {
  const response = await fetch(path, {
    method: "POST",
    credentials: "same-origin",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const result = (await response.json()) as T & { code?: string };
  if (!response.ok) throw new Error(result.code ?? `HTTP ${response.status}`);
  return result;
}

export function App() {
  const [authenticated, setAuthenticated] = useState(false);
  const [sessionId, setSessionId] = useState("");
  const [registration, setRegistration] = useState<PresentationRegistration>();
  const [cue, setCue] = useState<PresentationCue>();
  const [message, setMessage] = useState(
    "Connect this local synthetic surface, then bind a session.",
  );
  const [error, setError] = useState(false);

  useEffect(() => {
    if (!registration) return undefined;
    const source = new EventSource("/api/v1/cues", { withCredentials: true });
    const receive = (event: MessageEvent<string>) => {
      const received = JSON.parse(event.data) as PresentationCue;
      if (
        received.sessionId === registration.sessionId &&
        received.surfaceSlot === registration.surfaceSlot &&
        received.registrationId === registration.registrationId &&
        received.connectionGeneration === registration.connectionGeneration
      ) {
        setCue(received);
        setMessage(
          `Semantic cue ${received.semanticView} received; application is pending.`,
        );
      }
    };
    source.addEventListener("presentation-cue", receive as EventListener);
    source.onerror = () =>
      setMessage("Cue channel reconnecting; no presentation result inferred.");
    return () => source.close();
  }, [registration]);

  const run = async (operation: () => Promise<void>) => {
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

  const conclude = async () => {
    if (!cue || !registration) return;
    const expired = new Date(cue.expiresAt).getTime() <= Date.now();
    const outcome: PresentationCueOutcome = {
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
      result: expired ? "expired" : "applied",
      reason: expired ? "operational-expiry-passed" : undefined,
      concludedAt: new Date().toISOString(),
      businessCompletionClaimed: false,
    };
    await postJson("/api/v1/outcomes", outcome);
    setMessage(
      expired
        ? "Cue expired under operational time; no view was applied."
        : "Semantic view applied and a presentation-progress outcome was recorded.",
    );
  };

  return (
    <SurfaceShell
      surface={surfaceById("UX-04")}
      maturityLabel="In-development walking skeleton"
      notice="Synthetic development assurance only. An applied view proves presentation state, not human attention or business completion."
    >
      <article className="ppl-card ppl-live-card">
        <p className="ppl-card-label">Audience display binding</p>
        <div className="ppl-controls">
          <label className="ppl-field">
            Demonstration session ID
            <input
              value={sessionId}
              placeholder="session:…"
              onChange={(event) => setSessionId(event.target.value)}
            />
          </label>
        </div>
        <div className="ppl-button-row">
          <button
            className="ppl-button"
            type="button"
            disabled={authenticated}
            onClick={() =>
              void run(async () => {
                await postJson("/api/v1/development-session", {});
                setAuthenticated(true);
                setMessage(
                  "Local synthetic surface operator connected for 30 minutes.",
                );
              })
            }
          >
            Connect synthetic surface
          </button>
          <button
            className="ppl-button"
            type="button"
            disabled={!authenticated || !sessionId || Boolean(registration)}
            onClick={() =>
              void run(async () => {
                const result = await postJson<PresentationRegistration>(
                  "/api/v1/registrations",
                  {
                    sessionId,
                    surfaceSlot: "audience-display",
                    surfaceRole: "audience-display",
                  },
                );
                setRegistration(result);
                setMessage(
                  "Audience display registered; waiting for semantic cues.",
                );
              })
            }
          >
            Register audience display
          </button>
          <button
            className="ppl-button ppl-button-secondary"
            type="button"
            disabled={!cue}
            onClick={() => void run(conclude)}
          >
            Apply current semantic view
          </button>
        </div>

        {registration && (
          <div className="ppl-runtime-message">
            Registration{" "}
            <strong className="ppl-mono">{registration.registrationId}</strong>
            <br />
            Generation {registration.connectionGeneration}; lease ends{" "}
            {registration.leaseExpiresAt}
          </div>
        )}

        {cue?.semanticView === "assurance-welcome" && (
          <section className="ppl-semantic-view" aria-live="polite">
            <p className="ppl-eyebrow">Semantic view: assurance-welcome</p>
            <h2>{cue.context.heading}</h2>
            <p className="ppl-purpose">{cue.context.message}</p>
          </section>
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
