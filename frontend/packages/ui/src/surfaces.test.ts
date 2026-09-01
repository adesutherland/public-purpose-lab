import { describe, expect, it } from "vitest";
import { surfaceById, surfaceIds } from "./surfaces.ts";

describe("surface catalogue", () => {
  it("contains the three initial presentation boundaries", () => {
    expect(surfaceIds).toEqual(["UX-02", "UX-03", "UX-04"]);
  });

  it.each(surfaceIds)("describes %s as an in-development boundary", (id) => {
    const surface = surfaceById(id);

    expect(surface.id).toBe(id);
    expect(surface.maturity).toBe("in-development");
    expect(surface.purpose.toLowerCase()).not.toContain("compliance");
    expect(surface.actions).toHaveLength(3);
    expect(surface.evidence).toHaveLength(3);
  });
});
