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
import { SourceViewport } from "./features/preview/SourceViewport";
import { InputsPanel } from "./features/source/InputsPanel";
import { SourceStep } from "./features/source/SourceStep";
import { StudioWorkerClient } from "./worker/client";

const requestId = () => crypto.randomUUID();

function reduceSession(state: StudioSessionState, event: StudioSessionEvent) {
  return studioSessionReducer(state, event);
}

function bytesToHex(bytes: ArrayBuffer) {
  return [...new Uint8Array(bytes)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function fileSha256(file: File) {
  return bytesToHex(await crypto.subtle.digest("SHA-256", await file.arrayBuffer()));
}

function isGlb(file: File) {
  return file.name.toLowerCase().endsWith(".glb");
}

function isAppearanceTwoDa(file: File) {
  return file.name.toLowerCase() === "appearance.2da";
}

export function App() {
  const workerRef = useRef<StudioWorkerClient | undefined>(undefined);
  const sessionRef = useRef<StudioSessionState>(createInitialStudioSession());
  const [session, dispatch] = useReducer(reduceSession, sessionRef.current);
  const [sourceError, setSourceError] = useState<string>();
  const [appearanceError, setAppearanceError] = useState<string>();

  sessionRef.current = session;

  useEffect(() => {
    const worker = new StudioWorkerClient();
    workerRef.current = worker;
    return () => {
      worker.dispose();
      if (workerRef.current === worker) workerRef.current = undefined;
    };
  }, []);

  const sourceFile = session.source?.file;
  useEffect(() => {
    if (!sourceFile) return;
    const worker = workerRef.current;
    if (!worker) return;
    let cancelled = false;

    void sourceFile.arrayBuffer()
      .then((sourceGlb) => worker.request(
        { requestId: requestId(), type: "INSPECT_SOURCE", sourceGlb },
        [sourceGlb],
      ))
      .then((response) => {
        if (cancelled || sessionRef.current.source?.file !== sourceFile) return;
        if (!response.ok || response.type !== "SOURCE_INSPECTED") {
          throw new Error("Unexpected source inspection response");
        }
        const ingest = JSON.parse(response.ingestJson) as { ir?: { source?: { sha256?: string } } };
        const sha256 = ingest.ir?.source?.sha256;
        if (!sha256) throw new Error("Source inspection did not return SHA-256");
        dispatch({
          type: "INPUT_METADATA_UPDATED",
          input: "SOURCE",
          revision: sessionRef.current.revision,
          sha256,
          parse: { kind: "VALID" },
        });
      })
      .catch((error: unknown) => {
        if (cancelled || sessionRef.current.source?.file !== sourceFile) return;
        const message = error instanceof Error ? error.message : String(error);
        setSourceError(message);
        dispatch({
          type: "INPUT_METADATA_UPDATED",
          input: "SOURCE",
          revision: sessionRef.current.revision,
          parse: { kind: "INVALID", message },
        });
      });

    return () => { cancelled = true; };
  }, [sourceFile]);

  const appearanceFile = session.appearance?.file;
  useEffect(() => {
    if (!appearanceFile) return;
    let cancelled = false;
    void fileSha256(appearanceFile)
      .then((sha256) => {
        if (cancelled || sessionRef.current.appearance?.file !== appearanceFile) return;
        dispatch({
          type: "INPUT_METADATA_UPDATED",
          input: "APPEARANCE",
          revision: sessionRef.current.revision,
          sha256,
          parse: { kind: "NOT_STARTED" },
        });
      })
      .catch((error: unknown) => {
        if (cancelled || sessionRef.current.appearance?.file !== appearanceFile) return;
        setAppearanceError(error instanceof Error ? error.message : String(error));
      });
    return () => { cancelled = true; };
  }, [appearanceFile]);

  const selectSource = (file: File) => {
    if (!isGlb(file)) {
      setSourceError("Select a Meshy model in .glb format.");
      return;
    }
    setSourceError(undefined);
    dispatch({ type: "SOURCE_SELECTED", file });
  };

  const selectAppearance = (file: File) => {
    if (!isAppearanceTwoDa(file)) {
      setAppearanceError("Select the base file named appearance.2da.");
      return;
    }
    setAppearanceError(undefined);
    dispatch({ type: "APPEARANCE_SELECTED", file });
  };

  const removeSource = () => {
    setSourceError(undefined);
    dispatch({ type: "SOURCE_REMOVED" });
  };

  const removeAppearance = () => {
    setAppearanceError(undefined);
    dispatch({ type: "APPEARANCE_REMOVED" });
  };

  const clearFiles = () => {
    setSourceError(undefined);
    setAppearanceError(undefined);
    dispatch({ type: "START_NEW_CONVERSION" });
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
      inputs={inputs}
      aside={requirements}
      debugDrawer={(
        <section className="debug-drawer-placeholder" aria-label="Debug Drawer">
          <strong>Debug Drawer</strong>
          <span>{session.currentStep === "SOURCE" ? "Disabled until inspection begins" : "Collapsed"}</span>
        </section>
      )}
    >
      {session.currentStep === "SOURCE" ? (
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
        />
      ) : (
        <section className="inspect-scaffold" aria-labelledby="inspect-heading">
          <header className="inspect-scaffold__header">
            <div><p className="eyebrow">Step 2</p><h1 id="inspect-heading">Inspect source</h1></div>
            <span className="status-badge status-badge--neutral">FE-V2 next</span>
          </header>
          {session.source && session.source.sha256 ? (
            <SourceViewport input={{ provenance: "SOURCE", file: session.source.file, sourceSha256: session.source.sha256 }} onError={setSourceError} />
          ) : (
            <div className="empty-state"><strong>Preparing local source inspection</strong><span>No upload occurs.</span></div>
          )}
          <p className="inspect-scaffold__notice">The full inspection workspace, source metrics, validation and animation controls are implemented in FE-V2 and FE-V3.</p>
        </section>
      )}
    </StudioShell>
  );
}
