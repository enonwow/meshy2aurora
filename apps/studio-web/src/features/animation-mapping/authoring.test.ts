import { describe, expect, it } from "vitest";
import {
  assignAnimationSourceV1,
  clearAnimationSourceV1,
  getAnimationAttentionItems,
  getAvailableAnimationSourcesV1,
  selectAnimationCatalogRow,
} from "./authoring";
import {
  detectFallbackCyclesV1,
  resolveEffectiveAnimationSourceV1,
} from "./fallbacks";
import {
  addCustomAnimationV1,
  createCustomAnimationV1,
  removeCustomAnimationV1,
  updateCustomAnimationV1,
  validateCustomAnimationNameV1,
  validateCustomAnimationOutputNamesV1,
  validateCustomAnimationPhasesV1,
} from "./customAnimations";
import { projectAnimationCatalogRowsV1 } from "./catalog";
import { createCreatureAnimationAuthoringV1 } from "./state";

const inspection = {
  sourceClips: [
    {
      clipId: "walk",
      name: "Walk",
      durationSeconds: 1,
      trackCount: 2,
      targetNodeIds: [1],
      targetPaths: ["Hips.translation"],
    },
    {
      clipId: "empty",
      name: "Run",
      durationSeconds: 1,
      trackCount: 0,
      targetNodeIds: [],
      targetPaths: [],
    },
  ],
};

describe("animation mapping authoring functions", () => {
  it("lists structurally usable sources and assigns or clears one base slot", () => {
    const sources = getAvailableAnimationSourcesV1("cwalk", inspection);
    expect(sources).toEqual([
      expect.objectContaining({
        clipId: "walk",
        clipName: "Walk",
        score: 0.98,
        requiresReview: false,
      }),
    ]);
    let authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
    authoring = assignAnimationSourceV1(authoring, "cwalk", sources[0], {
      provider: "SOURCE_GLB",
      assetId: "source",
      ownership: "USER_OWNED",
    });
    expect(authoring.assignments).toContainEqual(expect.objectContaining({
      targetSlot: "cwalk",
      sourceClipName: "Walk",
    }));
    authoring = clearAnimationSourceV1(authoring, "cwalk");
    expect(authoring.assignments).toEqual([]);
  });

  it("selects rows and projects attention without mutating the catalog", () => {
    const authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
    const rows = projectAnimationCatalogRowsV1(authoring, inspection);
    expect(selectAnimationCatalogRow("base:cwalk", rows)?.slot).toBe("cwalk");
    expect(selectAnimationCatalogRow("missing", rows)).toBeNull();
    expect(getAnimationAttentionItems(authoring, inspection)).toHaveLength(42);
  });

  it("detects fallback cycles and resolves an accepted chain to a direct source", () => {
    const authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
    authoring.assignments.push({
      targetSlot: "cdamagel",
      sourceKind: "SOURCE_CLIP",
      sourceClipName: "Damage",
      customAnimationId: null,
      provenance: {
        provider: "SOURCE_GLB",
        assetId: "source",
        ownership: "USER_OWNED",
      },
    });
    authoring.fallbacks.push({
      id: "right-to-left",
      targetSlot: "cdamager",
      sourceSlot: "cdamagel",
      reason: "Mirror",
      review: "ACCEPTED",
    });
    expect(resolveEffectiveAnimationSourceV1(
      "cdamager",
      authoring.assignments,
      authoring.fallbacks,
    )).toMatchObject({
      resolvedSlot: "cdamagel",
      viaFallbackSlots: ["cdamager"],
      assignment: { sourceClipName: "Damage" },
    });

    const cycles = detectFallbackCyclesV1([
      ...authoring.fallbacks,
      {
        id: "left-to-right",
        targetSlot: "cdamagel",
        sourceSlot: "cdamager",
        reason: "Cycle",
        review: "PENDING",
      },
    ]);
    expect(cycles).toContainEqual(expect.objectContaining({
      code: "M2A-ANIMATION-FALLBACK-CYCLE",
      level: "BLOCKING",
    }));
  });

  it("creates, validates, updates and safely removes custom animations", () => {
    expect(validateCustomAnimationNameV1("custom_wave", [])).toEqual([]);
    expect(validateCustomAnimationNameV1("cwalk", [])).toContainEqual(
      expect.objectContaining({ code: "M2A-ANIMATION-CUSTOM-NAME-BASE-SLOT" }),
    );
    const custom = createCustomAnimationV1({
      id: "wave",
      name: "custom_wave",
      playback: "ONE_SHOT",
      sourceClipName: "Wave",
      phases: [],
      provenance: {
        provider: "USER_CUSTOM",
        assetId: "source",
        ownership: "USER_OWNED",
      },
    });
    expect(validateCustomAnimationPhasesV1({
      ...custom,
      phases: [{ phase: "END", sourceClipName: "WaveEnd" }],
    })).toContainEqual(expect.objectContaining({
      code: "M2A-ANIMATION-CUSTOM-ONE-SHOT-PHASES",
    }));
    expect(validateCustomAnimationPhasesV1({
      ...custom,
      playback: "LOOPING_PHASED",
      sourceClipName: null,
      phases: [{ phase: "LOOP", sourceClipName: "WaveLoop" }],
    })).toEqual([]);
    expect(validateCustomAnimationPhasesV1({
      ...custom,
      playback: "LOOPING_PHASED",
      sourceClipName: null,
      phases: [
        { phase: "START", sourceClipName: "WaveStart" },
        { phase: "LOOP", sourceClipName: "WaveLoop" },
        { phase: "END", sourceClipName: "WaveEnd" },
      ],
    })).toEqual([]);
    expect(validateCustomAnimationOutputNamesV1([
      {
        ...custom,
        id: "phased-wave",
        name: "wave",
        playback: "LOOPING_PHASED",
        sourceClipName: null,
        phases: [
          { phase: "START", sourceClipName: "WaveStart" },
          { phase: "LOOP", sourceClipName: "WaveLoop" },
        ],
      },
      {
        ...custom,
        id: "one-shot-wave-start",
        name: "wave_s",
      },
    ])).toContainEqual(expect.objectContaining({
      code: "M2A-ANIMATION-CUSTOM-OUTPUT-NAME-CONFLICT",
    }));

    let authoring = addCustomAnimationV1(
      createCreatureAnimationAuthoringV1("sha256:source", "S"),
      custom,
    );
    authoring = updateCustomAnimationV1(authoring, custom.id, { name: "custom_wave2" });
    expect(authoring.customAnimations[0].name).toBe("custom_wave2");
    authoring.assignments.push({
      targetSlot: "ctaunt",
      sourceKind: "CUSTOM",
      sourceClipName: null,
      customAnimationId: custom.id,
      provenance: custom.provenance,
    });
    expect(() => removeCustomAnimationV1(authoring, custom.id)).toThrow(/in use/i);
    expect(removeCustomAnimationV1(authoring, custom.id, true).customAnimations).toEqual([]);
  });
});
