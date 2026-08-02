import {
  useCallback,
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
import {
  migrateAnimationStudioDocumentToV3,
  migrateAnimationStudioDocumentV1ToV2,
} from "../animation-studio/schema";
import type {
  AnimationContributionMetadataV1,
  AnimationPresetCatalogEntryV1,
  AnimationPresetCompatibilityV1,
} from "../animation-library/types";
import { AnimationBoneTree, type AnimationRigNodeV1 } from "./AnimationBoneTree";
import { AnimationClipLibrary } from "./AnimationClipLibrary";
import { AnimationContributionDialog } from "./AnimationContributionDialog";
import { AnimationDopeSheet } from "./AnimationDopeSheet";
import { AnimationEditorDiagnostics } from "./AnimationEditorDiagnostics";
import { AnimationEditorViewport } from "./AnimationEditorViewport";
import { AnimationEventEditor } from "./AnimationEventEditor";
import { AnimationQualityPanel } from "./AnimationQualityPanel";
import { AnimationCurvePanel } from "./AnimationCurvePanel";
import { AnimationLayerPanel } from "./AnimationLayerPanel";
import { AnimationSequencePanel } from "./AnimationSequencePanel";
import { AnimationWorkbenchPanel } from "./AnimationWorkbenchPanel";
import {
  ImportAnimationFromModelDialog,
  type AnimationModelDonorV1,
} from "./ImportAnimationFromModelDialog";
import { AnimationRetimeDialog } from "./AnimationRetimeDialog";
import {
  type AnimationStudioAutosaveStateV1,
  AnimationStudioAutosaveStatus,
} from "./AnimationStudioAutosaveStatus";
import { AnimationTrimDialog } from "./AnimationTrimDialog";
import { BoneTransformInspector } from "./BoneTransformInspector";
import {
  HeldEquipmentPanel,
  type HeldEquipmentPreviewV1,
  type HeldWeaponSourceInspectionV1,
} from "./HeldEquipmentPanel";
import {
  animationKeySelectionIdV1,
  cloneSourceClipForEditingV1,
  collectAnimationKeyTimesV1,
  createAnimationEventDraftV1,
  createBlankPoseClipV1,
  createCustomDefinitionFromAuthoredClipV1,
  deleteSelectedKeysV1,
  getAuthoredClipUsageV1,
  insertKeyAtPlayheadV1,
  moveSelectedKeysV1,
  projectAnimationStudioLibraryV1,
  retimeAnimationEditorSelectionV1,
  sampleAnimationTrackLinearV1,
  selectKeysInRangeV1,
  trimAnimationEditorSelectionV1,
  validateAnimationEventDraftV1,
  type AnimationStudioLibrarySourceV1,
} from "./editing";
import type { ExternalAnimationSourceInspectionV1 } from "./animationImport";
import type {
  AnimationTransferModeV1,
  AnimationTransferBatchResultV2,
  HumanoidSemanticBoneMapV2,
  HumanoidSemanticOverrideV2,
} from "./animationImport";
import type {
  AnimationSequencePreviewRequestV1,
  AnimationSequencePreviewV1,
} from "./animationSequence";
import {
  type AnimationCurveResamplePolicyV1,
  type AnimationCurveResampleReportV1,
  type AnimationEditorCurveTrackV1,
} from "./animationCurves";
import type {
  AnimationEditLayerV1,
  AnimationLayerBakeReportV1,
  AnimationStudioRigWireV1,
} from "./animationLayers";
import type {
  AnimationWorkbenchRequestV1,
  AnimationWorkbenchResultV1,
  AnimationWorkbenchProjectStateV1,
  MotionVisualizationV1,
} from "./animationWorkbench";
import {
  DEFAULT_ANIMATION_QUALITY_POLICY_V1,
  type AnalyzeAnimationMotionQualityV1,
  animationAuthoringToolResultClipV1,
  type AnimationEditCommandV1,
  type AnimationAuthoringToolRequestV1,
  type AnimationQualityIssueV1,
  type AnimationQualityReportV1,
  type ApplyAnimationEditCommandBatchV1,
  type ApplyAnimationAuthoringToolV1,
} from "../animation-authoring-v2/types";
import "./animation-editor.css";

export interface AnimationViewportPlaybackUpdateV1 {
  readonly timeSeconds: number;
  readonly playing: boolean;
}

export interface AnimationViewportPlaybackControlV1 {
  readonly playing: boolean;
  readonly onUpdate: (snapshot: AnimationViewportPlaybackUpdateV1) => void;
}

export interface AnimationViewportTransformControlV1 {
  readonly selectedBone: AnimationRigNodeV1 | null;
  readonly path: "ROTATION" | "TRANSLATION";
  readonly space: "local" | "world";
  readonly enabled: boolean;
  readonly onBegin: () => void;
  readonly onPreview: (value: readonly number[]) => void;
  readonly onCommit: (value: readonly number[]) => void;
  readonly heldWeapon: HeldEquipmentPreviewV1 | null;
  readonly motionVisualization: MotionVisualizationV1 | null;
}

export interface AnimationStudioWorkspaceProps {
  document: AnimationStudioDocumentV1;
  authoring: CreatureAnimationAuthoringV2;
  sourceInventory: readonly AnimationStudioLibrarySourceV1[];
  rig: readonly AnimationRigNodeV1[];
  viewport: ReactNode | ((
    clip: AuthoredAnimationClipV1 | null,
    playheadSeconds: number,
    playback: AnimationViewportPlaybackControlV1,
    transform: AnimationViewportTransformControlV1,
  ) => ReactNode);
  sourceViewport?: ReactNode | ((
    clip: AuthoredAnimationClipV1 | null,
    playheadSeconds: number,
    playback: AnimationViewportPlaybackControlV1,
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
    overrides?: readonly HumanoidSemanticOverrideV2[],
    manualMappingConfirmed?: boolean,
  ) => Promise<ExternalAnimationSourceInspectionV1>;
  onImportAnimationModelClip?: (
    file: File,
    clipName: string,
    donorSourceRevision: string,
    mode: AnimationTransferModeV1,
    semanticMap: HumanoidSemanticBoneMapV2 | undefined,
    newId: string,
    newName: string,
  ) => Promise<AuthoredAnimationClipV1>;
  onPrepareAnimationModelBatch?: (
    file: File,
    inspection: ExternalAnimationSourceInspectionV1,
    clipNames: readonly string[],
    mode: AnimationTransferModeV1,
    semanticMap?: HumanoidSemanticBoneMapV2,
  ) => Promise<AnimationTransferBatchResultV2>;
  renderAnimationTransferPreview?: (
    clip: AuthoredAnimationClipV1,
    donorFile: File,
    donorClipName: string,
    donorSourceRevision: string,
  ) => ReactNode;
  animationModelDonors?: readonly AnimationModelDonorV1[];
  onValidateClip?: (
    candidate: AuthoredAnimationClipV1,
    prospectiveDocument: AnimationStudioDocumentV1,
  ) => Promise<readonly AnimationStudioDiagnosticV1[]>;
  onInspectLibraryPreset?: (
    preset: AnimationPresetCatalogEntryV1,
  ) => Promise<AnimationPresetCompatibilityV1>;
  onInstantiateLibraryPreset?: (
    preset: AnimationPresetCatalogEntryV1,
    newId: string,
    newName: string,
  ) => Promise<AuthoredAnimationClipV1>;
  onExportAnimationContribution?: (
    clip: AuthoredAnimationClipV1,
    metadata: AnimationContributionMetadataV1,
  ) => Promise<unknown>;
  onApplyAnimationEditCommandBatch?: ApplyAnimationEditCommandBatchV1;
  onAnalyzeMotionQuality?: AnalyzeAnimationMotionQualityV1;
  onApplyAnimationAuthoringTool?: ApplyAnimationAuthoringToolV1;
  onInspectHeldWeapon?: (file: File) => Promise<HeldWeaponSourceInspectionV1>;
  onHeldEquipmentRuntimeChange?: (value: HeldEquipmentPreviewV1 | null) => void;
  onBuildAnimationSequencePreview?: (
    request: AnimationSequencePreviewRequestV1,
    availableClips: readonly AuthoredAnimationClipV1[],
  ) => Promise<AnimationSequencePreviewV1>;
  onApplyAnimationWorkbenchOperation?: (
    request: AnimationWorkbenchRequestV1,
  ) => Promise<AnimationWorkbenchResultV1>;
  animationWorkbenchProjectState?: AnimationWorkbenchProjectStateV1;
  onAnimationWorkbenchProjectStateChange?: (
    value: AnimationWorkbenchProjectStateV1,
  ) => void;
  onResampleAnimationCurve?: (
    curve: AnimationEditorCurveTrackV1,
    policy: AnimationCurveResamplePolicyV1,
  ) => Promise<AnimationCurveResampleReportV1>;
  onBakeAnimationLayers?: (
    layers: readonly AnimationEditLayerV1[],
    rig: AnimationStudioRigWireV1,
  ) => Promise<AnimationLayerBakeReportV1>;
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
  onPrepareAnimationModelBatch,
  renderAnimationTransferPreview,
  animationModelDonors = [],
  onValidateClip,
  onInspectLibraryPreset,
  onInstantiateLibraryPreset,
  onExportAnimationContribution,
  onApplyAnimationEditCommandBatch,
  onAnalyzeMotionQuality,
  onApplyAnimationAuthoringTool,
  onInspectHeldWeapon,
  onHeldEquipmentRuntimeChange,
  onBuildAnimationSequencePreview,
  onApplyAnimationWorkbenchOperation,
  animationWorkbenchProjectState,
  onAnimationWorkbenchProjectStateChange,
  onResampleAnimationCurve,
  onBakeAnimationLayers,
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
  const [gizmoSpace, setGizmoSpace] = useState<"local" | "world">("local");
  const [gizmoPreviewValue, setGizmoPreviewValue] = useState<readonly number[] | null>(
    null,
  );
  const [heldEquipment, setHeldEquipment] = useState<HeldEquipmentPreviewV1 | null>(null);
  const [sequencePreview, setSequencePreview] = useState<AnimationSequencePreviewV1 | null>(null);
  const [workbenchPreviewClip, setWorkbenchPreviewClip] = useState<AuthoredAnimationClipV1 | null>(null);
  const [motionVisualization, setMotionVisualization] = useState<MotionVisualizationV1 | null>(null);
  const [sequenceBusy, setSequenceBusy] = useState(false);
  const [sequenceError, setSequenceError] = useState<string | null>(null);
  const [playheadSeconds, setPlayheadSeconds] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [selectedKeyIds, setSelectedKeyIds] = useState<Set<string>>(new Set());
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);
  const [dialog, setDialog] = useState<"TRIM" | "RETIME" | null>(null);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [contributionDialogOpen, setContributionDialogOpen] = useState(false);
  const [gestureDelta, setGestureDelta] = useState<number | null>(null);
  const [qualityLoopExpected, setQualityLoopExpected] = useState(false);
  const [qualityRootNodeId, setQualityRootNodeId] = useState(
    rig.find(({ parentId }) => parentId === null)?.id ?? rig[0]?.id ?? 0,
  );
  const [qualityContactNodeIds, setQualityContactNodeIds] = useState<Set<number>>(
    () => new Set(rig
      .filter(({ name }) => /(?:foot|toe|ankle)/iu.test(name))
      .map(({ id }) => id)),
  );
  const [qualityReport, setQualityReport] = useState<AnimationQualityReportV1 | null>(
    null,
  );
  const [qualityLoading, setQualityLoading] = useState(false);
  const [qualityError, setQualityError] = useState<string | null>(null);
  const [qualityToolBusy, setQualityToolBusy] = useState(false);
  const [qualityToolMessage, setQualityToolMessage] = useState<string | null>(null);
  const [curveBusy, setCurveBusy] = useState(false);
  const [curveError, setCurveError] = useState<string | null>(null);
  const [curveReport, setCurveReport] = useState<AnimationCurveResampleReportV1 | null>(null);
  const [layerBusy, setLayerBusy] = useState(false);
  const [layerError, setLayerError] = useState<string | null>(null);
  const [layerReport, setLayerReport] = useState<AnimationLayerBakeReportV1 | null>(null);
  const [operationError, setOperationError] = useState<{
    code: string;
    message: string;
    action: string;
  } | null>(null);
  const [validatingClipId, setValidatingClipId] = useState<string | null>(null);
  const [pendingEditCopies, setPendingEditCopies] = useState<
    readonly AuthoredAnimationClipV1[]
  >([]);
  const validationLockRef = useRef<string | null>(null);
  const saveValidationGenerationRef = useRef(0);
  const qualityAnalysisGenerationRef = useRef(0);
  const analyzeMotionQualityRef = useRef(onAnalyzeMotionQuality);
  const documentRef = useRef(document);
  const authoringRef = useRef(authoring);
  documentRef.current = document;
  authoringRef.current = authoring;
  analyzeMotionQualityRef.current = onAnalyzeMotionQuality;
  const dialogReturnFocusRef = useRef<HTMLElement | null>(null);
  const eventReturnFocusRef = useRef<HTMLElement | null>(null);
  const importReturnFocusRef = useRef<HTMLElement | null>(null);
  const contributionReturnFocusRef = useRef<HTMLElement | null>(null);
  const preparedAnimationImportRef = useRef<{
    clipId: string;
    documentSourceRevision: string;
    documentAuthoringRevision: number;
    donorSourceRevision: string;
    clipName: string;
    mode: AnimationTransferModeV1;
  } | null>(null);
  const preparedAnimationBatchRef = useRef<{
    documentSourceRevision: string;
    documentAuthoringRevision: number;
    donorSourceRevision: string;
    clipNames: readonly string[];
    mode: AnimationTransferModeV1;
    fingerprintSha256: string;
  } | null>(null);
  const prefersReducedMotion = usePrefersReducedMotionV1();
  const playbackClip = workbenchPreviewClip ?? sequencePreview?.previewClip ?? selectedClip;

  useEffect(() => {
    if (!playbackClip) setPlaying(false);
    if (playbackClip && playheadSeconds > playbackClip.lengthSeconds) {
      setPlayheadSeconds(playbackClip.lengthSeconds);
    }
  }, [playbackClip, playheadSeconds]);

  useEffect(() => {
    setSequencePreview(null);
    setSequenceError(null);
  }, [document.authoringRevision, document.sourceRevision]);

  useEffect(() => {
    if (prefersReducedMotion) setPlaying(false);
  }, [prefersReducedMotion]);

  useEffect(() => {
    if (rig.length === 0) return;
    const animationRoot = selectedClip
      ? rig.find(({ name }) => name === selectedClip.animationRoot)
      : undefined;
    setQualityRootNodeId((current) => (
      animationRoot?.id
      ?? (rig.some(({ id }) => id === current)
        ? current
        : rig.find(({ parentId }) => parentId === null)?.id ?? rig[0].id)
    ));
    setQualityContactNodeIds((current) => {
      const retained = new Set([...current].filter((id) => rig.some((node) => node.id === id)));
      if (retained.size > 0) return retained;
      return new Set(rig
        .filter(({ name }) => /(?:foot|toe|ankle)/iu.test(name))
        .map(({ id }) => id));
    });
  }, [rig, selectedClip?.animationRoot, selectedClip?.id]);

  useEffect(() => {
    const analyze = analyzeMotionQualityRef.current;
    if (!selectedClip || !analyze || !rig.some(({ id }) => id === qualityRootNodeId)) {
      setQualityLoading(false);
      setQualityReport(null);
      setQualityError(null);
      return;
    }
    const generation = qualityAnalysisGenerationRef.current + 1;
    qualityAnalysisGenerationRef.current = generation;
    const clipId = selectedClip.id;
    const clipRevision = selectedClip.revision;
    const sourceRevision = document.sourceRevision;
    setQualityLoading(true);
    setQualityError(null);
    const timeout = globalThis.setTimeout(() => {
      void analyze(
        selectedClip,
        { ...DEFAULT_ANIMATION_QUALITY_POLICY_V1, loopExpected: qualityLoopExpected },
        {
          rootNodeId: qualityRootNodeId,
          contactNodeIds: [...qualityContactNodeIds].sort((left, right) => left - right),
          groundAxis: 2,
          groundHeight: 0,
        },
      ).then((report) => {
        if (qualityAnalysisGenerationRef.current !== generation) return;
        const current = documentRef.current;
        const currentClip = current.authoredClips.find(({ id }) => id === clipId);
        if (
          current.sourceRevision !== sourceRevision
          || currentClip?.revision !== clipRevision
        ) return;
        setQualityReport(report);
        setQualityLoading(false);
      }).catch((error: unknown) => {
        if (qualityAnalysisGenerationRef.current !== generation) return;
        setQualityReport(null);
        setQualityLoading(false);
        setQualityError(error instanceof Error ? error.message : String(error));
      });
    }, 180);
    return () => globalThis.clearTimeout(timeout);
  }, [
    document.sourceRevision,
    qualityContactNodeIds,
    qualityLoopExpected,
    qualityRootNodeId,
    rig,
    selectedClip,
  ]);

  const handleViewportPlaybackUpdate = useCallback((
    snapshot: AnimationViewportPlaybackUpdateV1,
  ) => {
    if (!playbackClip) return;
    const timeSeconds = Math.min(
      Math.max(snapshot.timeSeconds, 0),
      playbackClip.lengthSeconds,
    );
    setPlayheadSeconds((current) => (
      Math.abs(current - timeSeconds) < 1e-4 ? current : timeSeconds
    ));
    if (!snapshot.playing) setPlaying(false);
  }, [playbackClip]);

  const viewportPlayback: AnimationViewportPlaybackControlV1 = {
    playing,
    onUpdate: handleViewportPlaybackUpdate,
  };

  const selectedBone = rig.find(({ id }) => id === selectedBoneId) ?? null;
  const selectedEvent = selectedClip?.events.find(({ id }) => id === selectedEventId)
    ?? null;
  const selectedTransformTrack = selectedClip?.tracks.find((track) => (
    track.targetNodeId === selectedBoneId && track.path === path
  ));
  const selectedTransformKeys = selectedTransformTrack?.keyframes.filter(
    (keyframe) => selectedKeyIds.has(animationKeySelectionIdV1(
      selectedTransformTrack.id,
      keyframe.id,
    )),
  ) ?? [];
  const selectedTransformKey = selectedTransformKeys.find(
    ({ timeSeconds }) => Math.abs(timeSeconds - playheadSeconds) < 1e-7,
  ) ?? (selectedTransformKeys.length === 1 ? selectedTransformKeys[0] : undefined);
  const inspectorCurrentValue = selectedTransformKey?.value
    ?? (selectedTransformTrack?.keyframes.length
      ? sampleAnimationTrackLinearV1(selectedTransformTrack, playheadSeconds)
      : undefined);
  const inspectorCurrentValueSource = selectedTransformKey
    ? "SELECTED_KEY" as const
    : selectedTransformTrack?.keyframes.length
      ? "PLAYHEAD" as const
      : "DEFAULT" as const;
  const editingLocked = validatingClipId !== null;
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
    preparedAnimationImportRef.current = null;
    setImportDialogOpen(false);
    queueMicrotask(() => importReturnFocusRef.current?.focus());
  };
  const closeContributionDialog = () => {
    setContributionDialogOpen(false);
    queueMicrotask(() => contributionReturnFocusRef.current?.focus());
  };

  const commitDocument = (
    authoredClips: readonly AuthoredAnimationClipV1[],
    status: AnimationStudioDocumentV1["status"] = "DRAFT",
    schemaVersion: AnimationStudioDocumentV1["schemaVersion"] = document.schemaVersion,
  ) => {
    if (validationLockRef.current !== null) return;
    onDocumentChange({
      ...document,
      schemaVersion,
      status,
      authoringRevision: document.authoringRevision + 1,
      authoredClips: [...authoredClips],
    });
  };
  const commitClip = (clip: AuthoredAnimationClipV1) => {
    commitDocument(document.authoredClips.map((item) => item.id === clip.id ? clip : item));
    selectClip(clip.id);
  };
  const applyCoreCommandBatch = async (
    commands: readonly AnimationEditCommandV1[],
  ) => {
    if (!onApplyAnimationEditCommandBatch || validationLockRef.current !== null) return false;
    const sourceDocument = documentRef.current;
    const commandId = uniqueId("animation-command");
    setOperationError(null);
    try {
      const result = await onApplyAnimationEditCommandBatch(sourceDocument, {
        schemaVersion: 1,
        commandId,
        context: {
          sourceRevision: sourceDocument.sourceRevision,
          documentAuthoringRevision: sourceDocument.authoringRevision,
        },
        commands,
      });
      const current = documentRef.current;
      if (
        current.sourceRevision !== sourceDocument.sourceRevision
        || current.authoringRevision !== sourceDocument.authoringRevision
      ) {
        setOperationError({
          code: "M2A-ANIMATION-EDIT-COMMAND-STALE",
          message: "The animation changed while the exact core command was running.",
          action: "Review the current pose and repeat the transform.",
        });
        return false;
      }
      documentRef.current = result.document;
      onDocumentChange(result.document);
      return true;
    } catch (error: unknown) {
      setOperationError({
        code: "M2A-ANIMATION-EDIT-COMMAND",
        message: error instanceof Error ? error.message : String(error),
        action: "Retry the edit against the current source and document revision.",
      });
      return false;
    }
  };

  useEffect(() => {
    if (validatingClipId !== null || pendingEditCopies.length === 0) return;
    let latestDocument = documentRef.current;
    let selectedImportedId: string | null = null;
    for (const resolvedClip of pendingEditCopies) {
      if (resolvedClip.source.sourceRevision !== latestDocument.sourceRevision) {
        setOperationError({
          code: "M2A-ANIMATION-EDIT-SOURCE-REVISION-STALE",
          message:
            "The source model changed before Edit copy finished, so the resolved clip was not added.",
          action: "Retry Edit copy against the current source model.",
        });
        continue;
      }
      const hasIdCollision = latestDocument.authoredClips.some(
        ({ id }) => id === resolvedClip.id,
      );
      const hasNameCollision = latestDocument.authoredClips.some(
        ({ name }) => name === resolvedClip.name,
      );
      const clip = hasIdCollision || hasNameCollision
        ? cloneSourceClipForEditingV1(resolvedClip, {
            id: uniqueId("authored"),
            name: uniqueOutputName(
              `${resolvedClip.name}_copy`,
              latestDocument.authoredClips,
            ),
          })
        : resolvedClip;
      latestDocument = {
        ...latestDocument,
        status: "DRAFT",
        authoringRevision: latestDocument.authoringRevision + 1,
        authoredClips: [...latestDocument.authoredClips, clip],
      };
      selectedImportedId = clip.id;
    }
    setPendingEditCopies([]);
    if (!selectedImportedId) return;
    documentRef.current = latestDocument;
    onDocumentChange(latestDocument);
    selectClip(selectedImportedId);
  }, [onDocumentChange, pendingEditCopies, validatingClipId]);

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

  const editCopy = (libraryId: string) => {
    const sourceId = libraryId.replace(/^source:/, "");
    const source = sourceInventory.find(({ clipId }) => clipId === sourceId);
    if (!source) return;
    const currentDocument = documentRef.current;
    const id = uniqueId("authored");
    const name = uniqueOutputName(
      `${source.name}_edited`,
      currentDocument.authoredClips,
    );
    if (!onEditSourceClip) {
      const placeholder = createBlankPoseClipV1({
        id,
        name,
        sourceRevision: currentDocument.sourceRevision,
        animationRoot: rig.find(({ parentId }) => parentId === null)?.name ?? "root",
        lengthSeconds: Math.max(source.durationSeconds, 0.001),
        rig,
      });
      commitDocument([
        ...currentDocument.authoredClips,
        {
          ...placeholder,
          source: {
            kind: "SOURCE_CLIP_COPY",
            sourceRevision: currentDocument.sourceRevision,
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
      setPendingEditCopies((current) => [...current, clip]);
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

  const useLibraryPreset = async (preset: AnimationPresetCatalogEntryV1) => {
    if (!onInstantiateLibraryPreset) {
      throw new Error("The exact Animation Studio library boundary is unavailable.");
    }
    const requestedDocument = documentRef.current;
    const requestedSourceRevision = requestedDocument.sourceRevision;
    const requestedAuthoringRevision = requestedDocument.authoringRevision;
    const newId = uniqueId(`library-${preset.presetId}`);
    const newName = uniqueOutputName(
      preset.outputName,
      requestedDocument.authoredClips,
    );
    const clip = await onInstantiateLibraryPreset(preset, newId, newName);
    if (clip.id !== newId || clip.name !== newName || clip.status !== "DRAFT") {
      throw new Error("Core returned an unexpected library preset instance identity.");
    }
    const latestDocument = documentRef.current;
    if (
      latestDocument.sourceRevision !== requestedSourceRevision ||
      latestDocument.authoringRevision !== requestedAuthoringRevision
    ) {
      throw new Error(
        "The Animation Studio document changed while the library preset was loading. Try again.",
      );
    }
    const migrated = migrateAnimationStudioDocumentV1ToV2(latestDocument);
    commitDocument([...migrated.authoredClips, clip], "DRAFT", migrated.schemaVersion);
    selectClip(clip.id);
    setPlayheadSeconds(0);
    setPlaying(false);
  };

  const prepareAnimationModelClip = async (
    file: File,
    clipName: string,
    donorSourceRevision: string,
    mode: AnimationTransferModeV1,
    semanticMap?: HumanoidSemanticBoneMapV2,
  ) => {
    if (validationLockRef.current !== null) {
      throw new Error("Wait for exact clip validation to finish before importing.");
    }
    if (!onImportAnimationModelClip) {
      throw new Error("Animation import is unavailable in this build.");
    }
    const currentDocument = documentRef.current;
    const requestedDocumentSourceRevision = currentDocument.sourceRevision;
    const requestedDocumentAuthoringRevision = currentDocument.authoringRevision;
    const id = uniqueId("authored");
    const name = uniquePortableImportedName(
      clipName,
      currentDocument.authoredClips,
    );
    const imported = await onImportAnimationModelClip(
      file,
      clipName,
      donorSourceRevision,
      mode,
      semanticMap,
      id,
      name,
    );
    if (validationLockRef.current !== null) {
      throw new Error("Wait for exact clip validation to finish before importing.");
    }
    const latestDocument = documentRef.current;
    if (
      latestDocument.sourceRevision !== requestedDocumentSourceRevision
      || latestDocument.authoringRevision !== requestedDocumentAuthoringRevision
    ) {
      throw new Error(
        "The Animation Studio document changed while the donor preview was being prepared. Inspect the donor again.",
      );
    }
    if (
      imported.source.kind !== (
        mode !== "EXACT_RIG_COPY_V1"
          ? "RETARGETED_MODEL_COPY"
          : "IMPORTED_MODEL_COPY"
      )
      || imported.source.sourceRevision !== donorSourceRevision
      || imported.source.sourceClipName !== clipName
      || (
        mode !== "EXACT_RIG_COPY_V1"
        && imported.source.retarget?.mode !== mode
      )
    ) {
      throw new Error(
        "The imported clip does not match the exact inspected donor lineage.",
      );
    }
    preparedAnimationImportRef.current = {
      clipId: imported.id,
      documentSourceRevision: requestedDocumentSourceRevision,
      documentAuthoringRevision: requestedDocumentAuthoringRevision,
      donorSourceRevision,
      clipName,
      mode,
    };
    return imported;
  };

  const commitPreparedAnimationModelClip = (
    imported: AuthoredAnimationClipV1,
    clipName: string,
    donorSourceRevision: string,
    mode: AnimationTransferModeV1,
    semanticMap?: HumanoidSemanticBoneMapV2,
  ) => {
    const latestDocument = documentRef.current;
    const prepared = preparedAnimationImportRef.current;
    if (
      !prepared
      || prepared.clipId !== imported.id
      || prepared.documentSourceRevision !== latestDocument.sourceRevision
      || prepared.documentAuthoringRevision !== latestDocument.authoringRevision
      || prepared.donorSourceRevision !== donorSourceRevision
      || prepared.clipName !== clipName
      || prepared.mode !== mode
    ) {
      throw new Error(
        "The target, donor, clip, mode, or document changed after preview. Generate a fresh preview.",
      );
    }
    const hasIdCollision = latestDocument.authoredClips.some(
      ({ id: currentId }) => currentId === imported.id,
    );
    const hasNameCollision = latestDocument.authoredClips.some(
      ({ name: currentName }) => currentName === imported.name,
    );
    const clip = hasIdCollision || hasNameCollision
      ? {
          ...imported,
          id: uniqueId("authored"),
          name: uniquePortableImportedName(
            clipName,
            latestDocument.authoredClips,
          ),
        }
      : imported;
    const insertionDocument = mode !== "EXACT_RIG_COPY_V1"
      ? migrateAnimationStudioDocumentToV3(latestDocument)
      : latestDocument;
    const nextDocument: AnimationStudioDocumentV1 = {
      ...insertionDocument,
      status: "DRAFT",
      authoringRevision: insertionDocument.authoringRevision + 1,
      authoredClips: [...insertionDocument.authoredClips, clip],
    };
    documentRef.current = nextDocument;
    onDocumentChange(nextDocument);
    if (
      semanticMap
      && semanticMap.status === "COMPATIBLE"
      && animationWorkbenchProjectState
      && onAnimationWorkbenchProjectStateChange
    ) {
      onAnimationWorkbenchProjectStateChange({
        ...animationWorkbenchProjectState,
        sourceRevision: latestDocument.sourceRevision,
        semanticMaps: [
          ...animationWorkbenchProjectState.semanticMaps.filter(({ id }) => (
            id !== `semantic:${semanticMap.donorSourceRevision}:${semanticMap.targetSourceRevision}`
          )),
          {
            id: `semantic:${semanticMap.donorSourceRevision}:${semanticMap.targetSourceRevision}`,
            name: `Semantic map ${semanticMap.donorSourceRevision.slice(0, 8)} â†’ ${semanticMap.targetSourceRevision.slice(0, 8)}`,
            kind: "SEMANTIC_MAP",
            sourceRevision: latestDocument.sourceRevision,
            fingerprintSha256: semanticMap.fingerprintSha256,
            payload: semanticMap as unknown as Record<string, unknown>,
          },
        ],
      });
    }
    selectClip(clip.id);
    preparedAnimationImportRef.current = null;
  };

  const prepareAnimationModelBatch = async (
    file: File,
    inspection: ExternalAnimationSourceInspectionV1,
    clipNames: readonly string[],
    mode: AnimationTransferModeV1,
    semanticMap?: HumanoidSemanticBoneMapV2,
  ) => {
    if (!onPrepareAnimationModelBatch) throw new Error("Batch transfer is unavailable.");
    const current = documentRef.current;
    const result = await onPrepareAnimationModelBatch(
      file, inspection, clipNames, mode, semanticMap,
    );
    if (result.clips.length !== clipNames.length) {
      throw new Error("Core batch result is incomplete; document remains unchanged.");
    }
    preparedAnimationBatchRef.current = {
      documentSourceRevision: current.sourceRevision,
      documentAuthoringRevision: current.authoringRevision,
      donorSourceRevision: inspection.sourceRevision,
      clipNames: [...clipNames],
      mode,
      fingerprintSha256: result.batchFingerprintSha256,
    };
    return result;
  };

  const commitPreparedAnimationModelBatch = (
    batch: AnimationTransferBatchResultV2,
    donorSourceRevision: string,
    mode: AnimationTransferModeV1,
  ) => {
    const current = documentRef.current;
    const prepared = preparedAnimationBatchRef.current;
    if (!prepared
      || prepared.documentSourceRevision !== current.sourceRevision
      || prepared.documentAuthoringRevision !== current.authoringRevision
      || prepared.donorSourceRevision !== donorSourceRevision
      || prepared.mode !== mode
      || prepared.fingerprintSha256 !== batch.batchFingerprintSha256
      || batch.clips.length !== prepared.clipNames.length) {
      throw new Error("Batch preview is stale; no clips were committed.");
    }
    const existingIds = new Set(current.authoredClips.map(({ id }) => id));
    const existingNames = new Set(current.authoredClips.map(({ name }) => name.toLowerCase()));
    const batchIds = new Set<string>();
    const batchNames = new Set<string>();
    for (const { clip } of batch.clips) {
      if (existingIds.has(clip.id) || existingNames.has(clip.name.toLowerCase())
        || !batchIds.add(clip.id) || !batchNames.add(clip.name.toLowerCase())) {
        throw new Error("Batch output identities collide; zero clips were committed.");
      }
    }
    const insertionDocument = mode === "EXACT_RIG_COPY_V1"
      ? current
      : migrateAnimationStudioDocumentToV3(current);
    const next = {
      ...insertionDocument,
      status: "DRAFT" as const,
      authoringRevision: insertionDocument.authoringRevision + 1,
      authoredClips: [...insertionDocument.authoredClips, ...batch.clips.map(({ clip }) => clip)],
    };
    documentRef.current = next;
    onDocumentChange(next);
    selectClip(batch.clips[0]!.clip.id);
    preparedAnimationBatchRef.current = null;
  };

  const saveClip = () => {
    if (!selectedClip || validationLockRef.current !== null) return;
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
    setPlaying(false);
    setDialog(null);
    setSelectedEventId(null);
    setImportDialogOpen(false);
    setGestureDelta(null);
    validationLockRef.current = candidate.id;
    setValidatingClipId(candidate.id);

    const finishSave = (
      exactDiagnostics: readonly AnimationStudioDiagnosticV1[],
    ) => {
      if (saveValidationGenerationRef.current !== validationGeneration) return;
      validationLockRef.current = null;
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
      const authoredClipIds = new Set(authoredClips.map(({ id }) => id));
      const retainedCustomAnimations = currentAuthoring.customAnimations.filter((custom) => (
        [custom.clipReference, ...custom.phases.map(({ clipReference }) => clipReference)]
          .filter((reference) => reference !== null)
          .every((reference) => (
            reference.sourceKind !== "AUTHORED_CLIP"
            || authoredClipIds.has(reference.authoredClipId ?? "")
          ))
      ));
      const nextCustomAnimations = retainedCustomAnimations.some((custom) => (
        custom.clipReference?.authoredClipId === saved.id
        || custom.phases.some(({ clipReference }) => (
          clipReference.authoredClipId === saved.id
        ))
      ))
        ? retainedCustomAnimations
        : [
            ...retainedCustomAnimations,
            createCustomDefinitionFromAuthoredClipV1(saved.id, {
              id: uniqueId("custom"),
              name: saved.name,
            }),
          ];
      const customAnimationIds = new Set(nextCustomAnimations.map(({ id }) => id));
      const assignments = currentAuthoring.assignments.filter((assignment) => (
        assignment.sourceKind !== "CUSTOM"
        || (
          assignment.customAnimationId !== null
          && customAnimationIds.has(assignment.customAnimationId)
        )
      ));
      if (
        nextCustomAnimations.length !== currentAuthoring.customAnimations.length
        || assignments.length !== currentAuthoring.assignments.length
        || !currentAuthoring.customAnimations.some((custom) => (
          custom.clipReference?.authoredClipId === saved.id
          || custom.phases.some(({ clipReference }) => (
            clipReference.authoredClipId === saved.id
          ))
        ))
      ) {
        onAuthoringChange({
          ...currentAuthoring,
          authoringRevision: currentAuthoring.authoringRevision + 1,
          assignments,
          customAnimations: nextCustomAnimations,
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
    if (!selectedClip || validationLockRef.current !== null) return;
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
    if (onApplyAnimationEditCommandBatch) {
      const existingKey = selectedTransformTrack?.keyframes.find(
        ({ timeSeconds }) => Math.abs(timeSeconds - playheadSeconds) < 1e-7,
      );
      void applyCoreCommandBatch([{
        kind: "SET_BONE_KEY",
        clipId: selectedClip.id,
        trackId: selectedTransformTrack?.id ?? uniqueId("track"),
        keyId: existingKey?.id ?? uniqueId("key"),
        targetNodeId: selectedBoneId,
        path,
        timeSeconds: playheadSeconds,
        value,
      }]);
      return;
    }
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

  const applyQualityTool = async (
    request: AnimationAuthoringToolRequestV1,
    successMessage: string,
  ) => {
    if (!selectedClip || !onApplyAnimationAuthoringTool || qualityToolBusy) return;
    const sourceRevision = document.sourceRevision;
    const sourceDocumentRevision = document.authoringRevision;
    const sourceClipRevision = selectedClip.revision;
    setQualityToolBusy(true);
    setQualityToolMessage(null);
    setOperationError(null);
    try {
      const result = await onApplyAnimationAuthoringTool(selectedClip, request);
      const current = documentRef.current;
      const currentClip = current.authoredClips.find(({ id }) => id === selectedClip.id);
      if (
        current.sourceRevision !== sourceRevision
        || current.authoringRevision !== sourceDocumentRevision
        || currentClip?.revision !== sourceClipRevision
      ) {
        throw new Error("The clip changed while the exact authoring tool was running.");
      }
      const outputClip = animationAuthoringToolResultClipV1(result);
      const nextDocument: AnimationStudioDocumentV1 = {
        ...current,
        status: "DRAFT",
        authoringRevision: current.authoringRevision + 1,
        authoredClips: current.authoredClips.map((clip) => (
          clip.id === outputClip.id ? outputClip : clip
        )),
      };
      documentRef.current = nextDocument;
      onDocumentChange(nextDocument);
      setQualityToolMessage(successMessage);
    } catch (error: unknown) {
      setOperationError({
        code: "M2A-ANIMATION-QUALITY-TOOL",
        message: error instanceof Error ? error.message : String(error),
        action: "Review the current clip and retry the tool with explicit settings.",
      });
    } finally {
      setQualityToolBusy(false);
    }
  };

  const buildSequencePreview = async (clipIds: readonly string[]) => {
    if (!onBuildAnimationSequencePreview || sequenceBusy) return;
    const current = documentRef.current;
    const sourceRevision = current.sourceRevision;
    const authoringRevision = current.authoringRevision;
    setSequenceBusy(true);
    setSequenceError(null);
    try {
      const preview = await onBuildAnimationSequencePreview({
        schemaVersion: 1,
        sourceRevision,
        sequenceId: `sequence-${clipIds.join("-")}`,
        outputName: "preview_idle_action_idle",
        clipIds,
      }, current.authoredClips);
      const latest = documentRef.current;
      if (
        latest.sourceRevision !== sourceRevision
        || latest.authoringRevision !== authoringRevision
        || preview.sourceRevision !== sourceRevision
        || preview.segments.map(({ clipId }) => clipId).join("\u0000")
          !== clipIds.join("\u0000")
      ) {
        throw new Error(
          "The project or sequence changed while Core was preparing the preview.",
        );
      }
      setSequencePreview(preview);
      setPlayheadSeconds(0);
      setPlaying(!prefersReducedMotion);
    } catch (error: unknown) {
      setSequencePreview(null);
      setPlaying(false);
      setSequenceError(error instanceof Error ? error.message : String(error));
    } finally {
      setSequenceBusy(false);
    }
  };

  const smoothSelectedTrack = async (
    curve: AnimationEditorCurveTrackV1,
    tolerance: number,
  ) => {
    if (
      !selectedClip
      || !selectedTransformTrack
      || !onResampleAnimationCurve
      || curveBusy
    ) return;
    const sourceRevision = document.sourceRevision;
    const authoringRevision = document.authoringRevision;
    const clipRevision = selectedClip.revision;
    const trackId = selectedTransformTrack.id;
    setCurveBusy(true);
    setCurveError(null);
    setCurveReport(null);
    try {
      const report = await onResampleAnimationCurve(
        curve,
        {
          schemaVersion: 1,
          maxPositionError: tolerance,
          maxAngularErrorRadians: tolerance,
          maxSubdivisionDepth: 12,
        },
      );
      const current = documentRef.current;
      const currentClip = current.authoredClips.find(({ id }) => id === selectedClip.id);
      if (
        current.sourceRevision !== sourceRevision
        || current.authoringRevision !== authoringRevision
        || currentClip?.revision !== clipRevision
        || report.track.id !== trackId
        || report.track.targetNodeId !== selectedTransformTrack.targetNodeId
        || report.track.path !== selectedTransformTrack.path
      ) {
        throw new Error("The selected track changed while Core was resampling its curve.");
      }
      const nextClip: AuthoredAnimationClipV1 = {
        ...currentClip,
        status: "DRAFT",
        revision: currentClip.revision + 1,
        tracks: currentClip.tracks.map((track) => (
          track.id === trackId ? report.track : track
        )),
      };
      const nextDocument: AnimationStudioDocumentV1 = {
        ...current,
        status: "DRAFT",
        authoringRevision: current.authoringRevision + 1,
        authoredClips: current.authoredClips.map((clip) => (
          clip.id === nextClip.id ? nextClip : clip
        )),
      };
      documentRef.current = nextDocument;
      onDocumentChange(nextDocument);
      setCurveReport(report);
    } catch (error: unknown) {
      setCurveError(error instanceof Error ? error.message : String(error));
    } finally {
      setCurveBusy(false);
    }
  };

  const bakeCorrectionLayer = async (input: {
    readonly layerClipId: string;
    readonly mode: "ADDITIVE" | "OVERRIDE";
    readonly weight: number;
    readonly selectedBoneOnly: boolean;
  }) => {
    if (!selectedClip || !onBakeAnimationLayers || layerBusy) return;
    const sourceRevision = document.sourceRevision;
    const authoringRevision = document.authoringRevision;
    const clipRevision = selectedClip.revision;
    const overlay = document.authoredClips.find(({ id }) => id === input.layerClipId);
    if (!overlay) return;
    const nodeIds = input.selectedBoneOnly && selectedBoneId !== null ? [selectedBoneId] : [];
    const layers: readonly AnimationEditLayerV1[] = [{
      schemaVersion: 1,
      id: `base-${selectedClip.id}`,
      stableOrder: 0,
      mode: "BASE",
      weight: 1,
      mute: false,
      solo: false,
      boneMask: { schemaVersion: 1, nodeIds: [] },
      clip: selectedClip,
    }, {
      schemaVersion: 1,
      id: `correction-${overlay.id}`,
      stableOrder: 1,
      mode: input.mode,
      weight: input.weight,
      mute: false,
      solo: false,
      boneMask: { schemaVersion: 1, nodeIds },
      clip: overlay,
    }];
    const rigWire: AnimationStudioRigWireV1 = {
      schemaVersion: 1,
      sourceRevision,
      animationRoot: selectedClip.animationRoot,
      nodes: rig.map((node) => ({
        nodeId: node.id,
        name: node.name,
        parentId: node.parentId,
        translation: node.translation,
        rotation: node.rotation,
      })),
    };
    setLayerBusy(true);
    setLayerError(null);
    setLayerReport(null);
    try {
      const report = await onBakeAnimationLayers(layers, rigWire);
      const current = documentRef.current;
      const currentClip = current.authoredClips.find(({ id }) => id === selectedClip.id);
      if (
        current.sourceRevision !== sourceRevision
        || current.authoringRevision !== authoringRevision
        || currentClip?.revision !== clipRevision
        || report.clip.id !== selectedClip.id
      ) {
        throw new Error("The layer stack changed while Core was baking it.");
      }
      const nextDocument: AnimationStudioDocumentV1 = {
        ...current,
        status: "DRAFT",
        authoringRevision: current.authoringRevision + 1,
        authoredClips: current.authoredClips.map((clip) => (
          clip.id === report.clip.id ? report.clip : clip
        )),
      };
      documentRef.current = nextDocument;
      onDocumentChange(nextDocument);
      setLayerReport(report);
    } catch (error: unknown) {
      setLayerError(error instanceof Error ? error.message : String(error));
    } finally {
      setLayerBusy(false);
    }
  };

  const eventDiagnostics = selectedClip && selectedEvent
    ? validateAnimationEventDraftV1(selectedEvent, selectedClip)
    : [];
  const viewportTransform: AnimationViewportTransformControlV1 = {
    selectedBone,
    path,
    space: gizmoSpace,
    enabled: Boolean(selectedClip) && !editingLocked && !sequencePreview && !workbenchPreviewClip,
    onBegin: () => setGizmoPreviewValue(inspectorCurrentValue ?? null),
    onPreview: setGizmoPreviewValue,
    onCommit: (value) => {
      setGizmoPreviewValue(null);
      insertTransformKey([...value]);
    },
    heldWeapon: heldEquipment,
    motionVisualization,
  };
  const jumpToQualityIssue = (issue: AnimationQualityIssueV1) => {
    setPlaying(false);
    setPlayheadSeconds(Math.max(0, Math.min(
      issue.startSeconds,
      selectedClip?.lengthSeconds ?? issue.startSeconds,
    )));
    if (issue.nodeId !== null && rig.some(({ id }) => id === issue.nodeId)) {
      setSelectedBoneId(issue.nodeId);
    }
  };

  return (
    <section
      className="animation-studio-workspace"
      aria-labelledby="animation-studio-title"
      data-layout="aurora-animation-workbench"
      data-reduced-motion={prefersReducedMotion}
      aria-busy={editingLocked}
      onKeyDown={(event) => {
        if (editingLocked) return;
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
            disabled={!canUndo || editingLocked}
            onClick={onUndo}
          >
            Undo
          </button>
          <button
            className="animation-studio-action animation-studio-action--quiet"
            type="button"
            data-action="redo"
            disabled={!canRedo || editingLocked}
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
        <button
          className="animation-studio-action animation-studio-action--quiet"
          type="button"
          data-action="export-contribution"
          disabled={
            !selectedClip
            || selectedClip.status !== "VALID"
            || !onExportAnimationContribution
          }
          onClick={(event) => {
            contributionReturnFocusRef.current = event.currentTarget;
            setContributionDialogOpen(true);
          }}
        >
          Export contribution
        </button>
        </div>
      </header>
      {prefersReducedMotion ? (
        <p className="sr-only" role="status">
          Reduced motion is active. Timeline playback remains user-controlled.
        </p>
      ) : null}
      {editingLocked ? (
        <p className="animation-studio-workspace__validation-lock" role="status">
          Exact core validation is running. Editing is temporarily locked so
          the validated revision cannot change in flight.
        </p>
      ) : null}
      {pendingEditCopies.length > 0 ? (
        <p className="animation-studio-workspace__pending-copy" role="status">
          Edit copy is ready and will be added after exact validation finishes.
        </p>
      ) : null}

      <fieldset
        className="animation-studio-workspace__main"
        disabled={editingLocked}
        inert={editingLocked}
      >
        <AnimationClipLibrary
          items={items}
          selectedId={selectedId}
          onSelect={selectClip}
          onCreateBlank={createBlank}
          onImportFromModel={
            onInspectAnimationModel && onImportAnimationModelClip
              ? openImportDialog
              : undefined
          }
          onEditCopy={editCopy}
          onDuplicate={duplicate}
          invalidRepairActions={invalidRepairActions}
          onInspectPreset={onInspectLibraryPreset}
          onUsePreset={useLibraryPreset}
        />
        <section
          className="animation-studio-workspace__center"
          aria-label="Animation viewport and timeline"
          data-panel="viewport-timeline"
        >
          <AnimationEditorViewport
            viewport={typeof viewport === "function"
              ? viewport(playbackClip, playheadSeconds, viewportPlayback, viewportTransform)
              : viewport}
            sourceViewport={typeof sourceViewport === "function"
              ? sourceViewport(selectedClip, playheadSeconds, viewportPlayback)
              : sourceViewport}
            selectedBoneName={selectedBone?.name ?? null}
            playheadSeconds={playheadSeconds}
            path={path}
            gizmoSpace={gizmoSpace}
            onGizmoSpaceChange={setGizmoSpace}
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
            currentValue={gizmoPreviewValue ?? inspectorCurrentValue}
            currentValueSource={inspectorCurrentValueSource}
            playheadSeconds={playheadSeconds}
            onPathChange={setPath}
            onInsert={insertTransformKey}
            onDeleteSelectedKeys={() => {
              if (!selectedClip || selectedKeyIds.size === 0) return;
              commitClip(deleteSelectedKeysV1(selectedClip, [...selectedKeyIds]));
              setSelectedKeyIds(new Set());
            }}
          />
          {onResampleAnimationCurve ? (
            <AnimationCurvePanel
              track={selectedTransformTrack ?? null}
              busy={curveBusy}
              error={curveError}
              report={curveReport}
              onSmooth={(curve, tolerance) => void smoothSelectedTrack(curve, tolerance)}
            />
          ) : null}
          {onBakeAnimationLayers ? (
            <AnimationLayerPanel
              clips={document.authoredClips}
              selectedClip={selectedClip}
              selectedBoneId={selectedBoneId}
              busy={layerBusy}
              error={layerError}
              report={layerReport}
              onBake={(input) => void bakeCorrectionLayer(input)}
            />
          ) : null}
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
          <HeldEquipmentPanel
            rig={rig}
            value={heldEquipment}
            onChange={(value) => {
              setHeldEquipment(value);
              onHeldEquipmentRuntimeChange?.(value);
            }}
            onInspect={onInspectHeldWeapon}
            persisted={animationWorkbenchProjectState?.heldWeapon}
            onPersistedChange={animationWorkbenchProjectState
              && onAnimationWorkbenchProjectStateChange
              ? (heldWeapon) => onAnimationWorkbenchProjectStateChange({
                  ...animationWorkbenchProjectState,
                  sourceRevision: document.sourceRevision,
                  heldWeapon,
                })
              : undefined}
          />
          {onBuildAnimationSequencePreview ? (
            <AnimationSequencePanel
              clips={document.authoredClips}
              selectedClipId={selectedClip?.id ?? null}
              preview={sequencePreview}
              busy={sequenceBusy}
              error={sequenceError}
              onBuild={(clipIds) => void buildSequencePreview(clipIds)}
              onClear={() => {
                setSequencePreview(null);
                setPlayheadSeconds(0);
                setPlaying(false);
              }}
            />
          ) : null}
          {selectedClip && onApplyAnimationWorkbenchOperation
            && animationWorkbenchProjectState
            && onAnimationWorkbenchProjectStateChange ? (
            <AnimationWorkbenchPanel
              sourceRevision={document.sourceRevision}
              clip={selectedClip}
              clips={document.authoredClips}
              rig={rig}
              playheadSeconds={playheadSeconds}
              onOperation={onApplyAnimationWorkbenchOperation}
              onReplaceClip={commitClip}
              onAddClip={(clip) => {
                const current = documentRef.current;
                if (current.authoredClips.some(({ id }) => id === clip.id)) {
                  setOperationError({
                    code: "M2A-ANIM-WORKBENCH-ID",
                    message: `Clip id ${clip.id} already exists.`,
                    action: "Choose another output id before saving the result.",
                  });
                  return;
                }
                const next = {
                  ...current,
                  status: "DRAFT" as const,
                  authoringRevision: current.authoringRevision + 1,
                  authoredClips: [...current.authoredClips, clip],
                };
                documentRef.current = next;
                onDocumentChange(next);
                setSelectedId(clip.id);
                onSelectedClipChange?.(clip.id);
                setWorkbenchPreviewClip(null);
              }}
              onPreviewClip={(candidate) => {
                setWorkbenchPreviewClip(candidate);
                setSequencePreview(null);
                setPlayheadSeconds(0);
                setPlaying(Boolean(candidate) && !prefersReducedMotion);
              }}
              onVisualization={setMotionVisualization}
              projectState={animationWorkbenchProjectState}
              onProjectStateChange={onAnimationWorkbenchProjectStateChange}
            />
          ) : null}
          {selectedClip && onAnalyzeMotionQuality ? (
            <AnimationQualityPanel
              rig={rig}
              report={qualityReport}
              loading={qualityLoading}
              error={qualityError}
              loopExpected={qualityLoopExpected}
              rootNodeId={qualityRootNodeId}
              contactNodeIds={qualityContactNodeIds}
              onLoopExpectedChange={setQualityLoopExpected}
              onRootNodeChange={setQualityRootNodeId}
              onContactNodeToggle={(nodeId) => setQualityContactNodeIds((current) => {
                const next = new Set(current);
                if (next.has(nodeId)) next.delete(nodeId);
                else next.add(nodeId);
                return next;
              })}
              onJumpToIssue={jumpToQualityIssue}
              toolBusy={qualityToolBusy}
              toolMessage={qualityToolMessage}
              onBlendLoop={onApplyAnimationAuthoringTool
                ? (blendWindowSeconds) => void applyQualityTool({
                    kind: "BLEND_LOOP",
                    policy: { schemaVersion: 1, blendWindowSeconds },
                  }, "Loop seam blended. Review the before/after quality report.")
                : undefined}
              onRootMotion={onApplyAnimationAuthoringTool
                ? (policy) => void applyQualityTool({
                    kind: "TRANSFORM_ROOT_MOTION",
                    policy: policy === "IN_PLACE"
                      ? { kind: "IN_PLACE" }
                      : { kind: "SCALE", factor: policy.scale },
                  }, policy === "IN_PLACE"
                    ? "Root motion converted to in-place."
                    : "Root motion scaled with a reversible Core transform.")
                : undefined}
              onLockContacts={onApplyAnimationAuthoringTool
                ? () => void applyQualityTool({
                    kind: "LOCK_SUGGESTED_CONTACTS",
                    contactSet: {
                      schemaVersion: 1,
                      nodeIds: [...qualityContactNodeIds].sort((left, right) => left - right),
                      groundAxis: 2,
                      groundHeight: 0,
                      contactHeight: (qualityReport?.skeletonHeight ?? 1) * 0.03,
                      maxContactSpeed: (qualityReport?.skeletonHeight ?? 1) * 0.15,
                      sampleRateHz: 30,
                    },
                  }, "Suggested contact intervals were baked to linear tracks.")
                : undefined}
              onReduceKeys={onApplyAnimationAuthoringTool
                ? (maxPositionError, maxAngularErrorRadians) => void applyQualityTool({
                    kind: "REDUCE_KEYS",
                    policy: {
                      schemaVersion: 1,
                      maxPositionError,
                      maxAngularErrorRadians,
                    },
                  }, "Redundant keys removed within the selected tolerances.")
                : undefined}
            />
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
            onSelectKey={(id, additive) => {
              const keyTime = selectedClip.tracks
                .flatMap((track) => track.keyframes.map((keyframe) => ({
                  id: animationKeySelectionIdV1(track.id, keyframe.id),
                  timeSeconds: keyframe.timeSeconds,
                })))
                .find((keyframe) => keyframe.id === id)?.timeSeconds;
              if (keyTime !== undefined) setPlayheadSeconds(keyTime);
              setSelectedKeyIds((current) => {
                if (!additive) return new Set([id]);
                const next = new Set(current);
                if (next.has(id)) next.delete(id);
                else next.add(id);
                return next;
              });
            }}
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
      </fieldset>

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
          availableDonors={animationModelDonors}
          targetRig={rig}
          onInspect={onInspectAnimationModel}
          onPrepare={prepareAnimationModelClip}
          onPrepareBatch={onPrepareAnimationModelBatch ? prepareAnimationModelBatch : undefined}
          onCommit={commitPreparedAnimationModelClip}
          onCommitBatch={onPrepareAnimationModelBatch ? commitPreparedAnimationModelBatch : undefined}
          renderPreview={renderAnimationTransferPreview}
          onClose={closeImportDialog}
        />
      ) : null}
      {contributionDialogOpen && selectedClip && onExportAnimationContribution ? (
        <AnimationContributionDialog
          clip={selectedClip}
          onExport={onExportAnimationContribution}
          onClose={closeContributionDialog}
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
