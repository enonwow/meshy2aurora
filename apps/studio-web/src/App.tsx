import {
  lazy,
  Suspense,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import {
  StudioHeader,
  type StudioProjectPersistence,
} from "./app/StudioHeader";
import { StudioShell } from "./app/StudioShell";
import {
  canContinueFromAnimationMapping,
  getUnlockedWorkflowSteps,
  getWorkflowStepStatus,
} from "./app/studioSelectors";
import {
  type StudioSessionState,
  type StudioTarget,
} from "./app/studioSession";
import {
  useStudioSessionController,
  useStudioWorkerController,
} from "./app/useStudioControllers";
import {
  getWorkflowStepsForTarget,
  workflowStepLabel,
  type WorkflowStep,
} from "./app/workflow";
import { WorkflowStepper } from "./app/WorkflowStepper";
import { BuildStep, type BuildStepState } from "./features/build/BuildStep";
import { DownloadStep } from "./features/downloads/DownloadStep";
import {
  createDownloadManifestV1,
  projectDownloadReadinessV1,
  type DownloadManifestInputIdentityV1,
} from "./features/downloads/downloadManifest";
import {
  projectBuildFailure,
} from "./features/build/projectBuildFailure";
import { createTargetBuildRequestV1 } from "./features/build/targetBuildRequest";
import {
  projectAppearanceInspection,
  type AppearanceInspectionSnapshot,
} from "./features/inspect/appearanceInspection";
import {
  InspectStep,
  type InspectProfileRequirement,
} from "./features/inspect/InspectStep";
import {
  projectSourceInspection,
  type SourceInspectionSnapshot,
} from "./features/inspect/sourceInspection";
import type { InspectValidationCheck } from "./features/inspect/ValidationPanel";
import {
  parsePlaceableAuthoringBootstrap,
  type PlaceableAuthoringBootstrap,
} from "./features/placeable-authoring/types";
import type { BinaryMdlInspectionReport, ModelPartRef } from "./features/preview/types";
import type { ReviewViewport } from "./features/review/ReviewModelDetails";
import { ReviewWorkflowActions } from "./features/review/ReviewWorkflowActions";
import type { CanonicalResultSnapshot } from "./features/results/projectCanonicalResult";
import type { PlaceableResultSnapshot } from "./features/results/projectPlaceableResult";
import type { TileResultSnapshot } from "./features/results/projectTileResult";
import {
  InputsPanel,
  type CreatureConversionProfileV1,
  type SkinAccessoryStabilizationModeV1,
  type TileAuthoringOptions,
} from "./features/source/InputsPanel";
import { SourceStep } from "./features/source/SourceStep";
import {
  canOfferGeneratedHumanoidProfileV1,
  hasFullNativeDirectCreatureProfileV1,
} from "./features/source/directCreatureAnimationProfile";
import { CreatureAnimationMappingStep } from "./features/animation-mapping/CreatureAnimationMappingStep";
import type { AnimationRigNodeV1 } from "./features/animation-editor/AnimationBoneTree";
import {
  parseHeldWeaponSourceInspectionV1,
  type HeldEquipmentPreviewV1,
} from "./features/animation-editor/HeldEquipmentPanel";
import {
  emptyAnimationWorkbenchProjectStateV1,
  parseAnimationWorkbenchResultV1,
} from "./features/animation-editor/animationWorkbench";
import type { AnimationMappingModeV1 } from "./features/animation-editor/AnimationMappingModeSwitch";
import { createBlankPoseClipV1 } from "./features/animation-editor/editing";
import {
  parseAnimationAuthoringToolResultV1,
  parseAnimationEditCommandResultV1,
  parseAnimationQualityReportV1,
} from "./features/animation-authoring-v2/types";
import {
  parseHumanoidSemanticBoneMapV2,
  parseAnimationTransferCompatibilityV1,
  parseAnimationTransferBatchResultV2,
  type AnimationTransferCompatibilityV1,
  type AnimationTransferModeV1,
  type ExternalAnimationSourceInspectionV1,
  type HumanoidSemanticBoneMapV2,
} from "./features/animation-editor/animationImport";
import { parseAnimationSequencePreviewV1 } from "./features/animation-editor/animationSequence";
import { parseAnimationCurveResampleReportV1 } from "./features/animation-editor/animationCurves";
import { parseAnimationLayerBakeReportV1 } from "./features/animation-editor/animationLayers";
import {
  commitAnimationStudioDocumentV1,
  createAnimationStudioStateV1,
  fingerprintAnimationStudioDocumentV1,
  loadAnimationStudioDocumentV1,
  loadCreatureAnimationAuthoringV2DraftV1,
  markAnimationStudioAutosaveFailedV1,
  markAnimationStudioAutosaveSavingV1,
  markAnimationStudioAutosavedV1,
  markAnimationStudioBuildRevisionV1,
  migrateCreatureAnimationAuthoringV1ToV2,
  reconcileAnimationStudioSourceRevisionV1,
  reduceAnimationStudioStateV1,
  saveAnimationStudioDocumentV1,
  saveCreatureAnimationAuthoringV2DraftV1,
  serializeAnimationStudioDocumentV1,
  serializeCreatureAnimationAuthoringV2,
  undoAnimationStudioEditV1,
  redoAnimationStudioEditV1,
  validateAnimationStudioSchemaV1,
  validateCreatureAnimationAuthoringV2,
  type AnimationStudioDiagnosticV1,
  type AnimationStudioDocumentV1,
  type AnimationStudioStateV1,
  type AuthoredAnimationClipV1,
  type CreatureAnimationAuthoringV2,
} from "./features/animation-studio";
import type { AnimationMappingSaveStateV1 } from "./features/animation-mapping/AnimationMappingStatusBar";
import {
  applyHighConfidenceAssignmentsV1,
  proposeCreatureAnimationMappingV1,
} from "./features/animation-mapping/autoMap";
import {
  assertDirectCreatureCatalogParityV1,
  getAuroraAnimationStateCatalogV1,
} from "./features/animation-mapping/catalog";
import {
  loadCreatureAnimationDraftV1,
  saveCreatureAnimationDraftV1,
} from "./features/animation-mapping/persistence";
import {
  getCreatureAnimationMappingStatusV1,
  validateCreatureAnimationAuthoringV1,
} from "./features/animation-mapping/readiness";
import {
  assertCreatureAnimationBuildInputCurrentV1,
  createCreatureAnimationBuildInputV1,
  reconcileAuthoredAndBuiltAnimationsV1,
  type AuthoredBuiltAnimationReconciliationV1,
  type CreatureAnimationBuildInputV1,
} from "./features/animation-mapping/buildIntegration";
import {
  mergeUiAndCoreAnimationDiagnosticsV1,
  projectCoreCreatureAnimationValidationV1,
} from "./features/animation-mapping/coreValidation";
import {
  getAnimationStudioDownloadGateV1,
  reconcileAnimationStudioReadbackV1,
  type AnimationStudioReadbackReconciliationV1,
} from "./features/review/reconcileAnimationStudioReadback";
import {
  AURORA_MODEL_TRIANGLE_BUDGET_V1,
  LocalMeshyBridgeClient,
  type MeshyArtifactProvenance,
  type MeshyBridgeClient,
} from "./features/meshy/bridge";
import { isMeshyLabEnabled } from "./features/meshy/feature";
import { isTileTargetEnabled } from "./features/tile/feature";
import { ProjectManagerDialog } from "./features/project/ProjectManagerDialog";
import {
  createMeshy2AuroraProjectV1,
  deleteProjectV1,
  duplicateMeshy2AuroraProjectV1,
  listProjectRecoveryRecordsV1,
  loadProjectV1,
  parseMeshy2AuroraProjectV1,
  projectBuildIdentityV1,
  projectExportFileNameV1,
  projectFileReferenceV1,
  renameMeshy2AuroraProjectV1,
  reviseMeshy2AuroraProjectV1,
  saveProjectV1,
  serializeMeshy2AuroraProjectV1,
  serializeProjectBuildIdentityV1,
  sameProjectBuildIdentityV1,
  type Meshy2AuroraProjectFilesV1,
  type Meshy2AuroraProjectV1,
  type ProjectBuildIdentityV1,
  type ProjectDatabaseV1,
  type ProjectRecoveryRecordV1,
} from "./features/project";
import { StudioWorkerClient } from "./worker/client";

const AuroraReadbackViewport = lazy(async () => ({
  default: (await import("./features/preview/AuroraReadbackViewport"))
    .AuroraReadbackViewport,
}));
const SourceViewport = lazy(async () => ({
  default: (await import("./features/preview/SourceViewport")).SourceViewport,
}));
const PlaceableAuthoringEditor = lazy(async () => ({
  default: (await import("./features/placeable-authoring/PlaceableAuthoringEditor"))
    .PlaceableAuthoringEditor,
}));
const ReviewModelDetails = lazy(async () => ({
  default: (await import("./features/review/ReviewModelDetails"))
    .ReviewModelDetails,
}));
const PlaceableReview = lazy(async () => ({
  default: (await import("./features/review/PlaceableReview")).PlaceableReview,
}));
const TileReview = lazy(async () => ({
  default: (await import("./features/review/TileReview")).TileReview,
}));
const AnimationStudioWorkspace = lazy(async () => ({
  default: (await import("./features/animation-editor/AnimationStudioWorkspace"))
    .AnimationStudioWorkspace,
}));
const AuthoredAnimationReview = lazy(async () => ({
  default: (await import("./features/review/AuthoredAnimationReview"))
    .AuthoredAnimationReview,
}));
const MeshyLab = lazy(async () => ({
  default: (await import("./features/meshy/MeshyLab")).MeshyLab,
}));

const requestId = () => crypto.randomUUID();

async function fileSha256(file: File): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", await file.arrayBuffer());
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function downloadProjectBackup(project: Meshy2AuroraProjectV1) {
  const url = URL.createObjectURL(new Blob(
    [serializeMeshy2AuroraProjectV1(project)],
    { type: "application/json" },
  ));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = projectExportFileNameV1(project);
  anchor.click();
  URL.revokeObjectURL(url);
}

function projectContentSignature(project: Meshy2AuroraProjectV1): string {
  return JSON.stringify({
    target: project.target,
    files: project.files,
    animationMappingV2: project.animationMappingV2,
    animationStudio: project.animationStudio,
    animationWorkbench: project.animationWorkbench,
    placeableAuthoring: project.placeableAuthoring,
    tileOptions: project.tileOptions,
  });
}

function heldWeaponPrimaryHandV1(targetBoneName: string): "RIGHT" | "LEFT" {
  const normalized = targetBoneName.toLocaleLowerCase("en-US").replaceAll(/[^a-z0-9]/g, "");
  return normalized.includes("left") || normalized.startsWith("lhand") || normalized.endsWith("handl")
    ? "LEFT"
    : "RIGHT";
}

function quaternionFromEulerDegreesV1(
  degrees: readonly [number, number, number],
): readonly [number, number, number, number] {
  const [x, y, z] = degrees.map((value) => value * Math.PI / 360);
  const sx = Math.sin(x);
  const cx = Math.cos(x);
  const sy = Math.sin(y);
  const cy = Math.cos(y);
  const sz = Math.sin(z);
  const cz = Math.cos(z);
  return [
    sx * cy * cz + cx * sy * sz,
    cx * sy * cz - sx * cy * sz,
    cx * cy * sz + sx * sy * cz,
    cx * cy * cz - sx * sy * sz,
  ];
}

interface StudioModelBuildResult {
  readonly kind: "MODEL";
  readonly projectIdentity: ProjectBuildIdentityV1;
  readonly canonical: CanonicalResultSnapshot;
  readonly readback: BinaryMdlInspectionReport;
  readonly readbackJson: string;
  readonly animationBuildInput: CreatureAnimationBuildInputV1;
  readonly animationReconciliation?: AuthoredBuiltAnimationReconciliationV1;
  readonly animationStudioDocument?: AnimationStudioDocumentV1;
  readonly animationAuthoringV2?: CreatureAnimationAuthoringV2;
  readonly animationStudioFingerprintSha256?: string;
  readonly animationStudioReconciliation?: AnimationStudioReadbackReconciliationV1;
}

interface StudioPlaceableBuildResult {
  readonly kind: "PLACEABLE";
  readonly projectIdentity: ProjectBuildIdentityV1;
  readonly placeable: PlaceableResultSnapshot;
  readonly readback: BinaryMdlInspectionReport;
  readonly readbackJson: string;
}

interface StudioTileBuildResult {
  readonly kind: "TILE";
  readonly projectIdentity: ProjectBuildIdentityV1;
  readonly tile: TileResultSnapshot;
  readonly readback: BinaryMdlInspectionReport;
  readonly readbackJson: string;
}

type StudioBuildResult =
  | StudioModelBuildResult
  | StudioPlaceableBuildResult
  | StudioTileBuildResult;

type StudioState = StudioSessionState<
  SourceInspectionSnapshot,
  StudioBuildResult,
  AppearanceInspectionSnapshot
>;

function isGlb(file: File) {
  return file.name.toLowerCase().endsWith(".glb");
}

function isBaseTwoDa(file: File) {
  return ["appearance.2da", "placeables.2da"].includes(file.name.toLowerCase());
}

function isJson(file: File) {
  return file.name.toLowerCase().endsWith(".json");
}

function studioCreatureProductIdentity(
  sourceSha256: string,
  textureArtifactCleanup: boolean,
) {
  if (!/^[0-9a-f]{64}$/.test(sourceSha256)) {
    throw new Error("Inspected source has no canonical SHA-256 identity");
  }
  const suffix = sourceSha256.slice(0, 8);
  const variant = textureArtifactCleanup ? "h" : "";
  return {
    modelResref: `m2c2${variant}m${suffix}`,
    textureResref: `m2c2${variant}t${suffix}`,
    hakResref: `m2c2${variant}h${suffix}`,
    appearanceLabel: `M2A_CREATURE_V2_${textureArtifactCleanup ? "H_" : ""}${suffix.toUpperCase()}`,
  };
}
function studioCreatureP100kPackageIdentity(
  sourceSha256: string,
  textureArtifactCleanup: boolean,
) {
  if (!/^[0-9a-f]{64}$/.test(sourceSha256)) {
    throw new Error("Inspected source has no canonical SHA-256 identity");
  }
  const suffix = sourceSha256.slice(0, 8);
  const variant = textureArtifactCleanup ? "h" : "";
  return {
    modelResref: `m2p1${variant}m${suffix}`,
    textureResref: `m2p1${variant}t${suffix}`,
    module: {
      moduleResref: `m2p1${variant}d${suffix}`,
      areaResref: `m2p1${variant}a${suffix}`,
      hakResref: `m2p1${variant}h${suffix}`,
    },
    creatureResref: `m2p1${variant}c${suffix}`,
  };
}
function studioCreatureP300kPackageIdentity(
  sourceSha256: string,
  textureArtifactCleanup: boolean,
) {
  if (!/^[0-9a-f]{64}$/.test(sourceSha256)) {
    throw new Error("Inspected source has no canonical SHA-256 identity");
  }
  const suffix = sourceSha256.slice(0, 8);
  const variant = textureArtifactCleanup ? "h" : "";
  return {
    modelResref: `m2p3${variant}m${suffix}`,
    textureResref: `m2p3${variant}t${suffix}`,
    module: {
      moduleResref: `m2p3${variant}d${suffix}`,
      areaResref: `m2p3${variant}a${suffix}`,
      hakResref: `m2p3${variant}h${suffix}`,
    },
    creatureResref: `m2p3${variant}c${suffix}`,
  };
}
const DEFAULT_TILE_OPTIONS: TileAuthoringOptions = {
  terrainName: "Grass",
  surface: "GRASS",
  interior: false,
};

function createEmptyAnimationStudioDocumentV1(
  sourceRevision: string,
): AnimationStudioDocumentV1 {
  return {
    schemaVersion: 1,
    sourceRevision,
    authoringRevision: 1,
    status: "DRAFT",
    authoredClips: [],
  };
}

function synchronizeCreatureAnimationAuthoringV2(
  v1: Parameters<typeof migrateCreatureAnimationAuthoringV1ToV2>[0],
  current: CreatureAnimationAuthoringV2 | null,
): CreatureAnimationAuthoringV2 {
  const migrated = migrateCreatureAnimationAuthoringV1ToV2(v1);
  if (!current || current.sourceRevision !== migrated.sourceRevision) return migrated;
  const authoredCustom = current.customAnimations.filter((custom) => (
    custom.clipReference?.sourceKind === "AUTHORED_CLIP"
    || custom.phases.some(({ clipReference }) => (
      clipReference.sourceKind === "AUTHORED_CLIP"
    ))
  ));
  const authoredCustomIds = new Set(authoredCustom.map(({ id }) => id));
  const authoredAssignments = current.assignments.filter((assignment) => (
    assignment.sourceKind === "CUSTOM"
    && assignment.customAnimationId !== null
    && authoredCustomIds.has(assignment.customAnimationId)
  ));
  const authoredSlots = new Set(authoredAssignments.map(({ targetSlot }) => targetSlot));
  return {
    ...migrated,
    authoringRevision: Math.max(
      migrated.authoringRevision,
      current.authoringRevision,
    ),
    assignments: [
      ...migrated.assignments.filter(({ targetSlot }) => !authoredSlots.has(targetSlot)),
      ...authoredAssignments,
    ],
    customAnimations: [
      ...migrated.customAnimations.filter(({ id }) => !authoredCustomIds.has(id)),
      ...authoredCustom,
    ],
  };
}

function parseEditableAnimationSourceV1(inspectionJson: string): {
  sourceRevision: string;
  rig: AnimationRigNodeV1[];
  clips: ExternalAnimationSourceInspectionV1["clips"];
  clip: AuthoredAnimationClipV1 | null;
} {
  const value = JSON.parse(inspectionJson) as {
    sourceRevision?: unknown;
    rig?: unknown;
    clips?: unknown;
    clip?: unknown;
  };
  if (
    typeof value.sourceRevision !== "string"
    || !/^[0-9a-f]{64}$/i.test(value.sourceRevision)
  ) {
    throw new Error("Editable animation source inspection has no SHA-256 revision.");
  }
  if (!Array.isArray(value.rig)) {
    throw new Error("Editable animation source inspection has no output rig.");
  }
  const rig = value.rig.map((item, index): AnimationRigNodeV1 => {
    if (
      item === null
      || typeof item !== "object"
      || !Number.isSafeInteger((item as { id?: unknown }).id)
      || typeof (item as { name?: unknown }).name !== "string"
    ) {
      throw new Error(`Editable animation rig node ${index} is invalid.`);
    }
    const node = item as {
      id: number;
      name: string;
      parentId?: unknown;
      translation?: unknown;
      rotation?: unknown;
    };
    if (
      !Array.isArray(node.translation)
      || node.translation.length !== 3
      || node.translation.some((component) => (
        typeof component !== "number" || !Number.isFinite(component)
      ))
      || !Array.isArray(node.rotation)
      || node.rotation.length !== 4
      || node.rotation.some((component) => (
        typeof component !== "number" || !Number.isFinite(component)
      ))
    ) {
      throw new Error(`Editable animation rig node ${index} has no exact local pose.`);
    }
    if (
      node.parentId !== null
      && (
        !Number.isSafeInteger(node.parentId)
        || Number(node.parentId) < 0
      )
    ) {
      throw new Error(`Editable animation rig node ${index} has an invalid parent.`);
    }
    return {
      id: node.id,
      name: node.name,
      parentId: node.parentId === null ? null : Number(node.parentId),
      translation: node.translation as unknown as [number, number, number],
      rotation: node.rotation as unknown as [number, number, number, number],
    };
  });
  if (!Array.isArray(value.clips)) {
    throw new Error("Editable animation source inspection has no clip inventory.");
  }
  const clips = value.clips.map((item, index) => {
    if (
      item === null
      || typeof item !== "object"
      || typeof (item as { name?: unknown }).name !== "string"
      || typeof (item as { durationSeconds?: unknown }).durationSeconds !== "number"
      || !Number.isFinite((item as { durationSeconds: number }).durationSeconds)
      || !Number.isSafeInteger((item as { trackCount?: unknown }).trackCount)
      || Number((item as { trackCount: number }).trackCount) < 0
    ) {
      throw new Error(`Editable animation clip inventory row ${index} is invalid.`);
    }
    const clip = item as {
      name: string;
      durationSeconds: number;
      trackCount: number;
    };
    return {
      name: clip.name,
      durationSeconds: clip.durationSeconds,
      trackCount: clip.trackCount,
    };
  });
  return {
    sourceRevision: value.sourceRevision,
    rig,
    clips,
    clip: value.clip === null || value.clip === undefined
      ? null
      : value.clip as AuthoredAnimationClipV1,
  };
}

function parseAnimationTransferResultV1(resultJson: string): {
  requiredDocumentSchemaVersion: number;
  compatibility: AnimationTransferCompatibilityV1;
  clip: AuthoredAnimationClipV1;
} {
  const value = JSON.parse(resultJson) as {
    schemaVersion?: unknown;
    requiredDocumentSchemaVersion?: unknown;
    compatibility?: unknown;
    clip?: unknown;
  };
  if (
    value.schemaVersion !== 1
    || !Number.isSafeInteger(value.requiredDocumentSchemaVersion)
    || value.clip === null
    || typeof value.clip !== "object"
  ) {
    throw new Error("Core returned an invalid animation-transfer result.");
  }
  return {
    requiredDocumentSchemaVersion: Number(value.requiredDocumentSchemaVersion),
    compatibility: parseAnimationTransferCompatibilityV1(
      JSON.stringify(value.compatibility),
    ),
    clip: value.clip as AuthoredAnimationClipV1,
  };
}

function sourceValidationChecks(snapshot?: SourceInspectionSnapshot): InspectValidationCheck[] {
  if (!snapshot) {
    return [{
      id: "source-inspection-pending",
      label: "Source inspection",
      status: "UNAVAILABLE",
    }];
  }

  const eligibility: InspectValidationCheck = {
    id: "conversion-eligibility",
    label: "Conversion eligibility",
    status: snapshot.conversionEligible ? "PASS" : "ERROR",
    evidence: {
      code: "report.conversionEligible",
      path: "report.conversionEligible",
      message: snapshot.conversionEligible
        ? "The source inspection reports this model as conversion eligible."
        : "One or more blocking source gates prevent conversion.",
    },
  };
  const gates: InspectValidationCheck[] = snapshot.gates.map((gate, index) => ({
    id: `gate-${gate.code}-${index}`,
    label: gate.code,
    status: gate.severity === "BLOCKING" ? "ERROR" : "WARNING",
    evidence: {
      code: gate.code,
      path: gate.path,
      message: gate.message,
    },
  }));
  const diagnostics: InspectValidationCheck[] = snapshot.diagnostics.map((diagnostic, index) => ({
    id: `diagnostic-${diagnostic.code}-${index}`,
    label: diagnostic.code,
    status: diagnostic.severity === "ERROR" || diagnostic.severity === "BLOCKING" || diagnostic.severity === "FATAL"
      ? "ERROR"
      : diagnostic.severity === "WARNING"
        ? "WARNING"
        : "INFO",
    evidence: {
      code: diagnostic.code,
      path: diagnostic.jsonPath ?? (diagnostic.byteOffset === null ? undefined : `byteOffset:${diagnostic.byteOffset}`),
      message: diagnostic.message,
    },
  }));
  return [eligibility, ...gates, ...diagnostics];
}

function creatureH1ProfileRequirements(
  snapshot?: SourceInspectionSnapshot,
): InspectProfileRequirement[] {
  const pending = "Inspection pending";
  const requirements = [
    {
      id: "h1-source-skin",
      label: "Source skin",
      expected: "exactly 1",
      actual: snapshot ? String(snapshot.inventory.skinCount) : pending,
      pass: snapshot ? snapshot.inventory.skinCount === 1 : null,
      repair: "Re-export one active skin and remove unused skin records.",
    },
    {
      id: "h1-skinned-mesh-node",
      label: "Skinned mesh node",
      expected: "exactly 1 node with both meshId and skinId",
      actual: snapshot ? String(snapshot.skinnedMeshNodeCount) : pending,
      pass: snapshot ? snapshot.skinnedMeshNodeCount === 1 : null,
      repair: "Merge the render surface under one mesh node bound to the selected skin.",
    },
    {
      id: "h1-source-primitive",
      label: "Source primitive",
      expected: "exactly 1",
      actual: snapshot ? String(snapshot.inventory.primitiveCount) : pending,
      pass: snapshot ? snapshot.inventory.primitiveCount === 1 : null,
      repair: "Join the render geometry and export it as one indexed primitive.",
    },
    {
      id: "h1-joints",
      label: "Skin joints",
      expected: "at least 1 unique joint and joint reference",
      actual: snapshot
        ? `${snapshot.boneCount} unique / ${snapshot.inventory.jointReferenceCount} references`
        : pending,
      pass: snapshot
        ? snapshot.boneCount > 0 && snapshot.inventory.jointReferenceCount > 0
        : null,
      repair: "Bind the mesh to a non-empty skeleton and export joint references.",
    },
    {
      id: "h1-triangle-budget",
      label: "Render-model triangle budget",
      expected: `at most ${AURORA_MODEL_TRIANGLE_BUDGET_V1.toLocaleString("en-US")}`,
      actual: snapshot
        ? snapshot.statistics.triangleCount.toLocaleString("en-US")
        : pending,
      pass: snapshot
        ? snapshot.statistics.triangleCount <= AURORA_MODEL_TRIANGLE_BUDGET_V1
        : null,
      repair: `Remesh or decimate to ${AURORA_MODEL_TRIANGLE_BUDGET_V1.toLocaleString("en-US")} triangles or fewer.`,
    },
    {
      id: "h1-normals",
      label: "Vertex normals",
      expected: "present on every primitive",
      actual: snapshot
        ? `${snapshot.statistics.primitivesMissingNormals} primitives missing`
        : pending,
      pass: snapshot ? snapshot.statistics.primitivesMissingNormals === 0 : null,
      repair: "Generate and export normals for the selected primitive.",
    },
    {
      id: "h1-uv0",
      label: "UV0",
      expected: "present on every primitive",
      actual: snapshot
        ? `${snapshot.statistics.primitivesMissingUv0} primitives missing`
        : pending,
      pass: snapshot ? snapshot.statistics.primitivesMissingUv0 === 0 : null,
      repair: "Create a UV0 unwrap and include TEXCOORD_0 in the GLB.",
    },
    {
      id: "h1-topology",
      label: "Primitive topology",
      expected: "triangles only",
      actual: snapshot
        ? `${snapshot.statistics.nonTrianglePrimitives} non-triangle primitives`
        : pending,
      pass: snapshot ? snapshot.statistics.nonTrianglePrimitives === 0 : null,
      repair: "Triangulate the model before exporting the GLB.",
    },
  ] as const;
  return requirements.map(({ pass, ...requirement }) => ({
    ...requirement,
    status: pass === null ? "UNAVAILABLE" : pass ? "PASS" : "ERROR",
  }));
}

function appearanceValidationChecks(snapshot?: AppearanceInspectionSnapshot): InspectValidationCheck[] {
  if (!snapshot) {
    return [{
      id: "appearance-inspection-pending",
      label: "Base appearance.2da schema",
      status: "UNAVAILABLE",
    }];
  }
  const schema: InspectValidationCheck = {
    id: "appearance-schema",
    label: "Base appearance.2da schema",
    status: "PASS",
    evidence: {
      code: `${snapshot.format} ${snapshot.version}`,
      path: "appearance.2da",
      message: `${snapshot.columns.length} columns and ${snapshot.physicalRowCount} physical rows parsed by WASM.`,
    },
  };
  const diagnostics: InspectValidationCheck[] = snapshot.diagnostics.map((diagnostic, index) => ({
    id: `appearance-diagnostic-${diagnostic.code}-${index}`,
    label: diagnostic.code,
    status: diagnostic.severity === "ERROR" || diagnostic.severity === "BLOCKING" || diagnostic.severity === "FATAL"
      ? "ERROR"
      : diagnostic.severity === "WARNING"
        ? "WARNING"
        : "INFO",
    evidence: {
      code: diagnostic.code,
      path: diagnostic.path,
      message: diagnostic.message,
    },
  }));
  return [schema, ...diagnostics];
}

export interface AppProps {
  readonly meshyBridge?: MeshyBridgeClient;
  readonly meshyLabEnabled?: boolean;
  readonly tileTargetEnabled?: boolean;
  readonly projectDatabase?: ProjectDatabaseV1;
  readonly visualQaFixture?: {
    readonly label: string;
    readonly load: () => Promise<{
      readonly source: File;
      readonly appearance: File;
    }>;
  };
}

export function App({
  meshyBridge,
  meshyLabEnabled = isMeshyLabEnabled(),
  tileTargetEnabled = isTileTargetEnabled(),
  projectDatabase,
  visualQaFixture,
}: AppProps = {}) {
  const { session, sessionRef, dispatch } = useStudioSessionController<
    SourceInspectionSnapshot,
    StudioBuildResult,
    AppearanceInspectionSnapshot
  >();
  const { workerRef, replaceWorker } = useStudioWorkerController();
  const [project, setProject] = useState<Meshy2AuroraProjectV1>(
    () => createMeshy2AuroraProjectV1(),
  );
  const projectRef = useRef(project);
  const [projectPersistence, setProjectPersistence] =
    useState<StudioProjectPersistence>("NOT_SAVED");
  const [projectManagerOpen, setProjectManagerOpen] = useState(false);
  const [projectBusy, setProjectBusy] = useState(false);
  const [projectError, setProjectError] = useState<string>();
  const [projectRecoveryRecords, setProjectRecoveryRecords] =
    useState<ProjectRecoveryRecordV1[]>([]);
  const [pendingProjectRebind, setPendingProjectRebind] = useState({
    sourceGlb: false,
    baseTwoDa: false,
    animationEvents: false,
  });
  const pendingProjectRebindRef = useRef(pendingProjectRebind);
  const [sourceError, setSourceError] = useState<string>();
  const [appearanceError, setAppearanceError] = useState<string>();
  const [animationEventsError, setAnimationEventsError] = useState<string>();
  const [visualQaFixtureState, setVisualQaFixtureState] =
    useState<"IDLE" | "LOADING" | "LOADED" | "ERROR">("IDLE");
  const [animationMappingSaveState, setAnimationMappingSaveState] =
    useState<AnimationMappingSaveStateV1>({ kind: "IDLE" });
  const [animationStudioState, setAnimationStudioState] =
    useState<AnimationStudioStateV1 | null>(null);
  const animationStudioStateRef = useRef<AnimationStudioStateV1 | null>(null);
  const [animationAuthoringV2, setAnimationAuthoringV2] =
    useState<CreatureAnimationAuthoringV2 | null>(null);
  const [animationAuthoringV2Dirty, setAnimationAuthoringV2Dirty] =
    useState(false);
  const animationAuthoringV2Ref = useRef<CreatureAnimationAuthoringV2 | null>(null);
  const [animationStudioCoreDiagnostics, setAnimationStudioCoreDiagnostics] =
    useState<AnimationStudioDiagnosticV1[]>([]);
  const [
    animationStudioMappingStorageDiagnostic,
    setAnimationStudioMappingStorageDiagnostic,
  ] = useState<AnimationStudioDiagnosticV1 | null>(null);
  const [editableAnimationRig, setEditableAnimationRig] =
    useState<AnimationRigNodeV1[]>([]);
  const [heldEquipmentRuntime, setHeldEquipmentRuntime] =
    useState<HeldEquipmentPreviewV1 | null>(null);
  const [animationStudioLoadedProjectId, setAnimationStudioLoadedProjectId] =
    useState<string | null>(null);
  const [tileOptions, setTileOptions] = useState<TileAuthoringOptions>(DEFAULT_TILE_OPTIONS);
  const [creatureProfile, setCreatureProfile] =
    useState<CreatureConversionProfileV1>("PRODUCT_300K");
  const [textureArtifactCleanup, setTextureArtifactCleanup] = useState(false);
  const [skinAccessoryStabilizationMode, setSkinAccessoryStabilizationMode] =
    useState<SkinAccessoryStabilizationModeV1>("AUTO");
  const [skinAccessorySelectedBoneName, setSkinAccessorySelectedBoneName] =
    useState("");
  const [placeableAuthoring, setPlaceableAuthoring] = useState<PlaceableAuthoringBootstrap>();
  const placeableAuthoringRef = useRef<PlaceableAuthoringBootstrap | undefined>(undefined);
  const [reviewViewport, setReviewViewport] = useState<ReviewViewport>("CONVERTED");
  const [selectedReadbackPart, setSelectedReadbackPart] = useState<ModelPartRef>();
  const [debugDrawerMessage, setDebugDrawerMessage] = useState<string>();
  const [showMeshyLab, setShowMeshyLab] = useState(false);
  const [meshyProvenance, setMeshyProvenance] = useState<MeshyArtifactProvenance>();
  const [meshyAnimationDonors, setMeshyAnimationDonors] = useState<readonly {
    readonly id: string;
    readonly file: File;
    readonly label: string;
    readonly detail: string;
  }[]>([]);
  const meshyBridgeRef = useRef<MeshyBridgeClient | undefined>(undefined);
  const coreAnimationValidationRequestRef = useRef<string | undefined>(undefined);

  if (!meshyBridgeRef.current) meshyBridgeRef.current = meshyBridge ?? new LocalMeshyBridgeClient();

  projectRef.current = project;
  pendingProjectRebindRef.current = pendingProjectRebind;
  placeableAuthoringRef.current = placeableAuthoring;
  animationStudioStateRef.current = animationStudioState;
  animationAuthoringV2Ref.current = animationAuthoringV2;

  const invalidateRunningBuild = () => {
    if (sessionRef.current.build.kind !== "RUNNING") return;
    replaceWorker();
    setReviewViewport("CONVERTED");
    setSelectedReadbackPart(undefined);
  };

  const refreshProjectRecoveryRecords = () => {
    void listProjectRecoveryRecordsV1(projectDatabase)
      .then(setProjectRecoveryRecords)
      .catch((error: unknown) => {
        setProjectError(
          `Local recovery is unavailable: ${
            error instanceof Error ? error.message : String(error)
          }`,
        );
      });
  };

  const applyProject = (
    nextProject: Meshy2AuroraProjectV1,
    persistence: StudioProjectPersistence,
  ) => {
    invalidateRunningBuild();
    projectRef.current = nextProject;
    setProject(nextProject);
    setProjectPersistence(persistence);
    setPendingProjectRebind({
      sourceGlb: nextProject.files.sourceGlb !== null,
      baseTwoDa: nextProject.files.baseTwoDa !== null,
      animationEvents: nextProject.files.animationEvents !== null,
    });
    setSourceError(nextProject.files.sourceGlb
      ? `Rebind the exact source file with SHA-256 ${nextProject.files.sourceGlb.sha256}.`
      : undefined);
    setAppearanceError(nextProject.files.baseTwoDa
      ? `Rebind the exact base table with SHA-256 ${nextProject.files.baseTwoDa.sha256}.`
      : undefined);
    setAnimationEventsError(nextProject.files.animationEvents
      ? `Rebind the exact event JSON with SHA-256 ${nextProject.files.animationEvents.sha256}.`
      : undefined);
    setAnimationMappingSaveState({ kind: "IDLE" });
    setAnimationStudioState(
      nextProject.animationStudio
        ? createAnimationStudioStateV1(nextProject.animationStudio)
        : null,
    );
    setAnimationAuthoringV2(nextProject.animationMappingV2);
    setAnimationAuthoringV2Dirty(false);
    setAnimationStudioCoreDiagnostics([]);
    setAnimationStudioMappingStorageDiagnostic(null);
    setAnimationStudioLoadedProjectId(null);
    setEditableAnimationRig([]);
    setHeldEquipmentRuntime(null);
    setTileOptions(nextProject.tileOptions);
    setPlaceableAuthoring(undefined);
    setReviewViewport("CONVERTED");
    setSelectedReadbackPart(undefined);
    setDebugDrawerMessage(undefined);
    setMeshyProvenance(undefined);
    setShowMeshyLab(false);
    setProjectManagerOpen(false);
    dispatch({ type: "PROJECT_OPENED", target: nextProject.target });
  };

  const createNewProject = () => {
    applyProject(createMeshy2AuroraProjectV1(), "DIRTY");
    setProjectError(undefined);
  };

  const importProject = (file: File) => {
    setProjectBusy(true);
    setProjectError(undefined);
    void file.text()
      .then((json) => {
        const parsed = parseMeshy2AuroraProjectV1(json);
        if (parsed.kind === "INVALID") {
          throw new Error(parsed.diagnostics.map(({ message }) => message).join(" "));
        }
        applyProject(parsed.value, "DIRTY");
      })
      .catch((error: unknown) => setProjectError(
        `Project import failed: ${error instanceof Error ? error.message : String(error)}`,
      ))
      .finally(() => setProjectBusy(false));
  };

  const duplicateProject = () => {
    applyProject(duplicateMeshy2AuroraProjectV1(projectRef.current), "DIRTY");
    setProjectError(undefined);
  };

  const recoverProject = (projectId: string) => {
    setProjectBusy(true);
    setProjectError(undefined);
    void loadProjectV1(projectId, projectDatabase)
      .then((result) => {
        if (result.kind !== "LOADED") {
          const message = result.kind === "ERROR"
            ? result.diagnostics.map(({ message: diagnosticMessage }) => diagnosticMessage).join(" ")
            : "The local project record no longer exists.";
          throw new Error(message);
        }
        applyProject(result.value, "SAVED");
      })
      .catch((error: unknown) => setProjectError(
        `Project recovery failed: ${error instanceof Error ? error.message : String(error)}`,
      ))
      .finally(() => setProjectBusy(false));
  };

  const removeRecoveryProject = (projectId: string) => {
    setProjectBusy(true);
    setProjectError(undefined);
    void deleteProjectV1(projectId, true, projectDatabase)
      .then((result) => {
        if (result.kind === "ERROR") throw new Error(result.diagnostic.message);
        refreshProjectRecoveryRecords();
      })
      .catch((error: unknown) => setProjectError(
        `Project delete failed: ${error instanceof Error ? error.message : String(error)}`,
      ))
      .finally(() => setProjectBusy(false));
  };

  const deleteCurrentProject = () => {
    const projectId = projectRef.current.identity.projectId;
    setProjectBusy(true);
    setProjectError(undefined);
    void deleteProjectV1(projectId, true, projectDatabase)
      .then((result) => {
        if (result.kind === "ERROR") throw new Error(result.diagnostic.message);
        applyProject(createMeshy2AuroraProjectV1(), "DIRTY");
        refreshProjectRecoveryRecords();
      })
      .catch((error: unknown) => setProjectError(
        `Project delete failed: ${error instanceof Error ? error.message : String(error)}`,
      ))
      .finally(() => setProjectBusy(false));
  };

  useEffect(() => {
    if (!projectManagerOpen) return;
    refreshProjectRecoveryRecords();
  }, [projectManagerOpen]);

  useEffect(() => {
    if (projectPersistence !== "DIRTY") return;
    const projectId = project.identity.projectId;
    const revision = project.revision;
    const timeout = window.setTimeout(() => {
      void saveProjectV1(project, projectDatabase).then((result) => {
        if (
          projectRef.current.identity.projectId !== projectId
          || projectRef.current.revision !== revision
        ) return;
        if (result.kind === "SAVED") {
          setProjectPersistence("SAVED");
          setProjectError(undefined);
          if (projectManagerOpen) refreshProjectRecoveryRecords();
        } else {
          setProjectPersistence("DIRTY");
          setProjectError(result.diagnostic.message);
        }
      });
    }, 350);
    return () => window.clearTimeout(timeout);
  }, [project, projectDatabase, projectManagerOpen, projectPersistence]);

  useEffect(() => {
    if (projectPersistence === "SAVED") return;
    const warnBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", warnBeforeUnload);
    return () => window.removeEventListener("beforeunload", warnBeforeUnload);
  }, [projectPersistence]);

  const sourceFile = session.source?.file;
  useEffect(() => {
    if (!sourceFile) return;
    const worker = workerRef.current;
    if (!worker) return;
    let cancelled = false;
    const inspectionRevision = session.revision;

    dispatch({
      type: "INPUT_METADATA_UPDATED",
      input: "SOURCE",
      revision: inspectionRevision,
      parse: { kind: "PARSING" },
    });

    void sourceFile.arrayBuffer()
      .then((sourceGlb) => worker.request(
        {
          requestId: requestId(),
          type: "INSPECT_SOURCE",
          sourceGlb,
          target: session.target,
          creatureProfile: session.target === "CREATURE" ? creatureProfile : undefined,
        },
        [sourceGlb],
      ))
      .then((response) => {
        if (
          cancelled
          || sessionRef.current.revision !== inspectionRevision
          || sessionRef.current.source?.file !== sourceFile
        ) return;
        if (!response.ok) throw new Error(response.message);
        if (response.type !== "SOURCE_INSPECTED") {
          throw new Error("Unexpected source inspection response");
        }
        const projection = projectSourceInspection(response.ingestJson);
        if (projection.kind === "FAILED") {
          throw new Error(`${projection.failure.code}: ${projection.failure.message}`);
        }
        const expectedSource = pendingProjectRebindRef.current.sourceGlb
          ? projectRef.current.files.sourceGlb
          : null;
        if (
          expectedSource
          && projection.snapshot.source.sha256 !== expectedSource.sha256
        ) {
          setSourceError(
            `Project expects source SHA-256 ${expectedSource.sha256}; selected file is ${
              projection.snapshot.source.sha256
            }. Select the exact source file.`,
          );
          dispatch({ type: "SOURCE_REMOVED" });
          return;
        }
        setSourceError(undefined);
        const bootstrap = response.placeableAuthoringJson
          ? parsePlaceableAuthoringBootstrap(response.placeableAuthoringJson)
          : undefined;
        const restoredPlaceable = projectRef.current.placeableAuthoring;
        setPlaceableAuthoring(
          bootstrap
            ? {
                ...bootstrap,
                document:
                  restoredPlaceable?.sourceSha256 === projection.snapshot.source.sha256
                    ? restoredPlaceable
                    : bootstrap.document,
              }
            : undefined,
        );
        dispatch({
          type: "SOURCE_INSPECTION_SUCCEEDED",
          revision: inspectionRevision,
          sha256: projection.snapshot.source.sha256,
          inspection: projection.snapshot,
        });
        if (expectedSource) {
          setPendingProjectRebind((current) => ({ ...current, sourceGlb: false }));
        }
      })
      .catch((error: unknown) => {
        if (
          cancelled
          || sessionRef.current.revision !== inspectionRevision
          || sessionRef.current.source?.file !== sourceFile
        ) return;
        const message = error instanceof Error ? error.message : String(error);
        setSourceError(message);
        dispatch({
          type: "INPUT_METADATA_UPDATED",
          input: "SOURCE",
          revision: inspectionRevision,
          parse: { kind: "INVALID", message },
        });
      });

    return () => { cancelled = true; };
  }, [creatureProfile, session.revision, session.target, sourceFile]);

  const appearanceFile = session.appearance?.file;
  useEffect(() => {
    if (!appearanceFile) return;
    const worker = workerRef.current;
    if (!worker) return;
    let cancelled = false;
    const inspectionRevision = session.revision;

    dispatch({
      type: "INPUT_METADATA_UPDATED",
      input: "APPEARANCE",
      revision: inspectionRevision,
      parse: { kind: "PARSING" },
    });

    void appearanceFile.arrayBuffer()
      .then((appearanceTwoDa) => worker.request(
        { requestId: requestId(), type: "INSPECT_APPEARANCE", appearanceTwoDa },
        [appearanceTwoDa],
      ))
      .then((response) => {
        if (
          cancelled
          || sessionRef.current.revision !== inspectionRevision
          || sessionRef.current.appearance?.file !== appearanceFile
        ) return;
        if (!response.ok) throw new Error(response.message);
        if (response.type !== "APPEARANCE_INSPECTED") {
          throw new Error("Unexpected appearance inspection response");
        }
        const inspection = projectAppearanceInspection(response.inspectionJson);
        const expectedAppearance = pendingProjectRebindRef.current.baseTwoDa
          ? projectRef.current.files.baseTwoDa
          : null;
        if (
          expectedAppearance
          && inspection.sourceSha256 !== expectedAppearance.sha256
        ) {
          setAppearanceError(
            `Project expects base 2DA SHA-256 ${expectedAppearance.sha256}; selected file is ${
              inspection.sourceSha256
            }. Select the exact base table.`,
          );
          dispatch({ type: "APPEARANCE_REMOVED" });
          return;
        }
        setAppearanceError(undefined);
        dispatch({
          type: "APPEARANCE_INSPECTION_SUCCEEDED",
          revision: inspectionRevision,
          sha256: inspection.sourceSha256,
          inspection,
        });
        if (expectedAppearance) {
          setPendingProjectRebind((current) => ({ ...current, baseTwoDa: false }));
        }
      })
      .catch((error: unknown) => {
        if (
          cancelled
          || sessionRef.current.revision !== inspectionRevision
          || sessionRef.current.appearance?.file !== appearanceFile
        ) return;
        const message = error instanceof Error ? error.message : String(error);
        setAppearanceError(message);
        dispatch({
          type: "INPUT_METADATA_UPDATED",
          input: "APPEARANCE",
          revision: inspectionRevision,
          parse: { kind: "INVALID", message },
        });
      });
    return () => { cancelled = true; };
  }, [appearanceFile, session.revision]);

  const animationEventsFile = session.animationEvents?.file;
  useEffect(() => {
    if (!animationEventsFile) return;
    let cancelled = false;
    const revision = session.revision;
    dispatch({
      type: "INPUT_METADATA_UPDATED",
      input: "ANIMATION_EVENTS",
      revision,
      parse: { kind: "PARSING" },
    });
    void Promise.all([
      animationEventsFile.text().then((json) => JSON.parse(json)),
      fileSha256(animationEventsFile),
    ])
      .then(([, sha256]) => {
        if (
          cancelled
          || sessionRef.current.revision !== revision
          || sessionRef.current.animationEvents?.file !== animationEventsFile
        ) return;
        const expectedEvents = pendingProjectRebindRef.current.animationEvents
          ? projectRef.current.files.animationEvents
          : null;
        if (expectedEvents && sha256 !== expectedEvents.sha256) {
          setAnimationEventsError(
            `Project expects animation-event SHA-256 ${expectedEvents.sha256}; selected file is ${
              sha256
            }. Select the exact event JSON.`,
          );
          dispatch({ type: "ANIMATION_EVENTS_REMOVED" });
          return;
        }
        setAnimationEventsError(undefined);
        dispatch({
          type: "INPUT_METADATA_UPDATED",
          input: "ANIMATION_EVENTS",
          revision,
          sha256,
          parse: { kind: "VALID" },
        });
        if (expectedEvents) {
          setPendingProjectRebind((current) => ({
            ...current,
            animationEvents: false,
          }));
        }
      })
      .catch((error: unknown) => {
        if (
          cancelled
          || sessionRef.current.revision !== revision
          || sessionRef.current.animationEvents?.file !== animationEventsFile
        ) return;
        const message = `Animation event JSON is invalid: ${
          error instanceof Error ? error.message : String(error)
        }`;
        setAnimationEventsError(message);
        dispatch({
          type: "INPUT_METADATA_UPDATED",
          input: "ANIMATION_EVENTS",
          revision,
          parse: { kind: "INVALID", message },
        });
      });
    return () => {
      cancelled = true;
    };
  }, [animationEventsFile, session.revision]);

  const selectSource = (file: File, provenance?: MeshyArtifactProvenance) => {
    if (!isGlb(file)) {
      setSourceError("Select a Meshy model in .glb format.");
      return;
    }
    invalidateRunningBuild();
    setSourceError(undefined);
    setAnimationEventsError(undefined);
    setPlaceableAuthoring(undefined);
    setMeshyProvenance(provenance);
    dispatch({ type: "SOURCE_SELECTED", file });
  };

  const selectTarget = (target: StudioTarget) => {
    if (target === "TILE" && !tileTargetEnabled) return;
    invalidateRunningBuild();
    setAppearanceError(undefined);
    setAnimationEventsError(undefined);
    setPlaceableAuthoring(undefined);
    if (target !== "CREATURE") setCreatureProfile("PRODUCT_300K");
    dispatch({ type: "TARGET_SELECTED", target });
  };

  const updateCreatureProfile = (profile: CreatureConversionProfileV1) => {
    invalidateRunningBuild();
    setCreatureProfile(profile);
    setAnimationEventsError(undefined);
    dispatch({ type: "AUTHORING_OPTIONS_CHANGED" });
  };

  const updateTextureArtifactCleanup = (enabled: boolean) => {
    invalidateRunningBuild();
    setTextureArtifactCleanup(enabled);
    dispatch({ type: "AUTHORING_OPTIONS_CHANGED" });
  };

  const updateSkinAccessoryStabilizationMode = (
    mode: SkinAccessoryStabilizationModeV1,
  ) => {
    invalidateRunningBuild();
    setSkinAccessoryStabilizationMode(mode);
    dispatch({ type: "AUTHORING_OPTIONS_CHANGED" });
  };

  const updateSkinAccessorySelectedBoneName = (boneName: string) => {
    invalidateRunningBuild();
    setSkinAccessorySelectedBoneName(boneName);
    dispatch({ type: "AUTHORING_OPTIONS_CHANGED" });
  };

  const updateTileOptions = (options: TileAuthoringOptions) => {
    invalidateRunningBuild();
    setTileOptions(options);
    dispatch({ type: "AUTHORING_OPTIONS_CHANGED" });
  };

  const selectAppearance = (file: File) => {
    if (!isBaseTwoDa(file)) {
      setAppearanceError("Select the base file named appearance.2da or placeables.2da.");
      return;
    }
    invalidateRunningBuild();
    setAppearanceError(undefined);
    setAnimationEventsError(undefined);
    setPlaceableAuthoring(undefined);
    dispatch({ type: "APPEARANCE_SELECTED", file });
  };

  const loadVisualQaFixture = async () => {
    if (!visualQaFixture || visualQaFixtureState === "LOADING") return;
    setVisualQaFixtureState("LOADING");
    try {
      const files = await visualQaFixture.load();
      if (sessionRef.current.target !== "CREATURE") selectTarget("CREATURE");
      selectSource(files.source);
      selectAppearance(files.appearance);
      setVisualQaFixtureState("LOADED");
    } catch {
      setVisualQaFixtureState("ERROR");
    }
  };

  const selectAnimationEvents = (file: File) => {
    if (!isJson(file)) {
      setAnimationEventsError("Select a creature animation event sidecar in .json format.");
      return;
    }
    invalidateRunningBuild();
    setAnimationEventsError(undefined);
    dispatch({ type: "ANIMATION_EVENTS_SELECTED", file });
  };

  const removeSource = () => {
    invalidateRunningBuild();
    setHeldEquipmentRuntime(null);
    setSourceError(undefined);
    setMeshyProvenance(undefined);
    setPlaceableAuthoring(undefined);
    setPendingProjectRebind((current) => ({ ...current, sourceGlb: false }));
    dispatch({ type: "SOURCE_REMOVED" });
  };

  const removeAppearance = () => {
    invalidateRunningBuild();
    setAppearanceError(undefined);
    setPendingProjectRebind((current) => ({ ...current, baseTwoDa: false }));
    dispatch({ type: "APPEARANCE_REMOVED" });
  };

  const removeAnimationEvents = () => {
    invalidateRunningBuild();
    setAnimationEventsError(undefined);
    setPendingProjectRebind((current) => ({ ...current, animationEvents: false }));
    dispatch({ type: "ANIMATION_EVENTS_REMOVED" });
  };

  const clearFiles = () => {
    invalidateRunningBuild();
    setHeldEquipmentRuntime(null);
    setSourceError(undefined);
    setMeshyProvenance(undefined);
    setAppearanceError(undefined);
    setAnimationEventsError(undefined);
    setSelectedReadbackPart(undefined);
    setDebugDrawerMessage(undefined);
    setTileOptions(DEFAULT_TILE_OPTIONS);
    setCreatureProfile("PRODUCT_300K");
    setTextureArtifactCleanup(false);
    setSkinAccessoryStabilizationMode("AUTO");
    setSkinAccessorySelectedBoneName("");
    setPlaceableAuthoring(undefined);
    setPendingProjectRebind({
      sourceGlb: false,
      baseTwoDa: false,
      animationEvents: false,
    });
    dispatch({ type: "START_NEW_CONVERSION" });
  };

  const startBuild = () => {
    const current = sessionRef.current;
    const worker = workerRef.current;
    if (
      !worker
      || current.currentStep !== "BUILD"
      || !current.source
      || !current.sourceInspection
      || (
        current.target !== "TILE"
        && (!current.appearance || !current.appearanceInspection)
      )
      || current.sourceInspection.revision !== current.revision
      || (
        current.target !== "TILE"
        && current.appearanceInspection?.revision !== current.revision
      )
    ) return;

    const buildRequestId = requestId();
    const buildRevision = current.revision;
    const projectBuildIdentity = projectBuildIdentityV1(projectRef.current);
    const projectIdentityJson = serializeProjectBuildIdentityV1(projectRef.current);
    const source = current.source.file;
    const appearance = current.appearance?.file;
    const animationEvents = current.animationEvents?.file;
    const tileLane = current.target === "TILE";
    const placeableLane = current.target === "PLACEABLE";
    const studioState = current.target === "CREATURE"
      ? animationStudioStateRef.current
      : null;
    const editedAnimationLane = Boolean(
      studioState && studioState.document.authoredClips.length > 0,
    );
    const persistedHeldWeapon = projectRef.current.animationWorkbench.heldWeapon;
    if (persistedHeldWeapon && !heldEquipmentRuntime) {
      setAnimationEventsError(
        `Reconnect the exact held weapon ${persistedHeldWeapon.source.name} before Build.`,
      );
      return;
    }
    if (heldEquipmentRuntime && !editedAnimationLane) {
      setAnimationEventsError(
        "Held-weapon export requires the current valid Animation Studio document.",
      );
      return;
    }
    const studioDocument = editedAnimationLane ? studioState?.document : undefined;
    const authoringV2 = editedAnimationLane
      ? animationAuthoringV2Ref.current
      : null;
    if (
      editedAnimationLane
      && (
        !studioState
        || !studioDocument
        || !authoringV2
        || studioState.document.status !== "VALID"
        || studioState.sourceStatus !== "CURRENT"
        || studioState.autosave.status !== "AUTOSAVED"
        || studioState.autosave.savedRevision
          !== studioState.document.authoringRevision
        || studioDocument.sourceRevision !== current.source.sha256
        || authoringV2.sourceRevision !== current.source.sha256
      )
    ) {
      setAnimationEventsError(
        "Edited animation build requires the exact current, VALID and autosaved Studio revision.",
      );
      return;
    }
    const placeableAuthoringJson = placeableLane && placeableAuthoringRef.current
      ? JSON.stringify(placeableAuthoringRef.current.document)
      : undefined;
    const fullNativeProfile = hasFullNativeDirectCreatureProfileV1(
      current.sourceInspection.value.clips,
    );
    const p100kLane = current.target === "CREATURE"
      && creatureProfile === "EXPERIMENTAL_P100K";
    const p300kLane = current.target === "CREATURE"
      && creatureProfile === "EXPERIMENTAL_P300K";
    const segmentedExperimentLane = p100kLane || p300kLane;
    let animationBuildInput: CreatureAnimationBuildInputV1 | undefined;
    if (current.target === "CREATURE") {
      try {
        animationBuildInput = createCreatureAnimationBuildInputV1(current);
      } catch (error) {
        setAnimationEventsError(error instanceof Error ? error.message : String(error));
        return;
      }
    }
    if (
      animationEvents
      && (tileLane || placeableLane || segmentedExperimentLane || !fullNativeProfile)
    ) {
      setAnimationEventsError(
        tileLane
          ? "Creature animation events cannot be used with tiles."
          : placeableLane
            ? "Creature animation events cannot be used with placeables.2da."
            : segmentedExperimentLane
              ? `The isolated ${p300kLane ? "300K" : "100K"} experiment authors its 42-state event profile inside the canonical pipeline.`
              : "Creature animation events require an exact source clip for every state in the 42-state profile.",
      );
      return;
    }
    if (
      animationEvents
      && studioDocument?.authoredClips.some(({ events }) => events.length > 0)
    ) {
      setAnimationEventsError(
        "Edited clip events and the legacy external animation-event sidecar cannot be combined. Remove one event source.",
      );
      return;
    }
    const packageLane = current.target !== "CREATURE"
      ? "M0_STATIC_RIGID" as const
      : editedAnimationLane
        ? "H1_SKINNED_FULL_42_EDITED" as const
        : p300kLane
          ? "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT" as const
          : p100kLane
            ? "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT" as const
            : current.sourceInspection.value.inventory.skinCount === 0
              && current.sourceInspection.value.clips.length === 0
              ? "M0_STATIC_RIGID" as const
              : fullNativeProfile
                ? "H1_SKINNED_FULL_42_AUTHORED" as const
                : "SKINNED_PROCEDURAL_HUMANOID_42" as const;
    const productPipelineLane = packageLane === "SKINNED_PROCEDURAL_HUMANOID_42"
      || packageLane === "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT"
      || packageLane === "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT";
    const animationAuthoringJson = editedAnimationLane && authoringV2
      ? serializeCreatureAnimationAuthoringV2(authoringV2)
      : animationBuildInput?.animationAuthoringJson;
    const animationStudioDocumentJson = studioDocument
      ? serializeAnimationStudioDocumentV1(studioDocument)
      : undefined;
    const heldWeaponTargetNode = heldEquipmentRuntime
      ? editableAnimationRig.find(({ name }) => name === heldEquipmentRuntime.targetBoneName)
      : undefined;
    if (heldEquipmentRuntime && !heldWeaponTargetNode) {
      setAnimationEventsError(
        `Held-weapon bone ${heldEquipmentRuntime.targetBoneName} is absent from the current rig.`,
      );
      return;
    }
    const heldWeaponRigJson = heldEquipmentRuntime && studioDocument
      ? JSON.stringify({
          schemaVersion: 1,
          sourceRevision: studioDocument.sourceRevision,
          animationRoot: studioDocument.authoredClips[0]?.animationRoot
            ?? editableAnimationRig.find(({ parentId }) => parentId === null)?.name
            ?? editableAnimationRig[0]?.name
            ?? "root",
          nodes: editableAnimationRig.map((node) => ({
            nodeId: node.id,
            name: node.name,
            parentId: node.parentId,
            translation: node.translation,
            rotation: node.rotation,
          })),
        })
      : undefined;
    if (
      (
        packageLane === "H1_SKINNED_FULL_42_AUTHORED"
        || packageLane === "H1_SKINNED_FULL_42_EDITED"
      )
      && !animationAuthoringJson
    ) {
      setAnimationEventsError("Creature build requires the current animation mapping document.");
      return;
    }
    const creatureIdentityJson = JSON.stringify(
      p300kLane
        ? studioCreatureP300kPackageIdentity(
            current.sourceInspection.value.source.sha256,
            textureArtifactCleanup,
          )
        : p100kLane
          ? studioCreatureP100kPackageIdentity(
              current.sourceInspection.value.source.sha256,
              textureArtifactCleanup,
            )
        : studioCreatureProductIdentity(
            current.sourceInspection.value.source.sha256,
            textureArtifactCleanup,
          ),
    );
    setDebugDrawerMessage(undefined);
    dispatch({ type: "BUILD_STARTED", requestId: buildRequestId, revision: buildRevision });

    void Promise.all([
      source.arrayBuffer(),
      appearance?.arrayBuffer(),
      animationEvents?.text(),
      studioDocument
        ? fingerprintAnimationStudioDocumentV1(studioDocument)
        : Promise.resolve(undefined),
      heldEquipmentRuntime?.file.arrayBuffer(),
    ])
      .then(([
        sourceGlb,
        appearanceTwoDa,
        eventAuthoringJson,
        animationStudioFingerprintSha256,
        heldWeaponGlb,
      ]) => {
        if (workerRef.current !== worker) return undefined;
        if (animationBuildInput) {
          assertCreatureAnimationBuildInputCurrentV1(
            sessionRef.current,
            animationBuildInput,
          );
        }
        if (!tileLane && !appearanceTwoDa) {
          throw new Error("The selected conversion target requires a base 2DA");
        }
        const targetBuild = tileLane
          ? createTargetBuildRequestV1({
              target: "TILE",
              requestId: buildRequestId,
              sourceGlb,
              tileOptions,
            })
          : placeableLane
            ? createTargetBuildRequestV1({
                target: "PLACEABLE",
                requestId: buildRequestId,
                sourceGlb,
                baseTwoDa: appearanceTwoDa!,
                projectIdentityJson,
                ...(placeableAuthoringJson
                  ? { authoringJson: placeableAuthoringJson }
                  : {}),
              })
            : packageLane === "H1_SKINNED_FULL_42_AUTHORED"
              || packageLane === "H1_SKINNED_FULL_42_EDITED"
              ? createTargetBuildRequestV1({
                  target: "CREATURE",
                  requestId: buildRequestId,
                  sourceGlb,
                  baseTwoDa: appearanceTwoDa!,
                  projectIdentityJson,
                  packageLane,
                  animationAuthoringJson: animationAuthoringJson ?? "",
                  ...(animationStudioDocumentJson
                    ? { animationStudioDocumentJson }
                    : {}),
                  ...(eventAuthoringJson ? { eventAuthoringJson } : {}),
                  ...(heldEquipmentRuntime && heldWeaponGlb && heldWeaponTargetNode && heldWeaponRigJson
                    ? {
                        heldWeapon: {
                          glb: heldWeaponGlb,
                          filename: heldEquipmentRuntime.file.name,
                          rigJson: heldWeaponRigJson,
                          primaryHand: heldWeaponPrimaryHandV1(
                            heldEquipmentRuntime.targetBoneName,
                          ),
                          targetNodeId: heldWeaponTargetNode.id,
                          localTransformJson: JSON.stringify({
                            translation: heldEquipmentRuntime.translation,
                            rotation: quaternionFromEulerDegreesV1(
                              heldEquipmentRuntime.rotationEulerDegrees,
                            ),
                            scale: heldEquipmentRuntime.scale,
                          }),
                          attachmentRevision: projectRef.current.revision,
                        },
                      }
                    : {}),
                })
              : packageLane === "SKINNED_PROCEDURAL_HUMANOID_42"
                || packageLane === "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT"
                || packageLane === "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT"
                ? {
                    request: {
                      requestId: buildRequestId,
                      type: "BUILD_MODEL_PACKAGE" as const,
                      sourceGlb,
                      appearanceTwoDa: appearanceTwoDa!,
                      packageLane,
                      identityJson: creatureIdentityJson,
                      textureArtifactCleanup,
                      skinAccessoryStabilization: {
                        mode: skinAccessoryStabilizationMode,
                        ...(skinAccessoryStabilizationMode === "SELECT_BONE"
                          ? { selectedBoneName: skinAccessorySelectedBoneName.trim() }
                          : {}),
                      },
                    },
                    transfer: [sourceGlb, appearanceTwoDa!],
                  }
                : {
                    request: {
                      requestId: buildRequestId,
                      type: "BUILD_MODEL_PACKAGE" as const,
                      sourceGlb,
                      appearanceTwoDa: appearanceTwoDa!,
                      packageLane,
                    },
                    transfer: [sourceGlb, appearanceTwoDa!],
                  };
        const response = worker.request(
          targetBuild.request,
          targetBuild.transfer,
        );
        return response.then((resolved) => ({
          response: resolved,
          animationStudioFingerprintSha256,
        }));
      })
      .then(async (buildOutput) => {
        if (!buildOutput || workerRef.current !== worker) return;
        const {
          response,
          animationStudioFingerprintSha256,
        } = buildOutput;
        const currentBuild = sessionRef.current.build;
        if (
          sessionRef.current.revision !== buildRevision
          || currentBuild.kind !== "RUNNING"
          || currentBuild.requestId !== buildRequestId
          || currentBuild.revision !== buildRevision
        ) return;
        if (!response.ok) throw new Error(response.message);
        if (
          response.type !== "PLACEABLE_PACKAGE_BUILT"
          && response.type !== "MODEL_PACKAGE_BUILT"
          && response.type !== "TILE_PACKAGE_BUILT"
        ) {
          throw new Error("Unexpected package build response");
        }
        const {
          projectCanonicalReadback,
          projectCanonicalResult,
          projectPlaceableResult,
          projectTileResult,
        } = await import("./features/results/buildResultParsers");
        const latestBuild = sessionRef.current.build;
        if (
          sessionRef.current.revision !== buildRevision
          || latestBuild.kind !== "RUNNING"
          || latestBuild.requestId !== buildRequestId
          || latestBuild.revision !== buildRevision
        ) return;
        const readbackJson = response.type === "TILE_PACKAGE_BUILT"
          ? response.modelReadbackJson
          : response.readbackJson;
        const readback = projectCanonicalReadback(readbackJson);
        setReviewViewport("CONVERTED");
        setSelectedReadbackPart(undefined);
        const canonical = response.type === "MODEL_PACKAGE_BUILT"
          ? projectCanonicalResult(
              response.reportJson,
              response.summaryJson,
              response.manifestJson,
              response.artifacts,
            )
          : undefined;
        const placeableResult = response.type === "PLACEABLE_PACKAGE_BUILT"
          ? projectPlaceableResult(response.reportJson, response.artifacts)
          : undefined;
        if (
          canonical
          && !productPipelineLane
          && !sameProjectBuildIdentityV1(
            canonical.projectIdentity,
            projectBuildIdentity,
          )
        ) {
          throw new Error(
            `Canonical build manifest does not match the frozen project identity (expected ${projectBuildIdentity.projectId}@${projectBuildIdentity.projectRevision}, received ${canonical.projectIdentity?.projectId ?? "missing"}@${canonical.projectIdentity?.projectRevision ?? "missing"})`,
          );
        }
        if (
          placeableResult
          && !sameProjectBuildIdentityV1(
            placeableResult.projectIdentity,
            projectBuildIdentity,
          )
        ) {
          throw new Error(
            `Placeable build report does not match the frozen project identity (expected ${projectBuildIdentity.projectId}@${projectBuildIdentity.projectRevision}, received ${placeableResult.projectIdentity.projectId}@${placeableResult.projectIdentity.projectRevision})`,
          );
        }
        let animationReconciliation: AuthoredBuiltAnimationReconciliationV1 | undefined;
        if (canonical && !editedAnimationLane && !productPipelineLane) {
          if (!animationBuildInput) {
            throw new Error("Authored creature build snapshot is unavailable");
          }
          const builtEvidence = canonical.animationMappingEvidence;
          if (
            !builtEvidence
            || builtEvidence.authoringRevision !== animationBuildInput.authoringRevision
            || builtEvidence.sourceRevision !== animationBuildInput.sourceRevision
            || builtEvidence.authoringFingerprintSha256
              !== animationBuildInput.authoringFingerprintSha256
          ) {
            throw new Error(
              "Canonical build animation evidence does not match the frozen authoring snapshot",
            );
          }
          animationReconciliation = reconcileAuthoredAndBuiltAnimationsV1(
            animationBuildInput.authoring,
            readback,
          );
          if (!animationReconciliation.matches) {
            throw new Error(
              `Authored/readback animation mismatch: ${
                animationReconciliation.diagnostics.map(({ code }) => code).join(", ")
              }`,
            );
          }
        }
        let animationStudioReconciliation:
          AnimationStudioReadbackReconciliationV1 | undefined;
        if (studioDocument) {
          const builtEvidence = canonical?.animationStudioEvidence;
          if (
            !builtEvidence
            || !animationStudioFingerprintSha256
            || builtEvidence.animationStudioRevision
              !== studioDocument.authoringRevision
            || builtEvidence.animationStudioFingerprintSha256
              !== animationStudioFingerprintSha256
            || builtEvidence.sourceRevision !== studioDocument.sourceRevision
          ) {
            throw new Error(
              "Canonical build Animation Studio evidence does not match the frozen document",
            );
          }
          animationStudioReconciliation = reconcileAnimationStudioReadbackV1(
            studioDocument,
            readback,
            builtEvidence,
          );
          if (animationStudioReconciliation.status !== "MATCH") {
            throw new Error(
              `Animation Studio/readback mismatch: ${
                animationStudioReconciliation.diagnostics
                  .map(({ code }) => code)
                  .join(", ")
              }`,
            );
          }
        }
        const result: StudioBuildResult = response.type === "PLACEABLE_PACKAGE_BUILT"
          ? {
              kind: "PLACEABLE",
              projectIdentity: projectBuildIdentity,
              placeable: placeableResult!,
              readback,
              readbackJson: response.readbackJson,
            }
          : response.type === "TILE_PACKAGE_BUILT"
            ? {
                kind: "TILE",
                projectIdentity: projectBuildIdentity,
                tile: projectTileResult(
                  response.reportJson,
                  response.wokReadbackJson,
                  response.setReadbackJson,
                  response.artifacts,
                ),
                readback,
                readbackJson: response.modelReadbackJson,
              }
          : response.type === "MODEL_PACKAGE_BUILT"
            ? {
                kind: "MODEL",
                projectIdentity: projectBuildIdentity,
                canonical: canonical!,
                readback,
                readbackJson: response.readbackJson,
                animationBuildInput: animationBuildInput!,
                ...(animationReconciliation
                  ? { animationReconciliation }
                  : {}),
                ...(studioDocument && authoringV2
                  && animationStudioFingerprintSha256
                  && animationStudioReconciliation ? {
                    animationStudioDocument: studioDocument,
                    animationAuthoringV2: authoringV2,
                    animationStudioFingerprintSha256,
                    animationStudioReconciliation,
                  } : {}),
              }
            : (() => { throw new Error("Unexpected package build response"); })();
        dispatch({
          type: "BUILD_SUCCEEDED",
          requestId: buildRequestId,
          revision: buildRevision,
          result,
        });
        if (studioDocument) {
          setAnimationStudioState((studioCurrent) => (
            studioCurrent?.document.authoringRevision
              === studioDocument.authoringRevision
              ? markAnimationStudioBuildRevisionV1(
                  studioCurrent,
                  studioDocument.authoringRevision,
                )
              : studioCurrent
          ));
        }
      })
      .catch((error: unknown) => {
        if (workerRef.current !== worker) return;
        const currentBuild = sessionRef.current.build;
        if (
          sessionRef.current.revision !== buildRevision
          || currentBuild.kind !== "RUNNING"
          || currentBuild.requestId !== buildRequestId
          || currentBuild.revision !== buildRevision
        ) return;
        const message = error instanceof Error ? error.message : String(error);
        dispatch({
          type: "BUILD_FAILED",
          requestId: buildRequestId,
          revision: buildRevision,
          failure: projectBuildFailure(message),
        });
      });
  };

  const cancelBuild = () => {
    const current = sessionRef.current;
    if (current.build.kind !== "RUNNING") return;
    dispatch({
      type: "BUILD_CANCELLED",
      requestId: current.build.requestId,
      revision: current.build.revision,
    });
    replaceWorker();
  };

  const workflowSteps = getWorkflowStepsForTarget(session.target);
  const unlockedSteps = getUnlockedWorkflowSteps(session);
  const completedSteps = workflowSteps.filter(
    (step) => getWorkflowStepStatus(session, step) === "COMPLETE",
  );
  const blockedSteps = workflowSteps.filter(
    (step) => getWorkflowStepStatus(session, step) === "LOCKED",
  );
  const sourceIdentity = session.source?.sha256
    ? { sha256: session.source.sha256 }
    : project.files.sourceGlb
      ? { sha256: project.files.sourceGlb.sha256 }
      : undefined;
  const appearanceIdentity = session.appearance?.sha256
    ? { sha256: session.appearance.sha256 }
    : project.files.baseTwoDa
      ? { sha256: project.files.baseTwoDa.sha256 }
      : undefined;
  const sourceInspection = session.sourceInspection?.revision === session.revision
    ? session.sourceInspection.value
    : undefined;
  const appearanceInspection = session.appearanceInspection?.revision === session.revision
    ? session.appearanceInspection.value
    : undefined;
  const animationInspection = useMemo(() => ({
    sourceClips: (sourceInspection?.clips ?? []).map((clip, index) => ({
      clipId: `source-clip:${clip.id}`,
      name: clip.name ?? `Unnamed clip ${index + 1}`,
      durationSeconds: clip.durationSeconds,
      trackCount: clip.channelCount,
      targetNodeIds: clip.targetNodeIds,
      targetPaths: clip.targetPaths,
    })),
  }), [sourceInspection]);
  const animationDiagnostics = useMemo(() => (
    session.animationMapping
      ? validateCreatureAnimationAuthoringV1(
          session.animationMapping.value,
          animationInspection,
        )
      : []
  ), [animationInspection, session.animationMapping]);
  const animationMappingStatus = getCreatureAnimationMappingStatusV1(
    animationDiagnostics,
  );

  const animationStudioSourceRevision = session.target === "CREATURE"
    ? session.source?.sha256 ?? null
    : null;
  const animationStudioSourceFile = session.target === "CREATURE"
    ? session.source?.file ?? null
    : null;
  const animationStudioProjectId = project.identity.projectId;

  useEffect(() => {
    const sourceRevision = animationStudioSourceRevision;
    const sourceFileForStudio = animationStudioSourceFile;
    if (!sourceRevision || !sourceFileForStudio) {
      setAnimationStudioState(null);
      setAnimationAuthoringV2(null);
      setAnimationAuthoringV2Dirty(false);
      setEditableAnimationRig([]);
      setHeldEquipmentRuntime(null);
      setAnimationStudioLoadedProjectId(null);
      setAnimationStudioMappingStorageDiagnostic(null);
      return;
    }
    let cancelled = false;
    setAnimationAuthoringV2Dirty(false);
    setAnimationStudioLoadedProjectId(null);
    setAnimationStudioCoreDiagnostics([]);
    const previousStudio = animationStudioStateRef.current;
    const projectDocument = projectRef.current.animationStudio;
    const preserveStaleStudio = previousStudio !== null
      && previousStudio.document.authoredClips.length > 0
      && previousStudio.document.sourceRevision !== sourceRevision;
    if (preserveStaleStudio) {
      // Keep the exact authored document observable and fail closed. Do not
      // silently replace it with an empty project for the newly selected GLB.
      setAnimationStudioState(
        reconcileAnimationStudioSourceRevisionV1(previousStudio, sourceRevision),
      );
    } else if (projectDocument?.sourceRevision === sourceRevision) {
      const restored = createAnimationStudioStateV1(projectDocument);
      setAnimationStudioState({
        ...restored,
        autosave: {
          status: "AUTOSAVED",
          savedRevision: projectDocument.authoringRevision,
          savedAt: projectRef.current.identity.updatedAt,
          error: null,
        },
      });
      setAnimationStudioLoadedProjectId(animationStudioProjectId);
    } else {
      setAnimationStudioState(
        createAnimationStudioStateV1(
          createEmptyAnimationStudioDocumentV1(sourceRevision),
        ),
      );
      void loadAnimationStudioDocumentV1(animationStudioProjectId)
        .then((loaded) => {
          if (cancelled || sessionRef.current.source?.sha256 !== sourceRevision) return;
          if (loaded.kind === "LOADED") {
            const restored = createAnimationStudioStateV1(loaded.value);
            setAnimationStudioState({
              ...restored,
              mode: loaded.selectedMode,
              selectedClipId: loaded.selectedClipId,
              autosave: {
                status: "AUTOSAVED",
                savedRevision: loaded.value.authoringRevision,
                savedAt: loaded.savedAt,
                error: null,
              },
            });
          } else if (loaded.kind === "ERROR") {
            setAnimationStudioCoreDiagnostics(loaded.diagnostics);
          }
          setAnimationStudioLoadedProjectId(animationStudioProjectId);
        });
    }

    const worker = workerRef.current;
    if (worker) {
      void sourceFileForStudio.arrayBuffer()
        .then((sourceGlb) => worker.inspectEditableAnimationSource(sourceGlb))
        .then((response) => {
          if (
            cancelled
            || !response.ok
            || response.type !== "EDITABLE_ANIMATION_SOURCE_INSPECTED"
            || sessionRef.current.source?.sha256 !== sourceRevision
          ) return;
          setEditableAnimationRig(
            parseEditableAnimationSourceV1(response.inspectionJson).rig,
          );
        })
        .catch((error: unknown) => {
          if (!cancelled) {
            setSourceError(
              `Editable animation source inspection failed: ${
                error instanceof Error ? error.message : String(error)
              }`,
            );
          }
        });
    }
    return () => {
      cancelled = true;
    };
  }, [
    animationStudioProjectId,
    animationStudioSourceFile,
    animationStudioSourceRevision,
  ]);

  useEffect(() => {
    const mapping = session.animationMapping?.value;
    if (!mapping || !animationStudioSourceRevision) return;
    const loaded = loadCreatureAnimationAuthoringV2DraftV1(
      animationStudioSourceRevision,
    );
    const projectMapping = projectRef.current.animationMappingV2;
    setAnimationStudioMappingStorageDiagnostic(
      loaded.kind === "ERROR" ? loaded.diagnostic : null,
    );
    setAnimationAuthoringV2((current) => synchronizeCreatureAnimationAuthoringV2(
      mapping,
      current
        ?? (
          projectMapping?.sourceRevision === animationStudioSourceRevision
            ? projectMapping
            : loaded.kind === "LOADED"
              ? loaded.value
              : null
        ),
    ));
  }, [animationStudioSourceRevision, session.animationMapping]);

  useEffect(() => {
    if (
      !animationAuthoringV2
      || !animationStudioSourceRevision
      || !animationAuthoringV2Dirty
    ) return;
    const saved = saveCreatureAnimationAuthoringV2DraftV1(
      animationStudioSourceRevision,
      animationAuthoringV2,
    );
    if (saved.kind === "SAVED") setAnimationAuthoringV2Dirty(false);
    setAnimationStudioMappingStorageDiagnostic(
      saved.kind === "ERROR" ? saved.diagnostic : null,
    );
  }, [
    animationAuthoringV2,
    animationAuthoringV2Dirty,
    animationStudioSourceRevision,
  ]);

  useEffect(() => {
    const studio = animationStudioState;
    const projectId = animationStudioProjectId;
    if (
      !studio
      || !animationStudioSourceRevision
      || animationStudioLoadedProjectId !== projectId
    ) return;
    const revision = studio.document.authoringRevision;
    const timeout = window.setTimeout(() => {
      setAnimationStudioState((current) => (
        current?.document.authoringRevision === revision
          ? markAnimationStudioAutosaveSavingV1(current)
          : current
      ));
      void saveAnimationStudioDocumentV1(
        projectId,
        studio.document,
        undefined,
        {
          selectedMode: studio.mode,
          selectedClipId: studio.selectedClipId,
        },
      ).then((result) => {
        setAnimationStudioState((current) => {
          if (!current || current.document.authoringRevision !== revision) return current;
          return result.kind === "SAVED"
            ? markAnimationStudioAutosavedV1(
                current,
                result.revision,
                result.savedAt,
              )
            : markAnimationStudioAutosaveFailedV1(
                current,
                result.diagnostic.message,
              );
        });
      });
    }, 250);
    return () => window.clearTimeout(timeout);
  }, [
    animationStudioLoadedProjectId,
    animationStudioProjectId,
    animationStudioSourceRevision,
    animationStudioState?.document,
    animationStudioState?.mode,
    animationStudioState?.selectedClipId,
  ]);

  useEffect(() => {
    const studio = animationStudioState;
    const authoring = animationAuthoringV2;
    const source = session.source;
    const worker = workerRef.current;
    if (!studio || !authoring || !source?.sha256 || !worker) return;
    const localDiagnostics = [
      ...validateAnimationStudioSchemaV1(studio.document),
      ...validateCreatureAnimationAuthoringV2(authoring, studio.document),
    ];
    if (studio.document.authoredClips.length === 0) {
      setAnimationStudioCoreDiagnostics(localDiagnostics);
      return;
    }
    let cancelled = false;
    const timeout = window.setTimeout(() => {
      void source.file.arrayBuffer()
        .then((sourceGlb) => worker.validateAnimationStudioDocument(
          sourceGlb,
          serializeAnimationStudioDocumentV1(studio.document),
        ))
        .then((response) => {
          if (
            cancelled
            || !response.ok
            || response.type !== "ANIMATION_STUDIO_DOCUMENT_VALIDATED"
          ) return;
          const parsed = JSON.parse(response.validationJson) as {
            diagnostics?: AnimationStudioDiagnosticV1[];
          } | AnimationStudioDiagnosticV1[];
          const coreDiagnostics = Array.isArray(parsed)
            ? parsed
            : parsed.diagnostics ?? [];
          setAnimationStudioCoreDiagnostics([
            ...localDiagnostics,
            ...coreDiagnostics,
          ]);
        })
        .catch((error: unknown) => {
          if (cancelled) return;
          setAnimationStudioCoreDiagnostics([
            ...localDiagnostics,
            {
              schemaVersion: 1,
              code: "M2A-ANIMATION-EDIT-WASM",
              path: "animationStudioDocument",
              level: "BLOCKING",
              message: error instanceof Error ? error.message : String(error),
              action: "Retry canonical Animation Studio validation.",
            },
          ]);
        });
    }, 200);
    return () => {
      cancelled = true;
      window.clearTimeout(timeout);
    };
  }, [animationAuthoringV2, animationStudioState?.document, session.source]);

  useEffect(() => {
    const studio = animationStudioState;
    const worker = workerRef.current;
    const previewClipId = studio?.selectedClipId
      ?? studio?.document.authoredClips[0]?.id;
    if (
      !studio
      || !worker
      || !previewClipId
      || studio.document.status !== "VALID"
      || studio.document.authoredClips.length === 0
      || studio.document.authoredClips.some(({ status }) => status !== "VALID")
    ) return;
    let cancelled = false;
    const timeout = window.setTimeout(() => {
      void worker.previewAuthoredAnimationClip(
        serializeAnimationStudioDocumentV1(studio.document),
        previewClipId,
      )
        .then((response) => {
          if (
            cancelled
            || !response.ok
            || response.type !== "AUTHORED_ANIMATION_CLIP_PREVIEWED"
          ) return;
          // The viewport stays a projection of the immutable Studio document.
          // This canonical preview is a debounced parity gate only.
          JSON.parse(response.previewJson);
        })
        .catch((error: unknown) => {
          if (cancelled || (error instanceof DOMException && error.name === "AbortError")) {
            return;
          }
          setSourceError(
            `Animation Studio preview materialization failed: ${
              error instanceof Error ? error.message : String(error)
            }`,
          );
        });
    }, 350);
    return () => {
      cancelled = true;
      window.clearTimeout(timeout);
      worker.cancelAnimationStudioPreviewBuild();
    };
  }, [
    animationStudioState?.document,
    animationStudioState?.selectedClipId,
  ]);

  useEffect(() => {
    const current = projectRef.current;
    const sourceGlb = session.source?.sha256
      ? current.files.sourceGlb?.sha256 === session.source.sha256
        ? current.files.sourceGlb
        : projectFileReferenceV1(session.source.file, session.source.sha256)
      : pendingProjectRebind.sourceGlb
        ? current.files.sourceGlb
        : null;
    const baseTwoDa = session.appearance?.sha256
      ? current.files.baseTwoDa?.sha256 === session.appearance.sha256
        ? current.files.baseTwoDa
        : projectFileReferenceV1(session.appearance.file, session.appearance.sha256)
      : pendingProjectRebind.baseTwoDa
        ? current.files.baseTwoDa
        : null;
    const animationEvents = session.animationEvents?.sha256
      ? current.files.animationEvents?.sha256 === session.animationEvents.sha256
        ? current.files.animationEvents
        : projectFileReferenceV1(
            session.animationEvents.file,
            session.animationEvents.sha256,
          )
      : pendingProjectRebind.animationEvents
        ? current.files.animationEvents
        : null;
    const files: Meshy2AuroraProjectFilesV1 = {
      sourceGlb,
      baseTwoDa,
      animationEvents,
    };
    const projected: Meshy2AuroraProjectV1 = {
      ...current,
      target: session.target,
      files,
      animationMappingV2: pendingProjectRebind.sourceGlb
        ? current.animationMappingV2
        : animationAuthoringV2,
      animationStudio: pendingProjectRebind.sourceGlb
        ? current.animationStudio
        : animationStudioState?.document ?? null,
      animationWorkbench: pendingProjectRebind.sourceGlb
        ? current.animationWorkbench
        : current.animationWorkbench.sourceRevision === (sourceGlb?.sha256 ?? null)
          ? current.animationWorkbench
          : {
              ...emptyAnimationWorkbenchProjectStateV1(),
              sourceRevision: sourceGlb?.sha256 ?? null,
            },
      placeableAuthoring: pendingProjectRebind.sourceGlb
        ? current.placeableAuthoring
        : placeableAuthoring?.document ?? null,
      tileOptions,
    };
    if (projectContentSignature(current) === projectContentSignature(projected)) {
      return;
    }
    const revised = reviseMeshy2AuroraProjectV1(current, {
      target: projected.target,
      files: projected.files,
      animationMappingV2: projected.animationMappingV2,
      animationStudio: projected.animationStudio,
      animationWorkbench: projected.animationWorkbench,
      placeableAuthoring: projected.placeableAuthoring,
      tileOptions: projected.tileOptions,
    });
    projectRef.current = revised;
    setProject(revised);
    setProjectPersistence("DIRTY");
  }, [
    animationAuthoringV2,
    animationStudioState?.document,
    pendingProjectRebind,
    placeableAuthoring?.document,
    session.animationEvents,
    session.appearance,
    session.source,
    session.target,
    tileOptions,
  ]);

  const animationStudioHasAuthoredClips =
    (animationStudioState?.document.authoredClips.length ?? 0) > 0;
  const animationStudioDiagnostics = animationStudioMappingStorageDiagnostic
    ? [
        animationStudioMappingStorageDiagnostic,
        ...animationStudioCoreDiagnostics,
      ]
    : animationStudioCoreDiagnostics;
  const animationStudioReadyForBuild = !animationStudioHasAuthoredClips || Boolean(
    animationStudioState
    && animationAuthoringV2
    && animationStudioState.document.status === "VALID"
    && animationStudioState.sourceStatus === "CURRENT"
    && animationStudioState.autosave.status === "AUTOSAVED"
    && animationStudioState.autosave.savedRevision
      === animationStudioState.document.authoringRevision
    && animationStudioDiagnostics.every(({ level }) => level !== "BLOCKING"),
  );

  useEffect(() => {
    const mapping = session.animationMapping;
    if (
      session.currentStep !== "ANIMATION_MAPPING"
      || !mapping
      || mapping.revision !== session.revision
    ) return;
    const current = session.animationMappingValidation?.value;
    if (current?.authoringRevision === mapping.value.authoringRevision) return;
    dispatch({
      type: "ANIMATION_MAPPING_VALIDATED",
      revision: session.revision,
      authoringRevision: mapping.value.authoringRevision,
      status: animationMappingStatus,
      diagnostics: animationDiagnostics,
    });
  }, [
    animationDiagnostics,
    animationMappingStatus,
    session.animationMapping,
    session.animationMappingValidation,
    session.currentStep,
    session.revision,
  ]);

  useEffect(() => {
    const mapping = session.animationMapping;
    const worker = workerRef.current;
    if (
      !worker
      || session.currentStep !== "ANIMATION_MAPPING"
      || !mapping
      || mapping.revision !== session.revision
    ) return;
    const key = `${session.revision}:${mapping.value.authoringRevision}:${
      JSON.stringify(mapping.value)
    }`;
    if (coreAnimationValidationRequestRef.current === key) return;
    coreAnimationValidationRequestRef.current = key;
    void worker.validateCreatureAnimationMapping(JSON.stringify(mapping.value))
      .then((response) => {
        if (
          !response.ok
          || response.type !== "CREATURE_ANIMATION_MAPPING_VALIDATED"
        ) return;
        assertDirectCreatureCatalogParityV1(response.catalogJson);
        const coreValidation = projectCoreCreatureAnimationValidationV1(
          response.validationJson,
        );
        const diagnostics = mergeUiAndCoreAnimationDiagnosticsV1(
          animationDiagnostics,
          coreValidation.diagnostics,
        );
        const current = sessionRef.current.animationMapping;
        if (
          current?.revision !== session.revision
          || current.value.authoringRevision !== mapping.value.authoringRevision
        ) return;
        dispatch({
          type: "ANIMATION_MAPPING_VALIDATED",
          revision: session.revision,
          authoringRevision: mapping.value.authoringRevision,
          authoringFingerprintSha256:
            coreValidation.authoringFingerprintSha256,
          status: getCreatureAnimationMappingStatusV1(diagnostics),
          diagnostics,
        });
      })
      .catch((error: unknown) => {
        const current = sessionRef.current.animationMapping;
        if (
          current?.revision !== session.revision
          || current.value.authoringRevision !== mapping.value.authoringRevision
        ) return;
        const diagnostics = mergeUiAndCoreAnimationDiagnosticsV1(
          animationDiagnostics,
          [{
            schemaVersion: 1,
            code: "M2A-ANIMATION-CORE-VALIDATION-UNAVAILABLE",
            path: "animationAuthoring",
            level: "BLOCKING",
            message: error instanceof Error ? error.message : String(error),
            action: "Retry after the canonical WASM worker is available.",
          }],
        );
        dispatch({
          type: "ANIMATION_MAPPING_VALIDATED",
          revision: session.revision,
          authoringRevision: mapping.value.authoringRevision,
          status: "BLOCKED",
          diagnostics,
        });
      });
  }, [
    animationDiagnostics,
    session.animationMapping,
    session.currentStep,
    session.revision,
  ]);

  useEffect(() => {
    const mapping = session.animationMapping;
    if (!mapping || mapping.revision !== session.revision || !session.source?.sha256) {
      setAnimationMappingSaveState({ kind: "IDLE" });
      return;
    }
    setAnimationMappingSaveState({ kind: "SAVING" });
    const saved = saveCreatureAnimationDraftV1(
      session.source.sha256,
      mapping.value,
    );
    setAnimationMappingSaveState(saved.kind === "SAVED"
      ? { kind: "SAVED" }
      : { kind: "ERROR", message: saved.message });
  }, [session.animationMapping, session.revision, session.source?.sha256]);

  const continueFromInspect = () => {
    const current = sessionRef.current;
    if (current.target !== "CREATURE") {
      dispatch({ type: "CONTINUE_TO_BUILD" });
      return;
    }
    dispatch({ type: "CONTINUE_TO_ANIMATION_MAPPING" });
    if (!current.source?.sha256) return;
    const loaded = loadCreatureAnimationDraftV1(current.source.sha256);
    if (
      loaded.kind === "LOADED"
      && loaded.value.sourceRevision === current.source.sha256
    ) {
      dispatch({
        type: "ANIMATION_MAPPING_INITIALIZED",
        revision: current.revision,
        authoring: loaded.value,
      });
    } else if (loaded.kind === "ERROR") {
      setAnimationMappingSaveState({
        kind: "ERROR",
        message: loaded.diagnostics[0]?.message ?? "Animation draft could not be loaded.",
      });
    }
  };

  const applySafeAnimationSuggestions = () => {
    const mapping = sessionRef.current.animationMapping;
    if (!mapping) return;
    const proposal = proposeCreatureAnimationMappingV1(
      getAuroraAnimationStateCatalogV1(),
      animationInspection.sourceClips,
    );
    const applied = applyHighConfidenceAssignmentsV1(mapping.value, proposal);
    const existing = new Map(
      mapping.value.assignments.map((assignment) => [assignment.targetSlot, assignment]),
    );
    applied.assignments.forEach((assignment) => {
      if (existing.has(assignment.targetSlot)) return;
      dispatch({ type: "ANIMATION_SOURCE_ASSIGNED", assignment });
    });
  };
  const useGeneratedAnimationProfile = () => {
    const mapping = sessionRef.current.animationMapping;
    if (!mapping) return;
    getAuroraAnimationStateCatalogV1().forEach(({ slot }) => {
      dispatch({
        type: "ANIMATION_SOURCE_ASSIGNED",
        assignment: {
          targetSlot: slot,
          sourceKind: "PROCEDURAL",
          sourceClipName: null,
          customAnimationId: null,
          provenance: {
            provider: "PROCEDURAL_GENERATOR",
            assetId: "M2A_PROCEDURAL_HUMANOID_42_V1",
            ownership: "PROJECT_GENERATED",
          },
        },
      });
    });
  };
  const sourceMetrics = sourceInspection ? {
    meshCount: sourceInspection.inventory.meshCount,
    vertexCount: sourceInspection.statistics.vertexCount,
    triangleCount: sourceInspection.statistics.triangleCount,
    materialCount: sourceInspection.inventory.materialCount,
    textureCount: sourceInspection.inventory.textureCount,
    boneCount: sourceInspection.boneCount,
    animationClipCount: sourceInspection.inventory.animationCount,
  } : {};
  const buildInputs = session.target !== "TILE"
    && session.source?.sha256
    && session.appearance?.sha256
    ? {
        source: {
          name: session.source.name,
          byteLength: session.source.size,
          sha256: session.source.sha256,
          inspectionStatus: "INSPECTED" as const,
        },
        appearance: {
          name: session.appearance.name,
          byteLength: session.appearance.size,
          sha256: session.appearance.sha256,
          inspectionStatus: "INSPECTED" as const,
        },
      }
    : undefined;
  const buildFailure = session.build.kind === "FAILED" ? session.build.failure : undefined;
  const buildStepState: BuildStepState = session.build.kind === "RUNNING"
    ? {
        kind: "RUNNING",
        message: "The local Worker is executing one atomic canonical build. No percentage or intra-Worker stage telemetry is available.",
      }
    : session.build.kind === "FAILED"
      ? {
          kind: "FAILED",
          failure: session.build.failure,
        }
      : {
          kind: "IDLE",
          message: session.build.kind === "SUCCEEDED"
            ? "The previous build completed. Rebuild will replace the current result."
            : session.target === "TILE"
              ? "The inspected GLB and tile authoring contract are ready for the local canonical build."
              : "Both inspected inputs are ready for the local canonical build.",
          ...(buildInputs ? { inputs: buildInputs } : {}),
        };
  const currentResult = session.result?.revision === session.revision
    ? session.result.value
    : undefined;
  const currentProjectIdentity = projectBuildIdentityV1(project);
  const currentDownloadArtifacts = currentResult?.kind === "MODEL"
    ? currentResult.canonical.artifacts
    : currentResult?.kind === "PLACEABLE"
      ? currentResult.placeable.artifacts
      : currentResult?.kind === "TILE"
        ? currentResult.tile.artifacts
        : undefined;
  const currentOfflineReconciled = currentResult
    ? currentResult.readback.validation?.status === "PASS"
      && (
        currentResult.kind !== "MODEL"
        || (
          currentResult.canonical.conversionEvidence.conversionEligible
          && currentResult.canonical.packageAssemblyEvidence.strictReconciled
          && currentResult.canonical.semanticEvidence.semanticDiff.length === 0
          && (
            currentResult.canonical.geometry.deformation !== "SKIN"
            || currentResult.canonical.animationMappingEvidence?.conformance.status === "READY"
            || (
              !currentResult.canonical.animationMappingEvidence
              && currentResult.canonical.animationCompletenessEvidence?.complete === true
            )
          )
          && (
            !currentResult.canonical.animationCompletenessEvidence
            || currentResult.canonical.animationCompletenessEvidence.complete
          )
          && (
            !currentResult.canonical.animationBehaviorEvidence
            || currentResult.canonical.animationBehaviorEvidence.behaviorCandidateEligible
          )
          && (
            !currentResult.canonical.skinAnimationEvidence
            || currentResult.canonical.skinAnimationEvidence.complete
          )
          && (
            !currentResult.animationStudioReconciliation
            || getAnimationStudioDownloadGateV1(
              currentResult.animationStudioReconciliation,
            ).allowed
          )
          && (
            project.animationWorkbench.heldWeapon === null
              ? currentResult.canonical.heldWeaponEvidence === undefined
              : currentResult.canonical.heldWeaponEvidence?.status === "MATCH"
                && currentResult.canonical.heldWeaponEvidence.sourceSha256
                  === project.animationWorkbench.heldWeapon.source.sha256
          )
        )
      )
    : false;
  const currentDownloadInputs: DownloadManifestInputIdentityV1[] = [];
  if (session.source?.sha256) {
    currentDownloadInputs.push({
      role: "SOURCE_GLB",
      fileName: session.source.name,
      byteLength: session.source.size,
      sha256: session.source.sha256,
    });
  }
  if (session.appearance?.sha256) {
    currentDownloadInputs.push({
      role: session.target === "PLACEABLE"
        ? "BASE_PLACEABLES_2DA"
        : "BASE_APPEARANCE_2DA",
      fileName: session.appearance.name,
      byteLength: session.appearance.size,
      sha256: session.appearance.sha256,
    });
  }
  if (session.animationEvents?.sha256) {
    currentDownloadInputs.push({
      role: "ANIMATION_EVENTS_JSON",
      fileName: session.animationEvents.name,
      byteLength: session.animationEvents.size,
      sha256: session.animationEvents.sha256,
    });
  }
  if (
    currentResult?.kind === "MODEL"
    && currentResult.canonical.animationMappingEvidence
  ) {
    currentDownloadInputs.push({
      role: "ANIMATION_MAPPING_V2",
      fileName: "animation-mapping-v2.json",
      byteLength: null,
      sha256:
        currentResult.canonical.animationMappingEvidence
          .authoringFingerprintSha256,
    });
  }
  if (
    currentResult?.kind === "MODEL"
    && currentResult.animationStudioFingerprintSha256
  ) {
    currentDownloadInputs.push({
      role: "ANIMATION_STUDIO_V1",
      fileName: "animation-studio-v1.json",
      byteLength: null,
      sha256: currentResult.animationStudioFingerprintSha256,
    });
  }
  if (
    currentResult?.kind === "MODEL"
    && currentResult.canonical.heldWeaponEvidence
    && project.animationWorkbench.heldWeapon
  ) {
    currentDownloadInputs.push({
      role: "HELD_WEAPON_GLB",
      fileName: project.animationWorkbench.heldWeapon.source.name,
      byteLength: project.animationWorkbench.heldWeapon.source.byteLength,
      sha256: currentResult.canonical.heldWeaponEvidence.sourceSha256,
    });
  }
  if (
    currentResult?.kind === "PLACEABLE"
    && currentResult.placeable.authoring
  ) {
    currentDownloadInputs.push({
      role: "PLACEABLE_AUTHORING_V1",
      fileName: "placeable-authoring-v1.json",
      byteLength: null,
      sha256: currentResult.placeable.authoring.authoringSha256,
    });
  }
  const currentDownloadManifest = currentResult && currentDownloadArtifacts
    ? createDownloadManifestV1({
        projectIdentity: currentResult.projectIdentity,
        target: currentResult.kind === "MODEL"
          ? "CREATURE"
          : currentResult.kind,
        inputs: currentDownloadInputs,
        artifacts: currentDownloadArtifacts,
        offlineReconciled: currentOfflineReconciled,
      })
    : undefined;
  const currentDownloadReadiness = currentResult
    ? projectDownloadReadinessV1({
        currentProjectIdentity,
        builtProjectIdentity: currentResult.projectIdentity,
        offlineReconciled: currentOfflineReconciled,
      })
    : undefined;

  const inputs = (
    <InputsPanel
      target={session.target}
      tileTargetEnabled={tileTargetEnabled}
      tileOptions={tileOptions}
      creatureProfile={creatureProfile}
      textureArtifactCleanup={textureArtifactCleanup}
      skinAccessoryStabilizationMode={skinAccessoryStabilizationMode}
      skinAccessorySelectedBoneName={skinAccessorySelectedBoneName}
      source={session.source?.file}
      appearance={session.appearance?.file}
      animationEvents={session.animationEvents?.file}
      sourceIdentity={sourceIdentity}
      appearanceIdentity={appearanceIdentity}
      sourceError={sourceError}
      appearanceError={appearanceError}
      animationEventsError={animationEventsError}
      onSelectSource={selectSource}
      onSelectAppearance={selectAppearance}
      onSelectAnimationEvents={selectAnimationEvents}
      onCreatureProfileChange={updateCreatureProfile}
      onTextureArtifactCleanupChange={updateTextureArtifactCleanup}
      onSkinAccessoryStabilizationModeChange={updateSkinAccessoryStabilizationMode}
      onSkinAccessorySelectedBoneNameChange={updateSkinAccessorySelectedBoneName}
      onRemoveSource={removeSource}
      onRemoveAppearance={removeAppearance}
      onRemoveAnimationEvents={removeAnimationEvents}
      onClear={clearFiles}
      onTargetChange={selectTarget}
      onTileOptionsChange={updateTileOptions}
    />
  );

  const requirements = (
    <aside className="panel requirements-panel" aria-labelledby="input-requirements-heading">
      <header className="panel__header"><h2 id="input-requirements-heading">Input Requirements</h2></header>
      <ul className="requirements-panel__list">
        <li><span>Meshy GLB</span><strong data-ready={Boolean(session.source)}>{session.source ? "selected" : "required"}</strong></li>
        <li>
          <span>Base appearance.2da / placeables.2da</span>
          <strong data-ready={session.target === "TILE" || Boolean(session.appearance)}>
            {session.target === "TILE" ? "not required" : session.appearance ? "selected" : "required"}
          </strong>
        </li>
        <li>
          <span>42-state creature event JSON</span>
          <strong data-ready="true">
            {session.target === "CREATURE" && session.animationEvents ? "selected" : session.target === "CREATURE" ? "optional" : "not applicable"}
          </strong>
        </li>
        {session.target === "TILE" ? (
          <li><span>Tile footprint / surface</span><strong data-ready="true">10×10 m / {tileOptions.surface}</strong></li>
        ) : null}
        <li><span>Local processing</span><strong data-ready="true">enabled</strong></li>
        <li><span>Files are not uploaded</span><strong data-ready="true">private</strong></li>
      </ul>
    </aside>
  );

  return (
    <>
      <StudioShell
      header={(
        <StudioHeader
          version="v0.1.0"
          environment="Offline project"
          context={studioHeaderContext(session.currentStep, session.target)}
          theme="dark"
          project={{
            name: project.identity.name,
            target: studioTargetLabel(session.target),
            sourceSha256: session.source?.sha256 ?? project.files.sourceGlb?.sha256,
            persistence: projectPersistence,
          }}
          onProjectMenu={() => {
            setProjectManagerOpen(true);
            setProjectError(undefined);
          }}
        />
      )}
      workflow={(
        <WorkflowStepper
          steps={workflowSteps}
          currentStep={session.currentStep}
          visitedSteps={unlockedSteps}
          completedSteps={completedSteps}
          blockedSteps={blockedSteps}
          onStepSelect={(step) => dispatch({ type: "NAVIGATE", step })}
        />
      )}
      inputs={showMeshyLab || session.currentStep === "SOURCE" ? null : inputs}
      aside={!showMeshyLab && session.currentStep === "SOURCE" ? requirements : undefined}
      expandPrimaryToWorkspace={
        showMeshyLab || session.currentStep === "ANIMATION_MAPPING"
      }
      workspaceMode={showMeshyLab}
      workflowRail={!showMeshyLab}
      workflowLabel={`${studioTargetLabel(session.target)} workflow`}
      debugDrawer={showMeshyLab || session.currentStep === "ANIMATION_MAPPING" ? null : (
        <section className="debug-drawer-placeholder" aria-label="Debug Drawer">
          <strong>Debug Drawer</strong>
          <span>{debugDrawerMessage ?? (session.currentStep === "SOURCE" ? "Disabled until inspection begins" : "Collapsed")}</span>
        </section>
      )}
    >
      <Suspense fallback={(
        <section className="empty-state" role="status" aria-label="Loading feature">
          <strong>Loading local workspace</strong>
          <span>The feature bundle stays in this browser.</span>
        </section>
      )}>
      {showMeshyLab ? (
        <MeshyLab
          bridge={meshyBridgeRef.current}
          onBack={() => setShowMeshyLab(false)}
          onImport={(file, provenance) => {
            selectSource(file, provenance);
            setShowMeshyLab(false);
          }}
          onImportAnimation={(file, provenance, actionId) => {
            const id = `${provenance.sha256}:${actionId}`;
            setMeshyAnimationDonors((current) => [
              ...current.filter((donor) => donor.id !== id),
              {
                id,
                file,
                label: `Meshy action ${actionId}`,
                detail: `${file.name} · SHA-256 ${provenance.sha256.slice(0, 12)}…`,
              },
            ]);
            setDebugDrawerMessage(
              `Meshy action ${actionId} is available in Animation Studio.`,
            );
          }}
        />
      ) : session.currentStep === "SOURCE" ? (
        <SourceStep
          target={session.target}
          tileTargetEnabled={tileTargetEnabled}
          tileOptions={tileOptions}
          creatureProfile={creatureProfile}
          textureArtifactCleanup={textureArtifactCleanup}
          skinAccessoryStabilizationMode={skinAccessoryStabilizationMode}
          skinAccessorySelectedBoneName={skinAccessorySelectedBoneName}
          source={session.source?.file}
          appearance={session.appearance?.file}
          animationEvents={session.animationEvents?.file}
          sourceIdentity={sourceIdentity}
          appearanceIdentity={appearanceIdentity}
          sourceError={sourceError}
          appearanceError={appearanceError}
          animationEventsError={animationEventsError}
          onSelectSource={selectSource}
          onSelectAppearance={selectAppearance}
          onSelectAnimationEvents={selectAnimationEvents}
          onCreatureProfileChange={updateCreatureProfile}
          onTextureArtifactCleanupChange={updateTextureArtifactCleanup}
          onSkinAccessoryStabilizationModeChange={updateSkinAccessoryStabilizationMode}
          onSkinAccessorySelectedBoneNameChange={updateSkinAccessorySelectedBoneName}
          onRemoveSource={removeSource}
          onRemoveAppearance={removeAppearance}
          onRemoveAnimationEvents={removeAnimationEvents}
          onClear={clearFiles}
          onTargetChange={selectTarget}
          onTileOptionsChange={updateTileOptions}
          onContinue={() => dispatch({ type: "CONTINUE_TO_INSPECT" })}
          onOpenMeshyLab={meshyLabEnabled ? () => setShowMeshyLab(true) : undefined}
          meshyProvenance={meshyProvenance}
          visualQaFixture={visualQaFixture ? {
            label: visualQaFixture.label,
            state: visualQaFixtureState,
            onLoad: () => {
              void loadVisualQaFixture();
            },
          } : undefined}
        />
      ) : session.currentStep === "INSPECT" ? (
        <InspectStep
          viewport={session.source && session.source.sha256 ? (
            session.target === "PLACEABLE" && placeableAuthoring ? (
              <PlaceableAuthoringEditor
                key={`${session.source.sha256}:${placeableAuthoring.document.sourceSha256}`}
                file={session.source.file}
                sourceSha256={session.source.sha256}
                bootstrap={placeableAuthoring}
                onDocumentChange={(document) => {
                  setPlaceableAuthoring((current) => current
                    ? { ...current, document }
                    : current);
                  dispatch({ type: "AUTHORING_DOCUMENT_CHANGED" });
                }}
                onError={setSourceError}
              />
            ) : (
              <SourceViewport
                input={{
                  provenance: "SOURCE",
                  file: session.source.file,
                  sourceSha256: session.source.sha256,
                }}
                onError={setSourceError}
              />
            )
          ) : (
            <div className="empty-state">
              <strong>Preparing local source inspection</strong>
              <span>{sourceError ?? "No upload occurs."}</span>
            </div>
          )}
          sourceMetrics={sourceMetrics}
          profileRequirements={session.target === "CREATURE"
            ? creatureH1ProfileRequirements(sourceInspection)
            : undefined}
          validationChecks={[
            ...sourceValidationChecks(sourceInspection),
            ...(session.target === "TILE" ? [] : appearanceValidationChecks(appearanceInspection)),
          ]}
          canContinue={
            sourceInspection?.conversionEligible === true
            && (session.target === "TILE" || Boolean(appearanceInspection))
          }
          continueLabel={session.target === "CREATURE"
            ? "Continue to Animation Mapping"
            : "Continue to Build"}
          wideViewport={session.target === "PLACEABLE" && Boolean(placeableAuthoring)}
          onBack={() => dispatch({ type: "NAVIGATE", step: "SOURCE" })}
          onContinue={continueFromInspect}
        />
      ) : session.currentStep === "ANIMATION_MAPPING"
        && session.animationMapping ? (
        <CreatureAnimationMappingStep
          authoring={session.animationMapping.value}
          inspection={animationInspection}
          saveState={animationMappingSaveState}
          canContinue={
            canContinueFromAnimationMapping(session)
            && animationStudioReadyForBuild
          }
          validationStatus={session.animationMappingValidation?.value.status}
          canonicalValidationPending={
            animationMappingStatus === "READY"
            && !/^[0-9a-f]{64}$/.test(
              session.animationMappingValidation?.value
                .authoringFingerprintSha256 ?? "",
            )
          }
          onBack={() => dispatch({ type: "NAVIGATE", step: "INSPECT" })}
          onContinue={() => {
            if (animationStudioReadyForBuild) {
              dispatch({ type: "CONTINUE_TO_BUILD" });
            }
          }}
          onApplySuggestions={applySafeAnimationSuggestions}
          canUseGeneratedProfile={sourceInspection
            ? canOfferGeneratedHumanoidProfileV1({
                skinCount: sourceInspection.inventory.skinCount,
                boneCount: sourceInspection.boneCount,
                clips: sourceInspection.clips,
              })
            : false}
          onUseGeneratedProfile={useGeneratedAnimationProfile}
          onAuthoringEvent={(event) => dispatch(event)}
          sourcePreviewInput={session.source?.sha256 ? {
            provenance: "SOURCE",
            file: session.source.file,
            sourceSha256: session.source.sha256,
          } : undefined}
          readback={currentResult?.kind === "MODEL" ? currentResult.readback : undefined}
          builtAuthoringRevision={
            currentResult?.kind === "MODEL"
              ? currentResult.animationBuildInput.authoringRevision
              : undefined
          }
          onPreviewError={setSourceError}
          animationMode={
            animationStudioState?.mode === "CREATE_EDIT" ? "EDIT" : "MAP"
          }
          onAnimationModeChange={(mode: AnimationMappingModeV1) => {
            setAnimationStudioState((current) => current
              ? reduceAnimationStudioStateV1(current, {
                  type: "ANIMATION_STUDIO_MODE_SELECTED",
                  mode: mode === "EDIT" ? "CREATE_EDIT" : "MAP_BASE_42",
                })
              : current);
          }}
          animationStudio={animationStudioState && animationAuthoringV2
            && animationStudioSourceFile && animationStudioSourceRevision ? (
            <AnimationStudioWorkspace
              document={animationStudioState.document}
              authoring={animationAuthoringV2}
              sourceInventory={animationInspection.sourceClips}
              rig={editableAnimationRig}
              animationModelDonors={meshyAnimationDonors}
              viewport={(clip, playheadSeconds, playback, transform) => (
                <SourceViewport
                  input={{
                    provenance: "SOURCE",
                    file: animationStudioSourceFile,
                    sourceSha256: animationStudioSourceRevision,
                  }}
                  authoredClip={clip}
                  authoredRig={editableAnimationRig}
                  controlledAnimationTimeSeconds={playheadSeconds}
                  controlledAnimationPlayback={playback}
                  transformGizmo={transform}
                  heldWeapon={transform.heldWeapon ?? undefined}
                  motionVisualization={transform.motionVisualization}
                  onError={setSourceError}
                />
              )}
              sourceViewport={(clip, playheadSeconds, playback) => (
                <SourceViewport
                  input={{
                    provenance: "SOURCE",
                    file: animationStudioSourceFile,
                    sourceSha256: animationStudioSourceRevision,
                  }}
                  initialAnimationName={
                    clip?.source.kind === "SOURCE_CLIP_COPY"
                      ? clip.source.sourceClipName
                      : undefined
                  }
                  controlledAnimationTimeSeconds={playheadSeconds}
                  controlledAnimationPlayback={playback}
                  hideAnimationControls
                  onError={setSourceError}
                />
              )}
              autosaveState={
                animationStudioState.autosave.status === "SAVING"
                  ? { kind: "SAVING" }
                  : animationStudioState.autosave.status === "AUTOSAVED"
                    ? { kind: "SAVED" }
                    : animationStudioState.autosave.status === "ERROR"
                      ? {
                          kind: "ERROR",
                          message: animationStudioState.autosave.error
                            ?? "The local project could not be saved.",
                        }
                      : { kind: "IDLE" }
              }
              diagnostics={animationStudioDiagnostics}
              onApplyAnimationEditCommandBatch={async (document, batch) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const response = await worker.applyAnimationEditCommandBatch(
                  await animationStudioSourceFile.arrayBuffer(),
                  serializeAnimationStudioDocumentV1(document),
                  JSON.stringify(batch),
                );
                if (
                  !response.ok
                  || response.type !== "ANIMATION_EDIT_COMMAND_BATCH_APPLIED"
                ) {
                  throw new Error("The exact core rejected the animation edit command.");
                }
                return parseAnimationEditCommandResultV1(
                  JSON.parse(response.resultJson) as unknown,
                );
              }}
              onAnalyzeMotionQuality={async (clip, policy, context) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const response = await worker.analyzeAnimationMotionQuality(
                  await animationStudioSourceFile.arrayBuffer(),
                  JSON.stringify(clip),
                  JSON.stringify(policy),
                  JSON.stringify(context),
                );
                if (
                  !response.ok
                  || response.type !== "ANIMATION_MOTION_QUALITY_ANALYZED"
                ) {
                  throw new Error("The exact core could not analyze this animation.");
                }
                return parseAnimationQualityReportV1(
                  JSON.parse(response.reportJson) as unknown,
                );
              }}
              onApplyAnimationAuthoringTool={async (clip, request) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const response = await worker.applyAnimationAuthoringTool(
                  await animationStudioSourceFile.arrayBuffer(),
                  JSON.stringify(clip),
                  JSON.stringify(request),
                );
                if (
                  !response.ok
                  || response.type !== "ANIMATION_AUTHORING_TOOL_APPLIED"
                ) {
                  throw new Error("The exact core could not apply this authoring tool.");
                }
                return parseAnimationAuthoringToolResultV1(
                  JSON.parse(response.resultJson) as unknown,
                );
              }}
              onInspectHeldWeapon={async (file) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const response = await worker.inspectHeldWeaponSource(
                  await file.arrayBuffer(),
                  file.name,
                );
                if (!response.ok || response.type !== "HELD_WEAPON_SOURCE_INSPECTED") {
                  throw new Error("The exact Core rejected this held weapon source.");
                }
                return parseHeldWeaponSourceInspectionV1(
                  JSON.parse(response.inspectionJson) as unknown,
                );
              }}
              onHeldEquipmentRuntimeChange={setHeldEquipmentRuntime}
              onApplyAnimationWorkbenchOperation={async (request) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const response = await worker.applyAnimationWorkbenchOperationV1(
                  JSON.stringify(request),
                );
                if (
                  !response.ok
                  || response.type !== "ANIMATION_WORKBENCH_OPERATION_V1_APPLIED"
                ) {
                  throw new Error("The exact Core rejected this animation workbench operation.");
                }
                return parseAnimationWorkbenchResultV1(response.resultJson);
              }}
              animationWorkbenchProjectState={project.animationWorkbench}
              onAnimationWorkbenchProjectStateChange={(animationWorkbench) => {
                const current = projectRef.current;
                const revised = reviseMeshy2AuroraProjectV1(current, { animationWorkbench });
                projectRef.current = revised;
                setProject(revised);
                setProjectPersistence("DIRTY");
              }}
              onDocumentChange={(document) => {
                setAnimationStudioState((current) => current
                  ? commitAnimationStudioDocumentV1(current, document)
                  : current);
                dispatch({ type: "AUTHORING_DOCUMENT_CHANGED" });
              }}
              onAuthoringChange={(authoring) => {
                setAnimationAuthoringV2(authoring);
                setAnimationAuthoringV2Dirty(true);
                dispatch({ type: "AUTHORING_DOCUMENT_CHANGED" });
              }}
              onEditSourceClip={async (sourceClipId, newId, newName) => {
                const sourceClip = animationInspection.sourceClips.find(
                  ({ clipId }) => clipId === sourceClipId,
                );
                const worker = workerRef.current;
                if (!sourceClip || !worker) {
                  throw new Error("The selected source clip is unavailable.");
                }
                const response = await worker.inspectEditableAnimationSource(
                  await animationStudioSourceFile.arrayBuffer(),
                  sourceClip.name,
                );
                if (
                  !response.ok
                  || response.type !== "EDITABLE_ANIMATION_SOURCE_INSPECTED"
                ) {
                  throw new Error("The source clip could not be projected to the output rig.");
                }
                const projected = parseEditableAnimationSourceV1(
                  response.inspectionJson,
                ).clip;
                if (!projected) {
                  throw new Error(`No editable tracks were returned for ${sourceClip.name}.`);
                }
                return {
                  ...projected,
                  id: newId,
                  name: newName,
                  status: "DRAFT",
                  source: {
                    ...projected.source,
                    kind: "SOURCE_CLIP_COPY",
                    sourceRevision: animationStudioSourceRevision,
                    sourceClipName: sourceClip.name,
                  },
                  revision: 1,
                };
              }}
              onInspectAnimationModel={async (
                file,
                overrides = [],
                manualMappingConfirmed = false,
              ) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const response = await worker.inspectEditableAnimationSource(
                  await file.arrayBuffer(),
                );
                if (
                  !response.ok
                  || response.type !== "EDITABLE_ANIMATION_SOURCE_INSPECTED"
                ) {
                  throw new Error("The donor GLB could not be inspected.");
                }
                const parsed = parseEditableAnimationSourceV1(response.inspectionJson);
                const compatibilityResponse = await worker
                  .inspectAnimationTransferCompatibility(
                    await animationStudioSourceFile.arrayBuffer(),
                    await file.arrayBuffer(),
                  );
                if (
                  !compatibilityResponse.ok
                  || compatibilityResponse.type
                    !== "ANIMATION_TRANSFER_COMPATIBILITY_INSPECTED"
                ) {
                  throw new Error("The donor/target rig compatibility could not be inspected.");
                }
                const transferCompatibility = parseAnimationTransferCompatibilityV1(
                  compatibilityResponse.compatibilityJson,
                );
                const semanticResponse = await worker
                  .inspectHumanoidRetargetCompatibilityV2(
                    await animationStudioSourceFile.arrayBuffer(),
                    await file.arrayBuffer(),
                    JSON.stringify(overrides),
                    manualMappingConfirmed,
                  );
                if (
                  !semanticResponse.ok
                  || semanticResponse.type
                    !== "HUMANOID_RETARGET_COMPATIBILITY_V2_INSPECTED"
                ) {
                  throw new Error(
                    "The donor/target humanoid semantic mapping could not be inspected.",
                  );
                }
                const semanticCompatibility = parseHumanoidSemanticBoneMapV2(
                  semanticResponse.compatibilityJson,
                );
                if (
                  transferCompatibility.donorSourceRevision !== parsed.sourceRevision
                  || transferCompatibility.targetSourceRevision
                    !== animationStudioSourceRevision
                  || semanticCompatibility.donorSourceRevision !== parsed.sourceRevision
                  || semanticCompatibility.targetSourceRevision
                    !== animationStudioSourceRevision
                ) {
                  throw new Error(
                    "Core compatibility report does not match the exact donor/target lineage.",
                  );
                }
                return {
                  sourceRevision: parsed.sourceRevision,
                  rig: parsed.rig,
                  clips: parsed.clips,
                  transferCompatibility,
                  semanticCompatibility,
                };
              }}
              onImportAnimationModelClip={async (
                file,
                clipName,
                donorSourceRevision,
                mode: AnimationTransferModeV1,
                semanticMap: HumanoidSemanticBoneMapV2 | undefined,
                newId,
                newName,
              ) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const parsed = mode === "HUMANOID_SEMANTIC_RETARGET_V2"
                  ? await (async () => {
                      if (!semanticMap || semanticMap.status !== "COMPATIBLE") {
                        throw new Error(
                          "Semantic retarget requires a compatible, source-bound Core map.",
                        );
                      }
                      const response = await worker.retargetAnimationClipHumanoidV2(
                        await animationStudioSourceFile.arrayBuffer(),
                        await file.arrayBuffer(),
                        clipName,
                        JSON.stringify(semanticMap),
                        newId,
                        newName,
                      );
                      if (
                        !response.ok
                        || response.type !== "ANIMATION_CLIP_HUMANOID_V2_RETARGETED"
                      ) {
                        throw new Error(
                          `The donor clip ${clipName} could not be semantically retargeted.`,
                        );
                      }
                      return parseAnimationTransferResultV1(response.resultJson);
                    })()
                  : await (async () => {
                      const response = await worker.retargetAnimationModelClip(
                        await animationStudioSourceFile.arrayBuffer(),
                        await file.arrayBuffer(),
                        clipName,
                        JSON.stringify({
                          mode,
                          clipId: newId,
                          outputName: newName,
                        }),
                      );
                      if (
                        !response.ok
                        || response.type !== "ANIMATION_MODEL_CLIP_RETARGETED"
                      ) {
                        throw new Error(
                          `The donor clip ${clipName} could not be transferred.`,
                        );
                      }
                      return parseAnimationTransferResultV1(response.resultJson);
                    })();
                if (
                  parsed.compatibility.donorSourceRevision !== donorSourceRevision
                  || parsed.compatibility.targetSourceRevision
                    !== animationStudioSourceRevision
                ) {
                  throw new Error(
                    "The donor or target GLB changed after inspection; choose the exact models again.",
                  );
                }
                const requiredSchema = mode === "EXACT_RIG_COPY_V1" ? 1 : 3;
                if (parsed.requiredDocumentSchemaVersion !== requiredSchema) {
                  throw new Error(
                    "Core returned an unexpected document schema requirement.",
                  );
                }
                return parsed.clip;
              }}
              onPrepareAnimationModelBatch={async (
                file,
                inspection,
                clipNames,
                mode,
                semanticMap,
              ) => {
                const worker = workerRef.current;
                if (!worker) throw new Error("The exact Animation Studio core is unavailable.");
                const usedNames = new Set(
                  (animationStudioStateRef.current?.document.authoredClips ?? [])
                    .map(({ name }) => name.toLowerCase()),
                );
                const clips = clipNames.map((donorClipName, index) => {
                  const slug = donorClipName.replace(/[^a-z0-9]+/giu, "_").replace(/^_+|_+$/g, "").slice(0, 10) || "motion";
                  let outputName = `b${index + 1}_${slug}`.slice(0, 16);
                  let suffix = 2;
                  while (usedNames.has(outputName.toLowerCase())) {
                    const ending = `_${suffix++}`;
                    outputName = `${`b${index + 1}_${slug}`.slice(0, 16 - ending.length)}${ending}`;
                  }
                  usedNames.add(outputName.toLowerCase());
                  return {
                    donorClipName,
                    mode,
                    clipId: `batch-${crypto.randomUUID()}`,
                    outputName,
                  };
                });
                const response = await worker.prepareAnimationTransferBatchV2(
                  await animationStudioSourceFile.arrayBuffer(),
                  await file.arrayBuffer(),
                  JSON.stringify({
                    schemaVersion: 2,
                    commit: "ALL_OR_NOTHING",
                    donorSourceRevision: inspection.sourceRevision,
                    targetSourceRevision: animationStudioSourceRevision,
                    clips,
                    semanticMap: mode === "HUMANOID_SEMANTIC_RETARGET_V2"
                      ? semanticMap ?? null
                      : null,
                  }),
                );
                if (!response.ok || response.type !== "ANIMATION_TRANSFER_BATCH_V2_PREPARED") {
                  throw new Error("The exact Core could not prepare this animation batch.");
                }
                return parseAnimationTransferBatchResultV2(response.resultJson);
              }}
              onBuildAnimationSequencePreview={async (request, availableClips) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const response = await worker.buildAnimationSequencePreview(
                  JSON.stringify(request),
                  JSON.stringify(availableClips),
                );
                if (
                  !response.ok
                  || response.type !== "ANIMATION_SEQUENCE_PREVIEW_BUILT"
                ) {
                  throw new Error("The animation sequence preview could not be built.");
                }
                return parseAnimationSequencePreviewV1(response.previewJson);
              }}
              onResampleAnimationCurve={async (curve, policy) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const response = await worker.resampleAnimationCurveToLinear(
                  JSON.stringify(curve),
                  JSON.stringify(policy),
                );
                if (
                  !response.ok
                  || response.type !== "ANIMATION_CURVE_RESAMPLED_TO_LINEAR"
                ) {
                  throw new Error("The selected animation curve could not be resampled.");
                }
                return parseAnimationCurveResampleReportV1(response.reportJson);
              }}
              onBakeAnimationLayers={async (layers, rigWire) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const response = await worker.bakeAnimationLayers(
                  JSON.stringify(layers),
                  JSON.stringify(rigWire),
                );
                if (!response.ok || response.type !== "ANIMATION_LAYERS_BAKED") {
                  throw new Error("The animation correction layers could not be baked.");
                }
                return parseAnimationLayerBakeReportV1(response.reportJson);
              }}
              renderAnimationTransferPreview={(
                clip,
                donorFile,
                donorClipName,
                donorSourceRevision,
              ) => (
                <div className="animation-import-dialog__preview-comparison">
                  <section aria-label="Donor animation preview">
                    <strong>Donor · {donorClipName}</strong>
                    <SourceViewport
                      input={{
                        provenance: "SOURCE",
                        file: donorFile,
                        sourceSha256: donorSourceRevision,
                      }}
                      initialAnimationName={donorClipName}
                      initialAnimationLoop
                      hideAnimationControls={false}
                      onError={setSourceError}
                    />
                  </section>
                  <section aria-label="Retargeted animation preview">
                    <strong>Current model · {clip.name}</strong>
                    <SourceViewport
                      input={{
                        provenance: "SOURCE",
                        file: animationStudioSourceFile,
                        sourceSha256: animationStudioSourceRevision,
                      }}
                      authoredClip={clip}
                      authoredRig={editableAnimationRig}
                      hideAnimationControls={false}
                      onError={setSourceError}
                    />
                  </section>
                </div>
              )}
              onValidateClip={async (candidate, prospectiveDocument) => {
                const worker = workerRef.current;
                if (!worker) {
                  throw new Error("The exact Animation Studio core is unavailable.");
                }
                const isolatedDocument: AnimationStudioDocumentV1 = {
                  ...prospectiveDocument,
                  status: "VALID",
                  authoredClips: [{ ...candidate, status: "VALID" }],
                };
                const response = await worker.validateAnimationStudioDocument(
                  await animationStudioSourceFile.arrayBuffer(),
                  serializeAnimationStudioDocumentV1(isolatedDocument),
                );
                if (
                  !response.ok
                  || response.type !== "ANIMATION_STUDIO_DOCUMENT_VALIDATED"
                ) {
                  throw new Error(
                    "The exact Animation Studio core could not validate this clip.",
                  );
                }
                const parsed = JSON.parse(response.validationJson) as {
                  diagnostics?: AnimationStudioDiagnosticV1[];
                } | AnimationStudioDiagnosticV1[];
                const coreDiagnostics = Array.isArray(parsed)
                  ? parsed
                  : parsed.diagnostics ?? [];
                return [
                  ...validateAnimationStudioSchemaV1(prospectiveDocument),
                  ...coreDiagnostics,
                ];
              }}
              onInspectLibraryPreset={async (preset) => {
                const worker = workerRef.current;
                if (!worker) throw new Error("The exact Animation Studio core is unavailable.");
                const { inspectAnimationLibraryPresetV1 } = await import(
                  "./features/animation-library/workerBoundary"
                );
                return inspectAnimationLibraryPresetV1(
                  worker,
                  animationStudioSourceFile,
                  preset,
                );
              }}
              onInstantiateLibraryPreset={async (
                preset,
                newId,
                newName,
              ) => {
                const worker = workerRef.current;
                if (!worker) throw new Error("The exact Animation Studio core is unavailable.");
                const { instantiateAnimationLibraryPresetV1 } = await import(
                  "./features/animation-library/workerBoundary"
                );
                return instantiateAnimationLibraryPresetV1(
                  worker,
                  animationStudioSourceFile,
                  preset,
                  newId,
                  newName,
                );
              }}
              onExportAnimationContribution={async (
                clip,
                metadata,
              ) => {
                const worker = workerRef.current;
                if (!worker) throw new Error("The exact Animation Studio core is unavailable.");
                const { exportAnimationLibraryContributionV1 } = await import(
                  "./features/animation-library/workerBoundary"
                );
                return exportAnimationLibraryContributionV1(
                  worker,
                  animationStudioSourceFile,
                  clip,
                  metadata,
                );
              }}
              onUndo={() => setAnimationStudioState((current) => current
                ? undoAnimationStudioEditV1(current)
                : current)}
              onRedo={() => setAnimationStudioState((current) => current
                ? redoAnimationStudioEditV1(current)
                : current)}
              canUndo={animationStudioState.undoStack.length > 0}
              canRedo={animationStudioState.redoStack.length > 0}
              requestedClipId={animationStudioState.selectedClipId}
              onSelectedClipChange={(clipId) => {
                setAnimationStudioState((current) => current
                  ? reduceAnimationStudioStateV1(current, {
                      type: "AUTHORED_CLIP_SELECTED",
                      clipId,
                    })
                  : current);
              }}
            />
          ) : (
            <div className="empty-state" role="status">
              <strong>Preparing Animation Studio</strong>
              <span>The exact output rig and local project are loading.</span>
            </div>
          )}
          animationStudioDocument={animationStudioState?.document}
          animationAuthoringV2={animationAuthoringV2 ?? undefined}
          onAnimationAuthoringV2Change={(authoring) => {
            setAnimationAuthoringV2(authoring);
            setAnimationAuthoringV2Dirty(true);
            dispatch({ type: "AUTHORING_DOCUMENT_CHANGED" });
          }}
          onCreateAuthoredAnimation={() => {
            setAnimationStudioState((current) => {
              if (!current) return current;
              const created = createBlankPoseClipV1({
                id: crypto.randomUUID(),
                name: `custom_animation_${current.document.authoredClips.length + 1}`,
                sourceRevision: current.document.sourceRevision,
                animationRoot: editableAnimationRig.find(({ parentId }) => parentId === null)
                  ?.name ?? editableAnimationRig[0]?.name ?? "root",
                rig: editableAnimationRig,
              });
              const committed = commitAnimationStudioDocumentV1(current, {
                ...current.document,
                status: "DRAFT",
                authoredClips: [...current.document.authoredClips, created],
              });
              return {
                ...committed,
                mode: "CREATE_EDIT",
                selectedClipId: created.id,
              };
            });
            dispatch({ type: "AUTHORING_DOCUMENT_CHANGED" });
          }}
          onOpenAuthoredAnimation={(clipId) => {
            setAnimationStudioState((current) => {
              if (!current) return current;
              const selected = reduceAnimationStudioStateV1(current, {
                type: "AUTHORED_CLIP_SELECTED",
                clipId,
              });
              return reduceAnimationStudioStateV1(selected, {
                type: "ANIMATION_STUDIO_MODE_SELECTED",
                mode: "CREATE_EDIT",
              });
            });
          }}
        />
      ) : session.currentStep === "BUILD" ? (
        <BuildStep
          state={buildStepState}
          backLabel={session.target === "CREATURE" ? "Back to Animation Mapping" : "Back to Inspect"}
          projectIdentity={{
            projectId: project.identity.projectId,
            revision: project.revision,
          }}
          canGoBack={session.build.kind !== "RUNNING"}
          canBuild={
            session.build.kind !== "RUNNING"
            && Boolean(sourceInspection)
            && (session.target === "TILE" || Boolean(appearanceInspection))
            && (session.target !== "CREATURE" || animationStudioReadyForBuild)
          }
          canRetry={session.build.kind === "FAILED"}
          canCancel={session.build.kind === "RUNNING"}
          onBack={() => dispatch({
            type: "NAVIGATE",
            step: session.target === "CREATURE" ? "ANIMATION_MAPPING" : "INSPECT",
          })}
          onBuild={startBuild}
          onRetry={startBuild}
          onCancel={cancelBuild}
          failureDiagnostics={buildFailure ? {
            canOpen: true,
            onOpen: () => setDebugDrawerMessage([
              "Build failure",
              buildFailure.code,
              buildFailure.stage,
              buildFailure.path,
              buildFailure.message,
            ].filter(Boolean).join(" · ")),
            reportPackage: {
              status: "UNAVAILABLE",
              reason: "The FE-D7 canonical debug report contract has not produced a report package for this failure.",
            },
          } : undefined}
        />
      ) : session.currentStep === "REVIEW" && currentResult?.kind === "PLACEABLE" ? (
        <>
          <ProjectBuildIdentity identity={currentResult.projectIdentity} />
          <PlaceableReview result={currentResult.placeable} readback={currentResult.readback} />
          <ReviewWorkflowActions
            canContinue={currentDownloadReadiness?.allowed === true}
            blockedReason={currentDownloadReadiness?.reason}
            onBack={() => dispatch({ type: "NAVIGATE", step: "BUILD" })}
            onContinue={() => dispatch({ type: "CONTINUE_TO_DOWNLOAD" })}
          />
        </>
      ) : session.currentStep === "REVIEW" && currentResult?.kind === "TILE" ? (
        <>
          <ProjectBuildIdentity identity={currentResult.projectIdentity} />
          <TileReview result={currentResult.tile} readback={currentResult.readback} />
          <ReviewWorkflowActions
            canContinue={currentDownloadReadiness?.allowed === true}
            blockedReason={currentDownloadReadiness?.reason}
            onBack={() => dispatch({ type: "NAVIGATE", step: "BUILD" })}
            onContinue={() => dispatch({ type: "CONTINUE_TO_DOWNLOAD" })}
          />
        </>
      ) : session.currentStep === "REVIEW" && currentResult?.kind === "MODEL" && session.source?.sha256 ? (
        <>
          <ProjectBuildIdentity identity={currentResult.projectIdentity} />
          <ReviewModelDetails
            result={currentResult.canonical}
            readback={currentResult.readback}
            activeViewport={reviewViewport}
            onViewportChange={setReviewViewport}
            onInspectBinary={() => setDebugDrawerMessage(
              `Binary Inspector is scheduled for FE-V8. Current readback evidence: ${currentResult.readback.validation?.status ?? "UNAVAILABLE"}.`,
            )}
            sourceViewport={(
              <SourceViewport
                input={{
                  provenance: "SOURCE",
                  file: session.source.file,
                  sourceSha256: session.source.sha256,
                }}
                onError={setSourceError}
              />
            )}
            convertedReadbackViewport={(
              <AuroraReadbackViewport
                report={currentResult.readback}
                selectedPart={selectedReadbackPart}
                onSelectPart={setSelectedReadbackPart}
                onError={setSourceError}
              />
            )}
          />
          {currentResult.animationStudioDocument
            && currentResult.animationAuthoringV2
            && currentResult.animationStudioFingerprintSha256
            && currentResult.animationStudioReconciliation
            && currentResult.canonical.animationStudioEvidence ? (
            <AuthoredAnimationReview
              studio={currentResult.animationStudioDocument}
              authoring={currentResult.animationAuthoringV2}
              studioFingerprintSha256={
                currentResult.animationStudioFingerprintSha256
              }
              reconciliation={currentResult.animationStudioReconciliation}
              evidence={currentResult.canonical.animationStudioEvidence}
              animationPlaybackAcceptance={
                currentResult.canonical.animationPlaybackAcceptance
              }
              readback={currentResult.readback}
              onOpenMismatch={(path) => {
                const clipId = /^authoredClips\.([^.]+)/.exec(path)?.[1];
                if (clipId) {
                  setAnimationStudioState((current) => {
                    if (!current) return current;
                    const selected = reduceAnimationStudioStateV1(current, {
                      type: "AUTHORED_CLIP_SELECTED",
                      clipId,
                    });
                    return reduceAnimationStudioStateV1(selected, {
                      type: "ANIMATION_STUDIO_MODE_SELECTED",
                      mode: "CREATE_EDIT",
                    });
                  });
                }
                dispatch({ type: "NAVIGATE", step: "ANIMATION_MAPPING" });
              }}
            />
          ) : null}
          {currentResult.animationStudioReconciliation
          && !getAnimationStudioDownloadGateV1(
            currentResult.animationStudioReconciliation,
          ).allowed ? (
            <section className="panel" role="alert" aria-label="Download blocked">
              <h3>Download blocked</h3>
              <p>
                {getAnimationStudioDownloadGateV1(
                  currentResult.animationStudioReconciliation,
                ).reason}
              </p>
            </section>
          ) : null}
          <ReviewWorkflowActions
            canContinue={currentDownloadReadiness?.allowed === true}
            blockedReason={currentDownloadReadiness?.reason}
            onBack={() => dispatch({ type: "NAVIGATE", step: "BUILD" })}
            onContinue={() => dispatch({ type: "CONTINUE_TO_DOWNLOAD" })}
          />
        </>
      ) : session.currentStep === "DOWNLOAD"
        && currentResult
        && currentDownloadArtifacts
        && currentDownloadManifest
        && currentDownloadReadiness ? (
        <DownloadStep
          artifacts={currentDownloadArtifacts}
          manifest={currentDownloadManifest}
          readiness={currentDownloadReadiness}
          onBack={() => dispatch({ type: "NAVIGATE", step: "REVIEW" })}
          onError={(message) => setDebugDrawerMessage(
            `Artifact download error: ${message}`,
          )}
        />
      ) : (
        <section className="inspect-scaffold" aria-labelledby="review-unavailable-heading">
          <header className="inspect-scaffold__header">
            <div><p className="eyebrow">Review output</p><h1 id="review-unavailable-heading">Result unavailable</h1></div>
          </header>
          <div className="empty-state"><strong>No current canonical result.</strong><span>Return to Build and run the pipeline.</span></div>
        </section>
      )}
      </Suspense>
      </StudioShell>
      {projectManagerOpen ? (
        <ProjectManagerDialog
          project={project}
          persistence={projectPersistence}
          recoveryRecords={projectRecoveryRecords}
          busy={projectBusy}
          error={projectError}
          onClose={() => setProjectManagerOpen(false)}
          onRename={(name) => {
            const renamed = renameMeshy2AuroraProjectV1(projectRef.current, name);
            projectRef.current = renamed;
            setProject(renamed);
            setProjectPersistence("DIRTY");
          }}
          onNew={createNewProject}
          onExport={() => downloadProjectBackup(projectRef.current)}
          onImport={importProject}
          onDuplicate={duplicateProject}
          onDelete={deleteCurrentProject}
          onRecover={recoverProject}
          onDeleteRecovery={removeRecoveryProject}
        />
      ) : null}
    </>
  );
}

function studioTargetLabel(target: StudioTarget) {
  switch (target) {
    case "CREATURE":
      return "Creature";
    case "PLACEABLE":
      return "Placeable";
    case "TILE":
      return "Tile";
  }
}

function ProjectBuildIdentity({
  identity,
}: {
  readonly identity: ProjectBuildIdentityV1;
}) {
  return (
    <section className="panel project-build-identity" aria-label="Built project identity">
      <strong>Built from project</strong>
      <code>{identity.projectId}</code>
      <span>Revision {identity.projectRevision}</span>
    </section>
  );
}

function studioHeaderContext(step: WorkflowStep, target: StudioTarget) {
  return `${studioTargetLabel(target)} · ${workflowStepLabel(step)}`;
}
