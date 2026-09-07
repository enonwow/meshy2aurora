import { describe, expect, it } from "vitest";
import { parseNwnDdsV1 } from "./nwnDds";

function nwnDds(width: number, height: number, channels: 3 | 4, payloadSize: number) {
  const bytes = new Uint8Array(20 + payloadSize);
  const view = new DataView(bytes.buffer);
  view.setUint32(0, width, true);
  view.setUint32(4, height, true);
  view.setUint32(8, channels, true);
  const blockBytes = channels === 3 ? 8 : 16;
  const topSize = Math.max(1, Math.ceil(width / 4)) * Math.max(1, Math.ceil(height / 4)) * blockBytes;
  view.setUint32(12, topSize, true);
  view.setFloat32(16, 1, true);
  bytes.fill(0x7f, 20);
  return bytes.buffer;
}

describe("BioWare/NWN DDS", () => {
  it("decodes a DXT1 mip chain after the exact 20-byte NWN header", () => {
    const parsed = parseNwnDdsV1(nwnDds(8, 8, 3, 40));
    expect(parsed).toMatchObject({ width: 8, height: 8, format: "DXT1", alphaMean: 1 });
    expect(parsed.mipmaps.map(({ width, height, data }) => [width, height, data.byteLength]))
      .toEqual([[8, 8, 32], [4, 4, 8]]);
  });

  it("decodes DXT5 and rejects unsupported channels or a partial mip payload", () => {
    expect(parseNwnDdsV1(nwnDds(4, 4, 4, 16))).toMatchObject({ format: "DXT5" });
    const unsupported = new Uint8Array(nwnDds(4, 4, 3, 8));
    new DataView(unsupported.buffer).setUint32(8, 2, true);
    expect(() => parseNwnDdsV1(unsupported.buffer)).toThrow(/channels/i);
    expect(() => parseNwnDdsV1(nwnDds(8, 8, 3, 39))).toThrow(/mip payload/i);
  });
});
