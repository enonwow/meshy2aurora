import type { ItemBaseItemRow } from "./types";

export type ItemAmmunitionChannelV1 = "ARROW" | "BOLT" | "BULLET";
export type ItemWielderClipV1 = "BOWSHOT" | "XBOWSHOT";
export type ItemProjectileAxisV1 =
  | "POSITIVE_X"
  | "NEGATIVE_X"
  | "POSITIVE_Y"
  | "NEGATIVE_Y"
  | "POSITIVE_Z"
  | "NEGATIVE_Z";

export interface ItemRangedAmmunitionDraftV1 {
  readonly enabled: boolean;
  readonly ammunitiontypesFile?: File;
  readonly damageTypesFile?: File;
  readonly projectileFile?: File;
  readonly ammunitionChannel: ItemAmmunitionChannelV1;
  readonly damageRangedProjectile: number;
  readonly damageTypeRow: 6;
  readonly damageTypeLabel: "Divine";
  readonly damagePropertySubtype: 8;
  readonly wielderClip: ItemWielderClipV1;
  readonly sourceForwardAxis: ItemProjectileAxisV1;
  readonly rotationXyzw: readonly [number, number, number, number];
  readonly rotationDegrees: readonly [number, number, number];
  readonly projectileScale: number;
  readonly inventoryScale: number;
  readonly projectileModelResref: string;
  readonly projectileTextureResref: string;
  readonly shotSoundResref: string;
  readonly impactSoundResref: string;
  readonly ammunitionBlueprintResref: string;
  readonly ammunitionModelResref: string;
  readonly ammunitionTextureResref: string;
  readonly ammunitionIconResref: string;
  readonly ammunitionVariant: 99;
  readonly localizedName: string;
  readonly description: string;
}

export interface ItemAmmunitionChannelProfileV1 {
  readonly channel: ItemAmmunitionChannelV1;
  readonly ammoBaseItem: 20 | 25 | 27;
  readonly ammunitionType: 1 | 2 | 3;
  readonly ammunitiontypesOffset: 0 | 1 | 2;
  readonly modelResref: string;
  readonly iconResref: string;
}

const RESREF = /^[a-z0-9_]{1,16}$/;

export function rangedAmmunitionChannelProfileV1(
  channel: ItemAmmunitionChannelV1,
): ItemAmmunitionChannelProfileV1 {
  switch (channel) {
    case "ARROW":
      return {
        channel,
        ammoBaseItem: 20,
        ammunitionType: 1,
        ammunitiontypesOffset: 0,
        modelResref: "wamar_099",
        iconResref: "iwamar_099",
      };
    case "BOLT":
      return {
        channel,
        ammoBaseItem: 25,
        ammunitionType: 2,
        ammunitiontypesOffset: 1,
        modelResref: "wambo_099",
        iconResref: "iwambo_099",
      };
    case "BULLET":
      return {
        channel,
        ammoBaseItem: 27,
        ammunitionType: 3,
        ammunitiontypesOffset: 2,
        modelResref: "wambu_099",
        iconResref: "iwambu_099",
      };
  }
}

export function defaultRangedAmmunitionDraftV1(): ItemRangedAmmunitionDraftV1 {
  const channel = rangedAmmunitionChannelProfileV1("BULLET");
  return {
    enabled: true,
    ammunitionChannel: "BULLET",
    damageRangedProjectile: 6,
    damageTypeRow: 6,
    damageTypeLabel: "Divine",
    damagePropertySubtype: 8,
    wielderClip: "XBOWSHOT",
    sourceForwardAxis: "POSITIVE_X",
    rotationXyzw: [0, 0, Math.SQRT1_2, Math.SQRT1_2],
    rotationDegrees: [0, 0, 90],
    projectileScale: 0.20,
    inventoryScale: 0.20,
    projectileModelResref: "m2ahxshell",
    projectileTextureResref: "m2ahxshtex",
    shotSoundResref: "cb_sh_prjtlelec",
    impactSoundResref: "scm_elec",
    ammunitionBlueprintResref: "m2ahxammo",
    ammunitionModelResref: channel.modelResref,
    ammunitionTextureResref: "m2ahxamtex",
    ammunitionIconResref: channel.iconResref,
    ammunitionVariant: 99,
    localizedName: "Hextech Shells",
    description: "Cylindrical hextech shells charged with divine energy.",
  };
}

export function withRangedAmmunitionChannelV1(
  draft: ItemRangedAmmunitionDraftV1,
  ammunitionChannel: ItemAmmunitionChannelV1,
): ItemRangedAmmunitionDraftV1 {
  const profile = rangedAmmunitionChannelProfileV1(ammunitionChannel);
  return {
    ...draft,
    ammunitionChannel,
    ammunitionModelResref: profile.modelResref,
    ammunitionIconResref: profile.iconResref,
  };
}

function channelForWeapon(row: ItemBaseItemRow): ItemAmmunitionChannelProfileV1 | undefined {
  return (["ARROW", "BOLT", "BULLET"] as const)
    .map(rangedAmmunitionChannelProfileV1)
    .find((profile) => (
      row.rangedWeapon === profile.ammoBaseItem
      && row.ammunitionType === profile.ammunitionType
    ));
}

function projectileAxisVector(axis: ItemProjectileAxisV1): [number, number, number] {
  switch (axis) {
    case "POSITIVE_X": return [1, 0, 0];
    case "NEGATIVE_X": return [-1, 0, 0];
    case "POSITIVE_Y": return [0, 1, 0];
    case "NEGATIVE_Y": return [0, -1, 0];
    case "POSITIVE_Z": return [0, 0, 1];
    case "NEGATIVE_Z": return [0, 0, -1];
  }
}

function rotatedProjectileForward(
  axis: ItemProjectileAxisV1,
  rotationXyzw: readonly [number, number, number, number],
): [number, number, number] | undefined {
  if (rotationXyzw.some((value) => !Number.isFinite(value))) return undefined;
  const length = Math.hypot(...rotationXyzw);
  if (length <= 1e-8) return undefined;
  const [qx, qy, qz, qw] = rotationXyzw.map((value) => value / length);
  const [vx, vy, vz] = projectileAxisVector(axis);
  const tx = 2 * (qy * vz - qz * vy);
  const ty = 2 * (qz * vx - qx * vz);
  const tz = 2 * (qx * vy - qy * vx);
  return [
    vx + qw * tx + (qy * tz - qz * ty),
    vy + qw * ty + (qz * tx - qx * tz),
    vz + qw * tz + (qx * ty - qy * tx),
  ];
}

export function validateRangedAmmunitionDraftV1(
  draft: ItemRangedAmmunitionDraftV1,
  row: ItemBaseItemRow,
): string[] {
  if (!draft.enabled) return [];
  const issues: string[] = [];
  const expectedChannel = channelForWeapon(row);
  if (row.weaponType !== 1 || !expectedChannel) {
    issues.push("Selected BaseItem is not an audited Arrow, Bolt or Bullet ranged weapon");
  } else if (expectedChannel.channel !== draft.ammunitionChannel) {
    issues.push(
      `${expectedChannel.channel} requires RangedWeapon ${expectedChannel.ammoBaseItem} and AmmunitionType ${expectedChannel.ammunitionType} for this weapon`,
    );
  }
  const expectedClip: ItemWielderClipV1 | undefined = row.weaponWield === 5
    ? "BOWSHOT"
    : row.weaponWield === 6
      ? "XBOWSHOT"
      : undefined;
  if (!expectedClip || draft.wielderClip !== expectedClip) {
    issues.push(`WeaponWield ${row.weaponWield ?? "****"} does not match ${draft.wielderClip}`);
  }
  if (!Number.isInteger(draft.damageRangedProjectile) || draft.damageRangedProjectile < 6 || draft.damageRangedProjectile > 255) {
    issues.push("DamageRangedProjectile must be an integer in 6..255");
  }
  if (!Number.isFinite(draft.projectileScale) || draft.projectileScale <= 0) {
    issues.push("Projectile scale must be positive and finite");
  }
  if (!Number.isFinite(draft.inventoryScale) || draft.inventoryScale <= 0) {
    issues.push("Ammunition item scale must be positive and finite");
  }
  const transformedForward = rotatedProjectileForward(
    draft.sourceForwardAxis,
    draft.rotationXyzw,
  );
  if (
    !transformedForward
    || Math.abs(transformedForward[0]) > 1e-4
    || Math.abs(transformedForward[1] - 1) > 1e-4
    || Math.abs(transformedForward[2]) > 1e-4
  ) {
    issues.push(
      "Declared projectile forward axis must resolve to Aurora +Y after the manual rotation",
    );
  }
  for (const [name, value] of [
    ["projectile model", draft.projectileModelResref],
    ["projectile texture", draft.projectileTextureResref],
    ["shot sound", draft.shotSoundResref],
    ["impact sound", draft.impactSoundResref],
    ["ammunition blueprint", draft.ammunitionBlueprintResref],
    ["ammunition model", draft.ammunitionModelResref],
    ["ammunition texture", draft.ammunitionTextureResref],
    ["ammunition icon", draft.ammunitionIconResref],
  ] as const) {
    if (!RESREF.test(value)) issues.push(`${name} resref must match [a-z0-9_]{1,16}`);
  }
  const channel = rangedAmmunitionChannelProfileV1(draft.ammunitionChannel);
  if (
    draft.ammunitionModelResref !== channel.modelResref
    || draft.ammunitionIconResref !== channel.iconResref
  ) {
    issues.push(
      `${draft.ammunitionChannel} variant 99 requires ${channel.modelResref} and ${channel.iconResref}`,
    );
  }
  return issues;
}

export function rangedAmmunitionFilesReadyV1(draft: ItemRangedAmmunitionDraftV1): boolean {
  return !draft.enabled || Boolean(
    draft.ammunitiontypesFile?.name.toLowerCase() === "ammunitiontypes.2da"
    && draft.damageTypesFile?.name.toLowerCase() === "damagetypes.2da"
    && draft.projectileFile?.name.toLowerCase().endsWith(".glb"),
  );
}

export function buildRangedProjectileOptionsV1(
  draft: ItemRangedAmmunitionDraftV1,
): Record<string, unknown> {
  return {
    schemaVersion: 1,
    sourceForwardAxis: draft.sourceForwardAxis,
    transform: {
      translation: [0, 0, 0],
      rotationXyzw: [...draft.rotationXyzw],
      uniformScale: draft.projectileScale,
      pivot: [0, 0, 0],
    },
    sourceNode: null,
  };
}

export function buildRangedAmmunitionItemOptionsV1(
  draft: ItemRangedAmmunitionDraftV1,
): Record<string, unknown> {
  return {
    schemaVersion: 1,
    transform: {
      translation: [0, 0, 0],
      rotationXyzw: [...draft.rotationXyzw],
      uniformScale: draft.inventoryScale,
      pivot: [0, 0, 0],
    },
    sourceNode: null,
    textureEncoding: "DIRECT_COLOR",
    iconSize: [32, 32],
    iconProjectionBounds: null,
    weaponColor: null,
    targetSpaceScaleXyz: [1, 1, 1],
  };
}

export function buildRangedAmmunitionBlueprintV1(
  draft: ItemRangedAmmunitionDraftV1,
): Record<string, unknown> & {
  templateResref: string;
  stackSize: 99;
  parts: Array<{ field: "ModelPart1"; value: 99 }>;
  properties: Array<{
    propertyName: 16;
    subtype: 8;
    costTable: 4;
    costValue: 1;
    param1: 255;
    param1Value: 0;
    chanceAppear: 100;
  }>;
} {
  return {
    schemaVersion: 1,
    templateResref: draft.ammunitionBlueprintResref,
    tag: draft.ammunitionBlueprintResref.toUpperCase(),
    localizedName: draft.localizedName,
    description: draft.description,
    identifiedDescription: draft.description,
    comment: "Generated by Meshy2Aurora ranged ammunition V1.",
    parts: [{ field: "ModelPart1", value: 99 }],
    properties: [{
      propertyName: 16,
      subtype: 8,
      costTable: 4,
      costValue: 1,
      param1: 255,
      param1Value: 0,
      chanceAppear: 100,
    }],
    colors: {
      leather1Color: null,
      leather2Color: null,
      cloth1Color: null,
      cloth2Color: null,
      metal1Color: null,
      metal2Color: null,
    },
    cost: 50,
    addCost: 0,
    charges: 0,
    stackSize: 99,
    paletteId: 4,
    identified: true,
    stolen: false,
    cursed: false,
    plot: false,
  };
}
