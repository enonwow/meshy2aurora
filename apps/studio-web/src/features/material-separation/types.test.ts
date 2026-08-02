import { describe, expect, it } from "vitest";
import { isCurrentModelMaterialResponseV1, parseModelMaterialResolutionV1 } from "./types";

const resolution = () => ({
  schemaVersion: 1,
  capabilities: {
    schemaVersion: 1,
    target: "PLACEABLE",
    materialSeparationSupported: true,
    maxMaterialSlots: 256,
    maxOutputSections: 4096,
    selectionGranularity: "CONNECTED_COMPONENTS",
    faceSelectionSupported: false,
    automaticMaterialInference: false,
    preservesSourceUv0: true,
  },
  report: {
    schemaVersion: 1,
    sourceSha256: "a".repeat(64),
    separationSha256: "b".repeat(64),
    sourceComponentCount: 2,
    assignedComponentCount: 2,
    unassignedComponentCount: 0,
    sourceTriangleCount: 2,
    outputTriangleCount: 2,
    sourceVertexCount: 6,
    outputVertexCount: 6,
    duplicatedBoundaryVertexCount: 0,
    outputSectionCount: 2,
    predictedTextureCount: 2,
    materialSlots: [{}, {}],
    unusedAuthoredMaterialIds: [],
    warnings: [],
  },
  textureAuthoring: {
    schemaVersion: 1,
    sourceSha256: "a".repeat(64),
    separationSha256: "b".repeat(64),
    bindings: [{}, {}],
  },
});

describe("Material Separation boundary parsing", () => {
  it("requires the predicted texture count emitted by Core", () => {
    expect(parseModelMaterialResolutionV1(JSON.stringify(resolution())).report.predictedTextureCount)
      .toBe(2);

    const missing = resolution();
    delete (missing.report as Partial<typeof missing.report>).predictedTextureCount;
    expect(() => parseModelMaterialResolutionV1(JSON.stringify(missing)))
      .toThrow("MODEL-MATERIAL-RESOLUTION-INVALID");
  });

  it("rejects stale Worker responses after either source, target or recipe changes", () => {
    const current = {
      responseSourceStateId: `PLACEABLE:${"a".repeat(64)}`,
      responseRecipeStateId: "recipe:v2",
      expectedSourceStateId: `PLACEABLE:${"a".repeat(64)}`,
      expectedRecipeStateId: "recipe:v2",
      currentSourceSha256: "a".repeat(64),
      expectedSourceSha256: "a".repeat(64),
      currentTarget: "PLACEABLE" as const,
      expectedTarget: "PLACEABLE" as const,
    };
    expect(isCurrentModelMaterialResponseV1(current)).toBe(true);
    expect(isCurrentModelMaterialResponseV1({
      ...current,
      responseRecipeStateId: "recipe:v1",
    })).toBe(false);
    expect(isCurrentModelMaterialResponseV1({
      ...current,
      currentSourceSha256: "b".repeat(64),
    })).toBe(false);
    expect(isCurrentModelMaterialResponseV1({
      ...current,
      currentTarget: "CREATURE",
    })).toBe(false);
  });
});
