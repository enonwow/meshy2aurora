// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { animationStudioClipFixtureV1 } from "../animation-studio/testFixtures";
import { AnimationContributionDialog } from "./AnimationContributionDialog";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const roots: Root[] = [];

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("AnimationContributionDialog", () => {
  it("requires rights and tags, exports portable metadata and restores a close boundary", async () => {
    const onExport = vi.fn(async () => ({ schemaVersion: 1, presetId: "community_m2a_custom" }));
    const onClose = vi.fn();
    const createObjectUrl = vi.fn(() => "blob:contribution");
    const revokeObjectUrl = vi.fn();
    vi.stubGlobal("URL", {
      ...URL,
      createObjectURL: createObjectUrl,
      revokeObjectURL: revokeObjectUrl,
    });
    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => undefined);
    const container = mount();
    await render(container, (
      <AnimationContributionDialog
        clip={animationStudioClipFixtureV1({ name: "m2a_custom", status: "VALID" })}
        onExport={onExport}
        onClose={onClose}
      />
    ));

    const presetId = labelledInput(container, "Preset ID");
    expect(document.activeElement).toBe(presetId);
    const download = buttonByText(container, "Download contribution JSON");
    expect(download.disabled).toBe(true);
    await setValue(labelledInput(container, "Author"), "Example contributor");
    await setValue(labelledTextArea(container, "Summary"), "Original guarded motion.");
    await click(buttonByText(container, "humanoid"));
    const rights = required<HTMLInputElement>(
      container.querySelector('.animation-contribution-dialog__rights input[type="checkbox"]'),
    );
    await act(async () => {
      rights.click();
    });
    expect(download.disabled).toBe(false);
    await click(download);
    await flush();

    expect(onExport).toHaveBeenCalledWith(
      expect.objectContaining({ status: "VALID", name: "m2a_custom" }),
      expect.objectContaining({
        presetId: "community_m2a_custom",
        outputName: "m2a_custom",
        authors: [{ name: "Example contributor" }],
        tags: ["humanoid"],
        license: "CC0-1.0",
        validationStatus: "PIPELINE_VERIFIED",
      }),
    );
    expect(createObjectUrl).toHaveBeenCalledWith(expect.any(Blob));
    expect(onClose).toHaveBeenCalledOnce();
    await flush();
    expect(revokeObjectUrl).toHaveBeenCalledWith("blob:contribution");
  });

  it("closes with Escape", async () => {
    const onClose = vi.fn();
    const container = mount();
    await render(container, (
      <AnimationContributionDialog
        clip={animationStudioClipFixtureV1({ name: "m2a_custom", status: "VALID" })}
        onExport={async () => {
          throw new Error("M2A-ANIMATION-LIBRARY-LICENSE: license is blocked");
        }}
        onClose={onClose}
      />
    ));
    const dialog = required<HTMLElement>(container.querySelector('[role="dialog"]'));
    await act(async () => {
      dialog.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("keeps the dialog open and exposes a Core rejection", async () => {
    const onClose = vi.fn();
    const container = mount();
    await render(container, (
      <AnimationContributionDialog
        clip={animationStudioClipFixtureV1({ name: "m2a_custom", status: "VALID" })}
        onExport={async () => {
          throw new Error("M2A-ANIMATION-LIBRARY-LICENSE: license is blocked");
        }}
        onClose={onClose}
      />
    ));
    await setValue(labelledInput(container, "Author"), "Example contributor");
    await setValue(labelledTextArea(container, "Summary"), "Original motion.");
    await click(buttonByText(container, "humanoid"));
    await act(async () => {
      required<HTMLInputElement>(
        container.querySelector('.animation-contribution-dialog__rights input[type="checkbox"]'),
      ).click();
    });
    await click(buttonByText(container, "Download contribution JSON"));
    await flush();
    expect(container.querySelector('[role="alert"]')?.textContent).toContain(
      "M2A-ANIMATION-LIBRARY-LICENSE",
    );
    expect(onClose).not.toHaveBeenCalled();
  });
});

function mount() {
  const container = document.createElement("div");
  document.body.append(container);
  roots.push(createRoot(container));
  return container;
}

async function render(container: HTMLElement, value: React.ReactNode) {
  await act(async () => roots.at(-1)?.render(value));
  return container;
}

function required<T>(value: T | null | undefined): T {
  if (value === null || value === undefined) throw new Error("required element missing");
  return value;
}

function labelledInput(container: HTMLElement, text: string) {
  return required<HTMLInputElement>(label(container, text).querySelector("input"));
}

function labelledTextArea(container: HTMLElement, text: string) {
  return required<HTMLTextAreaElement>(label(container, text).querySelector("textarea"));
}

function label(container: HTMLElement, text: string) {
  return required(Array.from(container.querySelectorAll("label"))
    .find((candidate) => candidate.textContent?.trim().startsWith(text)));
}

function buttonByText(container: HTMLElement, text: string) {
  return required(Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
    .find((button) => button.textContent?.trim() === text));
}

async function setValue(input: HTMLInputElement | HTMLTextAreaElement, value: string) {
  await act(async () => {
    const prototype = input instanceof HTMLInputElement
      ? HTMLInputElement.prototype
      : HTMLTextAreaElement.prototype;
    Object.getOwnPropertyDescriptor(prototype, "value")?.set?.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
}

async function click(button: HTMLButtonElement) {
  await act(async () => button.click());
}

async function flush() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}
