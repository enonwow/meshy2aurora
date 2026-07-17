import * as THREE from "three";
import type { BinaryMdlInspectionReport, ReadbackController, ReadbackNode } from "./types";

function finiteIncreasing(times: readonly number[]) {
  return times.length > 0
    && times.every((time, index) => Number.isFinite(time) && time >= 0 && (index === 0 || time >= (times[index - 1] ?? 0)));
}

function flatValues(controller: ReadbackController, columns: number) {
  if (controller.times.length !== controller.values.length || !finiteIncreasing(controller.times)) return undefined;
  if (!controller.values.every((row) => row.length === columns && row.every(Number.isFinite))) return undefined;
  return controller.values.flat();
}

function tracksForNode(node: ReadbackNode) {
  if (!node.name.trim()) return [];
  return node.controllers.flatMap((controller) => {
    const name = controller.controllerName;
    if (name === "position") {
      const values = flatValues(controller, 3);
      return values ? [new THREE.VectorKeyframeTrack(`${node.name}.position`, controller.times, values)] : [];
    }
    if (name === "orientation") {
      const values = flatValues(controller, 4);
      return values ? [new THREE.QuaternionKeyframeTrack(`${node.name}.quaternion`, controller.times, values)] : [];
    }
    if (name === "scale") {
      const values = flatValues(controller, 1);
      return values
        ? [new THREE.VectorKeyframeTrack(`${node.name}.scale`, controller.times, values.flatMap((value) => [value, value, value]))]
        : [];
    }
    return [];
  });
}

function tracksForNodes(nodes: readonly ReadbackNode[]): THREE.KeyframeTrack[] {
  return nodes.flatMap((node) => [...tracksForNode(node), ...tracksForNodes(node.children)]);
}

/**
 * Creates only tracks that are explicitly decoded by canonical MDL readback.
 * Unsupported or malformed controllers are intentionally omitted rather than
 * approximated in the browser.
 */
export function animationClipsFromReadback(report: BinaryMdlInspectionReport): THREE.AnimationClip[] {
  return report.animations.flatMap((animation) => {
    const tracks = tracksForNodes(animation.nodeTree.roots);
    return animation.name.trim() && Number.isFinite(animation.length) && animation.length > 0 && tracks.length > 0
      ? [new THREE.AnimationClip(animation.name, animation.length, tracks)]
      : [];
  });
}
