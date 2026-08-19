// @vitest-environment jsdom

import { act } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";
import { ItemReview } from "./ItemWorkflow";
import type { ItemBuildSnapshot } from "./types";

const sha = (character: string) => character.repeat(64);

function snapshot(): ItemBuildSnapshot {
  return {
    report: {
      status: "OFFLINE_ITEM_PACKAGE_PASSED",
      baseItem: 113,
      partCount: 3,
      meshyPartCount: 3,
      referenceSelectorCount: 0,
      iconLayerCount: 3,
      iconLayerMode: "AURORA_MODELTYPE2_ICON_LAYERS_V3",
      iconRuntimeParity: "offline_native_layer_composite_validated",
      weaponColorwayCoverage: {
        status: "COMPLETE",
        expectedResourceCount: 12,
        emittedResourceCount: 12,
        colors: [1, 2, 3, 4],
        geometryReuse: "ONE_MESHY_GLB_PER_PART",
      },
      itemPropertiesModelConformance: {
        status: "PASSED",
        algorithm: "ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V2",
        axialTargetAxis: 2,
        checkedMdlCount: 12,
        appendOrder: ["ModelPart1", "ModelPart2", "ModelPart3"],
        colorways: [],
      },
      itemIconConformance: {
        status: "PASSED",
        algorithm: "AURORA_MODELTYPE2_ICON_LAYER_COMPOSITE_V3",
        layoutProfile: "LONG_VERTICAL_PART_ORDER_V2",
        colorways: [],
      },
      triangleBudget: { triangleCount: 28_620, triangleBudget: 300_000, warning: false },
      seamValidation: { tolerance: 0.005, status: "PASSED", results: [] },
      modelVisibility: "not_tested",
      proofCompleteness: "missing",
      readyForOwnerProof: false,
      proofBlocker: "Native installation and owner proof remain pending.",
      hakSha256: sha("b"),
      moduleSha256: sha("a"),
      customWeaponBaseItem: {
        schemaVersion: 3,
        status: "APPENDED_STANDALONE_EXACT",
        outputBaseItem: 113,
        label: "hextech_shotgun",
        itemClass: "WHxSh",
        invSlotWidth: 2,
        invSlotHeight: 4,
        definitionSource: "EXPLICIT_COLUMN_ASSIGNMENTS",
        sourceSha256: sha("c"),
        outputSha256: sha("d"),
      },
      proofModule: {
        schemaVersion: 3,
        fixtureProfile: "ITEM_AND_EQUIPPED_MODELTYPE2_PARTS_V3",
        groundItemCount: 1,
        equippedItemCount: 1,
        outputSha256: sha("a"),
        semanticReadbackStatus: "PASS",
        modelVisibility: "not_tested",
        proofCompleteness: "missing",
      },
    },
    partReadbacks: ["ModelPart1", "ModelPart2", "ModelPart3"].map((field, index) => ({
      field,
      variant: 53,
      modelResref: `whxsh_${["b", "m", "t"][index]}_053`,
      readback: { nodeTree: { nodeCount: 2 }, diagnostics: [] },
    })),
    utiReport: {
      baseItem: 113,
      modelType: 2,
      partCount: 3,
      colorFieldCount: 0,
      weaponColorSelectorCount: 3,
      semanticReadbackStatus: "PASS",
    },
    artifacts: [],
  };
}

describe("ItemReview exact candidate gate", () => {
  it("requires all five views, six semantics and owner acceptance", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    const onDownload = vi.fn();
    await act(async () => root.render(
      <ItemReview
        snapshot={snapshot()}
        requiresSemanticReview
        onDownload={onDownload}
        onError={vi.fn()}
      />,
    ));

    const continueButton = Array.from(container.querySelectorAll("button"))
      .find((button) => button.textContent?.includes("Continue to Download"))!;
    expect(continueButton.disabled).toBe(true);
    expect(container.querySelectorAll('input[aria-label^="Review "]')).toHaveLength(5);
    expect(container.querySelectorAll('input[aria-label^="Pass "]')).toHaveLength(6);

    for (const input of container.querySelectorAll<HTMLInputElement>(
      'input[aria-label^="Review "], input[aria-label^="Pass "]',
    )) {
      await act(async () => input.click());
    }
    const acceptance = container.querySelector<HTMLInputElement>(
      'input[aria-label="Accept exact offline composition"]',
    )!;
    expect(acceptance.disabled).toBe(false);
    await act(async () => acceptance.click());

    expect(continueButton.disabled).toBe(false);
    expect(container.textContent).toContain("OFFLINE REVIEW ACCEPTED");
    await act(async () => continueButton.click());
    expect(onDownload).toHaveBeenCalledOnce();

    await act(async () => container.querySelector<HTMLInputElement>(
      'input[aria-label="Pass TRIGGER_DOWN"]',
    )!.click());
    expect(continueButton.disabled).toBe(true);
    expect(acceptance.checked).toBe(false);

    await act(async () => root.unmount());
    container.remove();
  });
});
