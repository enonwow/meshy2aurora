// @vitest-environment jsdom

import { act, useEffect, type ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { StudioWorkerRequest, StudioWorkerResponse } from "../../worker/types";
import { ItemWorkflow } from "./ItemWorkflow";
import type { ItemWorkerClient } from "./types";

vi.mock("./ItemCompositionViewport", () => ({
  ItemCompositionViewport: ({
    parts,
    referenceProfile,
    showReference,
    onSeams,
  }: {
    parts: Array<{ field: string; sourceKind: string }>;
    referenceProfile?: { slots: unknown[] };
    showReference?: boolean;
    onSeams: (results: unknown[]) => void;
  }) => {
    useEffect(() => {
      const fields = parts
        .filter((part) => part.sourceKind === "MESHY_GLB")
        .map((part) => part.field);
      onSeams(fields.slice(1).map((field, index) => ({
        firstField: fields[index],
        secondField: field,
        status: "GAP",
        gap: 1,
        overlapVolume: 0,
      })));
    }, [onSeams, parts]);
    return <div data-testid="mock-item-viewport" data-reference-visible={showReference} data-reference-slots={referenceProfile?.slots.length ?? 0} />;
  },
}));

const roots: Root[] = [];

function localFile(name: string, marker: number): File {
  return {
    name,
    size: 1,
    type: name.endsWith(".glb") ? "model/gltf-binary" : "text/plain",
    lastModified: marker,
    arrayBuffer: async () => new Uint8Array([marker]).buffer,
  } as File;
}

function catalogJson() {
  return JSON.stringify({
    schemaVersion: 1,
    sourceSha256: "a".repeat(64),
    physicalRowCount: 1,
    inactiveRowCount: 0,
    rows: [{
      schemaVersion: 1,
      baseItem: 4,
      label: "Three-part test item",
      itemClass: "sw",
      modelType: 2,
      minRange: 10,
      maxRange: 100,
      genderSpecific: false,
      defaultModel: "it_bag",
      defaultIcon: "isw",
      equipableSlots: 1,
      invSlotWidth: 2,
      invSlotHeight: 3,
      capability: {
        schemaVersion: 1,
        compositionProfile: "BOTTOM_MIDDLE_TOP",
        textureProfile: "DIRECT_COLOR",
        iconProfile: "STANDARD",
        meshySourceCount: 3,
        requiredReferenceTables: [],
      },
      partSlots: [
        { index: 0, field: "ModelPart1", label: "Bottom", token: "b", sourceKind: "MESHY_GLB", referenceTable: null, requiresExplicitResourceResrefs: false },
        { index: 1, field: "ModelPart2", label: "Middle", token: "m", sourceKind: "MESHY_GLB", referenceTable: null, requiresExplicitResourceResrefs: false },
        { index: 2, field: "ModelPart3", label: "Top", token: "t", sourceKind: "MESHY_GLB", referenceTable: null, requiresExplicitResourceResrefs: false },
      ],
      colorFields: [],
    }],
  });
}

function hextechDonorCatalogJson() {
  const catalog = JSON.parse(catalogJson()) as {
    physicalRowCount: number;
    rows: Array<Record<string, unknown>>;
  };
  catalog.physicalRowCount = 113;
  catalog.rows[0] = {
    ...catalog.rows[0],
    baseItem: 6,
    label: "heavy_crossbow",
    itemClass: "WBwXh",
    invSlotWidth: 2,
    invSlotHeight: 4,
  };
  return JSON.stringify(catalog);
}

class FakeItemClient implements ItemWorkerClient {
  readonly requests: StudioWorkerRequest[] = [];

  constructor(private readonly inspectedCatalogJson = catalogJson()) {}

  async request(request: StudioWorkerRequest): Promise<StudioWorkerResponse> {
    this.requests.push(request);
    if (request.type === "INSPECT_ITEM_BASEITEMS") {
      return {
        requestId: request.requestId,
        ok: true,
        type: "ITEM_BASEITEMS_INSPECTED",
        catalogJson: this.inspectedCatalogJson,
      };
    }
    if (request.type === "BUILD_ITEM_ATTACHMENT_PROFILE") {
      return {
        requestId: request.requestId,
        ok: true,
        type: "ITEM_ATTACHMENT_PROFILE_BUILT",
        attachmentProfileJson: JSON.stringify({
          schemaVersion: 1,
          algorithm: "AURORA_ITEM_REFERENCE_PROFILE_V1",
          status: "PASSED",
          identity: {
            schemaVersion: 1,
            resourceContextSha256: "c".repeat(64),
            baseitemsSha256: "a".repeat(64),
            baseItem: request.baseItem,
            itemClass: request.baseItem === 6 ? "WBwXh" : "sw",
            modelType: 2,
            referenceKind: "EXPLICIT_VARIANTS",
            referenceId: request.referenceId,
          },
          attachmentRoute: "HAND",
          equipableSlots: 1,
          commonOrigin: [0, 0, 0],
          axialAxis: 1,
          widthAxis: 2,
          depthAxis: 0,
          attachmentZoneMin: [-0.1, -0.1, -0.1],
          attachmentZoneMax: [0.1, 0.1, 0.1],
          attachmentEvidence: "ORIGIN_CONTAINING_REFERENCE_PARTS_V1",
          slots: request.models.map((model, index) => ({
            field: model.field,
            label: ["Bottom", "Middle", "Top"][index],
            token: ["b", "m", "t"][index],
            modelResref: model.modelResref,
            modelSha256: String(index + 4).repeat(64),
            controllerNodeName: `g_${model.modelResref}`,
            controllerTranslation: [0, [0, 0.29, 0.38][index], 0],
            controllerRotationXyzw: [0, 0, 0, 1],
            boundsMin: [-0.1, [0, 0.29, 0.38][index], -0.1],
            boundsMax: [0.1, [0.3, 0.39, 0.98][index], 0.1],
            allowAxialExtensionAtMin: false,
            allowAxialExtensionAtMax: index === 2,
          })),
          profileSha256: "p".repeat(64),
        }),
      };
    }
    if (request.type === "FIT_ITEM_PARTS") {
      const axialScaleFactors = request.targetAxialScaleFactors ?? [1, 1, 1];
      const baseLengths = [0.3, 0.1, 0.6];
      return {
        requestId: request.requestId,
        ok: true,
        type: "ITEM_PARTS_FITTED",
        fitReportJson: JSON.stringify({
          schemaVersion: 4,
          algorithm: request.manualFit
            ? "ITEM_REFERENCE_MANUAL_FIT_V2"
            : "ITEM_REFERENCE_SLOT_FRAME_FIT_V1",
          status: "PASSED",
          tolerance: request.tolerance,
          iterations: 1,
          solutionSha256: "f".repeat(64),
          referenceProfileSha256: "p".repeat(64),
          commonOrigin: [0, 0, 0],
          orientationFrame: {
            sourceAxialAxis: 2,
            sourceWidthAxis: 0,
            sourceDepthAxis: 1,
            targetAxialAxis: 1,
            targetWidthAxis: 2,
            targetDepthAxis: 0,
            widthToDepthRatio: 4,
            handednessDeterminant: 1,
            evidence: "GROUP_NORMALIZED_TRANSVERSE_EXTENTS_WITH_PROPER_HANDEDNESS_V1",
            status: "PASSED",
          },
          parts: request.parts.map((part, index) => {
            const authored = request.manualFit?.parts[index];
            const targetAxialLength = authored?.uniformScale
              ?? baseLengths[index] * axialScaleFactors[index];
            const axialMin = [0, 0.29, 0.38][index];
            return ({
            field: part.field,
            sourceSha256: String(index + 1).repeat(64),
            sourceNode: part.sourceNode,
            triangleCount: 12,
            inputBoundsMin: [0, 0, 0],
            inputBoundsMax: [1, 1, 1],
            axialSourceAxis: 1,
            axialTargetAxis: 1,
            targetAxialLength,
            transform: {
              translation: authored?.translation ?? [0, axialMin, 0],
              rotationXyzw: authored?.rotationXyzw ?? [0, 0, 0, 1],
              uniformScale: authored?.uniformScale ?? targetAxialLength,
              pivot: authored?.pivot ?? [0, 0, 0],
            },
            targetSpaceScaleXyz: authored?.targetSpaceScaleXyz ?? [1, 1, 1],
            transformSha256: "a".repeat(64),
            outputBoundsMin: [0, axialMin, 0],
            outputBoundsMax: [1, axialMin + targetAxialLength, 1],
            bottomConnector: index === 0 ? null : {
              kind: "BOTTOM",
              axialAxis: 1,
              position: [0.5, [0, 0.29, 0.38][index], 0.5],
            },
            topConnector: index === request.parts.length - 1 ? null : {
              kind: "TOP",
              axialAxis: 1,
              position: [0.5, [0.3, 0.39, 0.98][index], 0.5],
            },
          })}),
          adjacentSeams: request.parts.slice(0, -1).map((part, index) => ({
            firstField: part.field,
            secondField: request.parts[index + 1].field,
            status: "OVERLAP",
            gap: 0,
            overlap: true,
            measurementSha256: "b".repeat(64),
          })),
          adjacentConnectors: request.parts.slice(0, -1).map((part, index) => ({
            firstField: part.field,
            firstConnector: "TOP",
            secondField: request.parts[index + 1].field,
            secondConnector: "BOTTOM",
            axialAxis: 1,
            axialOverlap: 0.01,
            requiredMinOverlap: 0.0075,
            requiredMaxOverlap: 0.0125,
            surfaceStatus: "OVERLAP",
            status: "OVERLAPPING",
          })),
          nonAdjacentMeasurements: [],
        }),
      };
    }
    if (request.type === "BUILD_ITEM_PACKAGE") {
      return {
        requestId: request.requestId,
        ok: true,
        type: "ITEM_PACKAGE_BUILT",
        artifacts: [],
        reportJson: JSON.stringify({
          status: "OFFLINE_ITEM_PACKAGE_PASSED",
          baseItem: 4,
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
            axialTargetAxis: 1,
            checkedMdlCount: 12,
            appendOrder: ["ModelPart1", "ModelPart2", "ModelPart3"],
            colorways: [1, 2, 3, 4].map((color) => ({
              color,
              compositeBoundsMin: [0, 0, 0],
              compositeBoundsMax: [1, 0.98, 1],
              totalTriangleCount: 36,
              parts: ["ModelPart1", "ModelPart2", "ModelPart3"].map((field) => ({
                field,
                rootControllerOwner: "NONE",
                transformControllerOwner: "TRIMESH_CHILD",
                meshNodeNames: [`g_${field.toLowerCase()}`],
              })),
            })),
          },
          itemIconConformance: {
            status: "PASSED",
            algorithm: "AURORA_MODELTYPE2_ICON_LAYER_COMPOSITE_V3",
            layoutProfile: "LONG_VERTICAL_PART_ORDER_V2",
            colorways: [1, 2, 3, 4].map((color) => ({
              color,
              opaquePixelCount: 512,
              boundsMin: [4, 8],
              boundsMaxExclusive: [28, 120],
              axialFillRatio: 0.875,
              occupiedFillRatio: 0.2,
              partOrderStatus: "PASSED",
              compositeRgbaSha256: String(color).repeat(64),
            })),
          },
          triangleBudget: { triangleCount: 1234, triangleBudget: 300000, warning: false },
          seamValidation: {
            tolerance: 0.01,
            status: "PASSED",
            results: [
              {
                firstField: "ModelPart1",
                secondField: "ModelPart2",
                status: "TOUCHING",
                gap: 0,
                overlap: false,
                measurementSha256: "a".repeat(64),
              },
              {
                firstField: "ModelPart2",
                secondField: "ModelPart3",
                status: "TOUCHING",
                gap: 0,
                overlap: false,
                measurementSha256: "b".repeat(64),
              },
            ],
          },
          modelVisibility: "not_tested",
          proofCompleteness: "missing",
          readyForOwnerProof: false,
          proofBlocker: "Owner test module has not been emitted.",
          hakSha256: "b".repeat(64),
          moduleSha256: "a".repeat(64),
          customWeaponBaseItem: null,
          proofModule: {
            schemaVersion: 3,
            fixtureProfile: "ITEM_AND_EQUIPPED_MODELTYPE2_PARTS_V3",
            groundItemCount: 1,
            equippedItemCount: 1,
            outputSha256: "a".repeat(64),
            semanticReadbackStatus: "PASS",
            modelVisibility: "not_tested",
            proofCompleteness: "missing",
          },
        }),
        partReadbacksJson: JSON.stringify(request.parts.map((part) => ({
          field: part.field,
          variant: part.variant,
          modelResref: part.modelResref,
          readback: { nodeTree: { nodeCount: 2 }, diagnostics: [] },
        }))),
        utiReportJson: JSON.stringify({
          baseItem: 4,
          modelType: 2,
          partCount: 3,
          colorFieldCount: 0,
          semanticReadbackStatus: "PASS",
        }),
      };
    }
    return {
      requestId: request.requestId,
      ok: false,
      type: "FAILED",
      message: `Unexpected request ${request.type}`,
    };
  }
}

async function render(element: ReactNode) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  await act(async () => root.render(element));
  return container;
}

async function chooseFile(input: HTMLInputElement, file: File) {
  Object.defineProperty(input, "files", { configurable: true, value: [file] });
  await act(async () => {
    input.dispatchEvent(new window.Event("change", { bubbles: true }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
  });
}

function button(container: HTMLElement, label: string) {
  return Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
    .find((candidate) => candidate.textContent?.includes(label));
}

afterEach(async () => {
  await act(async () => {
    while (roots.length) roots.pop()?.unmount();
  });
  document.body.replaceChildren();
});

describe("ItemWorkflow", () => {
  it("requires an explicit action before projecting donor 6 as output 113", async () => {
    const client = new FakeItemClient(hextechDonorCatalogJson());
    const container = await render(
      <ItemWorkflow client={client} onTargetChange={vi.fn()} />,
    );
    await chooseFile(
      container.querySelector<HTMLInputElement>('input[aria-label="Base items table"]')!,
      localFile("baseitems.2da", 1),
    );

    expect(container.textContent).toContain("BaseItem 6");
    expect(container.textContent).toContain("WBwXh");
    expect(container.textContent).not.toContain("BaseItem 113");

    await act(async () => button(container, "Author Hextech Shotgun V2")?.click());

    expect(container.textContent).toContain("BaseItem 113");
    expect(container.textContent).toContain("WHxSh");
    expect(container.textContent).toContain("64 × 128");
    expect(button(container, "Use retail BaseItem 6")).not.toBeNull();
  });

  it("shows one Item case and builds the exact three-part Aurora recipe", async () => {
    const client = new FakeItemClient();
    const container = await render(
      <ItemWorkflow client={client} onTargetChange={vi.fn()} />,
    );

    expect(container.textContent).toContain("4 formal ModelTypes");
    expect(container.textContent).toContain("1 part");
    expect(container.textContent).toContain("3 parts");
    expect(container.textContent).toContain("19 selectors");
    expect(container.textContent).not.toContain("Weapon");
    expect(container.textContent).not.toContain("Shield");

    const tableInput = container.querySelector<HTMLInputElement>(
      'input[aria-label="Base items table"]',
    );
    expect(tableInput).not.toBeNull();
    await chooseFile(tableInput!, localFile("baseitems.2da", 1));

    expect(client.requests[0]).toMatchObject({ type: "INSPECT_ITEM_BASEITEMS" });
    expect(container.textContent).toContain("ModelType 2");
    expect(container.textContent).toContain("3 exact UTI slots");

    const partInputs = Array.from(container.querySelectorAll<HTMLInputElement>(
      'input[accept=".glb,model/gltf-binary"]',
    ));
    expect(partInputs).toHaveLength(3);
    for (const [index, input] of partInputs.entries()) {
      await chooseFile(input, localFile(`part-${index + 1}.glb`, index + 2));
    }
    const referenceInputs = Array.from(container.querySelectorAll<HTMLInputElement>(
      'input[aria-label$="reference MDL"]',
    ));
    expect(referenceInputs).toHaveLength(3);
    for (const [index, input] of referenceInputs.entries()) {
      await chooseFile(input, localFile(`sw_${["b", "m", "t"][index]}_023.mdl`, index + 10));
    }

    await act(async () => {
      button(container, "Continue to Prepare Item")?.click();
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    expect(container.textContent).toContain("Meshy sources become independent MDLs");
    expect(container.textContent).toContain("ModelPart1 = 1");
    expect(container.textContent).toContain("ModelPart2 = 1");
    expect(container.textContent).toContain("ModelPart3 = 1");
    expect(container.textContent).toContain("Numeric UTI fields");
    const fitRequest = client.requests.find((request) => request.type === "FIT_ITEM_PARTS");
    expect(fitRequest).toMatchObject({ attachmentProfileJson: expect.any(String) });
    expect(container.querySelectorAll('input[aria-label^="Target axial length"]')).toHaveLength(0);
    expect(container.textContent).toContain("Reference slot-frame contract");
    expect(container.textContent).toContain("Origin 0, 0, 0");
    expect(container.querySelector('input[aria-label="Weapon part model"]')).not.toBeNull();
    expect(container.querySelector('select[aria-label="Weapon part color"]')).not.toBeNull();
    expect(container.querySelector('input[aria-label="Part variant"]')).toBeNull();
    expect(container.textContent).toContain("model × 10 + color");
    expect(Array.from(container.querySelectorAll('select[aria-label="Texture encoding"] option')).map((option) => option.getAttribute("value"))).toEqual(["DIRECT_COLOR"]);
    expect(container.textContent).toContain("PREVIEW GAP");
    expect(container.textContent).toContain("Preview AABB is advisory");
    expect(container.querySelector('input[aria-label="UTI Cost"]')).not.toBeNull();
    expect(container.querySelector('input[aria-label="UTI StackSize"]')).not.toBeNull();

    const referenceToggle = container.querySelector<HTMLInputElement>('input[type="checkbox"]');
    expect(referenceToggle?.checked).toBe(true);
    expect(container.querySelector('[data-testid="mock-item-viewport"]')?.getAttribute("data-reference-visible")).toBe("true");
    await act(async () => {
      referenceToggle?.click();
    });
    expect(container.querySelector('[data-testid="mock-item-viewport"]')?.getAttribute("data-reference-visible")).toBe("false");

    expect(container.querySelector<HTMLInputElement>('input[aria-label="Part size percent"]')?.disabled).toBe(false);
    await act(async () => button(container, "Bottom")?.click());
    expect(container.querySelector<HTMLInputElement>('input[aria-label="Part size percent"]')?.disabled).toBe(true);
    expect(container.textContent).toContain("reference locked");
    await act(async () => button(container, "Top")?.click());

    const partSize = container.querySelector<HTMLInputElement>(
      'input[aria-label="Part size percent"]',
    );
    expect(partSize).not.toBeNull();
    expect(partSize?.value).toBe("100");
    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set?.call(partSize, "125");
      partSize?.dispatchEvent(new window.Event("input", { bubbles: true }));
    });
    expect(container.textContent).toContain("Manual change pending");
    expect(button(container, "Continue to Build")?.disabled).toBe(true);

    await act(async () => {
      button(container, "Validate exact fit")?.click();
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const validatedFit = client.requests.filter((request) => request.type === "FIT_ITEM_PARTS").at(-1);
    expect(validatedFit).toMatchObject({
      manualFit: {
        schemaVersion: 2,
        baselineFitSolutionSha256: "f".repeat(64),
        parts: [
          { field: "ModelPart1" },
          { field: "ModelPart2" },
          { field: "ModelPart3", uniformScale: 0.75 },
        ],
      },
    });
    expect(container.querySelector<HTMLInputElement>('input[aria-label="Part size percent"]')?.value).toBe("125");
    expect(container.textContent).toContain("Fit validated");
    expect(button(container, "Continue to Build")?.disabled).toBe(false);

    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set?.call(partSize, "150");
      partSize?.dispatchEvent(new window.Event("input", { bubbles: true }));
    });
    expect(container.querySelector<HTMLInputElement>('input[aria-label="Part size percent"]')?.value).toBe("150");
    await act(async () => button(container, "Discard changes")?.click());
    expect(container.querySelector<HTMLInputElement>('input[aria-label="Part size percent"]')?.value).toBe("125");
    expect(button(container, "Continue to Build")?.disabled).toBe(false);

    await act(async () => button(container, "Reset part")?.click());
    expect(container.querySelector<HTMLInputElement>('input[aria-label="Part size percent"]')?.value).toBe("100");
    expect(button(container, "Continue to Build")?.disabled).toBe(true);
    await act(async () => {
      button(container, "Validate exact fit")?.click();
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });
    const resetFit = client.requests.filter((request) => request.type === "FIT_ITEM_PARTS").at(-1);
    expect(resetFit).toMatchObject({
      manualFit: {
        parts: [
          { field: "ModelPart1" },
          { field: "ModelPart2" },
          { field: "ModelPart3", uniformScale: 0.6 },
        ],
      },
    });
    expect(button(container, "Continue to Build")?.disabled).toBe(false);

    await act(async () => button(container, "Continue to Build")?.click());
    expect(container.textContent).toContain("sw_b_011.mdl");
    expect(container.textContent).toContain("sw_m_011.mdl");
    expect(container.textContent).toContain("sw_t_011.mdl");

    await act(async () => {
      button(container, "Build Item package")?.click();
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });

    const build = client.requests.find((request) => request.type === "BUILD_ITEM_PACKAGE");
    expect(build).toMatchObject({
      type: "BUILD_ITEM_PACKAGE",
      baseItem: 4,
      parts: [
        { field: "ModelPart1", variant: 11, modelResref: "sw_b_011", iconResref: "isw_b_011" },
        { field: "ModelPart2", variant: 11, modelResref: "sw_m_011", iconResref: "isw_m_011" },
        { field: "ModelPart3", variant: 11, modelResref: "sw_t_011", iconResref: "isw_t_011" },
      ],
    });
    if (!build || build.type !== "BUILD_ITEM_PACKAGE") throw new Error("Item build was not emitted");
    expect(build.equippedProofContext).toBeNull();
    expect(build.parts.map((part) => JSON.parse(part.transformJson).rotationXyzw)).toEqual([
      [0, 0, 0, 1],
      [0, 0, 0, 1],
      [0, 0, 0, 1],
    ]);
    expect(build.parts.map((part) => JSON.parse(part.transformJson).translation)).toEqual([
      [0, 0, 0],
      [0, 0.29, 0],
      [0, 0.38, 0],
    ]);
    expect(JSON.parse(build.blueprintJson)).toMatchObject({
      parts: [
        { field: "ModelPart1", value: 11 },
        { field: "ModelPart2", value: 11 },
        { field: "ModelPart3", value: 11 },
      ],
      cost: 0,
      stackSize: 1,
    });
    expect(container.textContent).toContain("3authored Meshy MDL parts");
    expect(container.textContent).toContain("12MDLs: controllerless root + Trimesh transforms");
    expect(container.textContent).toContain("Aurora ModelType 2 append contract");
    expect(container.textContent).toContain("0retail numeric selectors");
    expect(container.textContent).toContain("3authored 2D icon layers");
    expect(container.textContent).toContain("Not ready for owner proof");
    expect(container.textContent).toContain("modelVisibility=not_tested");
  });
});
