import { describe, expect, it } from "vitest";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import {
  applyAcceptedItemAuthoringRecipeV2,
  applyOwnerDirectedItemCompositionV2,
  buildItemManualFitSnapshotV2,
  deriveHextechShotgunOutputRowV2,
  diffItemAuthoringScopesV2,
  hashItemAuthoringRecipeV2,
  HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2,
  itemPartSupportsReferenceScalingV2,
  resolveAcceptedItemAuthoringRecipeV2,
  resolveOwnerDirectedItemCompositionV2,
  validateItemAuthoringRecipeV2,
  type ItemAuthoringIdentityV2,
  type ItemAuthoringRecipeV2,
} from "./itemAuthoringRecipeV2";
import type { ItemFitReportV4, ItemPartDraft } from "./types";

const sha = (character: string) => character.repeat(64);

const identity: ItemAuthoringIdentityV2 = {
  schemaVersion: 2,
  archetypeId: "HEXTECH_SHOTGUN",
  runtimeDonor: {
    baseItem: 6,
    label: "heavycrossbow",
    itemClass: "WBwXh",
    runtimeClip: "xbowshot",
  },
  output: {
    baseItem: 113,
    label: "hextech_shotgun",
    itemClass: "WHxSh",
    modelType: 2,
  },
  sources: {
    ModelPart1: {
      role: "source-modelpart-bottom",
      sha256: "69c78999590b248bf9c642516ffa595d33774ead3436166963b27dfaa71ad48d",
    },
    ModelPart2: {
      role: "source-modelpart-middle",
      sha256: "8fafe6a55dd77107a67f29c7519f3b6edc390b310f918a89131b003517720147",
    },
    ModelPart3: {
      role: "source-modelpart-top",
      sha256: "6ce1281a4ed8a239bf0d6fc9388fe8a977a2811750d40eab4320e13b642c77bf",
    },
  },
  reference: {
    id: "wbwxh_b_014/wbwxh_m_014/wbwxh_t_014",
    baseitemsSha256: sha("a"),
    attachmentProfileSha256: sha("b"),
  },
};

const draftParts: ItemPartDraft[] = ["ModelPart1", "ModelPart2", "ModelPart3"].map(
  (field, index) => ({
    field,
    label: field,
    token: ["b", "m", "t"][index],
    sourceKind: "MESHY_GLB",
    referenceTable: null,
    requiresExplicitResourceResrefs: false,
    sourceNode: `node-${index}`,
    textureEncoding: "DIRECT_COLOR",
    variant: 1,
    weaponModel: 5,
    weaponColor: 3,
    explicitModelResref: "",
    explicitIconResref: "",
    translation: [index, index + 0.1, index + 0.2],
    rotationDegrees: [0, 0, 0],
    rotationXyzw: [0, 0, 0, 1],
    uniformScale: 0.2 + index * 0.1,
    pivot: [index + 0.3, index + 0.4, index + 0.5],
    targetSpaceScaleXyz: [1, 1, 1],
  }),
);

function recipe(ownerStatus: ItemAuthoringRecipeV2["review"]["ownerStatus"] = "OWNER_ACCEPTED"):
ItemAuthoringRecipeV2 {
  return {
    schemaVersion: 2,
    identity,
    assembly: {
      mode: "MANUAL_FIT",
      baselineFitSolutionSha256: sha("c"),
      validatedFitSolutionSha256: sha("d"),
      parts: draftParts.map((part, index) => ({
        field: part.field as "ModelPart1" | "ModelPart2" | "ModelPart3",
        sourceSha256: Object.values(identity.sources)[index].sha256,
        translation: [...part.translation],
        rotationXyzw: [...part.rotationXyzw],
        authoredRotationDegrees: [...part.rotationDegrees],
        uniformScale: part.uniformScale,
        pivot: [...part.pivot],
        targetSpaceScaleXyz: [...part.targetSpaceScaleXyz],
      })),
    },
    contexts: {
      ITEM_PROPERTIES: {
        cameraPreset: "AURORA_ITEM_PROPERTIES",
        axialRotationDegrees: 0,
      },
      GROUND: { bearingDegrees: 180 },
      EQUIPPED: {
        attachmentRoute: "HAND",
        attachmentProfileSha256: identity.reference.attachmentProfileSha256,
      },
      INVENTORY_ICON: {
        invSlotWidth: 2,
        invSlotHeight: 4,
        rotationDegrees: 0,
        zoom: 1,
        padding: 0.06,
        offset: [0, 0],
      },
    },
    review: {
      candidateSha256: sha("e"),
      technicalStatus: "PASSED",
      semanticChecks: [
        "BUTT_OUTER",
        "GRIP_AT_HAND",
        "TRIGGER_DOWN",
        "CORE_CENTERED",
        "MUZZLE_FORWARD",
        "BROAD_SIDE_VISIBLE",
      ].map((id) => ({
        id: id as ItemAuthoringRecipeV2["review"]["semanticChecks"][number]["id"],
        status: "PASSED" as const,
        evidence: "CANDIDATE_BOUND_REVIEW",
      })),
      ownerStatus,
    },
  };
}

describe("ItemAuthoringRecipeV2", () => {
  it("allows every custom BaseItem 113 part to enter exact manual-fit scaling", () => {
    const profile = {
      slots: [{
        field: "ModelPart1",
        allowAxialExtensionAtMin: false,
        allowAxialExtensionAtMax: false,
      }],
    } as unknown as import("./types").ItemAttachmentProfileV1;

    expect(itemPartSupportsReferenceScalingV2(113, "ModelPart1", profile)).toBe(true);
    expect(itemPartSupportsReferenceScalingV2(6, "ModelPart1", profile)).toBe(false);
  });

  it("projects output 113 only from an exact 113-row table and audited donor 6", () => {
    const donor = {
      baseItem: 6,
      label: "heavycrossbow",
      itemClass: "WBwXh",
      modelType: 2,
      capability: { compositionProfile: "BOTTOM_MIDDLE_TOP" },
      partSlots: [{}, {}, {}],
      invSlotWidth: 2,
      invSlotHeight: 4,
    } as unknown as import("./types").ItemBaseItemRow;
    const catalog = {
      physicalRowCount: 113,
      rows: [donor],
    } as unknown as import("./types").ItemBaseItemsCatalog;

    expect(deriveHextechShotgunOutputRowV2(catalog)).toMatchObject({
      baseItem: 113,
      label: "hextech_shotgun",
      itemClass: "WHxSh",
      invSlotWidth: 2,
      invSlotHeight: 4,
    });
    expect(() => deriveHextechShotgunOutputRowV2({
      ...catalog,
      physicalRowCount: 112,
    })).toThrow(/exact next BaseItem 113/i);
  });

  it("keeps the runtime donor and custom output BaseItem as separate identities", () => {
    const result = validateItemAuthoringRecipeV2(recipe());

    expect(result.ok).toBe(true);
    expect(identity.runtimeDonor.baseItem).toBe(6);
    expect(identity.output.baseItem).toBe(113);
    expect(identity.runtimeDonor.itemClass).not.toBe(identity.output.itemClass);
  });

  it("has no implicit accepted recipe for the canonical source hashes", () => {
    expect(resolveAcceptedItemAuthoringRecipeV2(identity)).toBeUndefined();
  });

  it("resolves the concept-bound directed candidate only for the exact output, references and sources", () => {
    const observed = {
      outputBaseItem: 113,
      referenceId: identity.reference.id,
      sourceSha256ByField: Object.fromEntries(Object.entries(identity.sources).map(
        ([field, source]) => [field, source.sha256],
      )) as Record<"ModelPart1" | "ModelPart2" | "ModelPart3", string>,
    };

    expect(resolveOwnerDirectedItemCompositionV2(observed)).toBe(
      HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2,
    );
    expect(resolveOwnerDirectedItemCompositionV2({
      ...observed,
      sourceSha256ByField: { ...observed.sourceSha256ByField, ModelPart2: sha("f") },
    })).toBeUndefined();
    expect(HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2).toMatchObject({
      concept: {
        sha256: "cfa31ccea74b53b1e0c55182ec3e1ed4a2072041b433009a448b7717509bd8f9",
      },
      ownerStatus: "NOT_REVIEWED",
      validationTolerance: 0.005,
      correction: {
        field: "ModelPart2",
        allowedTransformFields: ["translation", "rotation"],
      },
    });
  });

  it("binds the directed candidate to the exact tracked owner concept bytes", async () => {
    const concept = await readFile(new URL(
      "../../../../../documentation/concepts/firearm-hextech-shotgun-v1/hextech-shotgun-concept.png",
      import.meta.url,
    ));

    expect(createHash("sha256").update(concept).digest("hex")).toBe(
      HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2.concept.sha256,
    );
  });

  it("applies the exact Bottom and protected Top while limiting the correction to Middle", () => {
    const observed = {
      outputBaseItem: 113,
      referenceId: identity.reference.id,
      sourceSha256ByField: Object.fromEntries(Object.entries(identity.sources).map(
        ([field, source]) => [field, source.sha256],
      )) as Record<"ModelPart1" | "ModelPart2" | "ModelPart3", string>,
    };
    const directed = applyOwnerDirectedItemCompositionV2(
      draftParts,
      HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2,
      observed,
    );

    expect(directed.map(({ translation }) => translation)).toEqual([
      [-0.00431, 0.12917034, 0.15773459],
      [-0.00431, -0.00852164987, -0.18716540565],
      [-0.00431, 0.008945521, -0.49844033],
    ]);
    expect(directed.map(({ rotationDegrees }) => rotationDegrees)).toEqual([
      [90, 0, 90],
      [-90, 0, -90],
      [-90, 0, -90],
    ]);
    expect(directed.map(({ rotationXyzw }) => rotationXyzw)).toEqual([
      [0.5, -0.5, 0.5, 0.5],
      [-0.5, -0.5, -0.5, 0.5],
      [-0.5, -0.5, -0.5, 0.5],
    ]);
    expect(directed.map(({ uniformScale }) => uniformScale)).toEqual([
      0.15796308,
      0.21066014,
      0.13161969,
    ]);
    expect(directed.map(({ targetSpaceScaleXyz }) => targetSpaceScaleXyz)).toEqual([
      [1, 1, 1],
      [1, 1, 1],
      [1, 1, 1],
    ]);
  });

  it("rejects draft and owner-rejected recipes before they can alter editor parts", () => {
    for (const status of ["NOT_REVIEWED", "OWNER_REJECTED"] as const) {
      expect(() => applyAcceptedItemAuthoringRecipeV2(
        draftParts,
        recipe(status),
        identity,
      )).toThrow(/owner-accepted/i);
    }
  });

  it("applies an exact accepted recipe without replacing reference-derived scales or pivots", () => {
    const accepted = recipe();
    const changed = {
      ...accepted,
      assembly: {
        ...accepted.assembly,
        parts: accepted.assembly.parts.map((part, index) => ({
          ...part,
          translation: [index + 10, index + 20, index + 30] as const,
          uniformScale: 0.161 + index * 0.01,
          pivot: [index + 0.01, index + 0.02, index + 0.03] as const,
        })),
      },
    } satisfies ItemAuthoringRecipeV2;

    const applied = applyAcceptedItemAuthoringRecipeV2(draftParts, changed, identity);

    expect(applied.map(({ translation }) => translation)).toEqual([
      [10, 20, 30],
      [11, 21, 31],
      [12, 22, 32],
    ]);
    expect(applied.map(({ uniformScale }) => uniformScale)).toEqual([0.161, 0.171, 0.181]);
    expect(applied.map(({ pivot }) => pivot)).toEqual([
      [0.01, 0.02, 0.03],
      [1.01, 1.02, 1.03],
      [2.01, 2.02, 2.03],
    ]);
  });

  it("fails closed when any source or output identity differs", () => {
    const wrongIdentity: ItemAuthoringIdentityV2 = {
      ...identity,
      sources: {
        ...identity.sources,
        ModelPart2: { ...identity.sources.ModelPart2, sha256: sha("f") },
      },
    };
    expect(() => applyAcceptedItemAuthoringRecipeV2(
      draftParts,
      recipe(),
      wrongIdentity,
    )).toThrow(/identity/i);
  });

  it("reports presentation scopes independently while assembly invalidates every context", () => {
    const before = recipe();
    expect(diffItemAuthoringScopesV2(before, {
      ...before,
      contexts: {
        ...before.contexts,
        GROUND: { bearingDegrees: 90 },
      },
    })).toEqual(["GROUND"]);

    expect(diffItemAuthoringScopesV2(before, {
      ...before,
      assembly: {
        ...before.assembly,
        parts: before.assembly.parts.map((part, index) => index === 1
          ? { ...part, translation: [9, 9, 9] }
          : part),
      },
    })).toEqual([
      "ASSEMBLY",
      "ITEM_PROPERTIES",
      "GROUND",
      "EQUIPPED",
      "INVENTORY_ICON",
    ]);
  });

  it("requires candidate-bound technical, semantic and owner review", () => {
    const incomplete: ItemAuthoringRecipeV2 = {
      ...recipe(),
      review: {
        ...recipe().review,
        semanticChecks: recipe().review.semanticChecks.map((check, index) => index === 2
          ? { ...check, status: "NOT_EVALUATED" }
          : check),
      },
    };
    const result = validateItemAuthoringRecipeV2(incomplete);

    expect(result.ok).toBe(false);
    expect(result.issues).toContain("semantic check TRIGGER_DOWN is not PASSED");
  });

  it("freezes manual editor Q/T/S, pivots and source hashes without rerunning auto-fit", () => {
    const baseline = {
      schemaVersion: 4,
      algorithm: "ITEM_REFERENCE_SLOT_FRAME_FIT_V1",
      solutionSha256: sha("c"),
      parts: draftParts.map((part, index) => ({
        field: part.field,
        sourceSha256: Object.values(identity.sources)[index].sha256,
      })),
    } as unknown as ItemFitReportV4;

    const snapshot = buildItemManualFitSnapshotV2(draftParts, baseline);

    expect(snapshot.baselineFitSolutionSha256).toBe(sha("c"));
    expect(snapshot.parts[1]).toMatchObject({
      field: "ModelPart2",
      sourceSha256: identity.sources.ModelPart2.sha256,
      pivot: [1.3, 1.4, 1.5],
    });
    expect(snapshot.parts[1].uniformScale).toBeCloseTo(0.3, 12);
  });

  it("produces deterministic recipe hashes and binds presentation changes", async () => {
    const first = await hashItemAuthoringRecipeV2(recipe());
    const second = await hashItemAuthoringRecipeV2(recipe());
    const changed = await hashItemAuthoringRecipeV2({
      ...recipe(),
      contexts: { ...recipe().contexts, GROUND: { bearingDegrees: 90 } },
    });

    expect(first).toMatch(/^[0-9a-f]{64}$/);
    expect(second).toBe(first);
    expect(changed).not.toBe(first);
  });
});
