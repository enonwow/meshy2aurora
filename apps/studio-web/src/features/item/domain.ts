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
    explicitModelResref: "",
    explicitIconResref: "",
    translation: [0, 0, 0],
    rotationDegrees: [0, 0, 0],
    uniformScale: 1,
    pivot: [0, 0, 0],
  }));
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
  if (!Number.isInteger(draft.variant) || draft.variant < 0 || draft.variant > 255) {
    throw new Error(`${draft.field} variant must fit the UTI BYTE field (0..255)`);
  }
  if (draft.sourceKind !== "MESHY_GLB") {
    return {
      ...draft,
      modelResref: "",
      iconResref: "",
      textureResref: "",
    };
  }
  const suffix = draft.token
    ? `_${draft.token}_${String(draft.variant).padStart(3, "0")}`
    : `_${String(draft.variant).padStart(3, "0")}`;
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
  return { ...draft, modelResref, iconResref, textureResref };
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
  const parts = drafts.map((draft, ordinal): ResolvedItemPartDraft => {
    if (draft.sourceKind !== "MESHY_GLB") {
      return resolveItemPartDraft(row, draft, ordinal);
    }
    let resolved: ResolvedItemPartDraft | undefined;
    for (let probe = 0; probe < 256; probe += 1) {
      const variant = (draft.variant + probe) % 256;
      const candidate = resolveItemPartDraft(row, { ...draft, variant }, ordinal);
      const modelKey = resourceKey(2002, candidate.modelResref);
      const emitsCustomIcon = !["IPRP_SPELL", "CLOAK_MODEL", "CAPART_COMPOSITE"]
        .includes(row.capability.iconProfile);
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
  const proofCreature = ["CAPART_COMPOSITE", "CLOAK_MODEL"].includes(
    row.capability.iconProfile,
  )
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
  return {
    blueprintResref: blueprint.resref,
    hakResref: hak.resref,
    moduleResref: module.resref,
    areaResref: area.resref,
    proofCreatureResref: proofCreature?.resref ?? null,
    parts,
    occupiedKeyCount: occupied.size,
    collisionAvoidanceCount,
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
