import type {
  ItemBaseItemRow,
  ItemBaseItemsCatalog,
  ItemAttachmentProfileV1,
  ItemFitReport,
  ItemPartDraft,
} from "./types";

export type ItemModelPartFieldV2 = "ModelPart1" | "ModelPart2" | "ModelPart3";

export type ItemAuthoringScopeV2 =
  | "ASSEMBLY"
  | "ITEM_PROPERTIES"
  | "GROUND"
  | "EQUIPPED"
  | "INVENTORY_ICON";

export type ItemSemanticCheckIdV2 =
  | "BUTT_OUTER"
  | "GRIP_AT_HAND"
  | "TRIGGER_DOWN"
  | "CORE_CENTERED"
  | "MUZZLE_FORWARD"
  | "BROAD_SIDE_VISIBLE";

export interface ItemAuthoringIdentityV2 {
  readonly schemaVersion: 2;
  readonly archetypeId: "HEXTECH_SHOTGUN";
  /** Standalone product identity emitted from explicit 2DA cells. */
  readonly output: {
    readonly baseItem: 113;
    readonly label: "hextech_shotgun";
    readonly itemClass: "WHxSh";
    readonly modelType: 2;
  };
  readonly sources: Readonly<Record<ItemModelPartFieldV2, {
    readonly role:
      | "source-modelpart-bottom"
      | "source-modelpart-middle"
      | "source-modelpart-top";
    readonly sha256: string;
  }>>;
  readonly reference: {
    readonly id: string;
    readonly baseitemsSha256: string;
    readonly attachmentProfileSha256: string;
  };
}

export interface ItemAssemblyPartV2 {
  readonly field: ItemModelPartFieldV2;
  readonly sourceSha256: string;
  readonly translation: readonly [number, number, number];
  readonly rotationXyzw: readonly [number, number, number, number];
  readonly authoredRotationDegrees: readonly [number, number, number];
  /** Exact reference-derived scale. 100% is not synonymous with 1. */
  readonly uniformScale: number;
  readonly pivot: readonly [number, number, number];
  readonly targetSpaceScaleXyz: readonly [number, number, number];
}

export interface ItemAuthoringRecipeV2 {
  readonly schemaVersion: 2;
  readonly identity: ItemAuthoringIdentityV2;
  readonly assembly: {
    readonly mode: "AUTO_FIT" | "MANUAL_FIT";
    readonly baselineFitSolutionSha256: string;
    readonly validatedFitSolutionSha256: string;
    readonly parts: readonly ItemAssemblyPartV2[];
  };
  readonly contexts: {
    readonly ITEM_PROPERTIES: {
      readonly cameraPreset: "AURORA_ITEM_PROPERTIES";
      readonly axialRotationDegrees: 0 | 90 | 180 | 270;
    };
    readonly GROUND: {
      readonly bearingDegrees: 0 | 90 | 180 | 270;
    };
    readonly EQUIPPED: {
      readonly attachmentRoute: "HAND";
      readonly attachmentProfileSha256: string;
    };
    readonly INVENTORY_ICON: {
      readonly invSlotWidth: number;
      readonly invSlotHeight: number;
      readonly rotationDegrees: 0 | 90 | 180 | 270;
      readonly zoom: number;
      readonly padding: number;
      readonly offset: readonly [number, number];
    };
  };
  readonly review: {
    readonly candidateSha256: string | null;
    readonly technicalStatus: "NOT_VALIDATED" | "PASSED" | "FAILED";
    readonly semanticChecks: readonly {
      readonly id: ItemSemanticCheckIdV2;
      readonly status: "NOT_EVALUATED" | "PASSED" | "FAILED";
      readonly evidence: "CANDIDATE_BOUND_REVIEW" | "MISSING";
    }[];
    readonly ownerStatus: "NOT_REVIEWED" | "OWNER_ACCEPTED" | "OWNER_REJECTED";
  };
}

export interface ItemManualFitSnapshotV2 {
  readonly schemaVersion: 2;
  readonly baselineFitSolutionSha256: string;
  readonly baselineFitReportJson: string;
  readonly parts: readonly ItemAssemblyPartV2[];
}

export interface ItemDirectedCompositionIdentityV2 {
  readonly outputBaseItem: number;
  readonly referenceId: string;
  readonly sourceSha256ByField: Readonly<Record<ItemModelPartFieldV2, string>>;
}

export interface ItemDirectedCompositionContractV2 {
  readonly schemaVersion: 2;
  readonly id: "HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2";
  readonly archetypeId: "HEXTECH_SHOTGUN";
  readonly outputBaseItem: 113;
  readonly referenceId: string;
  readonly validationTolerance: 0.005;
  readonly sourceSha256ByField: Readonly<Record<ItemModelPartFieldV2, string>>;
  readonly parts: readonly ItemAssemblyPartV2[];
  /** Technical candidate only. This is deliberately separate from accepted recipes. */
  readonly ownerStatus: "NOT_REVIEWED";
  readonly correction: {
    readonly fields: readonly ["ModelPart1", "ModelPart2", "ModelPart3"];
    readonly reason: "AUTHOR_MANUAL_SOURCE_FRAME_ALIGNMENT";
    readonly allowedTransformFields: readonly ["translation", "rotation"];
  };
}

export interface ItemAuthoringRecipeValidationV2 {
  readonly ok: boolean;
  readonly issues: readonly string[];
}

const MODEL_PART_FIELDS: readonly ItemModelPartFieldV2[] = [
  "ModelPart1",
  "ModelPart2",
  "ModelPart3",
];

const SEMANTIC_CHECKS: readonly ItemSemanticCheckIdV2[] = [
  "BUTT_OUTER",
  "GRIP_AT_HAND",
  "TRIGGER_DOWN",
  "CORE_CENTERED",
  "MUZZLE_FORWARD",
  "BROAD_SIDE_VISIBLE",
];

const SCOPES: readonly ItemAuthoringScopeV2[] = [
  "ASSEMBLY",
  "ITEM_PROPERTIES",
  "GROUND",
  "EQUIPPED",
  "INVENTORY_ICON",
];

const SHA256 = /^[0-9a-f]{64}$/;

export const HEXTECH_SHOTGUN_BASEITEM_V2 = {
  outputBaseItem: 113,
  outputLabel: "hextech_shotgun",
  outputItemClass: "WHxSh",
  invSlotWidth: 2,
  invSlotHeight: 4,
} as const;

export const HEXTECH_SHOTGUN_STANDALONE_CELLS_V3 = [
  ["ModelType", "2"],
  ["GenderSpecific", "0"],
  ["DefaultModel", "it_bag"],
  ["DefaultIcon", "iwhxsh"],
  ["EquipableSlots", "0x00030"],
  ["InvSlotWidth", "2"],
  ["InvSlotHeight", "4"],
  ["MinRange", "10"],
  ["MaxRange", "100"],
  ["WeaponWield", "6"],
  ["WeaponType", "1"],
  ["RangedWeapon", "27"],
  ["AmmunitionType", "3"],
] as const;

/**
 * Exact author-positioned web candidate. These numbers are the result of the
 * manual source comparison; no image or retail item is consumed at runtime.
 */
export const HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2:
ItemDirectedCompositionContractV2 = {
  schemaVersion: 2,
  id: "HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2",
  archetypeId: "HEXTECH_SHOTGUN",
  outputBaseItem: 113,
  referenceId: "hextech-shotgun-manual-assembly-v3",
  validationTolerance: 0.005,
  sourceSha256ByField: {
    ModelPart1: "69c78999590b248bf9c642516ffa595d33774ead3436166963b27dfaa71ad48d",
    ModelPart2: "8fafe6a55dd77107a67f29c7519f3b6edc390b310f918a89131b003517720147",
    ModelPart3: "6ce1281a4ed8a239bf0d6fc9388fe8a977a2811750d40eab4320e13b642c77bf",
  },
  parts: [
    {
      field: "ModelPart1",
      sourceSha256: "69c78999590b248bf9c642516ffa595d33774ead3436166963b27dfaa71ad48d",
      translation: [-0.00431, -0.15773459, 0.13717034],
      rotationXyzw: [Math.SQRT1_2, -Math.SQRT1_2, 0, 0],
      authoredRotationDegrees: [180, 0, 90],
      uniformScale: 0.15796308,
      pivot: [0, 0, 0],
      targetSpaceScaleXyz: [1, 1, 1],
    },
    {
      field: "ModelPart2",
      sourceSha256: "8fafe6a55dd77107a67f29c7519f3b6edc390b310f918a89131b003517720147",
      translation: [-0.00431, 0.18716541, -0.00852165],
      rotationXyzw: [0, 0, -Math.SQRT1_2, Math.SQRT1_2],
      authoredRotationDegrees: [0, 0, -90],
      uniformScale: 0.21066014,
      pivot: [0, 0, 0],
      targetSpaceScaleXyz: [1, 1, 1],
    },
    {
      field: "ModelPart3",
      sourceSha256: "6ce1281a4ed8a239bf0d6fc9388fe8a977a2811750d40eab4320e13b642c77bf",
      translation: [-0.00431, 0.49844033, 0.008945521],
      rotationXyzw: [0, 0, -Math.SQRT1_2, Math.SQRT1_2],
      authoredRotationDegrees: [0, 0, -90],
      uniformScale: 0.13161969,
      pivot: [0, 0, 0],
      targetSpaceScaleXyz: [1, 1, 1],
    },
  ],
  ownerStatus: "NOT_REVIEWED",
  correction: {
    fields: ["ModelPart1", "ModelPart2", "ModelPart3"],
    reason: "AUTHOR_MANUAL_SOURCE_FRAME_ALIGNMENT",
    allowedTransformFields: ["translation", "rotation"],
  },
};

export function buildHextechShotgunAuthoredAttachmentProfileV3(
  baseitemsSha256: string,
): ItemAttachmentProfileV1 {
  const bounds = [
    [[-0.026681999, -0.3077346, 0.00862060356], [0.018061997, -0.007734582, 0.13717034]],
    [[-0.043864924, -0.012834594, -0.00852165], [0.035244923, 0.38716543, 0.11570865]],
    [[-0.050884686, 0.37344033, 0.008945521], [0.042264685, 0.6234403, 0.09897576]],
  ] as const;
  const hashes = HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2.sourceSha256ByField;
  return {
    schemaVersion: 1,
    algorithm: "AURORA_ITEM_REFERENCE_PROFILE_V1",
    status: "PASSED",
    identity: {
      schemaVersion: 1,
      resourceContextSha256: "01a78a8d378147d2551190b0ce64f7c9df9aab2324cb2284262bd194d33ffa71",
      baseitemsSha256,
      baseItem: 113,
      itemClass: "WHxSh",
      modelType: 2,
      referenceKind: "AUTHOR_DIRECTED_SOURCE_FRAME",
      referenceId: HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2.referenceId,
    },
    attachmentRoute: "HAND",
    equipableSlots: 0x00030,
    commonOrigin: [0, 0, 0],
    axialAxis: 1,
    widthAxis: 2,
    depthAxis: 0,
    attachmentZoneMin: [-0.050884686, -0.3077346, -0.00852165],
    attachmentZoneMax: [0.042264685, 0.6234403, 0.13717034],
    attachmentEvidence: "AUTHOR_MANUAL_ALIGNMENT_V1",
    slots: (["ModelPart1", "ModelPart2", "ModelPart3"] as const).map((field, index) => ({
      field,
      label: ["Bottom", "Middle", "Top"][index],
      token: ["b", "m", "t"][index],
      modelResref: ["whxsh_b_001", "whxsh_m_001", "whxsh_t_001"][index],
      modelSha256: hashes[field],
      controllerNodeName: `m2a_authored_slot_${index + 1}`,
      controllerTranslation: index === 1
        ? [-0.00431, 0, 0.0535935]
        : [
            (bounds[index][0][0] + bounds[index][1][0]) * 0.5,
            (bounds[index][0][1] + bounds[index][1][1]) * 0.5,
            (bounds[index][0][2] + bounds[index][1][2]) * 0.5,
          ],
      controllerRotationXyzw: [0, 0, 0, 1],
      boundsMin: bounds[index][0],
      boundsMax: bounds[index][1],
      allowAxialExtensionAtMin: false,
      allowAxialExtensionAtMax: false,
    })),
    profileSha256: "",
  };
}

function directedCompositionIdentityMatchesV2(
  contract: ItemDirectedCompositionContractV2,
  observed: ItemDirectedCompositionIdentityV2,
) {
  return contract.outputBaseItem === observed.outputBaseItem
    && contract.referenceId === observed.referenceId
    && MODEL_PART_FIELDS.every((field) => (
      contract.sourceSha256ByField[field] === observed.sourceSha256ByField[field]
    ));
}

export function resolveOwnerDirectedItemCompositionV2(
  observed: ItemDirectedCompositionIdentityV2,
): ItemDirectedCompositionContractV2 | undefined {
  return directedCompositionIdentityMatchesV2(
    HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2,
    observed,
  ) ? HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2 : undefined;
}

export function applyOwnerDirectedItemCompositionV2(
  parts: readonly ItemPartDraft[],
  contract: ItemDirectedCompositionContractV2,
  observed: ItemDirectedCompositionIdentityV2,
): ItemPartDraft[] {
  if (!directedCompositionIdentityMatchesV2(contract, observed)) {
    throw new Error("Owner-directed composition identity does not match the current candidate inputs.");
  }
  const meshFields = parts
    .filter(({ sourceKind }) => sourceKind === "MESHY_GLB")
    .map(({ field }) => field);
  if (
    meshFields.length !== MODEL_PART_FIELDS.length
    || MODEL_PART_FIELDS.some((field) => !meshFields.includes(field))
  ) {
    throw new Error("Current Item parts do not match the directed ModelPart1/2/3 composition.");
  }
  const transforms = new Map(contract.parts.map((part) => [part.field, part]));
  return parts.map((part) => {
    const transform = transforms.get(part.field as ItemModelPartFieldV2);
    if (!transform) return part;
    return {
      ...part,
      translation: [...transform.translation],
      rotationDegrees: [...transform.authoredRotationDegrees],
      rotationXyzw: [...transform.rotationXyzw],
      uniformScale: transform.uniformScale,
      pivot: [...transform.pivot],
      targetSpaceScaleXyz: [...transform.targetSpaceScaleXyz],
    };
  });
}

export function itemPartsMatchDirectedCompositionV2(
  parts: readonly ItemPartDraft[],
  contract: ItemDirectedCompositionContractV2,
) {
  const numberMatches = (first: number, second: number) => (
    first === second || Math.fround(first) === Math.fround(second)
  );
  const tupleMatches = (first: readonly number[], second: readonly number[]) => (
    first.length === second.length
    && first.every((value, index) => numberMatches(value, second[index]))
  );
  const transforms = new Map(contract.parts.map((part) => [part.field, part]));
  return parts.filter(({ sourceKind }) => sourceKind === "MESHY_GLB").length === 3
    && parts.every((part) => {
      if (part.sourceKind !== "MESHY_GLB") return true;
      const transform = transforms.get(part.field as ItemModelPartFieldV2);
      return Boolean(transform)
        && tupleMatches(part.translation, transform!.translation)
        && tupleMatches(part.rotationXyzw, transform!.rotationXyzw)
        && numberMatches(part.uniformScale, transform!.uniformScale)
        && tupleMatches(part.pivot, transform!.pivot)
        && tupleMatches(part.targetSpaceScaleXyz, transform!.targetSpaceScaleXyz);
    });
}

export function itemPartSupportsReferenceScalingV2(
  baseItem: number,
  field: string,
  attachmentProfile: ItemAttachmentProfileV1 | undefined,
) {
  const slot = attachmentProfile?.slots.find((candidate) => candidate.field === field);
  return !slot || slot.allowAxialExtensionAtMin || slot.allowAxialExtensionAtMax;
}

/**
 * Projects a standalone editor row after proving that 113 is the exact next
 * physical index. No existing row is read or cloned.
 */
export function deriveHextechShotgunOutputRowV2(
  catalog: ItemBaseItemsCatalog,
): ItemBaseItemRow {
  const identity = HEXTECH_SHOTGUN_BASEITEM_V2;
  if (catalog.physicalRowCount !== identity.outputBaseItem) {
    throw new Error(
      `Hextech Shotgun requires exact next BaseItem ${identity.outputBaseItem}; got ${catalog.physicalRowCount}.`,
    );
  }
  if (catalog.rows.some(({ baseItem }) => baseItem === identity.outputBaseItem)) {
    throw new Error(`BaseItem ${identity.outputBaseItem} already exists in the selected table.`);
  }
  return {
    schemaVersion: 1,
    baseItem: identity.outputBaseItem,
    label: identity.outputLabel,
    itemClass: identity.outputItemClass,
    modelType: 2,
    minRange: 10,
    maxRange: 100,
    genderSpecific: false,
    defaultModel: "it_bag",
    defaultIcon: "iwhxsh",
    equipableSlots: 0x00030,
    invSlotWidth: identity.invSlotWidth,
    invSlotHeight: identity.invSlotHeight,
    weaponWield: 6,
    weaponType: 1,
    rangedWeapon: 27,
    ammunitionType: 3,
    capability: {
      schemaVersion: 1,
      compositionProfile: "BOTTOM_MIDDLE_TOP",
      textureProfile: "DIRECT_COLOR",
      iconProfile: "STANDARD",
      meshySourceCount: 3,
      // An equipable ModelType 2 item is resolved against the player
      // appearance table. Keep the editor contract in lockstep with the
      // authoritative Rust capability resolver so the source step requests
      // this exact retail input before Build.
      requiredReferenceTables: ["Appearance"],
    },
    partSlots: [
      { index: 1, field: "ModelPart1", label: "Bottom", token: "b", sourceKind: "MESHY_GLB", referenceTable: null, requiresExplicitResourceResrefs: false },
      { index: 2, field: "ModelPart2", label: "Middle", token: "m", sourceKind: "MESHY_GLB", referenceTable: null, requiresExplicitResourceResrefs: false },
      { index: 3, field: "ModelPart3", label: "Top", token: "t", sourceKind: "MESHY_GLB", referenceTable: null, requiresExplicitResourceResrefs: false },
    ],
    colorFields: [],
  };
}

function canonicalValue(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.entries(value as Record<string, unknown>)
      .sort(([first], [second]) => first.localeCompare(second))
      .map(([key, nested]) => [key, canonicalValue(nested)]));
  }
  return value;
}

export function canonicalItemAuthoringRecipeV2(recipe: ItemAuthoringRecipeV2): string {
  return JSON.stringify(canonicalValue(recipe));
}

export async function hashItemAuthoringRecipeV2(recipe: ItemAuthoringRecipeV2): Promise<string> {
  const bytes = new TextEncoder().encode(canonicalItemAuthoringRecipeV2(recipe));
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((value) => value.toString(16).padStart(2, "0"))
    .join("");
}

function identityFingerprint(identity: ItemAuthoringIdentityV2) {
  return JSON.stringify(canonicalValue(identity));
}

function finiteTuple(values: readonly number[], size: number) {
  return values.length === size && values.every(Number.isFinite);
}

export function validateItemAuthoringRecipeV2(
  recipe: ItemAuthoringRecipeV2,
): ItemAuthoringRecipeValidationV2 {
  const issues: string[] = [];
  const { identity } = recipe;
  if (recipe.schemaVersion !== 2 || identity.schemaVersion !== 2) {
    issues.push("recipe and identity must use schema version 2");
  }
  if (
    identity.output.baseItem !== 113
    || identity.output.itemClass !== "WHxSh"
    || identity.output.label !== "hextech_shotgun"
  ) {
    issues.push("standalone output BaseItem 113 identity is invalid");
  }
  if (!SHA256.test(identity.reference.baseitemsSha256)) {
    issues.push("baseitemsSha256 is invalid");
  }
  if (!SHA256.test(identity.reference.attachmentProfileSha256)) {
    issues.push("attachmentProfileSha256 is invalid");
  }
  for (const field of MODEL_PART_FIELDS) {
    if (!SHA256.test(identity.sources[field]?.sha256 ?? "")) {
      issues.push(`${field} source SHA-256 is invalid`);
    }
  }
  if (!SHA256.test(recipe.assembly.baselineFitSolutionSha256)) {
    issues.push("baseline fit solution SHA-256 is invalid");
  }
  if (!SHA256.test(recipe.assembly.validatedFitSolutionSha256)) {
    issues.push("validated fit solution SHA-256 is invalid");
  }
  const assemblyFields = recipe.assembly.parts.map(({ field }) => field);
  if (
    assemblyFields.length !== MODEL_PART_FIELDS.length
    || new Set(assemblyFields).size !== MODEL_PART_FIELDS.length
    || MODEL_PART_FIELDS.some((field) => !assemblyFields.includes(field))
  ) {
    issues.push("assembly must bind ModelPart1, ModelPart2 and ModelPart3 exactly once");
  }
  for (const part of recipe.assembly.parts) {
    if (part.sourceSha256 !== identity.sources[part.field]?.sha256) {
      issues.push(`${part.field} assembly source does not match recipe identity`);
    }
    if (
      !finiteTuple(part.translation, 3)
      || !finiteTuple(part.rotationXyzw, 4)
      || !finiteTuple(part.authoredRotationDegrees, 3)
      || !finiteTuple(part.pivot, 3)
      || !finiteTuple(part.targetSpaceScaleXyz, 3)
      || !Number.isFinite(part.uniformScale)
      || part.uniformScale <= 0
    ) {
      issues.push(`${part.field} contains an invalid authored transform`);
    }
  }
  if (
    recipe.contexts.EQUIPPED.attachmentProfileSha256
    !== identity.reference.attachmentProfileSha256
  ) {
    issues.push("equipped context is not bound to the identity attachment profile");
  }
  if (
    recipe.contexts.INVENTORY_ICON.invSlotWidth !== 2
    || recipe.contexts.INVENTORY_ICON.invSlotHeight !== 4
  ) {
    issues.push("Hextech Shotgun inventory footprint must be 2x4");
  }
  if (recipe.review.technicalStatus !== "PASSED") {
    issues.push("technical validation is not PASSED");
  }
  if (!SHA256.test(recipe.review.candidateSha256 ?? "")) {
    issues.push("review is not bound to an exact candidate SHA-256");
  }
  for (const id of SEMANTIC_CHECKS) {
    const matches = recipe.review.semanticChecks.filter((check) => check.id === id);
    if (matches.length !== 1) {
      issues.push(`semantic check ${id} must occur exactly once`);
    } else if (matches[0].status !== "PASSED") {
      issues.push(`semantic check ${id} is not PASSED`);
    } else if (matches[0].evidence !== "CANDIDATE_BOUND_REVIEW") {
      issues.push(`semantic check ${id} has no candidate-bound evidence`);
    }
  }
  if (recipe.review.ownerStatus !== "OWNER_ACCEPTED") {
    issues.push("recipe is not owner-accepted");
  }
  return { ok: issues.length === 0, issues };
}

/**
 * Deliberately empty. An accepted recipe is added only after exact candidate
 * review; file recognition alone must never activate an approximate preset.
 */
const ACCEPTED_ITEM_AUTHORING_RECIPES_V2: readonly ItemAuthoringRecipeV2[] = [];

export function resolveAcceptedItemAuthoringRecipeV2(
  identity: ItemAuthoringIdentityV2,
): ItemAuthoringRecipeV2 | undefined {
  const fingerprint = identityFingerprint(identity);
  return ACCEPTED_ITEM_AUTHORING_RECIPES_V2.find((recipe) => (
    identityFingerprint(recipe.identity) === fingerprint
    && validateItemAuthoringRecipeV2(recipe).ok
  ));
}

export function applyAcceptedItemAuthoringRecipeV2(
  parts: readonly ItemPartDraft[],
  recipe: ItemAuthoringRecipeV2,
  observedIdentity: ItemAuthoringIdentityV2,
): ItemPartDraft[] {
  if (identityFingerprint(recipe.identity) !== identityFingerprint(observedIdentity)) {
    throw new Error("Item authoring recipe identity does not match the current candidate inputs.");
  }
  const validation = validateItemAuthoringRecipeV2(recipe);
  if (!validation.ok) {
    throw new Error(`Item authoring recipe is not owner-accepted: ${validation.issues.join("; ")}`);
  }
  const transforms = new Map(recipe.assembly.parts.map((part) => [part.field, part]));
  const meshFields = parts
    .filter(({ sourceKind }) => sourceKind === "MESHY_GLB")
    .map(({ field }) => field);
  if (
    meshFields.length !== MODEL_PART_FIELDS.length
    || MODEL_PART_FIELDS.some((field) => !meshFields.includes(field))
  ) {
    throw new Error("Current Item parts do not match the accepted ModelPart1/2/3 recipe.");
  }
  return parts.map((part) => {
    const transform = transforms.get(part.field as ItemModelPartFieldV2);
    if (!transform) return part;
    return {
      ...part,
      translation: [...transform.translation],
      rotationDegrees: [...transform.authoredRotationDegrees],
      rotationXyzw: [...transform.rotationXyzw],
      uniformScale: transform.uniformScale,
      pivot: [...transform.pivot],
      targetSpaceScaleXyz: [...transform.targetSpaceScaleXyz],
    };
  });
}

export function buildItemManualFitSnapshotV2(
  parts: readonly ItemPartDraft[],
  baseline: ItemFitReport,
): ItemManualFitSnapshotV2 {
  if (!SHA256.test(baseline.solutionSha256)) {
    throw new Error("Manual fit baseline has no exact solution SHA-256.");
  }
  const fittedParts = baseline.parts.map((fitPart) => {
    const part = parts.find((candidate) => candidate.field === fitPart.field);
    if (!part || part.sourceKind !== "MESHY_GLB") {
      throw new Error(`${fitPart.field} is missing from the manual fit editor state.`);
    }
    if (!SHA256.test(fitPart.sourceSha256)) {
      throw new Error(`${fitPart.field} has no exact source SHA-256.`);
    }
    return {
      field: fitPart.field as ItemModelPartFieldV2,
      sourceSha256: fitPart.sourceSha256,
      translation: [...part.translation] as [number, number, number],
      rotationXyzw: [...part.rotationXyzw] as [number, number, number, number],
      authoredRotationDegrees: [...part.rotationDegrees] as [number, number, number],
      uniformScale: part.uniformScale,
      pivot: [...part.pivot] as [number, number, number],
      targetSpaceScaleXyz: [...part.targetSpaceScaleXyz] as [number, number, number],
    };
  });
  const meshPartCount = parts.filter(({ sourceKind }) => sourceKind === "MESHY_GLB").length;
  if (
    fittedParts.length !== meshPartCount
    || new Set(fittedParts.map(({ field }) => field)).size !== fittedParts.length
  ) {
    throw new Error("Manual fit snapshot must bind every fitted Meshy part exactly once.");
  }
  return {
    schemaVersion: 2,
    baselineFitSolutionSha256: baseline.solutionSha256,
    baselineFitReportJson: JSON.stringify(baseline),
    parts: fittedParts,
  };
}

function itemAuthoringScopeFingerprintsV2(recipe: ItemAuthoringRecipeV2) {
  const assembly = JSON.stringify(canonicalValue({
    identity: recipe.identity,
    assembly: recipe.assembly,
  }));
  return {
    ASSEMBLY: assembly,
    ITEM_PROPERTIES: JSON.stringify(canonicalValue({
      assembly,
      presentation: recipe.contexts.ITEM_PROPERTIES,
    })),
    GROUND: JSON.stringify(canonicalValue({
      assembly,
      presentation: recipe.contexts.GROUND,
    })),
    EQUIPPED: JSON.stringify(canonicalValue({
      assembly,
      presentation: recipe.contexts.EQUIPPED,
    })),
    INVENTORY_ICON: JSON.stringify(canonicalValue({
      assembly,
      presentation: recipe.contexts.INVENTORY_ICON,
    })),
  } as const;
}

export function diffItemAuthoringScopesV2(
  before: ItemAuthoringRecipeV2,
  after: ItemAuthoringRecipeV2,
): ItemAuthoringScopeV2[] {
  const first = itemAuthoringScopeFingerprintsV2(before);
  const second = itemAuthoringScopeFingerprintsV2(after);
  return SCOPES.filter((scope) => first[scope] !== second[scope]);
}
