import { describe, expect, it } from "vitest";
import { surfaceById, surfaceIds } from "./surfaces.ts";

describe("surface catalogue", () => {
  it("contains the three initial presentation boundaries", () => {
    expect(surfaceIds).toEqual(["UX-02", "UX-03", "UX-04"]);
  });

  it.each(surfaceIds)("describes %s without claiming live capability", (id) => {
    const surface = surfaceById(id);

    expect(surface.id).toBe(id);
    expect(surface.maturity).toBe("repository-skeleton");
    expect(surface.purpose.toLowerCase()).toContain("future");
    expect(surface.actions).toHaveLength(3);
    expect(surface.evidence).toHaveLength(3);
  });
});
