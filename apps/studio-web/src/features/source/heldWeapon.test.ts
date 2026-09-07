import { describe, expect, it } from "vitest";
import {
  creatureDemoAuthoringV1,
  creatureHeldWeaponOptionsV1,
} from "./heldWeapon";

describe("Creature held weapon options", () => {
  it("keeps NONE free of a synthetic item identity", () => {
    expect(creatureHeldWeaponOptionsV1("NONE", "ignored")).toEqual({
      schemaVersion: 1,
      mode: "NONE",
    });
  });

  it("binds either selected native hand slot to the NWN base short sword", () => {
    expect(creatureHeldWeaponOptionsV1("RIGHT_HAND")).toEqual({
      schemaVersion: 1,
      mode: "RIGHT_HAND",
      itemResref: "nw_wswss001",
    });
    expect(creatureHeldWeaponOptionsV1("LEFT_HAND", "ignored")).toEqual({
      schemaVersion: 1,
      mode: "LEFT_HAND",
      itemResref: "nw_wswss001",
    });
  });

  it("authors the complete V10 embedded item recipe without an EquippedRes shortcut", () => {
    const authored = creatureDemoAuthoringV1("BOTH_HANDS");
    expect(authored).toMatchObject({
      schemaVersion: 1,
      blueprint: {
        schemaVersion: 1,
        firstName: "Meshy procedural creature",
        hitPoints: 22,
        maxHitPoints: 37,
        class: { classId: 11, level: 5 },
      },
      equipmentLoadout: {
        schemaVersion: 2,
        items: [{
          resref: "nw_wswbs001",
          resourceScope: "NWN_BASE_GAME",
          baseItem: 3,
          modelParts: [41, 11, 11],
          placement: "BOTH_HANDS",
          gripFamily: "SWORD",
          requiredProficiencyFeats: [44],
        }],
      },
    });
    expect(JSON.stringify(authored)).not.toContain("EquippedRes");
  });

  it("keeps complete gameplay authoring while NONE emits an empty loadout", () => {
    expect(creatureDemoAuthoringV1("NONE").equipmentLoadout).toEqual({
      schemaVersion: 2,
      items: [],
    });
  });
});
