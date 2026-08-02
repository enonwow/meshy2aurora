import { describe, expect, it } from "vitest";
import {
  overrideModelTextureBindingV1,
  prepareModelTexturePayloadsV1,
  sourceModelTextureBindingV1,
} from "./texturePayloads";
import type { ModelTextureBindingAuthoringV1 } from "./types";

const sourceBinding = (slot: number): ModelTextureBindingAuthoringV1 => ({
  authoredMaterialId: `material:${slot}`,
  materialSlot: slot,
  sourceMaterialId: 0,
  sourceImageSha256: "a".repeat(64),
  mode: "SOURCE",
  overrideAssetId: null,
  overrideSha256: null,
  overrideMimeType: null,
  overrideByteLength: null,
  alphaPolicy: "OPAQUE_ONLY",
});

describe("neutral model texture payload preparation", () => {
  it("binds an exact override and returns to immutable SOURCE metadata", async () => {
    const file = new File([new Uint8Array([137, 80, 78, 71])], "hull.png", { type: "image/png" });
    const override = await overrideModelTextureBindingV1(sourceBinding(0), file);
    expect(override.binding).toMatchObject({
      mode: "OVERRIDE",
      overrideAssetId: override.assetId,
      overrideMimeType: "image/png",
      overrideByteLength: 4,
    });
    expect(override.binding.overrideSha256).toMatch(/^[0-9a-f]{64}$/);
    expect(sourceModelTextureBindingV1(override.binding)).toEqual(sourceBinding(0));
  });

  it("packs referenced files in canonical asset-id order with an exact envelope", async () => {
    const firstFile = new File([new Uint8Array([1, 2, 3])], "first.png", { type: "image/png" });
    const secondFile = new File([new Uint8Array([4, 5])], "second.jpg", { type: "image/jpeg" });
    const first = await overrideModelTextureBindingV1(sourceBinding(0), firstFile);
    const second = await overrideModelTextureBindingV1(sourceBinding(1), secondFile);
    const files = new Map<string, File>([
      [first.assetId, firstFile],
      [second.assetId, secondFile],
    ]);
    const prepared = await prepareModelTexturePayloadsV1({
      document: {
        schemaVersion: 1,
        sourceSha256: "b".repeat(64),
        separationSha256: "c".repeat(64),
        bindings: [second.binding, first.binding],
      },
      files,
    });
    const descriptors = JSON.parse(prepared.descriptorsJson) as Array<{
      assetId: string;
      byteOffset: number;
      byteLength: number;
    }>;
    expect(descriptors.map((descriptor) => descriptor.assetId)).toEqual(
      [...files.keys()].sort(),
    );
    expect(descriptors[0].byteOffset).toBe(0);
    expect(descriptors[1].byteOffset).toBe(descriptors[0].byteLength);
    expect(descriptors.reduce((sum, descriptor) => sum + descriptor.byteLength, 0))
      .toBe(prepared.payloadBlob.byteLength);
  });
});
