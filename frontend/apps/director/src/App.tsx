import { useCallback, useState } from "react";
import type { ScenarioState } from "@public-purpose-lab/contracts";
import { SurfaceShell, surfaceById } from "@public-purpose-lab/ui";

interface SessionSnapshot {
  readonly sessionId: string;
  readonly state: ScenarioState;
  readonly revision: number;
  readonly logicalTime: string;
  readonly logicalTimeInitialised: boolean;
  readonly successorSessionId?: string;
}

async function requestJson<T>(path: string, body?: object): Promise<T> {
  const response = await fetch(path, {
    method: body ? "POST" : "GET",
    credentials: "same-origin",
    headers: body ? { "Content-Type": "application/json" } : undefined,
    body: body ? JSON.stringify(body) : undefined,
  });
  const result = (await response.json()) as T & { code?: string };
  if (!response.ok) throw new Error(result.code ?? `HTTP ${response.status}`);
  return result;
}

export function App() {
  const [authenticated, setAuthenticated] = useState(false);
  const [session, setSession] = useState<SessionSnapshot>();
  const [message, setMessage] = useState(
    "Connect the local assurance presenter to begin.",
  );
  const [error, setError] = useState(false);
  const [cueHeading, setCueHeading] = useState("Assurance demonstration");
  const [cueMessage, setCueMessage] = useState(
    "This view was selected by semantic event, using synthetic information only.",
  );

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

  const refresh = useCallback(
    async (sessionId = session?.sessionId) => {
      if (!sessionId) return;
      const status = await requestJson<{ session: SessionSnapshot }>(
        `/api/v1/status/${encodeURIComponent(sessionId)}`,
      );
      setSession(status.session);
      setMessage(
        `Current state: ${status.session.state}, revision ${status.session.revision}.`,
      );
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
          reason: `Presenter requested ${action} in the assurance console.`,
        },
      );
      setSession(result.successor ?? result.session);
      setMessage(
        result.successor
          ? `Reset created successor ${result.successor.sessionId}.`
          : `Lifecycle action ${action} accepted.`,
      );
    });

  return (
    <SurfaceShell
      surface={surfaceById("UX-03")}
      maturityLabel="In-development walking skeleton"
      notice="Synthetic development assurance only. Presentation progress is not business completion or evidence of compliance."
    >
      <article className="ppl-card ppl-live-card">
        <p className="ppl-card-label">Live demonstration control</p>
        <div className="ppl-button-row">
          <button
            className="ppl-button"
            type="button"
            disabled={authenticated}
            onClick={() =>
              void run(async () => {
                await requestJson("/api/v1/development-session", {});
                setAuthenticated(true);
                setMessage(
                  "Local synthetic presenter session established for 30 minutes.",
                );
              })
            }
          >
            Connect synthetic presenter
          </button>
          <button
            className="ppl-button"
            type="button"
            disabled={!authenticated || Boolean(session)}
            onClick={() =>
              void run(async () => {
                const result = await requestJson<{ session: SessionSnapshot }>(
                  "/api/v1/sessions",
                  {},
                );
                setSession(result.session);
                setMessage(
                  "Session created. Copy its ID to a presentation surface.",
                );
              })
            }
          >
            Create session
          </button>
          <button
            className="ppl-button ppl-button-secondary"
            type="button"
            disabled={!session}
            onClick={() => void run(() => refresh())}
          >
            Refresh status
          </button>
        </div>

        {session && (
          <>
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
            <div className="ppl-button-row" aria-label="Lifecycle controls">
              <button
                className="ppl-button"
                type="button"
                disabled={session.state !== "preparing"}
                onClick={() => void lifecycle("prepare")}
              >
                Prepare
              </button>
              <button
                className="ppl-button"
                type="button"
                disabled={session.state !== "ready"}
                onClick={() => void lifecycle("start")}
              >
                Start
              </button>
              <button
                className="ppl-button"
                type="button"
                disabled={session.state !== "running"}
                onClick={() => void lifecycle("pause")}
              >
                Pause
              </button>
              <button
                className="ppl-button"
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
                Stop
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

            <hr />
            <div className="ppl-controls">
              <label className="ppl-field">
                Cue heading
                <input
                  value={cueHeading}
                  onChange={(event) => setCueHeading(event.target.value)}
                />
              </label>
              <label className="ppl-field">
                Cue message
                <input
                  value={cueMessage}
                  onChange={(event) => setCueMessage(event.target.value)}
                />
              </label>
            </div>
            <div className="ppl-button-row">
              <button
                className="ppl-button ppl-button-secondary"
                type="button"
                disabled={
                  session.logicalTimeInitialised ||
                  session.state !== "preparing"
                }
                onClick={() =>
                  void run(async () => {
                    const result = await requestJson<{
                      sessionRevision: number;
                      logicalTime: string;
                    }>(
                      `/api/v1/sessions/${encodeURIComponent(session.sessionId)}/logical-time`,
                      {
                        operation: "set",
                        expectedRevision: session.revision,
                        logicalInstant: "2030-01-01T09:00:00Z",
                      },
                    );
                    setSession({
                      ...session,
                      revision: result.sessionRevision,
                      logicalTime: result.logicalTime,
                      logicalTimeInitialised: true,
                    });
                    setMessage(
                      "Package-declared initial scenario time established.",
                    );
                  })
                }
              >
                Set initial scenario time
              </button>
              <button
                className="ppl-button"
                type="button"
                disabled={
                  session.state !== "running" || !session.logicalTimeInitialised
                }
                onClick={() =>
                  void run(async () => {
                    await requestJson(
                      `/api/v1/sessions/${encodeURIComponent(session.sessionId)}/cue`,
                      {
                        surfaceSlot: "audience-display",
                        semanticView: "assurance-welcome",
                        heading: cueHeading,
                        message: cueMessage,
                        expiresInSeconds: 60,
                      },
                    );
                    setMessage(
                      "Semantic cue committed to the Director outbox.",
                    );
                  })
                }
              >
                Cue audience display
              </button>
              <button
                className="ppl-button ppl-button-secondary"
                type="button"
                disabled={
                  session.state !== "running" || !session.logicalTimeInitialised
                }
                onClick={() =>
                  void run(async () => {
                    const result = await requestJson<{
                      sessionRevision: number;
                      logicalTime: string;
                    }>(
                      `/api/v1/sessions/${encodeURIComponent(session.sessionId)}/logical-time`,
                      {
                        operation: "advance",
                        expectedRevision: session.revision,
                        advanceSeconds: 300,
                      },
                    );
                    setSession({
                      ...session,
                      revision: result.sessionRevision,
                      logicalTime: result.logicalTime,
                    });
                    setMessage(
                      "Scenario time advanced; operational expiry was unchanged.",
                    );
                  })
                }
              >
                Advance scenario time
              </button>
              <button
                className="ppl-button ppl-button-secondary"
                type="button"
                disabled={session.state !== "running"}
                onClick={() =>
                  void run(async () => {
                    await requestJson(
                      `/api/v1/sessions/${encodeURIComponent(session.sessionId)}/cue-delay`,
                      {
                        expectedRevision: session.revision,
                        delayMilliseconds: 750,
                      },
                    );
                    setMessage(
                      "One bounded 750 ms cue delay is armed for this session.",
                    );
                  })
                }
              >
                Arm next-cue delay
              </button>
            </div>
          </>
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
