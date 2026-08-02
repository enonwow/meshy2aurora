import type { AuthoredAnimationTrackV1 } from "../animation-studio/types";

export interface AnimationEditorCurveTrackV1 {
  readonly schemaVersion: 1;
  readonly id: string;
  readonly targetNodeId: number;
  readonly path: "TRANSLATION" | "ROTATION";
  readonly keys: readonly {
    readonly id: string;
    readonly timeSeconds: number;
    readonly value: readonly number[];
    readonly inTangent: readonly number[] | null;
    readonly outTangent: readonly number[] | null;
  }[];
}

export interface AnimationCurveResamplePolicyV1 {
  readonly schemaVersion: 1;
  readonly maxPositionError: number;
  readonly maxAngularErrorRadians: number;
  readonly maxSubdivisionDepth: number;
}

export interface AnimationCurveResampleReportV1 {
  readonly schemaVersion: 1;
  readonly editorKeyCount: number;
  readonly linearKeyCount: number;
  readonly maxPositionError: number;
  readonly maxAngularErrorRadians: number;
  readonly track: AuthoredAnimationTrackV1;
  readonly fingerprintSha256: string;
}

export function createAutoCurveFromTrackV1(
  track: AuthoredAnimationTrackV1,
): AnimationEditorCurveTrackV1 {
  return {
    schemaVersion: 1,
    id: track.id,
    targetNodeId: track.targetNodeId,
    path: track.path,
    keys: track.keyframes.map((key, index, keys) => {
      if (track.path === "ROTATION") {
        return {
          id: key.id,
          timeSeconds: key.timeSeconds,
          value: key.value,
          inTangent: null,
          outTangent: null,
        };
      }
      const previous = keys[Math.max(0, index - 1)]!;
      const next = keys[Math.min(keys.length - 1, index + 1)]!;
      const duration = next.timeSeconds - previous.timeSeconds;
      const tangent = duration <= 1e-8
        ? [0, 0, 0]
        : [0, 1, 2].map((axis) => (
            ((next.value[axis] ?? 0) - (previous.value[axis] ?? 0)) / duration
          ));
      return {
        id: key.id,
        timeSeconds: key.timeSeconds,
        value: key.value,
        inTangent: tangent,
        outTangent: tangent,
      };
    }),
  };
}

export function parseAnimationCurveResampleReportV1(
  json: string,
): AnimationCurveResampleReportV1 {
  const value = JSON.parse(json) as Partial<AnimationCurveResampleReportV1>;
  if (
    value.schemaVersion !== 1
    || !Number.isSafeInteger(value.editorKeyCount)
    || !Number.isSafeInteger(value.linearKeyCount)
    || !Number.isFinite(value.maxPositionError)
    || !Number.isFinite(value.maxAngularErrorRadians)
    || value.track === null
    || typeof value.track !== "object"
    || typeof value.fingerprintSha256 !== "string"
    || !/^[0-9a-f]{64}$/iu.test(value.fingerprintSha256)
  ) {
    throw new Error("Core returned an invalid curve-resample report.");
  }
  return value as AnimationCurveResampleReportV1;
}
