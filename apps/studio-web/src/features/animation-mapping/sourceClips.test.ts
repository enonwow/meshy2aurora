import * as THREE from "three";
import { describe, expect, it } from "vitest";
import {
  classifySourceAnimationClipV1,
  detectSourceClipDuplicatesV1,
  inventorySourceAnimationClipsV1,
  normalizeSourceAnimationNameV1,
  summarizeSourceAnimationCoverageV1,
  validateSourceClipRigTargetsV1,
} from "./sourceClips";

function clip(name: string, target = "Hips.position") {
  return new THREE.AnimationClip(name, 1, [
    new THREE.VectorKeyframeTrack(target, [0, 1], [0, 0, 0, 0, 0, 1]),
  ]);
}

describe("source animation inspection v1", () => {
  it("normalizes only comparison keys and preserves inventory names", () => {
    expect(normalizeSourceAnimationNameV1(" Walk-Cycle_01 ")).toBe("walkcycle01");
    const inventory = inventorySourceAnimationClipsV1([
      clip(" Walk "),
      clip("", "Spine.quaternion"),
    ]);

    expect(inventory).toEqual([
      expect.objectContaining({
        clipId: "source-clip:0",
        name: " Walk ",
        durationSeconds: 1,
        trackCount: 1,
        targetPaths: ["Hips.position"],
      }),
      expect.objectContaining({
        clipId: "source-clip:1",
        name: "Unnamed clip 2",
        targetPaths: ["Spine.quaternion"],
      }),
    ]);
  });

  it("classifies exact Aurora slots above aliases and leaves generic attack ambiguous", () => {
    expect(classifySourceAnimationClipV1({
      clipId: "exact",
      name: "CWALK",
      durationSeconds: 1,
      trackCount: 1,
      targetNodeIds: [],
      targetPaths: [],
    })[0]).toMatchObject({
      slot: "cwalk",
      score: 1,
      reasonCode: "EXACT_AURORA_SLOT",
    });
    expect(classifySourceAnimationClipV1({
      clipId: "alias",
      name: "Walking",
      durationSeconds: 1,
      trackCount: 1,
      targetNodeIds: [],
      targetPaths: [],
    })[0]).toMatchObject({
      slot: "cwalk",
      score: 0.98,
      reasonCode: "EXACT_SEMANTIC_ALIAS",
    });
    const attack = classifySourceAnimationClipV1({
      clipId: "ambiguous",
      name: "Attack",
      durationSeconds: 1,
      trackCount: 1,
      targetNodeIds: [],
      targetPaths: [],
    });
    expect(attack).toHaveLength(3);
    expect(attack.every(({ score }) => score < 0.9)).toBe(true);
  });

  it("keeps audited Meshy action names as review candidates instead of auto-mapping them", () => {
    const meshy = (name: string) => classifySourceAnimationClipV1({
      clipId: name,
      name,
      durationSeconds: 1,
      trackCount: 1,
      targetNodeIds: [],
      targetPaths: [],
    });
    expect(meshy("0 Idle")[0]).toMatchObject({
      slot: "cpause1",
      score: 0.85,
      reasonCode: "MESHY_ACTION_CANDIDATE",
    });
    expect(meshy("4 Attack")).toHaveLength(3);
    expect(meshy("613 Casual_Walk_inplace")[0]).toMatchObject({
      slot: "cwalk",
      score: 0.82,
      reasonCode: "MESHY_ACTION_CANDIDATE",
    });
  });

  it("reports normalized duplicates, semantic conflicts and rig target mismatches", () => {
    const clips = inventorySourceAnimationClipsV1([
      clip("Walk"),
      clip(" walk "),
      clip("Run", "Unknown.position"),
    ]);
    expect(detectSourceClipDuplicatesV1(clips)).toEqual(expect.arrayContaining([
      expect.objectContaining({ code: "M2A-ANIMATION-SOURCE-NAME-DUPLICATE" }),
      expect.objectContaining({ code: "M2A-ANIMATION-SOURCE-MEANING-DUPLICATE" }),
    ]));
    expect(validateSourceClipRigTargetsV1(clips[2], {
      requiredNodeIds: [],
      allowedNodeIds: [],
      requiredTargetPaths: ["Hips.position"],
      allowedTargetPaths: ["Hips.position"],
    })).toEqual(expect.arrayContaining([
      expect.objectContaining({ code: "M2A-ANIMATION-RIG-TARGET-MISSING" }),
      expect.objectContaining({ code: "M2A-ANIMATION-RIG-TARGET-FOREIGN" }),
    ]));
    expect(validateSourceClipRigTargetsV1({
      ...clips[2],
      trackCount: 0,
      targetPaths: [],
    }, {
      requiredNodeIds: [],
      allowedNodeIds: [],
      requiredTargetPaths: [],
      allowedTargetPaths: [],
    })).toContainEqual(expect.objectContaining({
      code: "M2A-ANIMATION-SOURCE-TRACKS-MISSING",
    }));
  });

  it("summarizes coverage without treating unknown clips as base coverage", () => {
    const summary = summarizeSourceAnimationCoverageV1(
      inventorySourceAnimationClipsV1([
        clip("cwalk"),
        clip("Running"),
        clip("Dance"),
      ]),
    );

    expect(summary).toMatchObject({
      sourceClipCount: 3,
      confidentlyCoveredSlotCount: 2,
      unknownClipCount: 1,
      conflictCount: 0,
    });
    expect(summary.confidentlyCoveredSlots).toEqual(["cwalk", "crun"]);
    expect(summary.missingBaseSlotCount).toBe(40);
  });
});
