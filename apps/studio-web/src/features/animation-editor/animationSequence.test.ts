import { describe, expect, it } from "vitest";
import { parseAnimationSequencePreviewV1 } from "./animationSequence";

describe("animation sequence preview parser", () => {
  it("accepts exact Core sequence identity and transition metrics", () => {
    const parsed = parseAnimationSequencePreviewV1(JSON.stringify({
      schemaVersion: 1,
      sourceRevision: "a".repeat(64),
      segments: [{ clipId: "idle", clipName: "idle", startSeconds: 0, endSeconds: 1 }, {
        clipId: "attack", clipName: "attack", startSeconds: 1, endSeconds: 2,
      }],
      transitionJumps: [{
        fromClipId: "idle",
        toClipId: "attack",
        boundarySeconds: 1,
        maxTranslationDelta: 0.2,
        maxTranslationNodeId: 3,
        maxAngularDeltaRadians: 0.4,
        maxAngularNodeId: 7,
      }],
      previewClip: { id: "sequence", name: "sequence" },
      fingerprintSha256: "b".repeat(64),
    }));

    expect(parsed.segments.map(({ clipId }) => clipId)).toEqual(["idle", "attack"]);
    expect(parsed.transitionJumps[0]?.maxAngularDeltaRadians).toBe(0.4);
  });

  it("rejects an unbound sequence report", () => {
    expect(() => parseAnimationSequencePreviewV1(JSON.stringify({
      schemaVersion: 1,
      sourceRevision: "stale",
      segments: [],
      transitionJumps: [],
      previewClip: {},
      fingerprintSha256: "b".repeat(64),
    }))).toThrow("invalid animation-sequence preview");
  });
});
