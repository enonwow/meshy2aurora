import { describe, expect, it } from "vitest";
import { itemWeaponColorwayMultiplier } from "./itemColorway";

describe("Item weapon colorway preview", () => {
  it("uses the same four deterministic color slots as concrete TGA authoring", () => {
    expect(itemWeaponColorwayMultiplier(1)).toEqual([1, 1, 1]);
    expect(itemWeaponColorwayMultiplier(2)).toEqual([0.75, 0.875, 1.125]);
    expect(itemWeaponColorwayMultiplier(3)).toEqual([1.125, 0.75, 0.5]);
    expect(itemWeaponColorwayMultiplier(4)).toEqual([0.5, 0.5, 0.5]);
  });
});
