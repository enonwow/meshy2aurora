// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AnimationSourcePicker } from "./AnimationSourcePicker";
import { createCreatureAnimationAuthoringV1 } from "./state";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const roots: Root[] = [];

afterEach(async () => {
  await act(async () => {
    roots.splice(0).forEach((root) => root.unmount());
  });
  document.body.replaceChildren();
});

describe("AnimationSourcePicker", () => {
  it("offers one accessible four-lane realization control and preserves source clips", async () => {
    const onEvent = vi.fn();
    const authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    await act(async () => {
      root.render(
        <AnimationSourcePicker
          slot="cwalk"
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
          onEvent={onEvent}
          customPanel={<div data-testid="custom-panel">Saved Custom picker</div>}
        />,
      );
    });

    const realization = container.querySelector<HTMLElement>(
      '[role="radiogroup"][aria-label="Source realization"]',
    )!;
    const radios = Array.from(
      realization.querySelectorAll<HTMLInputElement>('input[type="radio"]'),
    );
    expect(radios.map(({ value }) => value)).toEqual([
      "SOURCE_CLIP",
      "INHERITED_SUPERMODEL",
      "PROCEDURAL",
      "CUSTOM",
    ]);
    expect(radios.map((radio) => radio.labels?.[0]?.textContent)).toEqual([
      "Source clip",
      "Inherited supermodel",
      "Procedural",
      "Custom",
    ]);
    expect(radios[0]?.checked).toBe(true);
    for (const disabled of radios.slice(1, 3)) {
      expect(disabled.disabled).toBe(true);
      const descriptionId = disabled.getAttribute("aria-describedby");
      expect(descriptionId).toBeTruthy();
      expect(document.getElementById(descriptionId!)?.textContent)
        .toMatch(/not available|not packaged/i);
    }
    expect(radios[3]?.disabled).toBe(false);

    await act(async () => radios[3]!.click());
    expect(radios[3]?.checked).toBe(true);
    expect(container.querySelector('[data-testid="custom-panel"]')).not.toBeNull();
    expect(container.querySelector('select[aria-label="Source clip for cwalk"]'))
      .toBeNull();

    await act(async () => radios[0]!.click());
    const sourceSelect = container.querySelector<HTMLSelectElement>(
      'select[aria-label="Source clip for cwalk"]',
    )!;
    await act(async () => {
      Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value")?.set
        ?.call(sourceSelect, "walk");
      sourceSelect.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(onEvent).toHaveBeenLastCalledWith({
      type: "ANIMATION_SOURCE_ASSIGNED",
      assignment: expect.objectContaining({
        targetSlot: "cwalk",
        sourceKind: "SOURCE_CLIP",
        sourceClipName: "Walk",
        customAnimationId: null,
      }),
    });
  });
});
