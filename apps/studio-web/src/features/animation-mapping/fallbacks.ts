import type {
  AnimationFallbackDecisionV1,
  AnimationMappingDiagnosticV1,
  AnimationSourceAssignmentV1,
  CreatureAnimationAuthoringV1,
  DirectCreatureBaseSlotV1,
} from "./types";

export interface EffectiveAnimationSourceV1 {
  resolvedSlot: DirectCreatureBaseSlotV1;
  assignment: AnimationSourceAssignmentV1;
  viaFallbackSlots: DirectCreatureBaseSlotV1[];
}

export function createFallbackProposalV1(
  targetSlot: DirectCreatureBaseSlotV1,
  sourceSlot: DirectCreatureBaseSlotV1,
  reason: string,
): AnimationFallbackDecisionV1 {
  if (targetSlot === sourceSlot) {
    throw new RangeError("Fallback target and source slots must be different");
  }
  if (!reason.trim()) {
    throw new RangeError("Fallback proposal requires a reason");
  }
  return {
    id: `fallback:${targetSlot}:${sourceSlot}`,
    targetSlot,
    sourceSlot,
    reason: reason.trim(),
    review: "PENDING",
  };
}

export function approveFallbackV1(
  authoring: CreatureAnimationAuthoringV1,
  fallbackId: string,
): CreatureAnimationAuthoringV1 {
  const accepted = setFallbackReview(authoring, fallbackId, "ACCEPTED");
  if (detectFallbackCyclesV1(accepted.fallbacks).length > 0) {
    throw new RangeError("Accepted fallback would create an animation fallback cycle");
  }
  return accepted;
}

export function rejectFallbackV1(
  authoring: CreatureAnimationAuthoringV1,
  fallbackId: string,
): CreatureAnimationAuthoringV1 {
  return setFallbackReview(authoring, fallbackId, "REJECTED");
}

export function detectFallbackCyclesV1(
  fallbacks: readonly AnimationFallbackDecisionV1[],
): AnimationMappingDiagnosticV1[] {
  const edges = new Map<DirectCreatureBaseSlotV1, DirectCreatureBaseSlotV1>();
  fallbacks
    .filter(({ review }) => review !== "REJECTED")
    .forEach(({ targetSlot, sourceSlot }) => {
      if (!edges.has(targetSlot)) edges.set(targetSlot, sourceSlot);
    });
  const reported = new Set<string>();
  const diagnostics: AnimationMappingDiagnosticV1[] = [];

  edges.forEach((_source, start) => {
    const path: DirectCreatureBaseSlotV1[] = [];
    const positions = new Map<DirectCreatureBaseSlotV1, number>();
    let current: DirectCreatureBaseSlotV1 | undefined = start;
    while (current !== undefined) {
      const cycleStart = positions.get(current);
      if (cycleStart !== undefined) {
        const cycle = path.slice(cycleStart);
        const identity = canonicalCycleIdentity(cycle);
        if (!reported.has(identity)) {
          reported.add(identity);
          diagnostics.push({
            schemaVersion: 1,
            code: "M2A-ANIMATION-FALLBACK-CYCLE",
            path: `fallbacks.${cycle.join("->")}`,
            level: "BLOCKING",
            message: `Fallback cycle detected: ${[...cycle, current].join(" -> ")}.`,
            action: "Remove one fallback edge from this cycle.",
          });
        }
        break;
      }
      positions.set(current, path.length);
      path.push(current);
      current = edges.get(current);
    }
  });
  return diagnostics;
}

export function resolveEffectiveAnimationSourceV1(
  slot: DirectCreatureBaseSlotV1,
  assignments: readonly AnimationSourceAssignmentV1[],
  fallbacks: readonly AnimationFallbackDecisionV1[] = [],
): EffectiveAnimationSourceV1 {
  const assignmentsBySlot = new Map(
    assignments.map((assignment) => [assignment.targetSlot, assignment]),
  );
  const fallbacksByTarget = new Map(
    fallbacks
      .filter(({ review }) => review !== "REJECTED")
      .map((fallback) => [fallback.targetSlot, fallback]),
  );
  const visited = new Set<DirectCreatureBaseSlotV1>();
  const viaFallbackSlots: DirectCreatureBaseSlotV1[] = [];
  let current = slot;
  for (let depth = 0; depth <= 42; depth += 1) {
    const assignment = assignmentsBySlot.get(current);
    if (assignment) {
      return { resolvedSlot: current, assignment, viaFallbackSlots };
    }
    if (visited.has(current)) {
      throw new RangeError(`Fallback cycle while resolving ${slot}`);
    }
    visited.add(current);
    const fallback = fallbacksByTarget.get(current);
    if (!fallback) throw new RangeError(`No effective animation source for ${current}`);
    if (fallback.review !== "ACCEPTED") {
      throw new RangeError(`Fallback ${fallback.id} requires review`);
    }
    viaFallbackSlots.push(current);
    current = fallback.sourceSlot;
  }
  throw new RangeError(`Fallback chain for ${slot} exceeds the 42-slot limit`);
}

function setFallbackReview(
  authoring: CreatureAnimationAuthoringV1,
  fallbackId: string,
  review: "ACCEPTED" | "REJECTED",
): CreatureAnimationAuthoringV1 {
  const index = authoring.fallbacks.findIndex(({ id }) => id === fallbackId);
  if (index < 0) throw new RangeError(`Unknown animation fallback: ${fallbackId}`);
  if (authoring.fallbacks[index].review === review) return authoring;
  return {
    ...authoring,
    authoringRevision: authoring.authoringRevision + 1,
    fallbacks: authoring.fallbacks.map((fallback, fallbackIndex) => (
      fallbackIndex === index ? { ...fallback, review } : fallback
    )),
  };
}

function canonicalCycleIdentity(cycle: readonly DirectCreatureBaseSlotV1[]): string {
  if (cycle.length === 0) return "";
  return cycle
    .map((_slot, index) => [...cycle.slice(index), ...cycle.slice(0, index)].join(">"))
    .sort()[0];
}
