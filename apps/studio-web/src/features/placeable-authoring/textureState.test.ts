import { describe, expect, it } from "vitest";
import {
  createPlaceableTextureEditorState,
  placeableTextureEditorReducer,
} from "./textureState";
import type {
  PlaceableTextureAuthoringDocument,
  PlaceableTextureBindingAuthoring,
} from "./textureTypes";

const binding = (materialSlot: number): PlaceableTextureBindingAuthoring => ({
  materialSlot,
  sourceMaterialId: materialSlot + 10,
  sourceMaterialName: `material-${materialSlot}`,
  sourceImageSha256: String(materialSlot + 1).repeat(64),
  mode: "SOURCE",
  overrideAssetId: null,
  overrideSha256: null,
  overrideMimeType: null,
  overrideByteLength: null,
  alphaPolicy: "OPAQUE_ONLY",
});

const document: PlaceableTextureAuthoringDocument = {
  schemaVersion: 1,
  sourceSha256: "a".repeat(64),
  bindings: [binding(0), binding(1)],
};

describe("placeable texture editor reducer", () => {
  it("replaces exactly one material and supports independent undo/redo", () => {
    const initial = createPlaceableTextureEditorState(document);
    const replacement: PlaceableTextureBindingAuthoring = {
      ...document.bindings[1],
      mode: "OVERRIDE",
      overrideAssetId: "texture-b",
      overrideSha256: "b".repeat(64),
      overrideMimeType: "image/png",
      overrideByteLength: 3,
    };
    const edited = placeableTextureEditorReducer(initial, { type: "REPLACE", binding: replacement });
    expect(edited.present.bindings[0]).toEqual(document.bindings[0]);
    expect(edited.present.bindings[1]).toEqual(replacement);

    const undone = placeableTextureEditorReducer(edited, { type: "UNDO" });
    expect(undone.present).toEqual(document);
    const redone = placeableTextureEditorReducer(undone, { type: "REDO" });
    expect(redone.present.bindings[1]).toEqual(replacement);

    const source = placeableTextureEditorReducer(redone, { type: "USE_SOURCE", materialSlot: 1 });
    expect(source.present.bindings[1].mode).toBe("SOURCE");
    expect(source.present.bindings[1].overrideAssetId).toBeNull();
  });
});
