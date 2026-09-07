import { describe, expect, it } from "vitest";
import {
  canonicalCreatureWeaponGripOptionsV1,
  defaultCreatureWeaponGripOptionsV1,
  sameCreatureWeaponGripOptionsV1,
} from "./weaponGrip";

describe("Creature weapon grip options", () => {
  it("keeps the legacy Auto policy canonical and deterministic", () => {
    const automatic = defaultCreatureWeaponGripOptionsV1();
    expect(automatic.itemFamily).toBe("SWORD");
    expect(canonicalCreatureWeaponGripOptionsV1(automatic)).toEqual(automatic);
    expect(sameCreatureWeaponGripOptionsV1(automatic, structuredClone(automatic))).toBe(true);
  });

  it("normalizes negative zero without changing finite manual offsets", () => {
    const value = canonicalCreatureWeaponGripOptionsV1({
      schemaVersion: 1,
      mode: "AUTO_PLUS_OFFSETS",
      rightHand: { rollDegrees: -0, pitchDegrees: 12.5, yawDegrees: -7 },
      leftHand: { rollDegrees: 180, pitchDegrees: -180, yawDegrees: 0 },
    });
    expect(Object.is(value.rightHand.rollDegrees, -0)).toBe(false);
    expect(value).toMatchObject({
      rightHand: { rollDegrees: 0, pitchDegrees: 12.5, yawDegrees: -7 },
      leftHand: { rollDegrees: 180, pitchDegrees: -180, yawDegrees: 0 },
    });
  });

  it("fails closed for hidden Auto offsets, non-finite input, and values outside the supported range", () => {
    const automatic = defaultCreatureWeaponGripOptionsV1();
    expect(() => canonicalCreatureWeaponGripOptionsV1({
      ...automatic,
      rightHand: { ...automatic.rightHand, rollDegrees: 1 },
    })).toThrow(/AUTO mode requires zero/i);
    expect(() => canonicalCreatureWeaponGripOptionsV1({
      ...automatic,
      mode: "AUTO_PLUS_OFFSETS",
      rightHand: { ...automatic.rightHand, pitchDegrees: Number.NaN },
    })).toThrow(/finite/i);
    expect(() => canonicalCreatureWeaponGripOptionsV1({
      ...automatic,
      mode: "AUTO_PLUS_OFFSETS",
      leftHand: { ...automatic.leftHand, yawDegrees: 180.1 },
    })).toThrow(/\[-180, 180\]/i);
  });
});
