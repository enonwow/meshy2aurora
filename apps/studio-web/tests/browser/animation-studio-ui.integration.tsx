import { act, useState } from "react";
import { createRoot } from "react-dom/client";
import { userEvent } from "vitest/browser";
import { describe, expect, it } from "vitest";
import type {
  AuthoredAnimationClipV1,
  AnimationStudioDocumentV1,
  CreatureAnimationAuthoringV2,
} from "../../src/features/animation-studio/types";
import {
  deleteAnimationStudioDocumentV1,
  loadAnimationStudioDocumentV1,
  openAnimationStudioDatabaseV1,
  saveAnimationStudioDocumentV1,
} from "../../src/features/animation-studio/persistence";
import {
  AnimationStudioWorkspace,
} from "../../src/features/animation-editor/AnimationStudioWorkspace";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

const sourceRevision = "a".repeat(64);

describe("Animation Studio UI in a real browser", () => {
  it("creates, edits, saves and exposes the clip in Custom without JSON", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const root = createRoot(host);
    let latestDocument = emptyDocument();
    let latestAuthoring = emptyAuthoring();

    function Harness() {
      const [studio, setStudio] = useState(emptyDocument());
      const [authoring, setAuthoring] = useState(emptyAuthoring());
      latestDocument = studio;
      latestAuthoring = authoring;
      return (
        <AnimationStudioWorkspace
          document={studio}
          authoring={authoring}
          sourceInventory={[]}
          rig={[{
            id: 0,
            name: "root",
            parentId: null,
            translation: [0, 0, 0],
            rotation: [0, 0, 0, 1],
          }]}
          viewport={(_clip, time) => (
            <output data-testid="browser-playhead">{time.toFixed(3)}</output>
          )}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={setStudio}
          onAuthoringChange={setAuthoring}
          onUndo={() => undefined}
          onRedo={() => undefined}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    try {
      await act(async () => root.render(<Harness />));
      await click(button(host, "+ New animation"));
      await click(button(host, "From current pose"));
      expect(latestDocument.authoredClips).toHaveLength(1);
      expect(latestDocument.authoredClips[0]).toMatchObject({
        status: "DRAFT",
        source: { kind: "BLANK_POSE" },
      });

      const rotationX = host.querySelector<HTMLInputElement>(
        '.bone-transform-inspector input[type="number"]',
      );
      if (!rotationX) throw new Error("rotation X field is missing");
      await act(async () => {
        const setter = Object.getOwnPropertyDescriptor(
          HTMLInputElement.prototype,
          "value",
        )?.set;
        setter?.call(rotationX, "20");
        rotationX.dispatchEvent(new Event("input", { bubbles: true }));
        rotationX.dispatchEvent(new Event("change", { bubbles: true }));
      });
      await click(button(host, "+ Add keyframe"));
      await click(button(host, "Save to Custom"));

      expect(latestDocument.status).toBe("VALID");
      expect(latestDocument.authoredClips[0]).toMatchObject({
        status: "VALID",
      });
      expect(latestAuthoring.customAnimations).toHaveLength(1);
      expect(latestAuthoring.customAnimations[0]).toMatchObject({
        playback: "ONE_SHOT",
        clipReference: {
          sourceKind: "AUTHORED_CLIP",
          authoredClipId: latestDocument.authoredClips[0]!.id,
        },
      });
      expect(host.textContent).toContain("VALID");
      expect(host.textContent).toContain("Source GLB stays unchanged");
      expect(host.querySelectorAll(
        'input[aria-label="Animation playhead"]',
      )).toHaveLength(1);
    } finally {
      await act(async () => root.unmount());
      host.remove();
    }
  });

  it("creates an editable copy of a source clip without mutating source inventory", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const root = createRoot(host);
    const sourceInventory = [{
      clipId: "source-idle",
      name: "idle",
      durationSeconds: 1.25,
      trackCount: 1,
    }] as const;
    let latestDocument = emptyDocument();

    function Harness() {
      const [studio, setStudio] = useState(emptyDocument());
      latestDocument = studio;
      return (
        <AnimationStudioWorkspace
          document={studio}
          authoring={emptyAuthoring()}
          sourceInventory={sourceInventory}
          rig={rig()}
          viewport={<div />}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={setStudio}
          onAuthoringChange={() => undefined}
          onEditSourceClip={async (sourceClipId, newId, newName) => (
            sourceCopy(sourceClipId, newId, newName)
          )}
          onUndo={() => undefined}
          onRedo={() => undefined}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    try {
      await act(async () => root.render(<Harness />));
      await focusAndTypeKeys(button(host, "Edit copy"), "{Enter}");
      await act(async () => Promise.resolve());

      expect(sourceInventory).toEqual([{
        clipId: "source-idle",
        name: "idle",
        durationSeconds: 1.25,
        trackCount: 1,
      }]);
      expect(latestDocument.authoredClips).toHaveLength(1);
      expect(latestDocument.authoredClips[0]).toMatchObject({
        id: expect.stringMatching(/^authored-/),
        name: "idle_edited",
        status: "DRAFT",
        source: {
          kind: "SOURCE_CLIP_COPY",
          sourceClipName: "idle",
          sourceRevision,
        },
      });
    } finally {
      await act(async () => root.unmount());
      host.remove();
    }
  });

  it("supports basic authoring with the keyboard and restores the exact saved project after refresh", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const root = createRoot(host);
    let latestDocument = emptyDocument();
    let latestAuthoring = emptyAuthoring();
    const projectId = `browser-keyboard-refresh-${crypto.randomUUID()}`;
    const database = await openAnimationStudioDatabaseV1();

    function Harness({
      initialDocument = emptyDocument(),
    }: {
      initialDocument?: AnimationStudioDocumentV1;
    }) {
      const [studio, setStudio] = useState(initialDocument);
      const [authoring, setAuthoring] = useState(emptyAuthoring());
      latestDocument = studio;
      latestAuthoring = authoring;
      return (
        <AnimationStudioWorkspace
          document={studio}
          authoring={authoring}
          sourceInventory={[]}
          rig={rig()}
          viewport={(_clip, time) => (
            <output data-testid="keyboard-playhead">{time.toFixed(3)}</output>
          )}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={setStudio}
          onAuthoringChange={setAuthoring}
          onUndo={() => undefined}
          onRedo={() => undefined}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    try {
      await act(async () => root.render(<Harness />));
      await focusAndTypeKeys(button(host, "+ New animation"), "{ArrowDown}");
      expect(document.activeElement?.textContent?.trim()).toBe("From current pose");
      await focusAndTypeKeys(
        document.activeElement as HTMLButtonElement,
        "{Enter}",
      );
      expect(latestDocument.authoredClips).toHaveLength(1);

      const outputName = labelledInput(host, "Output name");
      await focusAndTypeKeys(
        outputName,
        "{Control>}a{/Control}keyboard_authored_pose",
      );
      expect(latestDocument.authoredClips[0]?.name).toBe(
        "keyboard_authored_pose",
      );

      await focusAndTypeKeys(button(host, "+ Event"), "{Enter}");
      const eventName = labelledInput(host, "Callback name");
      await focusAndTypeKeys(eventName, "impact");
      await focusAndTypeKeys(button(host, "Close event editor"), "{Enter}");
      expect(latestDocument.authoredClips[0]?.events).toMatchObject([{
        name: "impact",
        timeSeconds: 0,
      }]);

      await focusAndTypeKeys(button(host, "Save to Custom"), "{Enter}");
      expect(latestDocument.authoredClips[0]?.status).toBe("VALID");
      expect(latestAuthoring.customAnimations).toHaveLength(1);

      await expect(saveAnimationStudioDocumentV1(
        projectId,
        latestDocument,
        database,
        {
          selectedMode: "CREATE_EDIT",
          selectedClipId: latestDocument.authoredClips[0]!.id,
        },
      )).resolves.toMatchObject({ kind: "SAVED" });
      const exactJson = JSON.stringify(latestDocument);

      await act(async () => root.unmount());
      host.replaceChildren();
      const restored = await loadAnimationStudioDocumentV1(projectId, database);
      expect(restored).toMatchObject({
        kind: "LOADED",
        selectedMode: "CREATE_EDIT",
        selectedClipId: latestDocument.authoredClips[0]!.id,
      });
      if (restored.kind !== "LOADED") {
        throw new Error("Animation Studio project did not restore after refresh");
      }
      expect(JSON.stringify(restored.value)).toBe(exactJson);

      const restoredRoot = createRoot(host);
      await act(async () => restoredRoot.render(
        <Harness initialDocument={restored.value} />,
      ));
      expect(host.textContent).toContain("keyboard_authored_pose");
      expect(host.textContent).toContain("VALID");
      await act(async () => restoredRoot.unmount());
    } finally {
      await deleteAnimationStudioDocumentV1(projectId, true, database);
      database.adapter.close?.();
      if (host.childNodes.length > 0) {
        await act(async () => root.unmount());
      }
      host.remove();
    }
  });
});

async function click(element: HTMLButtonElement) {
  await act(async () => element.click());
}

function button(host: HTMLElement, label: string) {
  const candidate = [...host.querySelectorAll<HTMLButtonElement>("button")]
    .find((element) => (
      element.textContent?.trim() === label
      || element.getAttribute("aria-label") === label
    ));
  if (!candidate) throw new Error(`button not found: ${label}`);
  return candidate;
}

function emptyDocument(): AnimationStudioDocumentV1 {
  return {
    schemaVersion: 1,
    sourceRevision,
    authoringRevision: 1,
    status: "DRAFT",
    authoredClips: [],
  };
}

function emptyAuthoring(): CreatureAnimationAuthoringV2 {
  return {
    schemaVersion: 2,
    profile: "DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1",
    modelType: "S",
    sourceRevision,
    authoringRevision: 1,
    assignments: [],
    fallbacks: [],
    customAnimations: [],
  };
}

function rig() {
  return [{
    id: 0,
    name: "root",
    parentId: null,
    translation: [0, 0, 0] as [number, number, number],
    rotation: [0, 0, 0, 1] as [number, number, number, number],
  }];
}

function sourceCopy(
  sourceClipId: string,
  id: string,
  name: string,
): AuthoredAnimationClipV1 {
  return {
    id,
    name,
    kind: "STATIC_POSE",
    status: "DRAFT",
    source: {
      kind: "SOURCE_CLIP_COPY",
      sourceRevision,
      sourceClipName: "idle",
      sourceClipFingerprint: sourceClipId,
      proceduralTemplate: null,
    },
    lengthSeconds: 1.25,
    transitionSeconds: 0.1,
    animationRoot: "root",
    tracks: [{
      id: "track-root-rotation",
      targetNodeId: 0,
      path: "ROTATION",
      interpolation: "LINEAR",
      keyframes: [{
        id: "key-source-0",
        timeSeconds: 0,
        value: [0, 0, 0, 1],
      }],
    }],
    events: [],
    revision: 1,
  };
}

function labelledInput(host: HTMLElement, text: string) {
  const label = [...host.querySelectorAll<HTMLLabelElement>("label")]
    .find(({ textContent }) => textContent?.includes(text));
  const input = label?.querySelector<HTMLInputElement>("input");
  if (!input) throw new Error(`input not found: ${text}`);
  return input;
}

async function focusAndTypeKeys(element: HTMLElement, keys: string) {
  element.focus();
  await act(async () => userEvent.keyboard(keys));
}
