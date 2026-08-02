import { describe, expect, it } from "vitest";
import operationParityFixture from
  "../animation-studio/fixtures/animation-studio-operations-v1.json";
import {
  fingerprintAnimationStudioDocumentV1,
  migrateCreatureAnimationAuthoringV1ToV2,
  parseAnimationStudioDocumentV1,
  serializeAnimationStudioDocumentV1,
} from "../animation-studio/schema";
import type { CreatureAnimationAuthoringV1 } from
  "../animation-mapping/types";
import type {
  AnimationStudioDocumentV1,
  AuthoredAnimationClipV1,
  CreatureAnimationAuthoringV2,
} from "../animation-studio/types";
import {
  assignCustomAnimationToBaseSlotV2,
  applyCustomAttackDemoRouteV1,
  cloneSourceClipForEditingV1,
  collectAnimationKeyTimesV1,
  createAnimationEventDraftV1,
  createBlankPoseClipV1,
  createCustomDefinitionFromAuthoredClipV1,
  createHumanoidSwordSlashFromSourceClipV1,
  createPhasedCustomDefinitionFromAuthoredClipsV1,
  createProceduralTemplateClipV1,
  deleteSelectedKeysV1,
  getAuthoredClipUsageV1,
  insertKeyAtPlayheadV1,
  moveSelectedKeysV1,
  normalizeAnimationQuaternionsV1,
  normalizeQuaternionV1,
  openCustomAnimationInEditorV1,
  projectAnimationStudioLibraryV1,
  projectCustomAnimationLibraryV1,
  retimeAnimationEditorSelectionV1,
  snapAnimationTimeV1,
  trimAnimationEditorSelectionV1,
  validateAnimationEventDraftV1,
  validateCustomAnimationAssignmentV2,
} from "./editing";

const SOURCE_REVISION = "a".repeat(64);

type OperationParityOperationV1 =
  | {
      op: "INSERT_KEYFRAME";
      trackId: string;
      timeSeconds: number;
      value: number[];
    }
  | {
      op: "UPDATE_KEYFRAME";
      trackId: string;
      keyId: string;
      timeSeconds: number;
      value: number[];
    }
  | {
      op: "MOVE_KEYFRAME";
      trackId: string;
      keyId: string;
      timeSeconds: number;
    }
  | {
      op: "NORMALIZE_QUATERNIONS";
      trackId: string;
    }
  | {
      op: "TRIM_CLIP";
      startSeconds: number;
      endSeconds: number;
    }
  | {
      op: "RETIME_CLIP";
      newLengthSeconds: number;
    };

interface OperationParityFixtureV1 {
  schemaVersion: 1;
  initialDocument: AnimationStudioDocumentV1;
  operations: OperationParityOperationV1[];
  expectedDocument: AnimationStudioDocumentV1;
  expectedFingerprintSha256: string;
}

function clip(): AuthoredAnimationClipV1 {
  return {
    id: "authored-1",
    name: "custom_wave",
    kind: "MOTION",
    status: "VALID",
    source: {
      kind: "BLANK_POSE",
      sourceRevision: SOURCE_REVISION,
      sourceClipName: null,
      sourceClipFingerprint: null,
      proceduralTemplate: null,
    },
    lengthSeconds: 2,
    transitionSeconds: 0.1,
    animationRoot: "root",
    tracks: [{
      id: "track-1",
      targetNodeId: 7,
      path: "ROTATION",
      interpolation: "LINEAR",
      keyframes: [
        { id: "key-1", timeSeconds: 0, value: [0, 0, 0, 1] },
        { id: "key-2", timeSeconds: 1, value: [0, 0.7071068, 0, 0.7071068] },
        { id: "key-3", timeSeconds: 2, value: [0, 0, 0, 1] },
      ],
    }],
    events: [
      { id: "event-1", timeSeconds: 0.5, name: "snd_swing" },
      { id: "event-2", timeSeconds: 1.5, name: "hit" },
    ],
    revision: 1,
  };
}

function document(): AnimationStudioDocumentV1 {
  return {
    schemaVersion: 1,
    sourceRevision: SOURCE_REVISION,
    authoringRevision: 1,
    status: "VALID",
    authoredClips: [clip()],
  };
}

function authoring(): CreatureAnimationAuthoringV2 {
  const custom = createCustomDefinitionFromAuthoredClipV1("authored-1", {
    id: "custom-1",
    name: "custom_wave",
  });
  return {
    schemaVersion: 2,
    profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
    modelType: "S",
    sourceRevision: SOURCE_REVISION,
    authoringRevision: 1,
    assignments: [],
    fallbacks: [],
    customAnimations: [custom],
  };
}

describe("animation editor projections", () => {
  it("replays the exact shared Rust/TypeScript F2 operation golden", async () => {
    const fixture = operationParityFixture as unknown as OperationParityFixtureV1;
    const canonicalInitial = parseAnimationStudioDocumentV1(
      serializeAnimationStudioDocumentV1(fixture.initialDocument),
    );
    if (canonicalInitial.kind !== "VALID") {
      throw new Error("shared operation fixture initial document is invalid");
    }
    let current = canonicalInitial.value;
    for (const operation of fixture.operations) {
      const clip = current.authoredClips[0]!;
      let next: AuthoredAnimationClipV1;
      if (operation.op === "INSERT_KEYFRAME") {
        const track = clip.tracks.find(({ id }) => id === operation.trackId)!;
        next = insertKeyAtPlayheadV1(clip, {
          targetNodeId: track.targetNodeId,
          path: track.path,
          value: operation.value,
        }, operation.timeSeconds, "key-0000");
      } else if (operation.op === "UPDATE_KEYFRAME") {
        const track = clip.tracks.find(({ id }) => id === operation.trackId)!;
        next = insertKeyAtPlayheadV1(clip, {
          targetNodeId: track.targetNodeId,
          path: track.path,
          value: operation.value,
        }, operation.timeSeconds, operation.keyId, "REPLACE_EXISTING");
      } else if (operation.op === "MOVE_KEYFRAME") {
        const track = clip.tracks.find(({ id }) => id === operation.trackId)!;
        const key = track.keyframes.find(({ id }) => id === operation.keyId)!;
        next = moveSelectedKeysV1(
          clip,
          [operation.keyId],
          Math.fround(operation.timeSeconds - key.timeSeconds),
        );
      } else if (operation.op === "NORMALIZE_QUATERNIONS") {
        next = normalizeAnimationQuaternionsV1(clip, operation.trackId);
      } else if (operation.op === "TRIM_CLIP") {
        next = trimAnimationEditorSelectionV1(
          clip,
          operation.startSeconds,
          operation.endSeconds,
        );
      } else {
        next = retimeAnimationEditorSelectionV1(
          clip,
          operation.newLengthSeconds,
        );
      }
      current = {
        ...current,
        authoredClips: [next],
      };
    }

    expect(serializeAnimationStudioDocumentV1(current)).toBe(
      serializeAnimationStudioDocumentV1(fixture.expectedDocument),
    );
    expect(current).toEqual(fixture.expectedDocument);
    await expect(fingerprintAnimationStudioDocumentV1(current)).resolves.toBe(
      fixture.expectedFingerprintSha256,
    );
  });

  it("creates the same output-rig local pose rows as the Rust blank-pose operation", () => {
    expect(createBlankPoseClipV1({
      id: "authored-pose",
      name: "pose",
      sourceRevision: SOURCE_REVISION,
      animationRoot: "root",
      rig: [{
        id: 7,
        name: "root",
        parentId: null,
        translation: [1, 2, 3],
        rotation: [0, 0, 0, -2],
      }],
    }).tracks).toEqual([
      {
        id: "track-7-translation",
        targetNodeId: 7,
        path: "TRANSLATION",
        interpolation: "LINEAR",
        keyframes: [{ id: "key-0000", timeSeconds: 0, value: [1, 2, 3] }],
      },
      {
        id: "track-7-rotation",
        targetNodeId: 7,
        path: "ROTATION",
        interpolation: "LINEAR",
        keyframes: [{ id: "key-0000", timeSeconds: 0, value: [0, 0, 0, 1] }],
      },
    ]);
  });

  it("creates a stable blank draft and exposes it in Custom", () => {
    const created = createBlankPoseClipV1({
      id: "authored-new",
      name: "custom_pose",
      sourceRevision: SOURCE_REVISION,
      animationRoot: "root",
    });
    expect(created.status).toBe("DRAFT");
    expect(created.source.kind).toBe("BLANK_POSE");
    expect(projectAnimationStudioLibraryV1(
      { ...document(), authoredClips: [created] },
      [],
    )[0]).toMatchObject({ id: "authored-new", origin: "EDITED", status: "DRAFT" });
  });

  it("projects the canonical root-translation procedural template with LINEAR keys", () => {
    const created = createProceduralTemplateClipV1({
      id: "authored-procedural",
      name: "procedural_root_pulse",
      sourceRevision: SOURCE_REVISION,
      animationRoot: "root",
      lengthSeconds: 2,
      rig: [{
        id: 7,
        name: "root",
        parentId: null,
        translation: [1, 2, 3],
        rotation: [0, 0, 0, 1],
      }],
    }, "ROOT_TRANSLATION_PULSE");

    expect(created).toMatchObject({
      kind: "MOTION",
      status: "DRAFT",
      source: {
        kind: "PROCEDURAL_TEMPLATE",
        proceduralTemplate: "ROOT_TRANSLATION_PULSE",
      },
    });
    expect(created.tracks.every(({ interpolation }) => interpolation === "LINEAR"))
      .toBe(true);
    expect(created.tracks[0]?.keyframes).toEqual([
      { id: "key-0000", timeSeconds: 0, value: [1, 2, 3] },
      { id: "key-0001", timeSeconds: 1, value: [1.1, 2, 3] },
      { id: "key-0002", timeSeconds: 2, value: [1, 2, 3] },
    ]);
  });

  it("creates a humanoid sword slash across the torso and both arm chains", () => {
    const rig = [
      "Hips",
      "Spine02",
      "Spine01",
      "Spine",
      "RightShoulder",
      "RightArm",
      "RightForeArm",
      "RightHand",
      "LeftShoulder",
      "LeftArm",
      "LeftForeArm",
      "Head",
      "LeftUpLeg",
      "LeftLeg",
      "LeftFoot",
      "RightUpLeg",
      "RightLeg",
      "RightFoot",
    ].map((name, id) => ({
      id,
      name,
      parentId: id === 0 ? null : id - 1,
      translation: [0, id, 0] as [number, number, number],
      rotation: [0, 0, 0, 1] as [number, number, number, number],
    }));
    const created = createProceduralTemplateClipV1({
      id: "authored-sword-slash",
      name: "humanoid_slash",
      sourceRevision: SOURCE_REVISION,
      animationRoot: "Hips",
      lengthSeconds: 1,
      rig,
    }, "HUMANOID_SWORD_SLASH");

    expect(created).toMatchObject({
      kind: "MOTION",
      status: "DRAFT",
      source: {
        kind: "PROCEDURAL_TEMPLATE",
        proceduralTemplate: "HUMANOID_SWORD_SLASH",
      },
    });
    const changingRotations = created.tracks.filter((track) => (
      track.path === "ROTATION"
      && track.keyframes.length === 7
      && track.keyframes.some((key, index) => (
        index > 0
        && JSON.stringify(key.value)
          !== JSON.stringify(track.keyframes[index - 1]?.value)
      ))
    ));
    expect(changingRotations).toHaveLength(18);
    expect(changingRotations.every((track) => (
      JSON.stringify(track.keyframes[0]?.value)
        === JSON.stringify(track.keyframes.at(-1)?.value)
    ))).toBe(true);
    expect(changingRotations.flatMap(({ keyframes }) => keyframes).every(({ value }) => (
      Math.abs(value.reduce((sum, component) => (
        sum + component * component
      ), 0) - 1) < 1e-5
    ))).toBe(true);
    expect(
      changingRotations.find(({ targetNodeId }) => targetNodeId === 5)
        ?.keyframes.map(({ timeSeconds }) => timeSeconds),
    ).toEqual([0, 0.18, 0.34, 0.5, 0.66, 0.82, 1]);
    const rightArm = changingRotations.find(
      ({ targetNodeId }) => targetNodeId === 5,
    )!;
    const anticipation = rightArm.keyframes[1]!.value;
    const impact = rightArm.keyframes[2]!.value;
    const quaternionDot = Math.abs(anticipation.reduce(
      (sum, component, index) => sum + component * impact[index]!,
      0,
    ));
    const attackArcDegrees = 2 * Math.acos(Math.min(1, quaternionDot))
      * 180 / Math.PI;
    expect(attackArcDegrees).toBeGreaterThan(20);
    expect(attackArcDegrees).toBeLessThan(150);
    expect(changingRotations.filter(({ targetNodeId }) => (
      targetNodeId >= 12 && targetNodeId <= 17
    ))).toHaveLength(6);

    const hipsTranslation = created.tracks.find((track) => (
      track.targetNodeId === 0 && track.path === "TRANSLATION"
    ))!;
    expect(hipsTranslation.keyframes).toHaveLength(7);
    expect(hipsTranslation.keyframes[0]?.value).toEqual(
      hipsTranslation.keyframes.at(-1)?.value,
    );
    const forwardTravel = hipsTranslation.keyframes.map(({ value }) => value[1]!);
    const verticalTravel = hipsTranslation.keyframes.map(({ value }) => value[2]!);
    expect(Math.max(...forwardTravel) - Math.min(...forwardTravel))
      .toBeGreaterThan(0.1);
    expect(Math.max(...forwardTravel) - Math.min(...forwardTravel))
      .toBeLessThan(0.15);
    expect(Math.max(...verticalTravel) - Math.min(...verticalTravel))
      .toBeGreaterThan(0.02);
    expect(Math.max(...verticalTravel) - Math.min(...verticalTravel))
      .toBeLessThan(0.05);
  });

  it("fails closed when the sword slash preset is used on a non-humanoid rig", () => {
    expect(() => createProceduralTemplateClipV1({
      id: "authored-sword-slash",
      name: "humanoid_slash",
      sourceRevision: SOURCE_REVISION,
      animationRoot: "root",
      rig: [{
        id: 7,
        name: "root",
        parentId: null,
        translation: [0, 0, 0],
        rotation: [0, 0, 0, 1],
      }],
    }, "HUMANOID_SWORD_SLASH")).toThrow(/Hips/);
  });

  it("uses a sampled source pose as the base for the sword slash", () => {
    const rig = [
      "Hips",
      "Spine02",
      "Spine01",
      "Spine",
      "RightShoulder",
      "RightArm",
      "RightForeArm",
      "RightHand",
      "LeftShoulder",
      "LeftArm",
      "LeftForeArm",
      "Head",
      "LeftUpLeg",
      "LeftLeg",
      "LeftFoot",
      "RightUpLeg",
      "RightLeg",
      "RightFoot",
    ].map((name, id) => ({
      id,
      name,
      parentId: id === 0 ? null : id - 1,
      translation: [0, id, 0] as [number, number, number],
      rotation: [0, 0, 0, 1] as [number, number, number, number],
    }));
    const source = createBlankPoseClipV1({
      id: "source-cpause",
      name: "m2a_sword_slash",
      sourceRevision: SOURCE_REVISION,
      animationRoot: "Hips",
      lengthSeconds: 2,
      rig,
    });
    const halfTurn = Math.SQRT1_2;
    const sourceCopy: AuthoredAnimationClipV1 = {
      ...source,
      source: {
        kind: "SOURCE_CLIP_COPY",
        sourceRevision: SOURCE_REVISION,
        sourceClipName: "cpause1",
        sourceClipFingerprint: "b".repeat(64),
        proceduralTemplate: null,
      },
      tracks: source.tracks.map((track) => (
        track.targetNodeId === 0 && track.path === "ROTATION"
          ? {
              ...track,
              keyframes: [
                { id: "k0", timeSeconds: 0, value: [0, 0, 0, 1] },
                { id: "k1", timeSeconds: 1, value: [0, 0, halfTurn, halfTurn] },
              ],
            }
          : track
      )),
    };
    const attack = createHumanoidSwordSlashFromSourceClipV1(
      sourceCopy,
      rig,
      0.5,
    );
    const hips = attack.tracks.find((track) => (
      track.targetNodeId === 0 && track.path === "ROTATION"
    ));

    expect(attack.source).toMatchObject({
      kind: "SOURCE_CLIP_COPY",
      sourceClipName: "cpause1",
    });
    expect(hips?.keyframes).toHaveLength(7);
    expect(hips?.keyframes[0]?.value[2]).toBeCloseTo(Math.sin(Math.PI / 8), 6);
    expect(hips?.keyframes[0]?.value[3]).toBeCloseTo(Math.cos(Math.PI / 8), 6);
  });

  it("duplicates an authored procedural clip without corrupting its source lineage", () => {
    const source = createProceduralTemplateClipV1({
      id: "authored-procedural",
      name: "procedural_root_pulse",
      sourceRevision: SOURCE_REVISION,
      animationRoot: "root",
      rig: [{
        id: 7,
        name: "root",
        parentId: null,
        translation: [1, 2, 3],
        rotation: [0, 0, 0, 1],
      }],
    }, "ROOT_TRANSLATION_PULSE");

    const duplicate = cloneSourceClipForEditingV1(source, {
      id: "authored-procedural-copy",
      name: "procedural_copy",
    });

    expect(duplicate).toMatchObject({
      id: "authored-procedural-copy",
      name: "procedural_copy",
      status: "DRAFT",
      revision: 1,
      source: {
        kind: "PROCEDURAL_TEMPLATE",
        sourceRevision: SOURCE_REVISION,
        sourceClipName: null,
        sourceClipFingerprint: null,
        proceduralTemplate: "ROOT_TRANSLATION_PULSE",
      },
    });
    expect(() => serializeAnimationStudioDocumentV1({
      ...document(),
      authoredClips: [duplicate],
    })).not.toThrow();
  });

  it("edits keys deterministically and keeps strict time", () => {
    const inserted = insertKeyAtPlayheadV1(
      clip(),
      { targetNodeId: 7, path: "ROTATION", value: [0, 0, 0, 2] },
      0.5,
      "key-new",
    );
    expect(inserted.tracks[0]?.keyframes.map(({ timeSeconds }) => timeSeconds))
      .toEqual([0, 0.5, 1, 2]);
    expect(inserted.tracks[0]?.keyframes[1]?.value).toEqual([0, 0, 0, 1]);

    const moved = moveSelectedKeysV1(inserted, ["key-new"], 0.25);
    expect(moved.tracks[0]?.keyframes[1]?.timeSeconds).toBe(0.75);
    expect(deleteSelectedKeysV1(moved, ["key-new"]).tracks[0]?.keyframes)
      .toHaveLength(3);

    expect(() => insertKeyAtPlayheadV1(
      clip(),
      { targetNodeId: 7, path: "ROTATION", value: [0, 0, 0, 1] },
      1,
      "ignored-new-id",
    )).toThrow("explicit replace policy");
    const explicitlyReplaced = insertKeyAtPlayheadV1(
      clip(),
      { targetNodeId: 7, path: "ROTATION", value: [0, 0, 0, 2] },
      1,
      "ignored-new-id",
      "REPLACE_EXISTING",
    );
    expect(explicitlyReplaced.tracks[0]?.keyframes[1]).toEqual({
      id: "key-2",
      timeSeconds: 1,
      value: [0, 0, 0, 1],
    });
  });

  it("trims with sampled boundaries and retimes keys and events together", () => {
    const trimmed = trimAnimationEditorSelectionV1(clip(), 0.5, 1.5);
    expect(trimmed.lengthSeconds).toBe(1);
    expect(trimmed.tracks[0]?.keyframes.map(({ timeSeconds }) => timeSeconds))
      .toEqual([0, 0.5, 1]);
    expect(trimmed.events.map(({ timeSeconds }) => timeSeconds)).toEqual([0, 1]);

    const retimed = retimeAnimationEditorSelectionV1(trimmed, 2);
    expect(retimed.tracks[0]?.keyframes.map(({ timeSeconds }) => timeSeconds))
      .toEqual([0, 1, 2]);
    expect(retimed.events.map(({ timeSeconds }) => timeSeconds)).toEqual([0, 2]);

    const exactBoundaryTrim = trimAnimationEditorSelectionV1(clip(), 0, 2);
    expect(exactBoundaryTrim.tracks[0]?.keyframes.map(({ id }) => id))
      .toEqual(["key-1", "key-2", "key-3"]);
  });

  it("normalizes quaternion, key times, events and 30 fps snap", () => {
    expect(normalizeQuaternionV1([0, 0, 0, 2])).toEqual([0, 0, 0, 1]);
    expect(() => normalizeQuaternionV1([0, 0, 0, 1e-7])).toThrow(
      "norm greater than 1e-6",
    );
    expect(collectAnimationKeyTimesV1(clip())).toEqual([0, 0.5, 1, 1.5, 2]);
    expect(createAnimationEventDraftV1(0.5)).toMatchObject({ timeSeconds: 0.5 });
    expect(snapAnimationTimeV1(0.049, "FRAME_30")).toBeCloseTo(1 / 30);
  });

  it("accepts at most 31 printable ASCII bytes for an Aurora event name", () => {
    expect(validateAnimationEventDraftV1(
      { id: "event-ok", timeSeconds: 0.5, name: "a".repeat(31) },
      clip(),
    )).toEqual([]);
    for (const invalidName of ["", "a".repeat(32), "zażółć", "hit\0now"]) {
      expect(validateAnimationEventDraftV1(
        { id: "event-invalid", timeSeconds: 0.5, name: invalidName },
        clip(),
      )).toEqual([
        expect.objectContaining({ code: "M2A-ANIMATION-EDIT-EVENT" }),
      ]);
    }
  });

  it("assigns only valid authored custom clips without removing the library item", () => {
    const mapping = authoring();
    const library = projectCustomAnimationLibraryV1(mapping, document());
    expect(library[0]).toMatchObject({ id: "custom-1", status: "VALID", assignable: true });
    expect(validateCustomAnimationAssignmentV2("ca1slashl", "custom-1", mapping, document()))
      .toEqual([]);

    const assigned = assignCustomAnimationToBaseSlotV2(
      mapping,
      "ca1slashl",
      "custom-1",
      document(),
    );
    expect(assigned.assignments[0]).toMatchObject({
      sourceKind: "CUSTOM",
      customAnimationId: "custom-1",
      provenance: mapping.customAnimations[0]!.provenance,
    });
    expect(assigned.customAnimations).toHaveLength(1);
    expect(getAuthoredClipUsageV1("authored-1", assigned)).toEqual(["ca1slashl"]);
  });

  it("routes attack demos through all native attack variants without modifying cpause1", () => {
    const mapping = authoring();
    mapping.assignments = ["cpause1", "ca1slashl", "ca1slashr", "ca1stab"].map(
      (targetSlot) => ({
        targetSlot: targetSlot as "cpause1" | "ca1slashl" | "ca1slashr" | "ca1stab",
        sourceKind: "SOURCE_CLIP" as const,
        sourceClipName: targetSlot,
        customAnimationId: null,
        provenance: {
          provider: "SOURCE_GLB" as const,
          assetId: SOURCE_REVISION,
          ownership: "USER_OWNED" as const,
        },
      }),
    );
    const idleBefore = structuredClone(mapping.assignments[0]);

    const demo = applyCustomAttackDemoRouteV1(
      mapping,
      "custom-1",
      document(),
    );

    expect(demo.contract).toEqual({
      schemaVersion: 1,
      customAnimationId: "custom-1",
      runtimeBaseSlots: ["ca1slashl", "ca1slashr", "ca1stab"],
      allNativeAttackVariantsRouted: true,
      runtimeTriggerProfile: "ACTIVE_MONSTER_COMBAT_AI",
      productionIdleSlotPreserved: true,
    });
    expect(demo.authoring.assignments.find(
      ({ targetSlot }) => targetSlot === "cpause1",
    )).toEqual(idleBefore);
    expect(demo.authoring.assignments.filter(
      ({ targetSlot }) => targetSlot.startsWith("ca1"),
    )).toEqual([
      expect.objectContaining({
        targetSlot: "ca1slashl",
        sourceKind: "CUSTOM",
        customAnimationId: "custom-1",
      }),
      expect.objectContaining({
        targetSlot: "ca1slashr",
        sourceKind: "CUSTOM",
        customAnimationId: "custom-1",
      }),
      expect.objectContaining({
        targetSlot: "ca1stab",
        sourceKind: "CUSTOM",
        customAnimationId: "custom-1",
      }),
    ]);

    const phased = structuredClone(mapping);
    phased.customAnimations[0] = {
      ...phased.customAnimations[0]!,
      playback: "LOOPING_PHASED",
      clipReference: null,
      phases: [],
    };
    expect(() => applyCustomAttackDemoRouteV1(
      phased,
      "custom-1",
      document(),
    )).toThrow(/requires a ONE_SHOT Custom animation/);
  });

  it("routes attack demos when the required slots are supplied by accepted fallbacks", () => {
    const mapping = authoring();
    mapping.assignments = [{
      targetSlot: "cwalk",
      sourceKind: "SOURCE_CLIP",
      sourceClipName: "walk",
      customAnimationId: null,
      provenance: {
        provider: "SOURCE_GLB",
        assetId: SOURCE_REVISION,
        ownership: "USER_OWNED",
      },
    }];
    mapping.fallbacks = [
      "cpause1",
      "ca1slashl",
      "ca1slashr",
      "ca1stab",
    ].map((targetSlot) => ({
      id: `fallback:${targetSlot}:cwalk`,
      targetSlot: targetSlot as "cpause1" | "ca1slashl" | "ca1slashr" | "ca1stab",
      sourceSlot: "cwalk" as const,
      reason: "Owner-approved Base 42 fallback.",
      review: "ACCEPTED" as const,
    }));

    const demo = applyCustomAttackDemoRouteV1(
      mapping,
      "custom-1",
      document(),
    );

    expect(demo.authoring.assignments.find(
      ({ targetSlot }) => targetSlot === "cpause1",
    )).toMatchObject({
      targetSlot: "cpause1",
      sourceKind: "SOURCE_CLIP",
      sourceClipName: "walk",
    });
    expect(demo.authoring.assignments.filter(({ targetSlot }) => (
      targetSlot.startsWith("ca1")
    ))).toHaveLength(3);
  });

  it("keeps a migrated source-backed Custom assignable but not openable in the authored editor", () => {
    const legacy: CreatureAnimationAuthoringV1 = {
      schemaVersion: 1,
      profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
      modelType: "S",
      sourceRevision: SOURCE_REVISION,
      authoringRevision: 4,
      assignments: [],
      fallbacks: [],
      customAnimations: [{
        id: "legacy-source-custom",
        name: "legacy_idle",
        playback: "ONE_SHOT",
        sourceClipName: "idle",
        phases: [],
        provenance: {
          provider: "SOURCE_GLB",
          assetId: SOURCE_REVISION,
          ownership: "USER_OWNED",
        },
      }],
    };
    const migrated = migrateCreatureAnimationAuthoringV1ToV2(legacy);
    const emptyStudio: AnimationStudioDocumentV1 = {
      ...document(),
      status: "DRAFT",
      authoredClips: [],
    };
    const sourceInventory = [{
      clipId: "source-idle",
      name: "IDLE",
      durationSeconds: 1.25,
      trackCount: 8,
    }];
    expect(projectCustomAnimationLibraryV1(
      migrated,
      emptyStudio,
      sourceInventory,
    )).toEqual([
      expect.objectContaining({
        id: "legacy-source-custom",
        status: "VALID",
        assignable: true,
        openableInEditor: false,
        sourceClipCount: 1,
        durationSeconds: 1.25,
      }),
    ]);
    expect(validateCustomAnimationAssignmentV2(
      "cwalk",
      "legacy-source-custom",
      migrated,
      emptyStudio,
      sourceInventory,
    )).toEqual([]);
    expect(assignCustomAnimationToBaseSlotV2(
      migrated,
      "cwalk",
      "legacy-source-custom",
      emptyStudio,
      sourceInventory,
    ).assignments).toEqual([
      expect.objectContaining({
        targetSlot: "cwalk",
        sourceKind: "CUSTOM",
        customAnimationId: "legacy-source-custom",
      }),
    ]);
    expect(() => openCustomAnimationInEditorV1(
      "legacy-source-custom",
      migrated,
      emptyStudio,
    )).toThrow("does not reference an authored clip");
  });

  it("creates exactly START/LOOP/END phased references from three stable Valid authored clip IDs", () => {
    const clips = ["start-id", "loop-id", "end-id"].map((id, index) => ({
      ...clip(),
      id,
      name: `phase_${index}`,
      status: "VALID" as const,
    }));
    const studio = { ...document(), authoredClips: clips };
    const phased = createPhasedCustomDefinitionFromAuthoredClipsV1(studio, {
      startAuthoredClipId: "start-id",
      loopAuthoredClipId: "loop-id",
      endAuthoredClipId: "end-id",
    });

    expect(phased.id).toMatch(/^custom-/);
    expect(phased).toMatchObject({
      name: "custom_loop",
      playback: "LOOPING_PHASED",
      clipReference: null,
    });
    expect(phased.phases).toEqual([
      {
        phase: "START",
        clipReference: {
          sourceKind: "AUTHORED_CLIP",
          sourceClipName: null,
          authoredClipId: "start-id",
        },
      },
      {
        phase: "LOOP",
        clipReference: {
          sourceKind: "AUTHORED_CLIP",
          sourceClipName: null,
          authoredClipId: "loop-id",
        },
      },
      {
        phase: "END",
        clipReference: {
          sourceKind: "AUTHORED_CLIP",
          sourceClipName: null,
          authoredClipId: "end-id",
        },
      },
    ]);
    expect(new Set(phased.phases.map(({ phase }) => phase)).size).toBe(3);
    expect(() => createPhasedCustomDefinitionFromAuthoredClipsV1(studio, {
      startAuthoredClipId: "start-id",
      loopAuthoredClipId: "start-id",
      endAuthoredClipId: "end-id",
    })).toThrow(/three distinct authored clips/);
    expect(() => createPhasedCustomDefinitionFromAuthoredClipsV1({
      ...studio,
      authoredClips: clips.map((candidate) => (
        candidate.id === "loop-id" ? { ...candidate, status: "DRAFT" as const } : candidate
      )),
    }, {
      startAuthoredClipId: "start-id",
      loopAuthoredClipId: "loop-id",
      endAuthoredClipId: "end-id",
    })).toThrow(/must be Valid/);
  });
});
