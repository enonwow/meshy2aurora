import "./build.css";

export const BUILD_STAGE_IDS = [
  "INGEST_SOURCE",
  "NORMALIZE_CANONICAL_IR",
  "WRITE_BINARY_MDL",
  "CANONICAL_BINARY_READBACK",
  "UPDATE_APPEARANCE_2DA",
  "PACKAGE_HAK",
] as const;

export type BuildStageId = typeof BUILD_STAGE_IDS[number];

const BUILD_STAGE_LABELS: Readonly<Record<BuildStageId, string>> = {
  INGEST_SOURCE: "Ingest source",
  NORMALIZE_CANONICAL_IR: "Normalize canonical IR",
  WRITE_BINARY_MDL: "Write binary MDL",
  CANONICAL_BINARY_READBACK: "Canonical binary readback",
  UPDATE_APPEARANCE_2DA: "Update appearance.2da",
  PACKAGE_HAK: "Package HAK",
};

export interface BuildFailureEvidence {
  readonly message: string;
  readonly code?: string;
  readonly stage?: string;
  readonly path?: string;
}

export type BuildInputInspectionStatus = "INSPECTED" | "NOT_INSPECTED" | "INVALID";

export interface BuildInputEvidence {
  readonly name: string;
  readonly byteLength: number;
  readonly sha256: string;
  readonly inspectionStatus: BuildInputInspectionStatus;
}

export interface BuildInputSummary {
  readonly source: BuildInputEvidence;
  readonly appearance: BuildInputEvidence;
}

export interface BuildFailureDiagnosticsProps {
  readonly canOpen: boolean;
  readonly onOpen: () => void;
  readonly reportPackage: {
    readonly status: "UNAVAILABLE";
    readonly reason: string;
  };
}

export type BuildStepState =
  | {
      readonly kind: "IDLE";
      readonly message?: string;
      readonly inputs?: BuildInputSummary;
    }
  | {
      readonly kind: "RUNNING";
      readonly activeStage?: BuildStageId;
      readonly completedStages: readonly BuildStageId[];
      readonly message: string;
    }
  | {
      readonly kind: "FAILED";
      readonly failedStage?: BuildStageId;
      readonly completedStages: readonly BuildStageId[];
      readonly failure: BuildFailureEvidence;
    };

export interface BuildStepProps {
  readonly state: BuildStepState;
  readonly canGoBack: boolean;
  readonly canBuild: boolean;
  readonly canRetry: boolean;
  readonly canCancel: boolean;
  readonly onBack: () => void;
  readonly onBuild: () => void;
  readonly onRetry: () => void;
  readonly onCancel: () => void;
  readonly failureDiagnostics?: BuildFailureDiagnosticsProps;
}

type StagePresentationStatus = "Pending" | "Complete" | "Running" | "Failed" | "Not run" | "Unknown";

function stageStatus(state: BuildStepState, stage: BuildStageId): StagePresentationStatus {
  if (state.kind === "IDLE") return "Pending";
  if (state.kind === "RUNNING") {
    if (state.completedStages.includes(stage)) return "Complete";
    return state.activeStage === stage ? "Running" : "Pending";
  }
  if (!state.failedStage) {
    return state.completedStages.includes(stage) ? "Complete" : "Unknown";
  }
  if (state.failedStage === stage) return "Failed";
  const failedIndex = BUILD_STAGE_IDS.indexOf(state.failedStage);
  const stageIndex = BUILD_STAGE_IDS.indexOf(stage);
  if (stageIndex > failedIndex) return "Not run";
  return state.completedStages.includes(stage) ? "Complete" : "Not run";
}

function BuildLedger({ state }: { readonly state: BuildStepState }) {
  return (
    <ol className="build-ledger" aria-label="Build pipeline stages">
      {BUILD_STAGE_IDS.map((stage, index) => {
        const status = stageStatus(state, stage);
        return (
          <li className="build-ledger__stage" data-status={status.toUpperCase().replace(" ", "_")} key={stage}>
            <span className="build-ledger__index" aria-hidden="true">{index + 1}</span>
            <div className="build-ledger__stage-body">
              <strong>{BUILD_STAGE_LABELS[stage]}</strong>
              <span>{status}</span>
              {state.kind === "RUNNING" && state.activeStage === stage ? (
                <span className="build-ledger__indeterminate" aria-label="Build running without percentage">
                  <span aria-hidden="true" />
                </span>
              ) : null}
              {state.kind === "FAILED" && state.failedStage === stage ? (
                <div className="build-ledger__failure">
                  {state.failure.code ? <code>{state.failure.code}</code> : null}
                  <span>{state.failure.message}</span>
                </div>
              ) : null}
            </div>
          </li>
        );
      })}
    </ol>
  );
}

export function BuildStep({
  state,
  canGoBack,
  canBuild,
  canRetry,
  canCancel,
  onBack,
  onBuild,
  onRetry,
  onCancel,
  failureDiagnostics,
}: BuildStepProps) {
  const headingStatus = state.kind === "IDLE"
    ? "Not started"
    : state.kind === "RUNNING"
      ? "Running"
      : "Failed";

  return (
    <section className="build-step" aria-labelledby="build-step-heading">
      <header className="build-step__header">
        <div>
          <p className="build-step__eyebrow">Step 3</p>
          <h1 id="build-step-heading">Build</h1>
        </div>
        <span className="build-step__state" data-state={state.kind}>{headingStatus}</span>
      </header>

      <div className="build-step__workspace">
        <section className="build-step__pipeline" aria-labelledby="build-pipeline-heading">
          <header>
            <h2 id="build-pipeline-heading">Build pipeline</h2>
            <span>6 stages</span>
          </header>
          <BuildLedger state={state} />
        </section>

        <aside className="build-step__status" aria-labelledby="build-status-heading">
          <h2 id="build-status-heading">Build status</h2>
          {state.kind === "IDLE" ? (
            <p>{state.message ?? "Build has not started."}</p>
          ) : state.kind === "RUNNING" ? (
            <div className="build-step__running" role="status" aria-live="polite">
              <span className="build-step__spinner" aria-hidden="true" />
              <div>
                <strong>Running — indeterminate</strong>
                <p>{state.message}</p>
              </div>
            </div>
          ) : (
            <div className="build-step__error" role="alert">
              <strong>Build failed</strong>
              {state.failure.code ? <code>{state.failure.code}</code> : null}
              <p>{state.failure.message}</p>
            </div>
          )}
        </aside>
      </div>

      {state.kind === "IDLE" ? (
        <section className="build-input-summary" aria-labelledby="build-input-summary-heading">
          <header><h2 id="build-input-summary-heading">Inspected inputs</h2></header>
          {state.inputs ? (
            <dl className="build-input-summary__grid">
              {([
                ["Source GLB", state.inputs.source],
                ["Base appearance.2da", state.inputs.appearance],
              ] as const).map(([label, input]) => (
                <div className="build-input-summary__item" key={label}>
                  <dt>{label}</dt>
                  <dd>
                    <strong>{input.name}</strong>
                    <span>{input.byteLength.toLocaleString("en-US")} bytes</span>
                    <code title={input.sha256}>{input.sha256}</code>
                    <span className="build-input-summary__inspection" data-status={input.inspectionStatus}>
                      {input.inspectionStatus.replace("_", " ")}
                    </span>
                  </dd>
                </div>
              ))}
            </dl>
          ) : (
            <p className="build-input-summary__unavailable">Input inspection summary is unavailable.</p>
          )}
        </section>
      ) : null}

      {state.kind === "FAILED" ? (
        <section className="build-failure-diagnostics" aria-labelledby="build-failure-diagnostics-heading">
          <header><h2 id="build-failure-diagnostics-heading">Failure diagnostics</h2></header>
          <div className="build-failure-diagnostics__evidence">
            <span>Failure code</span>
            <strong>{state.failure.code ?? "Unavailable"}</strong>
            <span>Pipeline stage</span>
            <strong>{state.failure.stage ?? "Unknown"}</strong>
            <span>Failure path</span>
            <code>{state.failure.path ?? "Unavailable"}</code>
            <span>Failure message</span>
            <p>{state.failure.message}</p>
          </div>
          <div className="build-failure-diagnostics__report" data-status="UNAVAILABLE">
            <strong>Debug report package: Unavailable</strong>
            <span>{failureDiagnostics?.reportPackage.reason ?? "No debug report package was produced for this failure."}</span>
          </div>
          <button
            type="button"
            className="build-step__button build-step__button--secondary"
            onClick={failureDiagnostics?.onOpen}
            disabled={!failureDiagnostics?.canOpen}
          >
            Open failure diagnostics
          </button>
        </section>
      ) : null}

      <footer className="build-step__actions">
        <button type="button" className="build-step__button build-step__button--secondary" onClick={onBack} disabled={!canGoBack}>
          Back to Inspect
        </button>
        {state.kind === "RUNNING" ? (
          <button type="button" className="build-step__button build-step__button--danger" onClick={onCancel} disabled={!canCancel}>
            Cancel Build
          </button>
        ) : state.kind === "FAILED" ? (
          <button type="button" className="build-step__button build-step__button--primary" onClick={onRetry} disabled={!canRetry}>
            Retry Build
          </button>
        ) : (
          <button type="button" className="build-step__button build-step__button--primary" onClick={onBuild} disabled={!canBuild}>
            Build Package
          </button>
        )}
      </footer>
    </section>
  );
}
