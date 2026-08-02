import { describe, expect, it } from "vitest";
import { diagnosePlaceableAuthoring, measurePlaceableElements } from "./diagnostics";
import type { PlaceableAuthoringDocument, PlaceableElementInspection } from "./types";

const inspection: PlaceableElementInspection = {
  schemaVersion: 1,
  sourceSha256: "a".repeat(64),
  renderNodeCount: 1,
  primitiveCount: 1,
  connectedComponentCount: 2,
  nodes: [{
    elementId: "node:0",
    nodeId: 0,
    name: "reactor",
    meshId: 0,
    primitives: [{
      primitiveId: 0,
      materialId: 0,
      triangleCount: 2,
      components: [
        { elementId: "a", componentIndex: 0, triangleCount: 1, vertexCount: 3, boundsMin: [0, 0, 0], boundsMax: [1, 1, 1] },
        { elementId: "b", componentIndex: 1, triangleCount: 1, vertexCount: 3, boundsMin: [2, 0, 0], boundsMax: [3, 1, 1] },
      ],
    }],
  }],
};

const transform = (translation: [number, number, number]) => ({
  translation,
  rotationXyzw: [0, 0, 0, 1] as [number, number, number, number],
  scale: [1, 1, 1] as [number, number, number],
  pivot: [0, 0, 0] as [number, number, number],
});

const document: PlaceableAuthoringDocument = {
  schemaVersion: 2,
  sourceSha256: inspection.sourceSha256,
  elements: [{
    id: "a",
    name: "a",
    kind: "SOURCE_COMPONENT",
    source: { nodeId: 0, primitiveId: 0, componentIndex: 0 },
    parentId: null,
    transform: transform([0, 0.2, 0]),
    flags: { hidden: false, locked: false, renderable: true, includeInCollision: true, castShadow: true },
    deleted: false,
  }, {
    id: "b",
    name: "b",
    kind: "SOURCE_COMPONENT",
    source: { nodeId: 0, primitiveId: 0, componentIndex: 1 },
    parentId: null,
    transform: transform([20, -0.5, 0]),
    flags: { hidden: false, locked: false, renderable: true, includeInCollision: true, castShadow: true },
    deleted: false,
  }],
  collision: {
    schemaVersion: 1,
    mode: "AUTO_RECTANGLE",
    coordinateSpace: "GLTF_SOURCE_XZ_METERS",
    paddingMeters: 0,
    vertices: [],
  },
};

describe("placeable authoring diagnostics", () => {
  it("measures authored elements and reports floating, below-ground, distant, gaps and source topology", () => {
    const measurements = measurePlaceableElements(document, inspection);
    expect(measurements[0].boundsMin[1]).toBeCloseTo(0.2);
    const diagnostics = diagnosePlaceableAuthoring(document, inspection);
    expect(diagnostics.map((diagnostic) => diagnostic.code)).toEqual(expect.arrayContaining([
      "ELEMENT_FLOATING",
      "ELEMENT_BELOW_GROUND",
      "ELEMENT_DISTANT",
      "SOURCE_DISCONNECTED_COMPONENTS",
    ]));
  });

  it("fails the authoring preflight when every source element is excluded from collision", () => {
    const withoutCollision: PlaceableAuthoringDocument = {
      ...document,
      elements: document.elements.map((element) => ({
        ...element,
        flags: { ...element.flags, includeInCollision: false },
      })),
    };
    expect(diagnosePlaceableAuthoring(withoutCollision, inspection)).toContainEqual(
      expect.objectContaining({ code: "COLLISION_EMPTY", severity: "ERROR" }),
    );
  });

  it("skips quadratic gap analysis for large component documents", () => {
    const large: PlaceableAuthoringDocument = {
      ...document,
      elements: Array.from({ length: 513 }, (_, index) => ({
        ...document.elements[0],
        id: `component-${index}`,
        transform: transform([index * 0.001, 0, 0]),
      })),
    };
    const diagnostics = diagnosePlaceableAuthoring(large, inspection);
    expect(diagnostics).toContainEqual(expect.objectContaining({ code: "GAP_ANALYSIS_SKIPPED" }));
    expect(diagnostics.some(({ code }) => code === "GAP_BETWEEN_ELEMENTS")).toBe(false);
  });
});
