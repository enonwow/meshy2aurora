import {
  compareWorkflowSteps,
  getWorkflowStepsForTarget,
  type WorkflowStep,
} from "./workflow";
import {
  createCreatureAnimationAuthoringV1,
  isAnimationMappingCurrentV1,
  reduceCreatureAnimationAuthoringV1,
  type CreatureAnimationAuthoringEventV1,
} from "../features/animation-mapping/state";
import type {
  AnimationMappingDiagnosticV1,
  CreatureAnimationAuthoringV1,
  CreatureAnimationMappingStatusV1,
} from "../features/animation-mapping/types";

export type StudioInputKind = "SOURCE" | "APPEARANCE" | "ANIMATION_EVENTS";
export type StudioTarget = "CREATURE" | "PLACEABLE" | "TILE";

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
  readonly stage?: string;
  readonly path?: string;
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
  readonly target: StudioTarget;
  readonly currentStep: WorkflowStep;
  readonly lastAvailableStep: WorkflowStep;
  readonly source: StudioInputFile | null;
  readonly appearance: StudioInputFile | null;
  readonly animationEvents: StudioInputFile | null;
  readonly sourceInspection: RevisionBoundSnapshot<TInspection> | null;
  readonly appearanceInspection: RevisionBoundSnapshot<TAppearanceInspection> | null;
  readonly animationMapping: RevisionBoundSnapshot<CreatureAnimationAuthoringV1> | null;
  readonly animationMappingValidation: RevisionBoundSnapshot<{
    readonly authoringRevision: number;
    /** Present only after the canonical core/WASM validation completed. */
    readonly authoringFingerprintSha256: string | null;
    readonly status: CreatureAnimationMappingStatusV1;
    readonly diagnostics: readonly AnimationMappingDiagnosticV1[];
  }> | null;
  readonly build: BuildState<TResult>;
  readonly result: RevisionBoundSnapshot<TResult> | null;
  readonly download: DownloadState;
}

export type StudioSessionEvent<
  TInspection = unknown,
  TAppearanceInspection = unknown,
  TResult = unknown,
> =
  | { readonly type: "SOURCE_SELECTED"; readonly file: File }
  | { readonly type: "TARGET_SELECTED"; readonly target: StudioTarget }
  | { readonly type: "AUTHORING_OPTIONS_CHANGED" }
  | { readonly type: "AUTHORING_DOCUMENT_CHANGED" }
  | { readonly type: "APPEARANCE_SELECTED"; readonly file: File }
  | { readonly type: "ANIMATION_EVENTS_SELECTED"; readonly file: File }
  | { readonly type: "SOURCE_REMOVED" }
  | { readonly type: "APPEARANCE_REMOVED" }
  | { readonly type: "ANIMATION_EVENTS_REMOVED" }
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
  | { readonly type: "CONTINUE_TO_ANIMATION_MAPPING" }
  | { readonly type: "CONTINUE_TO_BUILD" }
  | { readonly type: "CONTINUE_TO_DOWNLOAD" }
  | {
      readonly type: "ANIMATION_MAPPING_INITIALIZED";
      readonly revision: number;
      readonly authoring: CreatureAnimationAuthoringV1;
    }
  | {
      readonly type: "ANIMATION_MAPPING_VALIDATED";
      readonly revision: number;
      readonly authoringRevision: number;
      readonly authoringFingerprintSha256?: string;
      readonly status: CreatureAnimationMappingStatusV1;
      readonly diagnostics: readonly AnimationMappingDiagnosticV1[];
    }
  | {
      readonly type: "BUILD_STARTED";
      readonly requestId: string;
      readonly revision: number;
    }
  | {
      readonly type: "BUILD_SUCCEEDED";
      readonly requestId: string;
      readonly revision: number;
      readonly result: TResult;
    }
  | {
      readonly type: "BUILD_FAILED";
      readonly requestId: string;
      readonly revision: number;
      readonly failure: BuildFailureSnapshot;
    }
  | {
      readonly type: "BUILD_CANCELLED";
      readonly requestId: string;
      readonly revision: number;
    }
  | { readonly type: "NAVIGATE"; readonly step: WorkflowStep }
  | { readonly type: "PROJECT_OPENED"; readonly target: StudioTarget }
  | { readonly type: "START_NEW_CONVERSION" }
  | CreatureAnimationAuthoringEventV1;

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
  target: StudioTarget = "CREATURE",
): StudioSessionState<TInspection, TResult, TAppearanceInspection> {
  return {
    revision,
    target,
    currentStep: "SOURCE",
    lastAvailableStep: "SOURCE",
    source: null,
    appearance: null,
    animationEvents: null,
    sourceInspection: null,
    appearanceInspection: null,
    animationMapping: null,
    animationMappingValidation: null,
    build: { kind: "IDLE" },
    result: null,
    download: { kind: "LOCKED" },
  };
}

function invalidateDownstream<TInspection, TResult, TAppearanceInspection>(
  state: StudioSessionState<TInspection, TResult, TAppearanceInspection>,
  inputs: Pick<
    StudioSessionState<TInspection, TResult, TAppearanceInspection>,
    "source" | "appearance" | "animationEvents"
  >,
): StudioSessionState<TInspection, TResult, TAppearanceInspection> {
  return {
    ...state,
    ...inputs,
    revision: state.revision + 1,
    currentStep: "SOURCE",
    lastAvailableStep: "SOURCE",
    sourceInspection: null,
    appearanceInspection: null,
    ...invalidateAnimationMappingAfterSourceChangeV1(),
    build: { kind: "IDLE" },
    result: null,
    download: { kind: "LOCKED" },
  };
}

export function invalidateAnimationMappingAfterSourceChangeV1(): Pick<
  StudioSessionState,
  "animationMapping" | "animationMappingValidation"
> {
  return {
    animationMapping: null,
    animationMappingValidation: null,
  };
}

function updateInputMetadata<TInspection, TResult, TAppearanceInspection>(
  state: StudioSessionState<TInspection, TResult, TAppearanceInspection>,
  event: Extract<StudioSessionEvent, { type: "INPUT_METADATA_UPDATED" }>,
): StudioSessionState<TInspection, TResult, TAppearanceInspection> {
  if (event.revision !== state.revision) return state;

  const key = event.input === "SOURCE"
    ? "source"
    : event.input === "APPEARANCE"
      ? "appearance"
      : "animationEvents";
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
  event: StudioSessionEvent<TInspection, TAppearanceInspection, TResult>,
): StudioSessionState<TInspection, TResult, TAppearanceInspection> {
  switch (event.type) {
    case "TARGET_SELECTED": {
      if (event.target === state.target) return state;
      const next = invalidateDownstream(state, {
        source: state.source,
        appearance: event.target === "TILE" ? null : state.appearance,
        animationEvents: event.target === "TILE" ? null : state.animationEvents,
      });
      return { ...next, target: event.target };
    }
    case "AUTHORING_OPTIONS_CHANGED":
      return invalidateDownstream(state, {
        source: state.source,
        appearance: state.appearance,
        animationEvents: state.animationEvents,
      });
    case "AUTHORING_DOCUMENT_CHANGED":
      return {
        ...state,
        lastAvailableStep: state.currentStep === "REVIEW" ? "BUILD" : state.lastAvailableStep,
        build: { kind: "IDLE" },
        result: null,
        download: { kind: "LOCKED" },
      };
    case "SOURCE_SELECTED":
      return invalidateDownstream(state, {
        source: selectedInput(event.file),
        appearance: state.appearance,
        animationEvents: state.animationEvents,
      });
    case "APPEARANCE_SELECTED": {
      const next = invalidateDownstream(state, {
        source: state.source,
        appearance: selectedInput(event.file),
        animationEvents: state.animationEvents,
      });
      return {
        ...next,
        target: event.file.name.toLowerCase() === "placeables.2da" ? "PLACEABLE" : "CREATURE",
      };
    }
    case "ANIMATION_EVENTS_SELECTED":
      return invalidateDownstream(state, {
        source: state.source,
        appearance: state.appearance,
        animationEvents: selectedInput(event.file),
      });
    case "SOURCE_REMOVED":
      return invalidateDownstream(state, {
        source: null,
        appearance: state.appearance,
        animationEvents: state.animationEvents,
      });
    case "APPEARANCE_REMOVED":
      return invalidateDownstream(state, {
        source: state.source,
        appearance: null,
        animationEvents: state.animationEvents,
      });
    case "ANIMATION_EVENTS_REMOVED":
      return invalidateDownstream(state, {
        source: state.source,
        appearance: state.appearance,
        animationEvents: null,
      });
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
      if (!state.source || (state.target !== "TILE" && !state.appearance)) return state;
      return {
        ...state,
        currentStep: "INSPECT",
        lastAvailableStep: compareWorkflowSteps(state.lastAvailableStep, "INSPECT") >= 0
          ? state.lastAvailableStep
          : "INSPECT",
      };
    case "CONTINUE_TO_ANIMATION_MAPPING": {
      if (
        state.target !== "CREATURE"
        || !state.source
        || !state.appearance
        || !state.source.sha256
        || state.sourceInspection?.revision !== state.revision
        || state.appearanceInspection?.revision !== state.revision
      ) return state;
      const animationMapping = isAnimationMappingCurrentV1(state)
        ? state.animationMapping
        : {
            revision: state.revision,
            value: createCreatureAnimationAuthoringV1(state.source.sha256, "S"),
          };
      return {
        ...state,
        currentStep: "ANIMATION_MAPPING",
        lastAvailableStep:
          compareWorkflowSteps(state.lastAvailableStep, "ANIMATION_MAPPING") >= 0
            ? state.lastAvailableStep
            : "ANIMATION_MAPPING",
        animationMapping,
        animationMappingValidation: isAnimationMappingCurrentV1(state)
          ? state.animationMappingValidation
          : null,
      };
    }
    case "ANIMATION_MAPPING_INITIALIZED":
      if (
        event.revision !== state.revision
        || state.target !== "CREATURE"
        || !state.source
        || !state.appearance
        || !state.source.sha256
        || event.authoring.sourceRevision !== state.source.sha256
        || state.sourceInspection?.revision !== state.revision
        || state.appearanceInspection?.revision !== state.revision
      ) return state;
      return {
        ...state,
        animationMapping: {
          revision: state.revision,
          value: event.authoring,
        },
        animationMappingValidation: null,
      };
    case "ANIMATION_SOURCE_ASSIGNED":
    case "ANIMATION_SOURCE_CLEARED":
    case "ANIMATION_FALLBACK_APPROVED":
    case "ANIMATION_FALLBACK_REJECTED":
    case "CUSTOM_ANIMATION_ADDED":
    case "CUSTOM_ANIMATION_UPDATED":
    case "CUSTOM_ANIMATION_REMOVED": {
      if (
        state.animationMapping?.revision !== state.revision
        || state.target !== "CREATURE"
      ) return state;
      const nextAuthoring = reduceCreatureAnimationAuthoringV1(
        state.animationMapping.value,
        event,
      );
      if (nextAuthoring === state.animationMapping.value) return state;
      return {
        ...state,
        lastAvailableStep: state.currentStep === "REVIEW"
          ? "BUILD"
          : state.lastAvailableStep,
        animationMapping: {
          revision: state.revision,
          value: nextAuthoring,
        },
        animationMappingValidation: null,
        build: { kind: "IDLE" },
        result: null,
        download: { kind: "LOCKED" },
      };
    }
    case "ANIMATION_MAPPING_VALIDATED":
      if (
        event.revision !== state.revision
        || state.animationMapping?.revision !== state.revision
        || state.animationMapping.value.authoringRevision !== event.authoringRevision
      ) return state;
      return {
        ...state,
        animationMappingValidation: {
          revision: state.revision,
          value: {
            authoringRevision: event.authoringRevision,
            authoringFingerprintSha256:
              event.authoringFingerprintSha256 ?? null,
            status: event.status,
            diagnostics: event.diagnostics,
          },
        },
      };
    case "CONTINUE_TO_BUILD":
      if (
        !state.source
        || (state.target !== "TILE" && !state.appearance)
        || !state.sourceInspection
        || (state.target !== "TILE" && !state.appearanceInspection)
        || state.sourceInspection.revision !== state.revision
        || (
          state.target !== "TILE"
          && state.appearanceInspection?.revision !== state.revision
        )
        || (
          state.target === "CREATURE"
          && !hasReadyCurrentAnimationMapping(state)
        )
      ) return state;
      return {
        ...state,
        currentStep: "BUILD",
        lastAvailableStep: compareWorkflowSteps(state.lastAvailableStep, "BUILD") >= 0
          ? state.lastAvailableStep
          : "BUILD",
      };
    case "BUILD_STARTED":
      if (
        event.revision !== state.revision
        || state.currentStep !== "BUILD"
        || state.build.kind === "RUNNING"
        || !state.sourceInspection
        || (state.target !== "TILE" && !state.appearanceInspection)
        || state.sourceInspection.revision !== state.revision
        || (
          state.target !== "TILE"
          && state.appearanceInspection?.revision !== state.revision
        )
        || (
          state.target === "CREATURE"
          && !hasReadyCurrentAnimationMapping(state)
        )
      ) return state;
      return {
        ...state,
        lastAvailableStep: "BUILD",
        build: {
          kind: "RUNNING",
          requestId: event.requestId,
          revision: event.revision,
        },
        result: null,
        download: { kind: "LOCKED" },
      };
    case "BUILD_SUCCEEDED": {
      if (
        event.revision !== state.revision
        || state.build.kind !== "RUNNING"
        || state.build.requestId !== event.requestId
        || state.build.revision !== event.revision
      ) return state;
      const result: RevisionBoundSnapshot<TResult> = {
        revision: event.revision,
        value: event.result,
      };
      return {
        ...state,
        currentStep: "REVIEW",
        lastAvailableStep: "REVIEW",
        build: {
          kind: "SUCCEEDED",
          requestId: event.requestId,
          revision: event.revision,
          result,
        },
        result,
        download: { kind: "LOCKED" },
      };
    }
    case "BUILD_FAILED":
      if (
        event.revision !== state.revision
        || state.build.kind !== "RUNNING"
        || state.build.requestId !== event.requestId
        || state.build.revision !== event.revision
      ) return state;
      return {
        ...state,
        currentStep: "BUILD",
        lastAvailableStep: "BUILD",
        build: {
          kind: "FAILED",
          requestId: event.requestId,
          revision: event.revision,
          failure: event.failure,
        },
        result: null,
        download: { kind: "LOCKED" },
      };
    case "BUILD_CANCELLED":
      if (
        event.revision !== state.revision
        || state.build.kind !== "RUNNING"
        || state.build.requestId !== event.requestId
        || state.build.revision !== event.revision
      ) return state;
      return {
        ...state,
        currentStep: "BUILD",
        lastAvailableStep: "BUILD",
        build: { kind: "IDLE" },
        result: null,
        download: { kind: "LOCKED" },
      };
    case "CONTINUE_TO_DOWNLOAD":
      if (
        state.currentStep !== "REVIEW"
        || state.build.kind !== "SUCCEEDED"
        || state.build.revision !== state.revision
        || state.result?.revision !== state.revision
      ) return state;
      return {
        ...state,
        currentStep: "DOWNLOAD",
        lastAvailableStep: "DOWNLOAD",
        download: { kind: "READY", revision: state.revision },
      };
    case "NAVIGATE":
      if (state.build.kind === "RUNNING") return state;
      if (!getWorkflowStepsForTarget(state.target).includes(event.step)) return state;
      if (compareWorkflowSteps(event.step, state.lastAvailableStep) > 0) return state;
      if (event.step === state.currentStep) return state;
      return { ...state, currentStep: event.step };
    case "START_NEW_CONVERSION":
      return createInitialStudioSession<TInspection, TResult, TAppearanceInspection>(state.revision + 1);
    case "PROJECT_OPENED":
      return createInitialStudioSession<TInspection, TResult, TAppearanceInspection>(
        state.revision + 1,
        event.target,
      );
  }
}

function hasReadyCurrentAnimationMapping(
  state: StudioSessionState,
): boolean {
  return isAnimationMappingCurrentV1(state)
    && state.animationMappingValidation?.revision === state.revision
    && state.animationMappingValidation.value.authoringRevision
      === state.animationMapping?.value.authoringRevision
    && state.animationMappingValidation.value.status === "READY"
    && /^[0-9a-f]{64}$/.test(
      state.animationMappingValidation.value.authoringFingerprintSha256 ?? "",
    );
}
