import type { ReactNode } from "react";
import type { SurfaceDefinition } from "./surfaces.ts";

export { surfaceById, surfaceIds, surfaces } from "./surfaces.ts";
export type { SurfaceDefinition, SurfaceId } from "./surfaces.ts";

interface SurfaceShellProps {
  readonly surface: SurfaceDefinition;
  readonly children?: ReactNode;
}

export function SurfaceShell({ surface, children }: SurfaceShellProps) {
  return (
    <div className="ppl-shell">
      <header className="ppl-header">
        <a className="ppl-brand" href="https://publicpurposelab.org">
          <span className="ppl-brand-mark" aria-hidden="true">
            PPL
          </span>
          <span>Public Purpose Lab</span>
        </a>
        <span className="ppl-maturity">Repository skeleton</span>
      </header>

      <main>
        <section className="ppl-hero">
          <p className="ppl-eyebrow">{surface.eyebrow}</p>
          <h1>{surface.title}</h1>
          <p className="ppl-purpose">{surface.purpose}</p>
          <div className="ppl-notice" role="note">
            This interface demonstrates the intended boundary only. Its actions
            are not yet connected.
          </div>
        </section>

        <section className="ppl-grid" aria-label="Surface outline">
          <article className="ppl-card">
            <p className="ppl-card-label">Intended workflow</p>
            <ol>
              {surface.actions.map((action) => (
                <li key={action}>{action}</li>
              ))}
            </ol>
          </article>
          <article className="ppl-card ppl-card-accent">
            <p className="ppl-card-label">Evidence made visible</p>
            <ul>
              {surface.evidence.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </article>
          {children}
        </section>
      </main>

      <footer>
        <span>{surface.id}</span>
        <span>Synthetic data only</span>
      </footer>
    </div>
  );
}
