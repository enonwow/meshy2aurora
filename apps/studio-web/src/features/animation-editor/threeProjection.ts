import {
  AnimationClip,
  Matrix4,
  Object3D,
  Quaternion,
  QuaternionKeyframeTrack,
  VectorKeyframeTrack,
} from "three";
import type {
  AuthoredAnimationClipV1,
  AuthoredAnimationTrackPathV1,
} from "../animation-studio/types";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import type { SceneViewportTransformSnapshotV1 } from "../preview/SceneViewport";
import { sampleAnimationTrackLinearV1 } from "./editing";

export function projectAuthoredClipToThreeV1(
  clip: AuthoredAnimationClipV1,
  rig: readonly AnimationRigNodeV1[],
  sourceRoot?: Object3D,
): AnimationClip {
  const rigById = new Map(rig.map((node) => [node.id, node]));
  const tracks = clip.tracks.map((track) => {
    const rigNode = rigById.get(track.targetNodeId);
    const name = rigNode?.name
      ?? `node_${track.targetNodeId}`;
    const times = track.keyframes.map(({ timeSeconds }) => timeSeconds);
    const sourceNode = sourceRoot?.getObjectByName(name);
    const values = track.keyframes.flatMap(({ value }) => {
      if (!rigNode || !sourceNode) return value;
      if (track.path === "ROTATION") {
        const authored = new Quaternion(
          value[0]!,
          value[1]!,
          value[2]!,
          value[3]!,
        ).normalize();
        const preview = profileAH1BasisConjugateQuaternionV1(authored);
        return [preview.x, preview.y, preview.z, preview.w];
      }
      const targetBaseLength = Math.hypot(...rigNode.translation);
      const sourceBaseLength = sourceNode.position.length();
      const targetScale = targetBaseLength > 1e-8 && sourceBaseLength > 1e-8
        ? targetBaseLength / sourceBaseLength
        : 1;
      return [
        sourceNode.position.x
          + (value[0]! - rigNode.translation[0]) / targetScale,
        sourceNode.position.y
          + (value[2]! - rigNode.translation[2]) / targetScale,
        sourceNode.position.z
          + (value[1]! - rigNode.translation[1]) / targetScale,
      ];
    });
    return track.path === "ROTATION"
      ? new QuaternionKeyframeTrack(`${name}.quaternion`, times, values)
      : new VectorKeyframeTrack(`${name}.position`, times, values);
  });
  return new AnimationClip(clip.name, clip.lengthSeconds, tracks);
}

export function profileAH1BasisConjugateQuaternionV1(
  quaternion: Quaternion,
): Quaternion {
  const basis = new Matrix4().set(
    1, 0, 0, 0,
    0, 0, 1, 0,
    0, 1, 0, 0,
    0, 0, 0, 1,
  );
  const rotation = new Matrix4().makeRotationFromQuaternion(quaternion);
  const converted = basis.clone().multiply(rotation).multiply(basis);
  return new Quaternion().setFromRotationMatrix(converted).normalize();
}

export function projectThreeBoneTransformToAuthoredV1(
  snapshot: SceneViewportTransformSnapshotV1,
  rigNode: AnimationRigNodeV1,
  path: AuthoredAnimationTrackPathV1,
): readonly number[] {
  if (path === "ROTATION") {
    const source = new Quaternion(...snapshot.quaternion).normalize();
    const authored = profileAH1BasisConjugateQuaternionV1(source);
    return [authored.x, authored.y, authored.z, authored.w];
  }
  const sourceRestLength = Math.hypot(...snapshot.sourceRestPosition);
  const targetRestLength = Math.hypot(...rigNode.translation);
  const targetScale = sourceRestLength > 1e-8 && targetRestLength > 1e-8
    ? targetRestLength / sourceRestLength
    : 1;
  const delta = snapshot.position.map((value, axis) => (
    value - snapshot.sourceRestPosition[axis]!
  ));
  return [
    rigNode.translation[0] + delta[0]! * targetScale,
    rigNode.translation[1] + delta[2]! * targetScale,
    rigNode.translation[2] + delta[1]! * targetScale,
  ];
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
