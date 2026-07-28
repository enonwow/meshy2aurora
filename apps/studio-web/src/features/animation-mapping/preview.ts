import type {
  BinaryMdlInspectionReport,
  ReadbackAnimation,
} from "../preview/types";
import { resolveEffectiveAnimationSourceV1 } from "./fallbacks";
import type {
  AnimationCatalogRowV1,
  AnimationMappingProvenanceV1,
  AnimationSourceAssignmentV1,
  CreatureAnimationAuthoringV1,
  CreatureAnimationMappingDiagnosticV1,
  DirectCreatureBaseSlotV1,
  SourceAnimationClipV1,
} from "./types";

export interface AnimationPreviewSelectionV1 {
  rowId: string;
  targetSlot: DirectCreatureBaseSlotV1 | null;
  source: AnimationSourceAssignmentV1 | null;
  sourceClipName: string | null;
  resultClipName: string | null;
  provenance: AnimationMappingProvenanceV1 | null;
  viaFallbackSlots: DirectCreatureBaseSlotV1[];
}

export interface AnimationPreviewTimingComparisonV1 {
  sourceDurationSeconds: number;
  resultDurationSeconds: number;
  durationDeltaSeconds: number;
  durationDeltaRatio: number;
  resultTransitionSeconds: number;
  sourceEventCount: null;
  resultEventCount: null;
}

export interface AnimationPreviewContextV1 {
  sourceClip?: SourceAnimationClipV1 | null;
  readbackAnimation?: ReadbackAnimation | null;
  builtAuthoringRevision?: number | null;
  currentAuthoringRevision?: number;
  rigCompatible?: boolean;
}

export function createAnimationPreviewSelectionV1(
  row: AnimationCatalogRowV1,
  authoring: CreatureAnimationAuthoringV1,
): AnimationPreviewSelectionV1 {
  if (row.kind === "CUSTOM") {
    const custom = authoring.customAnimations.find(({ id }) => id === row.id);
    const sourceClipName = custom?.sourceClipName
      ?? custom?.phases.find(({ phase }) => phase === "LOOP")?.sourceClipName
      ?? custom?.phases[0]?.sourceClipName
      ?? null;
    return {
      rowId: row.id,
      targetSlot: null,
      source: null,
      sourceClipName,
      resultClipName: custom?.name ?? null,
      provenance: custom?.provenance ?? null,
      viaFallbackSlots: [],
    };
  }
  if (!row.slot) return emptySelection(row.id);
  try {
    const resolved = resolveEffectiveAnimationSourceV1(
      row.slot,
      authoring.assignments,
      authoring.fallbacks,
    );
    return {
      rowId: row.id,
      targetSlot: row.slot,
      source: resolved.assignment,
      sourceClipName: resolved.assignment.sourceClipName,
      resultClipName: row.slot,
      provenance: resolved.assignment.provenance,
      viaFallbackSlots: resolved.viaFallbackSlots,
    };
  } catch {
    return {
      ...emptySelection(row.id),
      targetSlot: row.slot,
      resultClipName: row.slot,
    };
  }
}

export function loadSourceAnimationPreviewV1(
  selection: AnimationPreviewSelectionV1,
  clips: readonly SourceAnimationClipV1[],
): SourceAnimationClipV1 | null {
  const sourceClipName = selection.sourceClipName;
  if (!sourceClipName) return null;
  return clips.find(({ name }) => equalAnimationName(name, sourceClipName)) ?? null;
}

export function loadReadbackAnimationPreviewV1(
  readback: BinaryMdlInspectionReport | null | undefined,
  slot: string | null,
): ReadbackAnimation | null {
  if (!readback || !slot) return null;
  return readback.animations.find(({ name }) => equalAnimationName(name, slot)) ?? null;
}

export function compareAnimationPreviewTimingsV1(
  source: Pick<SourceAnimationClipV1, "durationSeconds">,
  result: Pick<ReadbackAnimation, "length" | "transition">,
): AnimationPreviewTimingComparisonV1 {
  const sourceDurationSeconds = finiteNonNegative(source.durationSeconds);
  const resultDurationSeconds = finiteNonNegative(result.length);
  const durationDeltaSeconds = resultDurationSeconds - sourceDurationSeconds;
  return {
    sourceDurationSeconds,
    resultDurationSeconds,
    durationDeltaSeconds,
    durationDeltaRatio: sourceDurationSeconds > 0
      ? durationDeltaSeconds / sourceDurationSeconds
      : 0,
    resultTransitionSeconds: finiteNonNegative(result.transition),
    // Event markers are authored through a separate verified contract. The
    // current source/readback structures do not expose them, so the preview
    // must not invent a count.
    sourceEventCount: null,
    resultEventCount: null,
  };
}

export function getAnimationPreviewDiagnosticsV1(
  selection: AnimationPreviewSelectionV1,
  context: AnimationPreviewContextV1 = {},
): CreatureAnimationMappingDiagnosticV1[] {
  const diagnostics: CreatureAnimationMappingDiagnosticV1[] = [];
  if (!selection.source) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-PREVIEW-SOURCE-UNRESOLVED",
      `rows.${selection.rowId}`,
      "No effective animation source is assigned to this row.",
      "Assign a compatible source or accept an explicit fallback.",
      "WARNING",
    ));
  } else if (
    selection.source.sourceKind === "SOURCE_CLIP"
    && !context.sourceClip
  ) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-PREVIEW-SOURCE-CLIP-MISSING",
      `rows.${selection.rowId}.source`,
      `Source clip ${selection.sourceClipName ?? "(unnamed)"} is not present in the current GLB inventory.`,
      "Choose a clip from the current source revision.",
      "BLOCKING",
    ));
  }
  if (context.rigCompatible === false) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-PREVIEW-RIG-INCOMPATIBLE",
      `rows.${selection.rowId}.rig`,
      "The selected clip targets nodes outside the compatible creature rig.",
      "Select a rig-compatible clip or repair the source rig.",
      "BLOCKING",
    ));
  }
  if (
    context.builtAuthoringRevision != null
    && context.currentAuthoringRevision != null
    && context.builtAuthoringRevision !== context.currentAuthoringRevision
  ) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-PREVIEW-READBACK-STALE",
      "readback.authoringRevision",
      "The built readback predates the current animation mapping.",
      "Build again before comparing the result.",
      "WARNING",
    ));
  } else if (!context.readbackAnimation) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-PREVIEW-READBACK-MISSING",
      `readback.animations.${selection.resultClipName ?? selection.rowId}`,
      "No current built readback clip is available for this row.",
      "Complete a build to compare the authored mapping with binary MDL readback.",
      "INFO",
    ));
  }
  return diagnostics;
}

function emptySelection(rowId: string): AnimationPreviewSelectionV1 {
  return {
    rowId,
    targetSlot: null,
    source: null,
    sourceClipName: null,
    resultClipName: null,
    provenance: null,
    viaFallbackSlots: [],
  };
}

function equalAnimationName(left: string, right: string) {
  return left.trim().localeCompare(right.trim(), undefined, { sensitivity: "base" }) === 0;
}

function finiteNonNegative(value: number) {
  return Number.isFinite(value) ? Math.max(value, 0) : 0;
}

function diagnostic(
  code: string,
  path: string,
  message: string,
  action: string,
  level: CreatureAnimationMappingDiagnosticV1["level"],
): CreatureAnimationMappingDiagnosticV1 {
  return { schemaVersion: 1, code, path, level, message, action };
}
