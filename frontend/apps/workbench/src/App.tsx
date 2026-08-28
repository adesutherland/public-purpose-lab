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
  const [connected, setConnected] = useState(false);
  const [sessionId, setSessionId] = useState("");
  const [registration, setRegistration] = useState<PresentationRegistration>();
  const [cue, setCue] = useState<PresentationCue>();
  const [message, setMessage] = useState(
    "This shell exercises the common presentation contract only.",
  );

  useEffect(() => {
    if (!registration) return undefined;
    const source = new EventSource("/api/v1/cues", { withCredentials: true });
    source.addEventListener("presentation-cue", ((
      event: MessageEvent<string>,
    ) => {
      const received = JSON.parse(event.data) as PresentationCue;
      if (
        received.sessionId === registration.sessionId &&
        received.surfaceSlot === registration.surfaceSlot
      ) {
        setCue(received);
        setMessage(
          "Workbench semantic cue received; no asset or policy operation was implied.",
        );
      }
    }) as EventListener);
    return () => source.close();
  }, [registration]);

  const perform = async (operation: () => Promise<void>) => {
    try {
      await operation();
    } catch (caught) {
      setMessage(
        caught instanceof Error ? caught.message : "Operation failed safely",
      );
    }
  };

  const applyCue = async () => {
    if (!cue) return;
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
      result:
        new Date(cue.expiresAt).getTime() > Date.now() ? "applied" : "expired",
      concludedAt: new Date().toISOString(),
      businessCompletionClaimed: false,
    };
    await postJson("/api/v1/outcomes", outcome);
    setMessage(
      "Workbench presentation-progress outcome recorded; business completion remains unclaimed.",
    );
  };

  return (
    <SurfaceShell
      surface={surfaceById("UX-02")}
      maturityLabel="In-development common surface"
      notice="M3.3 exercises registration, semantic cue and presentation outcome only. Asset, policy, RAG, workflow and reporting capabilities remain outside this slice."
    >
      <article className="ppl-card ppl-live-card">
        <p className="ppl-card-label">Reviewer-workbench presentation slot</p>
        <label className="ppl-field">
          Demonstration session ID
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
                await postJson("/api/v1/development-session", {});
                setConnected(true);
                setMessage("Local synthetic workbench operator connected.");
              })
            }
          >
            Connect synthetic workbench
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
                  "Reviewer workbench registered and waiting for a semantic cue.",
                );
              })
            }
          >
            Register workbench surface
          </button>
          <button
            className="ppl-button ppl-button-secondary"
            type="button"
            disabled={!cue}
            onClick={() => void perform(applyCue)}
          >
            Apply workbench view
          </button>
        </div>
        {cue && (
          <section className="ppl-semantic-view">
            <p className="ppl-eyebrow">Semantic view: {cue.semanticView}</p>
            <h2>{cue.context.heading}</h2>
            <p>{cue.context.message}</p>
          </section>
        )}
        <div className="ppl-runtime-message" role="status">
          {message}
        </div>
      </article>
    </SurfaceShell>
  );
}
