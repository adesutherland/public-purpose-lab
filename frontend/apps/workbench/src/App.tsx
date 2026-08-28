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
  const [connected, setConnected] = useState(false);
  const [loginMode, setLoginMode] = useState<"local-test" | "google-oidc">(
    "local-test",
  );
  const [sessionId, setSessionId] = useState("");
  const [registration, setRegistration] = useState<PresentationRegistration>();
  const [syntheticActor, setSyntheticActor] = useState<string>();
  const [cue, setCue] = useState<PresentationCue>();
  const [message, setMessage] = useState(
    "Connect the external workbench operator, then bind a synthetic reviewer.",
  );

  useEffect(() => {
    void getJson<{ mode: "local-test" | "google-oidc" }>("/api/v1/login-mode")
      .then((result) => setLoginMode(result.mode))
      .catch(() => undefined);
    void getJson<{
      syntheticStatus: string;
      syntheticActorId?: string;
      registration?: PresentationRegistration;
    }>("/api/v1/session-context")
      .then((context) => {
        setConnected(true);
        if (context.registration?.surfaceSlot === "reviewer-workbench") {
          setRegistration(context.registration);
          setSessionId(context.registration.sessionId);
        }
        if (
          context.syntheticStatus === "established" &&
          context.syntheticActorId
        ) {
          setSyntheticActor(context.syntheticActorId);
          setMessage(
            `Restart-safe workbench binding restored for ${context.syntheticActorId}.`,
          );
        } else {
          setMessage("External workbench-operator session restored.");
        }
      })
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!registration || !syntheticActor) return undefined;
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
  }, [registration, syntheticActor]);

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
      maturityLabel="In-development M3.4 common surface"
      notice="Synthetic development assurance only. M3.4 exercises distinct external and synthetic identities; asset, policy, RAG, workflow and reporting capabilities remain outside this slice."
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
                if (loginMode === "google-oidc") {
                  window.location.assign("/auth/google/start");
                  return;
                }
                await postJson("/api/v1/development-session", {});
                setConnected(true);
                setMessage("Local test workbench operator authenticated.");
              })
            }
          >
            {loginMode === "google-oidc"
              ? "Sign in with Google"
              : "Connect test workbench operator"}
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
            className="ppl-button"
            type="button"
            disabled={!registration || Boolean(syntheticActor)}
            onClick={() =>
              void perform(async () => {
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
                  `Synthetic reviewer ${context.syntheticActorId} is bound; semantic cues are active.`,
                );
              })
            }
          >
            Confirm synthetic reviewer
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
        {syntheticActor && (
          <div className="ppl-runtime-message">
            Synthetic workbench actor <strong>{syntheticActor}</strong>
          </div>
        )}
        <div className="ppl-runtime-message" role="status">
          {message}
        </div>
      </article>
    </SurfaceShell>
  );
}
