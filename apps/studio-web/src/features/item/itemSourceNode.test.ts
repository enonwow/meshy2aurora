import { describe, expect, it } from "vitest";
import { inspectItemPreviewSourceNode } from "./itemSourceNode";

function glb(document: unknown) {
  const encoded = new TextEncoder().encode(JSON.stringify(document));
  const jsonLength = (encoded.byteLength + 3) & ~3;
  const bytes = new Uint8Array(20 + jsonLength);
  const view = new DataView(bytes.buffer);
  view.setUint32(0, 0x4654_6c67, true);
  view.setUint32(4, 2, true);
  view.setUint32(8, bytes.byteLength, true);
  view.setUint32(12, jsonLength, true);
  view.setUint32(16, 0x4e4f_534a, true);
  bytes.fill(0x20, 20);
  bytes.set(encoded, 20);
  return bytes.buffer;
}

describe("item preview sourceNode contract", () => {
  it("returns only default-scene roots and accepts one globally unique root", () => {
    const result = inspectItemPreviewSourceNode(glb({
      scene: 0,
      scenes: [{ nodes: [0, 2] }, { nodes: [1] }],
      nodes: [{ name: "blade" }, { name: "other" }, { name: "guard", children: [3] }, { name: "child" }],
    }), "guard");
    expect(result).toEqual({ rootNames: ["blade", "guard"], selectedName: "guard" });
  });

  it("rejects a duplicate name even when the duplicate is outside the default scene", () => {
    expect(() => inspectItemPreviewSourceNode(glb({
      scene: 0,
      scenes: [{ nodes: [0] }, { nodes: [1] }],
      nodes: [{ name: "blade" }, { name: "blade" }],
    }), "blade")).toThrow("matched 2 nodes");
  });

  it("rejects a globally unique nested node", () => {
    expect(() => inspectItemPreviewSourceNode(glb({
      scene: 0,
      scenes: [{ nodes: [0] }],
      nodes: [{ name: "root", children: [1] }, { name: "blade" }],
    }), "blade")).toThrow("not a root");
  });
});
