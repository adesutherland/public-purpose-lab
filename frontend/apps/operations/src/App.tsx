import { useEffect, useMemo, useState } from "react";

interface ComponentStatus {
  componentId: string;
  componentName: string;
  status: "ready" | "stale" | "missing";
  instanceId?: string;
  workloadIdentity?: string;
  capability: string;
  sourceRevision?: string;
  imageDigest?: string;
  lastActivity?: string;
  ageSeconds?: number;
}

interface MeshSnapshot {
  environmentId: string;
  expected: number;
  ready: number;
  status: "ready" | "degraded";
  components: ComponentStatus[];
  observedAt: string;
}

interface OperationalEvent {
  eventId: string;
  eventType: string;
  componentId: string;
  componentName: string;
  status: string;
  occurredAt: string;
  correlationId?: string;
  commandName?: string;
  reasonCode?: string;
}

async function getJson<T>(path: string): Promise<T> {
  const response = await fetch(path, { credentials: "same-origin" });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return (await response.json()) as T;
}

async function postJson<T>(path: string): Promise<T> {
  const response = await fetch(path, {
    method: "POST",
    credentials: "same-origin",
  });
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return (await response.json()) as T;
}

export function App() {
  const [mesh, setMesh] = useState<MeshSnapshot>();
  const [events, setEvents] = useState<OperationalEvent[]>([]);
  const [componentFilter, setComponentFilter] = useState("all");
  const [showReadiness, setShowReadiness] = useState(false);
  const [probeMessage, setProbeMessage] = useState<string>();
  const [message, setMessage] = useState(
    "Waiting for authenticated component readiness events.",
  );

  const refresh = async () => {
    const [nextMesh, nextEvents] = await Promise.all([
      getJson<MeshSnapshot>("/api/v1/mesh"),
      getJson<{ events: OperationalEvent[] }>("/api/v1/events"),
    ]);
    setMesh(nextMesh);
    setEvents(nextEvents.events);
    setMessage(
      nextMesh.status === "ready"
        ? `${nextMesh.ready} of ${nextMesh.expected} component instances are reporting ready.`
        : `${nextMesh.ready} of ${nextMesh.expected} component instances are ready; inspect missing or stale entries.`,
    );
  };

  useEffect(() => {
    void refresh().catch(() =>
      setMessage("Operations projection is temporarily unavailable."),
    );
    const interval = window.setInterval(() => void refresh(), 2000);
    return () => window.clearInterval(interval);
  }, []);

  const filteredEvents = useMemo(
    () =>
      events
        .filter(
          (event) =>
            (componentFilter === "all" ||
              event.componentId === componentFilter) &&
            (showReadiness || event.eventType !== "component.ready"),
        )
        .slice(0, 18),
    [componentFilter, events, showReadiness],
  );

  const runProbe = async () => {
    try {
      const result = await postJson<{
        correlationId: string;
        targetCount: number;
      }>("/api/v1/probe");
      setProbeMessage(
        `Issued ${result.targetCount} bounded capability commands under ${result.correlationId}.`,
      );
      window.setTimeout(() => void refresh(), 400);
    } catch (error) {
      setProbeMessage(
        error instanceof Error ? error.message : "Probe failed safely",
      );
    }
  };

  return (
    <div className="ppl-shell ops-shell">
      <header className="ppl-header">
        <a className="ppl-brand" href="https://publicpurposelab.org">
          <span className="ppl-brand-mark" aria-hidden="true">
            PPL
          </span>
          <span>Public Purpose Lab</span>
        </a>
        <span className="ppl-maturity">Gate B · functional demonstration</span>
      </header>

      <main>
        <section className="ppl-hero ops-hero">
          <p className="ppl-eyebrow">Operations console · OPS-COMPONENTS</p>
          <h1>One mesh. Distinct responsibilities.</h1>
          <p className="ppl-purpose">
            Live readiness and correlated scenario, identity and semantic-view
            events from the component mesh in this synthetic environment.
          </p>
          <div className="ops-summary" role="status">
            <strong>{mesh?.ready ?? 0}</strong>
            <span>of {mesh?.expected ?? 12} ready</span>
            <span className={`ops-pill ops-${mesh?.status ?? "degraded"}`}>
              {mesh?.status ?? "connecting"}
            </span>
          </div>
          <div className="ppl-button-row">
            <button className="ppl-button" type="button" onClick={runProbe}>
              Probe component mesh
            </button>
            <a
              className="ppl-button ppl-button-secondary"
              href="http://localhost:18081/"
            >
              Open Director
            </a>
            <a
              className="ppl-button ppl-button-secondary"
              href="http://127.0.0.1:18082/"
            >
              Open Presentation
            </a>
            <a
              className="ppl-button ppl-button-secondary"
              href="http://[::1]:18082/workbench/"
            >
              Open Workbench
            </a>
          </div>
          <div className="ppl-runtime-message">{probeMessage ?? message}</div>
          <p className="ops-environment ppl-mono">
            {mesh?.environmentId ?? "environment pending"}
          </p>
        </section>

        <section className="ops-content">
          <div className="ops-section-heading">
            <div>
              <p className="ppl-card-label">OPS-COMPONENTS</p>
              <h2>Deployed component instances</h2>
            </div>
            <span>Readiness expires after 15 seconds</span>
          </div>
          <div className="ops-components">
            {(mesh?.components ?? []).map((component) => (
              <article className="ops-component" key={component.componentId}>
                <div className="ops-component-heading">
                  <span className="ppl-mono">{component.componentId}</span>
                  <span className={`ops-pill ops-${component.status}`}>
                    {component.status}
                  </span>
                </div>
                <h3>{component.componentName}</h3>
                <p>{component.capability}</p>
                <dl>
                  <div>
                    <dt>Instance</dt>
                    <dd>{component.instanceId ?? "not observed"}</dd>
                  </div>
                  <div>
                    <dt>Workload</dt>
                    <dd className="ppl-mono">
                      {component.workloadIdentity ?? "not observed"}
                    </dd>
                  </div>
                  <div>
                    <dt>Last activity</dt>
                    <dd>
                      {component.ageSeconds === undefined
                        ? "—"
                        : `${component.ageSeconds}s ago`}
                    </dd>
                  </div>
                </dl>
              </article>
            ))}
          </div>

          <div className="ops-section-heading ops-events-heading">
            <div>
              <p className="ppl-card-label">OPS-EVENTS</p>
              <h2>Operational event timeline</h2>
            </div>
            <div className="ops-event-filters">
              <label className="ppl-field ops-filter">
                Component
                <select
                  value={componentFilter}
                  onChange={(event) => setComponentFilter(event.target.value)}
                >
                  <option value="all">All components</option>
                  {mesh?.components.map((component) => (
                    <option
                      key={component.componentId}
                      value={component.componentId}
                    >
                      {component.componentId} · {component.componentName}
                    </option>
                  ))}
                </select>
              </label>
              <label className="ops-checkbox">
                <input
                  type="checkbox"
                  checked={showReadiness}
                  onChange={(event) => setShowReadiness(event.target.checked)}
                />
                Include readiness heartbeats
              </label>
            </div>
          </div>
          <div className="ops-timeline">
            {filteredEvents.length === 0 && (
              <p className="ops-empty-events">
                No scenario or command outcomes yet. Run Gate B in the Director
                or probe the component mesh.
              </p>
            )}
            {filteredEvents.map((event) => (
              <article key={event.eventId}>
                <time dateTime={event.occurredAt}>
                  {new Date(event.occurredAt).toLocaleTimeString()}
                </time>
                <span className="ppl-mono">{event.componentId}</span>
                <strong>{event.eventType}</strong>
                <span>{event.commandName ?? event.status}</span>
                <span className="ppl-mono ops-correlation">
                  {event.correlationId ?? event.reasonCode ?? "readiness"}
                </span>
              </article>
            ))}
          </div>
        </section>
      </main>

      <footer>
        <span>OPS-01 · O-001 · Gate B</span>
        <span>Synthetic data only · no compliance claim</span>
      </footer>
    </div>
  );
}
