import { useEffect, useReducer, useRef, useState } from "react";
import { StudioHeader } from "./app/StudioHeader";
import { StudioShell } from "./app/StudioShell";
import { getUnlockedWorkflowSteps, getWorkflowStepStatus } from "./app/studioSelectors";
import {
  createInitialStudioSession,
  studioSessionReducer,
  type StudioSessionEvent,
  type StudioSessionState,
  type StudioTarget,
} from "./app/studioSession";
import { WORKFLOW_STEPS } from "./app/workflow";
import { WorkflowStepper } from "./app/WorkflowStepper";
import { BuildStep, type BuildStepState } from "./features/build/BuildStep";
import { ArtifactDownloads } from "./features/downloads/ArtifactDownloads";
import {
  buildStageForPipelineStage,
  projectBuildFailure,
} from "./features/build/projectBuildFailure";
import {
  projectAppearanceInspection,
  type AppearanceInspectionSnapshot,
} from "./features/inspect/appearanceInspection";
import { InspectStep } from "./features/inspect/InspectStep";
import {
  projectSourceInspection,
  type SourceInspectionSnapshot,
} from "./features/inspect/sourceInspection";
import type { InspectValidationCheck } from "./features/inspect/ValidationPanel";
import { AuroraReadbackViewport } from "./features/preview/AuroraReadbackViewport";
import { SourceViewport } from "./features/preview/SourceViewport";
import { PlaceableAuthoringEditor } from "./features/placeable-authoring/PlaceableAuthoringEditor";
import { ItemWorkflow } from "./features/item/ItemWorkflow";
import {
  parsePlaceableAuthoringBootstrap,
  type PlaceableAuthoringBootstrap,
} from "./features/placeable-authoring/types";
import type { BinaryMdlInspectionReport, ModelPartRef } from "./features/preview/types";
import {
  ReviewModelDetails,
  type ReviewViewport,
} from "./features/review/ReviewModelDetails";
import { PlaceableReview } from "./features/review/PlaceableReview";
import { TileReview } from "./features/review/TileReview";
import {
  projectCanonicalResult,
  type CanonicalResultSnapshot,
} from "./features/results/projectCanonicalResult";
import {
  projectPlaceableResult,
  type PlaceableResultSnapshot,
} from "./features/results/projectPlaceableResult";
import {
  projectTileResult,
  type TileResultSnapshot,
} from "./features/results/projectTileResult";
import { projectCanonicalReadback } from "./features/results/projectReadback";
import {
  InputsPanel,
  type CreatureConversionProfileV1,
  type SkinAccessoryStabilizationModeV1,
  type TileAuthoringOptions,
} from "./features/source/InputsPanel";
import { SourceStep } from "./features/source/SourceStep";
import { hasFullNativeDirectCreatureProfileV1 } from "./features/source/directCreatureAnimationProfile";
import { LocalMeshyBridgeClient, type MeshyArtifactProvenance, type MeshyBridgeClient } from "./features/meshy/bridge";
import { isMeshyLabEnabled } from "./features/meshy/feature";
import { MeshyLab } from "./features/meshy/MeshyLab";
import { isTileTargetEnabled } from "./features/tile/feature";
import { StudioWorkerClient } from "./worker/client";

const requestId = () => crypto.randomUUID();

interface StudioModelBuildResult {
  readonly kind: "MODEL";
  readonly canonical: CanonicalResultSnapshot;
  readonly readback: BinaryMdlInspectionReport;
  readonly readbackJson: string;
}

interface StudioPlaceableBuildResult {
  readonly kind: "PLACEABLE";
  readonly placeable: PlaceableResultSnapshot;
  readonly readback: BinaryMdlInspectionReport;
  readonly readbackJson: string;
}

interface StudioTileBuildResult {
  readonly kind: "TILE";
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

function reduceSession(
  state: StudioState,
  event: StudioSessionEvent<
    SourceInspectionSnapshot,
    AppearanceInspectionSnapshot,
    StudioBuildResult
  >,
) {
  return studioSessionReducer(state, event);
}

function isGlb(file: File) {
  return file.name.toLowerCase().endsWith(".glb");
}

function isBaseTwoDa(file: File) {
  return ["appearance.2da", "placeables.2da"].includes(file.name.toLowerCase());
}

function isJson(file: File) {
  return file.name.toLowerCase().endsWith(".json");
}

const STUDIO_PLACEABLE_IDENTITY = {
  moduleResref: "m2a_s1_plc_mod",
  moduleFileName: "m2a_s1_plc_mod.mod",
  moduleDisplayName: "Meshy2Aurora S1 Placeable Proof",
  areaResref: "m2a_s1_plc_ar",
  areaName: "Meshy2Aurora S1 Ritual Pedestal",
  hakResref: "m2a_s1_plc_hak",
  hakFileName: "m2a_s1_plc_hak.hak",
  modelResref: "m2a_s1_plc_ped",
  textureResref: "m2a_s1_plc_tex",
  blueprintResref: "m2a_s1_plc_utp",
  objectTag: "m2a_s1_ritual_pedestal",
  displayName: "Meshy Ritual Pedestal",
} as const;
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
const STUDIO_PLACEABLE_PLACEMENT = { x: 10, y: 14.5, z: 0, bearing: 0 } as const;
const STUDIO_TILE_IDENTITY = {
  moduleResref: "m2atilestv1",
  moduleFileName: "m2a_tile_static_v1.mod",
  moduleDisplayName: "Meshy2Aurora Tile Static V1",
  areaResref: "m2atilearea",
  areaName: "M2A Tile Static 2x2",
  hakResref: "m2atilestv1",
  hakFileName: "m2a_tile_static_v1.hak",
  tilesetResref: "m2atilesetv1",
  modelResref: "m2atilemdl1",
  textureResref: "m2atiletex1",
  imageMapResref: "m2atilemap1",
} as const;
const DEFAULT_TILE_OPTIONS: TileAuthoringOptions = {
  terrainName: "Grass",
  surface: "GRASS",
  interior: false,
};

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
}

export function App({
  meshyBridge,
  meshyLabEnabled = isMeshyLabEnabled(),
  tileTargetEnabled = isTileTargetEnabled(),
}: AppProps = {}) {
  const workerRef = useRef<StudioWorkerClient | undefined>(undefined);
  const sessionRef = useRef<StudioState>(
    createInitialStudioSession<
      SourceInspectionSnapshot,
      StudioBuildResult,
      AppearanceInspectionSnapshot
    >(),
  );
  const [session, dispatch] = useReducer(reduceSession, sessionRef.current);
  const [sourceError, setSourceError] = useState<string>();
  const [appearanceError, setAppearanceError] = useState<string>();
  const [animationEventsError, setAnimationEventsError] = useState<string>();
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
  const meshyBridgeRef = useRef<MeshyBridgeClient | undefined>(undefined);

  if (!meshyBridgeRef.current) meshyBridgeRef.current = meshyBridge ?? new LocalMeshyBridgeClient();

  sessionRef.current = session;
  placeableAuthoringRef.current = placeableAuthoring;

  useEffect(() => {
    const worker = new StudioWorkerClient();
    workerRef.current = worker;
    return () => {
      workerRef.current?.dispose();
      workerRef.current = undefined;
    };
  }, []);

  const replaceWorker = () => {
    workerRef.current?.dispose();
    workerRef.current = new StudioWorkerClient();
  };

  const invalidateRunningBuild = () => {
    if (sessionRef.current.build.kind !== "RUNNING") return;
    replaceWorker();
    setReviewViewport("CONVERTED");
    setSelectedReadbackPart(undefined);
  };

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
        setSourceError(undefined);
        setPlaceableAuthoring(
          response.placeableAuthoringJson
            ? parsePlaceableAuthoringBootstrap(response.placeableAuthoringJson)
            : undefined,
        );
        dispatch({
          type: "SOURCE_INSPECTION_SUCCEEDED",
          revision: inspectionRevision,
          sha256: projection.snapshot.source.sha256,
          inspection: projection.snapshot,
        });
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
        setAppearanceError(undefined);
        dispatch({
          type: "APPEARANCE_INSPECTION_SUCCEEDED",
          revision: inspectionRevision,
          sha256: inspection.sourceSha256,
          inspection,
        });
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
    setSourceError(undefined);
    setMeshyProvenance(undefined);
    setPlaceableAuthoring(undefined);
    dispatch({ type: "SOURCE_REMOVED" });
  };

  const removeAppearance = () => {
    invalidateRunningBuild();
    setAppearanceError(undefined);
    dispatch({ type: "APPEARANCE_REMOVED" });
  };

  const removeAnimationEvents = () => {
    invalidateRunningBuild();
    setAnimationEventsError(undefined);
    dispatch({ type: "ANIMATION_EVENTS_REMOVED" });
  };

  const clearFiles = () => {
    invalidateRunningBuild();
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
    const source = current.source.file;
    const appearance = current.appearance?.file;
    const animationEvents = current.animationEvents?.file;
    const tileLane = current.target === "TILE";
    const placeableLane = current.target === "PLACEABLE";
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
    const packageLane = p300kLane
      ? "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT" as const
      : p100kLane
        ? "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT" as const
      : current.sourceInspection.value.inventory.skinCount === 0
      && current.sourceInspection.value.clips.length === 0
      ? "M0_STATIC_RIGID" as const
      : fullNativeProfile
        ? animationEvents
          ? "H1_SKINNED_FULL_42_EVENTS" as const
          : "H1_SKINNED_FULL_42" as const
        : "SKINNED_PROCEDURAL_HUMANOID_42" as const;
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
    ])
      .then(([sourceGlb, appearanceTwoDa, eventAuthoringJson]) => {
        if (workerRef.current !== worker) return undefined;
        if (tileLane) {
          return worker.request(
            {
              requestId: buildRequestId,
              type: "BUILD_TILE_PACKAGE",
              sourceGlb,
              optionsJson: JSON.stringify({
                schemaVersion: 1,
                identity: STUDIO_TILE_IDENTITY,
                interior: tileOptions.interior,
                terrainName: tileOptions.terrainName.trim(),
                surface: tileOptions.surface,
              }),
            },
            [sourceGlb],
          );
        }
        if (!appearanceTwoDa) throw new Error("The selected conversion target requires a base 2DA");
        return placeableLane
          ? worker.request(
              {
                requestId: buildRequestId,
                type: "BUILD_PLACEABLE_PACKAGE",
                sourceGlb,
                placeablesTwoDa: appearanceTwoDa,
                identityJson: JSON.stringify(STUDIO_PLACEABLE_IDENTITY),
                placementJson: JSON.stringify(STUDIO_PLACEABLE_PLACEMENT),
                paletteId: 7,
                authoringJson: placeableAuthoringJson,
              },
              [sourceGlb, appearanceTwoDa],
            )
          : packageLane === "H1_SKINNED_FULL_42_EVENTS"
            ? worker.request(
                {
                  requestId: buildRequestId,
                  type: "BUILD_MODEL_PACKAGE",
                  sourceGlb,
                  appearanceTwoDa,
                  packageLane,
                  eventAuthoringJson: eventAuthoringJson ?? "",
                },
                [sourceGlb, appearanceTwoDa],
              )
            : packageLane === "SKINNED_PROCEDURAL_HUMANOID_42"
              || packageLane === "SKINNED_PROCEDURAL_HUMANOID_P100K_EXPERIMENT"
              || packageLane === "SKINNED_PROCEDURAL_HUMANOID_P300K_EXPERIMENT"
              ? worker.request(
                  {
                    requestId: buildRequestId,
                    type: "BUILD_MODEL_PACKAGE",
                    sourceGlb,
                    appearanceTwoDa,
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
                  [sourceGlb, appearanceTwoDa],
                )
            : worker.request(
                {
                  requestId: buildRequestId,
                  type: "BUILD_MODEL_PACKAGE",
                  sourceGlb,
                  appearanceTwoDa,
                  packageLane,
                },
                [sourceGlb, appearanceTwoDa],
              );
      })
      .then((response) => {
        if (!response || workerRef.current !== worker) return;
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
        const readbackJson = response.type === "TILE_PACKAGE_BUILT"
          ? response.modelReadbackJson
          : response.readbackJson;
        const readback = projectCanonicalReadback(readbackJson);
        setReviewViewport("CONVERTED");
        setSelectedReadbackPart(undefined);
        const result: StudioBuildResult = response.type === "PLACEABLE_PACKAGE_BUILT"
          ? {
              kind: "PLACEABLE",
              placeable: projectPlaceableResult(response.reportJson, response.artifacts),
              readback,
              readbackJson: response.readbackJson,
            }
          : response.type === "TILE_PACKAGE_BUILT"
            ? {
                kind: "TILE",
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
                canonical: projectCanonicalResult(
                  response.reportJson,
                  response.summaryJson,
                  response.manifestJson,
                  response.artifacts,
                ),
                readback,
                readbackJson: response.readbackJson,
              }
            : (() => { throw new Error("Unexpected package build response"); })();
        dispatch({
          type: "BUILD_SUCCEEDED",
          requestId: buildRequestId,
          revision: buildRevision,
          result,
        });
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

  if (session.target === "ITEM") {
    return <ItemWorkflow onTargetChange={selectTarget} meshyBridge={meshyBridgeRef.current} />;
  }

  const unlockedSteps = getUnlockedWorkflowSteps(session);
  const completedSteps = WORKFLOW_STEPS.filter(
    (step) => getWorkflowStepStatus(session, step) === "COMPLETE",
  );
  const blockedSteps = WORKFLOW_STEPS.filter(
    (step) => getWorkflowStepStatus(session, step) === "LOCKED",
  );
  const sourceIdentity = session.source?.sha256 ? { sha256: session.source.sha256 } : undefined;
  const appearanceIdentity = session.appearance?.sha256 ? { sha256: session.appearance.sha256 } : undefined;
  const sourceInspection = session.sourceInspection?.revision === session.revision
    ? session.sourceInspection.value
    : undefined;
  const appearanceInspection = session.appearanceInspection?.revision === session.revision
    ? session.appearanceInspection.value
    : undefined;
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
        completedStages: [],
        message: "The local Worker is executing the canonical pipeline. Per-stage progress is unavailable.",
      }
    : session.build.kind === "FAILED"
      ? {
          kind: "FAILED",
          completedStages: [],
          failedStage: buildStageForPipelineStage(session.build.failure.stage),
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
    <StudioShell
      header={<StudioHeader version="v0.1.0" environment="local" theme="dark" />}
      workflow={(
        <WorkflowStepper
          currentStep={session.currentStep}
          visitedSteps={unlockedSteps}
          completedSteps={completedSteps}
          blockedSteps={blockedSteps}
          onStepSelect={(step) => dispatch({ type: "NAVIGATE", step })}
        />
      )}
      inputs={showMeshyLab ? null : inputs}
      aside={!showMeshyLab && session.currentStep === "SOURCE" ? requirements : undefined}
      expandPrimaryToWorkspace={showMeshyLab}
      workspaceMode={showMeshyLab}
      debugDrawer={showMeshyLab ? null : (
        <section className="debug-drawer-placeholder" aria-label="Debug Drawer">
          <strong>Debug Drawer</strong>
          <span>{debugDrawerMessage ?? (session.currentStep === "SOURCE" ? "Disabled until inspection begins" : "Collapsed")}</span>
        </section>
      )}
    >
      {showMeshyLab ? (
        <MeshyLab
          bridge={meshyBridgeRef.current}
          onBack={() => setShowMeshyLab(false)}
          onImport={(file, provenance) => {
            selectSource(file, provenance);
            setShowMeshyLab(false);
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
          validationChecks={[
            ...sourceValidationChecks(sourceInspection),
            ...(session.target === "TILE" ? [] : appearanceValidationChecks(appearanceInspection)),
          ]}
          canContinue={
            sourceInspection?.conversionEligible === true
            && (session.target === "TILE" || Boolean(appearanceInspection))
          }
          wideViewport={session.target === "PLACEABLE" && Boolean(placeableAuthoring)}
          onBack={() => dispatch({ type: "NAVIGATE", step: "SOURCE" })}
          onContinue={() => dispatch({ type: "CONTINUE_TO_BUILD" })}
        />
      ) : session.currentStep === "BUILD" ? (
        <BuildStep
          state={buildStepState}
          canGoBack={session.build.kind !== "RUNNING"}
          canBuild={
            session.build.kind !== "RUNNING"
            && Boolean(sourceInspection)
            && (session.target === "TILE" || Boolean(appearanceInspection))
          }
          canRetry={session.build.kind === "FAILED"}
          canCancel={session.build.kind === "RUNNING"}
          onBack={() => dispatch({ type: "NAVIGATE", step: "INSPECT" })}
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
          <PlaceableReview result={currentResult.placeable} readback={currentResult.readback} />
          <ArtifactDownloads
            artifacts={currentResult.placeable.artifacts}
            onError={(message) => setDebugDrawerMessage(`Artifact download error: ${message}`)}
          />
        </>
      ) : session.currentStep === "REVIEW" && currentResult?.kind === "TILE" ? (
        <>
          <TileReview result={currentResult.tile} readback={currentResult.readback} />
          <ArtifactDownloads
            artifacts={currentResult.tile.artifacts}
            onError={(message) => setDebugDrawerMessage(`Artifact download error: ${message}`)}
          />
        </>
      ) : session.currentStep === "REVIEW" && currentResult?.kind === "MODEL" && session.source?.sha256 ? (
        <>
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
          <ArtifactDownloads
            artifacts={currentResult.canonical.artifacts}
            onError={(message) => setDebugDrawerMessage(`Artifact download error: ${message}`)}
          />
        </>
      ) : (
        <section className="inspect-scaffold" aria-labelledby="review-unavailable-heading">
          <header className="inspect-scaffold__header">
            <div><p className="eyebrow">Review output</p><h1 id="review-unavailable-heading">Result unavailable</h1></div>
          </header>
          <div className="empty-state"><strong>No current canonical result.</strong><span>Return to Build and run the pipeline.</span></div>
        </section>
      )}
    </StudioShell>
  );
}
