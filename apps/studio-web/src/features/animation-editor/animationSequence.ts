import type { AuthoredAnimationClipV1 } from "../animation-studio/types";

export interface AnimationSequencePreviewRequestV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string;
  readonly sequenceId: string;
  readonly outputName: string;
  readonly clipIds: readonly string[];
}

export interface AnimationSequencePreviewV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string;
  readonly segments: readonly {
    readonly clipId: string;
    readonly clipName: string;
    readonly startSeconds: number;
    readonly endSeconds: number;
  }[];
  readonly transitionJumps: readonly {
    readonly fromClipId: string;
    readonly toClipId: string;
    readonly boundarySeconds: number;
    readonly maxTranslationDelta: number;
    readonly maxTranslationNodeId: number | null;
    readonly maxAngularDeltaRadians: number;
    readonly maxAngularNodeId: number | null;
  }[];
  readonly previewClip: AuthoredAnimationClipV1;
  readonly fingerprintSha256: string;
}

export function parseAnimationSequencePreviewV1(json: string): AnimationSequencePreviewV1 {
  const value = JSON.parse(json) as Partial<AnimationSequencePreviewV1>;
  const sha = /^[0-9a-f]{64}$/iu;
  if (
    value.schemaVersion !== 1
    || typeof value.sourceRevision !== "string"
    || !sha.test(value.sourceRevision)
    || !Array.isArray(value.segments)
    || value.segments.length < 2
    || !value.segments.every((segment) => (
      segment !== null
      && typeof segment === "object"
      && typeof segment.clipId === "string"
      && typeof segment.clipName === "string"
      && Number.isFinite(segment.startSeconds)
      && Number.isFinite(segment.endSeconds)
      && segment.endSeconds >= segment.startSeconds
    ))
    || !Array.isArray(value.transitionJumps)
    || !value.transitionJumps.every((jump) => (
      jump !== null
      && typeof jump === "object"
      && typeof jump.fromClipId === "string"
      && typeof jump.toClipId === "string"
      && Number.isFinite(jump.boundarySeconds)
      && Number.isFinite(jump.maxTranslationDelta)
      && Number.isFinite(jump.maxAngularDeltaRadians)
    ))
    || value.previewClip === null
    || typeof value.previewClip !== "object"
    || typeof value.fingerprintSha256 !== "string"
    || !sha.test(value.fingerprintSha256)
  ) {
    throw new Error("Core returned an invalid animation-sequence preview.");
  }
  return value as AnimationSequencePreviewV1;
}
