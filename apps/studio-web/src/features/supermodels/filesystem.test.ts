import { describe, expect, it } from "vitest";
import {
  fileListNwnResourceSourceV1,
  mergeCatalogHeaderSlicesV1,
} from "./filesystem";

function relativeFile(path: string, bytes = new Uint8Array([1])) {
  const file = new File([bytes], path.split("/").at(-1) ?? "file.bin");
  Object.defineProperty(file, "webkitRelativePath", { value: path });
  return file;
}

describe("supermodel local resource access", () => {
  it("resolves a selected installation tree without exposing an absolute local path", async () => {
    const key = relativeFile("Neverwinter Nights/data/nwn_base.key");
    const bif = relativeFile("Neverwinter Nights/data/models_01.bif");
    const source = fileListNwnResourceSourceV1([key, bif]);
    expect(await source.keyFile()).toBe(key);
    expect(await source.bifFile("data/models_01.bif")).toBe(bif);
    expect(source.label).toBe("Neverwinter Nights");
  });

  it("merges only selected catalog slices and produces deterministic descriptors", () => {
    const result = mergeCatalogHeaderSlicesV1([
      { itemId: "2", declaredPayloadSize: 244, bytes: new Uint8Array([1, 2]).buffer },
      { itemId: "7", declaredPayloadSize: 512, bytes: new Uint8Array([3]).buffer },
    ]);
    expect([...new Uint8Array(result.payloadBlob)]).toEqual([1, 2, 3]);
    expect(result.descriptors).toEqual([
      { itemId: "2", byteOffset: 0, byteLength: 2, declaredPayloadSize: 244 },
      { itemId: "7", byteOffset: 2, byteLength: 1, declaredPayloadSize: 512 },
    ]);
  });

  it("fails explicitly when the selected tree omits a referenced BIF", async () => {
    const source = fileListNwnResourceSourceV1([
      relativeFile("Neverwinter Nights/data/nwn_base.key"),
    ]);
    await expect(source.bifFile("data/models_01.bif")).rejects.toThrow(/not selected/i);
  });
});
