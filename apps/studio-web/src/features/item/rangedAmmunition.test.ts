import { describe, expect, it } from "vitest";
import {
  buildRangedAmmunitionBlueprintV1,
  defaultRangedAmmunitionDraftV1,
  rangedAmmunitionChannelProfileV1,
  validateRangedAmmunitionDraftV1,
  withRangedAmmunitionChannelV1,
} from "./rangedAmmunition";
import type { ItemBaseItemRow } from "./types";

const bulletWeapon: ItemBaseItemRow = {
  schemaVersion: 1,
  baseItem: 113,
  label: "hextech_shotgun",
  itemClass: "WHxSh",
  modelType: 2,
  minRange: 10,
  maxRange: 100,
  genderSpecific: false,
  defaultModel: "it_bag",
  defaultIcon: "iwhxsh",
  equipableSlots: 0x00030,
  invSlotWidth: 2,
  invSlotHeight: 4,
  weaponWield: 6,
  weaponType: 1,
  rangedWeapon: 27,
  ammunitionType: 3,
  capability: {
    schemaVersion: 1,
    compositionProfile: "BOTTOM_MIDDLE_TOP",
    textureProfile: "DIRECT_COLOR",
    iconProfile: "LAYERED",
    meshySourceCount: 3,
    requiredReferenceTables: [],
  },
  partSlots: [],
  colorFields: [],
};

describe("ranged ammunition V1", () => {
  it("maps all native channels atomically", () => {
    expect(rangedAmmunitionChannelProfileV1("ARROW")).toMatchObject({
      ammoBaseItem: 20,
      ammunitionType: 1,
      ammunitiontypesOffset: 0,
      modelResref: "wamar_099",
      iconResref: "iwamar_099",
    });
    expect(rangedAmmunitionChannelProfileV1("BOLT")).toMatchObject({
      ammoBaseItem: 25,
      ammunitionType: 2,
      ammunitiontypesOffset: 1,
      modelResref: "wambo_099",
      iconResref: "iwambo_099",
    });
    expect(rangedAmmunitionChannelProfileV1("BULLET")).toMatchObject({
      ammoBaseItem: 27,
      ammunitionType: 3,
      ammunitiontypesOffset: 2,
      modelResref: "wambu_099",
      iconResref: "iwambu_099",
    });
  });

  it("creates a bullet stack blueprint bound to the audited Divine damage route", () => {
    const draft = defaultRangedAmmunitionDraftV1();
    const blueprint = buildRangedAmmunitionBlueprintV1(draft);
    expect(blueprint.templateResref).toBe("m2ahxammo");
    expect(blueprint.stackSize).toBe(99);
    expect(blueprint.parts).toEqual([{ field: "ModelPart1", value: 99 }]);
    expect(blueprint.properties).toEqual([{
      propertyName: 16,
      subtype: 8,
      costTable: 4,
      costValue: 1,
      param1: 255,
      param1Value: 0,
      chanceAppear: 100,
    }]);
  });

  it("rejects a channel that disagrees with the weapon semantic columns", () => {
    const bullet = defaultRangedAmmunitionDraftV1();
    expect(validateRangedAmmunitionDraftV1(bullet, bulletWeapon)).toEqual([]);
    const bolt = withRangedAmmunitionChannelV1(bullet, "BOLT");
    expect(validateRangedAmmunitionDraftV1(bolt, bulletWeapon)).toContain(
      "BULLET requires RangedWeapon 27 and AmmunitionType 3 for this weapon",
    );
  });

  it("blocks a declared projectile nose that does not resolve to Aurora +Y", () => {
    const draft = {
      ...defaultRangedAmmunitionDraftV1(),
      rotationXyzw: [0, 0, 0, 1] as const,
    };
    expect(validateRangedAmmunitionDraftV1(draft, bulletWeapon)).toContain(
      "Declared projectile forward axis must resolve to Aurora +Y after the manual rotation",
    );
  });
});
