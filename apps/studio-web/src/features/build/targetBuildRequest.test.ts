import { describe, expect, it } from "vitest";
import { createTargetBuildRequestV1 } from "./targetBuildRequest";

describe("target build request controller", () => {
  it("keeps Placeable bound to the exact project identity", () => {
    const sourceGlb = new ArrayBuffer(3);
    const baseTwoDa = new ArrayBuffer(5);
    const built = createTargetBuildRequestV1({
      target: "PLACEABLE",
      requestId: "placeable-7",
      sourceGlb,
      baseTwoDa,
      projectIdentityJson: '{"projectRevision":7}',
      authoringJson: '{"schemaVersion":1}',
    });
    expect(built.request).toMatchObject({
      type: "BUILD_PLACEABLE_PACKAGE",
      projectIdentityJson: '{"projectRevision":7}',
      paletteId: 7,
    });
    expect(built.transfer).toEqual([sourceGlb, baseTwoDa]);
  });

  it("owns the experimental Tile identity outside App", () => {
    const built = createTargetBuildRequestV1({
      target: "TILE",
      requestId: "tile",
      sourceGlb: new ArrayBuffer(1),
      tileOptions: {
        terrainName: "  Grass  ",
        surface: "GRASS",
        interior: false,
      },
    });
    expect(JSON.parse(
      built.request.type === "BUILD_TILE_PACKAGE"
        ? built.request.optionsJson
        : "{}",
    )).toMatchObject({
      identity: { moduleFileName: "m2a_tile_static_v1.mod" },
      terrainName: "Grass",
    });
  });

  it("keeps edited Creature-only fields out of the authored V4 request", () => {
    const built = createTargetBuildRequestV1({
      target: "CREATURE",
      requestId: "creature",
      sourceGlb: new ArrayBuffer(1),
      baseTwoDa: new ArrayBuffer(1),
      projectIdentityJson: '{"projectRevision":4}',
      packageLane: "H1_SKINNED_FULL_42_AUTHORED",
      animationAuthoringJson: '{"assignments":[]}',
    });
    expect(built.request).toMatchObject({
      type: "BUILD_MODEL_PACKAGE",
      packageLane: "H1_SKINNED_FULL_42_AUTHORED",
    });
    expect("animationStudioDocumentJson" in built.request).toBe(false);
  });
});
