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
    onSeams,
  }: {
    parts: Array<{ field: string; sourceKind: string }>;
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
    return <div data-testid="mock-item-viewport" />;
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

class FakeItemClient implements ItemWorkerClient {
  readonly requests: StudioWorkerRequest[] = [];

  async request(request: StudioWorkerRequest): Promise<StudioWorkerResponse> {
    this.requests.push(request);
    if (request.type === "INSPECT_ITEM_BASEITEMS") {
      return {
        requestId: request.requestId,
        ok: true,
        type: "ITEM_BASEITEMS_INSPECTED",
        catalogJson: catalogJson(),
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
          iconLayerMode: "GEOMETRY_RASTER_TGA_V2",
          iconRuntimeParity: "offline_semantic_readback_only",
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

    await act(async () => button(container, "Continue to Prepare Item")?.click());
    expect(container.textContent).toContain("Meshy sources become independent MDLs");
    expect(container.textContent).toContain("ModelPart1 = 1");
    expect(container.textContent).toContain("ModelPart2 = 1");
    expect(container.textContent).toContain("ModelPart3 = 1");
    expect(container.textContent).toContain("Numeric UTI fields");
    expect(container.textContent).toContain("PREVIEW GAP");
    expect(container.textContent).toContain("Preview AABB is advisory");
    expect(container.querySelector('input[aria-label="UTI Cost"]')).not.toBeNull();
    expect(container.querySelector('input[aria-label="UTI StackSize"]')).not.toBeNull();

    await act(async () => button(container, "Continue to Build")?.click());
    expect(container.textContent).toContain("sw_b_001.mdl");
    expect(container.textContent).toContain("sw_m_001.mdl");
    expect(container.textContent).toContain("sw_t_001.mdl");

    await act(async () => {
      button(container, "Build Item package")?.click();
      await new Promise((resolve) => window.setTimeout(resolve, 0));
    });

    const build = client.requests.find((request) => request.type === "BUILD_ITEM_PACKAGE");
    expect(build).toMatchObject({
      type: "BUILD_ITEM_PACKAGE",
      baseItem: 4,
      parts: [
        { field: "ModelPart1", variant: 1, modelResref: "sw_b_001", iconResref: "isw_b_001" },
        { field: "ModelPart2", variant: 1, modelResref: "sw_m_001", iconResref: "isw_m_001" },
        { field: "ModelPart3", variant: 1, modelResref: "sw_t_001", iconResref: "isw_t_001" },
      ],
    });
    if (!build || build.type !== "BUILD_ITEM_PACKAGE") throw new Error("Item build was not emitted");
    expect(JSON.parse(build.blueprintJson)).toMatchObject({
      parts: [
        { field: "ModelPart1", value: 1 },
        { field: "ModelPart2", value: 1 },
        { field: "ModelPart3", value: 1 },
      ],
      cost: 0,
      stackSize: 1,
    });
    expect(container.textContent).toContain("3authored Meshy MDL parts");
    expect(container.textContent).toContain("0retail numeric selectors");
    expect(container.textContent).toContain("3authored 2D icon layers");
    expect(container.textContent).toContain("Not ready for owner proof");
    expect(container.textContent).toContain("modelVisibility=not_tested");
  });
});
