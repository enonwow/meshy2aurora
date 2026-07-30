import { describe, expect, it } from "vitest";
import {
  canonicalSkinAccessoryComponentBoneOverridesV2,
  parseSkinAccessoryComponentBoneOverridesV2,
} from "./skinAccessoryOverrides";

describe("skin accessory component overrides", () => {
  it("parses, sorts, and canonicalizes explicit component bindings", () => {
    expect(parseSkinAccessoryComponentBoneOverridesV2(
      "2:4=Spine\n0:1=Spine02",
    )).toEqual([
      { segmentIndex: 0, componentIndex: 1, boneName: "Spine02" },
      { segmentIndex: 2, componentIndex: 4, boneName: "Spine" },
    ]);
    expect(canonicalSkinAccessoryComponentBoneOverridesV2(
      "2:4=SPINE\n0:1=Spine02",
    )).toBe("0:1=spine02\n2:4=spine");
  });

  it("rejects malformed and duplicate component bindings", () => {
    expect(() => parseSkinAccessoryComponentBoneOverridesV2("0/1=Spine"))
      .toThrow(/segment:component=BoneName/);
    expect(() => parseSkinAccessoryComponentBoneOverridesV2("0:1=Spine\n0:1=Hips"))
      .toThrow(/declared more than once/);
  });
});
