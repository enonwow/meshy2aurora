// @vitest-environment jsdom

import { act } from "react";
import { createRoot } from "react-dom/client";
import { describe, expect, it, vi } from "vitest";
import { animationStudioClipFixtureV1 } from "../animation-studio/testFixtures";
import { AnimationWorkbenchPanel } from "./AnimationWorkbenchPanel";
import { emptyAnimationWorkbenchProjectStateV1 } from "./animationWorkbench";
import type {
  AnimationWorkbenchRequestV1,
  AnimationWorkbenchResultV1,
} from "./animationWorkbench";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const sourceRevision = "a".repeat(64);
const rig = [
  { id: 1, name: "Root", parentId: null, translation: [0, 0, 0] as const, rotation: [0, 0, 0, 1] as const },
  { id: 2, name: "LeftHand", parentId: 1, translation: [-1, 0, 0] as const, rotation: [0, 0, 0, 1] as const },
  { id: 3, name: "RightHand", parentId: 1, translation: [1, 0, 0] as const, rotation: [0, 0, 0, 1] as const },
];

function button(container: HTMLElement, label: string) {
  return Array.from(container.querySelectorAll("button"))
    .find((candidate) => candidate.textContent?.includes(label));
}

describe("AnimationWorkbenchPanel", () => {
  it("sends exact multi-bone pose selection through Core capture and paste", async () => {
    const clip = animationStudioClipFixtureV1({ id: "attack", name: "attack" });
    const requests: AnimationWorkbenchRequestV1[] = [];
    const onReplaceClip = vi.fn();
    const onOperation = vi.fn(async (request: AnimationWorkbenchRequestV1) => {
      requests.push(request);
      if (request.operation === "CAPTURE_POSE") {
        return {
          result: "POSE_CAPTURED",
          value: {
            schemaVersion: 1 as const,
            sourceRevision,
            rigSignatureSha256: "b".repeat(64),
            clipId: clip.id,
            clipRevision: clip.revision,
            timeSeconds: 0,
            bones: request.selectedNodeIds.map((nodeId) => ({
              nodeId,
              nodeName: rig.find((node) => node.id === nodeId)!.name,
              translation: [0, 0, 0] as const,
              rotation: [0, 0, 0, 1] as const,
            })),
            fingerprintSha256: "c".repeat(64),
          },
        } satisfies AnimationWorkbenchResultV1;
      }
      if (request.operation === "APPLY_POSE") {
        return {
          result: "POSE_APPLIED",
          value: {
            schemaVersion: 1 as const,
            mode: request.mode,
            changedNodeIds: [...request.selectedNodeIds],
            clip: { ...clip, revision: clip.revision + 1 },
            fingerprintSha256: "d".repeat(64),
          },
        } satisfies AnimationWorkbenchResultV1;
      }
      throw new Error(`unexpected ${request.operation}`);
    });
    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => root.render(
      <AnimationWorkbenchPanel
        sourceRevision={sourceRevision}
        clip={clip}
        clips={[clip]}
        rig={rig}
        playheadSeconds={0}
        onOperation={onOperation}
        onReplaceClip={onReplaceClip}
        onAddClip={vi.fn()}
        onPreviewClip={vi.fn()}
        onVisualization={vi.fn()}
        projectState={{ ...emptyAnimationWorkbenchProjectStateV1(), sourceRevision }}
        onProjectStateChange={vi.fn()}
      />,
    ));
    const checks = container.querySelectorAll<HTMLInputElement>(
      '[aria-label="Pose bone multi-select"] input[type="checkbox"]',
    );
    await act(async () => {
      checks[1]!.click();
      checks[2]!.click();
    });
    await act(async () => button(container, "Copy pose")?.click());
    await act(async () => button(container, "Paste pose")?.click());

    expect(requests[0]).toMatchObject({
      operation: "CAPTURE_POSE",
      selectedNodeIds: [2, 3],
    });
    expect(requests[1]).toMatchObject({
      operation: "APPLY_POSE",
      mode: "SELECTED_BONES",
      selectedNodeIds: [2, 3],
    });
    expect(onReplaceClip).toHaveBeenCalledTimes(1);
    await act(async () => root.unmount());
  });

  it("previews a four-segment sequence before atomically adding its Custom clip", async () => {
    const clips = Array.from({ length: 4 }, (_, index) => animationStudioClipFixtureV1({
      id: `clip-${index}`,
      name: `clip-${index}`,
    }));
    const custom = { ...clips[0]!, id: "sequence-custom", name: "Custom sequence" };
    const onAddClip = vi.fn();
    const onPreviewClip = vi.fn();
    const onProjectStateChange = vi.fn();
    const onOperation = vi.fn(async (request: AnimationWorkbenchRequestV1) => {
      if (request.operation !== "BAKE_SEQUENCE") throw new Error("unexpected operation");
      expect(request.document.segments).toHaveLength(4);
      return {
        result: "SEQUENCE_BAKED",
        value: {
          schemaVersion: 1,
          sourceRevision,
          documentRevision: 2,
          previewClip: { ...custom, id: "sequence-preview" },
          customClip: custom,
          placedSegments: request.document.segments.map((segment, index) => ({
            segmentId: segment.segmentId,
            clipId: segment.clipId,
            sourceInSeconds: 0,
            sourceOutSeconds: 1,
            startSeconds: index,
            endSeconds: index + 1,
            repeatCount: 1,
            transitionFromPrevious: segment.transitionFromPrevious,
          })),
          phaseMarkers: [],
          sourceProvenance: request.document.segments.map((segment) => ({
            clipId: segment.clipId,
            clipRevision: 1,
            clipFingerprintSha256: "f".repeat(64),
          })),
          fingerprintSha256: "e".repeat(64),
        },
      } satisfies AnimationWorkbenchResultV1;
    });
    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => root.render(
      <AnimationWorkbenchPanel
        sourceRevision={sourceRevision}
        clip={clips[0]!}
        clips={clips}
        rig={rig}
        playheadSeconds={0}
        onOperation={onOperation}
        onReplaceClip={vi.fn()}
        onAddClip={onAddClip}
        onPreviewClip={onPreviewClip}
        onVisualization={vi.fn()}
        projectState={{ ...emptyAnimationWorkbenchProjectStateV1(), sourceRevision }}
        onProjectStateChange={onProjectStateChange}
      />,
    ));
    await act(async () => button(container, "Bake + preview")?.click());
    expect(onPreviewClip).toHaveBeenCalledWith(expect.objectContaining({ id: "sequence-preview" }));
    expect(onAddClip).not.toHaveBeenCalled();
    expect(onProjectStateChange).toHaveBeenCalledTimes(1);
    await act(async () => button(container, "Save as Custom")?.click());
    expect(onAddClip).toHaveBeenCalledWith(custom);
    await act(async () => root.unmount());
  });

  it("uses editable attack templates and a reduced-motion revision-bound overlay request", async () => {
    const clip = animationStudioClipFixtureV1({ id: "overlay-attack", name: "overlay_attack", lengthSeconds: 2 });
    const requests: AnimationWorkbenchRequestV1[] = [];
    const onVisualization = vi.fn();
    const onPreviewClip = vi.fn();
    const onOperation = vi.fn(async (request: AnimationWorkbenchRequestV1): Promise<AnimationWorkbenchResultV1> => {
      requests.push(request);
      if (request.operation === "CREATE_COMBAT_PHASES") return {
        result: "COMBAT_PHASES_CREATED",
        value: {
          schemaVersion: 1,
          sourceRevision,
          clipId: clip.id,
          clipRevision: clip.revision,
          markers: (["READY", "WIND_UP", "STRIKE", "IMPACT", "RECOVERY"] as const).map((phase, index) => ({ phase, timeSeconds: request.phaseTimes[index]! })),
          fingerprintSha256: "1".repeat(64),
        },
      };
      if (request.operation === "BUILD_MOTION_VISUALIZATION") return {
        result: "MOTION_VISUALIZATION_BUILT",
        value: {
          schemaVersion: 1,
          sourceRevision,
          clipId: clip.id,
          clipRevision: clip.revision,
          trails: [],
          onionPoses: [],
          fingerprintSha256: "2".repeat(64),
        },
      };
      throw new Error(`unexpected ${request.operation}`);
    });
    const container = document.createElement("div");
    const root = createRoot(container);
    await act(async () => root.render(<AnimationWorkbenchPanel
      sourceRevision={sourceRevision}
      clip={clip}
      clips={[clip]}
      rig={rig}
      playheadSeconds={0.5}
      onOperation={onOperation}
      onReplaceClip={vi.fn()}
      onAddClip={vi.fn()}
      onPreviewClip={onPreviewClip}
      onVisualization={onVisualization}
      projectState={{ ...emptyAnimationWorkbenchProjectStateV1(), sourceRevision }}
      onProjectStateChange={vi.fn()}
    />));
    const template = Array.from(container.querySelectorAll("select")).find((select) => Array.from(select.options).some((option) => option.value === "OVERHEAD"));
    await act(async () => {
      if (!template) throw new Error("phase template select missing");
      template.value = "OVERHEAD";
      template.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await act(async () => button(container, "Save phase timeline")?.click());
    expect(requests.at(-1)).toMatchObject({ operation: "CREATE_COMBAT_PHASES", phaseTimes: [0, 0.76, 1.16, 1.44, 2] });
    await act(async () => button(container, "Preview IMPACT")?.click());
    expect(onPreviewClip).toHaveBeenCalledWith(expect.objectContaining({
      kind: "STATIC_POSE",
      name: expect.stringContaining("impact pose"),
      events: [],
    }));
    await act(async () => button(container, "Preview full one-shot")?.click());
    expect(onPreviewClip).toHaveBeenLastCalledWith(clip);
    const reduced = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')).find((input) => input.parentElement?.textContent?.includes("Reduced motion"));
    await act(async () => reduced?.click());
    await act(async () => button(container, "Show trails + onion skin")?.click());
    expect(requests.at(-1)).toMatchObject({
      operation: "BUILD_MOTION_VISUALIZATION",
      request: { sampleCount: 45, onionBefore: 0, onionAfter: 0, clipRevision: clip.revision },
    });
    expect(onVisualization).toHaveBeenCalledWith(expect.objectContaining({ clipId: clip.id }));
    await act(async () => button(container, "Show trails + onion skin")?.click());
    expect(requests.filter(({ operation }) => operation === "BUILD_MOTION_VISUALIZATION")).toHaveLength(1);
    const overlays = Array.from(container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')).find((input) => input.parentElement?.textContent?.includes("Motion overlays"));
    await act(async () => overlays?.click());
    expect(onVisualization).toHaveBeenLastCalledWith(null);
    await act(async () => root.unmount());
  });
});
