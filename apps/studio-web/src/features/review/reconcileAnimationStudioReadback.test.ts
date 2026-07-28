import { describe, expect, it } from "vitest";
import type { AnimationStudioDocumentV1 } from "../animation-studio/types";
import type { BinaryMdlInspectionReport, ReadbackAnimation } from "../preview/types";
import type {
  CanonicalAnimationStudioEvidenceV1,
} from "../results/projectCanonicalResult";
import {
  getAnimationStudioDownloadGateV1,
  reconcileAnimationStudioReadbackV1,
} from "./reconcileAnimationStudioReadback";

function studioFixture(): AnimationStudioDocumentV1 {
  return {
    schemaVersion: 1,
    sourceRevision: "sha256:source",
    authoringRevision: 4,
    status: "VALID",
    authoredClips: [{
      id: "clip-wave",
      name: "owned_wave",
      kind: "MOTION",
      status: "VALID",
      source: {
        kind: "SOURCE_CLIP_COPY",
        sourceRevision: "sha256:source",
        sourceClipName: "wave",
        sourceClipFingerprint: "sha256:wave",
        proceduralTemplate: null,
      },
      lengthSeconds: 1,
      transitionSeconds: 0.1,
      animationRoot: "root",
      events: [
        { id: "event-end", timeSeconds: 0.8, name: "end" },
        { id: "event-impact", timeSeconds: 0.5, name: "impact" },
      ],
      tracks: [{
        id: "track-position",
        targetNodeId: 7,
        path: "TRANSLATION",
        interpolation: "LINEAR",
        keyframes: [
          { id: "position-0", timeSeconds: 0, value: [0, 0, 0] },
          { id: "position-1", timeSeconds: 1, value: [0.25, 0, 0] },
        ],
      }, {
        id: "track-rotation",
        targetNodeId: 7,
        path: "ROTATION",
        interpolation: "LINEAR",
        keyframes: [
          { id: "rotation-0", timeSeconds: 0, value: [0, 0, 0, -1] },
          { id: "rotation-1", timeSeconds: 1, value: [0, -0.70710678118, 0, -0.70710678118] },
        ],
      }],
      revision: 3,
    }],
  };
}

function readbackAnimationFixture(): ReadbackAnimation {
  return {
    offset: 48,
    name: "owned_wave",
    length: 1.00000001,
    transition: 0.10000001,
    animationRoot: "root",
    events: [
      { time: 0.5, name: "impact" },
      { time: 0.8, name: "end" },
    ],
    nodeTree: {
      roots: [{
        offset: 64,
        number: 7,
        name: "hand",
        controllers: [{
          controllerName: "position",
          times: [0, 1],
          values: [[0, 0, 0], [0.25000001, 0, 0]],
        }, {
          controllerName: "orientation",
          times: [0, 1],
          values: [[0, 0, 0, 1], [0, 0.7071068, 0, 0.7071068]],
        }],
        children: [],
      }],
    },
  };
}

function readbackFixture(): BinaryMdlInspectionReport {
  return {
    schemaVersion: 1,
    format: "nwn1-binary-mdl",
    nodeTree: {
      roots: [{
        offset: 0,
        number: 7,
        name: "root",
        controllers: [],
        children: [],
      }],
    },
    animations: [readbackAnimationFixture()],
    diagnostics: [],
    validation: {
      status: "PASS",
      structure: {
        schemaVersion: 1,
        format: "nwn1-binary-mdl",
        rootNodeCount: 1,
        hasRootNodes: true,
        structuralErrors: [],
      },
      diagnostics: {
        total: 0,
        warnings: 0,
        errors: 0,
        informational: 0,
        unrecognizedSeverity: 0,
      },
    },
  };
}

describe("animation Studio binary readback reconciliation", () => {
  it("matches metadata, sorted events, target/path, times and canonical XYZW quaternions", () => {
    const result = reconcileAnimationStudioReadbackV1(
      studioFixture(),
      readbackFixture(),
    );

    expect(result).toEqual({
      schemaVersion: 1,
      status: "MATCH",
      checkedClipCount: 1,
      matchedClipIds: ["clip-wave"],
      diagnostics: [],
    });
    expect(getAnimationStudioDownloadGateV1(result)).toEqual({
      allowed: true,
      status: "READY",
      code: "M2A-ANIMATION-READBACK-MATCH",
      reason: "All valid authored animations match canonical binary MDL readback.",
    });
  });

  it("uses manifest stable-ID usage when an authored clip is renamed", () => {
    const studio = studioFixture();
    studio.authoredClips[0].name = "renamed_editor_label";
    const readback = readbackFixture();
    readback.animations[0].name = "custom_runtime_output";
    const sourceRevision = "a".repeat(64);
    const studioFingerprint = "b".repeat(64);
    const evidence: CanonicalAnimationStudioEvidenceV1 = {
      animationStudioSchemaVersion: 1,
      animationStudioFingerprintSha256: studioFingerprint,
      animationStudioRevision: studio.authoringRevision,
      creatureAnimationAuthoringSchemaVersion: 2,
      creatureAnimationAuthoringFingerprintSha256: "c".repeat(64),
      authoredClipCount: 1,
      authoredClipIds: ["clip-wave"],
      authoredClipOutputNames: ["renamed_editor_label"],
      authoredEventCount: 2,
      customAssignmentCount: 1,
      sourceRevision,
      readbackStatus: "MATCH",
      animationStudioReadback: {
        schemaVersion: 1,
        studioFingerprint,
        sourceRevision,
        status: "MATCH",
        clips: [{
          authoredClipId: "clip-wave",
          outputClipName: "custom_runtime_output",
          materializedFingerprint: "d".repeat(64),
        }],
        diagnostics: [],
      },
      sourceGlbUnchanged: true,
      authoredClips: [{
        id: "clip-wave",
        outputName: "renamed_editor_label",
        kind: "MOTION",
        status: "VALID",
        revision: 3,
        source: {
          kind: "SOURCE_CLIP_COPY",
          sourceRevision,
          sourceClipName: "wave",
          sourceClipFingerprint: null,
          proceduralTemplate: null,
        },
        keyframeCount: 4,
        eventCount: 2,
        usages: [{
          authoredClipId: "clip-wave",
          outputClipName: "custom_runtime_output",
          usageKind: "CUSTOM_ONE_SHOT",
          baseSlot: "ca1slashl",
          customAnimationId: "custom-wave",
          phase: null,
        }],
      }],
    };

    expect(reconcileAnimationStudioReadbackV1(
      studio,
      readback,
      evidence,
    )).toMatchObject({
      status: "MATCH",
      matchedClipIds: ["clip-wave"],
      diagnostics: [],
    });
  });

  it("reports exact metadata, event, time and value mismatch paths and blocks download", () => {
    const readback = readbackFixture();
    const animation = readback.animations[0];
    animation.length = 2;
    animation.events[0].name = "other";
    animation.nodeTree.roots[0].controllers[0].times[1] = 0.75;
    animation.nodeTree.roots[0].controllers[1].values[1] = [1, 0, 0, 0];

    const result = reconcileAnimationStudioReadbackV1(studioFixture(), readback);

    expect(result.status).toBe("MISMATCH");
    expect(result.matchedClipIds).toEqual([]);
    expect(result.diagnostics.map(({ code, path }) => ({ code, path }))).toEqual([
      {
        code: "M2A-ANIMATION-READBACK-LENGTH-MISMATCH",
        path: "authoredClips.clip-wave.lengthSeconds",
      },
      {
        code: "M2A-ANIMATION-READBACK-EVENTS-MISMATCH",
        path: "authoredClips.clip-wave.events",
      },
      {
        code: "M2A-ANIMATION-READBACK-TIMES-MISMATCH",
        path: "authoredClips.clip-wave.tracks.track-position.keyframes",
      },
      {
        code: "M2A-ANIMATION-READBACK-VALUES-MISMATCH",
        path: "authoredClips.clip-wave.tracks.track-rotation.keyframes",
      },
    ]);
    expect(getAnimationStudioDownloadGateV1(result)).toMatchObject({
      allowed: false,
      status: "BLOCKED",
      code: "M2A-ANIMATION-READBACK-MISMATCH",
    });
  });

  it.each(["DRAFT", "INVALID"] as const)(
    "fails closed when an authored clip remains %s",
    (status) => {
      const studio = studioFixture();
      studio.authoredClips[0].status = status;
      studio.status = status;
      const result = reconcileAnimationStudioReadbackV1(studio, readbackFixture());

      expect(result.status).toBe("MISMATCH");
      expect(result.diagnostics.map(({ code }) => code)).toEqual([
        "M2A-ANIMATION-READBACK-DOCUMENT-NOT-VALID",
        "M2A-ANIMATION-READBACK-CLIP-NOT-VALID",
      ]);
      expect(getAnimationStudioDownloadGateV1(result).allowed).toBe(false);
    },
  );

  it("fails closed for missing canonical validation and a missing authored clip", () => {
    const readback = readbackFixture();
    readback.validation = undefined;
    readback.animations = [];
    const result = reconcileAnimationStudioReadbackV1(studioFixture(), readback);

    expect(result.status).toBe("MISMATCH");
    expect(result.diagnostics.map(({ code }) => code)).toEqual([
      "M2A-ANIMATION-READBACK-VALIDATION-MISSING",
      "M2A-ANIMATION-READBACK-CLIP-MISSING",
    ]);
  });
});
