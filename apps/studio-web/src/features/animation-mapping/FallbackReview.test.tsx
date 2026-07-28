// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FallbackReview } from "./FallbackReview";
import { createCreatureAnimationAuthoringV1 } from "./state";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const roots: Root[] = [];

afterEach(async () => {
  await act(async () => {
    roots.splice(0).forEach((root) => root.unmount());
  });
  document.body.replaceChildren();
});

describe("FallbackReview", () => {
  it("blocks only cycle-forming acceptance and links the reason through ARIA", async () => {
    const authoring = createCreatureAnimationAuthoringV1("sha256:source", "S");
    authoring.fallbacks = [
      {
        id: "left-to-right",
        targetSlot: "cdamagel",
        sourceSlot: "cdamager",
        reason: "Mirror left from right",
        review: "PENDING",
      },
      {
        id: "right-to-left",
        targetSlot: "cdamager",
        sourceSlot: "cdamagel",
        reason: "Mirror right from left",
        review: "ACCEPTED",
      },
    ];
    const onEvent = vi.fn();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    await act(async () => {
      root.render(<FallbackReview authoring={authoring} onEvent={onEvent} />);
    });

    const reject = Array.from(container.querySelectorAll("button"))
      .find(({ textContent }) => textContent === "Reject")!;
    const accept = Array.from(container.querySelectorAll("button"))
      .find(({ textContent }) => textContent === "Accept fallback")!;
    const alert = container.querySelector<HTMLElement>('[role="alert"]')!;
    expect(reject.disabled).toBe(false);
    expect(accept.disabled).toBe(true);
    expect(accept.getAttribute("aria-describedby")).toBe(alert.id);

    reject.click();
    expect(onEvent).toHaveBeenCalledWith({
      type: "ANIMATION_FALLBACK_REJECTED",
      fallbackId: "left-to-right",
    });
  });
});
