import { describe, expect, it } from "vitest";
import { createPlaceableAuthoringEditorState, placeableAuthoringReducer } from "./state";
import type { PlaceableAuthoringBootstrap, PlaceableAuthoringDocument } from "./types";

const element = (id: string, nodeId: number) => ({
  id,
  name: id,
  kind: "SOURCE_NODE" as const,
  source: { nodeId, primitiveId: null, componentIndex: null },
  parentId: null,
  transform: {
    translation: [0, 0, 0] as [number, number, number],
    rotationXyzw: [0, 0, 0, 1] as [number, number, number, number],
    scale: [1, 1, 1] as [number, number, number],
    pivot: [0, 0, 0] as [number, number, number],
  },
  flags: {
    hidden: false,
    locked: false,
    renderable: true,
    includeInCollision: true,
    castShadow: true,
  },
  deleted: false,
});

const document: PlaceableAuthoringDocument = {
  schemaVersion: 2,
  sourceSha256: "a".repeat(64),
  elements: [element("node-0", 0), element("node-1", 1)],
  collision: {
    schemaVersion: 1,
    mode: "AUTO_RECTANGLE",
    coordinateSpace: "GLTF_SOURCE_XZ_METERS",
    paddingMeters: 0,
    vertices: [],
  },
};

const bootstrap: PlaceableAuthoringBootstrap = {
  schemaVersion: 2,
  document,
  inspection: {
    schemaVersion: 1,
    sourceSha256: document.sourceSha256,
    renderNodeCount: 2,
    primitiveCount: 2,
    connectedComponentCount: 3,
    nodes: [{
      elementId: "node-0",
      nodeId: 0,
      name: "cauldron",
      meshId: 0,
      primitives: [{
        primitiveId: 0,
        materialId: 0,
        triangleCount: 2,
        components: [
          { elementId: "node-0-p0-c0", componentIndex: 0, triangleCount: 1, vertexCount: 3, boundsMin: [0, 0, 0], boundsMax: [1, 1, 1] },
          { elementId: "node-0-p0-c1", componentIndex: 1, triangleCount: 1, vertexCount: 3, boundsMin: [2, 0, 0], boundsMax: [3, 1, 1] },
        ],
      }],
    }, {
      elementId: "node-1",
      nodeId: 1,
      name: "basin",
      meshId: 1,
      primitives: [{
        primitiveId: 0,
        materialId: 0,
        triangleCount: 1,
        components: [
          { elementId: "node-1-p0-c0", componentIndex: 0, triangleCount: 1, vertexCount: 3, boundsMin: [0, 0, 0], boundsMax: [1, 1, 1] },
        ],
      }],
    }],
  },
};

describe("placeable authoring reducer", () => {
  it("splits stable connected components and supports deterministic duplicate/delete undo/redo", () => {
    let state = createPlaceableAuthoringEditorState(document);
    state = placeableAuthoringReducer(state, { type: "SELECT", id: "node-0" });
    state = placeableAuthoringReducer(state, {
      type: "SPLIT_NODE_COMPONENTS",
      id: "node-0",
      inspection: bootstrap.inspection,
    });
    expect(state.present.elements.map((candidate) => candidate.id)).toEqual([
      "node-0-p0-c0",
      "node-0-p0-c1",
      "node-1",
    ]);

    state = placeableAuthoringReducer(state, { type: "DUPLICATE" });
    expect(state.present.elements.filter((candidate) => candidate.kind === "COPY")).toHaveLength(2);
    const afterDuplicate = state.present;
    state = placeableAuthoringReducer(state, { type: "DELETE" });
    expect(state.present.elements.filter((candidate) => candidate.deleted)).toHaveLength(2);
    state = placeableAuthoringReducer(state, { type: "UNDO" });
    expect(state.present).toEqual(afterDuplicate);
    state = placeableAuthoringReducer(state, { type: "REDO" });
    expect(state.present.elements.filter((candidate) => candidate.deleted)).toHaveLength(2);
  });

  it("groups and reparents without changing world transforms", () => {
    let state = createPlaceableAuthoringEditorState(document);
    state = placeableAuthoringReducer(state, { type: "SELECT", id: "node-0" });
    state = placeableAuthoringReducer(state, { type: "SELECT", id: "node-1", additive: true });
    state = placeableAuthoringReducer(state, { type: "GROUP" });
    const group = state.present.elements.find((candidate) => candidate.kind === "GROUP");
    expect(group).toBeDefined();
    expect(state.present.elements.filter((candidate) => candidate.parentId === group?.id)).toHaveLength(2);

    state = placeableAuthoringReducer(state, { type: "UNGROUP" });
    expect(state.present.elements.find((candidate) => candidate.id === group?.id)?.deleted).toBe(true);
    expect(state.present.elements.filter((candidate) => !candidate.deleted && candidate.parentId)).toHaveLength(0);
  });

  it("commits one history entry for a continuous gizmo gesture", () => {
    let state = createPlaceableAuthoringEditorState(document);
    state = placeableAuthoringReducer(state, { type: "BEGIN_GESTURE" });
    for (const x of [1, 2, 3]) {
      state = placeableAuthoringReducer(state, {
        type: "PREVIEW_TRANSFORMS",
        patches: [{
          id: "node-0",
          transform: { ...document.elements[0].transform, translation: [x, 0, 0] },
        }],
      });
    }
    state = placeableAuthoringReducer(state, { type: "END_GESTURE" });
    expect(state.past).toHaveLength(1);
    expect(state.present.elements[0].transform.translation).toEqual([3, 0, 0]);
    state = placeableAuthoringReducer(state, { type: "UNDO" });
    expect(state.present).toEqual(document);
  });

  it("maintains per-element export flags and copy/paste/mirror operations", () => {
    let state = createPlaceableAuthoringEditorState(document);
    state = placeableAuthoringReducer(state, { type: "SELECT", id: "node-0" });
    state = placeableAuthoringReducer(state, {
      type: "SET_TRANSFORMS",
      patches: [{
        id: "node-0",
        transform: { ...document.elements[0].transform, scale: [2, 3, 4] },
      }],
    });
    state = placeableAuthoringReducer(state, { type: "COPY_TRANSFORM" });
    state = placeableAuthoringReducer(state, { type: "SELECT", id: "node-1" });
    state = placeableAuthoringReducer(state, { type: "PASTE_TRANSFORM" });
    state = placeableAuthoringReducer(state, { type: "MIRROR", axis: 0 });
    state = placeableAuthoringReducer(state, {
      type: "PATCH_FLAGS",
      ids: ["node-1"],
      flags: { includeInCollision: false, castShadow: false },
    });
    const changed = state.present.elements[1];
    expect(changed.transform.scale).toEqual([-2, 3, 4]);
    expect(changed.flags.includeInCollision).toBe(false);
    expect(changed.flags.castShadow).toBe(false);
  });

  it("authors a custom collision polygon as one undoable document change", () => {
    let state = createPlaceableAuthoringEditorState(document);
    state = placeableAuthoringReducer(state, {
      type: "SET_COLLISION_MODE",
      mode: "CUSTOM_POLYGON",
      seedVertices: [[-1, -1], [1, -1], [1, 1], [-1, 1]],
    });
    state = placeableAuthoringReducer(state, {
      type: "MOVE_COLLISION_VERTEX",
      index: 2,
      vertex: [0.25, 0.5],
    });
    expect(state.present.collision).toMatchObject({
      mode: "CUSTOM_POLYGON",
      paddingMeters: 0,
      vertices: [[-1, -1], [1, -1], [0.25, 0.5], [-1, 1]],
    });
    expect(state.past).toHaveLength(2);
    state = placeableAuthoringReducer(state, { type: "UNDO" });
    expect(state.present.collision.vertices[2]).toEqual([1, 1]);
  });
});
