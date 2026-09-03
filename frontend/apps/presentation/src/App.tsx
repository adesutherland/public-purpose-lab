import { useEffect, useState } from "react";
import type {
  PresentationCue,
  PresentationCueOutcome,
  PresentationProcessingProgress,
  PresentationRegistration,
} from "@public-purpose-lab/contracts";
import { SurfaceShell, surfaceById } from "@public-purpose-lab/ui";

async function postJson<T>(path: string, body: object): Promise<T> {
  const csrf = document.cookie
    .split(";")
    .map((value) => value.trim())
    .find((value) => value.startsWith("PPL_CSRF="))
    ?.slice("PPL_CSRF=".length);
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
  const [authenticated, setAuthenticated] = useState(false);
  const [loginMode, setLoginMode] = useState<"local-test" | "google-oidc">(
    "local-test",
  );
  const [sessionId, setSessionId] = useState("");
  const [registration, setRegistration] = useState<PresentationRegistration>();
  const [activeCue, setActiveCue] = useState<PresentationCue>();
  const [processing, setProcessing] =
    useState<PresentationProcessingProgress>();
  const [message, setMessage] = useState(
    "Connect this display and register it to the Director's Demonstration Session.",
  );
  const [error, setError] = useState(false);

  useEffect(() => {
    void getJson<{ mode: "local-test" | "google-oidc" }>("/api/v1/login-mode")
      .then((result) => setLoginMode(result.mode))
      .catch(() => undefined);
    void getJson<{ registration?: PresentationRegistration }>(
      "/api/v1/session-context",
    )
      .then((context) => {
        setAuthenticated(true);
        if (context.registration?.surfaceSlot === "audience-display") {
          setRegistration(context.registration);
          setSessionId(context.registration.sessionId);
        }
        setMessage("External presentation-surface operator session restored.");
      })
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    if (!registration) return undefined;
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
        const supported = ["pres-intro", "pres-progress"].includes(
          cue.semanticView,
        );
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
          setActiveCue(cue);
          setMessage(
            `${cue.semanticView} resolved by this surface; presentation progress recorded.`,
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
      setMessage("Cue channel reconnecting; no presentation result inferred.");
    return () => source.close();
  }, [registration]);

  useEffect(() => {
    if (activeCue?.semanticView !== "pres-progress") return undefined;
    let active = true;
    const refresh = () =>
      getJson<PresentationProcessingProgress>("/api/v1/presentation-processing")
        .then((status) => {
          if (active) setProcessing(status);
        })
        .catch(() => undefined);
    void refresh();
    const interval = window.setInterval(refresh, 750);
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [activeCue?.semanticView]);

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

  return (
    <SurfaceShell
      surface={surfaceById("UX-04")}
      maturityLabel="Gate C · visible processing candidate"
      notice="Synthetic demonstration only. This read-only surface reports component-owned progress; it cannot stage a source, control processing or claim compliance."
    >
      <article className="ppl-card ppl-live-card">
        <p className="ppl-card-label">Audience presentation binding</p>
        <label className="ppl-field">
          Demonstration Session ID
          <input
            value={sessionId}
            placeholder="session:…"
            onChange={(event) => setSessionId(event.target.value)}
          />
        </label>
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
                setMessage("Local test presentation operator authenticated.");
              })
            }
          >
            {loginMode === "google-oidc"
              ? "Sign in with Google"
              : "Connect test presentation operator"}
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
                  "Audience display registered and listening for admitted semantic views.",
                );
              })
            }
          >
            Register audience display
          </button>
        </div>
        {registration && (
          <div className="ppl-status-grid ppl-runtime-message">
            <span>
              Registration
              <br />
              <strong className="ppl-mono">
                {registration.registrationId}
              </strong>
            </span>
            <span>
              Session
              <br />
              <strong className="ppl-mono">{registration.sessionId}</strong>
            </span>
            <span>
              Views
              <br />
              <strong>{registration.supportedViews.join(", ")}</strong>
            </span>
            <span>
              Lease ends
              <br />
              <strong>{registration.leaseExpiresAt}</strong>
            </span>
          </div>
        )}
      </article>

      <article className="ppl-card ppl-live-card">
        <p className="ppl-card-label">Target-owned semantic view</p>
        {activeCue?.semanticView === "pres-intro" ? (
          <section className="ppl-semantic-view" aria-live="polite">
            <p className="ppl-eyebrow">PRES-INTRO · scenario introduction</p>
            <h2>{activeCue.context.heading}</h2>
            <p className="ppl-purpose">{activeCue.context.message}</p>
            <div className="ppl-status-grid ppl-runtime-message">
              <span>
                Synthetic organisation
                <br />
                <strong>Harbour Community Support</strong>
              </span>
              <span>
                Actors
                <br />
                <strong>Presenter · synthetic reviewer</strong>
              </span>
              <span>
                Desired outcome
                <br />
                <strong>Govern a source before deriving guidance</strong>
              </span>
              <span>
                Current stage
                <br />
                <strong>Introduction and portal orchestration</strong>
              </span>
            </div>
            <h3>What this demonstration does not claim</h3>
            <ul>
              <li>No real person, organisation or protected information.</li>
              <li>No engagement or source has yet been created.</li>
              <li>
                No legal, regulatory or professional responsibility transfers to
                the Lab.
              </li>
            </ul>
          </section>
        ) : activeCue?.semanticView === "pres-progress" ? (
          <section className="ppl-semantic-view" aria-live="polite">
            <p className="ppl-eyebrow">PRES-PROGRESS · business progress</p>
            <h2>{activeCue.context.heading}</h2>
            <p className="ppl-purpose">{activeCue.context.message}</p>
            <div className="ppl-status-grid ppl-runtime-message">
              <span>
                Current component
                <br />
                <strong>{processing?.componentId ?? "KNO-01"}</strong>
              </span>
              <span>
                Processing state
                <br />
                <strong>{processing?.lifecycleStatus ?? "waiting"}</strong>
              </span>
              <span>
                Latest conclusive outcome
                <br />
                <strong>
                  {processing?.latestOutcome ?? "No processing fact observed"}
                </strong>
              </span>
              <span>
                Bounded result
                <br />
                <strong>
                  {processing?.byteCount === undefined
                    ? "Pending"
                    : `${processing.byteCount} bytes · ${processing.lineCount} lines · ${processing.sectionCount} sections`}
                </strong>
              </span>
            </div>
            <p className="ppl-mono">
              {processing?.sourceVersionId ?? "source version pending"}
            </p>
            <h3>Authority and content boundary</h3>
            <p>
              {processing?.limitation ??
                "The Presentation Surface receives progress metadata only and never receives the source body."}
            </p>
            <p>
              This surface has no control for receipt, validation, staging or
              processing. Its cue records presentation progress only.
            </p>
          </section>
        ) : (
          <p className="ppl-runtime-message">
            Waiting for the Director to request PRES-INTRO or PRES-PROGRESS.
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
