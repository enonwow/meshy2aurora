import { useCallback, useMemo } from "react";
import type * as THREE from "three";
import {
  buildAuroraReadbackAsset,
  type AuroraReadbackMaterialResolver,
} from "../preview/AuroraReadbackViewport";
import { animationClipsFromReadback } from "../preview/readbackAnimationPlayback";
import { SceneViewport, type SceneAnimationBindingV1 } from "../preview/SceneViewport";
import type { BinaryMdlInspectionReport, ReadbackNode } from "../preview/types";
import type { RigNodeDescriptorV1 } from "../preview/rigOverlay";

export function inheritedAnimationClipsV1(reports: readonly BinaryMdlInspectionReport[]) {
  const seen = new Set<string>();
  const result: THREE.AnimationClip[] = [];
  for (const report of reports) {
    for (const animation of report.animations) {
      const normalized = animation.name.trim().toLocaleLowerCase();
      if (!normalized || seen.has(normalized)) continue;
      const clips = animationClipsFromReadback({ ...report, animations: [animation] });
      if (clips.length === 0) continue;
      seen.add(normalized);
      result.push(...clips);
    }
  }
  return result;
}

function readbackNodeNames(roots: readonly ReadbackNode[]) {
  const result = new Set<string>();
  const visit = (node: ReadbackNode) => {
    const name = node.name.trim().toLocaleLowerCase();
    if (name) result.add(name);
    node.children.forEach(visit);
  };
  roots.forEach(visit);
  return result;
}

function controlledAnimationNodeNames(roots: readonly ReadbackNode[]) {
  const result = new Map<string, string>();
  const visit = (node: ReadbackNode) => {
    const normalized = node.name.trim().toLocaleLowerCase();
    const hasTransformController = node.controllers.some(({ controllerName, times, values }) => {
      const width = controllerName === "position" ? 3 : controllerName === "orientation" ? 4 : controllerName === "scale" ? 1 : 0;
      return width > 0
        && times.length > 0
        && times.length === values.length
        && times.every((time, index) => Number.isFinite(time) && time >= 0 && (index === 0 || time >= (times[index - 1] ?? 0)))
        && values.every((row) => row.length === width && row.every(Number.isFinite));
    });
    if (normalized && hasTransformController && !result.has(normalized)) result.set(normalized, node.name.trim());
    node.children.forEach(visit);
  };
  roots.forEach(visit);
  return result;
}

export function supermodelAnimationBindingReportV1(
  carrier: BinaryMdlInspectionReport,
  reports: readonly BinaryMdlInspectionReport[],
): SceneAnimationBindingV1 {
  const carrierNames = readbackNodeNames(carrier.nodeTree.roots);
  const seenClips = new Set<string>();
  const clips = reports.flatMap((report) => report.animations.flatMap((animation) => {
    const normalizedClip = animation.name.trim().toLocaleLowerCase();
    if (!normalizedClip || seenClips.has(normalizedClip) || !Number.isFinite(animation.length) || animation.length <= 0) return [];
    const controlled = controlledAnimationNodeNames(animation.nodeTree.roots);
    if (controlled.size === 0) return [];
    seenClips.add(normalizedClip);
    const unmatchedNodeNames = [...controlled]
      .filter(([normalized]) => !carrierNames.has(normalized))
      .map(([, original]) => original);
    return [{
      clipName: animation.name.trim(),
      animationSource: report.model?.name || animation.animationRoot || "unknown",
      controlledNodeCount: controlled.size,
      matchedNodeCount: controlled.size - unmatchedNodeNames.length,
      unmatchedNodeNames,
    }];
  }));
  return {
    schemaVersion: 1,
    rigSource: carrier.model?.name || carrier.nodeTree.roots[0]?.name || "unknown",
    clips,
  };
}

interface Props {
  reports: readonly BinaryMdlInspectionReport[];
  carrier?: BinaryMdlInspectionReport;
  detail: string;
  onError?: (message: string) => void;
  onSelectRigNode?: (node?: RigNodeDescriptorV1) => void;
  materialResolver?: AuroraReadbackMaterialResolver;
}

export function SupermodelPreviewViewport({ reports, carrier, detail, onError, onSelectRigNode, materialResolver }: Props) {
  const base = carrier ?? reports[0];
  const animationReports = useMemo(
    () => base ? [base, ...reports.filter((report) => report !== base)] : reports,
    [base, reports],
  );
  const animationBinding = useMemo(
    () => base ? supermodelAnimationBindingReportV1(base, animationReports) : undefined,
    [animationReports, base],
  );
  const buildRoot = useCallback(async () => {
    if (!base) throw new Error("Selected supermodel has no readable binary MDL resource");
    const asset = buildAuroraReadbackAsset(base, undefined, undefined, materialResolver);
    return { root: asset.root, animations: inheritedAnimationClipsV1(animationReports) };
  }, [animationReports, base, materialResolver]);
  const dependency = [
    carrier
      ? `${carrier.model?.name ?? "carrier"}:${carrier.byteLength ?? 0}:${carrier.animations.length}`
      : "native",
    ...reports.map((report) => `${report.model?.name ?? "model"}:${report.byteLength ?? 0}:${report.animations.length}`),
  ].join("|");

  return (
    <SceneViewport
      provenance="READBACK"
      detail={detail}
      dependency={dependency}
      buildRoot={buildRoot}
      onError={onError}
      onSelectRigNode={onSelectRigNode}
      animationBinding={animationBinding}
      tools={{ animationPlayback: true, overlays: true }}
    />
  );
}
