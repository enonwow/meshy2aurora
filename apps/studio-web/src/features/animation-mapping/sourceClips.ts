import * as THREE from "three";
import {
  FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
  type DirectCreatureBaseSlotV1,
} from "../source/directCreatureAnimationProfile";
import type {
  CreatureAnimationMappingDiagnosticV1,
  SourceAnimationClipV1,
  SourceAnimationCoverageSummaryV1,
  SourceAnimationMeaningCandidateV1,
  SourceAnimationRigV1,
} from "./types";

type AliasCandidate = readonly [
  slot: DirectCreatureBaseSlotV1,
  score: number,
  reasonCode: SourceAnimationMeaningCandidateV1["reasonCode"],
];

const slotOrder = new Map(
  FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((slot, index) => [slot, index]),
);
const normalizedSlots = new Map(
  FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((slot) => [
    normalizeSourceAnimationNameV1(slot),
    slot,
  ]),
);

const semanticAliases = new Map<string, readonly AliasCandidate[]>([
  ["walk", [["cwalk", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["walking", [["cwalk", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["walkcycle", [["cwalk", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["run", [["crun", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["running", [["crun", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["runcycle", [["crun", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["idle", [["cpause1", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["idling", [["cpause1", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["pause", [["cpause1", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["standingidle", [["cpause1", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["taunt", [["ctaunt", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["appear", [["cappear", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["spawn", [["cappear", 0.96, "EXACT_SEMANTIC_ALIAS"]]],
  ["disappear", [["cdisappear", 0.98, "EXACT_SEMANTIC_ALIAS"]]],
  ["despawn", [["cdisappear", 0.96, "EXACT_SEMANTIC_ALIAS"]]],
  ["slashleft", [["ca1slashl", 0.96, "EXACT_SEMANTIC_ALIAS"]]],
  ["leftslash", [["ca1slashl", 0.96, "EXACT_SEMANTIC_ALIAS"]]],
  ["slashright", [["ca1slashr", 0.96, "EXACT_SEMANTIC_ALIAS"]]],
  ["rightslash", [["ca1slashr", 0.96, "EXACT_SEMANTIC_ALIAS"]]],
  ["stab", [["ca1stab", 0.96, "EXACT_SEMANTIC_ALIAS"]]],
  ["damageleft", [["cdamagel", 0.96, "EXACT_SEMANTIC_ALIAS"]]],
  ["damageright", [["cdamager", 0.96, "EXACT_SEMANTIC_ALIAS"]]],
  ["attack", [
    ["ca1slashl", 0.65, "AMBIGUOUS_SEMANTIC_ALIAS"],
    ["ca1slashr", 0.65, "AMBIGUOUS_SEMANTIC_ALIAS"],
    ["ca1stab", 0.65, "AMBIGUOUS_SEMANTIC_ALIAS"],
  ]],
  ["damage", [
    ["cdamagel", 0.65, "AMBIGUOUS_SEMANTIC_ALIAS"],
    ["cdamager", 0.65, "AMBIGUOUS_SEMANTIC_ALIAS"],
    ["cdamages", 0.65, "AMBIGUOUS_SEMANTIC_ALIAS"],
  ]],
  ["dodge", [
    ["cdodgelr", 0.65, "AMBIGUOUS_SEMANTIC_ALIAS"],
    ["cdodges", 0.65, "AMBIGUOUS_SEMANTIC_ALIAS"],
  ]],
  ["death", [
    ["ckdbckdie", 0.75, "AMBIGUOUS_SEMANTIC_ALIAS"],
    ["cdead", 0.75, "AMBIGUOUS_SEMANTIC_ALIAS"],
  ]],
  ["die", [
    ["ckdbckdie", 0.75, "AMBIGUOUS_SEMANTIC_ALIAS"],
    ["cdead", 0.75, "AMBIGUOUS_SEMANTIC_ALIAS"],
  ]],
]);

const meshyActionCandidates = new Map<string, readonly AliasCandidate[]>([
  ["idle", [["cpause1", 0.85, "MESHY_ACTION_CANDIDATE"]]],
  ["attack", [
    ["ca1slashl", 0.65, "MESHY_ACTION_CANDIDATE"],
    ["ca1slashr", 0.65, "MESHY_ACTION_CANDIDATE"],
    ["ca1stab", 0.65, "MESHY_ACTION_CANDIDATE"],
  ]],
  ["dead", [
    ["ckdbckdie", 0.7, "MESHY_ACTION_CANDIDATE"],
    ["cdead", 0.7, "MESHY_ACTION_CANDIDATE"],
  ]],
  ["casualwalkinplace", [["cwalk", 0.82, "MESHY_ACTION_CANDIDATE"]]],
  ["hitreaction", [
    ["cdamagel", 0.7, "MESHY_ACTION_CANDIDATE"],
    ["cdamager", 0.7, "MESHY_ACTION_CANDIDATE"],
    ["cdamages", 0.7, "MESHY_ACTION_CANDIDATE"],
  ]],
  ["knockdown", [
    ["ckdbck", 0.65, "MESHY_ACTION_CANDIDATE"],
    ["ckdbckdie", 0.65, "MESHY_ACTION_CANDIDATE"],
  ]],
]);

export function normalizeSourceAnimationNameV1(name: string): string {
  return name
    .normalize("NFKD")
    .toLocaleLowerCase("en-US")
    .replace(/[^a-z0-9]+/g, "");
}

export function inventorySourceAnimationClipsV1(
  gltfAnimations: readonly THREE.AnimationClip[],
): SourceAnimationClipV1[] {
  return gltfAnimations.map((clip, index) => ({
    clipId: `source-clip:${index}`,
    name: clip.name || `Unnamed clip ${index + 1}`,
    durationSeconds: Number.isFinite(clip.duration) ? Math.max(clip.duration, 0) : 0,
    trackCount: clip.tracks.length,
    targetNodeIds: [],
    targetPaths: [...new Set(clip.tracks.map((track) => track.name))],
  }));
}

export function classifySourceAnimationClipV1(
  clip: SourceAnimationClipV1,
): SourceAnimationMeaningCandidateV1[] {
  const normalizedName = normalizeSourceAnimationNameV1(clip.name);
  const exactSlot = normalizedSlots.get(normalizedName);
  if (exactSlot) {
    return [{
      slot: exactSlot,
      score: 1,
      reasonCode: "EXACT_AURORA_SLOT",
      reason: `Source name matches the canonical Aurora slot ${exactSlot}.`,
    }];
  }
  const hasMeshyActionPrefix = /^\s*\d+\s+/.test(clip.name);
  const semanticKey = hasMeshyActionPrefix
    ? normalizedName.replace(/^\d+/, "")
    : normalizedName;
  const aliases = hasMeshyActionPrefix
    ? meshyActionCandidates.get(semanticKey)
    : semanticAliases.get(semanticKey);
  return (aliases ?? [])
    .map(([slot, score, reasonCode]) => ({
      slot,
      score,
      reasonCode,
      reason: reasonCode === "EXACT_SEMANTIC_ALIAS"
        ? `Source name is a curated semantic alias for ${slot}.`
        : reasonCode === "MESHY_ACTION_CANDIDATE"
          ? `Meshy action name is only a candidate for ${slot} and requires review.`
          : `Source name is ambiguous and may represent ${slot}.`,
    }))
    .sort(compareMeaningCandidates);
}

export function detectSourceClipDuplicatesV1(
  clips: readonly SourceAnimationClipV1[],
): CreatureAnimationMappingDiagnosticV1[] {
  const diagnostics: CreatureAnimationMappingDiagnosticV1[] = [];
  const byNormalizedName = groupBy(clips, (clip) => normalizeSourceAnimationNameV1(clip.name));
  for (const [name, matching] of byNormalizedName) {
    if (name && matching.length > 1) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-SOURCE-NAME-DUPLICATE",
        "sourceClips",
        "BLOCKING",
        `Multiple source clips normalize to ${name}: ${matching.map(({ name: raw }) => raw).join(", ")}.`,
        "Rename or remove duplicate source clips.",
      ));
    }
  }

  const confidentlyClassified = clips.flatMap((clip) => {
    const top = classifySourceAnimationClipV1(clip)[0];
    return top && top.score >= 0.9 ? [{ clip, top }] : [];
  });
  const byMeaning = groupBy(confidentlyClassified, ({ top }) => top.slot);
  for (const [slot, matching] of byMeaning) {
    if (matching.length > 1) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-SOURCE-MEANING-DUPLICATE",
        `sourceClips.${slot}`,
        "BLOCKING",
        `Multiple source clips confidently map to ${slot}: ${matching.map(({ clip }) => clip.name).join(", ")}.`,
        "Keep one source candidate or resolve the mapping manually.",
      ));
    }
  }
  return diagnostics;
}

export function validateSourceClipRigTargetsV1(
  clip: SourceAnimationClipV1,
  rig: SourceAnimationRigV1,
): CreatureAnimationMappingDiagnosticV1[] {
  const diagnostics: CreatureAnimationMappingDiagnosticV1[] = [];
  if (clip.trackCount === 0) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-SOURCE-TRACKS-MISSING",
      `sourceClips.${clip.clipId}.trackCount`,
      "BLOCKING",
      "Source animation clip does not contain animation tracks.",
      "Choose a clip with keyed rig animation.",
    ));
  }
  const nodeIds = new Set(clip.targetNodeIds);
  const targetPaths = new Set(clip.targetPaths);
  const missingNodes = rig.requiredNodeIds.filter((nodeId) => !nodeIds.has(nodeId));
  const missingPaths = rig.requiredTargetPaths.filter((path) => !targetPaths.has(path));
  if (missingNodes.length > 0 || missingPaths.length > 0) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-RIG-TARGET-MISSING",
      `sourceClips.${clip.clipId}.targets`,
      "BLOCKING",
      `Required rig targets are missing: ${[...missingNodes, ...missingPaths].join(", ")}.`,
      "Choose a clip authored for the selected creature rig.",
    ));
  }

  const allowedNodes = new Set(rig.allowedNodeIds);
  const allowedPaths = new Set(rig.allowedTargetPaths);
  const foreignNodes = allowedNodes.size === 0
    ? []
    : clip.targetNodeIds.filter((nodeId) => !allowedNodes.has(nodeId));
  const foreignPaths = allowedPaths.size === 0
    ? []
    : clip.targetPaths.filter((path) => !allowedPaths.has(path));
  if (foreignNodes.length > 0 || foreignPaths.length > 0) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-RIG-TARGET-FOREIGN",
      `sourceClips.${clip.clipId}.targets`,
      "BLOCKING",
      `Clip targets nodes outside the selected rig: ${[...foreignNodes, ...foreignPaths].join(", ")}.`,
      "Retarget the clip or select a compatible rig.",
    ));
  }
  return diagnostics;
}

export function summarizeSourceAnimationCoverageV1(
  clips: readonly SourceAnimationClipV1[],
): SourceAnimationCoverageSummaryV1 {
  const classifications = clips.map((clip) => ({
    clip,
    candidates: classifySourceAnimationClipV1(clip),
  }));
  const confidentlyClassified = classifications.filter(
    ({ candidates }) => (candidates[0]?.score ?? 0) >= 0.9,
  );
  const bySlot = groupBy(confidentlyClassified, ({ candidates }) => candidates[0].slot);
  const confidentlyCoveredSlots = FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.filter(
    (slot) => bySlot.has(slot),
  );
  return {
    sourceClipCount: clips.length,
    confidentlyCoveredSlotCount: confidentlyCoveredSlots.length,
    confidentlyCoveredSlots,
    missingBaseSlotCount:
      FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.length - confidentlyCoveredSlots.length,
    unknownClipCount: classifications.filter(({ candidates }) => candidates.length === 0).length,
    ambiguousClipCount: classifications.filter(
      ({ candidates }) => candidates.length > 0 && (candidates[0]?.score ?? 0) < 0.9,
    ).length,
    conflictCount: [...bySlot.values()].filter((matching) => matching.length > 1).length,
  };
}

function compareMeaningCandidates(
  left: SourceAnimationMeaningCandidateV1,
  right: SourceAnimationMeaningCandidateV1,
) {
  return right.score - left.score
    || (slotOrder.get(left.slot) ?? 0) - (slotOrder.get(right.slot) ?? 0);
}

function groupBy<T>(
  values: readonly T[],
  keyFor: (value: T) => string,
): Map<string, T[]> {
  const groups = new Map<string, T[]>();
  values.forEach((value) => {
    const key = keyFor(value);
    const group = groups.get(key) ?? [];
    group.push(value);
    groups.set(key, group);
  });
  return groups;
}

function diagnostic(
  code: string,
  path: string,
  level: CreatureAnimationMappingDiagnosticV1["level"],
  message: string,
  action: string,
): CreatureAnimationMappingDiagnosticV1 {
  return { schemaVersion: 1, code, path, level, message, action };
}
