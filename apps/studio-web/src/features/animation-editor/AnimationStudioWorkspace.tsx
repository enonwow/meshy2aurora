import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type {
  AnimationStudioDiagnosticV1,
  AnimationStudioDocumentV1,
  AuthoredAnimationClipV1,
  AuthoredAnimationEventV1,
  CreatureAnimationAuthoringV2,
} from "../animation-studio/types";
import { AnimationBoneTree, type AnimationRigNodeV1 } from "./AnimationBoneTree";
import { AnimationClipLibrary } from "./AnimationClipLibrary";
import { AnimationDopeSheet } from "./AnimationDopeSheet";
import { AnimationEditorDiagnostics } from "./AnimationEditorDiagnostics";
import { AnimationEditorViewport } from "./AnimationEditorViewport";
import { AnimationEventEditor } from "./AnimationEventEditor";
import { ImportAnimationFromModelDialog } from "./ImportAnimationFromModelDialog";
import { AnimationRetimeDialog } from "./AnimationRetimeDialog";
import {
  type AnimationStudioAutosaveStateV1,
  AnimationStudioAutosaveStatus,
} from "./AnimationStudioAutosaveStatus";
import { AnimationTrimDialog } from "./AnimationTrimDialog";
import { BoneTransformInspector } from "./BoneTransformInspector";
import {
  animationKeySelectionIdV1,
  cloneSourceClipForEditingV1,
  collectAnimationKeyTimesV1,
  createAnimationEventDraftV1,
  createBlankPoseClipV1,
  createCustomDefinitionFromAuthoredClipV1,
  createProceduralTemplateClipV1,
  deleteSelectedKeysV1,
  getAuthoredClipUsageV1,
  insertKeyAtPlayheadV1,
  moveSelectedKeysV1,
  projectAnimationStudioLibraryV1,
  retimeAnimationEditorSelectionV1,
  selectKeysInRangeV1,
  trimAnimationEditorSelectionV1,
  validateAnimationEventDraftV1,
  type AnimationStudioLibrarySourceV1,
} from "./editing";
import type { ExternalAnimationSourceInspectionV1 } from "./animationImport";
import "./animation-editor.css";

export interface AnimationStudioWorkspaceProps {
  document: AnimationStudioDocumentV1;
  authoring: CreatureAnimationAuthoringV2;
  sourceInventory: readonly AnimationStudioLibrarySourceV1[];
  rig: readonly AnimationRigNodeV1[];
  viewport: ReactNode | ((
    clip: AuthoredAnimationClipV1 | null,
    playheadSeconds: number,
  ) => ReactNode);
  sourceViewport?: ReactNode | ((
    clip: AuthoredAnimationClipV1 | null,
    playheadSeconds: number,
  ) => ReactNode);
  autosaveState: AnimationStudioAutosaveStateV1;
  diagnostics: readonly AnimationStudioDiagnosticV1[];
  onDocumentChange: (document: AnimationStudioDocumentV1) => void;
  onAuthoringChange: (authoring: CreatureAnimationAuthoringV2) => void;
  onEditSourceClip?: (
    sourceClipId: string,
    newId: string,
    newName: string,
  ) => Promise<AuthoredAnimationClipV1>;
  onInspectAnimationModel?: (
    file: File,
  ) => Promise<ExternalAnimationSourceInspectionV1>;
  onImportAnimationModelClip?: (
    file: File,
    clipName: string,
    newId: string,
    newName: string,
  ) => Promise<AuthoredAnimationClipV1>;
  onValidateClip?: (
    candidate: AuthoredAnimationClipV1,
    prospectiveDocument: AnimationStudioDocumentV1,
  ) => Promise<readonly AnimationStudioDiagnosticV1[]>;
  onUndo: () => void;
  onRedo: () => void;
  canUndo: boolean;
  canRedo: boolean;
  requestedClipId?: string | null;
  onSelectedClipChange?: (clipId: string | null) => void;
}

export function AnimationStudioWorkspace({
  document,
  authoring,
  sourceInventory,
  rig,
  viewport,
  sourceViewport,
  autosaveState,
  diagnostics,
  onDocumentChange,
  onAuthoringChange,
  onEditSourceClip,
  onInspectAnimationModel,
  onImportAnimationModelClip,
  onValidateClip,
  onUndo,
  onRedo,
  canUndo,
  canRedo,
  requestedClipId,
  onSelectedClipChange,
}: AnimationStudioWorkspaceProps) {
  const items = useMemo(
    () => projectAnimationStudioLibraryV1(document, sourceInventory),
    [document, sourceInventory],
  );
  const invalidRepairActions = useMemo(() => {
    const invalidItems = items.filter(({ status }) => status === "INVALID");
    const blockingDiagnostics = diagnostics.filter(
      ({ level }) => level === "BLOCKING",
    );
    return Object.fromEntries(invalidItems.map((item) => {
      const related = diagnostics.find((diagnostic) => (
        diagnostic.level === "BLOCKING"
        && (
          diagnostic.path.includes(item.authoredClipId ?? item.id)
          || diagnostic.path.includes(item.name)
        )
      ));
      const firstBlocking = related ?? (
        invalidItems.length === 1 ? blockingDiagnostics[0] : undefined
      );
      return [item.id, firstBlocking?.action];
    }).filter((entry): entry is [string, string] => entry[1] !== undefined),
    );
  }, [diagnostics, items]);
  const [selectedId, setSelectedId] = useState<string | null>(
    requestedClipId ?? document.authoredClips[0]?.id ?? items[0]?.id ?? null,
  );
  const selectClip = (clipId: string | null) => {
    setSelectedId(clipId);
    onSelectedClipChange?.(clipId);
  };
  useEffect(() => {
    if (
      requestedClipId !== undefined
      && requestedClipId !== selectedId
      && (
        requestedClipId === null
        || items.some(({ id }) => id === requestedClipId)
      )
    ) {
      setSelectedId(requestedClipId);
    }
  }, [items, requestedClipId, selectedId]);
  const selectedClip = document.authoredClips.find(({ id }) => id === selectedId) ?? null;
  const [selectedBoneId, setSelectedBoneId] = useState<number | null>(
    rig[0]?.id ?? null,
  );
  const [path, setPath] = useState<"ROTATION" | "TRANSLATION">("ROTATION");
  const [playheadSeconds, setPlayheadSeconds] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [selectedKeyIds, setSelectedKeyIds] = useState<Set<string>>(new Set());
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);
  const [dialog, setDialog] = useState<"TRIM" | "RETIME" | null>(null);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [gestureDelta, setGestureDelta] = useState<number | null>(null);
  const [operationError, setOperationError] = useState<{
    code: string;
    message: string;
    action: string;
  } | null>(null);
  const [validatingClipId, setValidatingClipId] = useState<string | null>(null);
  const saveValidationGenerationRef = useRef(0);
  const documentRef = useRef(document);
  const authoringRef = useRef(authoring);
  documentRef.current = document;
  authoringRef.current = authoring;
  const playStartedAt = useRef<number | null>(null);
  const playheadAtStart = useRef(0);
  const dialogReturnFocusRef = useRef<HTMLElement | null>(null);
  const eventReturnFocusRef = useRef<HTMLElement | null>(null);
  const importReturnFocusRef = useRef<HTMLElement | null>(null);
  const prefersReducedMotion = usePrefersReducedMotionV1();

  useEffect(() => {
    if (!selectedClip) setPlaying(false);
    if (selectedClip && playheadSeconds > selectedClip.lengthSeconds) {
      setPlayheadSeconds(selectedClip.lengthSeconds);
    }
  }, [playheadSeconds, selectedClip]);

  useEffect(() => {
    if (!playing || !selectedClip) return;
    playStartedAt.current = performance.now();
    playheadAtStart.current = playheadSeconds;
    let frame = 0;
    const tick = (now: number) => {
      const elapsed = (now - (playStartedAt.current ?? now)) / 1000;
      const next = playheadAtStart.current + elapsed;
      if (next >= selectedClip.lengthSeconds) {
        setPlayheadSeconds(selectedClip.lengthSeconds);
        setPlaying(false);
        return;
      }
      setPlayheadSeconds(next);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [playing, selectedClip]);

  useEffect(() => {
    if (prefersReducedMotion) setPlaying(false);
  }, [prefersReducedMotion]);

  const selectedBone = rig.find(({ id }) => id === selectedBoneId) ?? null;
  const selectedEvent = selectedClip?.events.find(({ id }) => id === selectedEventId)
    ?? null;
  const openDialog = (kind: "TRIM" | "RETIME") => {
    dialogReturnFocusRef.current = globalThis.document.activeElement instanceof HTMLElement
      ? globalThis.document.activeElement
      : null;
    setDialog(kind);
  };
  const closeDialog = () => {
    setDialog(null);
    queueMicrotask(() => dialogReturnFocusRef.current?.focus());
  };
  const openEventEditor = (eventId: string) => {
    eventReturnFocusRef.current = globalThis.document.activeElement instanceof HTMLElement
      ? globalThis.document.activeElement
      : null;
    setSelectedEventId(eventId);
  };
  const closeEventEditor = () => {
    setSelectedEventId(null);
    queueMicrotask(() => eventReturnFocusRef.current?.focus());
  };
  const openImportDialog = () => {
    importReturnFocusRef.current = globalThis.document.activeElement instanceof HTMLElement
      ? globalThis.document.activeElement
      : null;
    setOperationError(null);
    setImportDialogOpen(true);
  };
  const closeImportDialog = () => {
    setImportDialogOpen(false);
    queueMicrotask(() => importReturnFocusRef.current?.focus());
  };

  const commitDocument = (
    authoredClips: readonly AuthoredAnimationClipV1[],
    status: AnimationStudioDocumentV1["status"] = "DRAFT",
  ) => onDocumentChange({
    ...document,
    status,
    authoringRevision: document.authoringRevision + 1,
    authoredClips: [...authoredClips],
  });
  const commitClip = (clip: AuthoredAnimationClipV1) => {
    commitDocument(document.authoredClips.map((item) => item.id === clip.id ? clip : item));
    selectClip(clip.id);
  };

  const createBlank = () => {
    const id = uniqueId("authored");
    const created = createBlankPoseClipV1({
      id,
      name: uniqueOutputName("custom_animation", document.authoredClips),
      sourceRevision: document.sourceRevision,
      animationRoot: rig.find(({ parentId }) => parentId === null)?.name
        ?? rig[0]?.name
        ?? "root",
      rig,
    });
    commitDocument([...document.authoredClips, created]);
    selectClip(created.id);
  };

  const createProcedural = () => {
    setOperationError(null);
    try {
      const id = uniqueId("authored");
      const created = createProceduralTemplateClipV1({
        id,
        name: uniqueOutputName(
          "procedural_root_translation_pulse",
          document.authoredClips,
        ),
        sourceRevision: document.sourceRevision,
        animationRoot: rig.find(({ parentId }) => parentId === null)?.name
          ?? rig[0]?.name
          ?? "root",
        rig,
      }, "ROOT_TRANSLATION_PULSE");
      commitDocument([...document.authoredClips, created]);
      selectClip(created.id);
    } catch (error: unknown) {
      setOperationError({
        code: "M2A-ANIMATION-EDIT-BONE-MISSING",
        message: error instanceof Error ? error.message : String(error),
        action: "Inspect the exact output rig and retry the procedural template.",
      });
    }
  };

  const editCopy = (libraryId: string) => {
    const sourceId = libraryId.replace(/^source:/, "");
    const source = sourceInventory.find(({ clipId }) => clipId === sourceId);
    if (!source) return;
    const id = uniqueId("authored");
    const name = uniqueOutputName(`${source.name}_edited`, document.authoredClips);
    if (!onEditSourceClip) {
      const placeholder = createBlankPoseClipV1({
        id,
        name,
        sourceRevision: document.sourceRevision,
        animationRoot: rig.find(({ parentId }) => parentId === null)?.name ?? "root",
        lengthSeconds: Math.max(source.durationSeconds, 0.001),
        rig,
      });
      commitDocument([
        ...document.authoredClips,
        {
          ...placeholder,
          source: {
            kind: "SOURCE_CLIP_COPY",
            sourceRevision: document.sourceRevision,
            sourceClipName: source.name,
            sourceClipFingerprint: source.clipId,
            proceduralTemplate: null,
          },
        },
      ]);
      selectClip(id);
      return;
    }
    setOperationError(null);
    void onEditSourceClip(sourceId, id, name).then((clip) => {
      commitDocument([...document.authoredClips, clip]);
      selectClip(clip.id);
    }).catch((error: unknown) => {
      setOperationError({
        code: "M2A-ANIMATION-EDIT-SOURCE-COPY",
        message: error instanceof Error ? error.message : String(error),
        action: "Retry Edit copy with the exact current source clip.",
      });
    });
  };

  const duplicate = (clipId: string) => {
    const source = document.authoredClips.find(({ id }) => id === clipId);
    if (!source) return;
    const copy = cloneSourceClipForEditingV1(source, {
      id: uniqueId("authored"),
      name: uniqueOutputName(`${source.name}_copy`, document.authoredClips),
    });
    commitDocument([...document.authoredClips, copy]);
    selectClip(copy.id);
  };

  const importAnimationModelClip = async (file: File, clipName: string) => {
    if (!onImportAnimationModelClip) {
      throw new Error("Animation import is unavailable in this build.");
    }
    const currentDocument = documentRef.current;
    const id = uniqueId("authored");
    const name = uniquePortableImportedName(
      clipName,
      currentDocument.authoredClips,
    );
    const clip = await onImportAnimationModelClip(file, clipName, id, name);
    const latestDocument = documentRef.current;
    onDocumentChange({
      ...latestDocument,
      status: "DRAFT",
      authoringRevision: latestDocument.authoringRevision + 1,
      authoredClips: [...latestDocument.authoredClips, clip],
    });
    selectClip(clip.id);
  };

  const saveClip = () => {
    if (!selectedClip) return;
    const eventDiagnostics = selectedClip.events.flatMap((event) => (
      validateAnimationEventDraftV1(event, selectedClip)
    ));
    const hasKeys = selectedClip.tracks.some(({ keyframes }) => keyframes.length > 0);
    const localDiagnostics: AnimationStudioDiagnosticV1[] = [
      ...eventDiagnostics,
      ...(!hasKeys ? [{
        schemaVersion: 1 as const,
        code: "M2A-ANIMATION-EDIT-NO-MOTION",
        path: `authoredClips[${selectedClip.id}].tracks`,
        level: "BLOCKING" as const,
        message: "The authored clip has no transform keyframes.",
        action: "Add at least one rotation or translation keyframe, then save again.",
      }] : []),
    ];
    const candidate: AuthoredAnimationClipV1 = {
      ...selectedClip,
      kind: hasChangingMotion(selectedClip) ? "MOTION" : "STATIC_POSE",
      status: "VALID",
      revision: selectedClip.revision + 1,
    };
    const prospectiveDocument: AnimationStudioDocumentV1 = {
      ...document,
      authoredClips: document.authoredClips.map((clip) => (
        clip.id === candidate.id ? candidate : clip
      )),
    };
    const sourceDocumentRevision = document.authoringRevision;
    const sourceClipRevision = selectedClip.revision;
    const validationGeneration = saveValidationGenerationRef.current + 1;
    saveValidationGenerationRef.current = validationGeneration;
    setOperationError(null);
    setValidatingClipId(candidate.id);

    const finishSave = (
      exactDiagnostics: readonly AnimationStudioDiagnosticV1[],
    ) => {
      if (saveValidationGenerationRef.current !== validationGeneration) return;
      setValidatingClipId(null);
      const currentDocument = documentRef.current;
      const currentClip = currentDocument.authoredClips.find(
        ({ id }) => id === candidate.id,
      );
      if (
        currentDocument.authoringRevision !== sourceDocumentRevision
        || currentClip?.revision !== sourceClipRevision
      ) {
        setOperationError({
          code: "M2A-ANIMATION-EDIT-VALIDATION-STALE",
          message: "The clip changed while exact validation was running.",
          action: "Review the latest edit and choose Save to Custom again.",
        });
        return;
      }
      const blockers = [...localDiagnostics, ...exactDiagnostics].filter(
        ({ level }) => level === "BLOCKING",
      );
      const status = blockers.length > 0 ? "INVALID" : "VALID";
      const saved: AuthoredAnimationClipV1 = { ...candidate, status };
      const authoredClips = currentDocument.authoredClips.map((clip) => (
        clip.id === saved.id ? saved : clip
      ));
      onDocumentChange({
        ...currentDocument,
        status: deriveAnimationStudioDocumentStatusV1(authoredClips),
        authoringRevision: currentDocument.authoringRevision + 1,
        authoredClips,
      });
      selectClip(saved.id);
      if (blockers[0]) {
        setOperationError({
          code: blockers[0].code,
          message: blockers[0].message,
          action: blockers[0].action,
        });
        return;
      }
      const currentAuthoring = authoringRef.current;
      if (!currentAuthoring.customAnimations.some((custom) => (
        custom.clipReference?.authoredClipId === saved.id
        || custom.phases.some(({ clipReference }) => (
          clipReference.authoredClipId === saved.id
        ))
      ))) {
        onAuthoringChange({
          ...currentAuthoring,
          authoringRevision: currentAuthoring.authoringRevision + 1,
          customAnimations: [
            ...currentAuthoring.customAnimations,
            createCustomDefinitionFromAuthoredClipV1(saved.id, {
              id: uniqueId("custom"),
              name: saved.name,
            }),
          ],
        });
      }
    };

    if (!onValidateClip) {
      finishSave([]);
      return;
    }
    void onValidateClip(candidate, prospectiveDocument)
      .then(finishSave)
      .catch((error: unknown) => finishSave([{
        schemaVersion: 1,
        code: "M2A-ANIMATION-EDIT-WASM",
        path: `authoredClips[${candidate.id}]`,
        level: "BLOCKING",
        message: error instanceof Error ? error.message : String(error),
        action: "Retry exact core validation for the current source GLB.",
      }]));
  };

  const removeClip = () => {
    if (!selectedClip) return;
    const usage = getAuthoredClipUsageV1(selectedClip.id, authoring);
    if (usage.length > 0 && !globalThis.confirm(
      `This animation is used by Base 42 slots: ${usage.join(", ")}. Remove it and clear those assignments?`,
    )) return;
    const customIds = new Set(authoring.customAnimations
      .filter((custom) => (
        custom.clipReference?.authoredClipId === selectedClip.id
        || custom.phases.some(({ clipReference }) => (
          clipReference.authoredClipId === selectedClip.id
        ))
      ))
      .map(({ id }) => id));
    onAuthoringChange({
      ...authoring,
      authoringRevision: authoring.authoringRevision + 1,
      assignments: authoring.assignments.filter(({ customAnimationId }) => (
        customAnimationId === null || !customIds.has(customAnimationId)
      )),
      customAnimations: authoring.customAnimations.filter(({ id }) => !customIds.has(id)),
    });
    commitDocument(document.authoredClips.filter(({ id }) => id !== selectedClip.id));
    selectClip(null);
  };

  const insertTransformKey = (value: number[]) => {
    if (!selectedClip || selectedBoneId === null) return;
    commitClip(insertKeyAtPlayheadV1(selectedClip, {
      targetNodeId: selectedBoneId,
      path,
      value,
    }, playheadSeconds, undefined, "REPLACE_EXISTING"));
  };

  const moveSelectedKeys = (deltaSeconds: number) => {
    if (!selectedClip || selectedKeyIds.size === 0) return;
    const unsupportedTrack = selectedClip.tracks.find((track) => (
      track.interpolation !== "LINEAR"
      && track.keyframes.some(({ id }) => selectedKeyIds.has(
        animationKeySelectionIdV1(track.id, id),
      ))
    ));
    if (unsupportedTrack) {
      setOperationError({
        code: "M2A-ANIMATION-EDIT-PATH-UNSUPPORTED",
        message: "Animation Studio edits only LINEAR keyframe tracks.",
        action: "Convert the selected track to LINEAR before moving its keys.",
      });
      return;
    }
    setOperationError(null);
    try {
      commitClip(moveSelectedKeysV1(
        selectedClip,
        [...selectedKeyIds],
        deltaSeconds,
      ));
    } catch (error: unknown) {
      setOperationError({
        code: "M2A-ANIMATION-EDIT-TIME-NOT-STRICT",
        message: error instanceof Error ? error.message : String(error),
        action: "Move by a smaller amount so keyframe times stay strictly ordered.",
      });
    }
  };

  const updateEvent = (event: AuthoredAnimationEventV1) => {
    if (!selectedClip) return;
    commitClip({
      ...selectedClip,
      status: "DRAFT",
      revision: selectedClip.revision + 1,
      events: selectedClip.events.map((item) => item.id === event.id ? event : item)
        .map((item, stableOrder) => ({ item, stableOrder }))
        .sort((left, right) => (
          left.item.timeSeconds - right.item.timeSeconds
          || left.stableOrder - right.stableOrder
        ))
        .map(({ item }) => item),
    });
  };

  const eventDiagnostics = selectedClip && selectedEvent
    ? validateAnimationEventDraftV1(selectedEvent, selectedClip)
    : [];

  return (
    <section
      className="animation-studio-workspace"
      aria-labelledby="animation-studio-title"
      data-layout="aurora-animation-workbench"
      data-reduced-motion={prefersReducedMotion}
      onKeyDown={(event) => {
        if (isEditableControlV1(event.target)) return;
        if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
        const key = event.key.toLocaleLowerCase("en-US");
        if (key === "z" && event.shiftKey) {
          event.preventDefault();
          if (canRedo) onRedo();
        } else if (key === "z") {
          event.preventDefault();
          if (canUndo) onUndo();
        } else if (key === "y") {
          event.preventDefault();
          if (canRedo) onRedo();
        }
      }}
    >
      <header className="animation-studio-workspace__toolbar">
        <div>
          <h2 id="animation-studio-title">Create &amp; edit animations</h2>
          <p>Non-destructive local authoring for the exact output rig.</p>
        </div>
        <div className="animation-studio-workspace__toolbar-actions">
        <AnimationStudioAutosaveStatus state={autosaveState} />
          <button
            className="animation-studio-action animation-studio-action--quiet"
            type="button"
            data-action="undo"
            disabled={!canUndo}
            onClick={onUndo}
          >
            Undo
          </button>
          <button
            className="animation-studio-action animation-studio-action--quiet"
            type="button"
            data-action="redo"
            disabled={!canRedo}
            onClick={onRedo}
          >
            Redo
        </button>
        <button
          className="animation-studio-action animation-studio-action--primary"
          type="button"
          data-action="save-custom"
          data-variant="primary"
          disabled={!selectedClip || validatingClipId !== null}
          onClick={saveClip}
        >
          {validatingClipId === selectedClip?.id
            ? "Validating\u2026"
            : "Save to Custom"}
        </button>
        </div>
      </header>
      {prefersReducedMotion ? (
        <p className="sr-only" role="status">
          Reduced motion is active. Timeline playback remains user-controlled.
        </p>
      ) : null}

      <div className="animation-studio-workspace__main">
        <AnimationClipLibrary
          items={items}
          selectedId={selectedId}
          onSelect={selectClip}
          onCreateBlank={createBlank}
          onCreateProcedural={createProcedural}
          onImportFromModel={
            onInspectAnimationModel && onImportAnimationModelClip
              ? openImportDialog
              : undefined
          }
          onEditCopy={editCopy}
          onDuplicate={duplicate}
          invalidRepairActions={invalidRepairActions}
        />
        <section
          className="animation-studio-workspace__center"
          aria-label="Animation viewport and timeline"
          data-panel="viewport-timeline"
        >
          <AnimationEditorViewport
            viewport={typeof viewport === "function"
              ? viewport(selectedClip, playheadSeconds)
              : viewport}
            sourceViewport={typeof sourceViewport === "function"
              ? sourceViewport(selectedClip, playheadSeconds)
              : sourceViewport}
            selectedBoneName={selectedBone?.name ?? null}
            playheadSeconds={playheadSeconds}
            path={path}
            onGestureBegin={() => setGestureDelta(0)}
            onGestureUpdate={setGestureDelta}
            onGestureCommit={() => {
              if (gestureDelta === null) return;
              insertTransformKey(
                path === "ROTATION"
                  ? axisAngleZQuaternion(gestureDelta)
                  : [gestureDelta, 0, 0],
              );
              setGestureDelta(null);
            }}
            onGestureCancel={() => setGestureDelta(null)}
          />
        </section>
        <aside
          className="animation-studio-workspace__inspector"
          aria-label="Animation properties"
          data-panel="animation-inspector"
        >
          <h2 className="animation-studio-workspace__inspector-title">
            Bone &amp; clip
          </h2>
          <AnimationBoneTree
            nodes={rig}
            selectedNodeId={selectedBoneId}
            onSelect={setSelectedBoneId}
          />
          <BoneTransformInspector
            selectedBoneName={selectedBone?.name ?? null}
            path={path}
            selectedKeyCount={selectedKeyIds.size}
            onPathChange={setPath}
            onInsert={insertTransformKey}
            onDeleteSelectedKeys={() => {
              if (!selectedClip || selectedKeyIds.size === 0) return;
              commitClip(deleteSelectedKeysV1(selectedClip, [...selectedKeyIds]));
              setSelectedKeyIds(new Set());
            }}
          />
          {selectedClip ? (
            <section className="animation-clip-settings" aria-labelledby="clip-settings-title">
              <h3 id="clip-settings-title">Clip settings</h3>
              <label>
                Output name
                <input
                  value={selectedClip.name}
                  onChange={(event) => commitClip({
                    ...selectedClip,
                    name: event.currentTarget.value,
                    status: "DRAFT",
                    revision: selectedClip.revision + 1,
                  })}
                />
              </label>
              <label>
                Length (s)
                <input type="number" value={selectedClip.lengthSeconds} readOnly />
              </label>
              <label>
                Transition (s)
                <input
                  type="number"
                  min={0}
                  step={0.01}
                  value={selectedClip.transitionSeconds}
                  onChange={(event) => commitClip({
                    ...selectedClip,
                    transitionSeconds: event.currentTarget.valueAsNumber,
                    status: "DRAFT",
                    revision: selectedClip.revision + 1,
                  })}
                />
              </label>
              <label>
                Animation root
                <select
                  value={selectedClip.animationRoot}
                  onChange={(event) => commitClip({
                    ...selectedClip,
                    animationRoot: event.currentTarget.value,
                    status: "DRAFT",
                    revision: selectedClip.revision + 1,
                  })}
                >
                  {rig.map((node) => <option key={node.id}>{node.name}</option>)}
                </select>
              </label>
              <p>Interpolation: Linear output</p>
              <button type="button" onClick={removeClip}>Remove clip</button>
            </section>
          ) : null}
        </aside>
        {selectedClip ? (
          <AnimationDopeSheet
            clip={selectedClip}
            rig={rig}
            playheadSeconds={playheadSeconds}
            playing={playing}
            selectedKeyIds={selectedKeyIds}
            onPlayingChange={setPlaying}
            onSeek={setPlayheadSeconds}
            onSelectKey={(id, additive) => setSelectedKeyIds((current) => {
              if (!additive) return new Set([id]);
              const next = new Set(current);
              if (next.has(id)) next.delete(id);
              else next.add(id);
              return next;
            })}
            onSelectEvent={openEventEditor}
            onAddEvent={() => {
              const event = createAnimationEventDraftV1(playheadSeconds);
              commitClip({
                ...selectedClip,
                status: "DRAFT",
                revision: selectedClip.revision + 1,
                events: [...selectedClip.events, event],
              });
              openEventEditor(event.id);
            }}
            onDeleteKeys={() => {
              commitClip(deleteSelectedKeysV1(selectedClip, [...selectedKeyIds]));
              setSelectedKeyIds(new Set());
            }}
            onMoveSelectedKeys={moveSelectedKeys}
            onSelectRange={(startSeconds, endSeconds) => {
              setSelectedKeyIds(new Set(selectKeysInRangeV1(
                selectedClip,
                { start: startSeconds, end: endSeconds },
              )));
            }}
            onOpenTrim={() => openDialog("TRIM")}
            onOpenRetime={() => openDialog("RETIME")}
          />
        ) : null}
      </div>

      {selectedEvent && selectedClip ? (
        <AnimationEventEditor
          event={selectedEvent}
          diagnostics={eventDiagnostics}
          onChange={updateEvent}
          onRemove={() => {
            commitClip({
              ...selectedClip,
              revision: selectedClip.revision + 1,
              events: selectedClip.events.filter(({ id }) => id !== selectedEvent.id),
            });
            closeEventEditor();
          }}
          onClose={closeEventEditor}
        />
      ) : null}
      {importDialogOpen && onInspectAnimationModel ? (
        <ImportAnimationFromModelDialog
          currentRig={rig}
          onInspect={onInspectAnimationModel}
          onImport={importAnimationModelClip}
          onClose={closeImportDialog}
        />
      ) : null}
      {dialog === "TRIM" && selectedClip ? (
        <AnimationTrimDialog
          lengthSeconds={selectedClip.lengthSeconds}
          onApply={(start, end) => {
            commitClip(trimAnimationEditorSelectionV1(selectedClip, start, end));
            setPlayheadSeconds(0);
            closeDialog();
          }}
          onClose={closeDialog}
        />
      ) : null}
      {dialog === "RETIME" && selectedClip ? (
        <AnimationRetimeDialog
          lengthSeconds={selectedClip.lengthSeconds}
          onApply={(length) => {
            commitClip(retimeAnimationEditorSelectionV1(selectedClip, length));
            setPlayheadSeconds(Math.min(playheadSeconds, length));
            closeDialog();
          }}
          onClose={closeDialog}
        />
      ) : null}
      <AnimationEditorDiagnostics diagnostics={[
        ...diagnostics,
        ...eventDiagnostics,
        ...(operationError ? [{
          schemaVersion: 1 as const,
          code: operationError.code,
          path: "animationStudio.operation",
          level: "BLOCKING" as const,
          message: operationError.message,
          action: operationError.action,
        }] : []),
      ]} />
      <footer
        className="animation-studio-workspace__status"
        role="status"
        aria-live="polite"
        aria-atomic="true"
      >
        <strong>{selectedClip?.status ?? "No clip"}</strong>
        <span>{selectedClip?.tracks.length ?? 0} keyed bones/tracks</span>
        <span>{selectedClip?.tracks.reduce((sum, track) => sum + track.keyframes.length, 0) ?? 0} keyframes</span>
        <span>{selectedClip?.events.length ?? 0} events</span>
        <span>{selectedClip ? collectAnimationKeyTimesV1(selectedClip).length : 0} timeline moments</span>
        <span>Source GLB unchanged</span>
      </footer>
    </section>
  );
}

function hasChangingMotion(clip: AuthoredAnimationClipV1) {
  return clip.tracks.some((track) => {
    const first = track.keyframes[0];
    if (!first) return false;
    return track.keyframes.slice(1).some((keyframe) => (
      keyframe.value.some((component, componentIndex) => (
        Math.abs(component - (first.value[componentIndex] ?? component)) > 1e-6
      ))
    ));
  });
}

function deriveAnimationStudioDocumentStatusV1(
  clips: readonly AuthoredAnimationClipV1[],
): AnimationStudioDocumentV1["status"] {
  if (clips.some(({ status }) => status === "INVALID")) return "INVALID";
  if (clips.length > 0 && clips.every(({ status }) => status === "VALID")) {
    return "VALID";
  }
  return "DRAFT";
}

function axisAngleZQuaternion(degrees: number): [number, number, number, number] {
  const half = degrees * Math.PI / 360;
  return [0, 0, Math.sin(half), Math.cos(half)];
}

function uniqueOutputName(
  base: string,
  clips: readonly AuthoredAnimationClipV1[],
) {
  const names = new Set(clips.map(({ name }) => name.toLocaleLowerCase("en-US")));
  if (!names.has(base.toLocaleLowerCase("en-US"))) return base;
  let suffix = 2;
  while (names.has(`${base}_${suffix}`.toLocaleLowerCase("en-US"))) suffix += 1;
  return `${base}_${suffix}`;
}

function uniquePortableImportedName(
  sourceName: string,
  clips: readonly AuthoredAnimationClipV1[],
) {
  const normalized = sourceName
    .replace(/[^A-Za-z0-9_]/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_+|_+$/g, "");
  const base = `imp_${normalized || "animation"}`.slice(0, 16);
  const names = new Set(clips.map(({ name }) => name.toLocaleLowerCase("en-US")));
  if (!names.has(base.toLocaleLowerCase("en-US"))) return base;
  let suffix = 2;
  while (true) {
    const suffixText = `_${suffix}`;
    const candidate = `${base.slice(0, 16 - suffixText.length)}${suffixText}`;
    if (!names.has(candidate.toLocaleLowerCase("en-US"))) return candidate;
    suffix += 1;
  }
}

function uniqueId(prefix: string) {
  return typeof crypto.randomUUID === "function"
    ? `${prefix}-${crypto.randomUUID()}`
    : `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function isEditableControlV1(target: EventTarget | null) {
  return target instanceof HTMLInputElement
    || target instanceof HTMLSelectElement
    || target instanceof HTMLTextAreaElement
    || (target instanceof HTMLElement && target.isContentEditable);
}

function usePrefersReducedMotionV1() {
  const query = "(prefers-reduced-motion: reduce)";
  const [preferred, setPreferred] = useState(() => (
    typeof window !== "undefined" && typeof window.matchMedia === "function"
      ? window.matchMedia(query).matches
      : false
  ));
  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return;
    }
    const media = window.matchMedia(query);
    const update = () => setPreferred(media.matches);
    update();
    media.addEventListener?.("change", update);
    return () => media.removeEventListener?.("change", update);
  }, []);
  return preferred;
}
