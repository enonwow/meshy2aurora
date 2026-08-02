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
import {
  AnimationDopeSheet,
  projectTimelineMarkersV1,
} from "./AnimationDopeSheet";
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

function exactTransferCompatibility(donorSourceRevision: string) {
  return {
    schemaVersion: 1 as const,
    status: "EXACT_COPY" as const,
    donorSourceRevision,
    targetSourceRevision: sourceRevision,
    donorRigSignatureSha256: "1".repeat(64),
    targetRigSignatureSha256: "1".repeat(64),
    compatibilityFingerprintSha256: "2".repeat(64),
    allowedModes: ["EXACT_RIG_COPY_V1" as const],
    mapping: {
      schemaVersion: 1 as const,
      rootName: "root",
      entries: [],
    },
    diagnostics: [],
  };
}

function retargetTransferCompatibility(donorSourceRevision: string) {
  return {
    ...exactTransferCompatibility(donorSourceRevision),
    status: "RETARGETABLE_SAME_HIERARCHY" as const,
    donorRigSignatureSha256: "3".repeat(64),
    targetRigSignatureSha256: "4".repeat(64),
    compatibilityFingerprintSha256: "5".repeat(64),
    allowedModes: ["SAME_HIERARCHY_RETARGET_V1" as const],
    diagnostics: [{
      code: "M2A-ANIMATION-RETARGET-REST-TRANSLATION",
      path: "rig.nodes[root].translation",
      message: "rest translation differs for root",
      action: "Preview the explicit retarget mode.",
    }],
  };
}

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
    const secondFile = new File(["glb-two"], "donor-two.glb", {
      type: "model/gltf-binary",
    });
    const inspect = vi.fn().mockImplementation(async (file: File) => (
      file.name === "donor-two.glb"
        ? {
            sourceRevision: secondDonorRevision,
            rig,
            clips: [{ name: "attack", durationSeconds: 0.75, trackCount: 9 }],
            transferCompatibility: exactTransferCompatibility(secondDonorRevision),
          }
        : {
            sourceRevision: donorRevision,
            rig,
            clips: [{ name: "walk", durationSeconds: 1.25, trackCount: 8 }],
            transferCompatibility: exactTransferCompatibility(donorRevision),
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
          animationModelDonors={[{
            id: "meshy-action-198",
            file: secondFile,
            label: "Meshy action 198",
            detail: "Verified by Meshy Bridge",
          }]}
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
    expect(container.textContent).toContain("Exact rig copy");
    expect(container.textContent).toContain("walk · 1.25 s · 8 tracks");

    await click(buttonByText(container, "Preview exact copy"));
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(container.textContent).toContain("Preview ready");
    await click(buttonByText(container, "Copy exact to Custom"));

    expect(importClip).toHaveBeenCalledWith(
      file,
      "walk",
      donorRevision,
      "EXACT_RIG_COPY_V1",
      undefined,
      expect.stringMatching(/^authored-/),
      "imp_walk",
    );
    expect(latestDocument.authoredClips).toEqual([importedClip]);
    expect(container.querySelector('[role="dialog"]')).toBeNull();
    expect(container.textContent).toContain("walk_imported");

    await click(buttonByText(container, "+ New animation"));
    await click(buttonByText(container, "Copy from another model…"));
    expect(container.textContent).toContain("Available from Meshy Bridge");
    expect(container.textContent).toContain("Meshy action 198");
    await act(async () => {
      required<HTMLButtonElement>(
        container.querySelector(".animation-import-dialog__donors button"),
      ).click();
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(container.textContent).toContain("attack · 0.75 s · 9 tracks");
    await click(buttonByText(container, "Preview exact copy"));
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });
    await click(buttonByText(container, "Copy exact to Custom"));

    expect(latestDocument.authoredClips).toHaveLength(2);
    expect(latestDocument.authoredClips.map(({ source }) => (
      source.sourceRevision
    ))).toEqual([donorRevision, secondDonorRevision]);
    expect(latestDocument.authoredClips.map(({ source }) => (
      source.sourceClipFingerprint
    ))).toEqual(["c".repeat(64), "e".repeat(64)]);
  });

  it("previews a same-hierarchy retarget before committing a V3 Custom clip", async () => {
    let latestDocument = emptyDocument();
    const donorRevision = "b".repeat(64);
    const retargeted = animationStudioClipFixtureV1({
      id: "authored-retargeted-attack",
      name: "imp_attack",
      source: {
        kind: "RETARGETED_MODEL_COPY",
        sourceRevision: donorRevision,
        sourceClipName: "attack",
        sourceClipFingerprint: "6".repeat(64),
        proceduralTemplate: null,
        retarget: {
          donorSourceRevision: donorRevision,
          targetSourceRevision: sourceRevision,
          donorClipName: "attack",
          donorClipFingerprint: "6".repeat(64),
          donorRigSignatureSha256: "3".repeat(64),
          targetRigSignatureSha256: "4".repeat(64),
          compatibilityFingerprintSha256: "5".repeat(64),
          mode: "SAME_HIERARCHY_RETARGET_V1",
          rootMotionScale: 1.25,
          outputMotionFingerprintSha256: "7".repeat(64),
          algorithmVersion: "M2A_SAME_HIERARCHY_REST_DELTA_V1",
          algorithmLimits: "TR_ONLY",
        },
      },
    });
    const inspect = vi.fn().mockResolvedValue({
      sourceRevision: donorRevision,
      rig,
      clips: [{ name: "attack", durationSeconds: 0.8, trackCount: 4 }],
      transferCompatibility: retargetTransferCompatibility(donorRevision),
    });
    const prepare = vi.fn().mockResolvedValue(retargeted);
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
          onImportAnimationModelClip={prepare}
          renderAnimationTransferPreview={() => <div>Target motion preview</div>}
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
    const file = new File(["donor"], "fogbound.glb", { type: "model/gltf-binary" });
    const input = required<HTMLInputElement>(
      container.querySelector('.animation-import-dialog input[type="file"]'),
    );
    await act(async () => {
      Object.defineProperty(input, "files", { configurable: true, value: [file] });
      input.dispatchEvent(new Event("change", { bubbles: true }));
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(container.textContent).toContain("Retargetable rig");
    expect(container.textContent).toContain("1 rig difference");
    await click(buttonByText(container, "Preview retarget"));
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(latestDocument.authoredClips).toEqual([]);
    expect(container.textContent).toContain("Preview ready");
    expect(container.textContent).toContain("Target motion preview");
    expect(container.textContent).toContain("1.2500");
    await click(buttonByText(container, "Copy retargeted to Custom"));
    expect(latestDocument.schemaVersion).toBe(3);
    expect(latestDocument.authoredClips).toEqual([retargeted]);
    expect(prepare).toHaveBeenCalledWith(
      file,
      "attack",
      donorRevision,
      "SAME_HIERARCHY_RETARGET_V1",
      undefined,
      expect.stringMatching(/^authored-/),
      "imp_attack",
    );
  });

  it("offers a source-bound humanoid semantic V2 transfer for differently named rigs", async () => {
    let latestDocument = emptyDocument();
    const donorRevision = "b".repeat(64);
    const semanticMap = {
      schemaVersion: 2 as const,
      aliasDictionaryVersion: "M2A_HUMANOID_ALIASES_2026_07_V1",
      status: "COMPATIBLE" as const,
      donorSourceRevision: donorRevision,
      targetSourceRevision: sourceRevision,
      manualMappingConfirmed: false,
      entries: [{
        semantic: "RIGHT_HAND",
        required: true,
        donorNodeId: 7,
        donorNodeName: "mixamorig_RightHand",
        targetNodeId: 7,
        targetNodeName: "hand_r",
        mappingSource: "VERSIONED_ALIAS" as const,
      }],
      diagnostics: [],
      fingerprintSha256: "8".repeat(64),
    };
    const semanticClip = animationStudioClipFixtureV1({
      id: "authored-semantic-attack",
      name: "imp_attack",
      source: {
        kind: "RETARGETED_MODEL_COPY",
        sourceRevision: donorRevision,
        sourceClipName: "attack",
        sourceClipFingerprint: "6".repeat(64),
        proceduralTemplate: null,
        retarget: {
          donorSourceRevision: donorRevision,
          targetSourceRevision: sourceRevision,
          donorClipName: "attack",
          donorClipFingerprint: "6".repeat(64),
          donorRigSignatureSha256: "3".repeat(64),
          targetRigSignatureSha256: "4".repeat(64),
          compatibilityFingerprintSha256: semanticMap.fingerprintSha256,
          mode: "HUMANOID_SEMANTIC_RETARGET_V2",
          rootMotionScale: 1,
          outputMotionFingerprintSha256: "7".repeat(64),
          algorithmVersion: "M2A_HUMANOID_SEMANTIC_CHAIN_V2",
          algorithmLimits: "VERSIONED_ALIASES|TR_ONLY|LINEAR",
        },
      },
    });
    const inspect = vi.fn().mockResolvedValue({
      sourceRevision: donorRevision,
      rig,
      clips: [{ name: "attack", durationSeconds: 0.8, trackCount: 4 }],
      transferCompatibility: {
        ...retargetTransferCompatibility(donorRevision),
        status: "INCOMPATIBLE" as const,
        allowedModes: [],
      },
      semanticCompatibility: semanticMap,
    });
    const prepare = vi.fn().mockResolvedValue(semanticClip);
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
          onImportAnimationModelClip={prepare}
          onUndo={vi.fn()}
          onRedo={vi.fn()}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(buttonByText(container, "+ New animation"));
    await click(required(Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
      .find((button) => button.textContent?.includes("Copy from another model"))));
    const file = new File(["donor"], "semantic-donor.glb", {
      type: "model/gltf-binary",
    });
    const input = required<HTMLInputElement>(
      container.querySelector('.animation-import-dialog input[type="file"]'),
    );
    await act(async () => {
      Object.defineProperty(input, "files", { configurable: true, value: [file] });
      input.dispatchEvent(new Event("change", { bubbles: true }));
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(container.textContent).toContain("Semantic humanoid retarget");
    expect(container.textContent).toContain("Humanoid semantic retarget V2");
    await click(buttonByText(container, "Preview retarget"));
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });
    await click(buttonByText(container, "Copy retargeted to Custom"));

    expect(prepare).toHaveBeenCalledWith(
      file,
      "attack",
      donorRevision,
      "HUMANOID_SEMANTIC_RETARGET_V2",
      semanticMap,
      expect.stringMatching(/^authored-/),
      "imp_attack",
    );
    expect(latestDocument.schemaVersion).toBe(3);
    expect(latestDocument.authoredClips).toEqual([semanticClip]);
  });

  it("fails closed when the current model or inspected donor lineage changes during an async import", async () => {
    let latestDocument = emptyDocument();
    let updateDocument:
      | ((document: AnimationStudioDocumentV1) => void)
      | null = null;
    let resolveImport!: (clip: ReturnType<typeof animationStudioClipFixtureV1>) => void;
    const donorRevision = "b".repeat(64);
    const delayedImport = new Promise<
      ReturnType<typeof animationStudioClipFixtureV1>
    >((resolve) => {
      resolveImport = resolve;
    });
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studio, setStudio] = useState(emptyDocument());
      latestDocument = studio;
      updateDocument = setStudio;
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
          onInspectAnimationModel={async () => ({
            sourceRevision: donorRevision,
            rig,
            clips: [{
              name: "attack",
              durationSeconds: 0.8,
              trackCount: 4,
            }],
            transferCompatibility: exactTransferCompatibility(donorRevision),
          })}
          onImportAnimationModelClip={() => delayedImport}
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
    const file = new File(["donor"], "donor.glb", {
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
    });
    await click(buttonByText(container, "Preview exact copy"));
    await act(async () => {
      updateDocument?.({
        ...latestDocument,
        sourceRevision: "f".repeat(64),
      });
    });
    expect(latestDocument.sourceRevision).toBe("f".repeat(64));
    await act(async () => {
      resolveImport(animationStudioClipFixtureV1({
        id: "imported-attack",
        name: "imp_attack",
        source: {
          kind: "IMPORTED_MODEL_COPY",
          sourceRevision: donorRevision,
          sourceClipName: "attack",
          sourceClipFingerprint: "c".repeat(64),
          proceduralTemplate: null,
        },
      }));
      await delayedImport;
      await Promise.resolve();
    });

    expect(latestDocument.authoredClips).toEqual([]);
    expect(container.querySelector('[role="alert"]')?.textContent)
      .toContain("document changed while the donor preview was being prepared");
  });

  it("merges a delayed Edit copy into the latest document instead of overwriting newer work", async () => {
    let latestDocument = emptyDocument();
    let updateDocument: ((document: AnimationStudioDocumentV1) => void) | null = null;
    let resolveCopy!: (clip: ReturnType<typeof animationStudioClipFixtureV1>) => void;
    const delayedCopy = new Promise<ReturnType<typeof animationStudioClipFixtureV1>>(
      (resolve) => {
        resolveCopy = resolve;
      },
    );
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studio, setStudio] = useState(emptyDocument());
      latestDocument = studio;
      updateDocument = setStudio;
      return (
        <AnimationStudioWorkspace
          document={studio}
          authoring={emptyAuthoring()}
          sourceInventory={[{
            clipId: "source-attack",
            name: "attack",
            durationSeconds: 0.8,
            trackCount: 4,
          }]}
          rig={rig}
          viewport={<div />}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={setStudio}
          onAuthoringChange={vi.fn()}
          onEditSourceClip={() => delayedCopy}
          onUndo={vi.fn()}
          onRedo={vi.fn()}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(buttonByText(container, "Edit copy"));
    const newerClip = animationStudioClipFixtureV1({
      id: "newer-local-clip",
      name: "newer_local_clip",
    });
    await act(async () => {
      updateDocument?.({
        ...latestDocument,
        authoringRevision: latestDocument.authoringRevision + 1,
        authoredClips: [newerClip],
      });
    });
    const resolved = animationStudioClipFixtureV1({
      id: "delayed-source-copy",
      name: "attack_edited",
      source: {
        kind: "SOURCE_CLIP_COPY",
        sourceRevision,
        sourceClipName: "attack",
        sourceClipFingerprint: "source-attack",
        proceduralTemplate: null,
      },
    });
    await act(async () => {
      resolveCopy(resolved);
      await delayedCopy;
      await Promise.resolve();
    });

    expect(latestDocument.authoredClips.map(({ id }) => id).sort()).toEqual([
      "delayed-source-copy",
      "newer-local-clip",
    ]);
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

  });

  it("delegates smooth playback to the viewport runtime and consumes throttled snapshots", async () => {
    const clip = animationStudioClipFixtureV1({
      id: "smooth-playback",
      name: "smooth_playback",
      lengthSeconds: 1,
      status: "VALID",
    });
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    let playbackControl: {
      playing: boolean;
      onUpdate: (snapshot: { timeSeconds: number; playing: boolean }) => void;
    } | undefined;

    function Harness() {
      const [studio, setStudio] = useState(animationStudioDocumentFixtureV1({
        status: "VALID",
        authoredClips: [clip],
      }));
      return (
        <AnimationStudioWorkspace
          document={studio}
          authoring={emptyAuthoring()}
          sourceInventory={[]}
          rig={rig}
          viewport={(_selectedClip, playheadSeconds, playback) => {
            playbackControl = playback;
            return (
              <div
                data-testid="smooth-playback-viewport"
                data-playhead={playheadSeconds.toFixed(3)}
                data-playing={playback.playing}
              />
            );
          }}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={setStudio}
          onAuthoringChange={vi.fn()}
          onUndo={vi.fn()}
          onRedo={vi.fn()}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(required<HTMLButtonElement>(
      container.querySelector('button[aria-label="Play"]'),
    ));
    expect(playbackControl?.playing).toBe(true);
    expect(
      container.querySelector('[data-testid="smooth-playback-viewport"]')
        ?.getAttribute("data-playhead"),
    ).toBe("0.000");

    await act(async () => playbackControl?.onUpdate({
      timeSeconds: 0.42,
      playing: true,
    }));
    expect(
      container.querySelector('[data-testid="smooth-playback-viewport"]')
        ?.getAttribute("data-playhead"),
    ).toBe("0.420");

    await act(async () => playbackControl?.onUpdate({
      timeSeconds: 1,
      playing: false,
    }));
    expect(container.querySelector('button[aria-label="Play"]')).not.toBeNull();
    expect(
      container.querySelector('[data-testid="smooth-playback-viewport"]')
        ?.getAttribute("data-playing"),
    ).toBe("false");
  });

  it("prunes orphaned authored Custom references when saving a recovered clip", async () => {
    const clip = animationStudioClipFixtureV1({
      id: "authored-current",
      name: "current_attack",
      status: "DRAFT",
    });
    const initialDocument = animationStudioDocumentFixtureV1({
      status: "DRAFT",
      authoredClips: [clip],
    });
    const staleCustom = createCustomDefinitionFromAuthoredClipV1("authored-missing", {
      id: "custom-stale",
      name: "stale_attack",
    });
    let latestDocument = initialDocument;
    let latestAuthoring: CreatureAnimationAuthoringV2 = {
      ...emptyAuthoring(),
      assignments: [customAssignment("ca1slashl", staleCustom.id)],
      customAnimations: [staleCustom],
    };
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studioState, setStudioState] = useState(
        createAnimationStudioStateV1(initialDocument),
      );
      const [authoring, setAuthoring] = useState(latestAuthoring);
      latestDocument = studioState.document;
      latestAuthoring = authoring;
      return (
        <AnimationStudioWorkspace
          document={studioState.document}
          authoring={authoring}
          sourceInventory={[]}
          rig={rig}
          viewport={() => <div />}
          autosaveState={{ kind: "SAVED" }}
          diagnostics={[]}
          onDocumentChange={(next) => setStudioState((current) => (
            commitAnimationStudioDocumentV1(current, next)
          ))}
          onAuthoringChange={setAuthoring}
          onUndo={vi.fn()}
          onRedo={vi.fn()}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(buttonByText(container, "Save to Custom"));

    expect(latestDocument.authoredClips[0]?.status).toBe("VALID");
    expect(latestAuthoring.assignments).toEqual([]);
    expect(latestAuthoring.customAnimations).toHaveLength(1);
    expect(latestAuthoring.customAnimations[0]?.clipReference?.authoredClipId)
      .toBe(clip.id);
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

  it("locks editing while exact validation is in flight", async () => {
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

    expect(container.querySelector(".animation-studio-workspace__main"))
      .toHaveProperty("disabled", true);
    expect(container.textContent).toContain(
      "Editing is temporarily locked",
    );
    expect(labelInput(container, "Output name").matches(":disabled")).toBe(true);
    await setInputValue(labelInput(container, "Output name"), "changed_in_flight");
    await act(async () => resolveValidation([]));

    expect(latestDocument.authoredClips[0]).toMatchObject({
      name: "async_stale",
      status: "VALID",
    });
    expect(container.textContent).not.toContain(
      "Editing is temporarily locked",
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
    const boneSelect = required<HTMLSelectElement>(
      container.querySelector('select[aria-label="Output rig bone"]'),
    );
    await act(async () => {
      boneSelect.value = "7";
      boneSelect.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await click(required<HTMLButtonElement>(
      container.querySelector('button[aria-label*="key at 0.000 seconds"]'),
    ));
    await act(async () => {
      required<HTMLButtonElement>(
        container.querySelector('button[aria-label*="key at 0.500 seconds"]'),
      ).dispatchEvent(new MouseEvent("click", {
        bubbles: true,
        ctrlKey: true,
      }));
    });
    expect(required<HTMLInputElement>(
      container.querySelector('input[aria-label="Animation playhead"]'),
    ).valueAsNumber).toBeCloseTo(0.5);
    expect(container.textContent).toContain(
      "Editing the selected keyframe value.",
    );
    const rotationZ = Array.from(
      required<HTMLElement>(
        container.querySelector(".bone-transform-inspector"),
      ).querySelectorAll<HTMLLabelElement>("label"),
    ).find((label) => label.firstChild?.textContent?.trim() === "Z")
      ?.querySelector<HTMLInputElement>("input");
    expect(rotationZ?.valueAsNumber).toBeCloseTo(90);

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
  it("projects a bounded time window without re-sorting all markers per frame", () => {
    const candidates = Array.from({ length: 10_000 }, (_, index) => ({
      id: `key-${index}`,
      kind: "KEY" as const,
      timeSeconds: index / 100,
    }));
    const selected = new Set(["key-9999"]);
    const projected = projectTimelineMarkersV1(
      candidates,
      selected,
      5,
      2_000,
    );

    expect(projected).toHaveLength(2_000);
    expect(projected.some(({ id }) => id === "key-9999")).toBe(true);
    expect(projected.some(({ timeSeconds }) => timeSeconds === 5)).toBe(true);
    expect(projectTimelineMarkersV1(candidates.slice(0, 10), selected, 9, 2_000))
      .toEqual(candidates.slice(0, 10));
  });

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
  it("previews attack routing without mutation, then applies and reverts it explicitly", async () => {
    const attack = animationStudioClipFixtureV1({
      id: "clip-attack-demo",
      name: "attack_demo",
      status: "VALID",
    });
    const studio = animationStudioDocumentFixtureV1({
      status: "VALID",
      authoredClips: [attack],
    });
    const originalAssignments: CreatureAnimationAuthoringV2["assignments"] = [
      "cpause1",
      "ca1slashl",
      "ca1slashr",
      "ca1stab",
    ].map((targetSlot) => ({
      targetSlot: targetSlot as
        | "cpause1" | "ca1slashl" | "ca1slashr" | "ca1stab",
      sourceKind: "SOURCE_CLIP" as const,
      sourceClipName: targetSlot,
      customAnimationId: null,
      provenance: {
        provider: "SOURCE_GLB" as const,
        assetId: sourceRevision,
        ownership: "USER_OWNED" as const,
      },
    }));
    const initialAuthoring: CreatureAnimationAuthoringV2 = {
      ...emptyAuthoring(),
      assignments: originalAssignments,
      customAnimations: [
        createCustomDefinitionFromAuthoredClipV1(attack.id, {
          id: "custom-attack-demo",
          name: "Attack demo",
        }),
      ],
    };
    let latestAuthoring = initialAuthoring;
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [authoring, setAuthoring] = useState(initialAuthoring);
      latestAuthoring = authoring;
      return (
        <CustomAnimationMappingPanel
          slot="ca1slashl"
          authoring={authoring}
          studio={studio}
          onAuthoringChange={setAuthoring}
          onCreate={vi.fn()}
          onOpenClip={vi.fn()}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(buttonByText(container, "Preview attack demo (3 slots)"));
    expect(latestAuthoring).toBe(initialAuthoring);
    expect(container.textContent).toContain(
      "No production mapping has changed.",
    );

    await click(buttonByText(container, "Apply attack demo routing"));
    expect(latestAuthoring.assignments.filter(({ targetSlot }) => (
      targetSlot.startsWith("ca1")
    )).every(({ customAnimationId }) => (
      customAnimationId === "custom-attack-demo"
    ))).toBe(true);
    expect(latestAuthoring.assignments.find(({ targetSlot }) => (
      targetSlot === "cpause1"
    ))).toEqual(originalAssignments[0]);

    await click(buttonByText(container, "Revert applied attack demo"));
    expect(latestAuthoring.assignments).toEqual(originalAssignments);
    expect(latestAuthoring.authoringRevision)
      .toBe(initialAuthoring.authoringRevision + 2);
  });

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

  it("uses a compatible repository preset as a provenance-complete Draft Custom clip", async () => {
    let latestDocument = emptyDocument();
    const instantiate = vi.fn(async (
      preset: Parameters<NonNullable<React.ComponentProps<typeof AnimationStudioWorkspace>["onInstantiateLibraryPreset"]>>[0],
      newId: string,
      newName: string,
    ) => libraryPresetClip(newId, newName, preset));
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
          onInspectLibraryPreset={async (preset) => ({
            schemaVersion: 1,
            status: "COMPATIBLE",
            expectedRigSignatureSha256: preset.rigSignatureSha256,
            actualRigSignatureSha256: preset.rigSignatureSha256,
            missingBones: [],
            diagnostics: [],
          })}
          onInstantiateLibraryPreset={instantiate}
          onUndo={vi.fn()}
          onRedo={vi.fn()}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(tabByText(container, "Built-in", "Animation clip source"));
    await vi.waitFor(() => {
      expect(libraryOptionByText(container, "Right cross")).toBeDefined();
    });
    await click(libraryOptionByText(container, "Right cross"));
    await vi.waitFor(() => {
      expect(buttonByText(container, "Use as template").disabled).toBe(false);
    });
    await click(buttonByText(container, "Use as template"));
    await vi.waitFor(() => {
      expect(instantiate).toHaveBeenCalledTimes(1);
    });

    expect(instantiate).toHaveBeenCalledWith(
      expect.objectContaining({ presetId: "m2a_right_cross" }),
      expect.stringMatching(/^library-m2a_right_cross-/),
      "m2a_rightcross",
    );
    expect(latestDocument.schemaVersion).toBe(2);
    expect(latestDocument.authoringRevision).toBe(2);
    expect(latestDocument.authoredClips).toHaveLength(1);
    expect(latestDocument.authoredClips[0]).toMatchObject({
      name: "m2a_rightcross",
      status: "DRAFT",
      source: {
        kind: "LIBRARY_PRESET_COPY",
        sourceRevision,
        libraryPreset: {
          presetId: "m2a_right_cross",
          presetVersion: 1,
          source: "BUILT_IN",
          instantiationMode: "STRICT_RIG_V1",
        },
      },
    });
  });

  it("does not add a library clip when the project changes during instantiation", async () => {
    let latestDocument = emptyDocument();
    let updateDocument: ((document: AnimationStudioDocumentV1) => void) | null = null;
    let resolveInstantiation: (() => void) | null = null;
    const instantiate = vi.fn((
      preset: Parameters<NonNullable<React.ComponentProps<typeof AnimationStudioWorkspace>["onInstantiateLibraryPreset"]>>[0],
      newId: string,
      newName: string,
    ) => new Promise<ReturnType<typeof libraryPresetClip>>((resolve) => {
      resolveInstantiation = () => resolve(libraryPresetClip(newId, newName, preset));
    }));
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);

    function Harness() {
      const [studio, setStudio] = useState(emptyDocument());
      latestDocument = studio;
      updateDocument = setStudio;
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
          onInspectLibraryPreset={async (preset) => ({
            schemaVersion: 1,
            status: "COMPATIBLE",
            expectedRigSignatureSha256: preset.rigSignatureSha256,
            actualRigSignatureSha256: preset.rigSignatureSha256,
            missingBones: [],
            diagnostics: [],
          })}
          onInstantiateLibraryPreset={instantiate}
          onUndo={vi.fn()}
          onRedo={vi.fn()}
          canUndo={false}
          canRedo={false}
        />
      );
    }

    await act(async () => root.render(<Harness />));
    await click(tabByText(container, "Built-in", "Animation clip source"));
    await vi.waitFor(() => {
      expect(libraryOptionByText(container, "Right cross")).toBeDefined();
    });
    await click(libraryOptionByText(container, "Right cross"));
    await vi.waitFor(() => {
      expect(buttonByText(container, "Use as template").disabled).toBe(false);
    });
    await click(buttonByText(container, "Use as template"));
    await vi.waitFor(() => {
      expect(instantiate).toHaveBeenCalledTimes(1);
    });
    await act(async () => {
      const setStudio = required(updateDocument);
      setStudio({ ...latestDocument, authoringRevision: 2 });
    });
    await act(async () => required(resolveInstantiation)());
    await flushPromises();

    expect(latestDocument.authoringRevision).toBe(2);
    expect(latestDocument.authoredClips).toEqual([]);
    expect(container.querySelector('[role="alert"]')?.textContent).toContain(
      "document changed while the library preset was loading",
    );
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

function libraryOptionByText(container: HTMLElement, text: string): HTMLButtonElement {
  return required(Array.from(container.querySelectorAll<HTMLButtonElement>(
    '.animation-clip-library [role="option"]',
  )).find((button) => button.textContent?.includes(text)));
}

async function flushPromises() {
  await act(async () => {
    await Promise.resolve();
    await new Promise((resolve) => setTimeout(resolve, 10));
  });
}

function libraryPresetClip(
  id: string,
  name: string,
  preset: Parameters<NonNullable<React.ComponentProps<typeof AnimationStudioWorkspace>["onInstantiateLibraryPreset"]>>[0],
) {
  return animationStudioClipFixtureV1({
    id,
    name,
    status: "DRAFT",
    source: {
      kind: "LIBRARY_PRESET_COPY",
      sourceRevision,
      sourceClipName: null,
      sourceClipFingerprint: preset.motionSha256,
      proceduralTemplate: null,
      libraryPreset: {
        presetId: preset.presetId,
        presetVersion: preset.presetVersion,
        presetMotionSha256: preset.motionSha256,
        catalogSha256: "c".repeat(64),
        source: preset.source,
        authors: preset.authors.map(({ name: author }) => author),
        license: preset.license,
        rigSignatureSha256: preset.rigSignatureSha256,
        instantiationMode: "STRICT_RIG_V1",
      },
    },
  });
}
