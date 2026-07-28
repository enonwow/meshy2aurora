import { describe, expect, it } from "vitest";
import {
  commitAnimationStudioDocumentV1,
  commitAnimationStudioEditV1,
  createAnimationStudioStateV1,
  isAnimationStudioBuildCurrentV1,
  markAnimationStudioAutosaveFailedV1,
  markAnimationStudioAutosavedV1,
  markAnimationStudioBuildRevisionV1,
  reconcileAnimationStudioSourceRevisionV1,
  redoAnimationStudioEditV1,
  reduceAnimationStudioStateV1,
  undoAnimationStudioEditV1,
} from "./state";
import { animationStudioDocumentFixtureV1 } from "./testFixtures";

describe("Animation Studio session state V1", () => {
  it("handles mode, clip and bone selection without revising the document", () => {
    const initial = createAnimationStudioStateV1(animationStudioDocumentFixtureV1());
    const mode = reduceAnimationStudioStateV1(initial, {
      type: "ANIMATION_STUDIO_MODE_SELECTED",
      mode: "CREATE_EDIT",
    });
    const clip = reduceAnimationStudioStateV1(mode, {
      type: "AUTHORED_CLIP_SELECTED",
      clipId: "authored_attack_01",
    });
    const bone = reduceAnimationStudioStateV1(clip, {
      type: "BONE_SELECTED",
      boneId: 7,
    });

    expect(bone).toMatchObject({
      mode: "CREATE_EDIT",
      selectedClipId: "authored_attack_01",
      selectedBoneId: 7,
    });
    expect(bone.document.authoringRevision).toBe(1);
    expect(bone.undoStack).toEqual([]);
  });

  it("commits one edit revision and undo/redo restores exact JSON", () => {
    const initial = createAnimationStudioStateV1(animationStudioDocumentFixtureV1());
    const committed = commitAnimationStudioEditV1(initial, {
      type: "KEYFRAME_UPDATED",
      clipId: "authored_attack_01",
      trackId: "track-torso-rotation",
      keyframeId: "key-0",
      patch: { value: [0, 0.25, 0, 0.9682458] },
    });
    expect(committed.document.authoringRevision).toBe(2);
    expect(committed.document.authoredClips[0].revision).toBe(2);
    expect(committed.undoStack).toHaveLength(1);

    const undone = undoAnimationStudioEditV1(committed);
    expect(undone.document).toEqual(initial.document);
    const redone = redoAnimationStudioEditV1(undone);
    expect(redone.document).toEqual(committed.document);
  });

  it("does not revise a no-op command", () => {
    const initial = createAnimationStudioStateV1(animationStudioDocumentFixtureV1());
    const unchanged = commitAnimationStudioEditV1(initial, {
      type: "EVENT_UPDATED",
      clipId: "authored_attack_01",
      eventId: "event-impact",
      patch: { timeSeconds: 0.45 },
    });
    expect(unchanged).toBe(initial);
    expect(unchanged.undoStack).toEqual([]);
  });

  it("coalesces one gesture into one revision and one undo entry", () => {
    const initial = createAnimationStudioStateV1(animationStudioDocumentFixtureV1());
    const first = commitAnimationStudioEditV1(initial, {
      type: "KEYFRAME_UPDATED",
      clipId: "authored_attack_01",
      trackId: "track-torso-rotation",
      keyframeId: "key-0",
      patch: { value: [0, 0.1, 0, 0.995] },
      historyGroupId: "gizmo-rotation-7",
    });
    const last = commitAnimationStudioEditV1(first, {
      type: "KEYFRAME_UPDATED",
      clipId: "authored_attack_01",
      trackId: "track-torso-rotation",
      keyframeId: "key-0",
      patch: { value: [0, 0.5, 0, 0.866] },
      historyGroupId: "gizmo-rotation-7",
    });

    expect(last.document.authoringRevision).toBe(2);
    expect(last.document.authoredClips[0].revision).toBe(2);
    expect(last.undoStack).toHaveLength(1);
    expect(undoAnimationStudioEditV1(last).document).toEqual(initial.document);
  });

  it("requires confirmation before removing a referenced clip", () => {
    const initial = createAnimationStudioStateV1(animationStudioDocumentFixtureV1());
    const command = {
      type: "AUTHORED_CLIP_REMOVED" as const,
      clipId: "authored_attack_01",
      referencedByCustomAnimationIds: ["custom-attack"],
    };
    expect(() => commitAnimationStudioEditV1(initial, command))
      .toThrow(/requires confirmation/i);
    expect(commitAnimationStudioEditV1(initial, {
      ...command,
      confirmed: true,
    }).document.authoredClips).toEqual([]);
  });

  it("requires the exact autosaved revision for a current build", () => {
    const initial = createAnimationStudioStateV1(animationStudioDocumentFixtureV1());
    const built = markAnimationStudioBuildRevisionV1(initial, 1);
    expect(isAnimationStudioBuildCurrentV1(built)).toBe(false);
    const saved = markAnimationStudioAutosavedV1(
      built,
      1,
      "2026-07-28T12:00:00.000Z",
    );
    expect(isAnimationStudioBuildCurrentV1(saved)).toBe(true);

    const edited = commitAnimationStudioEditV1(saved, {
      type: "EVENT_UPDATED",
      clipId: "authored_attack_01",
      eventId: "event-impact",
      patch: { timeSeconds: 0.5 },
    });
    expect(isAnimationStudioBuildCurrentV1(edited)).toBe(false);
    expect(markAnimationStudioAutosaveFailedV1(edited, "quota").autosave)
      .toMatchObject({ status: "ERROR", error: "quota" });
  });

  it("marks a changed exact GLB revision as STALE_SOURCE", () => {
    const initial = createAnimationStudioStateV1(animationStudioDocumentFixtureV1());
    const stale = reconcileAnimationStudioSourceRevisionV1(
      initial,
      "b".repeat(64),
    );
    expect(stale.sourceStatus).toBe("STALE_SOURCE");
    expect(stale.document.authoringRevision).toBe(1);
    expect(reconcileAnimationStudioSourceRevisionV1(
      stale,
      "a".repeat(64),
    ).sourceStatus).toBe("CURRENT");
  });

  it("commits a complete trim/retime projection as one undo entry", () => {
    const initial = createAnimationStudioStateV1(animationStudioDocumentFixtureV1());
    const projected = structuredClone(initial.document);
    projected.authoredClips[0]!.lengthSeconds = 0.5;
    projected.authoredClips[0]!.events[0]!.timeSeconds = 0.25;
    const committed = commitAnimationStudioDocumentV1(initial, projected);
    expect(committed.document.authoringRevision).toBe(2);
    expect(committed.undoStack).toHaveLength(1);
    expect(undoAnimationStudioEditV1(committed).document).toEqual(initial.document);
  });
});
