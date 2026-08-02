import type { AuthoredAnimationClipV1 } from "../animation-studio/types";

export type AnimationLayerModeV1 = "BASE" | "ADDITIVE" | "OVERRIDE";

export interface AnimationEditLayerV1 {
  readonly schemaVersion: 1;
  readonly id: string;
  readonly stableOrder: number;
  readonly mode: AnimationLayerModeV1;
  readonly weight: number;
  readonly mute: boolean;
  readonly solo: boolean;
  readonly boneMask: { readonly schemaVersion: 1; readonly nodeIds: readonly number[] };
  readonly clip: AuthoredAnimationClipV1;
}

export interface AnimationLayerBakeReportV1 {
  readonly schemaVersion: 1;
  readonly activeLayerIds: readonly string[];
  readonly keyCountBefore: number;
  readonly keyCountAfter: number;
  readonly clip: AuthoredAnimationClipV1;
  readonly fingerprintSha256: string;
}

export interface AnimationStudioRigWireV1 {
  readonly schemaVersion: 1;
  readonly sourceRevision: string;
  readonly animationRoot: string;
  readonly nodes: readonly {
    readonly nodeId: number;
    readonly name: string;
    readonly parentId: number | null;
    readonly translation: readonly [number, number, number];
    readonly rotation: readonly [number, number, number, number];
  }[];
}

export function parseAnimationLayerBakeReportV1(json: string): AnimationLayerBakeReportV1 {
  const value = JSON.parse(json) as Partial<AnimationLayerBakeReportV1>;
  if (
    value.schemaVersion !== 1
    || !Array.isArray(value.activeLayerIds)
    || !value.activeLayerIds.every((id) => typeof id === "string" && id.length > 0)
    || !Number.isSafeInteger(value.keyCountBefore)
    || !Number.isSafeInteger(value.keyCountAfter)
    || value.clip === null
    || typeof value.clip !== "object"
    || typeof value.fingerprintSha256 !== "string"
    || !/^[0-9a-f]{64}$/iu.test(value.fingerprintSha256)
  ) {
    throw new Error("Core returned an invalid animation-layer bake report.");
  }
  return value as AnimationLayerBakeReportV1;
}
