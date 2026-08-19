import type {
  ItemBaseItemRow,
  ItemBaseItemsCatalog,
  ItemPartDraft,
  ResolvedItemPartDraft,
} from "./types";

function record(value: unknown, path: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${path} must be an object`);
  }
  return value as Record<string, unknown>;
}

function number(value: unknown, path: string): number {
  if (typeof value !== "number" || !Number.isInteger(value) || value < 0) {
    throw new Error(`${path} must be a non-negative integer`);
  }
  return value;
}

function nullableNumber(value: unknown, path: string): number | null {
  return value === null || value === undefined ? null : number(value, path);
}

function string(value: unknown, path: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${path} must be a non-empty string`);
  }
  return value;
}

function stringEnum<T extends string>(
  value: unknown,
  allowed: readonly T[],
  path: string,
): T {
  const resolved = string(value, path);
  if (!allowed.includes(resolved as T)) {
    throw new Error(`${path} is unsupported`);
  }
  return resolved as T;
}

export function projectItemBaseitemsCatalog(json: string): ItemBaseItemsCatalog {
  const root = record(JSON.parse(json), "catalog");
  if (root.schemaVersion !== 1 || !Array.isArray(root.rows)) {
    throw new Error("catalog must use Item BaseItems schema version 1");
  }
  const rows = root.rows.map((raw, rowIndex): ItemBaseItemRow => {
    const row = record(raw, `catalog.rows[${rowIndex}]`);
    const modelType = number(row.modelType, `catalog.rows[${rowIndex}].modelType`);
    if (modelType > 3) throw new Error(`catalog.rows[${rowIndex}].modelType is unsupported`);
    if (!Array.isArray(row.partSlots) || !Array.isArray(row.colorFields)) {
      throw new Error(`catalog.rows[${rowIndex}] has no resolved part/color schema`);
    }
    const capability = record(row.capability, `catalog.rows[${rowIndex}].capability`);
    if (
      capability.schemaVersion !== 1
      || !Array.isArray(capability.requiredReferenceTables)
    ) {
      throw new Error(`catalog.rows[${rowIndex}].capability is invalid`);
    }
    return {
      schemaVersion: 1,
      baseItem: number(row.baseItem, `catalog.rows[${rowIndex}].baseItem`),
      label: string(row.label, `catalog.rows[${rowIndex}].label`),
      itemClass: string(row.itemClass, `catalog.rows[${rowIndex}].itemClass`),
      modelType: modelType as 0 | 1 | 2 | 3,
      minRange: nullableNumber(row.minRange, `catalog.rows[${rowIndex}].minRange`),
      maxRange: nullableNumber(row.maxRange, `catalog.rows[${rowIndex}].maxRange`),
      genderSpecific: row.genderSpecific === true,
      defaultModel: row.defaultModel === null
        ? null
        : string(row.defaultModel, `catalog.rows[${rowIndex}].defaultModel`),
      defaultIcon: row.defaultIcon === null
        ? null
        : string(row.defaultIcon, `catalog.rows[${rowIndex}].defaultIcon`),
      equipableSlots: number(row.equipableSlots, `catalog.rows[${rowIndex}].equipableSlots`),
      invSlotWidth: number(row.invSlotWidth, `catalog.rows[${rowIndex}].invSlotWidth`),
      invSlotHeight: number(row.invSlotHeight, `catalog.rows[${rowIndex}].invSlotHeight`),
      weaponWield: nullableNumber(row.weaponWield, `catalog.rows[${rowIndex}].weaponWield`),
      weaponType: nullableNumber(row.weaponType, `catalog.rows[${rowIndex}].weaponType`),
      rangedWeapon: nullableNumber(row.rangedWeapon, `catalog.rows[${rowIndex}].rangedWeapon`),
      ammunitionType: nullableNumber(row.ammunitionType, `catalog.rows[${rowIndex}].ammunitionType`),
      capability: {
        schemaVersion: 1,
        compositionProfile: stringEnum(
          capability.compositionProfile,
          ["SINGLE_PART", "BOTTOM_MIDDLE_TOP", "CAPART_ARMOR", "CLOAK_MODEL"] as const,
          `catalog.rows[${rowIndex}].capability.compositionProfile`,
        ),
        textureProfile: stringEnum(
          capability.textureProfile,
          ["DIRECT_COLOR", "PALETTE_LAYERS", "CAPART_PALETTE_LAYERS"] as const,
          `catalog.rows[${rowIndex}].capability.textureProfile`,
        ),
        iconProfile: stringEnum(
          capability.iconProfile,
          ["STANDARD", "LAYERED", "IPRP_SPELL", "CAPART_COMPOSITE", "CLOAK_MODEL"] as const,
          `catalog.rows[${rowIndex}].capability.iconProfile`,
        ),
        meshySourceCount: number(
          capability.meshySourceCount,
          `catalog.rows[${rowIndex}].capability.meshySourceCount`,
        ),
        requiredReferenceTables: capability.requiredReferenceTables.map((value, index) => string(
          value,
          `catalog.rows[${rowIndex}].capability.requiredReferenceTables[${index}]`,
        )),
      },
      partSlots: row.partSlots.map((rawSlot, slotIndex) => {
        const slot = record(rawSlot, `catalog.rows[${rowIndex}].partSlots[${slotIndex}]`);
        return {
          index: number(slot.index, `partSlots[${slotIndex}].index`),
          field: string(slot.field, `partSlots[${slotIndex}].field`),
          label: string(slot.label, `partSlots[${slotIndex}].label`),
          token: slot.token === null ? null : string(slot.token, `partSlots[${slotIndex}].token`),
          sourceKind: stringEnum(
            slot.sourceKind,
            ["MESHY_GLB", "CAPART_SELECTION", "CLOAK_MODEL_SELECTION"] as const,
            `partSlots[${slotIndex}].sourceKind`,
          ),
          referenceTable: slot.referenceTable === undefined || slot.referenceTable === null
            ? null
            : string(slot.referenceTable, `partSlots[${slotIndex}].referenceTable`),
          requiresExplicitResourceResrefs: slot.requiresExplicitResourceResrefs === true,
        };
      }),
      colorFields: row.colorFields.map((value, index) => string(value, `colorFields[${index}]`)),
    };
  });
  return {
    schemaVersion: 1,
    sourceSha256: string(root.sourceSha256, "catalog.sourceSha256"),
    physicalRowCount: number(root.physicalRowCount, "catalog.physicalRowCount"),
    inactiveRowCount: number(root.inactiveRowCount, "catalog.inactiveRowCount"),
    rows,
  };
}

export function initialItemPartDrafts(row: ItemBaseItemRow): ItemPartDraft[] {
  const initialWeaponModel = row.modelType === 2 && row.minRange !== null
    ? Math.ceil(row.minRange / 10)
    : 0;
  return row.partSlots.map((slot) => ({
    field: slot.field,
    label: slot.label,
    token: slot.token,
    sourceKind: slot.sourceKind,
    referenceTable: slot.referenceTable,
    requiresExplicitResourceResrefs: slot.requiresExplicitResourceResrefs,
    sourceNode: "",
    textureEncoding: row.capability.textureProfile === "DIRECT_COLOR"
      ? "DIRECT_COLOR"
      : "PLT_LEATHER1",
    variant: 1,
    weaponModel: row.modelType === 2 ? initialWeaponModel : null,
    weaponColor: row.modelType === 2 ? 1 : null,
    explicitModelResref: "",
    explicitIconResref: "",
    translation: [0, 0, 0],
    rotationDegrees: [0, 0, 0],
    rotationXyzw: [0, 0, 0, 1],
    uniformScale: 1,
    pivot: [0, 0, 0],
    targetSpaceScaleXyz: [1, 1, 1],
  }));
}

export interface WeaponPartAppearance {
  readonly model: number;
  readonly color: number;
  readonly encoded: number;
}

export function encodeWeaponPartAppearance(model: number, color: number): number {
  if (!Number.isInteger(color) || color < 1 || color > 4) {
    throw new Error("ModelType2 weapon color must be an integer in the range 1..4");
  }
  if (!Number.isInteger(model) || model < 0) {
    throw new Error("ModelType2 weapon model must be a non-negative integer");
  }
  const encoded = model * 10 + color;
  if (encoded > 255) {
    throw new Error("ModelType2 weapon model and color must encode into one UTI BYTE");
  }
  return encoded;
}

export function decodeWeaponPartAppearance(encoded: number): WeaponPartAppearance {
  if (!Number.isInteger(encoded) || encoded < 0 || encoded > 255) {
    throw new Error("ModelType2 weapon appearance must fit one UTI BYTE");
  }
  const model = Math.floor(encoded / 10);
  const color = encoded % 10;
  return { model, color, encoded: encodeWeaponPartAppearance(model, color) };
}

function resref(value: string, path: string): string {
  const normalized = value.trim().toLowerCase();
  if (!/^[a-z0-9_]{1,16}$/.test(normalized)) {
    throw new Error(`${path} must be a 1..16 character Aurora resref`);
  }
  return normalized;
}

export function resolveItemPartDraft(
  row: ItemBaseItemRow,
  draft: ItemPartDraft,
  ordinal: number,
): ResolvedItemPartDraft {
  const variant = row.modelType === 2
    ? encodeWeaponPartAppearance(
        draft.weaponModel ?? Number.NaN,
        draft.weaponColor ?? Number.NaN,
      )
    : draft.variant;
  if (!Number.isInteger(variant) || variant < 0 || variant > 255) {
    throw new Error(`${draft.field} variant must fit the UTI BYTE field (0..255)`);
  }
  const normalizedDraft = { ...draft, variant };
  if (draft.sourceKind !== "MESHY_GLB") {
    return {
      ...normalizedDraft,
      modelResref: "",
      iconResref: "",
      textureResref: "",
      weaponColorways: [],
    };
  }
  const suffix = draft.token
    ? `_${draft.token}_${String(variant).padStart(3, "0")}`
    : `_${String(variant).padStart(3, "0")}`;
  const modelResref = draft.requiresExplicitResourceResrefs
    ? resref(draft.explicitModelResref, `${draft.field} model resref`)
    : resref(`${row.itemClass.toLowerCase()}${suffix}`, `${draft.field} model resref`);
  const iconResref = draft.requiresExplicitResourceResrefs
    ? resref(draft.explicitIconResref, `${draft.field} icon resref`)
    : resref(`i${row.itemClass.toLowerCase()}${suffix}`, `${draft.field} icon resref`);
  const textureResref = resref(
    `m2ait${row.baseItem.toString(36)}${ordinal.toString(36)}`,
    `${draft.field} texture resref`,
  );
  return { ...normalizedDraft, modelResref, iconResref, textureResref, weaponColorways: [] };
}

export interface ItemNamespaceAllocation {
  readonly blueprintResref: string;
  readonly hakResref: string;
  readonly moduleResref: string;
  readonly areaResref: string;
  readonly proofCreatureResref: string | null;
  readonly parts: ResolvedItemPartDraft[];
  readonly occupiedKeyCount: number;
  readonly collisionAvoidanceCount: number;
  readonly selectedWeaponModels: readonly number[];
  readonly sourceMinRange: number | null;
  readonly sourceMaxRange: number | null;
  readonly effectiveMaxRange: number | null;
  readonly baseitemsOverrideRequired: boolean;
}

export function requiresEquippedItemProof(row: ItemBaseItemRow): boolean {
  return ["CAPART_COMPOSITE", "CLOAK_MODEL"].includes(row.capability.iconProfile);
}

function resourceKey(resourceType: number | "HAK" | "MOD", resourceResref: string) {
  return `${resourceType}:${resourceResref}`.toLowerCase();
}

function allocateFreeResref(
  prefix: string,
  resourceType: number | "HAK" | "MOD",
  occupied: Set<string>,
  allocated: Set<string>,
): { resref: string; probes: number } {
  for (let probe = 0; probe < 46_656; probe += 1) {
    const suffix = probe === 0 ? "" : probe.toString(36);
    const candidate = resref(
      `${prefix.slice(0, 16 - suffix.length)}${suffix}`,
      "allocated resref",
    );
    const key = resourceKey(resourceType, candidate);
    if (!occupied.has(key) && !allocated.has(key)) {
      allocated.add(key);
      return { resref: candidate, probes: probe };
    }
  }
  throw new Error(`${prefix} exhausted the deterministic resref namespace`);
}

function allocateFreeAreaResref(
  prefix: string,
  occupied: Set<string>,
  allocated: Set<string>,
): { resref: string; probes: number } {
  for (let probe = 0; probe < 46_656; probe += 1) {
    const suffix = probe === 0 ? "" : probe.toString(36);
    const candidate = resref(
      `${prefix.slice(0, 16 - suffix.length)}${suffix}`,
      "allocated Area resref",
    );
    const keys = [2012, 2023, 2046].map((type) => resourceKey(type, candidate));
    if (keys.every((key) => !occupied.has(key) && !allocated.has(key))) {
      keys.forEach((key) => allocated.add(key));
      return { resref: candidate, probes: probe };
    }
  }
  throw new Error(`${prefix} exhausted the deterministic Area namespace`);
}

export function allocateItemNamespace(
  row: ItemBaseItemRow,
  drafts: readonly ItemPartDraft[],
  occupiedKeys: readonly string[],
): ItemNamespaceAllocation {
  const normalizedOccupiedKeys = occupiedKeys
    .map((value) => value.trim().toLowerCase())
    .filter(Boolean);
  if (normalizedOccupiedKeys.some(
    (key) => !/^(?:[0-9]+|hak|mod):[a-z0-9_]{1,16}$/.test(key),
  )) {
    throw new Error(
      "occupied Item namespace entries must use numeric-type:resref, HAK:resref or MOD:resref",
    );
  }
  const occupied = new Set(normalizedOccupiedKeys);
  const allocated = new Set<string>();
  let collisionAvoidanceCount = 0;
  const emitsCustomIcon = !["IPRP_SPELL", "CLOAK_MODEL", "CAPART_COMPOSITE"]
    .includes(row.capability.iconProfile);
  const atomicWeaponParts = new Map<number, {
    readonly part: ResolvedItemPartDraft;
    readonly colorways: readonly ResolvedItemPartDraft[];
  }>();

  if (row.modelType === 2) {
    if (
      row.minRange === null
      || row.maxRange === null
      || row.minRange > row.maxRange
      || row.minRange % 10 !== 0
      || row.maxRange % 10 !== 0
    ) {
      throw new Error("ModelType2 BaseItem requires valid 10-aligned MinRange/MaxRange values");
    }
    const minWeaponModel = row.minRange / 10;
    const selectorCount = 26 - minWeaponModel;
    if (selectorCount <= 0) {
      throw new Error("ModelType2 MinRange leaves no encodable model selector");
    }
    const meshyDrafts = drafts
      .map((draft, ordinal) => ({ draft, ordinal }))
      .filter(({ draft }) => draft.sourceKind === "MESHY_GLB");
    let resolvedGroup: Array<{
      ordinal: number;
      part: ResolvedItemPartDraft;
      colorways: ResolvedItemPartDraft[];
    }> | undefined;
    for (let probe = 0; probe < selectorCount; probe += 1) {
      const candidates = meshyDrafts.map(({ draft, ordinal }) => {
        const requestedModel = draft.weaponModel ?? Number.NaN;
        if (!Number.isInteger(requestedModel) || requestedModel < 0 || requestedModel > 25) {
          throw new Error(`${draft.field} weapon model must be in the range 0..25`);
        }
        const startModel = Math.max(requestedModel, minWeaponModel);
        const weaponModel = minWeaponModel
          + ((startModel - minWeaponModel + probe) % selectorCount);
        const colorways = ([1, 2, 3, 4] as const).map((weaponColor) => (
          resolveItemPartDraft(row, { ...draft, weaponModel, weaponColor }, ordinal)
        ));
        return {
          ordinal,
          part: colorways[(draft.weaponColor ?? 1) - 1],
          colorways,
        };
      });
      const candidateKeys = candidates.flatMap(({ colorways }) => colorways.flatMap((part) => [
        resourceKey(2002, part.modelResref),
        ...(emitsCustomIcon ? [resourceKey(3, part.iconResref)] : []),
      ]));
      const uniqueKeys = new Set(candidateKeys);
      const collision = uniqueKeys.size !== candidateKeys.length
        || candidateKeys.some((key) => occupied.has(key) || allocated.has(key));
      if (!collision) {
        candidateKeys.forEach((key) => allocated.add(key));
        collisionAvoidanceCount += probe;
        resolvedGroup = candidates;
        break;
      }
      if (meshyDrafts.some(({ draft }) => draft.requiresExplicitResourceResrefs)) {
        throw new Error("ModelType2 explicit resource group collides with the occupied namespace");
      }
    }
    if (!resolvedGroup) {
      throw new Error("ModelType2 weapon group exhausted the BaseItem-compatible model selectors");
    }
    resolvedGroup.forEach(({ ordinal, part, colorways }) => atomicWeaponParts.set(
      ordinal,
      { part, colorways },
    ));
  }

  const parts = drafts.map((draft, ordinal): ResolvedItemPartDraft => {
    if (draft.sourceKind !== "MESHY_GLB") {
      return resolveItemPartDraft(row, draft, ordinal);
    }
    const atomicWeaponPart = atomicWeaponParts.get(ordinal);
    if (atomicWeaponPart) {
      const weaponColorways = atomicWeaponPart.colorways.map((colorway) => {
        const texture = allocateFreeResref(
          `m2ait${row.baseItem.toString(36)}${ordinal.toString(36)}c${colorway.weaponColor}`,
          3,
          occupied,
          allocated,
        );
        collisionAvoidanceCount += texture.probes;
        return {
          color: colorway.weaponColor as 1 | 2 | 3 | 4,
          variant: colorway.variant,
          modelResref: colorway.modelResref,
          iconResref: colorway.iconResref,
          textureResref: texture.resref,
        };
      });
      const selectedColorway = weaponColorways.find(
        (colorway) => colorway.color === atomicWeaponPart.part.weaponColor,
      );
      if (!selectedColorway) throw new Error(`${draft.field} has no selected weapon colorway`);
      return {
        ...atomicWeaponPart.part,
        textureResref: selectedColorway.textureResref,
        weaponColorways,
      };
    }
    let resolved: ResolvedItemPartDraft | undefined;
    for (let probe = 0; probe < 256; probe += 1) {
      const variant = (draft.variant + probe) % 256;
      const candidate = resolveItemPartDraft(row, { ...draft, variant }, ordinal);
      const modelKey = resourceKey(2002, candidate.modelResref);
      const iconKey = resourceKey(3, candidate.iconResref);
      if (
        !occupied.has(modelKey)
        && !allocated.has(modelKey)
        && (!emitsCustomIcon || (!occupied.has(iconKey) && !allocated.has(iconKey)))
      ) {
        allocated.add(modelKey);
        if (emitsCustomIcon) allocated.add(iconKey);
        collisionAvoidanceCount += probe;
        resolved = candidate;
        break;
      }
      if (draft.requiresExplicitResourceResrefs) {
        throw new Error(`${draft.field} explicit resource resrefs collide with the occupied namespace`);
      }
    }
    if (!resolved) throw new Error(`${draft.field} exhausted all 256 Aurora item variants`);
    const texture = allocateFreeResref(
      `m2ait${row.baseItem.toString(36)}${ordinal.toString(36)}`,
      resolved.textureEncoding === "DIRECT_COLOR" ? 3 : 6,
      occupied,
      allocated,
    );
    collisionAvoidanceCount += texture.probes;
    return { ...resolved, textureResref: texture.resref };
  });
  const blueprint = allocateFreeResref(
    `m2aitm${row.baseItem.toString(36)}`,
    2025,
    occupied,
    allocated,
  );
  const hak = allocateFreeResref(
    `m2aihak${row.baseItem.toString(36)}`,
    "HAK",
    occupied,
    allocated,
  );
  const module = allocateFreeResref(
    `m2aimod${row.baseItem.toString(36)}`,
    "MOD",
    occupied,
    allocated,
  );
  const area = allocateFreeAreaResref(
    `m2aia${row.baseItem.toString(36)}`,
    occupied,
    allocated,
  );
  const proofCreature = requiresEquippedItemProof(row)
    ? allocateFreeResref(
        `m2ainpc${row.baseItem.toString(36)}`,
        2027,
        occupied,
        allocated,
      )
    : null;
  collisionAvoidanceCount += blueprint.probes
    + hak.probes
    + module.probes
    + area.probes
    + (proofCreature?.probes ?? 0);
  const selectedWeaponModels = row.modelType === 2
    ? [...new Set(parts.flatMap((part) => (
        part.sourceKind === "MESHY_GLB" && part.weaponModel !== null ? [part.weaponModel] : []
      )))].sort((first, second) => first - second)
    : [];
  const effectiveMaxRange = row.modelType === 2
    ? Math.max(row.maxRange ?? 0, ...selectedWeaponModels.map((model) => model * 10))
    : null;
  return {
    blueprintResref: blueprint.resref,
    hakResref: hak.resref,
    moduleResref: module.resref,
    areaResref: area.resref,
    proofCreatureResref: proofCreature?.resref ?? null,
    parts,
    occupiedKeyCount: occupied.size,
    collisionAvoidanceCount,
    selectedWeaponModels,
    sourceMinRange: row.modelType === 2 ? row.minRange : null,
    sourceMaxRange: row.modelType === 2 ? row.maxRange : null,
    effectiveMaxRange,
    baseitemsOverrideRequired: row.modelType === 2
      && effectiveMaxRange !== null
      && row.maxRange !== null
      && effectiveMaxRange > row.maxRange,
  };
}

export function eulerDegreesToQuaternion(
  rotation: readonly [number, number, number],
): [number, number, number, number] {
  const [x, y, z] = rotation.map((value) => value * Math.PI / 180);
  const [sx, cx] = [Math.sin(x / 2), Math.cos(x / 2)];
  const [sy, cy] = [Math.sin(y / 2), Math.cos(y / 2)];
  const [sz, cz] = [Math.sin(z / 2), Math.cos(z / 2)];
  return [
    sx * cy * cz + cx * sy * sz,
    cx * sy * cz - sx * cy * sz,
    cx * cy * sz + sx * sy * cz,
    cx * cy * cz - sx * sy * sz,
  ];
}

export function quaternionToEulerDegrees(
  rotation: readonly [number, number, number, number],
): [number, number, number] {
  const [x, y, z, w] = rotation;
  const m11 = 1 - 2 * (y * y + z * z);
  const m12 = 2 * (x * y - z * w);
  const m13 = 2 * (x * z + y * w);
  const m22 = 1 - 2 * (x * x + z * z);
  const m23 = 2 * (y * z - x * w);
  const m32 = 2 * (y * z + x * w);
  const m33 = 1 - 2 * (x * x + y * y);
  const eulerY = Math.asin(Math.max(-1, Math.min(1, m13)));
  const [eulerX, eulerZ] = Math.abs(m13) < 0.9999999
    ? [Math.atan2(-m23, m33), Math.atan2(-m12, m11)]
    : [Math.atan2(m32, m22), 0];
  const degrees = 180 / Math.PI;
  return [eulerX * degrees, eulerY * degrees, eulerZ * degrees];
}
