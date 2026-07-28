import {
  FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
  type DirectCreatureBaseSlotV1,
} from "../source/directCreatureAnimationProfile";
import sharedCatalog from "../../../../../contracts/creature-animation-catalog-v1.json";
import { resolveEffectiveAnimationSourceV1 } from "./fallbacks";
import type {
  AnimationCatalogFilterV1,
  AnimationCatalogRowV1,
  AnimationSourceAssignmentV1,
  AnimationFallbackDecisionV1,
  AuroraAnimationStateDefinitionV1,
  CreatureAnimationCatalogContractV1,
  CreatureAnimationAuthoringV1,
  CreatureAnimationInspectionV1,
  CustomAnimationDefinitionV1,
  DirectCreatureModelTypeV1,
  PlaybackPolicyV1,
} from "./types";

export function validateDirectCreatureCatalogContractV1(
  contract: CreatureAnimationCatalogContractV1,
): void {
  if (
    contract.schemaVersion !== 1
    || contract.profile !== "DIRECT_CREATURE_S_L_BASE_42_CATALOG_V1"
    || contract.playbackPolicy !== "ENGINE_MANAGED"
    || contract.supportedModelTypes.join(",") !== "S,L"
    || contract.states.length !== FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.length
  ) {
    throw new Error("Invalid shared creature animation catalog v1");
  }
  const stateIds = new Set<string>();
  contract.states.forEach((state, index) => {
    const expectedSlot = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1[index];
    if (
      state.slot !== expectedSlot
      || !state.stateId
      || stateIds.has(state.stateId)
    ) {
      throw new Error(
        `Shared creature animation catalog state ${state.stateId} / slot ${state.slot} is invalid at ${index}`,
      );
    }
    stateIds.add(state.stateId);
  });
}

validateDirectCreatureCatalogContractV1(
  sharedCatalog as CreatureAnimationCatalogContractV1,
);

export const AURORA_ANIMATION_STATE_CATALOG_V1 = Object.freeze(
  sharedCatalog.states.map((state, index): AuroraAnimationStateDefinitionV1 => {
    const expectedSlot = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1[index];
    if (state.slot !== expectedSlot) {
      throw new Error(
        `Shared creature animation catalog slot ${state.slot} does not match ${expectedSlot} at ${index}`,
      );
    }
    return Object.freeze({
      stateId: state.stateId,
      label: state.label,
      description: state.description,
      slot: expectedSlot,
      supportedModelTypes: Object.freeze(["S", "L"] as const),
      playbackPolicy: "ENGINE_MANAGED",
    });
  }),
);

const definitionsByState = new Map(
  AURORA_ANIMATION_STATE_CATALOG_V1.map((definition) => [
    definition.stateId,
    definition,
  ]),
);
const definitionsBySlot = new Map(
  AURORA_ANIMATION_STATE_CATALOG_V1.map((definition) => [
    definition.slot,
    definition,
  ]),
);
const baseSlots = new Set<string>(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1);

export function getDirectCreatureBaseCatalogV1(): readonly AuroraAnimationStateDefinitionV1[] {
  return AURORA_ANIMATION_STATE_CATALOG_V1;
}

export function getAuroraAnimationStateCatalogV1(): readonly AuroraAnimationStateDefinitionV1[] {
  return AURORA_ANIMATION_STATE_CATALOG_V1;
}

export function resolveBaseSlotForStateV1(
  stateId: string,
  modelType: DirectCreatureModelTypeV1,
): DirectCreatureBaseSlotV1 {
  const definition = definitionsByState.get(stateId);
  if (!definition) throw new RangeError(`Unknown Aurora animation state: ${stateId}`);
  if (!definition.supportedModelTypes.includes(modelType)) {
    throw new RangeError(`Aurora animation state ${stateId} does not support MODELTYPE=${modelType}`);
  }
  return definition.slot;
}

export function getPlaybackPolicyForStateV1(stateId: string): PlaybackPolicyV1 {
  const definition = definitionsByState.get(stateId);
  if (!definition) throw new RangeError(`Unknown Aurora animation state: ${stateId}`);
  return definition.playbackPolicy;
}

export function isDirectCreatureBaseSlotV1(
  value: string,
): value is DirectCreatureBaseSlotV1 {
  return baseSlots.has(value);
}

export function findBaseSlotDefinitionV1(
  value: string,
): AuroraAnimationStateDefinitionV1 | undefined {
  return isDirectCreatureBaseSlotV1(value) ? definitionsBySlot.get(value) : undefined;
}

export function assertDirectCreatureCatalogParityV1(rustCatalogJson?: string): true {
  if (AURORA_ANIMATION_STATE_CATALOG_V1.length !== 42) {
    throw new Error("Direct-creature catalog must contain exactly 42 base slots");
  }
  const slots = AURORA_ANIMATION_STATE_CATALOG_V1.map(({ slot }) => slot);
  const states = AURORA_ANIMATION_STATE_CATALOG_V1.map(({ stateId }) => stateId);
  if (
    new Set(slots).size !== 42
    || new Set(states).size !== 42
    || slots.some((slot, index) => slot !== FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1[index])
  ) {
    throw new Error("Direct-creature catalog does not match the canonical 42-slot namespace");
  }
  if (rustCatalogJson !== undefined) {
    let parsed: unknown;
    try {
      parsed = JSON.parse(rustCatalogJson);
    } catch {
      throw new Error("Rust direct-creature catalog export is not valid JSON");
    }
    if (!Array.isArray(parsed) || parsed.length !== 42) {
      throw new Error("Rust direct-creature catalog export must contain exactly 42 rows");
    }
    const local = AURORA_ANIMATION_STATE_CATALOG_V1.map((definition) => ({
      stateId: definition.stateId,
      label: definition.label,
      description: definition.description,
      slot: definition.slot,
      supportedModelTypes: definition.supportedModelTypes,
      playbackPolicy: definition.playbackPolicy,
    }));
    if (JSON.stringify(parsed) !== JSON.stringify(local)) {
      throw new Error("Rust and TypeScript direct-creature catalogs diverge");
    }
  }
  return true;
}

export function projectAnimationCatalogRowsV1(
  authoring: CreatureAnimationAuthoringV1,
  inspection: CreatureAnimationInspectionV1,
): readonly AnimationCatalogRowV1[] {
  const clipsByName = new Map(
    inspection.sourceClips.map((clip) => [clip.name, clip]),
  );
  const assignmentsBySlot = new Map(
    authoring.assignments.map((assignment) => [assignment.targetSlot, assignment]),
  );
  const fallbacksByTarget = new Map(
    authoring.fallbacks.map((fallback) => [fallback.targetSlot, fallback]),
  );
  const customById = new Map(
    authoring.customAnimations.map((custom) => [custom.id, custom]),
  );

  const baseRows = AURORA_ANIMATION_STATE_CATALOG_V1.map((definition) => {
    const assignment = assignmentsBySlot.get(definition.slot);
    const fallback = fallbacksByTarget.get(definition.slot);
    const projection = assignment
      ? projectAssignment(assignment, clipsByName, customById)
      : fallback
        ? projectFallback(
            fallback,
            authoring.assignments,
            authoring.fallbacks,
            clipsByName,
            customById,
          )
        : {
            status: "BLOCKED" as const,
            sourceLabel: null,
            provenance: null,
            diagnosticCodes: ["M2A-ANIMATION-SOURCE-UNASSIGNED"],
          };

    return Object.freeze({
      id: `base:${definition.slot}`,
      kind: "BASE" as const,
      stateId: definition.stateId,
      label: definition.label,
      description: definition.description,
      slot: definition.slot,
      modelType: authoring.modelType,
      playbackPolicy: definition.playbackPolicy,
      ...projection,
    });
  });

  const customRows = authoring.customAnimations.map((custom) => {
    const projection = projectCustomAnimation(custom, clipsByName);
    return Object.freeze({
      id: `custom:${custom.id}`,
      kind: "CUSTOM" as const,
      stateId: null,
      label: custom.name,
      description: custom.playback === "ONE_SHOT"
        ? "User-defined one-shot animation."
        : "User-defined phased looping animation.",
      slot: null,
      modelType: authoring.modelType,
      playbackPolicy: custom.playback,
      ...projection,
    });
  });

  return Object.freeze([...baseRows, ...customRows]);
}

export function filterAnimationCatalogV1(
  rows: readonly AnimationCatalogRowV1[],
  query: string,
  filter: AnimationCatalogFilterV1,
): readonly AnimationCatalogRowV1[] {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  return rows.filter((row) => {
    const inFilter = filter === "NEEDS_ATTENTION"
      ? row.status !== "MAPPED"
      : filter === "BASE_42"
        ? row.kind === "BASE"
        : row.kind === "CUSTOM";
    if (!inFilter) return false;
    if (!normalizedQuery) return true;
    return [
      row.stateId,
      row.label,
      row.description,
      row.slot,
      row.sourceLabel,
      row.status,
    ].some((value) => value?.toLocaleLowerCase().includes(normalizedQuery));
  });
}

function projectAssignment(
  assignment: AnimationSourceAssignmentV1,
  clipsByName: ReadonlyMap<string, unknown>,
  customById: ReadonlyMap<string, CustomAnimationDefinitionV1>,
): RowResolution {
  if (assignment.sourceKind === "SOURCE_CLIP") {
    if (!assignment.sourceClipName || !clipsByName.has(assignment.sourceClipName)) {
      return blockedSourceClip(assignment.sourceClipName, assignment.provenance);
    }
    return mapped(assignment.sourceClipName, assignment.provenance);
  }
  if (assignment.sourceKind === "CUSTOM") {
    if (!assignment.customAnimationId || !customById.has(assignment.customAnimationId)) {
      return {
        status: "BLOCKED",
        sourceLabel: assignment.customAnimationId,
        provenance: assignment.provenance,
        diagnosticCodes: ["M2A-ANIMATION-CUSTOM-SOURCE-MISSING"],
      };
    }
    return mapped(
      customById.get(assignment.customAnimationId)!.name,
      assignment.provenance,
    );
  }
  return mapped(assignment.provenance.assetId, assignment.provenance);
}

function projectFallback(
  fallback: AnimationFallbackDecisionV1,
  assignments: readonly AnimationSourceAssignmentV1[],
  fallbacks: readonly AnimationFallbackDecisionV1[],
  clipsByName: ReadonlyMap<string, unknown>,
  customById: ReadonlyMap<string, CustomAnimationDefinitionV1>,
): RowResolution {
  const sourceAssignment = assignments.find(
    ({ targetSlot }) => targetSlot === fallback.sourceSlot,
  );
  if (fallback.review === "PENDING") {
    return {
      status: "NEEDS_REVIEW",
      sourceLabel: fallback.sourceSlot,
      provenance: sourceAssignment?.provenance ?? null,
      diagnosticCodes: ["M2A-ANIMATION-FALLBACK-REVIEW-REQUIRED"],
    };
  }
  if (fallback.review === "REJECTED") {
    return {
      status: "BLOCKED",
      sourceLabel: fallback.sourceSlot,
      provenance: sourceAssignment?.provenance ?? null,
      diagnosticCodes: ["M2A-ANIMATION-FALLBACK-REJECTED"],
    };
  }
  let resolved;
  try {
    resolved = resolveEffectiveAnimationSourceV1(
      fallback.targetSlot,
      assignments,
      fallbacks,
    );
  } catch {
    return {
      status: "BLOCKED",
      sourceLabel: fallback.sourceSlot,
      provenance: null,
      diagnosticCodes: ["M2A-ANIMATION-FALLBACK-SOURCE-UNASSIGNED"],
    };
  }
  const source = projectAssignment(resolved.assignment, clipsByName, customById);
  return source.status === "MAPPED"
    ? mapped(resolved.resolvedSlot, source.provenance)
    : {
        status: "BLOCKED",
        sourceLabel: resolved.resolvedSlot,
        provenance: source.provenance,
        diagnosticCodes: ["M2A-ANIMATION-FALLBACK-SOURCE-BLOCKED"],
      };
}

function projectCustomAnimation(
  custom: CustomAnimationDefinitionV1,
  clipsByName: ReadonlyMap<string, unknown>,
): RowResolution {
  if (custom.playback === "ONE_SHOT") {
    if (!custom.sourceClipName || !clipsByName.has(custom.sourceClipName)) {
      return blockedSourceClip(custom.sourceClipName, custom.provenance);
    }
    if (custom.phases.length > 0) {
      return {
        status: "BLOCKED",
        sourceLabel: custom.sourceClipName,
        provenance: custom.provenance,
        diagnosticCodes: ["M2A-ANIMATION-CUSTOM-ONE-SHOT-PHASES"],
      };
    }
    return mapped(custom.sourceClipName, custom.provenance);
  }

  const loopPhases = custom.phases.filter(({ phase }) => phase === "LOOP");
  if (custom.sourceClipName !== null || loopPhases.length !== 1) {
    return {
      status: "BLOCKED",
      sourceLabel: loopPhases[0]?.sourceClipName ?? null,
      provenance: custom.provenance,
      diagnosticCodes: ["M2A-ANIMATION-CUSTOM-LOOP-PHASES"],
    };
  }
  const missingPhase = custom.phases.find(
    ({ sourceClipName }) => !clipsByName.has(sourceClipName),
  );
  if (missingPhase) {
    return blockedSourceClip(missingPhase.sourceClipName, custom.provenance);
  }
  return mapped(loopPhases[0].sourceClipName, custom.provenance);
}

type RowResolution = Pick<
  AnimationCatalogRowV1,
  "status" | "sourceLabel" | "provenance" | "diagnosticCodes"
>;

function mapped(
  sourceLabel: string,
  provenance: AnimationCatalogRowV1["provenance"],
): RowResolution {
  return { status: "MAPPED", sourceLabel, provenance, diagnosticCodes: [] };
}

function blockedSourceClip(
  sourceLabel: string | null,
  provenance: AnimationCatalogRowV1["provenance"],
): RowResolution {
  return {
    status: "BLOCKED",
    sourceLabel,
    provenance,
    diagnosticCodes: ["M2A-ANIMATION-SOURCE-CLIP-MISSING"],
  };
}
