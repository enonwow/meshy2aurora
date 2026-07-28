import { describe, expect, it } from "vitest";
import { FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 } from "../source/directCreatureAnimationProfile";
import type {
  AnimationMappingProvenanceV1,
  CreatureAnimationAuthoringV1,
  SourceAnimationClipV1,
} from "./types";
import {
  AURORA_ANIMATION_STATE_CATALOG_V1,
  assertDirectCreatureCatalogParityV1,
  findBaseSlotDefinitionV1,
  getAuroraAnimationStateCatalogV1,
  getDirectCreatureBaseCatalogV1,
  getPlaybackPolicyForStateV1,
  filterAnimationCatalogV1,
  isDirectCreatureBaseSlotV1,
  projectAnimationCatalogRowsV1,
  resolveBaseSlotForStateV1,
  validateDirectCreatureCatalogContractV1,
} from "./catalog";
import type { CreatureAnimationCatalogContractV1 } from "./types";
import sharedCatalog from "../../../../../contracts/creature-animation-catalog-v1.json";

describe("creature animation mapping contract v1", () => {
  it("projects the exact existing 42-slot namespace in stable order", () => {
    const catalog = getDirectCreatureBaseCatalogV1();

    expect(catalog).toHaveLength(42);
    expect(catalog.map(({ slot }) => slot)).toEqual(FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1);
    expect(new Set(catalog.map(({ stateId }) => stateId))).toHaveLength(42);
    expect(getAuroraAnimationStateCatalogV1()).toEqual(AURORA_ANIMATION_STATE_CATALOG_V1);
    expect(assertDirectCreatureCatalogParityV1()).toBe(true);
  });

  it("resolves semantic states for S/L and keeps derived fields read-only", () => {
    const walk = AURORA_ANIMATION_STATE_CATALOG_V1.find(({ slot }) => slot === "cwalk");
    expect(walk).toBeDefined();
    expect(resolveBaseSlotForStateV1(walk!.stateId, "S")).toBe("cwalk");
    expect(resolveBaseSlotForStateV1(walk!.stateId, "L")).toBe("cwalk");
    expect(getPlaybackPolicyForStateV1(walk!.stateId)).toBe("ENGINE_MANAGED");
    expect({
      stateId: walk!.stateId,
      modelTypeS: resolveBaseSlotForStateV1(walk!.stateId, "S"),
      modelTypeL: resolveBaseSlotForStateV1(walk!.stateId, "L"),
      playback: getPlaybackPolicyForStateV1(walk!.stateId),
    }).toMatchInlineSnapshot(`
      {
        "modelTypeL": "cwalk",
        "modelTypeS": "cwalk",
        "playback": "ENGINE_MANAGED",
        "stateId": "aurora.direct-creature.locomotion.walk",
      }
    `);
    expect(() => resolveBaseSlotForStateV1("aurora.direct-creature.unknown", "S")).toThrow(
      /unknown Aurora animation state/i,
    );
  });

  it("rejects missing, duplicate and reordered shared catalog slots", () => {
    const clone = () => (
      JSON.parse(JSON.stringify(sharedCatalog)) as CreatureAnimationCatalogContractV1
    );
    const missing = clone();
    missing.states.pop();
    expect(() => validateDirectCreatureCatalogContractV1(missing)).toThrow(
      /invalid shared creature animation catalog/i,
    );

    const duplicate = clone();
    duplicate.states[41] = { ...duplicate.states[0] };
    expect(() => validateDirectCreatureCatalogContractV1(duplicate)).toThrow(
      /invalid at 41/i,
    );

    const reordered = clone();
    [reordered.states[0], reordered.states[1]] = [
      reordered.states[1],
      reordered.states[0],
    ];
    expect(() => validateDirectCreatureCatalogContractV1(reordered)).toThrow(
      /invalid at 0/i,
    );
  });

  it("guards and looks up canonical slots without accepting custom names", () => {
    expect(isDirectCreatureBaseSlotV1("cpause1")).toBe(true);
    expect(isDirectCreatureBaseSlotV1("custom1")).toBe(false);
    expect(findBaseSlotDefinitionV1("cpause1")?.slot).toBe("cpause1");
    expect(findBaseSlotDefinitionV1("custom1")).toBeUndefined();
  });

  it("keeps provider, asset and ownership as separate provenance dimensions", () => {
    const provenance: AnimationMappingProvenanceV1 = {
      provider: "SOURCE_GLB",
      assetId: "selected-source-glb",
      ownership: "USER_OWNED",
    };
    const sourceClip: SourceAnimationClipV1 = {
      clipId: "clip-1",
      name: "Walk",
      durationSeconds: 1.25,
      trackCount: 8,
      targetNodeIds: [1, 2, 3],
      targetPaths: ["Hips.position"],
    };
    const authoring = {
      schemaVersion: 1,
      profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
      modelType: "S",
      sourceRevision: "sha256:owner-source",
      authoringRevision: 1,
      assignments: [{
        targetSlot: "cwalk",
        sourceKind: "SOURCE_CLIP",
        sourceClipName: sourceClip.name,
        customAnimationId: null,
        provenance,
      }],
      fallbacks: [],
      customAnimations: [],
    } satisfies CreatureAnimationAuthoringV1;

    expect(JSON.parse(JSON.stringify(authoring))).toMatchObject({
      schemaVersion: 1,
      modelType: "S",
      assignments: [{
        targetSlot: "cwalk",
        provenance: {
          provider: "SOURCE_GLB",
          assetId: "selected-source-glb",
          ownership: "USER_OWNED",
        },
      }],
    });
  });

  it("projects base, review and custom rows without changing the Base 42 count", () => {
    const provenance: AnimationMappingProvenanceV1 = {
      provider: "SOURCE_GLB",
      assetId: "selected-source-glb",
      ownership: "USER_OWNED",
    };
    const authoring: CreatureAnimationAuthoringV1 = {
      schemaVersion: 1,
      profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
      modelType: "S",
      sourceRevision: "sha256:owner-source",
      authoringRevision: 2,
      assignments: [{
        targetSlot: "cwalk",
        sourceKind: "SOURCE_CLIP",
        sourceClipName: "Walk",
        customAnimationId: null,
        provenance,
      }],
      fallbacks: [{
        id: "fallback-run",
        targetSlot: "crun",
        sourceSlot: "cwalk",
        reason: "Temporary locomotion fallback",
        review: "PENDING",
      }],
      customAnimations: [{
        id: "custom-wave",
        name: "custom1",
        playback: "ONE_SHOT",
        sourceClipName: "Wave",
        phases: [],
        provenance,
      }],
    };

    const rows = projectAnimationCatalogRowsV1(authoring, {
      sourceClips: [{
        clipId: "clip-walk",
        name: "Walk",
        durationSeconds: 1,
        trackCount: 3,
        targetNodeIds: [1, 2],
        targetPaths: ["Hips.position"],
      }],
    });

    expect(filterAnimationCatalogV1(rows, "", "BASE_42")).toHaveLength(42);
    expect(filterAnimationCatalogV1(rows, "", "CUSTOM")).toHaveLength(1);
    expect(rows.find(({ slot }) => slot === "cwalk")).toMatchObject({
      kind: "BASE",
      status: "MAPPED",
      modelType: "S",
      playbackPolicy: "ENGINE_MANAGED",
      sourceLabel: "Walk",
      provenance,
    });
    expect(rows.find(({ slot }) => slot === "crun")).toMatchObject({
      kind: "BASE",
      status: "NEEDS_REVIEW",
      sourceLabel: "cwalk",
    });
    expect(rows.find(({ slot }) => slot === "cpause1")).toMatchObject({
      status: "BLOCKED",
      diagnosticCodes: ["M2A-ANIMATION-SOURCE-UNASSIGNED"],
    });
    expect(rows.find(({ id }) => id === "custom:custom-wave")).toMatchObject({
      kind: "CUSTOM",
      status: "BLOCKED",
      diagnosticCodes: ["M2A-ANIMATION-SOURCE-CLIP-MISSING"],
    });
    expect(filterAnimationCatalogV1(rows, "run", "BASE_42").map(({ slot }) => slot))
      .toContain("crun");
    expect(filterAnimationCatalogV1(rows, "", "NEEDS_ATTENTION"))
      .not.toContainEqual(expect.objectContaining({ slot: "cwalk" }));
  });
});
