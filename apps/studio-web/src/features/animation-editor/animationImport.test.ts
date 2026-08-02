import { describe, expect, it } from "vitest";
import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import {
  createImportedModelClipV1,
  parseAnimationTransferCompatibilityV1,
  parseHumanoidSemanticBoneMapV2,
} from "./animationImport";

const donorRevision = "b".repeat(64);
const targetRevision = "a".repeat(64);

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
    keyframes: [{ id: "key-0000", timeSeconds: 0, value: [0, 0, 0] }, {
      id: "key-0001",
      timeSeconds: 1,
      value: [1, 0, 0],
    }],
  }],
  events: [],
  revision: 1,
};

describe("animation import from another model", () => {
  it("projects Core compatibility without maintaining a second rig algorithm", () => {
    const compatibility = parseAnimationTransferCompatibilityV1(JSON.stringify({
      schemaVersion: 1,
      status: "RETARGETABLE_SAME_HIERARCHY",
      donorSourceRevision: donorRevision,
      targetSourceRevision: targetRevision,
      donorRigSignatureSha256: "1".repeat(64),
      targetRigSignatureSha256: "2".repeat(64),
      compatibilityFingerprintSha256: "3".repeat(64),
      allowedModes: ["SAME_HIERARCHY_RETARGET_V1"],
      mapping: {
        schemaVersion: 1,
        rootName: "root",
        entries: [{
          boneName: "root",
          parentName: null,
          donorNodeId: 7,
          targetNodeId: 0,
        }],
      },
      diagnostics: [{
        code: "M2A-ANIMATION-RETARGET-REST-TRANSLATION",
        path: "rig.nodes[root].translation",
        message: "rest translation differs for root",
        action: "Preview the explicit retarget mode.",
      }],
    }));

    expect(compatibility.status).toBe("RETARGETABLE_SAME_HIERARCHY");
    expect(compatibility.allowedModes).toEqual(["SAME_HIERARCHY_RETARGET_V1"]);
    expect(compatibility.mapping?.entries[0]).toEqual(expect.objectContaining({
      donorNodeId: 7,
      targetNodeId: 0,
    }));
  });

  it("rejects an unbound Core compatibility response", () => {
    expect(() => parseAnimationTransferCompatibilityV1(JSON.stringify({
      schemaVersion: 1,
      status: "RETARGETABLE_SAME_HIERARCHY",
      donorSourceRevision: "not-a-sha",
      targetSourceRevision: targetRevision,
      compatibilityFingerprintSha256: "3".repeat(64),
      allowedModes: ["AUTO"],
      diagnostics: [],
    }))).toThrow("invalid animation-transfer compatibility report");
  });

  it("accepts a source-bound humanoid semantic V2 map", () => {
    const semanticMap = parseHumanoidSemanticBoneMapV2(JSON.stringify({
      schemaVersion: 2,
      aliasDictionaryVersion: "M2A_HUMANOID_ALIASES_2026_07_V1",
      status: "COMPATIBLE",
      donorSourceRevision: donorRevision,
      targetSourceRevision: targetRevision,
      manualMappingConfirmed: false,
      entries: [{
        semantic: "RIGHT_HAND",
        required: true,
        donorNodeId: 17,
        donorNodeName: "mixamorig_RightHand",
        targetNodeId: 42,
        targetNodeName: "hand_r",
        mappingSource: "VERSIONED_ALIAS",
      }],
      diagnostics: [],
      fingerprintSha256: "4".repeat(64),
    }));

    expect(semanticMap.status).toBe("COMPATIBLE");
    expect(semanticMap.entries[0]).toEqual(expect.objectContaining({
      semantic: "RIGHT_HAND",
      donorNodeId: 17,
      targetNodeId: 42,
    }));
  });

  it("rejects a semantic map without exact donor lineage", () => {
    expect(() => parseHumanoidSemanticBoneMapV2(JSON.stringify({
      schemaVersion: 2,
      aliasDictionaryVersion: "M2A_HUMANOID_ALIASES_2026_07_V1",
      status: "COMPATIBLE",
      donorSourceRevision: "stale",
      targetSourceRevision: targetRevision,
      manualMappingConfirmed: false,
      entries: [],
      diagnostics: [],
      fingerprintSha256: "4".repeat(64),
    }))).toThrow("invalid humanoid semantic-retarget report");
  });

  it("materializes a self-contained exact-copy Custom draft with donor provenance", () => {
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
      source: expect.objectContaining({
        kind: "IMPORTED_MODEL_COPY",
        sourceRevision: donorRevision,
        sourceClipName: "walk",
        sourceClipFingerprint: "c".repeat(64),
      }),
    }));
    expect(imported.tracks).toEqual(projected.tracks);
  });
});
