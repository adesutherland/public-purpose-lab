export const surfaceIds = ["UX-02", "UX-03", "UX-04"] as const;

export type SurfaceId = (typeof surfaceIds)[number];

export interface SurfaceDefinition {
  readonly id: SurfaceId;
  readonly eyebrow: string;
  readonly title: string;
  readonly purpose: string;
  readonly actions: readonly string[];
  readonly evidence: readonly string[];
}

export const surfaces: Readonly<Record<SurfaceId, SurfaceDefinition>> = {
  "UX-02": {
    id: "UX-02",
    eyebrow: "Governed workbench",
    title: "Understand assets. Keep authority visible.",
    purpose:
      "A future workspace for linking and staging assets, asking evidence-bounded questions, and preparing reviewable reports, relationships and diagrams.",
    actions: [
      "Link or upload assets",
      "Review staged evidence",
      "Prepare an accountable output",
    ],
    evidence: [
      "Source provenance",
      "Policy and approval state",
      "Report and audit trail",
    ],
  },
  "UX-03": {
    id: "UX-03",
    eyebrow: "Scenario director",
    title: "Direct the story through governed events.",
    purpose:
      "A future control surface for preparing synthetic actors, advancing a scenario and observing outcomes without fragile browser-to-browser control.",
    actions: [
      "Prepare an environment",
      "Select a scenario",
      "Issue reviewed demonstration commands",
    ],
    evidence: [
      "Environment identity",
      "Command correlation",
      "Scenario outcome",
    ],
  },
  "UX-04": {
    id: "UX-04",
    eyebrow: "Presentation surface",
    title: "Show outcomes without exposing control.",
    purpose:
      "A future audience-facing view that follows demonstration events and presents evidence, decisions and service outcomes clearly.",
    actions: [
      "Follow the demonstration",
      "Explain an outcome",
      "Inspect supporting evidence",
    ],
    evidence: [
      "Current scenario step",
      "Decision explanation",
      "Visible limitations",
    ],
  },
};

export function surfaceById(id: SurfaceId): SurfaceDefinition {
  return surfaces[id];
}
