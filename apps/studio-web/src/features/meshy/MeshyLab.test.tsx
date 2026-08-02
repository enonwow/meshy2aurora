// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { InMemoryMeshyBridgeClient, MeshyBridgeError } from "./bridge";
import { MeshyLab } from "./MeshyLab";

vi.mock("./MeshyModelViewport", () => ({
  MeshyModelViewport: ({ label, onClose }: { readonly label: string; readonly onClose?: () => void }) => <div data-testid="meshy-model-viewport">{label}<button type="button" aria-label="Close Meshy viewport" onClick={onClose}>Close</button></div>,
}));

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

function navigationButton(container: HTMLElement, label: string) {
  return Array.from(container.querySelectorAll<HTMLButtonElement>(".meshy-lab__navigation-item"))
    .find((candidate) => candidate.getAttribute("aria-label") === label || candidate.textContent?.includes(label));
}

function historyCard(container: HTMLElement, taskId: string) {
  return container.querySelector<HTMLButtonElement>(`[data-history-task-id="${taskId}"]`);
}

function setValue(element: HTMLInputElement | HTMLTextAreaElement, value: string) {
  const prototype = element instanceof HTMLTextAreaElement
    ? HTMLTextAreaElement.prototype
    : HTMLInputElement.prototype;
  Object.getOwnPropertyDescriptor(prototype, "value")?.set?.call(element, value);
  element.dispatchEvent(new Event("input", { bubbles: true }));
}

function setSelectValue(element: HTMLSelectElement, value: string) {
  Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value")?.set?.call(element, value);
  element.dispatchEvent(new Event("change", { bubbles: true }));
}

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
  window.sessionStorage.clear();
});

describe("MeshyLab", () => {
  it("connects without exposing a pairing code when the local Bridge offers same-origin automatic pairing", async () => {
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 120, automaticPairingSupported: true });
    const container = await render(<MeshyLab bridge={bridge} onBack={vi.fn()} onImport={vi.fn()} />);
    await settle();
    expect(container.querySelector("#meshy-pairing-code")).toBeNull();
    await act(async () => button(container, "Connect local bridge")?.click());
    await settle();
    expect(container.textContent).toContain("120 credits");
    expect(container.querySelectorAll(".meshy-lab__tool-icon svg")).toHaveLength(3);
    expect(container.querySelectorAll(".meshy-lab__source-icon svg")).toHaveLength(3);
    expect(container.querySelectorAll(".meshy-lab__segmented-control")).toHaveLength(2);
    expect(container.querySelectorAll('input[name="meshy-model-type"]')).toHaveLength(2);
    expect(container.querySelector(".meshy-lab__creator-controls .meshy-lab__review-cta")).not.toBeNull();
    const navigationToggle = container.querySelector<HTMLButtonElement>('[aria-label="Toggle workspace navigation"]');
    expect(navigationToggle).not.toBeNull();
    await act(async () => navigationToggle?.click());
    expect(container.querySelector(".meshy-lab__layout")?.classList.contains("meshy-lab__layout--navigation-collapsed")).toBe(true);
    expect(container.querySelector<HTMLDetailsElement>(".meshy-lab__advanced")?.open).toBe(false);
    const sortModels = container.querySelector<HTMLButtonElement>('[aria-label="Sort models oldest first"]');
    expect(sortModels).not.toBeNull();
    await act(async () => sortModels?.click());
    expect(container.querySelector('[aria-label="Sort models newest first"]')).not.toBeNull();
  });

  it("explains that an invalid pairing code is not a Meshy API key", async () => {
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    const bridge = new InMemoryMeshyBridgeClient();
    vi.spyOn(bridge, "pair").mockRejectedValue(new MeshyBridgeError("PAIRING_REQUIRED", "The local Bridge pairing code is invalid."));
    const container = await render(<MeshyLab bridge={bridge} onBack={vi.fn()} onImport={vi.fn()} />);
    await act(async () => {
      setValue(container.querySelector<HTMLInputElement>("#meshy-pairing-code")!, "not-a-pairing-code");
      button(container, "Connect local bridge")?.click();
    });
    await settle();

    expect(container.textContent).toContain("Pairing code is invalid. It is not your Meshy API key.");
    expect(container.textContent).toContain("docker compose --profile meshy logs --tail 1 meshy-bridge");
  });

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

    expect(container.textContent).toContain("120 credits");
    const aiModel = container.querySelector<HTMLSelectElement>(".meshy-lab__api-grid select");
    expect(aiModel?.value).toBe("meshy-6");
    expect(Array.from(aiModel?.options ?? []).map((option) => option.value)).not.toContain("latest");
    const prompt = container.querySelector<HTMLTextAreaElement>("#meshy-asset-prompt")!;
    const adaptiveDecimation = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]'))
      .find((checkbox) => checkbox.parentElement?.textContent?.includes("Automatyczne dzielenie"));
    expect(adaptiveDecimation).toBeTruthy();
    expect(adaptiveDecimation?.checked).toBe(false);
    expect(container.querySelector(".meshy-lab__inline-toggle .meshy-lab__switch-track")).not.toBeNull();
    await act(async () => {
      setValue(prompt, "A neutral humanoid adventurer in A-pose");
      const rig = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]'))
        .find((checkbox) => checkbox.parentElement?.textContent?.includes("Rig as standard humanoid"));
      rig?.click();
      button(container, "Review generation")?.click();
    });
    await settle();

    expect(container.textContent).toContain("Review generation");
    expect(container.textContent).toContain("43 credits maximum");
    await act(async () => button(container, "Generate model")?.click());
    await settle();

    expect(container.textContent).toContain("Generation queued");
    expect(onImport).not.toHaveBeenCalled();
  });

  it("recovers the newest Bridge run after the Studio component reloads", async () => {
    const bridge = new InMemoryMeshyBridgeClient({
      availableCredits: 120,
      automaticPairingSupported: true,
    });
    const seededSession = await bridge.pairAutomatically();
    const preview = await bridge.previewRun(seededSession.sessionToken, {
      profileId: "S1-static-prop/v1",
      prompt: "recovery proof prop",
      geometryTarget: "BALANCED",
    });
    const seededRun = await bridge.createRun(seededSession.sessionToken, {
      previewId: preview.previewId,
      confirmationNonce: "recovery-proof-nonce",
    });
    window.sessionStorage.setItem("m2a.meshy.activeRun.v2", seededRun.id);

    const container = await render(
      <MeshyLab bridge={bridge} onBack={vi.fn()} onImport={vi.fn()} />,
    );
    await settle();
    await act(async () => button(container, "Connect local bridge")?.click());
    await settle();

    expect(container.textContent).toContain("Generation queued");
  });

  it("configures up to ten explicit Meshy action to NWN clip mappings in the paid preview", async () => {
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 120 });
    const previewRun = vi.spyOn(bridge, "previewRun");
    const container = await render(<MeshyLab bridge={bridge} onBack={vi.fn()} onImport={vi.fn()} />);

    await act(async () => {
      setValue(container.querySelector<HTMLInputElement>("#meshy-pairing-code")!, "local-proof");
      button(container, "Connect local bridge")?.click();
    });
    await settle();
    await act(async () => {
      setValue(container.querySelector<HTMLTextAreaElement>("#meshy-asset-prompt")!, "A neutral humanoid adventurer in T-pose");
      const rig = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]'))
        .find((checkbox) => checkbox.parentElement?.textContent?.includes("Rig as standard humanoid"));
      rig?.click();
    });
    await settle();

    expect(container.querySelector<HTMLInputElement>('[aria-label="Meshy action ID 1"]')?.value).toBe("0");
    expect(container.querySelector<HTMLSelectElement>('[aria-label="NWN clip 1"]')?.value).toBe("cpause1");
    await act(async () => button(container, "Add Meshy animation")?.click());
    const secondActionInput = container.querySelector<HTMLInputElement>('[aria-label="Meshy action ID 2"]')!;
    secondActionInput.focus();
    await act(async () => {
      setValue(secondActionInput, "198");
    });
    expect(document.activeElement).toBe(secondActionInput);
    await act(async () => {
      setSelectValue(container.querySelector<HTMLSelectElement>('[aria-label="NWN clip 2"]')!, "ca1slashl");
      button(container, "Review generation")?.click();
    });
    await settle();

    expect(previewRun).toHaveBeenCalledOnce();
    expect(previewRun.mock.calls[0][1].apiOptions?.animationActions).toEqual([
      { actionId: 0, clipName: "cpause1" },
      { actionId: 198, clipName: "ca1slashl" },
    ]);
    expect(container.textContent).toContain("46 credits maximum");
    expect(container.textContent).toContain("0 → cpause1");
    expect(container.textContent).toContain("198 → ca1slashl");
  });

  it("describes Meshy polycount as a measured target instead of an exact guarantee", async () => {
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 20 });
    const container = await render(<MeshyLab bridge={bridge} onBack={vi.fn()} onImport={vi.fn()} />);
    await act(async () => {
      setValue(container.querySelector<HTMLInputElement>("#meshy-pairing-code")!, "local-proof");
      button(container, "Connect local bridge")?.click();
    });
    await settle();

    expect(container.textContent).toContain("Meshy target, not an exact output");
    expect(container.textContent).toContain("300,000 triangles");
  });

  it("reviews a 2D concept generation before it can create its paid task", async () => {
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 20 });
    const container = await render(<MeshyLab bridge={bridge} onBack={vi.fn()} onImport={vi.fn()} />);
    await act(async () => {
      setValue(container.querySelector<HTMLInputElement>("#meshy-pairing-code")!, "local-proof");
      button(container, "Connect local bridge")?.click();
    });
    await settle();
    await act(async () => navigationButton(container, "Concept images")?.click());
    expect(container.textContent).toContain("Concept images");
    expect(container.querySelector(".meshy-lab__creator-library")).not.toBeNull();
    await act(async () => {
      setValue(container.querySelector<HTMLTextAreaElement>("#meshy-image-prompt")!, "A clear wood and brass lantern concept");
      button(container, "Review image generation")?.click();
    });
    await settle();
    expect(container.textContent).toContain("Review image generation");
    expect(container.textContent).toContain("3 credits maximum");
    expect(container.textContent).toContain("No task exists yet");
  });

  it("uses a Meshy-styled, real file control for Image to 3D", async () => {
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 20 });
    const container = await render(<MeshyLab bridge={bridge} onBack={vi.fn()} onImport={vi.fn()} />);
    await act(async () => {
      setValue(container.querySelector<HTMLInputElement>("#meshy-pairing-code")!, "local-proof");
      button(container, "Connect local bridge")?.click();
    });
    await settle();
    await act(async () => container.querySelector<HTMLInputElement>('[aria-label="Image to 3D"] input')?.click());

    const input = container.querySelector<HTMLInputElement>("#meshy-reference-images");
    expect(input?.classList.contains("meshy-lab__reference-input")).toBe(true);
    expect(input?.closest(".meshy-lab__reference-dropzone")).not.toBeNull();
    expect(container.textContent).toContain("Choose a reference image");
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
    await act(async () => button(container, "Generate model")?.click());
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

  it("shows prior Meshy work and imports a recovered refined GLB without creating a paid run", async () => {
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 120 });
    await bridge.addRecoverableHistoryForTest({
      taskId: "preview-history-task",
      stage: "PREVIEW",
      status: "SUCCEEDED",
      prompt: "Preview of the stone golem",
      createdAt: "2026-07-18T14:58:00.000Z",
      finishedAt: "2026-07-18T14:59:00.000Z",
      consumedCredits: 0,
      glbAvailable: false,
    }, new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
    await bridge.addRecoverableHistoryForTest({
      taskId: "refine-history-task",
      stage: "REFINE",
      status: "SUCCEEDED",
      prompt: "Recovered stone golem",
      createdAt: "2026-07-18T15:00:00.000Z",
      finishedAt: "2026-07-18T15:01:00.000Z",
      consumedCredits: 20,
      glbAvailable: true,
    }, new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
    const onImport = vi.fn();
    const container = await render(<MeshyLab bridge={bridge} onBack={vi.fn()} onImport={onImport} />);

    await act(async () => {
      setValue(container.querySelector<HTMLInputElement>("#meshy-pairing-code")!, "local-proof");
      button(container, "Connect local bridge")?.click();
    });
    await settle();
    expect(navigationButton(container, "New model")).toBeTruthy();
    expect(navigationButton(container, "Generated models")).toBeTruthy();
    expect(container.querySelectorAll(".meshy-lab__library-card")).toHaveLength(1);
    expect(container.textContent).not.toContain("Preview of the stone golem");
    await act(async () => navigationButton(container, "Generated models")?.click());
    await settle();

    expect(container.textContent).toContain("Meshy history");
    expect(container.textContent).toContain("Recovered stone golem");
    expect(container.querySelector(".meshy-lab__history-grid")).not.toBeNull();
    expect(container.textContent).not.toContain("Preview of the stone golem");
    expect(button(container, "Recover selected GLB")?.disabled).toBe(true);
    expect(button(container, "Ready to recover (1)")?.getAttribute("aria-pressed")).toBe("true");
    await act(async () => button(container, "All tasks (2)")?.click());
    expect(container.textContent).toContain("Preview of the stone golem");
    expect(container.textContent).toContain("Preview only");
    await act(async () => navigationButton(container, "New model")?.click());
    expect(container.textContent).toContain("Meshy API generation");
    await act(async () => navigationButton(container, "Generated models")?.click());
    await settle();
    await act(async () => historyCard(container, "refine-history-task")?.click());
    expect(historyCard(container, "refine-history-task")?.getAttribute("aria-pressed")).toBe("true");
    await act(async () => button(container, "Recover selected GLB")?.click());
    await settle();

    expect(onImport).toHaveBeenCalledOnce();
    const [file, provenance] = onImport.mock.calls[0];
    expect(file.name).toBe("meshy-recovered-text-to-3d.glb");
    expect(provenance.profileId).toBe("RECOVERED-text-to-3d/v1");
  });

  it("opens a verified library model in the separate Meshy viewport", async () => {
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    const bridge = new InMemoryMeshyBridgeClient({ availableCredits: 20 });
    await bridge.addRecoverableHistoryForTest({
      taskId: "viewer-history-task", stage: "REFINE", status: "SUCCEEDED", prompt: "Viewport stone lantern",
      glbAvailable: true, thumbnailAvailable: true,
    }, new Uint8Array([0x67, 0x6c, 0x54, 0x46]));
    const container = await render(<MeshyLab bridge={bridge} onBack={vi.fn()} onImport={vi.fn()} />);
    await act(async () => {
      setValue(container.querySelector<HTMLInputElement>("#meshy-pairing-code")!, "local-proof");
      button(container, "Connect local bridge")?.click();
    });
    await settle();
    const asset = container.querySelector<HTMLButtonElement>('[aria-label="Open Viewport stone lantern in the Meshy viewport"]');
    expect(asset).not.toBeNull();
    await act(async () => asset?.click());
    await settle();
    expect(container.querySelector('[data-testid="meshy-model-viewport"]')).not.toBeNull();
    const close = container.querySelector<HTMLButtonElement>('[aria-label="Close Meshy viewport"]');
    expect(close).not.toBeNull();
    await act(async () => close?.click());
    expect(container.querySelector('[data-testid="meshy-model-viewport"]')).toBeNull();
  });
});
