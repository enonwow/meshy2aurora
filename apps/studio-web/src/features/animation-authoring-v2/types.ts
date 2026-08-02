import type {
  AnimationStudioDocumentV1,
  AuthoredAnimationClipV1,
  AuthoredAnimationEventV1,
  AuthoredAnimationTrackPathV1,
} from "../animation-studio/types";

export type AnimationEditCommandV1 =
  | {
      kind: "SET_TRANSITION";
      clipId: string;
      transitionSeconds: number;
    }
  | {
      kind: "SET_BONE_KEY";
      clipId: string;
      trackId: string;
      keyId: string;
      targetNodeId: number;
      path: AuthoredAnimationTrackPathV1;
      timeSeconds: number;
      value: readonly number[];
    }
  | {
      kind: "MOVE_KEY";
      clipId: string;
      trackId: string;
      keyId: string;
      timeSeconds: number;
    }
  | {
      kind: "REMOVE_KEY";
      clipId: string;
      trackId: string;
      keyId: string;
    }
  | {
      kind: "TRIM_CLIP";
      clipId: string;
      startSeconds: number;
      endSeconds: number;
    }
  | {
      kind: "RETIME_CLIP";
      clipId: string;
      newLengthSeconds: number;
    }
  | { kind: "ADD_EVENT"; clipId: string; event: AuthoredAnimationEventV1 }
  | {
      kind: "UPDATE_EVENT";
      clipId: string;
      eventId: string;
      timeSeconds?: number;
      name?: string;
    }
  | { kind: "MOVE_EVENT"; clipId: string; eventId: string; timeSeconds: number }
  | { kind: "REMOVE_EVENT"; clipId: string; eventId: string };

export interface AnimationEditCommandBatchV1 {
  readonly schemaVersion: 1;
  readonly commandId: string;
  readonly context: {
    readonly sourceRevision: string;
    readonly documentAuthoringRevision: number;
  };
  readonly commands: readonly AnimationEditCommandV1[];
}

export interface AnimationEditCommandResultV1 {
  readonly schemaVersion: 1;
  readonly commandId: string;
  readonly commandFingerprintSha256: string;
  readonly documentFingerprintSha256: string;
  readonly document: AnimationStudioDocumentV1;
}

export type AnimationQualityIssueKindV1 =
  | "STATIC_PLAYBACK"
  | "LOOP_DISCONTINUITY"
  | "ROOT_DRIFT"
  | "GROUND_PENETRATION"
  | "FOOT_SLIDING"
  | "MOTION_SPIKE";

export interface AnimationQualityIssueV1 {
  readonly kind: AnimationQualityIssueKindV1;
  readonly severity: "INFO" | "WARNING" | "BLOCKING";
  readonly nodeId: number | null;
  readonly startSeconds: number;
  readonly endSeconds: number;
  readonly metric: number;
  readonly threshold: number;
  readonly message: string;
  readonly action: string;
}

export interface AnimationQualityPolicyV1 {
  readonly schemaVersion: 1;
  readonly sampleRateHz: number;
  readonly loopExpected: boolean;
  readonly staticTranslationEpsilonPerHeight: number;
  readonly staticAngularEpsilonRadians: number;
  readonly loopTranslationEpsilonPerHeight: number;
  readonly loopAngularEpsilonRadians: number;
  readonly rootDriftWarningPerHeight: number;
  readonly groundPenetrationWarningPerHeight: number;
  readonly contactHeightPerHeight: number;
  readonly footSlideWarningPerHeight: number;
  readonly maxTranslationSpeedPerHeight: number;
  readonly maxAngularSpeedRadians: number;
  readonly maxSamples: number;
}

export interface AnimationQualityContextV1 {
  readonly rootNodeId: number;
  readonly contactNodeIds: readonly number[];
  readonly groundAxis: number;
  readonly groundHeight: number;
}

export interface AnimationQualityReportV1 {
  readonly schemaVersion: 1;
  readonly clipId: string;
  readonly sampleCount: number;
  readonly skeletonHeight: number;
  readonly distinctPoseCount: number;
  readonly issues: readonly AnimationQualityIssueV1[];
  readonly policyFingerprintSha256: string;
  readonly reportFingerprintSha256: string;
}

export const DEFAULT_ANIMATION_QUALITY_POLICY_V1: AnimationQualityPolicyV1 = {
  schemaVersion: 1,
  sampleRateHz: 30,
  loopExpected: false,
  staticTranslationEpsilonPerHeight: 1e-5,
  staticAngularEpsilonRadians: 1e-4,
  loopTranslationEpsilonPerHeight: 0.01,
  loopAngularEpsilonRadians: 0.05,
  rootDriftWarningPerHeight: 0.1,
  groundPenetrationWarningPerHeight: 0.01,
  contactHeightPerHeight: 0.03,
  footSlideWarningPerHeight: 0.02,
  maxTranslationSpeedPerHeight: 8,
  maxAngularSpeedRadians: 20,
  maxSamples: 2_048,
};

export type ApplyAnimationEditCommandBatchV1 = (
  document: AnimationStudioDocumentV1,
  batch: AnimationEditCommandBatchV1,
) => Promise<AnimationEditCommandResultV1>;

export type AnalyzeAnimationMotionQualityV1 = (
  clip: AuthoredAnimationClipV1,
  policy: AnimationQualityPolicyV1,
  context: AnimationQualityContextV1,
) => Promise<AnimationQualityReportV1>;

export type AnimationAuthoringToolRequestV1 =
  | {
      kind: "BLEND_LOOP";
      policy: { schemaVersion: 1; blendWindowSeconds: number };
    }
  | {
      kind: "TRANSFORM_ROOT_MOTION";
      policy: { kind: "PRESERVE" | "IN_PLACE" } | { kind: "SCALE"; factor: number };
    }
  | {
      kind: "LOCK_SUGGESTED_CONTACTS";
      contactSet: {
        schemaVersion: 1;
        nodeIds: readonly number[];
        groundAxis: number;
        groundHeight: number;
        contactHeight: number;
        maxContactSpeed: number;
        sampleRateHz: number;
      };
    }
  | {
      kind: "REDUCE_KEYS";
      policy: {
        schemaVersion: 1;
        maxPositionError: number;
        maxAngularErrorRadians: number;
      };
    };

export interface AnimationAuthoringToolResultV1 {
  readonly kind:
    | "LOOP_BLENDED"
    | "ROOT_MOTION_TRANSFORMED"
    | "SUGGESTED_CONTACTS_LOCKED"
    | "KEYS_REDUCED";
  readonly result?: { readonly clip?: AuthoredAnimationClipV1 };
  readonly report?: { readonly clip?: AuthoredAnimationClipV1 };
  readonly clip?: AuthoredAnimationClipV1;
  readonly intervals?: readonly unknown[];
  readonly results?: readonly unknown[];
}

export type ApplyAnimationAuthoringToolV1 = (
  clip: AuthoredAnimationClipV1,
  request: AnimationAuthoringToolRequestV1,
) => Promise<AnimationAuthoringToolResultV1>;

export function parseAnimationAuthoringToolResultV1(
  value: unknown,
): AnimationAuthoringToolResultV1 {
  const record = object(value, "Animation authoring tool result");
  if (![
    "LOOP_BLENDED",
    "ROOT_MOTION_TRANSFORMED",
    "SUGGESTED_CONTACTS_LOCKED",
    "KEYS_REDUCED",
  ].includes(String(record.kind))) {
    throw new Error("Animation authoring tool result does not match V1.");
  }
  animationAuthoringToolResultClipV1(value as AnimationAuthoringToolResultV1);
  return value as AnimationAuthoringToolResultV1;
}

export function animationAuthoringToolResultClipV1(
  result: AnimationAuthoringToolResultV1,
): AuthoredAnimationClipV1 {
  const clip = result.kind === "KEYS_REDUCED"
    ? result.report?.clip
    : result.kind === "SUGGESTED_CONTACTS_LOCKED"
      ? result.clip
      : result.result?.clip;
  if (!clip || typeof clip !== "object") {
    throw new Error("Animation authoring tool result is missing its output clip.");
  }
  return clip;
}

export function parseAnimationEditCommandResultV1(
  value: unknown,
): AnimationEditCommandResultV1 {
  const record = object(value, "Animation edit command result");
  if (
    record.schemaVersion !== 1
    || typeof record.commandId !== "string"
    || !sha256(record.commandFingerprintSha256)
    || !sha256(record.documentFingerprintSha256)
    || typeof record.document !== "object"
    || record.document === null
  ) {
    throw new Error("Animation edit command result does not match V1.");
  }
  return value as AnimationEditCommandResultV1;
}

export function parseAnimationQualityReportV1(value: unknown): AnimationQualityReportV1 {
  const record = object(value, "Animation quality report");
  if (
    record.schemaVersion !== 1
    || typeof record.clipId !== "string"
    || !Number.isSafeInteger(record.sampleCount)
    || !finite(record.skeletonHeight)
    || !Number.isSafeInteger(record.distinctPoseCount)
    || !Array.isArray(record.issues)
    || !sha256(record.policyFingerprintSha256)
    || !sha256(record.reportFingerprintSha256)
  ) {
    throw new Error("Animation quality report does not match V1.");
  }
  record.issues.forEach((candidate, index) => {
    const issue = object(candidate, `Animation quality issue ${index}`);
    if (
      !["STATIC_PLAYBACK", "LOOP_DISCONTINUITY", "ROOT_DRIFT",
        "GROUND_PENETRATION", "FOOT_SLIDING", "MOTION_SPIKE"].includes(
          String(issue.kind),
        )
      || !["INFO", "WARNING", "BLOCKING"].includes(String(issue.severity))
      || !(issue.nodeId === null || Number.isSafeInteger(issue.nodeId))
      || !finite(issue.startSeconds)
      || !finite(issue.endSeconds)
      || !finite(issue.metric)
      || !finite(issue.threshold)
      || typeof issue.message !== "string"
      || typeof issue.action !== "string"
    ) {
      throw new Error(`Animation quality issue ${index} does not match V1.`);
    }
  });
  return value as AnimationQualityReportV1;
}

function object(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object.`);
  }
  return value as Record<string, unknown>;
}

function sha256(value: unknown) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function finite(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}
