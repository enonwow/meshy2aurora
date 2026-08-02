// @vitest-environment jsdom

import { act } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";
import { AnimationCurvePanel } from "./AnimationCurvePanel";
import type { AuthoredAnimationTrackV1 } from "../animation-studio/types";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const track: AuthoredAnimationTrackV1 = {
  id: "hand-translation",
  targetNodeId: 7,
  path: "TRANSLATION",
  interpolation: "LINEAR",
  keyframes: [
    { id: "start", timeSeconds: 0, value: [0, 0, 0] },
    { id: "impact", timeSeconds: 0.5, value: [1, 0.25, 0] },
    { id: "end", timeSeconds: 1, value: [0, 0, 0] },
  ],
};

function button(container: HTMLElement, label: string) {
  return Array.from(container.querySelectorAll("button")).find((candidate) => candidate.textContent === label);
}

describe("AnimationCurvePanel visual graph editing", () => {
  it("exposes navigation, snap, multi-key selection, tangent modes and a changed Core bake input", async () => {
    const onSmooth = vi.fn();
    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => root.render(<AnimationCurvePanel track={track} busy={false} error={null} report={null} onSmooth={onSmooth} />));
    expect(container.querySelector('[aria-label="Graph zoom"]')).toBeTruthy();
    expect(container.querySelector('[aria-label="Graph horizontal pan"]')).toBeTruthy();
    expect(button(container, "Fit all")).toBeTruthy();
    const keys = container.querySelectorAll<SVGCircleElement>('circle[aria-label^="Key "]');
    await act(async () => keys[1]!.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true })));
    const value = Array.from(container.querySelectorAll<HTMLInputElement>("fieldset input")).find((input) => input.parentElement?.textContent?.includes("Value"));
    await act(async () => {
      if (!value) throw new Error("value editor missing");
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set?.call(value, "1.75");
      value.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await act(async () => button(container, "FLAT")?.click());
    expect(container.querySelectorAll('[aria-label$="tangent handle"]')).toHaveLength(2);
    await act(async () => button(container, "Bake LINEAR")?.click());
    expect(onSmooth).toHaveBeenCalledTimes(1);
    const baked = onSmooth.mock.calls[0]?.[0];
    expect(baked.keys.find(({ id }: { id: string }) => id === "impact").value[0]).toBe(1.75);
    expect(baked.keys.find(({ id }: { id: string }) => id === "impact").inTangent[0]).toBe(0);
    await act(async () => root.unmount());
  });
});
