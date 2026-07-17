// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { InMemoryMeshyBridgeClient } from "./bridge";
import { MeshyLab } from "./MeshyLab";

const roots: Root[] = [];

async function render(element: React.ReactNode) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  await act(async () => root.render(element));
  return container;
}

async function settle() {
  await act(async () => { await new Promise((resolve) => window.setTimeout(resolve, 0)); });
}

function button(container: HTMLElement, label: string) {
  return Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
    .find((candidate) => candidate.textContent?.trim() === label);
}

function setValue(element: HTMLInputElement | HTMLTextAreaElement, value: string) {
  const prototype = element instanceof HTMLTextAreaElement
    ? HTMLTextAreaElement.prototype
    : HTMLInputElement.prototype;
  Object.getOwnPropertyDescriptor(prototype, "value")?.set?.call(element, value);
  element.dispatchEvent(new Event("input", { bubbles: true }));
}

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
});

describe("MeshyLab", () => {
  it("requires pairing and an explicit review before it creates a Meshy run", async () => {
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 120 });
    const onImport = vi.fn();
    const container = await render(<MeshyLab bridge={bridge} onBack={vi.fn()} onImport={onImport} />);

    expect(container.textContent).toContain("Connect local bridge");
    const pairingCode = container.querySelector<HTMLInputElement>("#meshy-pairing-code")!;
    await act(async () => {
      setValue(pairingCode, "local-proof");
      button(container, "Connect local bridge")?.click();
    });
    await settle();

    expect(container.textContent).toContain("Available balance: 120 credits");
    const prompt = container.querySelector<HTMLTextAreaElement>("#meshy-asset-prompt")!;
    await act(async () => {
      setValue(prompt, "A neutral humanoid adventurer in A-pose");
      container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]').forEach((checkbox) => checkbox.click());
      button(container, "Review generation")?.click();
    });
    await settle();

    expect(container.textContent).toContain("Review generation");
    expect(container.textContent).toContain("38 credits maximum");
    await act(async () => button(container, "Generate H1 asset")?.click());
    await settle();

    expect(container.textContent).toContain("Generation queued");
    expect(onImport).not.toHaveBeenCalled();
  });

  it("imports only a ready, hash-verified GLB through the caller boundary", async () => {
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 120 });
    const onImport = vi.fn();
    const container = await render(<MeshyLab bridge={bridge} onBack={vi.fn()} onImport={onImport} />);

    await act(async () => {
      setValue(container.querySelector<HTMLInputElement>("#meshy-pairing-code")!, "local-proof");
      button(container, "Connect local bridge")?.click();
    });
    await settle();
    await act(async () => {
      setValue(container.querySelector<HTMLTextAreaElement>("#meshy-asset-prompt")!, "A weathered stone lantern");
    });
    await settle();
    await act(async () => {
      const profile = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
        .find((candidate) => candidate.textContent?.includes("S1 · Static Prop"));
      profile?.click();
    });
    await settle();
    await act(async () => button(container, "Review generation")?.click());
    await settle();
    await act(async () => button(container, "Generate S1 asset")?.click());
    await settle();

    await bridge.completeRunForTest(bridge.latestRunIdForTest()!, new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
    await act(async () => button(container, "Refresh status")?.click());
    await settle();
    await act(async () => button(container, "Import verified GLB to Source")?.click());
    await settle();

    expect(onImport).toHaveBeenCalledOnce();
    const [file, provenance] = onImport.mock.calls[0];
    expect(file.name).toBe("meshy-s1-static-prop.glb");
    expect(provenance.sha256).toMatch(/^[a-f0-9]{64}$/);
  });
});
