// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { InputsPanel } from "./InputsPanel";
import { SourceStep, type SourceStepProps } from "./SourceStep";

const roots: Root[] = [];

const source = () => new File(["glb"], "hero.glb", { type: "model/gltf-binary" });
const appearance = () => new File(["2DA V2.0"], "appearance.2da", { type: "text/plain" });

function handlers() {
  return {
    onSelectSource: vi.fn(),
    onSelectAppearance: vi.fn(),
    onRemoveSource: vi.fn(),
    onRemoveAppearance: vi.fn(),
    onClear: vi.fn(),
  };
}

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
});

describe("SourceStep", () => {
  it("keeps Continue disabled for empty and partial selections and names the missing input", async () => {
    const emptyHandlers = handlers();
    const container = await render(<SourceStep {...emptyHandlers} onContinue={vi.fn()} />);
    const continueButton = Array.from(container.querySelectorAll("button"))
      .find((button) => button.textContent?.includes("Continue to Inspect"));

    expect(continueButton?.disabled).toBe(true);
    expect(container.textContent).toContain("Select both required files to continue.");

    await act(async () => {
      rootRender(container, <SourceStep {...emptyHandlers} source={source()} onContinue={vi.fn()} />);
    });
    expect(container.textContent).toContain("Select the base appearance.2da file to continue.");
  });

  it("shows neutral Selected metadata and enables Continue only when both inputs exist", async () => {
    const onContinue = vi.fn();
    const container = await render(
      <SourceStep
        {...handlers()}
        source={source()}
        appearance={appearance()}
        sourceIdentity={{ sha256: "abcdef1234567890" }}
        onContinue={onContinue}
      />,
    );

    expect(container.textContent?.match(/Selected/g)).toHaveLength(2);
    expect(container.textContent).toContain("SHA-256 abcdef123456...");
    expect(container.textContent).not.toContain("Valid");

    const continueButton = Array.from(container.querySelectorAll("button"))
      .find((button) => button.textContent?.includes("Continue to Inspect"));
    expect(continueButton?.disabled).toBe(false);
    await act(async () => continueButton?.click());
    expect(onContinue).toHaveBeenCalledOnce();
  });

  it("routes dropped files through the same selection callback", async () => {
    const callbacks = handlers();
    const container = await render(<SourceStep {...callbacks} onContinue={vi.fn()} />);
    const dropZone = container.querySelector(".source-drop-zone");
    const dropped = source();
    const event = new Event("drop", { bubbles: true, cancelable: true });
    Object.defineProperty(event, "dataTransfer", { value: { files: [dropped] } });

    await act(async () => dropZone?.dispatchEvent(event));
    expect(callbacks.onSelectSource).toHaveBeenCalledWith(dropped);
  });
});

describe("InputsPanel", () => {
  it("exposes native file inputs and remove/clear actions", async () => {
    const callbacks = handlers();
    const container = await render(
      <InputsPanel {...callbacks} source={source()} appearance={appearance()} />,
    );

    const inputs = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="file"]'));
    expect(inputs.map((input) => input.accept)).toEqual([".glb,model/gltf-binary", ".2da"]);

    const removeSource = Array.from(container.querySelectorAll("button"))
      .find((button) => button.getAttribute("aria-label") === "Remove Meshy GLB model");
    await act(async () => removeSource?.click());
    expect(callbacks.onRemoveSource).toHaveBeenCalledOnce();
  });
});

function rootRender(container: HTMLElement, element: React.ReactNode) {
  const index = Array.from(document.body.children).indexOf(container);
  roots[index]?.render(element);
}
