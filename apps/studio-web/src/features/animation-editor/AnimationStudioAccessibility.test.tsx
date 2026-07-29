// @vitest-environment jsdom

import { act, useState } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  CREATURE_ANIMATION_AUTHORING_PROFILE_V2,
  type CreatureAnimationAuthoringV2,
} from "../animation-studio/types";
import {
  animationStudioClipFixtureV1,
  animationStudioDocumentFixtureV1,
} from "../animation-studio/testFixtures";
import { AnimationBoneTree, type AnimationRigNodeV1 } from "./AnimationBoneTree";
import { AnimationClipLibrary } from "./AnimationClipLibrary";
import { AnimationEditorViewport } from "./AnimationEditorViewport";
import { AnimationMappingModeSwitch } from "./AnimationMappingModeSwitch";
import { AnimationStudioWorkspace } from "./AnimationStudioWorkspace";
import { AnimationTrimDialog } from "./AnimationTrimDialog";
import { NewAnimationMenu } from "./NewAnimationMenu";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const roots: Root[] = [];
const originalMatchMedia = window.matchMedia;
const sourceRevision = "a".repeat(64);
const rig: readonly AnimationRigNodeV1[] = [{
  id: 0,
  name: "root",
  parentId: null,
  translation: [0, 0, 0],
  rotation: [0, 0, 0, 1],
}, {
  id: 7,
  name: "torso",
  parentId: 0,
  translation: [0, 0, 1],
  rotation: [0, 0, 0, 1],
}];

afterEach(async () => {
  await act(async () => {
    roots.splice(0).forEach((root) => root.unmount());
  });
  document.body.replaceChildren();
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    writable: true,
    value: originalMatchMedia,
  });
  vi.restoreAllMocks();
});

describe("Animation Studio accessibility", () => {
  it("exposes labelled landmarks, assertive failures, polite document status, and complete track/key names", async () => {
    const clip = animationStudioClipFixtureV1({
      id: "accessible-clip",
      status: "INVALID",
    });
    const container = mount();
    await render(container, (
      <AnimationStudioWorkspace
        document={animationStudioDocumentFixtureV1({
          status: "INVALID",
          authoredClips: [clip],
        })}
        authoring={emptyAuthoring()}
        sourceInventory={[]}
        rig={rig}
        viewport={<div aria-label="Three dimensional model preview" />}
        autosaveState={{ kind: "ERROR", message: "IndexedDB unavailable" }}
        diagnostics={[{
          schemaVersion: 1,
          code: "M2A-ANIMATION-EDIT-QUATERNION",
          path: "authoredClips.accessible-clip.tracks.0",
          level: "BLOCKING",
          message: "Quaternion is invalid.",
          action: "Normalize the first key.",
        }]}
        onDocumentChange={vi.fn()}
        onAuthoringChange={vi.fn()}
        onUndo={vi.fn()}
        onRedo={vi.fn()}
        canUndo={false}
        canRedo={false}
      />
    ));

    const workspace = required<HTMLElement>(
      container.querySelector(".animation-studio-workspace"),
    );
    expect(workspace.getAttribute("data-layout")).toBe("aurora-animation-workbench");
    expect(workspace.getAttribute("aria-labelledby")).toBe("animation-studio-title");
    expect(workspace.querySelector("h2#animation-studio-title")?.textContent)
      .toBe("Create & edit animations");
    expect(workspace.querySelector("h1")).toBeNull();
    expect(workspace.querySelector('aside[aria-label="Animation clips"]')).not.toBeNull();
    expect(workspace.querySelector('section[aria-label="Animation viewport and timeline"]'))
      .not.toBeNull();
    expect(workspace.querySelector('aside[aria-label="Animation properties"]'))
      .not.toBeNull();
    expect(workspace.querySelector('nav[aria-label="Output rig bones"]')).not.toBeNull();
    expect(
      workspace.querySelector(".animation-studio-workspace__toolbar-actions"),
    ).not.toBeNull();
    expect(
      workspace.querySelector('[data-variant="primary"][data-action="save-custom"]'),
    ).not.toBeNull();
    expect(
      workspace.querySelector('[data-panel="clip-library"]'),
    ).not.toBeNull();
    expect(
      workspace.querySelector('[data-panel="viewport-timeline"]'),
    ).not.toBeNull();
    expect(
      workspace.querySelector('[data-panel="animation-inspector"]'),
    ).not.toBeNull();
    expect(
      workspace.querySelector('[role="toolbar"][aria-label="Animation viewport display"]'),
    ).not.toBeNull();
    expect(
      workspace.querySelector('button[aria-label="Show source animation"]'),
    ).not.toBeNull();
    expect(
      workspace.querySelector('button[aria-label="Show edited animation"]'),
    ).not.toBeNull();
    expect(
      workspace.querySelector('select[aria-label="Output rig bone"]'),
    ).not.toBeNull();
    expect(
      workspace.querySelector('button[aria-label="Delete selected keyframes"]'),
    ).not.toBeNull();
    expect(
      workspace.querySelector("details.animation-timeline-advanced"),
    ).not.toBeNull();

    const autosave = required<HTMLElement>(
      workspace.querySelector(".animation-studio-autosave"),
    );
    expect(autosave.getAttribute("role")).toBe("alert");
    expect(autosave.getAttribute("aria-live")).toBe("assertive");
    expect(autosave.textContent).toContain("IndexedDB unavailable");
    const diagnosticAlert = required<HTMLElement>(
      workspace.querySelector(".animation-editor-diagnostics [role=\"alert\"]"),
    );
    expect(diagnosticAlert.getAttribute("aria-live")).toBe("assertive");
    expect(diagnosticAlert.textContent).toContain("Normalize the first key.");
    const documentStatus = required<HTMLElement>(
      workspace.querySelector("footer.animation-studio-workspace__status"),
    );
    expect(documentStatus.getAttribute("role")).toBe("status");
    expect(documentStatus.getAttribute("aria-live")).toBe("polite");
    expect(documentStatus.textContent).toContain("INVALID");

    expect(
      workspace.querySelector(".animation-track-row")?.getAttribute("aria-label"),
    ).toBe("torso ROTATION track");
    const key = required<HTMLButtonElement>(
      workspace.querySelector(".animation-keyframe-marker"),
    );
    expect(key.getAttribute("aria-label")).toBe(
      "torso ROTATION key at 0.000 seconds",
    );
    expect(key.getAttribute("aria-pressed")).toBe("false");
    expect(
      workspace.querySelector('output[aria-label="Current animation time"]'),
    ).not.toBeNull();
  });

  it("keeps one tab stop per composite widget and supports arrow/Escape focus movement", async () => {
    const container = mount();
    function FocusFixture() {
      const [mode, setMode] = useState<"MAP" | "EDIT">("MAP");
      const [boneId, setBoneId] = useState<number | null>(0);
      return (
        <>
          <AnimationMappingModeSwitch value={mode} onChange={setMode} />
          <NewAnimationMenu
            onCreateBlank={vi.fn()}
            onCreateProcedural={vi.fn()}
          />
          <AnimationBoneTree
            nodes={rig}
            selectedNodeId={boneId}
            onSelect={setBoneId}
          />
        </>
      );
    }
    await render(container, <FocusFixture />);

    const tabs = Array.from(
      container.querySelectorAll<HTMLButtonElement>('[role="tab"]'),
    );
    const boneSelect = required<HTMLSelectElement>(
      container.querySelector('select[aria-label="Output rig bone"]'),
    );
    expect(tabs.map(({ tabIndex }) => tabIndex)).toEqual([0, -1]);
    expect(tabbableLabels(container).slice(0, 3)).toEqual([
      "Map animations",
      "+ New animation",
      "Output rig bone",
    ]);

    tabs[0]!.focus();
    await key(tabs[0]!, { key: "ArrowRight" });
    expect(document.activeElement).toBe(tabs[1]);
    expect(tabs[1]!.getAttribute("aria-selected")).toBe("true");

    const menuTrigger = buttonByText(container, "+ New animation");
    menuTrigger.focus();
    await key(menuTrigger, { key: "ArrowDown" });
    const menuItems = Array.from(
      container.querySelectorAll<HTMLButtonElement>('[role="menuitem"]'),
    );
    expect(document.activeElement).toBe(menuItems[0]);
    await key(menuItems[0]!, { key: "ArrowDown" });
    expect(document.activeElement).toBe(menuItems[1]);
    await key(menuItems[1]!, { key: "Escape" });
    expect(document.activeElement).toBe(menuTrigger);

    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value")?.set
        ?.call(boneSelect, "7");
      boneSelect.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(boneSelect.value).toBe("7");
  });

  it("keeps the visible clip list keyboard-reachable when its selected item is filtered out", async () => {
    const container = mount();
    await render(container, (
      <AnimationClipLibrary
        items={[{
          id: "source:hidden-by-custom-filter",
          name: "Source walk",
          origin: "SOURCE",
          status: "SOURCE",
          durationSeconds: 1,
          keyframeCount: 2,
          authoredClipId: null,
          diagnosticCodes: [],
        }, {
          id: "authored-visible",
          name: "Authored wave",
          origin: "EDITED",
          status: "VALID",
          durationSeconds: 1,
          keyframeCount: 3,
          authoredClipId: "authored-visible",
          diagnosticCodes: [],
        }]}
        selectedId="source:hidden-by-custom-filter"
        onSelect={vi.fn()}
        onCreateBlank={vi.fn()}
        onCreateProcedural={vi.fn()}
        onEditCopy={vi.fn()}
        onDuplicate={vi.fn()}
      />
    ));

    const customOption = required<HTMLButtonElement>(
      container.querySelector('[role="listbox"] [role="option"]'),
    );
    expect(customOption.textContent).toContain("Authored wave");
    expect(customOption.tabIndex).toBe(0);
    const tabs = Array.from(
      container.querySelectorAll<HTMLButtonElement>('[role="tab"]'),
    );
    expect(tabs.map(({ tabIndex }) => tabIndex)).toEqual([-1, 0]);
    tabs[1]!.focus();
    await key(tabs[1]!, { key: "ArrowLeft" });
    expect(document.activeElement).toBe(tabs[0]);
    expect(tabs[0]!.getAttribute("aria-selected")).toBe("true");
    const sourceOption = required<HTMLButtonElement>(
      container.querySelector('[role="listbox"] [role="option"]'),
    );
    expect(sourceOption.textContent).toContain("Source walk");
    expect(sourceOption.tabIndex).toBe(0);
  });

  it("commits the gizmo with keyboard controls and closes an auto-focused dialog with Escape", async () => {
    const onBegin = vi.fn();
    const onUpdate = vi.fn();
    const onCommit = vi.fn();
    const onCancel = vi.fn();
    const container = mount();
    await render(container, (
      <AnimationEditorViewport
        viewport={<div />}
        selectedBoneName="torso"
        playheadSeconds={0.5}
        path="ROTATION"
        onGestureBegin={onBegin}
        onGestureUpdate={onUpdate}
        onGestureCommit={onCommit}
        onGestureCancel={onCancel}
      />
    ));
    const gizmo = required<HTMLInputElement>(
      container.querySelector('input[aria-label="Rotation gizmo delta for torso"]'),
    );
    gizmo.focus();
    await key(gizmo, { key: "ArrowRight", type: "keydown" });
    await setInputValue(gizmo, "1");
    await key(gizmo, { key: "ArrowRight", type: "keyup" });
    expect(onBegin).toHaveBeenCalledOnce();
    expect(onUpdate).toHaveBeenCalledWith(1);
    expect(onCommit).toHaveBeenCalledOnce();
    await key(gizmo, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledOnce();
    expect(gizmo.getAttribute("aria-describedby"))
      .toBe("animation-gizmo-keyboard-help");

    const onClose = vi.fn();
    await render(container, (
      <AnimationTrimDialog
        lengthSeconds={1}
        onApply={vi.fn()}
        onClose={onClose}
      />
    ));
    const dialog = required<HTMLElement>(
      container.querySelector('[role="dialog"][aria-modal="true"]'),
    );
    expect(document.activeElement).toBe(dialog.querySelector("input"));
    await key(required(dialog.querySelector("input")), { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("projects reduced-motion preference into the workspace and keeps playback opt-in", async () => {
    const addEventListener = vi.fn();
    const removeEventListener = vi.fn();
    Object.defineProperty(window, "matchMedia", {
      configurable: true,
      writable: true,
      value: vi.fn(() => ({
        matches: true,
        media: "(prefers-reduced-motion: reduce)",
        onchange: null,
        addEventListener,
        removeEventListener,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });
    const container = mount();
    await render(container, (
      <AnimationStudioWorkspace
        document={animationStudioDocumentFixtureV1()}
        authoring={emptyAuthoring()}
        sourceInventory={[]}
        rig={rig}
        viewport={<div />}
        autosaveState={{ kind: "SAVED" }}
        diagnostics={[]}
        onDocumentChange={vi.fn()}
        onAuthoringChange={vi.fn()}
        onUndo={vi.fn()}
        onRedo={vi.fn()}
        canUndo={false}
        canRedo={false}
      />
    ));

    const workspace = required<HTMLElement>(
      container.querySelector(".animation-studio-workspace"),
    );
    expect(workspace.getAttribute("data-reduced-motion")).toBe("true");
    expect(workspace.textContent).toContain(
      "Reduced motion is active. Timeline playback remains user-controlled.",
    );
    expect(buttonByAriaLabel(container, "Play")).not.toBeNull();
    expect(addEventListener).toHaveBeenCalledWith("change", expect.any(Function));
    await act(async () => roots[roots.length - 1]?.unmount());
    roots.pop();
    expect(removeEventListener).toHaveBeenCalledWith("change", expect.any(Function));
  });
});

function emptyAuthoring(): CreatureAnimationAuthoringV2 {
  return {
    schemaVersion: 2,
    profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V2,
    modelType: "S",
    sourceRevision,
    authoringRevision: 1,
    assignments: [],
    fallbacks: [],
    customAnimations: [],
  };
}

function mount() {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  roots.push(root);
  return container;
}

async function render(container: HTMLElement, node: React.ReactNode) {
  const index = Array.from(document.body.children).indexOf(container);
  const root = roots[index];
  if (!root) throw new Error("Missing test root.");
  await act(async () => root.render(node));
}

function required<T>(value: T | null | undefined): T {
  if (value === null || value === undefined) {
    throw new Error("Required accessibility test element is missing.");
  }
  return value;
}

function buttonByText(container: HTMLElement, text: string) {
  return required(Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
    .find((button) => button.textContent?.trim() === text));
}

function buttonByAriaLabel(container: HTMLElement, label: string) {
  return required<HTMLButtonElement>(
    container.querySelector(`button[aria-label="${label}"]`),
  );
}

function tabbableLabels(container: HTMLElement) {
  return Array.from(container.querySelectorAll<HTMLElement>(
    "button, input, select, [tabindex]",
  )).filter((element) => (
    element.tabIndex >= 0
    && !(element instanceof HTMLButtonElement && element.disabled)
    && !(element instanceof HTMLInputElement && element.disabled)
    && !(element instanceof HTMLSelectElement && element.disabled)
  )).map((element) => element.getAttribute("aria-label") || element.textContent?.trim() || "");
}

async function key(
  target: HTMLElement,
  input: KeyboardEventInit & { type?: "keydown" | "keyup" },
) {
  const { type = "keydown", ...init } = input;
  await act(async () => {
    target.dispatchEvent(new KeyboardEvent(type, {
      bubbles: true,
      ...init,
    }));
    await Promise.resolve();
  });
}

async function setInputValue(input: HTMLInputElement, value: string) {
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set
      ?.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
}
