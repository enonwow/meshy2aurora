// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import type {
  AnimationPresetCatalogEntryV1,
  AnimationPresetCompatibilityV1,
} from "../animation-library/types";
import { AnimationClipLibrary } from "./AnimationClipLibrary";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const roots: Root[] = [];

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
  vi.restoreAllMocks();
});

describe("AnimationClipLibrary repository presets", () => {
  it("searches metadata, combines accessible tags and creates only a Custom copy", async () => {
    const inspect = vi.fn(async (preset: AnimationPresetCatalogEntryV1) => ({
      schemaVersion: 1,
      status: "COMPATIBLE",
      expectedRigSignatureSha256: preset.rigSignatureSha256,
      actualRigSignatureSha256: preset.rigSignatureSha256,
      missingBones: [],
      diagnostics: [],
    } satisfies AnimationPresetCompatibilityV1));
    const usePreset = vi.fn(async () => undefined);
    const container = mount();
    await render(container, (
      <AnimationClipLibrary
        items={[]}
        selectedId={null}
        onSelect={vi.fn()}
        onCreateBlank={vi.fn()}
        onEditCopy={vi.fn()}
        onDuplicate={vi.fn()}
        onInspectPreset={inspect}
        onUsePreset={usePreset}
      />
    ));

    await click(buttonByText(container, "Built-in"));
    expect(inspect).not.toHaveBeenCalled();
    expect(options(container)).toHaveLength(7);

    const search = required<HTMLInputElement>(
      container.querySelector('input[type="search"]'),
    );
    await setInputValue(search, "Meshy2Aurora contributors");
    expect(options(container)).toHaveLength(7);
    expect(inspect).not.toHaveBeenCalled();
    await setInputValue(search, "");

    const attack = buttonByText(container, "attack");
    const boxing = buttonByText(container, "boxing");
    expect(attack.getAttribute("aria-pressed")).toBe("false");
    await click(attack);
    await click(boxing);
    expect(attack.getAttribute("aria-pressed")).toBe("true");
    expect(boxing.getAttribute("aria-pressed")).toBe("true");
    expect(options(container)).toHaveLength(4);

    await click(optionByText(container, "Right cross"));
    await waitForText(container, "Compatible");
    expect(inspect).toHaveBeenCalledOnce();
    expect(container.textContent).toContain("Compatible");
    expect(container.textContent).toContain("LicenseRef-Meshy2Aurora-Project-Generated");
    expect(container.textContent).toContain("Offline pipeline verified");
    expect(container.textContent).toContain("v1");
    expect(container.textContent).toContain(
      "Repository presets are immutable. Editing always creates a local Custom copy.",
    );

    await click(buttonByText(container, "Use as template"));
    await flush();
    expect(usePreset).toHaveBeenCalledWith(expect.objectContaining({
      presetId: "m2a_right_cross",
      presetVersion: 1,
    }));
    expect(buttonByText(container, "Custom").getAttribute("aria-selected"))
      .toBe("true");
  });

  it("fails closed for an incompatible rig and exposes Community/error/empty states", async () => {
    const incompatible: AnimationPresetCompatibilityV1 = {
      schemaVersion: 1,
      status: "INCOMPATIBLE",
      expectedRigSignatureSha256: "a".repeat(64),
      actualRigSignatureSha256: "b".repeat(64),
      missingBones: ["RightArm"],
      diagnostics: [{
        schemaVersion: 1,
        code: "M2A-ANIMATION-LIBRARY-RIG-SIGNATURE",
        path: "rig",
        message: "The current rig is different.",
        action: "Choose a preset for the exact rig profile; automatic retargeting is not available.",
      }],
    };
    const inspect = vi.fn(async () => incompatible);
    const usePreset = vi.fn(async () => undefined);
    const container = mount();
    await render(container, (
      <AnimationClipLibrary
        items={[]}
        selectedId={null}
        onSelect={vi.fn()}
        onCreateBlank={vi.fn()}
        onEditCopy={vi.fn()}
        onDuplicate={vi.fn()}
        onInspectPreset={inspect}
        onUsePreset={usePreset}
      />
    ));

    await click(buttonByText(container, "Built-in"));
    await click(optionByText(container, "Right cross"));
    await waitForText(container, "Incompatible");
    expect(container.textContent).toContain("Incompatible");
    expect(container.textContent).toContain("automatic retargeting is not available");
    expect(buttonByText(container, "Use as template").disabled).toBe(true);
    expect(usePreset).not.toHaveBeenCalled();

    await click(buttonByText(container, "Community"));
    expect(container.textContent).toContain("No animations match this view.");
    expect(options(container)).toHaveLength(0);
  });

  it("renders the asynchronous preview state without mojibake", async () => {
    let resolveInspection: ((value: AnimationPresetCompatibilityV1) => void) | null = null;
    const inspect = vi.fn((preset: AnimationPresetCatalogEntryV1) => (
      new Promise<AnimationPresetCompatibilityV1>((resolve) => {
        resolveInspection = resolve;
      }).then((value) => ({
        ...value,
        expectedRigSignatureSha256: preset.rigSignatureSha256,
      }))
    ));
    const container = mount();
    await render(container, (
      <AnimationClipLibrary
        items={[]}
        selectedId={null}
        onSelect={vi.fn()}
        onCreateBlank={vi.fn()}
        onEditCopy={vi.fn()}
        onDuplicate={vi.fn()}
        onInspectPreset={inspect}
      />
    ));

    await click(buttonByText(container, "Built-in"));
    await click(optionByText(container, "Right cross"));
    expect(container.textContent).toContain("Loading preview…");
    expect(container.textContent).not.toMatch(/[ÂÃâĂ]/u);

    await act(async () => {
      required(resolveInspection)({
        schemaVersion: 1,
        status: "COMPATIBLE",
        expectedRigSignatureSha256: "a".repeat(64),
        actualRigSignatureSha256: "a".repeat(64),
        missingBones: [],
        diagnostics: [],
      });
      await Promise.resolve();
    });
  });
});

function mount() {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  return container;
}

async function render(container: HTMLElement, value: React.ReactNode) {
  const root = roots.at(-1);
  if (!root || root !== roots.find((candidate) => candidate === root)) {
    throw new Error("test root is unavailable");
  }
  await act(async () => root.render(value));
  return container;
}

function required<T>(value: T | null): T {
  if (value === null) throw new Error("required test element is missing");
  return value;
}

function buttons(container: HTMLElement) {
  return Array.from(container.querySelectorAll<HTMLButtonElement>("button"));
}

function buttonByText(container: HTMLElement, text: string) {
  return required(buttons(container).find((button) => button.textContent?.trim() === text) ?? null);
}

function optionByText(container: HTMLElement, text: string) {
  return required(Array.from(
    container.querySelectorAll<HTMLButtonElement>('[role="option"]'),
  ).find((button) => button.textContent?.includes(text)) ?? null);
}

function options(container: HTMLElement) {
  return Array.from(container.querySelectorAll<HTMLButtonElement>('[role="option"]'));
}

async function click(button: HTMLButtonElement) {
  await act(async () => {
    button.click();
    await Promise.resolve();
  });
}

async function setInputValue(input: HTMLInputElement, value: string) {
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set
      ?.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
}

async function flush() {
  await act(async () => {
    await Promise.resolve();
    await new Promise((resolve) => setTimeout(resolve, 10));
  });
}

async function waitForText(container: HTMLElement, expected: string) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (container.textContent?.includes(expected)) return;
    await act(async () => new Promise((resolve) => setTimeout(resolve, 10)));
  }
  throw new Error(`timed out waiting for ${expected}`);
}
