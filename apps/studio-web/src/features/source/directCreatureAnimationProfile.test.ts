import { describe, expect, it } from "vitest";
import {
  canOfferGeneratedHumanoidProfileV1,
  FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
  hasFullNativeDirectCreatureProfileV1,
} from "./directCreatureAnimationProfile";

describe("full native direct-creature animation profile", () => {
  it("requires the exact 42-name namespace and accepts ASCII case differences", () => {
    expect(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1).toHaveLength(42);
    expect(
      hasFullNativeDirectCreatureProfileV1(
        FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((name) => ({ name: name.toUpperCase() })),
      ),
    ).toBe(true);

    expect(
      hasFullNativeDirectCreatureProfileV1(
        FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.slice(0, -1).map((name) => ({ name })),
      ),
    ).toBe(false);
    expect(
      hasFullNativeDirectCreatureProfileV1(
        FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((name, index) => ({
          name: index === 41 ? FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1[0] : name,
        })),
      ),
    ).toBe(false);
    expect(
      hasFullNativeDirectCreatureProfileV1([
        ...FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((name) => ({ name })),
        { name: "custom_extra" },
      ]),
    ).toBe(false);
  });

  it("offers procedural Base 42 for a skinned humanoid without requiring a raw cpause1 name", () => {
    expect(canOfferGeneratedHumanoidProfileV1({
      skinCount: 1,
      boneCount: 22,
      clips: [{ name: "meshy_animation_0" }],
    })).toBe(true);
    expect(canOfferGeneratedHumanoidProfileV1({
      skinCount: 0,
      boneCount: 22,
      clips: [{ name: "meshy_animation_0" }],
    })).toBe(false);
    expect(canOfferGeneratedHumanoidProfileV1({
      skinCount: 1,
      boneCount: 10,
      clips: [{ name: "cpause1" }],
    })).toBe(false);
    expect(canOfferGeneratedHumanoidProfileV1({
      skinCount: 1,
      boneCount: 22,
      clips: [],
    })).toBe(false);
  });
});
