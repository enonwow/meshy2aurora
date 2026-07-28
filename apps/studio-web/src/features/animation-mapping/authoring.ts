import {
  findBaseSlotDefinitionV1,
  projectAnimationCatalogRowsV1,
} from "./catalog";
import { rankSourceClipCandidatesV1 } from "./autoMap";
import { reduceCreatureAnimationAuthoringV1 } from "./state";
import type {
  AnimationCatalogRowV1,
  AnimationMappingProvenanceV1,
  AvailableAnimationSourceV1,
  CreatureAnimationAuthoringV1,
  CreatureAnimationInspectionV1,
  DirectCreatureBaseSlotV1,
} from "./types";

export function selectAnimationCatalogRow(
  id: string,
  rows: readonly AnimationCatalogRowV1[],
): AnimationCatalogRowV1 | null {
  return rows.find((row) => row.id === id) ?? null;
}

export function getAnimationAttentionItems(
  authoring: CreatureAnimationAuthoringV1,
  inspection: CreatureAnimationInspectionV1,
): readonly AnimationCatalogRowV1[] {
  return projectAnimationCatalogRowsV1(authoring, inspection)
    .filter(({ status }) => status !== "MAPPED");
}

export function getAvailableAnimationSourcesV1(
  slot: DirectCreatureBaseSlotV1,
  inspection: CreatureAnimationInspectionV1,
): AvailableAnimationSourceV1[] {
  const state = findBaseSlotDefinitionV1(slot);
  if (!state) return [];
  const usableClipIds = new Set(
    inspection.sourceClips
      .filter(({ trackCount }) => trackCount > 0)
      .map(({ clipId }) => clipId),
  );
  return rankSourceClipCandidatesV1(state, inspection.sourceClips)
    .filter(({ clipId }) => usableClipIds.has(clipId))
    .map((candidate) => ({
      clipId: candidate.clipId,
      clipName: candidate.clipName,
      score: candidate.score,
      requiresReview: candidate.score < 0.95,
      reason: candidate.reason,
    }));
}

export function getIncompatibleAnimationSourcesV1(
  slot: DirectCreatureBaseSlotV1,
  inspection: CreatureAnimationInspectionV1,
): Array<{ clipId: string; clipName: string; reason: string }> {
  const rankedIds = new Set(
    getAvailableAnimationSourcesV1(slot, inspection).map(({ clipId }) => clipId),
  );
  return inspection.sourceClips
    .filter(({ clipId }) => !rankedIds.has(clipId))
    .map((clip) => ({
      clipId: clip.clipId,
      clipName: clip.name,
      reason: clip.trackCount === 0
        ? "No animation tracks."
        : `No audited semantic match for ${slot}.`,
    }));
}

export function assignAnimationSourceV1(
  authoring: CreatureAnimationAuthoringV1,
  slot: DirectCreatureBaseSlotV1,
  source: AvailableAnimationSourceV1,
  provenance: AnimationMappingProvenanceV1,
): CreatureAnimationAuthoringV1 {
  return reduceCreatureAnimationAuthoringV1(authoring, {
    type: "ANIMATION_SOURCE_ASSIGNED",
    assignment: {
      targetSlot: slot,
      sourceKind: "SOURCE_CLIP",
      sourceClipName: source.clipName,
      customAnimationId: null,
      provenance,
    },
  });
}

export function clearAnimationSourceV1(
  authoring: CreatureAnimationAuthoringV1,
  slot: DirectCreatureBaseSlotV1,
): CreatureAnimationAuthoringV1 {
  return reduceCreatureAnimationAuthoringV1(authoring, {
    type: "ANIMATION_SOURCE_CLEARED",
    slot,
  });
}
