export type CreatureHeldWeaponModeV1 = "NONE" | "RIGHT_HAND" | "LEFT_HAND" | "BOTH_HANDS";

export type CreatureEquipmentPlacementV2 = "RIGHT_HAND" | "LEFT_HAND" | "BOTH_HANDS";
export type CreatureItemGripFamilyV1 =
  | "SWORD"
  | "AXE_MACE"
  | "SPEAR_POLEARM"
  | "BOW_CROSSBOW"
  | "SHIELD";

export interface CreatureEquipmentItemV2 {
  readonly resref: string;
  readonly displayName: string;
  readonly resourceScope: "NWN_BASE_GAME" | "MODULE_OWNED_RECIPE";
  readonly baseItem: number;
  readonly modelParts: readonly [number, number, number];
  readonly placement: CreatureEquipmentPlacementV2;
  readonly gripFamily: CreatureItemGripFamilyV1;
  readonly requiredProficiencyFeats: readonly number[];
  readonly localizedNameStrref: number;
  readonly cost: number;
}

export interface CreatureDemoAuthoringV1 {
  readonly schemaVersion: 1;
  readonly blueprint: {
    readonly schemaVersion: 1;
    readonly firstName: string;
    readonly lastName: string;
    readonly description: string;
    readonly tag: string;
    readonly conversation: string;
    readonly factionId: number;
    readonly portraitId: number;
    readonly soundSetFile: number;
    readonly race: number;
    readonly gender: number;
    readonly hitPoints: number;
    readonly currentHitPoints: number;
    readonly maxHitPoints: number;
    readonly challengeRating: number;
    readonly abilities: readonly [number, number, number, number, number, number];
    readonly naturalAc: number;
    readonly perceptionRange: number;
    readonly class: { readonly classId: number; readonly level: number };
    readonly feats: readonly number[];
    readonly scripts: Readonly<Record<
      | "heartbeat" | "notice" | "spellAt" | "attacked" | "damaged"
      | "disturbed" | "endRound" | "dialogue" | "spawn" | "rested"
      | "death" | "userDefined" | "blocked",
      string
    >>;
    readonly plot: boolean;
    readonly immortal: boolean;
    readonly interruptable: boolean;
    readonly lootable: boolean;
  };
  readonly equipmentLoadout: {
    readonly schemaVersion: 2;
    readonly items: readonly CreatureEquipmentItemV2[];
  };
}

export interface CreatureHeldWeaponOptionsV1 {
  readonly schemaVersion: 1;
  readonly mode: CreatureHeldWeaponModeV1;
  readonly itemResref?: string;
}

export const CREATURE_HELD_WEAPON_RECIPE_V1 = "NWN_BASE_SHORTSWORD_V2";
export const CREATURE_HELD_WEAPON_RESREF_V1 = "nw_wswss001";
export const CREATURE_EQUIPMENT_RECIPE_V2 = "NWN_V10_BASTARD_SWORD_EMBEDDED_V2";

export function creatureHeldWeaponOptionsV1(
  mode: CreatureHeldWeaponModeV1,
  _itemResref?: string,
): CreatureHeldWeaponOptionsV1 {
  if (mode === "NONE") return { schemaVersion: 1, mode };
  return { schemaVersion: 1, mode, itemResref: CREATURE_HELD_WEAPON_RESREF_V1 };
}

/**
 * Complete gameplay/demo authoring. The item is an embedded GFF struct in
 * both GIT and UTC; this deliberately does not emit an EquippedRes shortcut.
 */
export function creatureDemoAuthoringV1(
  mode: CreatureHeldWeaponModeV1,
): CreatureDemoAuthoringV1 {
  const scripts = {
    heartbeat: "nw_c2_default1",
    notice: "nw_c2_default2",
    spellAt: "nw_c2_defaultb",
    attacked: "nw_c2_default5",
    damaged: "nw_c2_default6",
    disturbed: "nw_c2_default8",
    endRound: "nw_c2_default3",
    dialogue: "nw_c2_default4",
    spawn: "nw_c2_default9",
    rested: "nw_c2_defaulta",
    death: "nw_c2_default7",
    userDefined: "nw_c2_defaultd",
    blocked: "nw_c2_defaulte",
  } as const;
  const items: CreatureEquipmentItemV2[] = mode === "NONE" ? [] : [{
    resref: "nw_wswbs001",
    displayName: "Bastard Sword",
    resourceScope: "NWN_BASE_GAME",
    baseItem: 3,
    modelParts: [41, 11, 11],
    placement: mode,
    gripFamily: "SWORD",
    requiredProficiencyFeats: [44],
    localizedNameStrref: 168,
    cost: 70,
  }];
  return {
    schemaVersion: 1,
    blueprint: {
      schemaVersion: 1,
      firstName: "Meshy procedural creature",
      lastName: "",
      description: "",
      tag: "m2a_creature",
      conversation: "",
      factionId: 1,
      portraitId: 236,
      soundSetFile: 53,
      race: 7,
      gender: 2,
      hitPoints: 22,
      currentHitPoints: 22,
      maxHitPoints: 37,
      challengeRating: 5,
      abilities: [19, 15, 17, 6, 12, 6],
      naturalAc: 6,
      perceptionRange: 11,
      class: { classId: 11, level: 5 },
      feats: [],
      scripts,
      plot: false,
      immortal: false,
      interruptable: true,
      lootable: false,
    },
    equipmentLoadout: { schemaVersion: 2, items },
  };
}
