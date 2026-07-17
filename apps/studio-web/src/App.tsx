import { useEffect, useReducer, useRef, useState } from "react";
import { StudioHeader } from "./app/StudioHeader";
import { StudioShell } from "./app/StudioShell";
import { getUnlockedWorkflowSteps, getWorkflowStepStatus } from "./app/studioSelectors";
import {
  createInitialStudioSession,
  studioSessionReducer,
  type StudioSessionEvent,
  type StudioSessionState,
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
import type { BinaryMdlInspectionReport, ModelPartRef } from "./features/preview/types";
import {
  ReviewModelDetails,
  type ReviewViewport,
} from "./features/review/ReviewModelDetails";
import {
  projectCanonicalResult,
  type CanonicalResultSnapshot,
} from "./features/results/projectCanonicalResult";
import { projectCanonicalReadback } from "./features/results/projectReadback";
import { InputsPanel } from "./features/source/InputsPanel";
import { SourceStep } from "./features/source/SourceStep";
import { LocalMeshyBridgeClient, type MeshyArtifactProvenance, type MeshyBridgeClient } from "./features/meshy/bridge";
import { isMeshyLabEnabled } from "./features/meshy/feature";
import { MeshyLab } from "./features/meshy/MeshyLab";
import { StudioWorkerClient } from "./worker/client";

const requestId = () => crypto.randomUUID();

interface StudioBuildResult {
  readonly canonical: CanonicalResultSnapshot;
  readonly readback: BinaryMdlInspectionReport;
  readonly readbackJson: string;
}

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

function isAppearanceTwoDa(file: File) {
  return file.name.toLowerCase() === "appearance.2da";
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
}

export function App({ meshyBridge, meshyLabEnabled = isMeshyLabEnabled() }: AppProps = {}) {
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
  const [reviewViewport, setReviewViewport] = useState<ReviewViewport>("CONVERTED");
  const [selectedReadbackPart, setSelectedReadbackPart] = useState<ModelPartRef>();
  const [debugDrawerMessage, setDebugDrawerMessage] = useState<string>();
  const [showMeshyLab, setShowMeshyLab] = useState(false);
  const [meshyProvenance, setMeshyProvenance] = useState<MeshyArtifactProvenance>();
  const meshyBridgeRef = useRef<MeshyBridgeClient | undefined>(undefined);

  if (!meshyBridgeRef.current) meshyBridgeRef.current = meshyBridge ?? new LocalMeshyBridgeClient();

  sessionRef.current = session;

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
        { requestId: requestId(), type: "INSPECT_SOURCE", sourceGlb },
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
  }, [session.revision, sourceFile]);

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
    setMeshyProvenance(provenance);
    dispatch({ type: "SOURCE_SELECTED", file });
  };

  const selectAppearance = (file: File) => {
    if (!isAppearanceTwoDa(file)) {
      setAppearanceError("Select the base file named appearance.2da.");
      return;
    }
    invalidateRunningBuild();
    setAppearanceError(undefined);
    dispatch({ type: "APPEARANCE_SELECTED", file });
  };

  const removeSource = () => {
    invalidateRunningBuild();
    setSourceError(undefined);
    setMeshyProvenance(undefined);
    dispatch({ type: "SOURCE_REMOVED" });
  };

  const removeAppearance = () => {
    invalidateRunningBuild();
    setAppearanceError(undefined);
    dispatch({ type: "APPEARANCE_REMOVED" });
  };

  const clearFiles = () => {
    invalidateRunningBuild();
    setSourceError(undefined);
    setMeshyProvenance(undefined);
    setAppearanceError(undefined);
    setSelectedReadbackPart(undefined);
    setDebugDrawerMessage(undefined);
    dispatch({ type: "START_NEW_CONVERSION" });
  };

  const startBuild = () => {
    const current = sessionRef.current;
    const worker = workerRef.current;
    if (
      !worker
      || current.currentStep !== "BUILD"
      || !current.source
      || !current.appearance
      || !current.sourceInspection
      || !current.appearanceInspection
      || current.sourceInspection.revision !== current.revision
      || current.appearanceInspection.revision !== current.revision
    ) return;

    const buildRequestId = requestId();
    const buildRevision = current.revision;
    const source = current.source.file;
    const appearance = current.appearance.file;
    setDebugDrawerMessage(undefined);
    dispatch({ type: "BUILD_STARTED", requestId: buildRequestId, revision: buildRevision });

    void Promise.all([source.arrayBuffer(), appearance.arrayBuffer()])
      .then(([sourceGlb, appearanceTwoDa]) => {
        if (workerRef.current !== worker) return undefined;
        return worker.request(
          {
            requestId: buildRequestId,
            type: "BUILD_MODEL_PACKAGE",
            sourceGlb,
            appearanceTwoDa,
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
        if (response.type !== "MODEL_PACKAGE_BUILT") {
          throw new Error("Unexpected model package build response");
        }
        const canonical = projectCanonicalResult(
          response.reportJson,
          response.summaryJson,
          response.manifestJson,
          response.artifacts,
        );
        const readback = projectCanonicalReadback(response.readbackJson);
        setReviewViewport("CONVERTED");
        setSelectedReadbackPart(undefined);
        dispatch({
          type: "BUILD_SUCCEEDED",
          requestId: buildRequestId,
          revision: buildRevision,
          result: { canonical, readback, readbackJson: response.readbackJson },
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
  const buildInputs = session.source?.sha256 && session.appearance?.sha256
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
            : "Both inspected inputs are ready for the local canonical build.",
          ...(buildInputs ? { inputs: buildInputs } : {}),
        };
  const currentResult = session.result?.revision === session.revision
    ? session.result.value
    : undefined;

  const inputs = (
    <InputsPanel
      source={session.source?.file}
      appearance={session.appearance?.file}
      sourceIdentity={sourceIdentity}
      appearanceIdentity={appearanceIdentity}
      sourceError={sourceError}
      appearanceError={appearanceError}
      onSelectSource={selectSource}
      onSelectAppearance={selectAppearance}
      onRemoveSource={removeSource}
      onRemoveAppearance={removeAppearance}
      onClear={clearFiles}
    />
  );

  const requirements = (
    <aside className="panel requirements-panel" aria-labelledby="input-requirements-heading">
      <header className="panel__header"><h2 id="input-requirements-heading">Input Requirements</h2></header>
      <ul className="requirements-panel__list">
        <li><span>Meshy GLB</span><strong data-ready={Boolean(session.source)}>{session.source ? "selected" : "required"}</strong></li>
        <li><span>Base appearance.2da</span><strong data-ready={Boolean(session.appearance)}>{session.appearance ? "selected" : "required"}</strong></li>
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
      debugDrawer={(
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
          source={session.source?.file}
          appearance={session.appearance?.file}
          sourceIdentity={sourceIdentity}
          appearanceIdentity={appearanceIdentity}
          sourceError={sourceError}
          appearanceError={appearanceError}
          onSelectSource={selectSource}
          onSelectAppearance={selectAppearance}
          onRemoveSource={removeSource}
          onRemoveAppearance={removeAppearance}
          onClear={clearFiles}
          onContinue={() => dispatch({ type: "CONTINUE_TO_INSPECT" })}
          onOpenMeshyLab={meshyLabEnabled ? () => setShowMeshyLab(true) : undefined}
          meshyProvenance={meshyProvenance}
        />
      ) : session.currentStep === "INSPECT" ? (
        <InspectStep
          viewport={session.source && session.source.sha256 ? (
            <SourceViewport
              input={{
                provenance: "SOURCE",
                file: session.source.file,
                sourceSha256: session.source.sha256,
              }}
              onError={setSourceError}
            />
          ) : (
            <div className="empty-state">
              <strong>Preparing local source inspection</strong>
              <span>{sourceError ?? "No upload occurs."}</span>
            </div>
          )}
          sourceMetrics={sourceMetrics}
          validationChecks={[
            ...sourceValidationChecks(sourceInspection),
            ...appearanceValidationChecks(appearanceInspection),
          ]}
          canContinue={sourceInspection?.conversionEligible === true && Boolean(appearanceInspection)}
          onBack={() => dispatch({ type: "NAVIGATE", step: "SOURCE" })}
          onContinue={() => dispatch({ type: "CONTINUE_TO_BUILD" })}
        />
      ) : session.currentStep === "BUILD" ? (
        <BuildStep
          state={buildStepState}
          canGoBack={session.build.kind !== "RUNNING"}
          canBuild={session.build.kind !== "RUNNING" && Boolean(sourceInspection && appearanceInspection)}
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
      ) : session.currentStep === "REVIEW" && currentResult && session.source?.sha256 ? (
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
