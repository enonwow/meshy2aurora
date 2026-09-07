import { describe, expect, it } from "vitest";
import {
  locateHakResourceV1,
  locateHakModelResourceV1,
  parseHakRangeIndexPlanV1,
} from "./hakRange";

function writeU32(bytes: Uint8Array, offset: number, value: number) {
  new DataView(bytes.buffer).setUint32(offset, value, true);
}

function header(entryCount = 2) {
  const bytes = new Uint8Array(160);
  bytes.set(new TextEncoder().encode("HAK V1.0"), 0);
  writeU32(bytes, 16, entryCount);
  writeU32(bytes, 24, 160);
  writeU32(bytes, 28, 160 + entryCount * 24);
  return bytes.buffer;
}

function tables() {
  const keys = new Uint8Array(48);
  keys.set(new TextEncoder().encode("other_model"), 0);
  writeU32(keys, 16, 1);
  new DataView(keys.buffer).setUint16(20, 2002, true);
  keys.set(new TextEncoder().encode("fwp_cobr_sap"), 24);
  writeU32(keys, 40, 0);
  new DataView(keys.buffer).setUint16(44, 2002, true);

  const resources = new Uint8Array(16);
  writeU32(resources, 0, 4096);
  writeU32(resources, 4, 2048);
  writeU32(resources, 8, 8192);
  writeU32(resources, 12, 1024);
  return { keys: keys.buffer, resources: resources.buffer };
}

describe("HAK range index", () => {
  it("plans only the fixed key and resource table ranges", () => {
    expect(parseHakRangeIndexPlanV1(header())).toEqual({
      entryCount: 2,
      keyTableOffset: 160,
      keyTableByteLength: 48,
      resourceTableOffset: 208,
      resourceTableByteLength: 16,
    });
  });

  it("locates an exact MDL payload through its resource id", () => {
    const { keys, resources } = tables();
    expect(locateHakModelResourceV1(keys, resources, 2, "FWP_COBR_SAP")).toEqual({
      resref: "fwp_cobr_sap",
      resourceId: 0,
      payloadOffset: 4096,
      payloadSize: 2048,
    });
  });

  it("locates a requested non-MDL resource type without confusing equal resrefs", () => {
    const { keys, resources } = tables();
    const textureKeys = new Uint8Array(keys);
    new DataView(textureKeys.buffer).setUint16(44, 2033, true);
    expect(locateHakResourceV1(textureKeys.buffer, resources, 2, "fwp_cobr_sap", 2033)).toEqual({
      resref: "fwp_cobr_sap",
      resourceId: 0,
      resourceType: 2033,
      payloadOffset: 4096,
      payloadSize: 2048,
    });
    expect(() => locateHakResourceV1(textureKeys.buffer, resources, 2, "fwp_cobr_sap", 3))
      .toThrow(/absent/i);
  });

  it("fails closed for a wrong signature, absent resref and invalid resource id", () => {
    const invalid = new Uint8Array(header());
    invalid.set(new TextEncoder().encode("MOD "), 0);
    expect(() => parseHakRangeIndexPlanV1(invalid.buffer)).toThrow(/HAK V1\.0/i);

    const { keys, resources } = tables();
    expect(() => locateHakModelResourceV1(keys, resources, 2, "missing")).toThrow(/absent/i);

    const badKeys = new Uint8Array(keys);
    writeU32(badKeys, 40, 2);
    expect(() => locateHakModelResourceV1(badKeys.buffer, resources, 2, "fwp_cobr_sap"))
      .toThrow(/resource id/i);
  });
});
