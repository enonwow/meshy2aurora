import { describe, expect, it } from "vitest";
import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import {
  ANIMATION_IMPORT_RIG_MISMATCH_V1,
  compareAnimationImportRigsV1,
  createImportedModelClipV1,
} from "./animationImport";

const donorRevision = "b".repeat(64);
const rig: AnimationRigNodeV1[] = [{
  id: 0,
  name: "root",
  parentId: null,
  translation: [0, 0, 0],
  rotation: [0, 0, 0, 1],
}, {
  id: 1,
  name: "torso",
  parentId: 0,
  translation: [0, 0, 1],
  rotation: [0, 0, 0, 1],
}];

const projected: AuthoredAnimationClipV1 = {
  id: "source-clip-walk",
  name: "walk",
  kind: "MOTION",
  status: "DRAFT",
  source: {
    kind: "SOURCE_CLIP_COPY",
    sourceRevision: donorRevision,
    sourceClipName: "walk",
    sourceClipFingerprint: "c".repeat(64),
    proceduralTemplate: null,
  },
  lengthSeconds: 1,
  transitionSeconds: 0.1,
  animationRoot: "root",
  tracks: [{
    id: "track-0-translation",
    targetNodeId: 0,
    path: "TRANSLATION",
    interpolation: "LINEAR",
    keyframes: [{
      id: "key-0000",
      timeSeconds: 0,
      value: [0, 0, 0],
    }, {
      id: "key-0001",
      timeSeconds: 1,
      value: [1, 0, 0],
    }],
  }],
  events: [],
  revision: 1,
};

describe("animation import from another model", () => {
  it("accepts an exact rig match including quaternion sign equivalence", () => {
    const donor = rig.map((node) => ({
      ...node,
      rotation: node.id === 1
        ? [0, 0, 0, -1] as const
        : node.rotation,
    }));

    expect(compareAnimationImportRigsV1(rig, donor)).toEqual(
      expect.objectContaining({
        compatible: true,
        code: null,
        mismatches: [],
      }),
    );
  });

  it("fails closed when the donor rest rig differs", () => {
    const donor = rig.map((node) => node.id === 1
      ? { ...node, translation: [0, 0, 1.25] as const }
      : node);

    expect(compareAnimationImportRigsV1(rig, donor)).toEqual(
      expect.objectContaining({
        compatible: false,
        code: ANIMATION_IMPORT_RIG_MISMATCH_V1,
        mismatches: ["Rest translation differs for torso."],
      }),
    );
  });

  it("rejects the same bone count when the donor hierarchy differs", () => {
    const donor = rig.map((node) => node.id === 1
      ? { ...node, parentId: null }
      : node);
    const compatibility = compareAnimationImportRigsV1(rig, donor);

    expect(donor).toHaveLength(rig.length);
    expect(compatibility).toEqual(expect.objectContaining({
      compatible: false,
      code: ANIMATION_IMPORT_RIG_MISMATCH_V1,
    }));
    expect(compatibility.mismatches).toContain(
      "Parent differs for torso: current 0, donor null.",
    );
  });

  it("materializes a self-contained Custom draft with donor provenance", () => {
    const imported = createImportedModelClipV1(projected, {
      id: "authored-imported-walk",
      name: "walk_imported",
      donorSourceRevision: donorRevision,
    });

    expect(imported).toEqual(expect.objectContaining({
      id: "authored-imported-walk",
      name: "walk_imported",
      status: "DRAFT",
      revision: 1,
      source: {
        kind: "IMPORTED_MODEL_COPY",
        sourceRevision: donorRevision,
        sourceClipName: "walk",
        sourceClipFingerprint: "c".repeat(64),
        proceduralTemplate: null,
      },
    }));
    expect(imported.tracks).toEqual(projected.tracks);
  });
});
