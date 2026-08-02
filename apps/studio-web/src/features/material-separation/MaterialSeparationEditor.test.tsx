// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import type {
  ModelComponentInspectionBootstrapV1,
  ModelMaterialResolutionV1,
  ModelMaterialSeparationDocumentV1,
} from "./types";

vi.mock("../preview/SourceViewport", () => ({
  SourceViewport: ({ componentOverlays }: { componentOverlays: unknown[] }) => (
    <div data-testid="material-overlay">{componentOverlays.length}</div>
  ),
}));

import { MaterialSeparationEditor } from "./MaterialSeparationEditor";

const roots: Root[] = [];
(globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

async function render(element: React.ReactNode) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  await act(async () => root.render(element));
  return container;
}

async function click(element: Element | null) {
  if (!(element instanceof HTMLElement)) throw new Error("missing click target");
  await act(async () => element.click());
}

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

const sourceSha256 = "a".repeat(64);
const bootstrap: ModelComponentInspectionBootstrapV1 = {
  schemaVersion: 1,
  capabilities: {
    schemaVersion: 1,
    target: "CREATURE",
    materialSeparationSupported: true,
    maxMaterialSlots: 256,
    maxOutputSections: 4096,
    selectionGranularity: "CONNECTED_COMPONENTS",
    faceSelectionSupported: false,
    automaticMaterialInference: false,
    preservesSourceUv0: true,
  },
  inventory: {
    schemaVersion: 1,
    sourceSha256,
    sceneId: 0,
    renderNodeCount: 1,
    primitiveInstanceCount: 1,
    triangleCount: 2,
    components: [0, 1].map((componentIndex) => ({
      key: { sceneId: 0, nodeId: 0, primitiveId: 0, componentIndex },
      sourceMaterialId: 0,
      sourceMaterialName: "Source",
      triangleCount: 1,
      vertexCount: 3,
      boundsMin: [componentIndex * 2, 0, 0] as const,
      boundsMax: [componentIndex * 2 + 1, 1, 1] as const,
    })),
  },
  document: { schemaVersion: 1, sourceSha256, materials: [], assignments: [] },
};
const resolution: ModelMaterialResolutionV1 = {
  schemaVersion: 1,
  capabilities: bootstrap.capabilities,
  report: {
    schemaVersion: 1,
    sourceSha256,
    separationSha256: "b".repeat(64),
    sourceComponentCount: 2,
    assignedComponentCount: 0,
    unassignedComponentCount: 2,
    sourceTriangleCount: 2,
    outputTriangleCount: 2,
    sourceVertexCount: 6,
    outputVertexCount: 6,
    duplicatedBoundaryVertexCount: 0,
    outputSectionCount: 1,
    predictedTextureCount: 1,
    warnings: ["MATERIAL-SEPARATION-UV0-UNCHANGED"],
    materialSlots: [{
      materialSlot: 0,
      authoredMaterialId: "source:0",
      displayName: "Source",
      previewColor: "#808080",
      sourceMaterialId: 0,
      sourceMaterialName: "Source",
      sourceImageSha256: "c".repeat(64),
      systemSourceMaterial: true,
      componentCount: 2,
      triangleCount: 2,
    }],
  },
  textureAuthoring: {
    schemaVersion: 1,
    sourceSha256,
    separationSha256: "b".repeat(64),
    bindings: [],
  },
};

describe("MaterialSeparationEditor", () => {
  it("supports the complete V1 edit lifecycle and publishes only on Apply", async () => {
    const applied: unknown[] = [];
    const previews: unknown[] = [];
    const container = await render(
      <MaterialSeparationEditor
        file={new File(["glb"], "ship.glb", { type: "model/gltf-binary" })}
        sourceSha256={sourceSha256}
        bootstrap={bootstrap}
        resolution={resolution}
        onPreviewDocumentChange={(document) => previews.push(document)}
        onApply={(document) => applied.push(document)}
      />,
    );
    expect(container.textContent).toContain("Slots 1 / 256");
    expect(container.textContent).toContain("Sections 1");
    expect(container.textContent).toContain("Textures 1");
    expect(container.textContent).toContain("MATERIAL-SEPARATION-UV0-UNCHANGED");
    expect(container.querySelector("[data-testid='material-overlay']")?.textContent).toBe("2");

    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Select node 0") ?? null);
    expect(container.querySelectorAll("[data-selected='true']")).toHaveLength(2);
    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Select primitive 0/0") ?? null);
    expect(container.querySelectorAll("[data-selected='true']")).toHaveLength(2);

    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "New Material") ?? null);
    const materialAfterCreate = (previews.at(-1) as ModelMaterialSeparationDocumentV1).materials[0];
    expect(materialAfterCreate.sourceFallbackMaterialId).toBe(0);
    expect(materialAfterCreate.sourceFallbackImageSha256).toBe("c".repeat(64));
    await click([...container.querySelectorAll(".material-separation__components button")][0]);
    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Assign Selection") ?? null);
    expect(previews.length).toBeGreaterThan(1);
    expect(applied).toHaveLength(0);

    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Apply") ?? null);
    expect(applied).toHaveLength(1);
    expect((applied[0] as { assignments: unknown[] }).assignments).toHaveLength(1);

    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Unassign to Source") ?? null);
    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Cancel") ?? null);
    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Select Unassigned") ?? null);
    expect(container.querySelectorAll("[data-selected='true']")).toHaveLength(1);

    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Reset") ?? null);
    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Undo") ?? null);
    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Redo") ?? null);
    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Material ID Colors") ?? null);
    expect(container.querySelector("[data-testid='material-overlay']")?.textContent).toBe("0");
    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "Isolate") ?? null);
  });

  it("requires an override before Apply when the source has no exact texture fallback", async () => {
    const withoutFallback: ModelMaterialResolutionV1 = {
      ...resolution,
      report: {
        ...resolution.report,
        materialSlots: [{
          ...resolution.report.materialSlots[0],
          sourceMaterialId: null,
          sourceImageSha256: null,
        }],
      },
    };
    const container = await render(
      <MaterialSeparationEditor
        file={new File(["glb"], "ship.glb", { type: "model/gltf-binary" })}
        sourceSha256={sourceSha256}
        bootstrap={bootstrap}
        resolution={withoutFallback}
        onApply={() => undefined}
      />,
    );

    await click([...container.querySelectorAll("button")].find((button) => button.textContent === "New Material") ?? null);
    expect(container.textContent).toContain("MATERIAL-SEPARATION-TEXTURE-OVERRIDE-REQUIRED");
    const apply = [...container.querySelectorAll("button")].find((button) => button.textContent === "Apply");
    expect((apply as HTMLButtonElement | undefined)?.disabled).toBe(true);
  });
});
