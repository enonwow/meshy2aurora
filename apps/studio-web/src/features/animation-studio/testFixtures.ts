import type {
  AnimationStudioDocumentV1,
  AuthoredAnimationClipV1,
} from "./types";

export function animationStudioClipFixtureV1(
  patch: Partial<AuthoredAnimationClipV1> = {},
): AuthoredAnimationClipV1 {
  return {
    id: "authored_attack_01",
    name: "custom_attack_overhead",
    kind: "MOTION",
    status: "DRAFT",
    source: {
      kind: "BLANK_POSE",
      sourceRevision: "a".repeat(64),
      sourceClipName: null,
      sourceClipFingerprint: null,
      proceduralTemplate: null,
    },
    lengthSeconds: 0.9,
    transitionSeconds: 0.1,
    animationRoot: "torso",
    tracks: [{
      id: "track-torso-rotation",
      targetNodeId: 7,
      path: "ROTATION",
      interpolation: "LINEAR",
      keyframes: [{
        id: "key-0",
        timeSeconds: 0,
        value: [0, 0, 0, 1],
      }],
    }],
    events: [{
      id: "event-impact",
      timeSeconds: 0.45,
      name: "impact",
    }],
    revision: 1,
    ...patch,
  };
}

export function animationStudioDocumentFixtureV1(
  patch: Partial<AnimationStudioDocumentV1> = {},
): AnimationStudioDocumentV1 {
  return {
    schemaVersion: 1,
    sourceRevision: "a".repeat(64),
    authoringRevision: 1,
    status: "DRAFT",
    authoredClips: [animationStudioClipFixtureV1()],
    ...patch,
  };
}
