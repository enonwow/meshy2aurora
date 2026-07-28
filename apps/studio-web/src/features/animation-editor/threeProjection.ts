import {
  AnimationClip,
  Object3D,
  QuaternionKeyframeTrack,
  VectorKeyframeTrack,
} from "three";
import type {
  AuthoredAnimationClipV1,
  AuthoredAnimationTrackPathV1,
} from "../animation-studio/types";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import { sampleAnimationTrackLinearV1 } from "./editing";

export function projectAuthoredClipToThreeV1(
  clip: AuthoredAnimationClipV1,
  rig: readonly AnimationRigNodeV1[],
): AnimationClip {
  const names = new Map(rig.map((node) => [node.id, node.name]));
  const tracks = clip.tracks.map((track) => {
    const name = names.get(track.targetNodeId)
      ?? `node_${track.targetNodeId}`;
    const times = track.keyframes.map(({ timeSeconds }) => timeSeconds);
    const values = track.keyframes.flatMap(({ value }) => value);
    return track.path === "ROTATION"
      ? new QuaternionKeyframeTrack(`${name}.quaternion`, times, values)
      : new VectorKeyframeTrack(`${name}.position`, times, values);
  });
  return new AnimationClip(clip.name, clip.lengthSeconds, tracks);
}

export function applyEditorPoseAtTimeV1(
  root: Object3D,
  clip: AuthoredAnimationClipV1,
  rig: readonly AnimationRigNodeV1[],
  timeSeconds: number,
): void {
  const names = new Map(rig.map((node) => [node.id, node.name]));
  clip.tracks.forEach((track) => {
    if (track.keyframes.length === 0) return;
    const node = root.getObjectByName(
      names.get(track.targetNodeId) ?? `node_${track.targetNodeId}`,
    );
    if (!node) return;
    const value = sampleAnimationTrackLinearV1(track, timeSeconds);
    if (track.path === "ROTATION") {
      node.quaternion.set(value[0]!, value[1]!, value[2]!, value[3]!).normalize();
    } else {
      node.position.set(value[0]!, value[1]!, value[2]!);
    }
  });
}

export function selectAnimationEditorBoneV1(nodeId: number) {
  if (!Number.isSafeInteger(nodeId) || nodeId < 0) {
    throw new Error("Animation editor bone ID must be a non-negative integer.");
  }
  return { nodeId };
}

export interface BoneTransformGestureV1 {
  readonly nodeId: number;
  readonly path: AuthoredAnimationTrackPathV1;
  readonly initialValue: readonly number[];
  readonly previewValue: readonly number[];
}

export function beginBoneTransformGestureV1(input: {
  readonly nodeId: number;
  readonly path: AuthoredAnimationTrackPathV1;
  readonly value: readonly number[];
}): BoneTransformGestureV1 {
  return {
    nodeId: input.nodeId,
    path: input.path,
    initialValue: [...input.value],
    previewValue: [...input.value],
  };
}

export function updateBoneTransformGestureV1(
  gesture: BoneTransformGestureV1,
  value: readonly number[],
): BoneTransformGestureV1 {
  return { ...gesture, previewValue: [...value] };
}

export function commitBoneTransformGestureV1(
  gesture: BoneTransformGestureV1,
) {
  return {
    nodeId: gesture.nodeId,
    path: gesture.path,
    value: [...gesture.previewValue],
    changed: gesture.previewValue.some((value, index) => (
      value !== gesture.initialValue[index]
    )),
  };
}

export function cancelBoneTransformGestureV1(
  _gesture?: BoneTransformGestureV1,
): null {
  return null;
}

export function projectDopeSheetRowsV1(
  clip: AuthoredAnimationClipV1,
  rig: readonly AnimationRigNodeV1[],
) {
  const names = new Map(rig.map((node) => [node.id, node.name]));
  return clip.tracks.map((track) => ({
    id: track.id,
    targetNodeId: track.targetNodeId,
    label: names.get(track.targetNodeId) ?? `Node ${track.targetNodeId}`,
    path: track.path,
    keyframes: track.keyframes,
  }));
}
