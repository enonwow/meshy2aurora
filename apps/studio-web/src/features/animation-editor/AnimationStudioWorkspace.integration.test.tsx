// @vitest-environment jsdom

import { act, useState } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  CREATURE_ANIMATION_AUTHORING_PROFILE_V2,
  type AnimationStudioDiagnosticV1,
  type AnimationStudioDocumentV1,
  type CreatureAnimationAuthoringV2,
} from "../animation-studio/types";
import {
  commitAnimationStudioDocumentV1,
  createAnimationStudioStateV1,
  redoAnimationStudioEditV1,
  undoAnimationStudioEditV1,
} from "../animation-studio/state";
import {
  animationStudioClipFixtureV1,
  animationStudioDocumentFixtureV1,
} from "../animation-studio/testFixtures";
import type { DirectCreatureBaseSlotV1 } from "../animation-mapping/types";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";
import { AnimationDopeSheet } from "./AnimationDopeSheet";
import { AnimationStudioWorkspace } from "./AnimationStudioWorkspace";
import { CustomAnimationMappingPanel } from "./CustomAnimationMappingPanel";
import { createCustomDefinitionFromAuthoredClipV1 } from "./editing";

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

const roots: Root[] = [];
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
  vi.restoreAllMocks();
});

describe("AnimationStudioWorkspace integration", () => {
  it("copies sequential compatible donors into one Custom library without losing provenance", async () => {
    let latestDocument = emptyDocument();
    const donorRevision = "b".repeat(64);
    const secondDonorRevision = "d".repeat(64);
    const importedClip = animationStudioClipFixtureV1({
      id: "authored-imported-walk",
      name: "walk_imported",
      source: {
        kind: "IMPORTED_MODEL_COPY",
        sourceRevision: donorRevision,
        sourceClipName: "walk",
        sourceClipFingerprint: "c".repeat(64),
        proceduralTemplate: null,
      },
    });
    const secondImportedClip = animationStudioClipFixtureV1({
      id: "authored-imported-attack",
      name: "attack_imported",
      source: {
        kind: "IMPORTED_MODEL_COPY",
        sourceRevision: secondDonorRevision,
        sourceClipName: "attack",
        sourceClipFingerprint: "e".repeat(64),
        proceduralTemplate: null,
      },
    });
    const inspect = vi.fn().mockImplementation(async (file: File) => (
      file.name === "donor-two.glb"
        ? {
            sourceRevision: secondDonorRevision,
            rig,
            clips: [{ name: "attack", durationSeconds: 0.75, trackCount: 9 }],
          }
        : {
            sourceRevision: donorRevision,
            rig,
            clips: [{ name: "walk", durationSeconds: 1.25, trackCount: 8 }],
          }
    ));
    const importClip = vi.fn().mockImplementation(async (file: File) => (
      file.name === "donor-two.glb" ? secondImportedClip : importedClip
    ));
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studio, setStudio] = useState(emptyDocument());
      latestDocument = studio;
      return (
        <AnimationStudioWorkspace
          document={studio}
          authoring={emptyAuthoring()}
          sourceInventory={[]}
          rig={rig}
          viewport={<div />}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={setStudio}
          onAuthoringChange={vi.fn()}
          onInspectAnimationModel={inspect}
          onImportAnimationModelClip={importClip}
          onUndo={vi.fn()}
          onRedo={vi.fn()}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(buttonByText(container, "+ New animation"));
    await click(buttonByText(container, "Copy from another model…"));
    expect(container.querySelector('[role="dialog"]')?.textContent)
      .toContain("Copy animation from another model");

    const file = new File(["glb"], "donor.glb", {
      type: "model/gltf-binary",
    });
    const input = required<HTMLInputElement>(
      container.querySelector('.animation-import-dialog input[type="file"]'),
    );
    await act(async () => {
      Object.defineProperty(input, "files", {
        configurable: true,
        value: [file],
      });
      input.dispatchEvent(new Event("change", { bubbles: true }));
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(inspect).toHaveBeenCalledWith(file);
    expect(container.textContent).toContain("Compatible rig");
    expect(container.textContent).toContain("walk · 1.25 s · 8 tracks");

    await click(buttonByText(container, "Copy to Custom"));
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(importClip).toHaveBeenCalledWith(
      file,
      "walk",
      expect.stringMatching(/^authored-/),
      "imp_walk",
    );
    expect(latestDocument.authoredClips).toEqual([importedClip]);
    expect(container.querySelector('[role="dialog"]')).toBeNull();
    expect(container.textContent).toContain("walk_imported");

    await click(buttonByText(container, "+ New animation"));
    await click(buttonByText(container, "Copy from another model…"));
    const secondFile = new File(["glb-two"], "donor-two.glb", {
      type: "model/gltf-binary",
    });
    const secondInput = required<HTMLInputElement>(
      container.querySelector('.animation-import-dialog input[type="file"]'),
    );
    await act(async () => {
      Object.defineProperty(secondInput, "files", {
        configurable: true,
        value: [secondFile],
      });
      secondInput.dispatchEvent(new Event("change", { bubbles: true }));
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(container.textContent).toContain("attack · 0.75 s · 9 tracks");
    await click(buttonByText(container, "Copy to Custom"));
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(latestDocument.authoredClips).toHaveLength(2);
    expect(latestDocument.authoredClips.map(({ source }) => (
      source.sourceRevision
    ))).toEqual([donorRevision, secondDonorRevision]);
    expect(latestDocument.authoredClips.map(({ source }) => (
      source.sourceClipFingerprint
    ))).toEqual(["c".repeat(64), "e".repeat(64)]);
  });

  it("creates from the current pose, edits translation and rotation at one playhead, saves to Custom, and keeps rename identity stable", async () => {
    let latestDocument = emptyDocument();
    let latestAuthoring = emptyAuthoring();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studioState, setStudioState] = useState(
        createAnimationStudioStateV1(emptyDocument()),
      );
      const [authoring, setAuthoring] = useState(emptyAuthoring());
      latestDocument = studioState.document;
      latestAuthoring = authoring;
      return (
        <AnimationStudioWorkspace
          document={studioState.document}
          authoring={authoring}
          sourceInventory={[]}
          rig={rig}
          viewport={(clip, playheadSeconds) => (
            <div
              data-testid="controlled-editor-viewport"
              data-clip-id={clip?.id ?? ""}
              data-playhead={playheadSeconds.toFixed(3)}
            />
          )}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={(next) => setStudioState((current) => (
            commitAnimationStudioDocumentV1(current, next)
          ))}
          onAuthoringChange={setAuthoring}
          onUndo={() => setStudioState(undoAnimationStudioEditV1)}
          onRedo={() => setStudioState(redoAnimationStudioEditV1)}
          canUndo={studioState.undoStack.length > 0}
          canRedo={studioState.redoStack.length > 0}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(buttonByText(container, "+ New animation"));
    await click(buttonByText(container, "From current pose"));

    expect(latestDocument.authoredClips).toHaveLength(1);
    expect(latestDocument.authoredClips[0]?.source.kind).toBe("BLANK_POSE");
    expect(container.textContent).toContain("custom_animation");
    const stableClipId = latestDocument.authoredClips[0]!.id;

    const playhead = required<HTMLInputElement>(
      container.querySelector('input[aria-label="Animation playhead"]'),
    );
    await setInputValue(playhead, "0.5");
    expect(container.querySelectorAll('input[aria-label="Animation playhead"]')).toHaveLength(1);
    expect(container.querySelectorAll(".animation-playhead")).toHaveLength(1);
    expect(
      container.querySelector('[data-testid="controlled-editor-viewport"]')
        ?.getAttribute("data-playhead"),
    ).toBe("0.500");

    await click(tabByText(container, "Translation", "Transform path"));
    await setInspectorAxis(container, "X", "0.25");
    await click(buttonByText(container, "+ Add keyframe"));

    await click(tabByText(container, "Rotation", "Transform path"));
    await setInspectorAxis(container, "Z", "90");
    await click(buttonByText(container, "+ Add keyframe"));

    expect(keyAt(latestDocument, "TRANSLATION", 0.5)?.value).toEqual([0.25, 0, 0]);
    expect(keyAt(latestDocument, "ROTATION", 0.5)?.value).toEqual([
      0,
      0,
      expect.closeTo(Math.SQRT1_2, 6),
      expect.closeTo(Math.SQRT1_2, 6),
    ]);

    await click(buttonByText(container, "Undo"));
    expect(keyAt(latestDocument, "ROTATION", 0.5)).toBeUndefined();
    await click(buttonByText(container, "Redo"));
    expect(keyAt(latestDocument, "ROTATION", 0.5)).toBeDefined();

    await click(buttonByText(container, "Save to Custom"));
    expect(latestDocument.authoredClips[0]?.status).toBe("VALID");
    expect(latestAuthoring.customAnimations).toHaveLength(1);
    expect(
      latestAuthoring.customAnimations[0]?.clipReference?.authoredClipId,
    ).toBe(stableClipId);
    expect(container.textContent).toContain("VALID");

    const outputName = labelInput(container, "Output name");
    await setInputValue(outputName, "renamed_custom_attack");
    expect(latestDocument.authoredClips[0]?.id).toBe(stableClipId);
    expect(latestDocument.authoredClips[0]?.name).toBe("renamed_custom_attack");
    expect(
      latestAuthoring.customAnimations[0]?.clipReference?.authoredClipId,
    ).toBe(stableClipId);

    await click(buttonByText(container, "+ New animation"));
    await click(buttonByText(container, "From procedural template"));
    const procedural = latestDocument.authoredClips.find(
      ({ source }) => source.kind === "PROCEDURAL_TEMPLATE",
    );
    expect(procedural).toMatchObject({
      kind: "MOTION",
      status: "DRAFT",
      source: { proceduralTemplate: "ROOT_TRANSLATION_PULSE" },
    });
    expect(procedural?.tracks.every(
      ({ interpolation }) => interpolation === "LINEAR",
    )).toBe(true);
    expect(procedural?.tracks.find(({ path }) => path === "TRANSLATION")
      ?.keyframes.map(({ timeSeconds }) => timeSeconds)).toEqual([0, 0.5, 1]);
    expect(container.textContent).toContain("Generated");
  });

  it("keeps a false-valid clip Invalid and unassignable when exact core validation blocks it", async () => {
    const clip = animationStudioClipFixtureV1({
      id: "foreign-bone-clip",
      name: "foreign_bone_clip",
      status: "DRAFT",
    });
    const initialDocument = animationStudioDocumentFixtureV1({
      status: "DRAFT",
      authoredClips: [clip],
    });
    let latestDocument = initialDocument;
    let latestAuthoring = emptyAuthoring();
    const validate = vi.fn().mockResolvedValue([{
      schemaVersion: 1,
      code: "M2A-ANIMATION-EDIT-BONE-MISSING",
      path: `authoredClips[${clip.id}].tracks[0].targetNodeId`,
      level: "BLOCKING",
      message: "The track targets a node outside the exact output rig.",
      action: "Select a bone from the exact output rig.",
    } satisfies AnimationStudioDiagnosticV1]);
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studio, setStudio] = useState(initialDocument);
      const [authoring, setAuthoring] = useState(emptyAuthoring());
      latestDocument = studio;
      latestAuthoring = authoring;
      return (
        <AnimationStudioWorkspace
          document={studio}
          authoring={authoring}
          sourceInventory={[]}
          rig={rig}
          viewport={<div />}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={setStudio}
          onAuthoringChange={setAuthoring}
          onValidateClip={validate}
          onUndo={vi.fn()}
          onRedo={vi.fn()}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(buttonByText(container, "Save to Custom"));

    expect(validate).toHaveBeenCalledOnce();
    expect(latestDocument).toMatchObject({
      status: "INVALID",
      authoredClips: [{ id: clip.id, status: "INVALID" }],
    });
    expect(latestAuthoring.customAnimations).toEqual([]);
    expect(container.textContent).toContain("Select a bone from the exact output rig.");
  });

  it("derives the document status from every clip instead of masking another Draft", async () => {
    const first = animationStudioClipFixtureV1({
      id: "save-one",
      name: "save_one",
      status: "DRAFT",
    });
    const second = animationStudioClipFixtureV1({
      id: "leave-draft",
      name: "leave_draft",
      status: "DRAFT",
    });
    const initialDocument = animationStudioDocumentFixtureV1({
      status: "DRAFT",
      authoredClips: [first, second],
    });
    let latestDocument = initialDocument;
    let latestAuthoring = emptyAuthoring();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studio, setStudio] = useState(initialDocument);
      const [authoring, setAuthoring] = useState(emptyAuthoring());
      latestDocument = studio;
      latestAuthoring = authoring;
      return (
        <AnimationStudioWorkspace
          document={studio}
          authoring={authoring}
          sourceInventory={[]}
          rig={rig}
          viewport={<div />}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={setStudio}
          onAuthoringChange={setAuthoring}
          onValidateClip={async () => []}
          onUndo={vi.fn()}
          onRedo={vi.fn()}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(buttonByText(container, "Save to Custom"));

    expect(latestDocument.status).toBe("DRAFT");
    expect(latestDocument.authoredClips.map(({ id, status }) => [id, status]))
      .toEqual([
        [first.id, "VALID"],
        [second.id, "DRAFT"],
      ]);
    expect(latestAuthoring.customAnimations).toHaveLength(1);
  });

  it("discards an exact validation result when the clip changes in flight", async () => {
    const clip = animationStudioClipFixtureV1({
      id: "async-stale",
      name: "async_stale",
      status: "DRAFT",
    });
    const initialDocument = animationStudioDocumentFixtureV1({
      status: "DRAFT",
      authoredClips: [clip],
    });
    let latestDocument = initialDocument;
    let resolveValidation!: (
      diagnostics: readonly AnimationStudioDiagnosticV1[],
    ) => void;
    const validation = new Promise<readonly AnimationStudioDiagnosticV1[]>(
      (resolve) => {
        resolveValidation = resolve;
      },
    );
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studio, setStudio] = useState(initialDocument);
      latestDocument = studio;
      return (
        <AnimationStudioWorkspace
          document={studio}
          authoring={emptyAuthoring()}
          sourceInventory={[]}
          rig={rig}
          viewport={<div />}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={setStudio}
          onAuthoringChange={vi.fn()}
          onValidateClip={() => validation}
          onUndo={vi.fn()}
          onRedo={vi.fn()}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await act(async () => buttonByText(container, "Save to Custom").click());
    expect(buttonByText(container, "Validating…").disabled).toBe(true);

    await setInputValue(labelInput(container, "Output name"), "changed_in_flight");
    await act(async () => resolveValidation([]));

    expect(latestDocument.authoredClips[0]).toMatchObject({
      name: "changed_in_flight",
      status: "DRAFT",
    });
    expect(container.textContent).toContain(
      "The clip changed while exact validation was running.",
    );
  });

  it("range-selects and moves keys, zooms and pans, then routes delete and undo/redo keyboard actions", async () => {
    const clip = animationStudioClipFixtureV1({
      status: "VALID",
      tracks: [{
        id: "track-torso-rotation",
        targetNodeId: 7,
        path: "ROTATION",
        interpolation: "LINEAR",
        keyframes: [{
          id: "key-start",
          timeSeconds: 0,
          value: [0, 0, 0, 1],
        }, {
          id: "key-delete",
          timeSeconds: 0.5,
          value: [0, 0, Math.SQRT1_2, Math.SQRT1_2],
        }],
      }],
    });
    let latestDocument = animationStudioDocumentFixtureV1({
      status: "VALID",
      authoredClips: [clip],
    });
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studioState, setStudioState] = useState(
        createAnimationStudioStateV1(latestDocument),
      );
      latestDocument = studioState.document;
      return (
        <AnimationStudioWorkspace
          document={studioState.document}
          authoring={emptyAuthoring()}
          sourceInventory={[]}
          rig={rig}
          viewport={<div data-testid="viewport" />}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={(next) => setStudioState((current) => (
            commitAnimationStudioDocumentV1(current, next)
          ))}
          onAuthoringChange={vi.fn()}
          onUndo={() => setStudioState(undoAnimationStudioEditV1)}
          onRedo={() => setStudioState(redoAnimationStudioEditV1)}
          canUndo={studioState.undoStack.length > 0}
          canRedo={studioState.redoStack.length > 0}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await setInputValue(
      required(container.querySelector('input[aria-label="Range start seconds"]')),
      "0.45",
    );
    await setInputValue(
      required(container.querySelector('input[aria-label="Range end seconds"]')),
      "0.55",
    );
    await click(buttonByText(container, "Select keys in range"));
    expect(
      container.querySelector('button[aria-label*="key at 0.500 seconds, selected"]'),
    ).not.toBeNull();

    const dopeSheet = required<HTMLElement>(
      container.querySelector(".animation-dope-sheet"),
    );
    await act(async () => {
      dopeSheet.dispatchEvent(new KeyboardEvent("keydown", {
        key: "ArrowRight",
        altKey: true,
        bubbles: true,
      }));
    });
    const movedTime = 0.5 + 1 / 30;
    expect(keyAt(latestDocument, "ROTATION", movedTime)?.id).toBe("key-delete");

    const timelineSurface = required<HTMLElement>(
      container.querySelector(".animation-timeline-surface"),
    );
    expect(timelineSurface.getAttribute("data-pixels-per-second")).toBe("100.00");
    await act(async () => {
      dopeSheet.dispatchEvent(new KeyboardEvent("keydown", {
        key: "+",
        bubbles: true,
      }));
    });
    expect(timelineSurface.getAttribute("data-pixels-per-second")).toBe("110.00");
    await act(async () => {
      dopeSheet.dispatchEvent(new KeyboardEvent("keydown", {
        key: "ArrowRight",
        shiftKey: true,
        bubbles: true,
      }));
    });
    expect(Number(timelineSurface.getAttribute("data-offset-seconds"))).toBeGreaterThan(0);
    expect(container.querySelectorAll(".animation-playhead")).toHaveLength(1);

    await act(async () => {
      dopeSheet.dispatchEvent(new KeyboardEvent("keydown", {
        key: "Delete",
        bubbles: true,
      }));
    });
    expect(keyAt(latestDocument, "ROTATION", movedTime)).toBeUndefined();

    await pressWorkspaceShortcut(container, { key: "z", ctrlKey: true });
    expect(keyAt(latestDocument, "ROTATION", movedTime)?.id).toBe("key-delete");
    await pressWorkspaceShortcut(container, {
      key: "z",
      ctrlKey: true,
      shiftKey: true,
    });
    expect(keyAt(latestDocument, "ROTATION", movedTime)).toBeUndefined();

    await pressWorkspaceShortcut(container, { key: "z", metaKey: true });
    expect(keyAt(latestDocument, "ROTATION", movedTime)?.id).toBe("key-delete");
    await pressWorkspaceShortcut(container, { key: "y", ctrlKey: true });
    expect(keyAt(latestDocument, "ROTATION", movedTime)).toBeUndefined();
  });

  it("shows the first concrete repair action next to an Invalid library item", async () => {
    const invalid = animationStudioClipFixtureV1({
      id: "clip-invalid-action",
      name: "invalid_action_clip",
      status: "INVALID",
    });
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    await act(async () => root.render(
      <AnimationStudioWorkspace
        document={animationStudioDocumentFixtureV1({
          status: "INVALID",
          authoredClips: [invalid],
        })}
        authoring={emptyAuthoring()}
        sourceInventory={[]}
        rig={rig}
        viewport={<div />}
        autosaveState={{ kind: "SAVED" }}
        diagnostics={[{
          schemaVersion: 1,
          code: "M2A-ANIMATION-EDIT-QUATERNION",
          path: `authoredClips.${invalid.id}.tracks.0`,
          level: "BLOCKING",
          message: "Quaternion is not canonical.",
          action: "Normalize the first rotation key and save again.",
        }]}
        onDocumentChange={vi.fn()}
        onAuthoringChange={vi.fn()}
        onUndo={vi.fn()}
        onRedo={vi.fn()}
        canUndo={false}
        canRedo={false}
      />,
    ));

    expect(
      container.querySelector(".animation-clip-repair-action")?.textContent,
    ).toContain("First fix: Normalize the first rotation key and save again.");
  });

  it("matches Rust motion detection against the first key with a strict greater-than 1e-6 threshold", async () => {
    const thresholdStatic = animationStudioClipFixtureV1({
      id: "threshold-static",
      name: "threshold_static",
      status: "DRAFT",
      kind: "MOTION",
      tracks: [{
        id: "threshold-static-track",
        targetNodeId: 7,
        path: "TRANSLATION",
        interpolation: "LINEAR",
        keyframes: [
          { id: "static-0", timeSeconds: 0, value: [0, 0, 0] },
          { id: "static-1", timeSeconds: 0.5, value: [0.0000005, 0, 0] },
          { id: "static-2", timeSeconds: 0.9, value: [0.000001, 0, 0] },
        ],
      }],
    });
    const firstKeyMotion = animationStudioClipFixtureV1({
      id: "first-key-motion",
      name: "first_key_motion",
      status: "DRAFT",
      kind: "STATIC_POSE",
      tracks: [{
        id: "first-key-motion-track",
        targetNodeId: 7,
        path: "TRANSLATION",
        interpolation: "LINEAR",
        keyframes: [
          { id: "motion-0", timeSeconds: 0, value: [0, 0, 0] },
          { id: "motion-1", timeSeconds: 0.5, value: [0.0000008, 0, 0] },
          { id: "motion-2", timeSeconds: 0.9, value: [0.0000016, 0, 0] },
        ],
      }],
    });
    const initialDocument = animationStudioDocumentFixtureV1({
      status: "DRAFT",
      authoredClips: [thresholdStatic, firstKeyMotion],
    });
    let latestDocument = initialDocument;
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studioState, setStudioState] = useState(
        createAnimationStudioStateV1(initialDocument),
      );
      const [authoring, setAuthoring] = useState(emptyAuthoring());
      latestDocument = studioState.document;
      return (
        <AnimationStudioWorkspace
          document={studioState.document}
          authoring={authoring}
          sourceInventory={[]}
          rig={rig}
          viewport={<div />}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={(next) => setStudioState((current) => (
            commitAnimationStudioDocumentV1(current, next)
          ))}
          onAuthoringChange={setAuthoring}
          onUndo={() => setStudioState(undoAnimationStudioEditV1)}
          onRedo={() => setStudioState(redoAnimationStudioEditV1)}
          canUndo={studioState.undoStack.length > 0}
          canRedo={studioState.redoStack.length > 0}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(buttonByText(container, "Save to Custom"));
    expect(
      latestDocument.authoredClips.find(({ id }) => id === thresholdStatic.id)?.kind,
    ).toBe("STATIC_POSE");

    await click(required(Array.from(container.querySelectorAll<HTMLButtonElement>(
      ".animation-clip-library [role=\"option\"]",
    )).find((button) => button.textContent?.includes("first_key_motion"))));
    await click(buttonByText(container, "Save to Custom"));
    expect(
      latestDocument.authoredClips.find(({ id }) => id === firstKeyMotion.id)?.kind,
    ).toBe("MOTION");
  });

  it("requires confirmation before removing an authored clip used by Base 42 and clears its relational references after approval", async () => {
    const clip = animationStudioClipFixtureV1({ status: "VALID" });
    const custom = createCustomDefinitionFromAuthoredClipV1(clip.id, {
      id: "custom-stable",
      name: "Custom attack",
    });
    const initialDocument = animationStudioDocumentFixtureV1({
      status: "VALID",
      authoredClips: [clip],
    });
    const initialAuthoring: CreatureAnimationAuthoringV2 = {
      ...emptyAuthoring(),
      customAnimations: [custom],
      assignments: [customAssignment("cwalk", custom.id)],
    };
    let latestDocument = initialDocument;
    let latestAuthoring = initialAuthoring;
    const confirm = vi.spyOn(globalThis, "confirm").mockReturnValue(false);
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studioState, setStudioState] = useState(
        createAnimationStudioStateV1(initialDocument),
      );
      const [authoring, setAuthoring] = useState(initialAuthoring);
      latestDocument = studioState.document;
      latestAuthoring = authoring;
      return (
        <AnimationStudioWorkspace
          document={studioState.document}
          authoring={authoring}
          sourceInventory={[]}
          rig={rig}
          viewport={<div />}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={(next) => setStudioState((current) => (
            commitAnimationStudioDocumentV1(current, next)
          ))}
          onAuthoringChange={setAuthoring}
          onUndo={() => setStudioState(undoAnimationStudioEditV1)}
          onRedo={() => setStudioState(redoAnimationStudioEditV1)}
          canUndo={studioState.undoStack.length > 0}
          canRedo={studioState.redoStack.length > 0}
          requestedClipId={clip.id}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(buttonByText(container, "Remove clip"));
    expect(confirm).toHaveBeenCalledWith(
      "This animation is used by Base 42 slots: cwalk. Remove it and clear those assignments?",
    );
    expect(latestDocument.authoredClips).toHaveLength(1);
    expect(latestAuthoring.customAnimations).toHaveLength(1);
    expect(latestAuthoring.assignments).toHaveLength(1);

    confirm.mockReturnValue(true);
    await click(buttonByText(container, "Remove clip"));
    expect(latestDocument.authoredClips).toHaveLength(0);
    expect(latestAuthoring.customAnimations).toHaveLength(0);
    expect(latestAuthoring.assignments).toHaveLength(0);
  });

});

describe("AnimationDopeSheet bounded rendering", () => {
  it("keeps one playhead and at most 2,000 key/event marker nodes", async () => {
    const largeClip = animationStudioClipFixtureV1({
      lengthSeconds: 2,
      events: [],
      tracks: [{
        id: "large-track",
        targetNodeId: 7,
        path: "ROTATION",
        interpolation: "LINEAR",
        keyframes: Array.from({ length: 2_005 }, (_, index) => ({
          id: `large-key-${index}`,
          timeSeconds: index / 1_002,
          value: [0, 0, 0, 1],
        })),
      }],
    });
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    await act(async () => root.render(
      <AnimationDopeSheet
        clip={largeClip}
        rig={rig}
        playheadSeconds={0}
        playing={false}
        selectedKeyIds={new Set()}
        onPlayingChange={vi.fn()}
        onSeek={vi.fn()}
        onSelectKey={vi.fn()}
        onSelectEvent={vi.fn()}
        onAddEvent={vi.fn()}
        onDeleteKeys={vi.fn()}
        onMoveSelectedKeys={vi.fn()}
        onSelectRange={vi.fn()}
        onOpenTrim={vi.fn()}
        onOpenRetime={vi.fn()}
      />,
    ));

    expect(container.querySelectorAll(".animation-keyframe-marker")).toHaveLength(2_000);
    expect(container.querySelectorAll(".animation-playhead")).toHaveLength(1);
    expect(container.textContent).toContain(
      "Timeline virtualized: showing 2000 markers nearest the playhead out of 2005.",
    );
  });
});

describe("CustomAnimationMappingPanel integration", () => {
  it("creates and assigns a Valid phased Custom from three stable authored clip IDs without asking for an ID", async () => {
    const start = animationStudioClipFixtureV1({
      id: "phase-start-stable",
      name: "phase_start_before",
      status: "VALID",
    });
    const loop = animationStudioClipFixtureV1({
      id: "phase-loop-stable",
      name: "phase_loop_before",
      status: "VALID",
    });
    const end = animationStudioClipFixtureV1({
      id: "phase-end-stable",
      name: "phase_end_before",
      status: "VALID",
    });
    const draft = animationStudioClipFixtureV1({
      id: "phase-draft-disabled",
      name: "phase_draft",
      status: "DRAFT",
    });
    const invalid = animationStudioClipFixtureV1({
      id: "phase-invalid-disabled",
      name: "phase_invalid",
      status: "INVALID",
    });
    const initialStudio = animationStudioDocumentFixtureV1({
      status: "INVALID",
      authoredClips: [start, loop, end, draft, invalid],
    });
    let latestAuthoring = emptyAuthoring();
    const onOpenClip = vi.fn();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness({ studio }: { studio: AnimationStudioDocumentV1 }) {
      const [authoring, setAuthoring] = useState(emptyAuthoring());
      latestAuthoring = authoring;
      return (
        <CustomAnimationMappingPanel
          slot="cwalk"
          authoring={authoring}
          studio={studio}
          onAuthoringChange={setAuthoring}
          onCreate={vi.fn()}
          onOpenClip={onOpenClip}
        />
      );
    }

    await act(async () => root.render(<Harness studio={initialStudio} />));
    const phaseSelects = (["Start", "Loop", "End"] as const).map((phase) => (
      required<HTMLSelectElement>(
        container.querySelector(`select[aria-label="${phase} clip"]`),
      )
    ));
    expect(phaseSelects.map(({ value }) => value)).toEqual([
      start.id,
      loop.id,
      end.id,
    ]);
    for (const select of phaseSelects) {
      expect(required<HTMLOptionElement>(
        select.querySelector(`option[value="${draft.id}"]`),
      ).disabled).toBe(true);
      expect(required<HTMLOptionElement>(
        select.querySelector(`option[value="${invalid.id}"]`),
      ).disabled).toBe(true);
    }
    expect(container.querySelector('input[aria-label*="ID" i]')).toBeNull();
    expect(
      required<HTMLInputElement>(
        container.querySelector('input[aria-label="Phased Custom name"]'),
      ).value,
    ).toBe("custom_loop");

    await click(buttonByText(container, "Create phased Custom"));
    expect(latestAuthoring.customAnimations).toHaveLength(1);
    const created = latestAuthoring.customAnimations[0]!;
    const stableCustomId = created.id;
    expect(stableCustomId).toMatch(/^custom-/);
    expect(created.playback).toBe("LOOPING_PHASED");
    expect(created.clipReference).toBeNull();
    expect(created.phases.map(({ phase }) => phase)).toEqual([
      "START",
      "LOOP",
      "END",
    ]);
    expect(new Set(created.phases.map(({ phase }) => phase)).size).toBe(3);
    expect(created.phases.map(({ clipReference }) => (
      clipReference.authoredClipId
    ))).toEqual([start.id, loop.id, end.id]);

    const assign = buttonByText(container, "Assign custom animation");
    expect(assign.disabled).toBe(false);
    await click(assign);
    expect(latestAuthoring.assignments).toEqual([
      expect.objectContaining({
        targetSlot: "cwalk",
        sourceKind: "CUSTOM",
        customAnimationId: stableCustomId,
      }),
    ]);

    const renamedStudio: AnimationStudioDocumentV1 = {
      ...initialStudio,
      authoredClips: initialStudio.authoredClips.map((clip) => ({
        ...clip,
        name: `${clip.name}_renamed`,
      })),
    };
    await act(async () => root.render(<Harness studio={renamedStudio} />));
    expect(latestAuthoring.customAnimations[0]?.id).toBe(stableCustomId);
    expect(latestAuthoring.customAnimations[0]?.phases.map(
      ({ clipReference }) => clipReference.authoredClipId,
    )).toEqual([start.id, loop.id, end.id]);
    expect(latestAuthoring.assignments[0]?.customAnimationId).toBe(stableCustomId);

    await click(buttonByText(container, "Open selected in editor"));
    expect(onOpenClip).toHaveBeenCalledWith(start.id);
  });

  it("keeps Draft and Invalid entries unassignable, previews before explicit assignment, and opens the exact authored clip after rename", async () => {
    const draft = animationStudioClipFixtureV1({
      id: "clip-draft",
      name: "draft_output",
      status: "DRAFT",
    });
    const invalid = animationStudioClipFixtureV1({
      id: "clip-invalid",
      name: "invalid_output",
      status: "INVALID",
    });
    const valid = animationStudioClipFixtureV1({
      id: "clip-valid-stable",
      name: "valid_output",
      status: "VALID",
    });
    const initialStudio = animationStudioDocumentFixtureV1({
      status: "INVALID",
      authoredClips: [draft, invalid, valid],
    });
    const initialAuthoring: CreatureAnimationAuthoringV2 = {
      ...emptyAuthoring(),
      customAnimations: [
        createCustomDefinitionFromAuthoredClipV1(draft.id, {
          id: "custom-draft",
          name: "Draft custom",
        }),
        createCustomDefinitionFromAuthoredClipV1(invalid.id, {
          id: "custom-invalid",
          name: "Invalid custom",
        }),
        createCustomDefinitionFromAuthoredClipV1(valid.id, {
          id: "custom-valid-stable",
          name: "Valid custom",
        }),
      ],
    };
    let latestAuthoring = initialAuthoring;
    const onOpenClip = vi.fn();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness({ studio }: { studio: AnimationStudioDocumentV1 }) {
      const [authoring, setAuthoring] = useState(initialAuthoring);
      latestAuthoring = authoring;
      return (
        <CustomAnimationMappingPanel
          slot="cwalk"
          authoring={authoring}
          studio={studio}
          onAuthoringChange={setAuthoring}
          onCreate={vi.fn()}
          onOpenClip={onOpenClip}
        />
      );
    }

    await act(async () => root.render(<Harness studio={initialStudio} />));
    const draftRow = pickerRow(container, "Draft custom");
    const invalidRow = pickerRow(container, "Invalid custom");
    const validRow = pickerRow(container, "Valid custom");
    expect(draftRow.getAttribute("aria-disabled")).toBe("true");
    expect(invalidRow.getAttribute("aria-disabled")).toBe("true");
    expect(validRow.getAttribute("aria-disabled")).toBe("false");
    expect([draftRow.tabIndex, invalidRow.tabIndex, validRow.tabIndex])
      .toEqual([-1, -1, 0]);
    validRow.focus();
    await act(async () => {
      validRow.dispatchEvent(new KeyboardEvent("keydown", {
        key: "ArrowDown",
        bubbles: true,
      }));
    });
    expect(document.activeElement).toBe(draftRow);
    expect(latestAuthoring.assignments).toHaveLength(0);

    await click(draftRow);
    await click(invalidRow);
    expect(latestAuthoring.assignments).toHaveLength(0);

    await click(validRow);
    expect(latestAuthoring.assignments).toHaveLength(0);
    expect(validRow.getAttribute("aria-label")).toContain("provider USER_CUSTOM");
    expect(validRow.getAttribute("aria-label")).toContain(
      "asset clip-valid-stable",
    );
    expect(container.textContent).toContain(
      "Provider: USER_CUSTOM · Asset: clip-valid-stable · Ownership: USER_OWNED",
    );

    await click(buttonByText(container, "Preview selected Custom"));
    expect(onOpenClip).toHaveBeenCalledWith("clip-valid-stable");
    expect(latestAuthoring.assignments).toHaveLength(0);

    await click(buttonByText(container, "Assign custom animation"));
    expect(latestAuthoring.assignments).toEqual([
      expect.objectContaining({
        targetSlot: "cwalk",
        sourceKind: "CUSTOM",
        customAnimationId: "custom-valid-stable",
      }),
    ]);

    const renamedStudio: AnimationStudioDocumentV1 = {
      ...initialStudio,
      authoredClips: initialStudio.authoredClips.map((clip) => (
        clip.id === valid.id ? { ...clip, name: "renamed_valid_output" } : clip
      )),
    };
    await act(async () => root.render(<Harness studio={renamedStudio} />));
    expect(
      latestAuthoring.assignments[0]?.customAnimationId,
    ).toBe("custom-valid-stable");

    await click(buttonByText(container, "Open selected in editor"));
    expect(onOpenClip).toHaveBeenLastCalledWith("clip-valid-stable");
  });

  it("assigns a migrated source-backed Custom while keeping Open in editor disabled", async () => {
    const sourceBacked: CreatureAnimationAuthoringV2 = {
      ...emptyAuthoring(),
      customAnimations: [{
        id: "source-backed",
        name: "Source idle",
        playback: "ONE_SHOT",
        clipReference: {
          sourceKind: "SOURCE_CLIP",
          sourceClipName: "idle",
          authoredClipId: null,
        },
        phases: [],
        provenance: {
          provider: "SOURCE_GLB",
          assetId: sourceRevision,
          ownership: "USER_OWNED",
        },
      }],
    };
    let latestAuthoring = sourceBacked;
    const onOpenClip = vi.fn();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [authoring, setAuthoring] = useState(sourceBacked);
      latestAuthoring = authoring;
      return (
        <CustomAnimationMappingPanel
          slot="cwalk"
          authoring={authoring}
          studio={emptyDocument()}
          sourceInventory={[{
            clipId: "source-idle",
            name: "IDLE",
            durationSeconds: 1.25,
            trackCount: 8,
          }]}
          onAuthoringChange={setAuthoring}
          onCreate={vi.fn()}
          onOpenClip={onOpenClip}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    const row = pickerRow(container, "Source idle");
    expect(row.getAttribute("aria-disabled")).toBe("false");
    expect(row.getAttribute("aria-label")).toContain("read-only source clips");
    const open = buttonByText(container, "Open selected in editor");
    const preview = buttonByText(container, "Preview selected Custom");
    expect(open.disabled).toBe(true);
    expect(preview.disabled).toBe(true);
    expect(open.title).toContain("Edit copy");
    expect(row.getAttribute("aria-label")).toContain("provider SOURCE_GLB");
    expect(row.getAttribute("aria-label")).toContain(`asset ${sourceRevision}`);

    await click(row);
    expect(latestAuthoring.assignments).toHaveLength(0);
    await click(buttonByText(container, "Assign custom animation"));
    expect(latestAuthoring.assignments).toEqual([
      expect.objectContaining({
        targetSlot: "cwalk",
        sourceKind: "CUSTOM",
        customAnimationId: "source-backed",
      }),
    ]);
    expect(onOpenClip).not.toHaveBeenCalled();
  });
});

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
    profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V2,
    modelType: "S",
    sourceRevision,
    authoringRevision: 1,
    assignments: [],
    fallbacks: [],
    customAnimations: [],
  };
}

function customAssignment(
  targetSlot: DirectCreatureBaseSlotV1,
  customAnimationId: string,
): CreatureAnimationAuthoringV2["assignments"][number] {
  return {
    targetSlot,
    sourceKind: "CUSTOM",
    sourceClipName: null,
    customAnimationId,
    provenance: {
      provider: "USER_CUSTOM",
      assetId: customAnimationId,
      ownership: "USER_OWNED",
    },
  };
}

function keyAt(
  document: AnimationStudioDocumentV1,
  path: "TRANSLATION" | "ROTATION",
  timeSeconds: number,
) {
  return document.authoredClips[0]?.tracks
    .find((track) => track.path === path)
    ?.keyframes.find((key) => Math.abs(key.timeSeconds - timeSeconds) < 1e-7);
}

function required<T>(value: T | null | undefined): T {
  if (value === null || value === undefined) {
    throw new Error("Required test element is missing.");
  }
  return value;
}

function buttonByText(container: HTMLElement, text: string): HTMLButtonElement {
  return required(Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
    .find((button) => button.textContent?.trim() === text));
}

function tabByText(
  container: HTMLElement,
  text: string,
  tablistLabel: string,
): HTMLButtonElement {
  const tablist = required(
    Array.from(container.querySelectorAll<HTMLElement>('[role="tablist"]'))
      .find((element) => element.getAttribute("aria-label") === tablistLabel),
  );
  return required(Array.from(tablist.querySelectorAll<HTMLButtonElement>('[role="tab"]'))
    .find((button) => button.textContent?.trim() === text));
}

function labelInput(container: HTMLElement, labelText: string): HTMLInputElement {
  const label = required(Array.from(container.querySelectorAll("label"))
    .find((candidate) => candidate.textContent?.trim().startsWith(labelText)));
  return required(label.querySelector("input"));
}

async function setInspectorAxis(
  container: HTMLElement,
  axis: "X" | "Y" | "Z",
  value: string,
) {
  const fieldset = required(container.querySelector(".bone-transform-inspector fieldset"));
  const label = required(Array.from(fieldset.querySelectorAll("label"))
    .find((candidate) => candidate.textContent?.trim() === axis));
  await setInputValue(required(label.querySelector("input")), value);
}

async function setInputValue(input: HTMLInputElement, value: string) {
  await act(async () => {
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set
      ?.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
}

async function click(button: HTMLButtonElement) {
  await act(async () => button.click());
}

async function pressWorkspaceShortcut(
  container: HTMLElement,
  init: KeyboardEventInit,
) {
  await act(async () => {
    required<HTMLElement>(container.querySelector(".animation-studio-workspace"))
      .dispatchEvent(new KeyboardEvent("keydown", {
        bubbles: true,
        ...init,
      }));
  });
}

function pickerRow(container: HTMLElement, name: string): HTMLButtonElement {
  return required(Array.from(container.querySelectorAll<HTMLButtonElement>(
    '.custom-animation-picker [role="option"]',
  )).find((button) => button.querySelector("strong")?.textContent === name));
}
