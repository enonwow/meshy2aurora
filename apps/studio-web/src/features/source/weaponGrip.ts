export type CreatureWeaponGripModeV1 = "AUTO" | "AUTO_PLUS_OFFSETS";
export type CreatureItemGripFamilyV1 =
  | "SWORD"
  | "AXE_MACE"
  | "SPEAR_POLEARM"
  | "BOW_CROSSBOW"
  | "SHIELD";

export interface CreatureWeaponEulerOffsetV1 {
  readonly rollDegrees: number;
  readonly pitchDegrees: number;
  readonly yawDegrees: number;
}

export interface CreatureWeaponGripOptionsV1 {
  readonly schemaVersion: 1;
  readonly mode: CreatureWeaponGripModeV1;
  readonly itemFamily?: CreatureItemGripFamilyV1;
  readonly rightHand: CreatureWeaponEulerOffsetV1;
  readonly leftHand: CreatureWeaponEulerOffsetV1;
}

export const ZERO_WEAPON_EULER_OFFSET_V1: CreatureWeaponEulerOffsetV1 = Object.freeze({
  rollDegrees: 0,
  pitchDegrees: 0,
  yawDegrees: 0,
});

export function defaultCreatureWeaponGripOptionsV1(): CreatureWeaponGripOptionsV1 {
  return {
    schemaVersion: 1,
    mode: "AUTO",
    itemFamily: "SWORD",
    rightHand: { ...ZERO_WEAPON_EULER_OFFSET_V1 },
    leftHand: { ...ZERO_WEAPON_EULER_OFFSET_V1 },
  };
}

const canonicalAngle = (value: number, path: string) => {
  if (!Number.isFinite(value)) throw new Error(`${path} must be finite`);
  if (value < -180 || value > 180) throw new Error(`${path} must be within [-180, 180] degrees`);
  return Object.is(value, -0) ? 0 : value;
};

const canonicalOffset = (
  value: CreatureWeaponEulerOffsetV1,
  path: string,
): CreatureWeaponEulerOffsetV1 => ({
  rollDegrees: canonicalAngle(value.rollDegrees, `${path}.rollDegrees`),
  pitchDegrees: canonicalAngle(value.pitchDegrees, `${path}.pitchDegrees`),
  yawDegrees: canonicalAngle(value.yawDegrees, `${path}.yawDegrees`),
});

export function canonicalCreatureWeaponGripOptionsV1(
  value: CreatureWeaponGripOptionsV1,
): CreatureWeaponGripOptionsV1 {
  if (value.schemaVersion !== 1) throw new Error("weaponGrip.schemaVersion must be 1");
  if (value.mode !== "AUTO" && value.mode !== "AUTO_PLUS_OFFSETS") {
    throw new Error("weaponGrip.mode is unsupported");
  }
  const result = {
    schemaVersion: 1 as const,
    mode: value.mode,
    itemFamily: value.itemFamily ?? "SWORD",
    rightHand: canonicalOffset(value.rightHand, "weaponGrip.rightHand"),
    leftHand: canonicalOffset(value.leftHand, "weaponGrip.leftHand"),
  };
  if (result.mode === "AUTO" && [
    ...Object.values(result.rightHand),
    ...Object.values(result.leftHand),
  ].some((angle) => angle !== 0)) {
    throw new Error("weaponGrip AUTO mode requires zero hand offsets");
  }
  return result;
}

export function sameCreatureWeaponGripOptionsV1(
  left: CreatureWeaponGripOptionsV1,
  right: CreatureWeaponGripOptionsV1,
) {
  return JSON.stringify(canonicalCreatureWeaponGripOptionsV1(left))
    === JSON.stringify(canonicalCreatureWeaponGripOptionsV1(right));
}
