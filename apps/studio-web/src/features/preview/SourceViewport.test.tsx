// @vitest-environment jsdom

import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { animationStudioClipFixtureV1 } from "../animation-studio/testFixtures";
import { SourceViewport } from "./SourceViewport";

const sceneViewportCapture = vi.hoisted(() => ({
  current: null as Record<string, unknown> | null,
}));

vi.mock("./SceneViewport", () => ({
  SceneViewport: (props: Record<string, unknown>) => {
    sceneViewportCapture.current = props;
    return <div data-testid="scene-viewport" />;
  },
}));

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

afterEach(() => {
  document.body.replaceChildren();
  sceneViewportCapture.current = null;
});

describe("SourceViewport authored playback lifecycle", () => {
  it("keeps the GLB loader stable while replacing only the authored clip", async () => {
    const file = new File(["glb"], "source.glb", {
      type: "model/gltf-binary",
    });
    const firstClip = animationStudioClipFixtureV1({
      id: "clip-first",
      name: "first",
    });
    const secondClip = animationStudioClipFixtureV1({
      id: "clip-second",
      name: "second",
    });
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    const onPlaybackUpdate = vi.fn();
    const render = async (clip: typeof firstClip) => act(async () => root.render(
      <SourceViewport
        input={{
          provenance: "SOURCE",
          file,
          sourceSha256: "a".repeat(64),
        }}
        authoredClip={clip}
        authoredRig={[]}
        controlledAnimationTimeSeconds={0.25}
        controlledAnimationPlayback={{
          playing: true,
          onUpdate: onPlaybackUpdate,
        }}
      />,
    ));

    await render(firstClip);
    const firstProps = sceneViewportCapture.current;
    const firstBuildRoot = firstProps?.buildRoot;
    expect(firstProps).toMatchObject({
      controlledAnimationPlaying: true,
      controlledAnimationTimeSeconds: 0.25,
      hideAnimationControls: true,
      initialAnimationLoop: false,
      initialAnimationName: "first",
    });

    await render(secondClip);
    const secondProps = sceneViewportCapture.current;
    expect(secondProps?.buildRoot).toBe(firstBuildRoot);
    expect(secondProps?.animationOverride).not.toBe(firstProps?.animationOverride);
    expect(secondProps).toMatchObject({
      initialAnimationName: "second",
    });

    await act(async () => root.unmount());
  });
});
