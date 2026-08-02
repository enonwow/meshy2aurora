import { describe, expect, it } from "vitest";
import {
  MATERIAL_SEPARATION_HISTORY_LIMIT,
  createMaterialSeparationEditorState,
  materialSeparationReducer,
} from "./state";
import type { ModelComponentInventoryV1, ModelMaterialSeparationDocumentV1 } from "./types";

const sourceSha256 = "a".repeat(64);
const document: ModelMaterialSeparationDocumentV1 = {
  schemaVersion: 1,
  sourceSha256,
  materials: [],
  assignments: [],
};
const inventory: ModelComponentInventoryV1 = {
  schemaVersion: 1,
  sourceSha256,
  sceneId: 0,
  renderNodeCount: 1,
  primitiveInstanceCount: 1,
  triangleCount: 2,
  components: [0, 1].map((componentIndex) => ({
    key: { sceneId: 0, nodeId: 3, primitiveId: 7, componentIndex },
    sourceMaterialId: 0,
    sourceMaterialName: "Source",
    triangleCount: 1,
    vertexCount: 3,
    boundsMin: [componentIndex * 2, 0, 0] as const,
    boundsMax: [componentIndex * 2 + 1, 1, 1] as const,
  })),
};
const wood = {
  authoredMaterialId: "material:wood",
  displayName: "Wood",
  previewColor: "#704020",
  sourceFallbackMaterialId: 0,
  sourceFallbackImageSha256: null,
} as const;

describe("materialSeparationReducer", () => {
  it("expands node/primitive selection and supports assign, unassign and material queries", () => {
    let state = createMaterialSeparationEditorState(document);
    state = materialSeparationReducer(state, { type: "NEW_MATERIAL", material: wood });
    state = materialSeparationReducer(state, { type: "SELECT_NODE", nodeId: 3, inventory });
    expect(state.selectedComponentKeys).toHaveLength(2);
    state = materialSeparationReducer(state, { type: "ASSIGN_SELECTION", authoredMaterialId: wood.authoredMaterialId });
    expect(state.draft.assignments).toHaveLength(2);
    state = materialSeparationReducer(state, { type: "CLEAR_SELECTION" });
    state = materialSeparationReducer(state, { type: "SELECT_BY_MATERIAL", authoredMaterialId: wood.authoredMaterialId });
    expect(state.selectedComponentKeys).toHaveLength(2);
    state = materialSeparationReducer(state, { type: "SELECT_COMPONENT", key: "0/3/7/0" });
    state = materialSeparationReducer(state, { type: "UNASSIGN_SELECTION" });
    expect(state.draft.assignments).toHaveLength(1);
    state = materialSeparationReducer(state, { type: "SELECT_UNASSIGNED", inventory });
    expect(state.selectedComponentKeys).toEqual(["0/3/7/0"]);
    state = materialSeparationReducer(state, { type: "SELECT_PRIMITIVE", nodeId: 3, primitiveId: 7, inventory });
    expect(state.selectedComponentKeys).toHaveLength(2);
  });

  it("keeps Apply, Cancel and Reset explicit while undo/redo history stays bounded", () => {
    let state = createMaterialSeparationEditorState(document);
    state = materialSeparationReducer(state, { type: "NEW_MATERIAL", material: wood });
    expect(state.applied.materials).toHaveLength(0);
    state = materialSeparationReducer(state, { type: "APPLY" });
    expect(state.applied.materials).toHaveLength(1);
    state = materialSeparationReducer(state, {
      type: "RENAME_MATERIAL",
      authoredMaterialId: wood.authoredMaterialId,
      displayName: "Hull",
    });
    state = materialSeparationReducer(state, { type: "CANCEL" });
    expect(state.draft.materials[0].displayName).toBe("Wood");
    state = materialSeparationReducer(state, { type: "RESET" });
    expect(state.draft.materials).toHaveLength(0);
    state = materialSeparationReducer(state, { type: "UNDO" });
    expect(state.draft.materials).toHaveLength(1);
    state = materialSeparationReducer(state, { type: "REDO" });
    expect(state.draft.materials).toHaveLength(0);

    for (let index = 0; index < MATERIAL_SEPARATION_HISTORY_LIMIT + 10; index += 1) {
      state = materialSeparationReducer(state, {
        type: "NEW_MATERIAL",
        material: { ...wood, authoredMaterialId: `material:m-${index}`, displayName: `M ${index}` },
      });
    }
    expect(state.past).toHaveLength(MATERIAL_SEPARATION_HISTORY_LIMIT);
  });

  it("toggles overlay and isolate without mutating the recipe", () => {
    const state = createMaterialSeparationEditorState(document);
    const overlay = materialSeparationReducer(state, { type: "TOGGLE_OVERLAY" });
    const isolate = materialSeparationReducer(overlay, { type: "TOGGLE_ISOLATE" });
    expect(isolate.overlayEnabled).toBe(false);
    expect(isolate.isolateSelection).toBe(true);
    expect(isolate.draft).toEqual(document);
  });
});
