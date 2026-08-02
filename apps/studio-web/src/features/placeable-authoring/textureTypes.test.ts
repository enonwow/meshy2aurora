import { describe, expect, it } from "vitest";
import {
  createTextureOverrideBindingV1,
  preparePlaceableTexturePayloadsV1,
  type PlaceableTextureAuthoringDocument,
} from "./textureTypes";

const sourceBinding = {
  materialSlot: 0,
  sourceMaterialId: 4,
  sourceMaterialName: "hull",
  sourceImageSha256: "a".repeat(64),
  mode: "SOURCE" as const,
  overrideAssetId: null,
  overrideSha256: null,
  overrideMimeType: null,
  overrideByteLength: null,
  alphaPolicy: "OPAQUE_ONLY" as const,
};

describe("placeable texture payload preparation", () => {
  it("binds exact PNG bytes outside JSON and rejects stale payloads", async () => {
    const file = new File([new Uint8Array([1, 2, 3, 4])], "hull.png", { type: "image/png" });
    const replacement = await createTextureOverrideBindingV1(sourceBinding, file);
    const document: PlaceableTextureAuthoringDocument = {
      schemaVersion: 1,
      sourceSha256: "f".repeat(64),
      bindings: [replacement.binding],
    };
    const prepared = await preparePlaceableTexturePayloadsV1({
      document,
      files: new Map([[replacement.assetId, file]]),
      preview: "EDITED",
      selectedMaterialSlot: 0,
    });
    expect([...new Uint8Array(prepared.payloadBlob)]).toEqual([1, 2, 3, 4]);
    expect(JSON.parse(prepared.descriptorsJson)).toEqual([expect.objectContaining({
      assetId: replacement.assetId,
      byteOffset: 0,
      byteLength: 4,
      mimeType: "image/png",
    })]);
    expect(JSON.parse(prepared.authoringJson)).toEqual(document);

    const stale = new File([new Uint8Array([9])], "hull.png", { type: "image/png" });
    await expect(preparePlaceableTexturePayloadsV1({
      document,
      files: new Map([[replacement.assetId, stale]]),
      preview: "EDITED",
      selectedMaterialSlot: 0,
    })).rejects.toThrow("PLACEABLE-TEXTURE-PAYLOAD-STALE");
  });
});
