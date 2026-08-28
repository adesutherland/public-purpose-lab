import { useEffect, useState } from "react";
import type {
  PresentationCue,
  PresentationCueOutcome,
  PresentationRegistration,
} from "@public-purpose-lab/contracts";
import { SurfaceShell, surfaceById } from "@public-purpose-lab/ui";

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

export function App() {
  const [authenticated, setAuthenticated] = useState(false);
  const [loginMode, setLoginMode] = useState<"local-test" | "google-oidc">(
    "local-test",
  );
  const [sessionId, setSessionId] = useState("");
  const [registration, setRegistration] = useState<PresentationRegistration>();
  const [syntheticActor, setSyntheticActor] = useState<string>();
  const [cue, setCue] = useState<PresentationCue>();
  const [message, setMessage] = useState(
    "Connect the external surface operator, then bind a synthetic demonstration session.",
  );
  const [error, setError] = useState(false);

  useEffect(() => {
    void getJson<{ mode: "local-test" | "google-oidc" }>("/api/v1/login-mode")
      .then((result) => setLoginMode(result.mode))
      .catch(() => undefined);
    void getJson<{
      externalPrincipalId: string;
      syntheticStatus: string;
      syntheticActorId?: string;
      registration?: PresentationRegistration;
    }>("/api/v1/session-context")
      .then((context) => {
        setAuthenticated(true);
        if (context.registration) {
          setRegistration(context.registration);
          setSessionId(context.registration.sessionId);
        }
        if (context.syntheticStatus === "established") {
          setSyntheticActor(context.syntheticActorId);
          setMessage(
            `Restart-safe session restored for ${context.syntheticActorId ?? "the synthetic actor"}.`,
          );
        } else {
          setMessage("External surface-operator session restored.");
        }
      })
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!registration || !syntheticActor) return undefined;
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
  }, [registration, syntheticActor]);

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
                if (loginMode === "google-oidc") {
                  window.location.assign("/auth/google/start");
                  return;
                }
                await postJson("/api/v1/development-session", {});
                setAuthenticated(true);
                setMessage(
                  "Local test surface operator authenticated for 30 minutes.",
                );
              })
            }
          >
            {loginMode === "google-oidc"
              ? "Sign in with Google"
              : "Connect test surface operator"}
          </button>
          <button
            className="ppl-button"
            type="button"
            disabled={!registration || Boolean(syntheticActor)}
            onClick={() =>
              void run(async () => {
                const context = await getJson<{
                  syntheticStatus: string;
                  syntheticActorId?: string;
                }>("/api/v1/session-context");
                if (
                  context.syntheticStatus !== "established" ||
                  !context.syntheticActorId
                ) {
                  throw new Error("synthetic-session-not-established");
                }
                setSyntheticActor(context.syntheticActorId);
                setMessage(
                  `Synthetic actor ${context.syntheticActorId} is bound; the semantic cue channel is active.`,
                );
              })
            }
          >
            Confirm synthetic sign-in
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

        {syntheticActor && (
          <div className="ppl-runtime-message">
            Synthetic application actor <strong>{syntheticActor}</strong>
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
