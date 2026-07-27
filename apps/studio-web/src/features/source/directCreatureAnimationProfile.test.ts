import { describe, expect, it } from "vitest";
import {
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
});
