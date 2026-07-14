// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import { buildM7PayloadEnvelope } from "./envelope";

const file = (name: string, bytes: number[]) => ({
  name,
  size: bytes.length,
  arrayBuffer: async () => new Uint8Array(bytes).buffer,
}) as File;

describe("M7 transferable payload envelope", () => {
  it("packs sources before appearances with exact deterministic offsets", async () => {
    const envelope = await buildM7PayloadEnvelope(
      [
        { role: "SOURCE", relativePath: "a.glb", file: file("a.glb", [1, 2]) },
        { role: "SOURCE", relativePath: "b.glb", file: file("b.glb", [3]) },
      ],
      [{
        role: "RIGGED_HUMANOID_APPEARANCE_2DA",
        sampleId: "humanoid",
        file: file("appearance.2da", [4, 5]),
      }],
    );

    expect([...new Uint8Array(envelope.payloadBlob)]).toEqual([1, 2, 3, 4, 5]);
    expect(JSON.parse(envelope.descriptorsJson)).toEqual({
      schemaVersion: 1,
      payloads: [
        { role: "SOURCE", relativePath: "a.glb", payloadOffset: 0, payloadSize: 2 },
        { role: "SOURCE", relativePath: "b.glb", payloadOffset: 2, payloadSize: 1 },
        {
          role: "RIGGED_HUMANOID_APPEARANCE_2DA",
          sampleId: "humanoid",
          payloadOffset: 3,
          payloadSize: 2,
        },
      ],
    });
  });

  it("rejects oversized metadata before reading any file", async () => {
    let reads = 0;
    const oversized = {
      name: "oversized.glb",
      size: 0x1_0000_0000,
      arrayBuffer: async () => {
        reads += 1;
        return new ArrayBuffer(0);
      },
    } as File;

    await expect(buildM7PayloadEnvelope(
      [{ role: "SOURCE", relativePath: "oversized.glb", file: oversized }],
      [],
    )).rejects.toThrow("invalid u32 byte size");
    expect(reads).toBe(0);
  });
});
