// @vitest-environment jsdom

import { act } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";
import { animationStudioClipFixtureV1 } from "../animation-studio/testFixtures";
import { AnimationSequencePanel } from "./AnimationSequencePanel";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

describe("AnimationSequencePanel", () => {
  it("requests idle-action-idle and shows Core transition evidence", async () => {
    const idle = animationStudioClipFixtureV1({ id: "idle", name: "idle" });
    const attack = animationStudioClipFixtureV1({ id: "attack", name: "attack" });
    const onBuild = vi.fn();
    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => root.render(
      <AnimationSequencePanel
        clips={[idle, attack]}
        selectedClipId="attack"
        preview={{
          schemaVersion: 1,
          sourceRevision: "a".repeat(64),
          segments: [
            { clipId: "idle", clipName: "idle", startSeconds: 0, endSeconds: 1 },
            { clipId: "attack", clipName: "attack", startSeconds: 1, endSeconds: 2 },
            { clipId: "idle", clipName: "idle", startSeconds: 2, endSeconds: 3 },
          ],
          transitionJumps: [{
            fromClipId: "idle",
            toClipId: "attack",
            boundarySeconds: 1,
            maxTranslationDelta: 0.2,
            maxTranslationNodeId: 0,
            maxAngularDeltaRadians: 0.4,
            maxAngularNodeId: 1,
          }],
          previewClip: { ...attack, lengthSeconds: 3 },
          fingerprintSha256: "b".repeat(64),
        }}
        busy={false}
        error={null}
        onBuild={onBuild}
        onClear={vi.fn()}
      />,
    ));

    const previewButton = Array.from(container.querySelectorAll("button"))
      .find((button) => button.textContent?.includes("Preview idle"));
    await act(async () => previewButton?.click());

    expect(onBuild).toHaveBeenCalledWith(["idle", "attack", "idle"]);
    expect(container.textContent).toContain("3.00 s sequence ready");
    expect(container.textContent).toContain("Δp 0.2000");
    await act(async () => root.unmount());
  });
});
