// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { PlaceableAuthoringBootstrap, PlaceableAuthoringDocument } from "./types";

vi.mock("./PlaceableAuthoringViewport", () => ({
  PlaceableAuthoringViewport: ({ editable }: { editable: boolean }) => (
    <div data-testid="placeable-3d-viewport">{editable ? "edited-3d" : "original-3d"}</div>
  ),
}));

import { PlaceableAuthoringEditor } from "./PlaceableAuthoringEditor";

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

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

const bootstrap: PlaceableAuthoringBootstrap = {
  schemaVersion: 2,
  inspection: {
    schemaVersion: 1,
    sourceSha256: "a".repeat(64),
    renderNodeCount: 1,
    primitiveCount: 1,
    connectedComponentCount: 2,
    nodes: [{
      elementId: "node:0",
      nodeId: 0,
      name: "reactor",
      meshId: 0,
      primitives: [{
        primitiveId: 0,
        materialId: 0,
        triangleCount: 2,
        components: [
          { elementId: "component:0", componentIndex: 0, triangleCount: 1, vertexCount: 3, boundsMin: [0, 0, 0], boundsMax: [1, 1, 1] },
          { elementId: "component:1", componentIndex: 1, triangleCount: 1, vertexCount: 3, boundsMin: [1.2, 0, 0], boundsMax: [2.2, 1, 1] },
        ],
      }],
    }],
  },
  document: {
    schemaVersion: 2,
    sourceSha256: "a".repeat(64),
    elements: [{
      id: "node:0",
      name: "reactor",
      kind: "SOURCE_NODE",
      source: { nodeId: 0, primitiveId: null, componentIndex: null },
      parentId: null,
      transform: {
        translation: [0, 0, 0],
        rotationXyzw: [0, 0, 0, 1],
        scale: [1, 1, 1],
        pivot: [0, 0, 0],
      },
      flags: {
        hidden: false,
        locked: false,
        renderable: true,
        includeInCollision: true,
        castShadow: true,
      },
      deleted: false,
    }],
    collision: {
      schemaVersion: 1,
      mode: "AUTO_RECTANGLE",
      coordinateSpace: "GLTF_SOURCE_XZ_METERS",
      paddingMeters: 0,
      vertices: [],
    },
  },
};

describe("PlaceableAuthoringEditor", () => {
  it("exposes standard 3D authoring controls and publishes structural and export-flag edits", async () => {
    const documents: PlaceableAuthoringDocument[] = [];
    const container = await render(
      <PlaceableAuthoringEditor
        file={new File(["glb"], "reactor.glb", { type: "model/gltf-binary" })}
        sourceSha256={bootstrap.document.sourceSha256}
        bootstrap={bootstrap}
        onDocumentChange={(document) => documents.push(document)}
      />,
    );

    expect(container.textContent).toContain("Outliner");
    expect(container.textContent).toContain("World");
    expect(container.textContent).toContain("Local");
    expect(container.textContent).toContain("Surface");
    expect(container.textContent).toContain("Vertex");
    expect(container.querySelector('[data-testid="placeable-3d-viewport"]')?.textContent).toBe("edited-3d");

    const reactor = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent?.includes("reactor") && button.textContent !== "Split");
    await act(async () => reactor?.click());
    expect(container.textContent).toContain("Set Origin");
    expect(container.querySelector<HTMLSelectElement>('select[aria-label="Align mode"]')?.value).toBe("CENTER");
    expect(container.textContent).toContain("Min");
    expect(container.textContent).toContain("Max");

    const split = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent === "Split");
    await act(async () => split?.click());
    expect(container.textContent).toContain("reactor · P0 C0");
    expect(container.textContent).toContain("reactor · P0 C1");

    const collision = container.querySelector<HTMLInputElement>('input[type="checkbox"]');
    const collisionInputs = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]'));
    expect(collision).not.toBeNull();
    await act(async () => collisionInputs[2]?.click());
    expect(documents.at(-1)?.elements.at(-1)?.flags.includeInCollision).toBe(false);

    const duplicate = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent === "Duplicate");
    await act(async () => duplicate?.click());
    expect(documents.at(-1)?.elements.filter((element) => element.kind === "COPY")).toHaveLength(2);
  });

  it("switches from automatic PWK to an editable custom polygon", async () => {
    const documents: PlaceableAuthoringDocument[] = [];
    const container = await render(
      <PlaceableAuthoringEditor
        file={new File(["glb"], "reactor.glb")}
        sourceSha256={bootstrap.document.sourceSha256}
        bootstrap={bootstrap}
        resolvedCollision={{
          schemaVersion: 1,
          mode: "AUTO_RECTANGLE",
          sourceCoordinateSpace: "GLTF_SOURCE_XZ_METERS",
          outputCoordinateSpace: "AURORA_XY_METERS",
          inputVertices: [[-1, -1], [1, -1], [1, 1], [-1, 1]],
          sourceVertices: [[-1, -1], [1, -1], [1, 1], [-1, 1]],
          vertices: [[-1, -1], [1, -1], [1, 1], [-1, 1]],
          triangles: [[0, 1, 2], [0, 2, 3]],
          boundsMin: [-1, -1],
          boundsMax: [1, 1],
          surfaceId: 7,
          authoringSha256: "b".repeat(64),
          collisionSha256: "c".repeat(64),
          pwkSha256: "d".repeat(64),
        }}
        onDocumentChange={(document) => documents.push(document)}
      />,
    );
    const mode = container.querySelector<HTMLSelectElement>('select[aria-label="Collision mode"]')!;
    await act(async () => {
      mode.value = "CUSTOM_POLYGON";
      mode.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(documents.at(-1)?.collision).toMatchObject({
      mode: "CUSTOM_POLYGON",
      vertices: [[-1, -1], [1, -1], [1, 1], [-1, 1]],
    });
    expect(container.querySelectorAll(".placeable-authoring__collision-vertices li")).toHaveLength(4);
    expect(container.textContent).toContain("GLTF X/Z → Aurora X/Y");
  });

  it("switches between immutable original and edited viewport modes", async () => {
    const container = await render(
      <PlaceableAuthoringEditor
        file={new File(["glb"], "reactor.glb")}
        sourceSha256={bootstrap.document.sourceSha256}
        bootstrap={bootstrap}
        onDocumentChange={() => undefined}
      />,
    );
    const original = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent === "Original");
    await act(async () => original?.click());
    expect(container.querySelector('[data-testid="placeable-3d-viewport"]')?.textContent).toBe("original-3d");
  });

  it("virtualizes a large component outliner", async () => {
    const largeBootstrap: PlaceableAuthoringBootstrap = {
      ...bootstrap,
      document: {
        ...bootstrap.document,
        elements: Array.from({ length: 4_000 }, (_, index) => ({
          ...bootstrap.document.elements[0],
          id: `component:${index}`,
          name: `component ${index}`,
          kind: "SOURCE_COMPONENT" as const,
          source: { nodeId: 0, primitiveId: 0, componentIndex: index % 2 },
        })),
      },
    };
    const container = await render(
      <PlaceableAuthoringEditor
        file={new File(["glb"], "large.glb")}
        sourceSha256={largeBootstrap.document.sourceSha256}
        bootstrap={largeBootstrap}
        onDocumentChange={() => undefined}
      />,
    );
    expect(container.textContent).toContain("4000 elements");
    expect(container.querySelectorAll(".placeable-authoring__tree li").length).toBeLessThan(50);
  });

  it("requires confirmation before splitting more than 512 connected components", async () => {
    const largeBootstrap: PlaceableAuthoringBootstrap = {
      ...bootstrap,
      inspection: {
        ...bootstrap.inspection,
        connectedComponentCount: 513,
        nodes: [{
          ...bootstrap.inspection.nodes[0],
          primitives: [{
            ...bootstrap.inspection.nodes[0].primitives[0],
            triangleCount: 513,
            components: Array.from({ length: 513 }, (_, index) => ({
              elementId: `component:${index}`,
              componentIndex: index,
              triangleCount: 1,
              vertexCount: 3,
              boundsMin: [index, 0, 0] as [number, number, number],
              boundsMax: [index + 0.5, 0.5, 0.5] as [number, number, number],
            })),
          }],
        }],
      },
    };
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
    const container = await render(
      <PlaceableAuthoringEditor
        file={new File(["glb"], "large-split.glb")}
        sourceSha256={largeBootstrap.document.sourceSha256}
        bootstrap={largeBootstrap}
        onDocumentChange={() => undefined}
      />,
    );

    const split = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent === "Split");
    await act(async () => split?.click());

    expect(confirm).toHaveBeenCalledOnce();
    expect(confirm.mock.calls[0][0]).toContain("513 connected components");
    expect(container.textContent).toContain("1 elements");
  });
});
