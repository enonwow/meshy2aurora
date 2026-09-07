import { describe, expect, it } from "vitest";
import type { ReadbackMesh, ReadbackNode } from "./types";
import {
  classifyReadbackRigV1,
  visibleRigConnectionsV1,
} from "./rigOverlay";

const mesh: ReadbackMesh = {
  vertices: [{ x: 0, y: 0, z: 0 }],
  normals: [],
  uv0: [],
  rawIndices: [[0, 0, 0]],
  faces: [{ vertexIndices: [0, 0, 0] }],
};

function node(
  number: number,
  name: string,
  children: ReadbackNode[] = [],
  ownsMesh = false,
): ReadbackNode {
  return {
    offset: number * 4,
    number,
    name,
    controllers: [],
    ...(ownsMesh ? { mesh } : {}),
    children,
  };
}

describe("Aurora rig semantics", () => {
  it("separates the representative wolf rigid pivots from roots and attachment helpers", () => {
    const roots = [node(1, "c_Barghest", [
      node(2, "Wolf_rootdummy", [
        node(3, "torso", [node(4, "head_g", [], true)], true),
      ]),
      node(5, "impact"),
      node(6, "headconjure"),
      node(7, "handconjure"),
    ])];

    const rig = classifyReadbackRigV1(roots);

    expect(rig.counts).toEqual({
      MODEL_ROOT: 1,
      RIG_ROOT: 1,
      SKIN_BONE: 0,
      RIGID_PIVOT: 2,
      ATTACHMENT: 3,
      HELPER: 0,
      UNKNOWN: 0,
    });
    expect(rig.nodes.map(({ name, category }) => [name, category])).toEqual([
      ["c_Barghest", "MODEL_ROOT"],
      ["Wolf_rootdummy", "RIG_ROOT"],
      ["torso", "RIGID_PIVOT"],
      ["head_g", "RIGID_PIVOT"],
      ["impact", "ATTACHMENT"],
      ["headconjure", "ATTACHMENT"],
      ["handconjure", "ATTACHMENT"],
    ]);

    expect(visibleRigConnectionsV1(rig, false).map(({ parentName, childName }) => `${parentName}->${childName}`))
      .toEqual(["Wolf_rootdummy->torso", "torso->head_g"]);
    expect(visibleRigConnectionsV1(rig, true).map(({ parentName, childName }) => `${parentName}->${childName}`))
      .toEqual(expect.arrayContaining([
        "Wolf_rootdummy->torso",
        "torso->head_g",
        "c_Barghest->impact",
        "c_Barghest->headconjure",
        "c_Barghest->handconjure",
      ]));
  });

  it("marks active skin slots as skin bones instead of rigid pivots", () => {
    const roots = [node(1, "model", [node(2, "rootdummy", [node(3, "spine", [], true)])])];
    roots[0]!.skin = {
      nodeToBoneMap: [-1, -1, 0],
      inlineMapping: [2],
      inverseBoneRotationsRaw: Array.from({ length: 3 }, () => [1, 0, 0, 0]),
      inverseBoneTranslations: Array.from({ length: 3 }, () => ({ x: 0, y: 0, z: 0 })),
      vertexWeights: [[1, 0, 0, 0]],
      boneReferences: [[0, 0, 0, 0]],
    };

    expect(classifyReadbackRigV1(roots).nodes.find(({ name }) => name === "spine")?.category)
      .toBe("SKIN_BONE");
  });
});
