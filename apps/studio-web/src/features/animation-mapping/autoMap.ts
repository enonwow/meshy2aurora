import type {
  AnimationMappingProposalItemV1,
  AnimationSourceAssignmentV1,
  AuroraAnimationStateDefinitionV1,
  CreatureAnimationAuthoringV1,
  CreatureAnimationMappingProposalV1,
  DirectCreatureBaseSlotV1,
  RankedSourceClipCandidateV1,
  SourceAnimationClipV1,
} from "./types";
import {
  classifySourceAnimationClipV1,
  normalizeSourceAnimationNameV1,
} from "./sourceClips";

export {
  approveFallbackV1,
  createFallbackProposalV1,
  rejectFallbackV1,
} from "./fallbacks";

export function rankSourceClipCandidatesV1(
  state: AuroraAnimationStateDefinitionV1,
  clips: readonly SourceAnimationClipV1[],
): RankedSourceClipCandidateV1[] {
  return clips
    .flatMap((clip) => classifySourceAnimationClipV1(clip)
      .filter(({ slot }) => slot === state.slot)
      .map((candidate): RankedSourceClipCandidateV1 => ({
        targetSlot: state.slot,
        clipId: clip.clipId,
        clipName: clip.name,
        score: candidate.score,
        reasonCode: candidate.reasonCode,
        reason: candidate.reason,
      })))
    .sort((left, right) => (
      right.score - left.score
      || compareText(
        normalizeSourceAnimationNameV1(left.clipName),
        normalizeSourceAnimationNameV1(right.clipName),
      )
      || compareText(left.clipId, right.clipId)
    ));
}

export function proposeCreatureAnimationMappingV1(
  catalog: readonly AuroraAnimationStateDefinitionV1[],
  clips: readonly SourceAnimationClipV1[],
): CreatureAnimationMappingProposalV1 {
  return {
    schemaVersion: 1,
    items: catalog.map((state) => {
      const candidates = rankSourceClipCandidatesV1(state, clips);
      const top = candidates[0];
      const highConfidenceCandidates = candidates.filter(({ score }) => score >= 0.9);
      const decision = top
        ? top.score >= 0.95 && highConfidenceCandidates.length === 1
          ? "AUTO_ASSIGN"
          : "REVIEW"
        : "UNRESOLVED";
      return {
        stateId: state.stateId,
        targetSlot: state.slot,
        decision,
        recommendedClipId: top?.clipId ?? null,
        confidence: top?.score ?? 0,
        candidates,
        reason: proposalReason(decision, top),
      };
    }),
  };
}

export function applyHighConfidenceAssignmentsV1(
  authoring: CreatureAnimationAuthoringV1,
  proposal: CreatureAnimationMappingProposalV1,
): CreatureAnimationAuthoringV1 {
  const assignedSlots = new Set(authoring.assignments.map(({ targetSlot }) => targetSlot));
  const additions = proposal.items.flatMap((item) => {
    if (item.decision !== "AUTO_ASSIGN" || assignedSlots.has(item.targetSlot)) return [];
    const assignment = assignmentFromProposalItem(authoring, item);
    if (!assignment) return [];
    assignedSlots.add(item.targetSlot);
    return [assignment];
  });
  if (additions.length === 0) return authoring;
  return {
    ...authoring,
    authoringRevision: authoring.authoringRevision + 1,
    assignments: [...authoring.assignments, ...additions],
  };
}

export function resetAssignmentToProposalV1(
  authoring: CreatureAnimationAuthoringV1,
  slot: DirectCreatureBaseSlotV1,
  proposal: CreatureAnimationMappingProposalV1,
): CreatureAnimationAuthoringV1 {
  const proposalItem = proposal.items.find(({ targetSlot }) => targetSlot === slot);
  if (!proposalItem) throw new RangeError(`Proposal does not contain base slot ${slot}`);
  const proposed = proposalItem.decision === "AUTO_ASSIGN"
    ? assignmentFromProposalItem(authoring, proposalItem)
    : null;
  const assignments = authoring.assignments.filter(({ targetSlot }) => targetSlot !== slot);
  if (proposed) assignments.push(proposed);
  const previous = authoring.assignments.find(({ targetSlot }) => targetSlot === slot);
  if (sameAssignment(previous, proposed)) return authoring;
  return {
    ...authoring,
    authoringRevision: authoring.authoringRevision + 1,
    assignments,
  };
}

export function explainMappingProposalV1(
  proposal: AnimationMappingProposalItemV1,
): string {
  if (proposal.decision === "AUTO_ASSIGN") {
    return `${proposal.targetSlot} can be assigned automatically: ${proposal.reason}`;
  }
  if (proposal.decision === "REVIEW") {
    return `${proposal.targetSlot} needs review: ${proposal.reason}`;
  }
  return `${proposal.targetSlot} is unresolved: ${proposal.reason}`;
}

function assignmentFromProposalItem(
  authoring: CreatureAnimationAuthoringV1,
  item: AnimationMappingProposalItemV1,
): AnimationSourceAssignmentV1 | null {
  const candidate = item.candidates.find(({ clipId }) => clipId === item.recommendedClipId);
  if (!candidate) return null;
  return {
    targetSlot: item.targetSlot,
    sourceKind: "SOURCE_CLIP",
    sourceClipName: candidate.clipName,
    customAnimationId: null,
    provenance: {
      provider: "SOURCE_GLB",
      assetId: authoring.sourceRevision,
      ownership: "USER_OWNED",
    },
  };
}

function sameAssignment(
  left: AnimationSourceAssignmentV1 | undefined,
  right: AnimationSourceAssignmentV1 | null,
): boolean {
  if (!left || !right) return !left && !right;
  return JSON.stringify(left) === JSON.stringify(right);
}

function proposalReason(
  decision: AnimationMappingProposalItemV1["decision"],
  top: RankedSourceClipCandidateV1 | undefined,
): string {
  if (!top) return "No source clip matches this Aurora base slot.";
  if (decision === "AUTO_ASSIGN") {
    return `${top.clipName} is the only high-confidence candidate (${top.reasonCode}).`;
  }
  return `${top.clipName} is not an unambiguous high-confidence match (${top.reasonCode}).`;
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
