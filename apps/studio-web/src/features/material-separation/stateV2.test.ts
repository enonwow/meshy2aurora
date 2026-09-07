import { describe, expect, it } from "vitest";
import type { ModelMaterialSeparationDocumentV2 } from "./types";
import {
  compactFaceAssignmentsV2,
  createMaterialSeparationEditorStateV2,
  materialSeparationReducerV2,
} from "./stateV2";

const sourceSha256 = "a".repeat(64);
const document: ModelMaterialSeparationDocumentV2 = {
  schemaVersion: 2,
  sourceSha256,
  materials: [{
    authoredMaterialId: "material:wood",
    displayName: "Wood",
    previewColor: "#704020",
    sourceFallbackMaterialId: 0,
    sourceFallbackImageSha256: null,
  }],
  componentAssignments: [],
  faceAssignments: [],
};

describe("Face Mode V2 state", () => {
  it("compacts consecutive face ordinals into canonical ranges", () => {
    const assignments = new Map([
      ["0/0/0/3", "material:wood"],
      ["0/0/0/1", "material:wood"],
      ["0/0/0/2", "material:wood"],
      ["0/0/0/8", "material:wood"],
    ]);
    expect(compactFaceAssignmentsV2(assignments)).toEqual([{
      selection: {
        sceneId: 0,
        nodeId: 0,
        primitiveId: 0,
        triangleRanges: [
          { startTriangle: 1, triangleCount: 3 },
          { startTriangle: 8, triangleCount: 1 },
        ],
      },
      authoredMaterialId: "material:wood",
    }]);
  });

  it("paints, reassigns, unassigns and restores exact face selections", () => {
    let state = createMaterialSeparationEditorStateV2(document);
    state = materialSeparationReducerV2(state, {
      type: "SELECT_FACES",
      keys: ["0/0/0/1", "0/0/0/2", "0/0/0/3"],
    });
    state = materialSeparationReducerV2(state, {
      type: "ASSIGN_SELECTION",
      authoredMaterialId: "material:wood",
    });
    expect(state.draft.faceAssignments[0].selection.triangleRanges).toEqual([
      { startTriangle: 1, triangleCount: 3 },
    ]);
    state = materialSeparationReducerV2(state, { type: "APPLY" });
    state = materialSeparationReducerV2(state, { type: "UNASSIGN_SELECTION" });
    expect(state.draft.faceAssignments).toEqual([]);
    state = materialSeparationReducerV2(state, { type: "CANCEL" });
    expect(state.draft.faceAssignments).toHaveLength(1);
  });
});
