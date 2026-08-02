import { describe, expect, it } from "vitest";
import * as THREE from "three";
import {
  allocateItemNamespace,
  eulerDegreesToQuaternion,
  initialItemPartDrafts,
  projectItemBaseitemsCatalog,
  resolveItemPartDraft,
} from "./domain";
import type { ItemBaseItemRow } from "./types";

const armorFields = [
  "ArmorPart_RFoot",
  "ArmorPart_LFoot",
  "ArmorPart_RShin",
  "ArmorPart_LShin",
  "ArmorPart_RThigh",
  "ArmorPart_LThigh",
  "ArmorPart_Pelvis",
  "ArmorPart_Torso",
  "ArmorPart_Belt",
  "ArmorPart_Neck",
  "ArmorPart_RFArm",
  "ArmorPart_LFArm",
  "ArmorPart_RBicep",
  "ArmorPart_LBicep",
  "ArmorPart_RShoul",
  "ArmorPart_LShoul",
  "ArmorPart_RHand",
  "ArmorPart_LHand",
  "ArmorPart_Robe",
] as const;

const colors = [
  "Leather1Color",
  "Leather2Color",
  "Cloth1Color",
  "Cloth2Color",
  "Metal1Color",
  "Metal2Color",
] as const;

function row(
  baseItem: number,
  modelType: 0 | 1 | 2 | 3,
  itemClass: string,
  partSlots: Array<{
    index: number;
    field: string;
    label: string;
    token: string | null;
    sourceKind: "MESHY_GLB" | "CAPART_SELECTION" | "CLOAK_MODEL_SELECTION";
    referenceTable?: string | null;
    requiresExplicitResourceResrefs: boolean;
  }>,
  colorFields: readonly string[],
): ItemBaseItemRow {
  return {
    schemaVersion: 1,
    baseItem,
    label: `Row ${baseItem}`,
    itemClass,
    modelType,
    genderSpecific: modelType === 3,
    defaultModel: "it_bag",
    defaultIcon: `i${itemClass}`,
    equipableSlots: 1,
    invSlotWidth: 2,
    invSlotHeight: 3,
    capability: {
      schemaVersion: 1,
      compositionProfile: modelType === 2
        ? "BOTTOM_MIDDLE_TOP"
        : modelType === 3 ? "CAPART_ARMOR" : "SINGLE_PART",
      textureProfile: modelType === 1
        ? "PALETTE_LAYERS"
        : modelType === 3 ? "CAPART_PALETTE_LAYERS" : "DIRECT_COLOR",
      iconProfile: modelType === 1
        ? "LAYERED"
        : modelType === 3 ? "CAPART_COMPOSITE" : "STANDARD",
      meshySourceCount: modelType === 3 ? 0 : partSlots.length,
      requiredReferenceTables: modelType === 3 ? ["CAPART", "PARTS_ROBE"] : [],
    },
    partSlots: partSlots.map((slot) => ({
      ...slot,
      referenceTable: slot.referenceTable ?? null,
    })),
    colorFields,
  };
}

describe("Item domain projection", () => {
  it("keeps the exact 1, 1, 3 and 19-part ModelType schemas without usage categories", () => {
    const catalog = projectItemBaseitemsCatalog(JSON.stringify({
      schemaVersion: 1,
      sourceSha256: "a".repeat(64),
      physicalRowCount: 4,
      inactiveRowCount: 0,
      rows: [
        row(10, 0, "ring", [{ index: 0, field: "ModelPart1", label: "Model", token: null, sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false }], []),
        row(11, 1, "belt", [{ index: 0, field: "ModelPart1", label: "Model", token: null, sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false }], colors),
        row(12, 2, "sw", [
          { index: 0, field: "ModelPart1", label: "Bottom", token: "b", sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false },
          { index: 1, field: "ModelPart2", label: "Middle", token: "m", sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false },
          { index: 2, field: "ModelPart3", label: "Top", token: "t", sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false },
        ], []),
        row(13, 3, "armor", armorFields.map((field, index) => ({
          index,
          field,
          label: field.slice("ArmorPart_".length),
          token: null,
          sourceKind: "CAPART_SELECTION",
          requiresExplicitResourceResrefs: false,
        })), colors),
      ],
    }));

    expect(catalog.rows.map((value) => value.partSlots.length)).toEqual([1, 1, 3, 19]);
    expect(catalog.rows.map((value) => value.colorFields.length)).toEqual([0, 6, 0, 6]);
    expect(catalog.rows[2].partSlots.map((slot) => [slot.field, slot.token])).toEqual([
      ["ModelPart1", "b"],
      ["ModelPart2", "m"],
      ["ModelPart3", "t"],
    ]);
    expect(catalog.rows[3].partSlots.map((slot) => slot.field)).toEqual(armorFields);
  });

  it("allocates a deterministic collision-free item namespace by advancing variants and suffixes", () => {
    const threePart = row(12, 2, "sw", [
      { index: 0, field: "ModelPart1", label: "Bottom", token: "b", sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false },
      { index: 1, field: "ModelPart2", label: "Middle", token: "m", sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false },
      { index: 2, field: "ModelPart3", label: "Top", token: "t", sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false },
    ], []);
    const allocation = allocateItemNamespace(threePart, initialItemPartDrafts(threePart), [
      "2002:sw_b_001",
      "2025:m2aitmc",
      "hak:m2aihakc",
    ]);

    expect(allocation.parts.map((part) => part.variant)).toEqual([2, 1, 1]);
    expect(allocation.blueprintResref).toBe("m2aitmc1");
    expect(allocation.hakResref).toBe("m2aihakc1");
    expect(allocation.collisionAvoidanceCount).toBe(3);
  });

  it("rejects malformed occupied namespace entries instead of silently dropping them", () => {
    const singlePart = row(12, 0, "sw", [{
      index: 0,
      field: "ModelPart1",
      label: "Model",
      token: null,
      sourceKind: "MESHY_GLB",
      requiresExplicitResourceResrefs: true,
    }], []);

    expect(() => allocateItemNamespace(
      singlePart,
      initialItemPartDrafts(singlePart),
      ["sw_b_001"],
    )).toThrow(/numeric-type:resref/);
  });

  it("keeps DefaultModel as a fallback resref and initializes authoring variants independently", () => {
    const threePart = row(12, 2, "sw", [
      { index: 0, field: "ModelPart1", label: "Bottom", token: "b", sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false },
      { index: 1, field: "ModelPart2", label: "Middle", token: "m", sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false },
      { index: 2, field: "ModelPart3", label: "Top", token: "t", sourceKind: "MESHY_GLB", requiresExplicitResourceResrefs: false },
    ], []);
    const drafts = initialItemPartDrafts(threePart);

    expect(threePart.defaultModel).toBe("it_bag");
    expect(drafts.map((draft) => draft.variant)).toEqual([1, 1, 1]);
    expect(drafts.map((draft, index) => resolveItemPartDraft(threePart, draft, index).modelResref))
      .toEqual(["sw_b_001", "sw_m_001", "sw_t_001"]);
    expect(drafts.map((draft, index) => resolveItemPartDraft(threePart, draft, index).iconResref))
      .toEqual(["isw_b_001", "isw_m_001", "isw_t_001"]);
  });

  it("keeps CAPART armor as numeric selectors instead of inventing Meshy part resrefs", () => {
    const armor = row(13, 3, "armor", [{
      index: 0,
      field: armorFields[0],
      label: "RFoot",
      token: null,
      sourceKind: "CAPART_SELECTION",
      requiresExplicitResourceResrefs: false,
    }], colors);
    const draft = initialItemPartDrafts(armor)[0];

    expect(draft.sourceKind).toBe("CAPART_SELECTION");
    expect(armor.capability.meshySourceCount).toBe(0);
  });

  it("allocates a separate collision-safe UTC resref for an equipped CAPART proof", () => {
    const armor = row(16, 3, "armor", armorFields.map((field, index) => ({
      index,
      field,
      label: field.slice("ArmorPart_".length),
      token: null,
      sourceKind: "CAPART_SELECTION",
      requiresExplicitResourceResrefs: false,
    })), colors);
    const allocation = allocateItemNamespace(
      armor,
      initialItemPartDrafts(armor),
      ["2027:m2ainpcg"],
    );

    expect(allocation.proofCreatureResref).toBe("m2ainpcg1");
    expect(allocation.collisionAvoidanceCount).toBe(1);
  });

  it("uses the exact Three.js XYZ Euler convention sent by the visual preview", () => {
    const degrees: [number, number, number] = [37, -23, 71];
    const expected = new THREE.Quaternion().setFromEuler(new THREE.Euler(
      ...degrees.map((value) => value * Math.PI / 180) as [number, number, number],
      "XYZ",
    ));

    expect(eulerDegreesToQuaternion(degrees)).toEqual([
      expected.x,
      expected.y,
      expected.z,
      expected.w,
    ]);
  });
});
