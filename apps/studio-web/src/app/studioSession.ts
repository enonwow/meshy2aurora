import { compareWorkflowSteps, type WorkflowStep } from "./workflow";

export type StudioInputKind = "SOURCE" | "APPEARANCE";

export type StudioInputParseState =
  | { kind: "NOT_STARTED" }
  | { kind: "PARSING" }
  | { kind: "VALID" }
  | { kind: "INVALID"; message: string };

export interface StudioInputFile {
  readonly file: File;
  readonly name: string;
  readonly size: number;
  readonly type: string;
  readonly lastModified: number;
  readonly sha256: string | null;
  readonly parse: StudioInputParseState;
}

export interface RevisionBoundSnapshot<T> {
  readonly revision: number;
  readonly value: T;
}

export interface BuildProgressSnapshot {
  readonly phase: string;
  readonly message?: string;
}

export interface BuildFailureSnapshot {
  readonly message: string;
  readonly code?: string;
}

export type BuildState<TResult = unknown> =
  | { readonly kind: "IDLE" }
  | {
      readonly kind: "RUNNING";
      readonly requestId: string;
      readonly revision: number;
      readonly progress?: BuildProgressSnapshot;
    }
  | {
      readonly kind: "FAILED";
      readonly requestId: string;
      readonly revision: number;
      readonly failure: BuildFailureSnapshot;
    }
  | {
      readonly kind: "SUCCEEDED";
      readonly requestId: string;
      readonly revision: number;
      readonly result: RevisionBoundSnapshot<TResult>;
    };

export type DownloadState =
  | { readonly kind: "LOCKED" }
  | { readonly kind: "READY"; readonly revision: number };

export interface StudioSessionState<TInspection = unknown, TResult = unknown> {
  readonly revision: number;
  readonly currentStep: WorkflowStep;
  readonly lastAvailableStep: WorkflowStep;
  readonly source: StudioInputFile | null;
  readonly appearance: StudioInputFile | null;
  readonly sourceInspection: RevisionBoundSnapshot<TInspection> | null;
  readonly build: BuildState<TResult>;
  readonly result: RevisionBoundSnapshot<TResult> | null;
  readonly download: DownloadState;
}

export type StudioSessionEvent =
  | { readonly type: "SOURCE_SELECTED"; readonly file: File }
  | { readonly type: "APPEARANCE_SELECTED"; readonly file: File }
  | { readonly type: "SOURCE_REMOVED" }
  | { readonly type: "APPEARANCE_REMOVED" }
  | {
      readonly type: "INPUT_METADATA_UPDATED";
      readonly input: StudioInputKind;
      readonly revision: number;
      readonly sha256?: string;
      readonly parse?: StudioInputParseState;
    }
  | { readonly type: "CONTINUE_TO_INSPECT" }
  | { readonly type: "NAVIGATE"; readonly step: WorkflowStep }
  | { readonly type: "START_NEW_CONVERSION" };

function selectedInput(file: File): StudioInputFile {
  return {
    file,
    name: file.name,
    size: file.size,
    type: file.type,
    lastModified: file.lastModified,
    sha256: null,
    parse: { kind: "NOT_STARTED" },
  };
}

export function createInitialStudioSession<TInspection = unknown, TResult = unknown>(
  revision = 0,
): StudioSessionState<TInspection, TResult> {
  return {
    revision,
    currentStep: "SOURCE",
    lastAvailableStep: "SOURCE",
    source: null,
    appearance: null,
    sourceInspection: null,
    build: { kind: "IDLE" },
    result: null,
    download: { kind: "LOCKED" },
  };
}

function invalidateDownstream<TInspection, TResult>(
  state: StudioSessionState<TInspection, TResult>,
  inputs: Pick<StudioSessionState<TInspection, TResult>, "source" | "appearance">,
): StudioSessionState<TInspection, TResult> {
  return {
    ...state,
    ...inputs,
    revision: state.revision + 1,
    currentStep: "SOURCE",
    lastAvailableStep: "SOURCE",
    sourceInspection: null,
    build: { kind: "IDLE" },
    result: null,
    download: { kind: "LOCKED" },
  };
}

function updateInputMetadata<TInspection, TResult>(
  state: StudioSessionState<TInspection, TResult>,
  event: Extract<StudioSessionEvent, { type: "INPUT_METADATA_UPDATED" }>,
): StudioSessionState<TInspection, TResult> {
  if (event.revision !== state.revision) return state;

  const key = event.input === "SOURCE" ? "source" : "appearance";
  const input = state[key];
  if (!input) return state;

  return {
    ...state,
    [key]: {
      ...input,
      sha256: event.sha256 ?? input.sha256,
      parse: event.parse ?? input.parse,
    },
  };
}

export function studioSessionReducer<TInspection, TResult>(
  state: StudioSessionState<TInspection, TResult>,
  event: StudioSessionEvent,
): StudioSessionState<TInspection, TResult> {
  switch (event.type) {
    case "SOURCE_SELECTED":
      return invalidateDownstream(state, {
        source: selectedInput(event.file),
        appearance: state.appearance,
      });
    case "APPEARANCE_SELECTED":
      return invalidateDownstream(state, {
        source: state.source,
        appearance: selectedInput(event.file),
      });
    case "SOURCE_REMOVED":
      return invalidateDownstream(state, { source: null, appearance: state.appearance });
    case "APPEARANCE_REMOVED":
      return invalidateDownstream(state, { source: state.source, appearance: null });
    case "INPUT_METADATA_UPDATED":
      return updateInputMetadata(state, event);
    case "CONTINUE_TO_INSPECT":
      if (!state.source || !state.appearance) return state;
      return {
        ...state,
        currentStep: "INSPECT",
        lastAvailableStep: compareWorkflowSteps(state.lastAvailableStep, "INSPECT") >= 0
          ? state.lastAvailableStep
          : "INSPECT",
      };
    case "NAVIGATE":
      if (compareWorkflowSteps(event.step, state.lastAvailableStep) > 0) return state;
      if (event.step === state.currentStep) return state;
      return { ...state, currentStep: event.step };
    case "START_NEW_CONVERSION":
      return createInitialStudioSession(state.revision + 1);
  }
}
