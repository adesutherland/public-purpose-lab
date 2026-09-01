import { useCallback, useEffect, useState } from "react";
import type { ScenarioState } from "@public-purpose-lab/contracts";
import { SurfaceShell, surfaceById } from "@public-purpose-lab/ui";

interface SessionSnapshot {
  readonly sessionId: string;
  readonly state: ScenarioState;
  readonly revision: number;
  readonly logicalTime: string;
  readonly logicalTimeInitialised: boolean;
}

interface PresenterContext {
  readonly externalPrincipalId: string;
  readonly roles: readonly string[];
  readonly expiresAt: string;
}

interface CatalogueEntry {
  readonly title: string;
  readonly purpose: string;
  readonly maturity: string;
  readonly estimatedDuration: string;
  readonly actors: readonly string[];
  readonly requiredComponents: readonly string[];
  readonly status: "ready" | "degraded" | "unavailable";
  readonly limitations: readonly string[];
}

interface EnvironmentSnapshot {
  readonly environmentId: string;
  readonly runtimeProfile: string;
  readonly trustProfile: string;
  readonly trustDescription: string;
  readonly presentationSurfaceUrl: string | null;
  readonly workbenchSurfaceUrl: string | null;
  readonly componentReadinessUrl: string | null;
  readonly catalogue: readonly CatalogueEntry[];
}

async function requestJson<T>(path: string, body?: object): Promise<T> {
  const csrf = document.cookie
    .split(";")
    .map((value) => value.trim())
    .find((value) => value.startsWith("PPL_CSRF="))
    ?.slice("PPL_CSRF=".length);
  const response = await fetch(path, {
    method: body ? "POST" : "GET",
    credentials: "same-origin",
    headers: body
      ? {
          "Content-Type": "application/json",
          ...(csrf ? { "X-PPL-CSRF": csrf } : {}),
        }
      : undefined,
    body: body ? JSON.stringify(body) : undefined,
  });
  const result = (await response.json()) as T & { code?: string };
  if (!response.ok) throw new Error(result.code ?? `HTTP ${response.status}`);
  return result;
}

export function App() {
  const [loginMode, setLoginMode] = useState<"local-test" | "google-oidc">(
    "local-test",
  );
  const [presenter, setPresenter] = useState<PresenterContext>();
  const [environment, setEnvironment] = useState<EnvironmentSnapshot>();
  const [session, setSession] = useState<SessionSnapshot>();
  const [message, setMessage] = useState(
    "Sign in to inspect the environment and admitted demonstration.",
  );
  const [error, setError] = useState(false);

  const run = useCallback(async (operation: () => Promise<void>) => {
    try {
      setError(false);
      await operation();
    } catch (caught) {
      setError(true);
      setMessage(
        caught instanceof Error ? caught.message : "Operation failed safely",
      );
    }
  }, []);

  const restore = useCallback(async () => {
    const [nextPresenter, nextEnvironment] = await Promise.all([
      requestJson<PresenterContext>("/api/v1/session-context"),
      requestJson<EnvironmentSnapshot>("/api/v1/environment"),
    ]);
    setPresenter(nextPresenter);
    setEnvironment(nextEnvironment);
    const restored = window.sessionStorage.getItem(
      "ppl-current-demonstration-session",
    );
    if (restored) {
      const status = await requestJson<{ session: SessionSnapshot }>(
        `/api/v1/status/${encodeURIComponent(restored)}`,
      );
      setSession(status.session);
      setMessage(
        `Presenter and demonstration session restored at revision ${status.session.revision}.`,
      );
    } else {
      setMessage(
        "Environment evaluated. Select the admitted scenario to begin.",
      );
    }
  }, []);

  useEffect(() => {
    void requestJson<{ mode: "local-test" | "google-oidc" }>(
      "/api/v1/login-mode",
    )
      .then((result) => setLoginMode(result.mode))
      .catch(() => undefined);
    void restore().catch(() => undefined);
  }, [restore]);

  const refresh = useCallback(
    async (sessionId = session?.sessionId) => {
      if (!sessionId) return undefined;
      const status = await requestJson<{ session: SessionSnapshot }>(
        `/api/v1/status/${encodeURIComponent(sessionId)}`,
      );
      setSession(status.session);
      window.sessionStorage.setItem(
        "ppl-current-demonstration-session",
        status.session.sessionId,
      );
      return status.session;
    },
    [session?.sessionId],
  );

  const lifecycle = (action: string) =>
    run(async () => {
      if (!session) return;
      const result = await requestJson<{
        session?: SessionSnapshot;
        successor?: SessionSnapshot;
      }>(
        `/api/v1/sessions/${encodeURIComponent(session.sessionId)}/lifecycle`,
        {
          action,
          expectedState: session.state,
          expectedRevision: session.revision,
          reason: `Presenter requested ${action} in Gate B.`,
        },
      );
      const next = result.successor ?? result.session;
      if (next) {
        setSession(next);
        window.sessionStorage.setItem(
          "ppl-current-demonstration-session",
          next.sessionId,
        );
      }
      setMessage(
        result.successor
          ? `Reset created successor ${result.successor.sessionId}; the presenter remains signed in.`
          : `Scenario ${action} accepted and recorded.`,
      );
    });

  const prepare = () =>
    run(async () => {
      if (!session) return;
      let revision = session.revision;
      if (!session.logicalTimeInitialised) {
        const time = await requestJson<{
          sessionRevision: number;
          logicalTime: string;
        }>(
          `/api/v1/sessions/${encodeURIComponent(session.sessionId)}/logical-time`,
          {
            operation: "set",
            expectedRevision: revision,
            logicalInstant: "2030-01-01T09:00:00Z",
          },
        );
        revision = time.sessionRevision;
      }
      await requestJson(
        `/api/v1/sessions/${encodeURIComponent(session.sessionId)}/lifecycle`,
        {
          action: "prepare",
          expectedState: "preparing",
          expectedRevision: revision,
          reason: "Prepare the approved Gate B scenario.",
        },
      );
      const next = await refresh(session.sessionId);
      setMessage(
        `Scenario prepared${next ? ` at revision ${next.revision}` : ""}. Register the two target surfaces next.`,
      );
    });

  const requestView = (
    surfaceSlot: "audience-display" | "reviewer-workbench",
    semanticView: string,
    heading: string,
    detail: string,
  ) =>
    run(async () => {
      if (!session) return;
      await requestJson(
        `/api/v1/sessions/${encodeURIComponent(session.sessionId)}/cue`,
        {
          surfaceSlot,
          semanticView,
          heading,
          message: detail,
          expiresInSeconds: 90,
        },
      );
      setMessage(
        `${semanticView} requested by semantic event; the target application owns resolution and outcome.`,
      );
    });

  const scenario = environment?.catalogue[0];
  const running = session?.state === "running";

  return (
    <SurfaceShell
      surface={surfaceById("UX-03")}
      maturityLabel="Gate B · functional demonstration"
      notice="Synthetic development demonstration only. The Director sequences presentation state; it cannot claim a business action or compliance outcome."
    >
      <article className="ppl-card ppl-live-card">
        <p className="ppl-card-label">DIR-ENVIRONMENT · sign-in and trust</p>
        <div className="ppl-button-row">
          <button
            className="ppl-button"
            type="button"
            disabled={Boolean(presenter)}
            onClick={() =>
              void run(async () => {
                if (loginMode === "google-oidc") {
                  window.location.assign("/auth/google/start");
                  return;
                }
                await requestJson("/api/v1/development-session", {});
                await restore();
              })
            }
          >
            {loginMode === "google-oidc"
              ? "Sign in with Google"
              : "Connect local test presenter"}
          </button>
          {environment?.componentReadinessUrl && (
            <a
              className="ppl-button ppl-button-secondary"
              href={environment.componentReadinessUrl}
            >
              Open OPS-COMPONENTS
            </a>
          )}
        </div>
        <div className="ppl-status-grid ppl-runtime-message">
          <span>
            Presenter
            <br />
            <strong>{presenter?.externalPrincipalId ?? "not signed in"}</strong>
          </span>
          <span>
            Environment
            <br />
            <strong className="ppl-mono">
              {environment?.environmentId ?? "not evaluated"}
            </strong>
          </span>
          <span>
            Runtime
            <br />
            <strong>{environment?.runtimeProfile ?? "—"}</strong>
          </span>
          <span>
            Trust
            <br />
            <strong>{environment?.trustProfile ?? "—"}</strong>
          </span>
        </div>
        {environment && (
          <p className="ppl-runtime-message">{environment.trustDescription}</p>
        )}
      </article>

      <article className="ppl-card ppl-live-card">
        <p className="ppl-card-label">DIR-CATALOGUE · admitted scenario</p>
        <div className="ppl-status-grid">
          <div>
            <h2>{scenario?.title ?? "Catalogue available after sign-in"}</h2>
            <p>{scenario?.purpose}</p>
          </div>
          <div className="ppl-runtime-message">
            Status <strong>{scenario?.status ?? "not evaluated"}</strong>
            <br />
            {scenario?.maturity} · {scenario?.estimatedDuration}
          </div>
        </div>
        {scenario && (
          <>
            <p>
              Actors: {scenario.actors.join(", ")} · Required:{" "}
              {scenario.requiredComponents.join(", ")}
            </p>
            <ul>
              {scenario.limitations.map((limitation) => (
                <li key={limitation}>{limitation}</li>
              ))}
            </ul>
          </>
        )}
        <div className="ppl-button-row">
          <button
            className="ppl-button"
            type="button"
            disabled={
              !presenter ||
              !scenario ||
              scenario.status !== "ready" ||
              Boolean(session)
            }
            onClick={() =>
              void run(async () => {
                const result = await requestJson<{ session: SessionSnapshot }>(
                  "/api/v1/sessions",
                  {},
                );
                setSession(result.session);
                window.sessionStorage.setItem(
                  "ppl-current-demonstration-session",
                  result.session.sessionId,
                );
                setMessage(
                  "Demonstration Session created without a synthetic application session.",
                );
              })
            }
          >
            Create Demonstration Session
          </button>
        </div>
      </article>

      {session && (
        <article className="ppl-card ppl-live-card">
          <p className="ppl-card-label">DIR-RUN · approved Gate B journey</p>
          <div className="ppl-status-grid ppl-runtime-message">
            <span>
              Session
              <br />
              <strong className="ppl-mono">{session.sessionId}</strong>
            </span>
            <span>
              State
              <br />
              <strong>{session.state}</strong>
            </span>
            <span>
              Revision
              <br />
              <strong>{session.revision}</strong>
            </span>
            <span>
              Scenario time
              <br />
              <strong>{session.logicalTime}</strong>
            </span>
          </div>

          <h2>1. Prepare identities and surfaces</h2>
          <div className="ppl-button-row">
            <button
              className="ppl-button"
              type="button"
              disabled={session.state !== "preparing"}
              onClick={() => void prepare()}
            >
              Prepare scenario
            </button>
            {environment?.presentationSurfaceUrl && (
              <a
                className="ppl-button ppl-button-secondary"
                href={environment.presentationSurfaceUrl}
              >
                Open Presentation
              </a>
            )}
            {environment?.workbenchSurfaceUrl && (
              <a
                className="ppl-button ppl-button-secondary"
                href={environment.workbenchSurfaceUrl}
              >
                Open Workbench
              </a>
            )}
            <button
              className="ppl-button"
              type="button"
              disabled={
                !(["ready", "running", "paused"] as string[]).includes(
                  session.state,
                )
              }
              onClick={() =>
                void run(async () => {
                  await requestJson(
                    `/api/v1/sessions/${encodeURIComponent(session.sessionId)}/synthetic-sign-in`,
                    {
                      actorId: "synthetic-reviewer",
                      surfaceSlot: "reviewer-workbench",
                    },
                  );
                  setMessage(
                    "Environment- and session-bound synthetic reviewer requested for the Workbench; no grant entered the browser.",
                  );
                })
              }
            >
              Assign synthetic-reviewer
            </button>
            <button
              className="ppl-button"
              type="button"
              disabled={session.state !== "ready"}
              onClick={() => void lifecycle("start")}
            >
              Start scenario
            </button>
          </div>

          <h2>2. Direct semantic views</h2>
          <div className="ppl-button-row">
            <button
              className="ppl-button"
              type="button"
              disabled={!running}
              onClick={() =>
                void requestView(
                  "audience-display",
                  "pres-intro",
                  "A governed source, not a magic answer",
                  "Harbour Community Support is reviewing synthetic policy material through accountable components and human authority.",
                )
              }
            >
              Show introduction
            </button>
            <button
              className="ppl-button"
              type="button"
              disabled={!running}
              onClick={() =>
                void requestView(
                  "reviewer-workbench",
                  "wb-engagement",
                  "Harbour support policy review",
                  "Open the bounded synthetic engagement context for the assigned reviewer.",
                )
              }
            >
              Open engagement context
            </button>
            <button
              className="ppl-button"
              type="button"
              disabled={!running}
              onClick={() =>
                void requestView(
                  "reviewer-workbench",
                  "wb-source-intake",
                  "Add a source for governed review",
                  "Show human-operated upload, paste and link controls; do not submit anything yet.",
                )
              }
            >
              Open source intake
            </button>
            <button
              className="ppl-button ppl-button-secondary"
              type="button"
              disabled={!running}
              onClick={() =>
                void requestView(
                  "reviewer-workbench",
                  "wb-not-admitted",
                  "Unsupported view test",
                  "This request must be refused before delivery.",
                )
              }
            >
              Test unsupported view refusal
            </button>
          </div>

          <h2>3. Pause and close safely</h2>
          <div className="ppl-button-row">
            <button
              className="ppl-button"
              type="button"
              disabled={!running}
              onClick={() => void lifecycle("pause")}
            >
              Pause scenario
            </button>
            <button
              className="ppl-button ppl-button-secondary"
              type="button"
              disabled={session.state !== "paused"}
              onClick={() => void lifecycle("resume")}
            >
              Resume
            </button>
            <button
              className="ppl-button ppl-button-secondary"
              type="button"
              disabled={
                !(
                  ["preparing", "ready", "running", "paused"] as string[]
                ).includes(session.state)
              }
              onClick={() => void lifecycle("stop")}
            >
              Stop and terminate synthetic sessions
            </button>
            <button
              className="ppl-button ppl-button-secondary"
              type="button"
              disabled={
                !(["stopped", "completed", "failed"] as string[]).includes(
                  session.state,
                )
              }
              onClick={() => void lifecycle("reset")}
            >
              Reset to successor
            </button>
          </div>
        </article>
      )}

      <article className="ppl-card ppl-live-card">
        <p className="ppl-card-label">Current system response</p>
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
