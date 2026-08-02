import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import type { AnimationStudioRigWireV1 } from "./animationLayers";

export interface AnimationWorkbenchProjectArtifactV1 {
  readonly id: string;
  readonly name: string;
  readonly kind: "POSE" | "PHASE_TIMELINE" | "SEQUENCE" | "SEMANTIC_MAP" | "VARIANT_RECIPE";
  readonly sourceRevision: string;
  readonly fingerprintSha256: string;
  readonly payload: Record<string, unknown>;
}

export interface AnimationWorkbenchProjectStateV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string | null;
  readonly heldWeapon: {
    readonly source: {
      readonly name: string;
      readonly byteLength: number;
      readonly lastModified: number;
      readonly sha256: string;
    };
    readonly attachment: {
      readonly targetBoneName: string;
      readonly translation: readonly [number, number, number];
      readonly rotationEulerDegrees: readonly [number, number, number];
      readonly scale: readonly [number, number, number];
    };
  } | null;
  readonly posePresets: readonly AnimationWorkbenchProjectArtifactV1[];
  readonly phaseTimelines: readonly AnimationWorkbenchProjectArtifactV1[];
  readonly sequences: readonly AnimationWorkbenchProjectArtifactV1[];
  readonly semanticMaps: readonly AnimationWorkbenchProjectArtifactV1[];
  readonly variantRecipes: readonly AnimationWorkbenchProjectArtifactV1[];
}

export function emptyAnimationWorkbenchProjectStateV1(): AnimationWorkbenchProjectStateV1 {
  return {
    schemaVersion: 1,
    sourceRevision: null,
    heldWeapon: null,
    posePresets: [],
    phaseTimelines: [],
    sequences: [],
    semanticMaps: [],
    variantRecipes: [],
  };
}

export function parseAnimationWorkbenchProjectStateV1(
  value: unknown,
): AnimationWorkbenchProjectStateV1 {
  if (!isRecord(value)) throw new Error("$.animationWorkbench must be an object.");
  exactKeys(value, ["schemaVersion", "sourceRevision", "heldWeapon", "posePresets", "phaseTimelines", "sequences", "semanticMaps", "variantRecipes"]);
  if (value.schemaVersion !== 1 || !(value.sourceRevision === null || isSha(value.sourceRevision))) {
    throw new Error("$.animationWorkbench has an invalid schema or source revision.");
  }
  if (value.heldWeapon !== null) parseHeldWeaponProjectState(value.heldWeapon);
  for (const key of ["posePresets", "phaseTimelines", "sequences", "semanticMaps", "variantRecipes"] as const) {
    if (!Array.isArray(value[key]) || !value[key].every(isWorkbenchArtifact)) {
      throw new Error(`$.animationWorkbench.${key} contains an invalid artifact.`);
    }
  }
  return structuredClone(value) as unknown as AnimationWorkbenchProjectStateV1;
}

export type AnimationMirrorAxisV1 = "X" | "Y" | "Z";

export interface AnimationPoseSnapshotV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string;
  readonly rigSignatureSha256: string;
  readonly clipId: string;
  readonly clipRevision: number;
  readonly timeSeconds: number;
  readonly bones: readonly {
    readonly nodeId: number;
    readonly nodeName: string;
    readonly translation: readonly [number, number, number];
    readonly rotation: readonly [number, number, number, number];
  }[];
  readonly fingerprintSha256: string;
}

export interface AnimationPoseApplyResultV1 {
  readonly schemaVersion: 1;
  readonly mode: "WHOLE_BODY" | "SELECTED_BONES";
  readonly changedNodeIds: readonly number[];
  readonly clip: AuthoredAnimationClipV1;
  readonly fingerprintSha256: string;
}

export interface HumanoidMirrorMapV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string;
  readonly rigSignatureSha256: string;
  readonly axis: AnimationMirrorAxisV1;
  readonly pairs: readonly { readonly leftNodeId: number; readonly rightNodeId: number }[];
  readonly centerNodeIds: readonly number[];
  readonly fingerprintSha256: string;
}

export type CombatAttackPhaseKindV1 =
  | "READY" | "WIND_UP" | "STRIKE" | "IMPACT" | "RECOVERY";

export interface CombatAttackPhaseTimelineV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string;
  readonly clipId: string;
  readonly clipRevision: number;
  readonly markers: readonly {
    readonly phase: CombatAttackPhaseKindV1;
    readonly timeSeconds: number;
  }[];
  readonly fingerprintSha256: string;
}

export interface CombatQualityReportV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string;
  readonly clipId: string;
  readonly phaseFingerprintSha256: string;
  readonly skeletonHeight: number;
  readonly anticipationDisplacement: number;
  readonly attackArcLength: number;
  readonly peakSpeed: number;
  readonly peakSpeedTimeSeconds: number;
  readonly impactPeakOffsetSeconds: number;
  readonly directionReversals: number;
  readonly recoveryError: number;
  readonly rootDisplacement: number;
  readonly impactContactDisplacement: number;
  readonly bodyContributions: readonly {
    readonly nodeId: number;
    readonly displacement: number;
    readonly normalizedShare: number;
  }[];
  readonly issues: readonly {
    readonly kind: string;
    readonly severity: "INFO" | "WARNING" | "BLOCKING";
    readonly startSeconds: number;
    readonly endSeconds: number;
    readonly measured: number;
    readonly threshold: number;
    readonly message: string;
  }[];
  readonly fingerprintSha256: string;
}

export interface MotionVisualizationV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string;
  readonly clipId: string;
  readonly clipRevision: number;
  readonly trails: readonly {
    readonly nodeId: number;
    readonly points: readonly {
      readonly timeSeconds: number;
      readonly translation: readonly [number, number, number];
    }[];
  }[];
  readonly onionPoses: readonly {
    readonly offsetSamples: number;
    readonly timeSeconds: number;
    readonly pose: {
      readonly schemaVersion: 1;
      readonly timeSeconds: number;
      readonly bones: readonly {
        readonly nodeId: number;
        readonly translation: readonly [number, number, number];
        readonly rotation: readonly [number, number, number, number];
      }[];
      readonly fingerprintSha256: string;
    };
  }[];
  readonly fingerprintSha256: string;
}

export interface AnimationSequenceDocumentV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string;
  readonly sequenceId: string;
  readonly outputName: string;
  readonly segments: readonly AnimationSequenceSegmentRecipeV1[];
  readonly revision: number;
}

export interface AnimationSequenceSegmentRecipeV1 {
  readonly segmentId: string;
  readonly clipId: string;
  readonly sourceInSeconds: number;
  readonly sourceOutSeconds: number;
  readonly repeatCount: number;
  readonly transitionFromPrevious: {
    readonly kind: "CUT" | "CROSS_FADE";
    readonly durationSeconds: number;
  };
  readonly phaseMarkers: readonly {
    readonly kind: string;
    readonly timeSeconds: number;
  }[];
}

export interface AnimationSequenceBakeV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string;
  readonly documentRevision: number;
  readonly placedSegments: readonly {
    readonly segmentId: string;
    readonly clipId: string;
    readonly startSeconds: number;
    readonly endSeconds: number;
    readonly sourceInSeconds: number;
    readonly sourceOutSeconds: number;
    readonly repeatCount: number;
    readonly transitionFromPrevious: {
      readonly kind: "CUT" | "CROSS_FADE";
      readonly durationSeconds: number;
    };
  }[];
  readonly phaseMarkers: readonly {
    readonly segmentId: string;
    readonly kind: string;
    readonly timeSeconds: number;
  }[];
  readonly sourceProvenance: readonly {
    readonly clipId: string;
    readonly clipRevision: number;
    readonly clipFingerprintSha256: string;
  }[];
  readonly previewClip: AuthoredAnimationClipV1;
  readonly customClip: AuthoredAnimationClipV1;
  readonly fingerprintSha256: string;
}

export type AnimationWorkbenchRequestV1 =
  | { readonly operation: "FINGERPRINT_CLIP"; readonly clip: AuthoredAnimationClipV1 }
  | { readonly operation: "CAPTURE_POSE"; readonly clip: AuthoredAnimationClipV1; readonly rig: AnimationStudioRigWireV1; readonly timeSeconds: number; readonly selectedNodeIds: readonly number[] }
  | { readonly operation: "BUILD_MIRROR_MAP"; readonly rig: AnimationStudioRigWireV1; readonly axis: AnimationMirrorAxisV1; readonly pairs: readonly { readonly leftNodeId: number; readonly rightNodeId: number }[]; readonly centerNodeIds: readonly number[] }
  | { readonly operation: "MIRROR_POSE"; readonly pose: AnimationPoseSnapshotV1; readonly rig: AnimationStudioRigWireV1; readonly mapping: HumanoidMirrorMapV1 }
  | { readonly operation: "APPLY_POSE"; readonly clip: AuthoredAnimationClipV1; readonly rig: AnimationStudioRigWireV1; readonly pose: AnimationPoseSnapshotV1; readonly timeSeconds: number; readonly mode: "WHOLE_BODY" | "SELECTED_BONES"; readonly selectedNodeIds: readonly number[] }
  | { readonly operation: "RESET_POSE"; readonly clip: AuthoredAnimationClipV1; readonly rig: AnimationStudioRigWireV1; readonly timeSeconds: number; readonly selectedNodeIds: readonly number[] }
  | { readonly operation: "CREATE_COMBAT_PHASES"; readonly clip: AuthoredAnimationClipV1; readonly phaseTimes: readonly [number, number, number, number, number] }
  | { readonly operation: "RETIME_COMBAT_PHASES"; readonly clip: AuthoredAnimationClipV1; readonly timeline: CombatAttackPhaseTimelineV1; readonly target: CombatAttackPhaseTimelineV1 }
  | { readonly operation: "ANALYZE_COMBAT_QUALITY"; readonly clip: AuthoredAnimationClipV1; readonly rig: AnimationStudioRigWireV1; readonly timeline: CombatAttackPhaseTimelineV1; readonly policy: Record<string, unknown>; readonly context: Record<string, unknown> }
  | { readonly operation: "BUILD_MOTION_VISUALIZATION"; readonly clip: AuthoredAnimationClipV1; readonly rig: AnimationStudioRigWireV1; readonly request: Record<string, unknown> }
  | { readonly operation: "CREATE_VARIANT"; readonly source: AuthoredAnimationClipV1; readonly rig: AnimationStudioRigWireV1; readonly recipe: Record<string, unknown> }
  | { readonly operation: "BAKE_SEQUENCE"; readonly document: AnimationSequenceDocumentV1; readonly availableClips: readonly AuthoredAnimationClipV1[] };

export type AnimationWorkbenchResultV1 =
  | { readonly result: "CLIP_FINGERPRINTED"; readonly value: string }
  | { readonly result: "POSE_CAPTURED"; readonly value: AnimationPoseSnapshotV1 }
  | { readonly result: "MIRROR_MAP_BUILT"; readonly value: HumanoidMirrorMapV1 }
  | { readonly result: "POSE_MIRRORED"; readonly value: AnimationPoseSnapshotV1 }
  | { readonly result: "POSE_APPLIED" | "POSE_RESET"; readonly value: AnimationPoseApplyResultV1 }
  | { readonly result: "COMBAT_PHASES_CREATED"; readonly value: CombatAttackPhaseTimelineV1 }
  | { readonly result: "COMBAT_PHASES_RETIMED"; readonly value: AuthoredAnimationClipV1 }
  | { readonly result: "COMBAT_QUALITY_ANALYZED"; readonly value: CombatQualityReportV1 }
  | { readonly result: "MOTION_VISUALIZATION_BUILT"; readonly value: MotionVisualizationV1 }
  | { readonly result: "VARIANT_CREATED"; readonly value: { readonly clip: AuthoredAnimationClipV1; readonly effectiveTags: readonly string[]; readonly fingerprintSha256: string } }
  | { readonly result: "SEQUENCE_BAKED"; readonly value: AnimationSequenceBakeV1 };

export function parseAnimationWorkbenchResultV1(json: string): AnimationWorkbenchResultV1 {
  const value = JSON.parse(json) as Partial<AnimationWorkbenchResultV1>;
  if (typeof value.result !== "string" || value.value === null || typeof value.value !== "object") {
    throw new Error("Core returned an invalid animation workbench result.");
  }
  return value as AnimationWorkbenchResultV1;
}

export function rigWireV1(
  sourceRevision: string,
  animationRoot: string,
  rig: readonly { readonly id: number; readonly name: string; readonly parentId: number | null; readonly translation: readonly [number, number, number]; readonly rotation: readonly [number, number, number, number] }[],
): AnimationStudioRigWireV1 {
  return {
    schemaVersion: 1,
    sourceRevision,
    animationRoot,
    nodes: rig.map((node) => ({
      nodeId: node.id,
      name: node.name,
      parentId: node.parentId,
      translation: node.translation,
      rotation: node.rotation,
    })),
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
function isSha(value: unknown): value is string {
  return typeof value === "string" && /^[0-9a-f]{64}$/iu.test(value);
}
function exactKeys(value: Record<string, unknown>, keys: readonly string[]) {
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) {
    throw new Error("Animation Workbench object has missing or unknown fields.");
  }
}
function isWorkbenchArtifact(value: unknown): value is AnimationWorkbenchProjectArtifactV1 {
  if (!isRecord(value)) return false;
  try { exactKeys(value, ["id", "name", "kind", "sourceRevision", "fingerprintSha256", "payload"]); } catch { return false; }
  return typeof value.id === "string" && value.id.length > 0
    && typeof value.name === "string" && value.name.length > 0
    && ["POSE", "PHASE_TIMELINE", "SEQUENCE", "SEMANTIC_MAP", "VARIANT_RECIPE"].includes(String(value.kind))
    && isSha(value.sourceRevision) && isSha(value.fingerprintSha256) && isRecord(value.payload);
}
function parseHeldWeaponProjectState(value: unknown) {
  if (!isRecord(value)) throw new Error("$.animationWorkbench.heldWeapon must be an object.");
  exactKeys(value, ["source", "attachment"]);
  if (!isRecord(value.source) || !isRecord(value.attachment)) throw new Error("Held weapon project state is invalid.");
  exactKeys(value.source, ["name", "byteLength", "lastModified", "sha256"]);
  exactKeys(value.attachment, ["targetBoneName", "translation", "rotationEulerDegrees", "scale"]);
  const tuple = (candidate: unknown) => Array.isArray(candidate) && candidate.length === 3 && candidate.every((item) => typeof item === "number" && Number.isFinite(item));
  if (typeof value.source.name !== "string" || !Number.isSafeInteger(value.source.byteLength) || !Number.isSafeInteger(value.source.lastModified) || !isSha(value.source.sha256)
    || typeof value.attachment.targetBoneName !== "string" || !tuple(value.attachment.translation) || !tuple(value.attachment.rotationEulerDegrees) || !tuple(value.attachment.scale)) {
    throw new Error("Held weapon identity or attachment is invalid.");
  }
}
