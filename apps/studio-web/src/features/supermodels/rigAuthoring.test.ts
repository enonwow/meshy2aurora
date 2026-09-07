import { describe, expect, it } from "vitest";
import {
  composeJointMatrixV1,
  decomposeJointMatrixV1,
  parseReferenceSupermodelRigAuthoringV1,
  parseReferenceSupermodelRigAuthoringV2,
  parseReferenceSupermodelTargetRigV1,
  upsertComponentBindingV2,
  upsertJointOverrideV1,
  upsertLandmarkOverrideV2,
  upsertRegionWeightConstraintV2,
  type ReferenceSupermodelRigAuthoringDocumentV1,
  type ReferenceSupermodelRigAuthoringDocumentV2,
} from "./rigAuthoring";

const hash = (character: string) => character.repeat(64);

function document(): ReferenceSupermodelRigAuthoringDocumentV1 {
  return {
    schemaVersion: 1,
    sourceSha256: hash("a"),
    sourceForward: "POSITIVE_Z",
    selectedSupermodelResref: "c_wolf",
    exactChainSha256: hash("b"),
    motionContractSha256: hash("c"),
    baseRigSha256: hash("d"),
    jointOverrides: [],
    weightOverrides: [],
    contentSha256: hash("e"),
  };
}

function documentV2(): ReferenceSupermodelRigAuthoringDocumentV2 {
  return {
    schemaVersion: 2,
    sourceSha256: hash("a"),
    sourceForward: "POSITIVE_Z",
    selectedSupermodelResref: "c_wolf",
    exactChainSha256: hash("b"),
    motionContractSha256: hash("c"),
    structuralProfileSha256: hash("d"),
    surfaceAnatomySha256: hash("e"),
    fitterAlgorithm: "STRUCTURAL_ANATOMY_LOCAL_SKINNING_V40",
    baseRigSha256: hash("f"),
    landmarkOverrides: [],
    jointOverrides: [],
    componentBindings: [],
    regionWeightConstraints: [],
    weightOverrides: [],
    contentSha256: hash("0"),
  };
}

describe("reference supermodel rig authoring", () => {
  it("strictly parses the sealed context and target nodes", () => {
    expect(parseReferenceSupermodelRigAuthoringV1(JSON.stringify(document())).selectedSupermodelResref).toBe("c_wolf");
    const target = parseReferenceSupermodelTargetRigV1(JSON.stringify({
      schemaVersion: 1,
      profileId: "target",
      contentSha256: hash("f"),
      provenance: {},
      targetBounds: { min: [0, 0, 0], max: [1, 1, 1] },
      alignmentAnchor: [0, 0, 0],
      nodes: [{ id: 7, name: "tail", parentId: 1, bindLocalMatrix: [
        1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, -0.5, 0.2, 1,
      ] }],
      segments: [],
    }));
    expect(target.nodes[0]).toMatchObject({ id: 7, name: "tail", parentId: 1 });
  });

  it("round-trips translation, rotation and scale and replaces one carrier override", () => {
    const matrix = composeJointMatrixV1({
      translation: [1.25, -0.5, 0.2],
      rotationDegrees: [15, -20, 35],
      scale: [1, 1.1, 0.9],
    });
    const transform = decomposeJointMatrixV1(matrix);
    expect(transform.translation).toEqual([1.25, -0.5, 0.2]);
    expect(transform.rotationDegrees[0]).toBeCloseTo(15, 4);
    expect(transform.rotationDegrees[1]).toBeCloseTo(-20, 4);
    expect(transform.rotationDegrees[2]).toBeCloseTo(35, 4);
    expect(transform.scale[1]).toBeCloseTo(1.1, 5);

    const first = upsertJointOverrideV1(document(), {
      carrierPartNumber: 7,
      bindLocalMatrix: matrix,
      semanticRole: "tail_base",
      jointAxis: [0, 1, 0],
      locked: false,
    });
    const second = upsertJointOverrideV1(first, {
      carrierPartNumber: 7,
      bindLocalMatrix: composeJointMatrixV1({
        translation: [2, 0, 0], rotationDegrees: [0, 0, 0], scale: [1, 1, 1],
      }),
      semanticRole: null,
      jointAxis: null,
      locked: false,
    });
    expect(second.jointOverrides).toHaveLength(1);
    expect(second.jointOverrides[0]?.bindLocalMatrix[12]).toBe(2);
    expect(second.sourceSha256).toBe(hash("a"));
  });

  it("rejects malformed matrices before they reach WASM", () => {
    const malformed = { ...document(), jointOverrides: [{
      carrierPartNumber: 7,
      bindLocalMatrix: [1, 2],
      semanticRole: null,
      jointAxis: null,
      locked: false,
    }] };
    expect(() => parseReferenceSupermodelRigAuthoringV1(JSON.stringify(malformed))).toThrow(/bindLocalMatrix/);
  });

  it("parses and deterministically edits anatomy authoring V2", () => {
    const parsed = parseReferenceSupermodelRigAuthoringV2(JSON.stringify(documentV2()));
    const withLandmark = upsertLandmarkOverrideV2(parsed, {
      landmarkId: "appendage_tip_0",
      carrierPartNumber: 21,
      targetWorldPosition: [0, -1.2, 0.6],
      locked: true,
    });
    const withComponent = upsertComponentBindingV2(withLandmark, {
      segmentId: 3,
      componentIndex: 18,
      regionId: "tail_fur",
      allowedBoneNodeIds: [20, 21],
      locked: false,
    });
    const withRegion = upsertRegionWeightConstraintV2(withComponent, {
      segmentId: 3,
      regionId: "tail_tip",
      vertexIndices: [2, 8, 13],
      allowedBoneNodeIds: [21],
      forbiddenBoneNodeIds: [18, 19],
      maximumInfluenceCount: 2,
      locked: false,
    });
    expect(withRegion.landmarkOverrides).toHaveLength(1);
    expect(withRegion.componentBindings[0]).toMatchObject({ componentIndex: 18, regionId: "tail_fur" });
    expect(withRegion.regionWeightConstraints[0]?.vertexIndices).toEqual([2, 8, 13]);
    expect(withRegion.structuralProfileSha256).toBe(hash("d"));
  });

  it("rejects a V2 region that asks for more than four influences", () => {
    const malformed = {
      ...documentV2(),
      regionWeightConstraints: [{
        segmentId: 0,
        regionId: "body",
        vertexIndices: [0],
        allowedBoneNodeIds: [1],
        forbiddenBoneNodeIds: [],
        maximumInfluenceCount: 5,
        locked: false,
      }],
    };
    expect(() => parseReferenceSupermodelRigAuthoringV2(JSON.stringify(malformed))).toThrow(/maximumInfluenceCount/);
  });
});
