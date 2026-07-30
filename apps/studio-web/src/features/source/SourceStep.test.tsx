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
    onSelectAnimationEvents: vi.fn(),
    onRemoveSource: vi.fn(),
    onRemoveAppearance: vi.fn(),
    onRemoveAnimationEvents: vi.fn(),
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
  it("hides Tile by default and exposes it only when explicitly enabled", async () => {
    const callbacks = { ...handlers(), onTargetChange: vi.fn() };
    const hidden = await render(
      <SourceStep {...callbacks} onContinue={vi.fn()} />,
    );
    expect(hidden.querySelector('input[value="TILE"]')).toBeNull();
    expect(hidden.textContent).toContain("choose Creature or Placeable");

    const enabled = await render(
      <SourceStep {...callbacks} tileTargetEnabled onContinue={vi.fn()} />,
    );
    expect(enabled.querySelector('input[value="TILE"]')).not.toBeNull();
    expect(enabled.textContent).toContain("choose Creature, Placeable, or Tile");
  });

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
    expect(container.textContent).toContain(
      "Select appearance.2da or placeables.2da to continue.",
    );
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

  it("shows redacted Meshy provenance only when the source was explicitly imported from Meshy Lab", async () => {
    const container = await render(
      <SourceStep
        {...handlers()}
        source={source()}
        meshyProvenance={{
          profileId: "S1-static-prop/v1",
          bridgeProtocolVersion: 1,
          sha256: "a".repeat(64),
          byteLength: 4,
          taskIds: { PREVIEW: "task-preview", REFINE: "task-refine" },
        }}
        onContinue={vi.fn()}
      />,
    );

    expect(container.textContent).toContain("Imported from Meshy Lab: S1-static-prop/v1");
    expect(container.textContent).toContain("SHA-256 aaaaaaaaaaaa...");
    expect(container.textContent).not.toContain("MESHY_API_KEY");
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

  it("keeps the event JSON optional and routes it through a dedicated callback", async () => {
    const callbacks = handlers();
    const container = await render(
      <SourceStep
        {...callbacks}
        source={source()}
        appearance={appearance()}
        onContinue={vi.fn()}
      />,
    );
    const inputs = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="file"]'));
    expect(inputs).toHaveLength(3);
    expect(inputs[2].required).toBe(false);

    const events = new File(['{"schemaVersion":1,"clips":[]}'], "animation-events.json", {
      type: "application/json",
    });
    Object.defineProperty(inputs[2], "files", { configurable: true, value: [events] });
    await act(async () => {
      inputs[2].dispatchEvent(new window.Event("change", { bubbles: true }));
    });
    expect(callbacks.onSelectAnimationEvents).toHaveBeenCalledWith(events);
  });

  it("exposes opt-in texture artifact repair without enabling it by default", async () => {
    const onTextureArtifactCleanupChange = vi.fn();
    const container = await render(
      <SourceStep
        {...handlers()}
        onCreatureProfileChange={vi.fn()}
        onTextureArtifactCleanupChange={onTextureArtifactCleanupChange}
        onContinue={vi.fn()}
      />,
    );
    const checkbox = container.querySelector<HTMLInputElement>(
      'input[aria-label="Repair texture artifacts"]',
    );

    expect(checkbox).not.toBeNull();
    expect(checkbox?.checked).toBe(false);
    expect(container.textContent).toContain("one-pixel alpha holes");
    await act(async () => checkbox?.click());
    expect(onTextureArtifactCleanupChange).toHaveBeenCalledWith(true);
  });

  it("exposes audited detached-accessory modes and accepts a global bone or valid component overrides", async () => {
    const onModeChange = vi.fn();
    const onBoneChange = vi.fn();
    const container = await render(
      <SourceStep
        {...handlers()}
        source={source()}
        appearance={appearance()}
        onCreatureProfileChange={vi.fn()}
        onSkinAccessoryStabilizationModeChange={onModeChange}
        onSkinAccessorySelectedBoneNameChange={onBoneChange}
        onContinue={vi.fn()}
      />,
    );
    const mode = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Detached accessory skinning"]',
    );

    expect(mode?.value).toBe("AUTO");
    expect(Array.from(mode?.options ?? []).map((option) => option.textContent)).toEqual([
      "Auto",
      "Keep source weights",
      "Select bone",
    ]);
    expect(container.textContent).toContain("spatially welded detached parts");

    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value")
        ?.set?.call(mode, "SELECT_BONE");
      mode?.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(onModeChange).toHaveBeenCalledWith("SELECT_BONE");

    await act(async () => {
      rootRender(
        container,
        <SourceStep
          {...handlers()}
          source={source()}
          appearance={appearance()}
          skinAccessoryStabilizationMode="SELECT_BONE"
          skinAccessorySelectedBoneName=""
          onCreatureProfileChange={vi.fn()}
          onSkinAccessoryStabilizationModeChange={onModeChange}
          onSkinAccessorySelectedBoneNameChange={onBoneChange}
          onContinue={vi.fn()}
        />,
      );
    });
    const bone = container.querySelector<HTMLInputElement>(
      'input[aria-label="Accessory bone name"]',
    );
    const continueButton = Array.from(container.querySelectorAll("button"))
      .find((candidate) => candidate.textContent?.includes("Continue to Inspect"));
    expect(bone).not.toBeNull();
    expect(continueButton?.disabled).toBe(true);

    await act(async () => {
      rootRender(
        container,
        <SourceStep
          {...handlers()}
          source={source()}
          appearance={appearance()}
          skinAccessoryStabilizationMode="SELECT_BONE"
          skinAccessorySelectedBoneName=""
          skinAccessoryComponentBoneOverrides={"0:4=Spine\n0:2=Spine02"}
          onCreatureProfileChange={vi.fn()}
          onSkinAccessoryStabilizationModeChange={onModeChange}
          onSkinAccessorySelectedBoneNameChange={onBoneChange}
          onContinue={vi.fn()}
        />,
      );
    });
    expect(continueButton?.disabled).toBe(false);
    expect(container.textContent).not.toContain("Invalid override");

    await act(async () => {
      rootRender(
        container,
        <SourceStep
          {...handlers()}
          source={source()}
          appearance={appearance()}
          skinAccessoryStabilizationMode="SELECT_BONE"
          skinAccessorySelectedBoneName=""
          skinAccessoryComponentBoneOverrides="0:2"
          onCreatureProfileChange={vi.fn()}
          onSkinAccessoryStabilizationModeChange={onModeChange}
          onSkinAccessorySelectedBoneNameChange={onBoneChange}
          onContinue={vi.fn()}
        />,
      );
    });
    expect(continueButton?.disabled).toBe(true);
    expect(container.textContent).toContain("segment:component=BoneName");
  });

  it("uses the shared 300K product profile by default", async () => {
    const container = await render(
      <SourceStep
        {...handlers()}
        onCreatureProfileChange={vi.fn()}
        onContinue={vi.fn()}
      />,
    );
    const profile = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Creature conversion profile"]',
    );

    expect(profile?.value).toBe("PRODUCT_300K");
    expect(profile?.selectedOptions[0]?.textContent).toContain("300,000 triangle limit");
    expect(container.textContent).not.toContain("shared product budget remains 20,000");
  });
});

describe("InputsPanel", () => {
  it("omits Tile from the compact target selector unless explicitly enabled", async () => {
    const callbacks = { ...handlers(), onTargetChange: vi.fn() };
    const hidden = await render(<InputsPanel {...callbacks} />);
    expect(hidden.querySelector('option[value="TILE"]')).toBeNull();

    const enabled = await render(<InputsPanel {...callbacks} tileTargetEnabled />);
    expect(enabled.querySelector('option[value="TILE"]')).not.toBeNull();
  });

  it("exposes native file inputs and remove/clear actions", async () => {
    const callbacks = handlers();
    const container = await render(
      <InputsPanel {...callbacks} source={source()} appearance={appearance()} />,
    );

    const inputs = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="file"]'));
    expect(inputs.map((input) => input.accept)).toEqual([
      ".glb,model/gltf-binary",
      ".2da",
      ".json,application/json",
    ]);
    expect(inputs.map((input) => input.required)).toEqual([true, true, false]);

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
