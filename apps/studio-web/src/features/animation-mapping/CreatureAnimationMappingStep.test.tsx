// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import type {
  AnimationStudioDocumentV1,
  CreatureAnimationAuthoringV2,
} from "../animation-studio/types";
import { createCreatureAnimationAuthoringV1 } from "./state";
import { CreatureAnimationMappingStep } from "./CreatureAnimationMappingStep";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const roots: Root[] = [];

afterEach(async () => {
  await act(async () => {
    roots.splice(0).forEach((root) => root.unmount());
  });
  document.body.replaceChildren();
  window.localStorage.clear();
  window.history.replaceState(null, "", "/");
});

describe("CreatureAnimationMappingStep", () => {
  it("renders the Base 42 workspace, derived fields and guarded navigation", async () => {
    const onBack = vi.fn();
    const onContinue = vi.fn();
    const onApplySuggestions = vi.fn();
    const authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
    authoring.assignments.push({
      targetSlot: "cwalk",
      sourceKind: "SOURCE_CLIP",
      sourceClipName: "Walk",
      customAnimationId: null,
      provenance: {
        provider: "SOURCE_GLB",
        assetId: "source",
        ownership: "USER_OWNED",
      },
    });
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    await act(async () => {
      root.render(
        <CreatureAnimationMappingStep
          authoring={authoring}
          inspection={{
            sourceClips: [{
              clipId: "walk",
              name: "Walk",
              durationSeconds: 1,
              trackCount: 2,
              targetNodeIds: [1],
              targetPaths: ["Hips.translation"],
            }],
          }}
          saveState={{ kind: "SAVED" }}
          onBack={onBack}
          onContinue={onContinue}
          onApplySuggestions={onApplySuggestions}
          canContinue={false}
        />,
      );
    });

    expect(container.querySelector("h1")?.textContent).toBe("Creature Animation Mapping");
    expect(container.textContent).toContain("42 Aurora base slots");
    const tabs = Array.from(container.querySelectorAll<HTMLButtonElement>(
      '.animation-catalog__tabs [role="tab"]',
    ));
    expect(tabs.map(({ textContent }) => textContent)).toEqual([
      expect.stringMatching(/Needs attention/),
      expect.stringMatching(/Base 42/),
      expect.stringMatching(/Custom/),
    ]);

    tabs[0].focus();
    await act(async () => {
      tabs[0].dispatchEvent(new KeyboardEvent("keydown", {
        key: "ArrowRight",
        bubbles: true,
      }));
    });
    expect(document.activeElement).toBe(tabs[1]);
    expect(tabs[1].getAttribute("aria-selected")).toBe("true");

    await act(async () => tabs[1].click());
    expect(container.querySelectorAll('[role="option"]')).toHaveLength(42);
    const firstRow = container.querySelector<HTMLButtonElement>('[role="option"]')!;
    await act(async () => {
      firstRow.focus();
      firstRow.dispatchEvent(new KeyboardEvent("keydown", {
        key: "End",
        bubbles: true,
      }));
    });
    expect(document.activeElement).toBe(
      container.querySelectorAll('[role="option"]')[41],
    );
    const search = container.querySelector<HTMLInputElement>(
      'input[aria-label="Search animation states"]',
    )!;
    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set
        ?.call(search, "cwalk");
      search.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(container.querySelectorAll('[role="option"]')).toHaveLength(5);
    await act(async () => {
      Array.from(container.querySelectorAll<HTMLButtonElement>('[role="option"]'))
        .find((button) => button.querySelector("small")?.textContent === "cwalk")!
        .click();
    });
    expect(container.textContent).toContain("aurora.direct-creature.locomotion.walk");
    expect(container.textContent).toContain("MODELTYPE S");
    expect(container.textContent).toContain("ENGINE_MANAGED");
    expect(container.textContent).toContain(
      "Preview is local inspection evidence, not proof of behavior in Aurora Toolset or NWN.",
    );

    const continueButton = Array.from(container.querySelectorAll("button"))
      .find(({ textContent }) => textContent === "Continue to Build")!;
    expect(continueButton.disabled).toBe(true);
    continueButton.click();
    expect(onContinue).not.toHaveBeenCalled();

    Array.from(container.querySelectorAll("button"))
      .find(({ textContent }) => textContent === "Back to Inspect")!
      .click();
    expect(onBack).toHaveBeenCalledOnce();
    Array.from(container.querySelectorAll("button"))
      .find(({ textContent }) => textContent === "Apply safe suggestions")!
      .click();
    expect(onApplySuggestions).toHaveBeenCalledOnce();
  });

  it("keeps Create & edit inside step 3 and restores the deep-linked sub-mode", async () => {
    window.history.replaceState(null, "", "/?animationMode=edit");
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    await act(async () => {
      root.render(
        <CreatureAnimationMappingStep
          authoring={createCreatureAnimationAuthoringV1("sha256:source", "S")}
          inspection={{ sourceClips: [] }}
          saveState={{ kind: "SAVED" }}
          canContinue={false}
          onBack={vi.fn()}
          onContinue={vi.fn()}
          onApplySuggestions={vi.fn()}
          animationStudio={<div data-testid="animation-studio">Editor workspace</div>}
        />,
      );
    });
    expect(container.querySelector('[data-testid="animation-studio"]')).not.toBeNull();
    expect(container.textContent).toContain("Creature Animation Mapping");
    expect(container.textContent).not.toContain("Continue to Build");
    expect(window.localStorage.getItem("m2a.animationMapping.mode.v1")).toBe("EDIT");
    expect(new URL(window.location.href).searchParams.get("animationMode")).toBe("edit");

    const mapButton = Array.from(container.querySelectorAll<HTMLButtonElement>('[role="tab"]'))
      .find(({ textContent }) => textContent?.includes("Map animations"))!;
    await act(async () => mapButton.click());
    expect(container.querySelector('[data-testid="animation-studio"]')).toBeNull();
    expect(container.textContent).toContain("Continue to Build");
  });

  it("opens the saved Custom picker as the fourth source realization without manual IDs", async () => {
    const sourceRevision = "a".repeat(64);
    const studio: AnimationStudioDocumentV1 = {
      schemaVersion: 1,
      sourceRevision,
      authoringRevision: 1,
      status: "VALID",
      authoredClips: [],
    };
    const authoringV2: CreatureAnimationAuthoringV2 = {
      schemaVersion: 2,
      profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
      modelType: "S",
      sourceRevision,
      authoringRevision: 1,
      assignments: [],
      fallbacks: [],
      customAnimations: [],
    };
    const onCreate = vi.fn();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    await act(async () => {
      root.render(
        <CreatureAnimationMappingStep
          authoring={createCreatureAnimationAuthoringV1(sourceRevision, "S")}
          inspection={{ sourceClips: [] }}
          saveState={{ kind: "SAVED" }}
          canContinue={false}
          onBack={vi.fn()}
          onContinue={vi.fn()}
          onApplySuggestions={vi.fn()}
          onAuthoringEvent={vi.fn()}
          animationStudioDocument={studio}
          animationAuthoringV2={authoringV2}
          onAnimationAuthoringV2Change={vi.fn()}
          onCreateAuthoredAnimation={onCreate}
          onOpenAuthoredAnimation={vi.fn()}
        />,
      );
    });

    expect(container.querySelector(".custom-animation-mapping-panel")).toBeNull();
    const realization = container.querySelector<HTMLElement>(
      '[role="radiogroup"][aria-label="Source realization"]',
    )!;
    const custom = realization.querySelector<HTMLInputElement>(
      'input[value="CUSTOM"]',
    )!;
    await act(async () => custom.click());

    expect(custom.checked).toBe(true);
    expect(container.querySelector(".custom-animation-mapping-panel")).not.toBeNull();
    expect(container.querySelector('[aria-label="Saved Custom animations"]'))
      .not.toBeNull();
    expect(container.querySelector('input[aria-label="Custom animation ID"]'))
      .toBeNull();
    expect(container.querySelector('input[aria-label="Custom animation name"]'))
      .toBeNull();

    const create = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find(({ textContent }) => textContent === "+ Create new animation")!;
    await act(async () => create.click());
    expect(onCreate).toHaveBeenCalledOnce();
  });
});
