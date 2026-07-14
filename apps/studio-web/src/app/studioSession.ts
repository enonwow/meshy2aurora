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

export interface StudioSessionState<
  TInspection = unknown,
  TResult = unknown,
  TAppearanceInspection = unknown,
> {
  readonly revision: number;
  readonly currentStep: WorkflowStep;
  readonly lastAvailableStep: WorkflowStep;
  readonly source: StudioInputFile | null;
  readonly appearance: StudioInputFile | null;
  readonly sourceInspection: RevisionBoundSnapshot<TInspection> | null;
  readonly appearanceInspection: RevisionBoundSnapshot<TAppearanceInspection> | null;
  readonly build: BuildState<TResult>;
  readonly result: RevisionBoundSnapshot<TResult> | null;
  readonly download: DownloadState;
}

export type StudioSessionEvent<TInspection = unknown, TAppearanceInspection = unknown> =
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
  | {
      readonly type: "SOURCE_INSPECTION_SUCCEEDED";
      readonly revision: number;
      readonly sha256: string;
      readonly inspection: TInspection;
    }
  | {
      readonly type: "APPEARANCE_INSPECTION_SUCCEEDED";
      readonly revision: number;
      readonly sha256: string;
      readonly inspection: TAppearanceInspection;
    }
  | { readonly type: "CONTINUE_TO_INSPECT" }
  | { readonly type: "CONTINUE_TO_BUILD" }
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

export function createInitialStudioSession<
  TInspection = unknown,
  TResult = unknown,
  TAppearanceInspection = unknown,
>(
  revision = 0,
): StudioSessionState<TInspection, TResult, TAppearanceInspection> {
  return {
    revision,
    currentStep: "SOURCE",
    lastAvailableStep: "SOURCE",
    source: null,
    appearance: null,
    sourceInspection: null,
    appearanceInspection: null,
    build: { kind: "IDLE" },
    result: null,
    download: { kind: "LOCKED" },
  };
}

function invalidateDownstream<TInspection, TResult, TAppearanceInspection>(
  state: StudioSessionState<TInspection, TResult, TAppearanceInspection>,
  inputs: Pick<StudioSessionState<TInspection, TResult, TAppearanceInspection>, "source" | "appearance">,
): StudioSessionState<TInspection, TResult, TAppearanceInspection> {
  return {
    ...state,
    ...inputs,
    revision: state.revision + 1,
    currentStep: "SOURCE",
    lastAvailableStep: "SOURCE",
    sourceInspection: null,
    appearanceInspection: null,
    build: { kind: "IDLE" },
    result: null,
    download: { kind: "LOCKED" },
  };
}

function updateInputMetadata<TInspection, TResult, TAppearanceInspection>(
  state: StudioSessionState<TInspection, TResult, TAppearanceInspection>,
  event: Extract<StudioSessionEvent, { type: "INPUT_METADATA_UPDATED" }>,
): StudioSessionState<TInspection, TResult, TAppearanceInspection> {
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

export function studioSessionReducer<TInspection, TResult, TAppearanceInspection>(
  state: StudioSessionState<TInspection, TResult, TAppearanceInspection>,
  event: StudioSessionEvent<TInspection, TAppearanceInspection>,
): StudioSessionState<TInspection, TResult, TAppearanceInspection> {
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
    case "SOURCE_INSPECTION_SUCCEEDED":
      if (event.revision !== state.revision || !state.source) return state;
      return {
        ...state,
        source: {
          ...state.source,
          sha256: event.sha256,
          parse: { kind: "VALID" },
        },
        sourceInspection: {
          revision: event.revision,
          value: event.inspection,
        },
      };
    case "APPEARANCE_INSPECTION_SUCCEEDED":
      if (event.revision !== state.revision || !state.appearance) return state;
      return {
        ...state,
        appearance: {
          ...state.appearance,
          sha256: event.sha256,
          parse: { kind: "VALID" },
        },
        appearanceInspection: {
          revision: event.revision,
          value: event.inspection,
        },
      };
    case "CONTINUE_TO_INSPECT":
      if (!state.source || !state.appearance) return state;
      return {
        ...state,
        currentStep: "INSPECT",
        lastAvailableStep: compareWorkflowSteps(state.lastAvailableStep, "INSPECT") >= 0
          ? state.lastAvailableStep
          : "INSPECT",
      };
    case "CONTINUE_TO_BUILD":
      if (
        !state.source
        || !state.appearance
        || !state.sourceInspection
        || !state.appearanceInspection
        || state.sourceInspection.revision !== state.revision
        || state.appearanceInspection.revision !== state.revision
      ) return state;
      return {
        ...state,
        currentStep: "BUILD",
        lastAvailableStep: compareWorkflowSteps(state.lastAvailableStep, "BUILD") >= 0
          ? state.lastAvailableStep
          : "BUILD",
      };
    case "NAVIGATE":
      if (compareWorkflowSteps(event.step, state.lastAvailableStep) > 0) return state;
      if (event.step === state.currentStep) return state;
      return { ...state, currentStep: event.step };
    case "START_NEW_CONVERSION":
      return createInitialStudioSession<TInspection, TResult, TAppearanceInspection>(state.revision + 1);
  }
}
