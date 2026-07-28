import { describe, expect, it } from "vitest";
import type { BinaryMdlInspectionReport } from "../preview/types";
import { createCreatureAnimationAuthoringV1 } from "./state";
import type {
  AnimationCatalogRowV1,
  AnimationSourceAssignmentV1,
  SourceAnimationClipV1,
} from "./types";
import {
  compareAnimationPreviewTimingsV1,
  createAnimationPreviewSelectionV1,
  getAnimationPreviewDiagnosticsV1,
  loadReadbackAnimationPreviewV1,
  loadSourceAnimationPreviewV1,
} from "./preview";

const row: AnimationCatalogRowV1 = {
  id: "state:idle",
  kind: "BASE",
  stateId: "IDLE",
  label: "Idle",
  description: "Idle",
  slot: "cpause1",
  modelType: "S",
  playbackPolicy: "ENGINE_MANAGED",
  sourceLabel: "Idle Source",
  provenance: null,
  status: "MAPPED",
  diagnosticCodes: [],
};

const assignment: AnimationSourceAssignmentV1 = {
  targetSlot: "cpause1",
  sourceKind: "SOURCE_CLIP",
  sourceClipName: "Idle Source",
  customAnimationId: null,
  provenance: {
    provider: "SOURCE_GLB",
    assetId: "sha256:source",
    ownership: "USER_OWNED",
  },
};

const sourceClip: SourceAnimationClipV1 = {
  clipId: "clip:idle",
  name: "Idle Source",
  durationSeconds: 1.25,
  trackCount: 2,
  targetNodeIds: [1],
  targetPaths: ["root.position"],
};

const readback: BinaryMdlInspectionReport = {
  schemaVersion: 1,
  format: "NWN1_BINARY_MDL",
  nodeTree: { roots: [] },
  animations: [{
    offset: 1,
    name: "cpause1",
    length: 1.5,
    transition: 0.2,
    animationRoot: "root",
    events: [],
    nodeTree: { roots: [] },
  }],
  diagnostics: [],
};

describe("animation preview projection", () => {
  it("resolves a selected base row to source and result clip names", () => {
    const authoring = {
      ...createCreatureAnimationAuthoringV1("sha256:source", "S"),
      assignments: [assignment],
    };

    const selection = createAnimationPreviewSelectionV1(row, authoring);
    expect(selection).toMatchObject({
      targetSlot: "cpause1",
      sourceClipName: "Idle Source",
      resultClipName: "cpause1",
      viaFallbackSlots: [],
    });
    expect(loadSourceAnimationPreviewV1(selection, [sourceClip])).toEqual(sourceClip);
    expect(loadReadbackAnimationPreviewV1(readback, selection.resultClipName)?.name).toBe("cpause1");
  });

  it("reports exact timing delta without inventing unavailable event counts", () => {
    expect(compareAnimationPreviewTimingsV1(sourceClip, readback.animations[0])).toEqual({
      sourceDurationSeconds: 1.25,
      resultDurationSeconds: 1.5,
      durationDeltaSeconds: 0.25,
      durationDeltaRatio: 0.2,
      resultTransitionSeconds: 0.2,
      sourceEventCount: null,
      resultEventCount: null,
    });
  });

  it("distinguishes unresolved source, missing readback and stale readback", () => {
    const authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
    const selection = createAnimationPreviewSelectionV1(row, authoring);
    expect(getAnimationPreviewDiagnosticsV1(selection).map(({ code }) => code)).toEqual([
      "M2A-ANIMATION-PREVIEW-SOURCE-UNRESOLVED",
      "M2A-ANIMATION-PREVIEW-READBACK-MISSING",
    ]);
    expect(getAnimationPreviewDiagnosticsV1(selection, {
      builtAuthoringRevision: 1,
      currentAuthoringRevision: 2,
    }).map(({ code }) => code)).toContain("M2A-ANIMATION-PREVIEW-READBACK-STALE");
  });
});
