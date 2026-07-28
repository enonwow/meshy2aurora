import { describe, expect, it } from "vitest";
import type {
  CreatureAnimationAuthoringV1,
  SourceAnimationClipV1,
} from "./types";
import { getAuroraAnimationStateCatalogV1 } from "./catalog";
import {
  applyHighConfidenceAssignmentsV1,
  approveFallbackV1,
  createFallbackProposalV1,
  explainMappingProposalV1,
  proposeCreatureAnimationMappingV1,
  rankSourceClipCandidatesV1,
  rejectFallbackV1,
  resetAssignmentToProposalV1,
} from "./autoMap";

const clips: SourceAnimationClipV1[] = [
  {
    clipId: "walk",
    name: "Walking",
    durationSeconds: 1,
    trackCount: 2,
    targetNodeIds: [1],
    targetPaths: ["Hips.position"],
  },
  {
    clipId: "run",
    name: "CRUN",
    durationSeconds: 0.75,
    trackCount: 2,
    targetNodeIds: [1],
    targetPaths: ["Hips.position"],
  },
  {
    clipId: "attack",
    name: "Attack",
    durationSeconds: 0.8,
    trackCount: 2,
    targetNodeIds: [1],
    targetPaths: ["Arm.quaternion"],
  },
];

function emptyAuthoring(): CreatureAnimationAuthoringV1 {
  return {
    schemaVersion: 1,
    profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
    modelType: "S",
    sourceRevision: "sha256:owner-source",
    authoringRevision: 1,
    assignments: [],
    fallbacks: [],
    customAnimations: [],
  };
}

describe("creature animation auto-mapping v1", () => {
  it("ranks deterministically and auto-assigns only unambiguous high-confidence clips", () => {
    const walkState = getAuroraAnimationStateCatalogV1()
      .find(({ slot }) => slot === "cwalk")!;
    expect(rankSourceClipCandidatesV1(walkState, [...clips].reverse()))
      .toEqual(rankSourceClipCandidatesV1(walkState, clips));
    expect(rankSourceClipCandidatesV1(walkState, clips)[0]).toMatchObject({
      clipId: "walk",
      score: 0.98,
    });

    const proposal = proposeCreatureAnimationMappingV1(
      getAuroraAnimationStateCatalogV1(),
      clips,
    );
    expect(proposal.items.find(({ targetSlot }) => targetSlot === "cwalk"))
      .toMatchObject({ decision: "AUTO_ASSIGN", recommendedClipId: "walk" });
    expect(proposal.items.find(({ targetSlot }) => targetSlot === "crun"))
      .toMatchObject({ decision: "AUTO_ASSIGN", recommendedClipId: "run" });
    expect(proposal.items.find(({ targetSlot }) => targetSlot === "ca1slashl"))
      .toMatchObject({ decision: "REVIEW" });
    expect(proposal.items.find(({ targetSlot }) => targetSlot === "cpause1"))
      .toMatchObject({ decision: "UNRESOLVED", recommendedClipId: null });

    const applied = applyHighConfidenceAssignmentsV1(emptyAuthoring(), proposal);
    expect(applied.assignments.map(({ targetSlot }) => targetSlot)).toEqual([
      "cwalk",
      "crun",
    ]);
    expect(applied.authoringRevision).toBe(2);
    expect(applied.assignments).not.toContainEqual(
      expect.objectContaining({ targetSlot: "ca1slashl" }),
    );
  });

  it("creates explicit pending fallbacks and records accept or reject decisions", () => {
    const fallback = createFallbackProposalV1(
      "cdamager",
      "cdamagel",
      "Mirror the authored left damage response.",
    );
    expect(fallback).toMatchObject({
      targetSlot: "cdamager",
      sourceSlot: "cdamagel",
      review: "PENDING",
    });

    const pending = { ...emptyAuthoring(), fallbacks: [fallback] };
    expect(approveFallbackV1(pending, fallback.id)).toMatchObject({
      authoringRevision: 2,
      fallbacks: [expect.objectContaining({ review: "ACCEPTED" })],
    });
    expect(rejectFallbackV1(pending, fallback.id)).toMatchObject({
      authoringRevision: 2,
      fallbacks: [expect.objectContaining({ review: "REJECTED" })],
    });
  });

  it("can reset one edited assignment to its current proposal and explain the decision", () => {
    const proposal = proposeCreatureAnimationMappingV1(
      getAuroraAnimationStateCatalogV1(),
      clips,
    );
    const edited = emptyAuthoring();
    edited.assignments.push({
      targetSlot: "cwalk",
      sourceKind: "PROCEDURAL",
      sourceClipName: null,
      customAnimationId: null,
      provenance: {
        provider: "PROCEDURAL_GENERATOR",
        assetId: "generator-v1",
        ownership: "PROJECT_GENERATED",
      },
    });

    const reset = resetAssignmentToProposalV1(edited, "cwalk", proposal);
    expect(reset.assignments).toContainEqual(expect.objectContaining({
      targetSlot: "cwalk",
      sourceKind: "SOURCE_CLIP",
      sourceClipName: "Walking",
    }));
    expect(explainMappingProposalV1(
      proposal.items.find(({ targetSlot }) => targetSlot === "ca1slashl")!,
    )).toMatch(/review/i);
  });
});
