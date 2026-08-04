// @vitest-environment jsdom

import { act, type ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { InMemoryMeshyBridgeClient } from "../meshy/bridge";
import { ItemGenerationPanel } from "./ItemGenerationPanel";
import type { ItemBaseItemRow } from "./types";

const roots: Root[] = [];

function longsword(): ItemBaseItemRow {
  return {
    schemaVersion: 1,
    baseItem: 1,
    label: "Longsword",
    itemClass: "wswls",
    modelType: 2,
    minRange: 10,
    maxRange: 100,
    genderSpecific: false,
    defaultModel: null,
    defaultIcon: "iw_swls",
    equipableSlots: 0x10,
    invSlotWidth: 1,
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
  };
}

class AutoCompleteBridge extends InMemoryMeshyBridgeClient {
  override async createRun(...args: Parameters<InMemoryMeshyBridgeClient["createRun"]>) {
    const run = await super.createRun(...args);
    await this.completeRunForTest(run.id, new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
    return run;
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

function conceptFile(name: string, marker: number) {
  return {
    name,
    size: 4,
    type: "image/png",
    lastModified: marker,
    arrayBuffer: async () => new Uint8Array([0x89, 0x50, 0x4e, marker]).buffer,
  } as File;
}

async function chooseFile(input: HTMLInputElement, file: File) {
  Object.defineProperty(input, "files", { configurable: true, value: [file] });
  await act(async () => {
    input.dispatchEvent(new window.Event("change", { bubbles: true }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
  });
}

async function click(container: HTMLElement, label: string) {
  const target = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
    .find((button) => button.textContent?.includes(label));
  await act(async () => {
    target?.click();
    await new Promise((resolve) => window.setTimeout(resolve, 0));
  });
}

async function enter(input: HTMLInputElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
  await act(async () => {
    setter?.call(input, value);
    input.dispatchEvent(new window.Event("input", { bubbles: true }));
  });
}

afterEach(async () => {
  await act(async () => {
    while (roots.length) roots.pop()?.unmount();
  });
  document.body.replaceChildren();
  window.localStorage.clear();
});

describe("ItemGenerationPanel", () => {
  it("reviews one capped batch and assigns every generated GLB directly to its Aurora field", async () => {
    const bridge = new AutoCompleteBridge({ availableCredits: 120 });
    const onArtifact = vi.fn();
    const container = await render(
      <ItemGenerationPanel
        row={longsword()}
        baseitemsSha256={"a".repeat(64)}
        bridge={bridge}
        onArtifact={onArtifact}
      />,
    );

    const pairing = container.querySelector<HTMLInputElement>('input[aria-label="Item Meshy pairing code"]')!;
    await enter(pairing, "item-pair");
    await click(container, "Pair Bridge");
    expect(container.textContent).toContain("Bridge paired");

    for (const [index, field] of ["ModelPart1", "ModelPart2", "ModelPart3"].entries()) {
      const input = container.querySelector<HTMLInputElement>(`input[aria-label="${field} concept image"]`)!;
      await chooseFile(input, conceptFile(`${field}.png`, index + 1));
    }

    const cap = container.querySelector<HTMLInputElement>('input[aria-label="Item generation owner credit cap"]')!;
    await enter(cap, "89");
    await click(container, "Review batch cost");
    await vi.waitFor(() => expect(container.textContent).toMatch(/exceeds the owner cap/i));
    expect(onArtifact).not.toHaveBeenCalled();

    await enter(cap, "90");
    await click(container, "Review batch cost");
    await vi.waitFor(() => expect(container.textContent).toContain("90 credits"));

    await click(container, "Confirm paid Item batch");
    await vi.waitFor(() => expect(onArtifact).toHaveBeenCalledTimes(3));
    expect(onArtifact.mock.calls.map(([field]) => field)).toEqual([
      "ModelPart1", "ModelPart2", "ModelPart3",
    ]);
    expect(container.textContent).toContain("ARTIFACTS_VERIFIED");

    const persistedKey = Object.keys(window.localStorage).find((key) => (
      key.startsWith("meshy2aurora:item-generation:v1:")
    ));
    expect(persistedKey).toBeTruthy();
    const persisted = JSON.parse(window.localStorage.getItem(persistedKey!)!) as {
      slots: Array<{
        field: string;
        run: { taskId: string; createdAt: string };
        artifact: { consumedCredits: number; finishedAt: string };
      }>;
    };

    const restartedBridge = new InMemoryMeshyBridgeClient({ availableCredits: 120 });
    for (const slot of persisted.slots) {
      await restartedBridge.addRecoverableImageTo3dForTest({
        taskId: slot.run.taskId,
        consumedCredits: slot.artifact.consumedCredits,
        createdAt: slot.run.createdAt,
        finishedAt: slot.artifact.finishedAt,
      }, new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
    }
    const originalRoot = roots.pop();
    await act(async () => originalRoot?.unmount());
    container.remove();

    const recoveredArtifacts = vi.fn();
    const restored = await render(
      <ItemGenerationPanel
        row={longsword()}
        baseitemsSha256={"a".repeat(64)}
        bridge={restartedBridge}
        onArtifact={recoveredArtifacts}
      />,
    );
    expect(restored.textContent).toContain("ARTIFACTS_VERIFIED");
    await enter(restored.querySelector<HTMLInputElement>('input[aria-label="Item Meshy pairing code"]')!, "restart-pair");
    await click(restored, "Pair Bridge");
    const redownloads = Array.from(restored.querySelectorAll<HTMLButtonElement>("button"))
      .filter((button) => button.textContent?.includes("Re-download exact task"));
    for (const button of redownloads) {
      await act(async () => {
        button.click();
        await new Promise((resolve) => window.setTimeout(resolve, 0));
      });
    }
    expect(recoveredArtifacts.mock.calls.map(([field]) => field)).toEqual([
      "ModelPart1", "ModelPart2", "ModelPart3",
    ]);
  });
});
