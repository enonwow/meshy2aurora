import { describe, expect, it } from "vitest";
import {
  inheritedAnimationClipsV1,
  supermodelAnimationBindingReportV1,
} from "./SupermodelPreviewViewport";
import type { BinaryMdlInspectionReport } from "../preview/types";

function report(name: string, clipNames: readonly string[]): BinaryMdlInspectionReport {
  return {
    schemaVersion: 1,
    format: "nwn1-binary-mdl",
    nodeTree: { roots: [] },
    diagnostics: [],
    animations: clipNames.map((clipName, index) => ({
      offset: index,
      name: clipName,
      length: 1,
      transition: 0.25,
      animationRoot: name,
      nodeTree: {
        roots: [{
          offset: index,
          number: index,
          name: "root",
          controllers: [{ controllerName: "position", times: [0, 1], values: [[0, 0, 0], [1, 0, 0]] }],
          children: [],
        }],
      },
    })),
  };
}

describe("supermodel inherited preview clips", () => {
  it("uses the nearest supermodel definition before ancestors case-insensitively", () => {
    const clips = inheritedAnimationClipsV1([
      report("child", ["walk", "attack"]),
      report("parent", ["WALK", "idle"]),
    ]);
    expect(clips.map((clip) => clip.name)).toEqual(["walk", "attack", "idle"]);
  });

  it("reports the exact carrier rig, animation source and unmatched controller nodes per clip", () => {
    const carrier = report("c_barghest", []);
    carrier.model = { name: "c_barghest", classification: 0, animationScale: 1, supermodelName: "c_wolf" };
    carrier.nodeTree.roots = [{
      offset: 1,
      number: 1,
      name: "root",
      controllers: [],
      children: [{ offset: 2, number: 2, name: "leg", controllers: [], children: [] }],
    }];
    const wolf = report("c_wolf", ["crun"]);
    wolf.model = { name: "c_wolf", classification: 0, animationScale: 1, supermodelName: "null" };
    wolf.animations[0]!.nodeTree.roots[0]!.children = [{
      offset: 2,
      number: 2,
      name: "leg",
      controllers: [{ controllerName: "orientation", times: [0, 1], values: [[0, 0, 0, 1], [0, 0, 1, 0]] }],
      children: [{
        offset: 3,
        number: 3,
        name: "tail_missing",
        controllers: [{ controllerName: "position", times: [0, 1], values: [[0, 0, 0], [1, 0, 0]] }],
        children: [],
      }],
    }];

    expect(supermodelAnimationBindingReportV1(carrier, [wolf])).toEqual({
      schemaVersion: 1,
      rigSource: "c_barghest",
      clips: [{
        clipName: "crun",
        animationSource: "c_wolf",
        controlledNodeCount: 3,
        matchedNodeCount: 2,
        unmatchedNodeNames: ["tail_missing"],
      }],
    });
  });
});
