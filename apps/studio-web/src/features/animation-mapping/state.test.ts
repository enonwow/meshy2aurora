import { describe, expect, it } from "vitest";
import {
  createCreatureAnimationAuthoringV1,
  isAnimationMappingCurrentV1,
  reduceCreatureAnimationAuthoringV1,
} from "./state";

describe("creature animation authoring state v1", () => {
  it("creates a versioned empty document bound to the selected source", () => {
    expect(createCreatureAnimationAuthoringV1("sha256:source", "S")).toEqual({
      schemaVersion: 1,
      profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
      modelType: "S",
      sourceRevision: "sha256:source",
      authoringRevision: 1,
      assignments: [],
      fallbacks: [],
      customAnimations: [],
    });
  });

  it("reduces assignment, fallback and custom events with one authoring revision each", () => {
    let authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
    authoring = reduceCreatureAnimationAuthoringV1(authoring, {
      type: "ANIMATION_SOURCE_ASSIGNED",
      assignment: {
        targetSlot: "cwalk",
        sourceKind: "SOURCE_CLIP",
        sourceClipName: "Walk",
        customAnimationId: null,
        provenance: {
          provider: "SOURCE_GLB",
          assetId: "source",
          ownership: "USER_OWNED",
        },
      },
    });
    expect(authoring.authoringRevision).toBe(2);
    expect(authoring.assignments).toHaveLength(1);

    authoring = reduceCreatureAnimationAuthoringV1(authoring, {
      type: "ANIMATION_SOURCE_CLEARED",
      slot: "cwalk",
    });
    expect(authoring.authoringRevision).toBe(3);
    expect(authoring.assignments).toEqual([]);

    authoring = {
      ...authoring,
      fallbacks: [{
        id: "fallback",
        targetSlot: "cdamager",
        sourceSlot: "cdamagel",
        reason: "Mirror",
        review: "PENDING",
      }],
    };
    authoring = reduceCreatureAnimationAuthoringV1(authoring, {
      type: "ANIMATION_FALLBACK_APPROVED",
      fallbackId: "fallback",
    });
    expect(authoring.fallbacks[0].review).toBe("ACCEPTED");
    authoring = reduceCreatureAnimationAuthoringV1(authoring, {
      type: "ANIMATION_FALLBACK_REJECTED",
      fallbackId: "fallback",
    });
    expect(authoring.fallbacks[0].review).toBe("REJECTED");

    authoring = reduceCreatureAnimationAuthoringV1(authoring, {
      type: "CUSTOM_ANIMATION_ADDED",
      custom: {
        id: "wave",
        name: "custom1",
        playback: "ONE_SHOT",
        sourceClipName: "Wave",
        phases: [],
        provenance: {
          provider: "USER_CUSTOM",
          assetId: "source",
          ownership: "USER_OWNED",
        },
      },
    });
    authoring = reduceCreatureAnimationAuthoringV1(authoring, {
      type: "CUSTOM_ANIMATION_UPDATED",
      id: "wave",
      patch: { name: "custom2" },
    });
    expect(authoring.customAnimations[0].name).toBe("custom2");
    authoring = reduceCreatureAnimationAuthoringV1(authoring, {
      type: "CUSTOM_ANIMATION_REMOVED",
      id: "wave",
    });
    expect(authoring.customAnimations).toEqual([]);
  });

  it("detects whether a session snapshot is bound to the current source revision", () => {
    const authoring = createCreatureAnimationAuthoringV1("a".repeat(64), "S");
    const current = {
      revision: 3,
      target: "CREATURE" as const,
      source: { sha256: "a".repeat(64) },
      animationMapping: { revision: 3, value: authoring },
    };
    expect(isAnimationMappingCurrentV1(current)).toBe(true);
    expect(isAnimationMappingCurrentV1({
      ...current,
      source: { sha256: "b".repeat(64) },
    })).toBe(false);
    expect(isAnimationMappingCurrentV1({
      ...current,
      target: "PLACEABLE",
    })).toBe(false);
  });
});
