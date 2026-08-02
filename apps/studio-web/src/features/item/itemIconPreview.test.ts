import { describe, expect, it } from "vitest";
import {
  compositeItemIconLayers,
  decodeItemIconTga,
} from "./itemIconPreview";

function tga(width: number, height: number, bgra: number[]) {
  const bytes = new Uint8Array(18 + bgra.length);
  bytes[2] = 2;
  new DataView(bytes.buffer).setUint16(12, width, true);
  new DataView(bytes.buffer).setUint16(14, height, true);
  bytes[16] = 32;
  bytes[17] = 8;
  bytes.set(bgra, 18);
  return bytes.buffer;
}

describe("item icon artifact preview", () => {
  it("decodes bottom-left BGRA TGA pixels into top-left RGBA pixels", () => {
    const decoded = decodeItemIconTga(tga(1, 2, [
      30, 20, 10, 255,
      60, 50, 40, 128,
    ]));
    expect([...decoded.rgba]).toEqual([
      40, 50, 60, 128,
      10, 20, 30, 255,
    ]);
  });

  it("composites exact layers with source-over alpha", () => {
    const composite = compositeItemIconLayers([
      { width: 1, height: 1, rgba: new Uint8ClampedArray([200, 0, 0, 255]) },
      { width: 1, height: 1, rgba: new Uint8ClampedArray([0, 0, 200, 128]) },
    ]);
    expect([...composite.rgba]).toEqual([100, 0, 100, 255]);
  });
});
