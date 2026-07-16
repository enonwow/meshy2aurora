// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import * as THREE from "three";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { BinaryMdlInspectionReport } from "./types";

interface CapturedSceneViewportProps {
  provenance: string;
  buildRoot: () => Promise<THREE.Object3D | { root: THREE.Object3D; animations?: readonly THREE.AnimationClip[] }>;
  tools?: { animationPlayback?: boolean; overlays?: boolean };
  animationUnavailableReason?: string;
}

const { capturedProps } = vi.hoisted(() => ({
  capturedProps: [] as CapturedSceneViewportProps[],
}));

vi.mock("./SceneViewport", () => ({
  SceneViewport: (props: CapturedSceneViewportProps) => {
    capturedProps.push(props);
    return (
      <section aria-label={`${props.provenance} model viewport`}>
        {props.animationUnavailableReason ? <p>{props.animationUnavailableReason}</p> : null}
      </section>
    );
  },
}));

import { AuroraReadbackViewport } from "./AuroraReadbackViewport";

const roots: Root[] = [];

const readback: BinaryMdlInspectionReport = {
  schemaVersion: 1,
  format: "BINARY_MDL",
  nodeTree: {
    roots: [{ offset: 12, number: 1, name: "body", controllers: [], children: [] }],
  },
  diagnostics: [],
};

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  capturedProps.length = 0;
  document.body.replaceChildren();
});

describe("AuroraReadbackViewport", () => {
  it("enables real readback overlays and exposes disabled animation evidence without clips", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    await act(async () => root.render(
      <AuroraReadbackViewport report={readback} onSelectPart={vi.fn()} />,
    ));

    expect(capturedProps).toHaveLength(1);
    const props = capturedProps[0];
    expect(props.tools).toEqual({ overlays: true });
    expect(props.tools?.animationPlayback).not.toBe(true);
    expect(props.animationUnavailableReason).toContain("canonical readback does not expose clip/controller mapping");
    expect(container.textContent).toContain("Converted animation playback is unavailable");

    const asset = await props.buildRoot();
    expect(asset).toBeInstanceOf(THREE.Object3D);
    expect((asset as THREE.Object3D).animations).toEqual([]);
    const clips: THREE.AnimationClip[] = [];
    (asset as THREE.Object3D).traverse((object) => clips.push(...object.animations));
    expect(clips).toEqual([]);
  });
});
