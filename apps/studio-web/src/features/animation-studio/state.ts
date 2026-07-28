import {
  parseAnimationStudioDocumentV1,
  serializeAnimationStudioDocumentV1,
} from "./schema";
import type {
  AnimationKeyframeV1,
  AnimationStudioDocumentV1,
  AuthoredAnimationClipV1,
  AuthoredAnimationEventV1,
} from "./types";

export type AnimationStudioModeV1 = "MAP_BASE_42" | "CREATE_EDIT";
export type AnimationStudioAutosaveStatusV1 =
  | "IDLE"
  | "SAVING"
  | "AUTOSAVED"
  | "ERROR";

export interface AnimationStudioAutosaveV1 {
  status: AnimationStudioAutosaveStatusV1;
  savedRevision: number | null;
  savedAt: string | null;
  error: string | null;
}

export interface AnimationStudioStateV1 {
  document: AnimationStudioDocumentV1;
  sourceStatus: "CURRENT" | "STALE_SOURCE";
  mode: AnimationStudioModeV1;
  selectedClipId: string | null;
  selectedBoneId: number | null;
  undoStack: string[];
  redoStack: string[];
  lastCommitGroupId: string | null;
  buildRevision: number | null;
  autosave: AnimationStudioAutosaveV1;
}

interface AnimationStudioGroupedEditV1 {
  /**
   * Repeated commits with the same group ID form one undo entry and one
   * revision. Gesture previews stay outside this reducer; only the committed
   * value should normally be sent.
   */
  historyGroupId?: string;
}

export type AnimationStudioEditCommandV1 =
  | (AnimationStudioGroupedEditV1 & {
      type: "AUTHORED_CLIP_CREATED";
      clip: AuthoredAnimationClipV1;
    })
  | (AnimationStudioGroupedEditV1 & {
      type: "AUTHORED_CLIP_UPDATED";
      clipId: string;
      patch: Partial<Omit<AuthoredAnimationClipV1, "id" | "revision">>;
    })
  | (AnimationStudioGroupedEditV1 & {
      type: "AUTHORED_CLIP_REMOVED";
      clipId: string;
      referencedByCustomAnimationIds?: readonly string[];
      confirmed?: boolean;
    })
  | (AnimationStudioGroupedEditV1 & {
      type: "KEYFRAME_INSERTED";
      clipId: string;
      trackId: string;
      keyframe: AnimationKeyframeV1;
    })
  | (AnimationStudioGroupedEditV1 & {
      type: "KEYFRAME_UPDATED";
      clipId: string;
      trackId: string;
      keyframeId: string;
      patch: Partial<Omit<AnimationKeyframeV1, "id">>;
    })
  | (AnimationStudioGroupedEditV1 & {
      type: "KEYFRAME_REMOVED";
      clipId: string;
      trackId: string;
      keyframeId: string;
    })
  | (AnimationStudioGroupedEditV1 & {
      type: "EVENT_INSERTED";
      clipId: string;
      event: AuthoredAnimationEventV1;
    })
  | (AnimationStudioGroupedEditV1 & {
      type: "EVENT_UPDATED";
      clipId: string;
      eventId: string;
      patch: Partial<Omit<AuthoredAnimationEventV1, "id">>;
    })
  | (AnimationStudioGroupedEditV1 & {
      type: "EVENT_REMOVED";
      clipId: string;
      eventId: string;
    });

export type AnimationStudioEventV1 =
  | { type: "ANIMATION_STUDIO_MODE_SELECTED"; mode: AnimationStudioModeV1 }
  | AnimationStudioEditCommandV1
  | { type: "AUTHORED_CLIP_SELECTED"; clipId: string | null }
  | { type: "BONE_SELECTED"; boneId: number | null }
  | { type: "ANIMATION_STUDIO_UNDO" }
  | { type: "ANIMATION_STUDIO_REDO" };

export function createAnimationStudioStateV1(
  document: AnimationStudioDocumentV1,
): AnimationStudioStateV1 {
  // Serialization is also the strict version gate at the state boundary.
  const exact = parseHistoryDocument(serializeAnimationStudioDocumentV1(document));
  return {
    document: exact,
    sourceStatus: "CURRENT",
    mode: "MAP_BASE_42",
    selectedClipId: null,
    selectedBoneId: null,
    undoStack: [],
    redoStack: [],
    lastCommitGroupId: null,
    buildRevision: null,
    autosave: {
      status: "IDLE",
      savedRevision: null,
      savedAt: null,
      error: null,
    },
  };
}

export function reduceAnimationStudioStateV1(
  state: AnimationStudioStateV1,
  event: AnimationStudioEventV1,
): AnimationStudioStateV1 {
  switch (event.type) {
    case "ANIMATION_STUDIO_MODE_SELECTED":
      return event.mode === state.mode
        ? state
        : { ...state, mode: event.mode, lastCommitGroupId: null };
    case "AUTHORED_CLIP_SELECTED":
      if (
        event.clipId !== null
        && !state.document.authoredClips.some(({ id }) => id === event.clipId)
      ) {
        throw new RangeError(`Unknown authored clip: ${event.clipId}`);
      }
      return {
        ...state,
        selectedClipId: event.clipId,
        selectedBoneId: null,
        lastCommitGroupId: null,
      };
    case "BONE_SELECTED":
      if (
        event.boneId !== null
        && (!Number.isSafeInteger(event.boneId) || event.boneId < 0)
      ) {
        throw new RangeError("Selected bone ID must be a non-negative integer");
      }
      return { ...state, selectedBoneId: event.boneId, lastCommitGroupId: null };
    case "ANIMATION_STUDIO_UNDO":
      return undoAnimationStudioEditV1(state);
    case "ANIMATION_STUDIO_REDO":
      return redoAnimationStudioEditV1(state);
    default:
      return commitAnimationStudioEditV1(state, event);
  }
}

export function commitAnimationStudioEditV1(
  state: AnimationStudioStateV1,
  command: AnimationStudioEditCommandV1,
): AnimationStudioStateV1 {
  const groupId = command.historyGroupId?.trim() || null;
  const coalescing = groupId !== null && groupId === state.lastCommitGroupId;
  const nextDocument = applyEditCommand(state.document, command, !coalescing);
  const currentJson = serializeAnimationStudioDocumentV1(state.document);
  const nextJson = serializeAnimationStudioDocumentV1(nextDocument);
  if (currentJson === nextJson) return state;

  return {
    ...state,
    document: nextDocument,
    selectedClipId: nextSelectedClipId(state, command),
    undoStack: coalescing
      ? state.undoStack
      : [...state.undoStack, currentJson],
    redoStack: [],
    lastCommitGroupId: groupId,
    autosave: {
      status: "IDLE",
      savedRevision: state.autosave.savedRevision,
      savedAt: state.autosave.savedAt,
      error: null,
    },
  };
}

/**
 * Commits a complete editor projection as one logical history entry.
 *
 * Complex timeline operations (trim, retime and a completed gizmo gesture)
 * are computed as one document projection by the UI, then cross the state
 * boundary once through this function. Preview-only updates never call it.
 */
export function commitAnimationStudioDocumentV1(
  state: AnimationStudioStateV1,
  document: AnimationStudioDocumentV1,
): AnimationStudioStateV1 {
  const normalized = parseHistoryDocument(
    serializeAnimationStudioDocumentV1({
      ...document,
      authoringRevision: state.document.authoringRevision + 1,
    }),
  );
  const currentJson = serializeAnimationStudioDocumentV1(state.document);
  const nextJson = serializeAnimationStudioDocumentV1(normalized);
  if (currentJson === nextJson) return state;
  return {
    ...state,
    document: normalized,
    selectedClipId: retainExistingClipSelection(
      normalized,
      state.selectedClipId,
    ),
    undoStack: [...state.undoStack, currentJson],
    redoStack: [],
    lastCommitGroupId: null,
    autosave: staleAutosave(state.autosave),
  };
}

export function undoAnimationStudioEditV1(
  state: AnimationStudioStateV1,
): AnimationStudioStateV1 {
  const previousJson = state.undoStack.at(-1);
  if (previousJson === undefined) return state;
  const previous = parseHistoryDocument(previousJson);
  return {
    ...state,
    document: previous,
    selectedClipId: retainExistingClipSelection(previous, state.selectedClipId),
    undoStack: state.undoStack.slice(0, -1),
    redoStack: [
      serializeAnimationStudioDocumentV1(state.document),
      ...state.redoStack,
    ],
    lastCommitGroupId: null,
    autosave: staleAutosave(state.autosave),
  };
}

export function redoAnimationStudioEditV1(
  state: AnimationStudioStateV1,
): AnimationStudioStateV1 {
  const nextJson = state.redoStack[0];
  if (nextJson === undefined) return state;
  const next = parseHistoryDocument(nextJson);
  return {
    ...state,
    document: next,
    selectedClipId: retainExistingClipSelection(next, state.selectedClipId),
    undoStack: [
      ...state.undoStack,
      serializeAnimationStudioDocumentV1(state.document),
    ],
    redoStack: state.redoStack.slice(1),
    lastCommitGroupId: null,
    autosave: staleAutosave(state.autosave),
  };
}

export function markAnimationStudioBuildRevisionV1(
  state: AnimationStudioStateV1,
  revision: number,
): AnimationStudioStateV1 {
  if (!Number.isSafeInteger(revision) || revision < 1) {
    throw new RangeError("Build revision must be a positive integer");
  }
  return { ...state, buildRevision: revision };
}

export function isAnimationStudioBuildCurrentV1(
  state: AnimationStudioStateV1,
): boolean {
  return state.buildRevision === state.document.authoringRevision
    && state.sourceStatus === "CURRENT"
    && state.autosave.status === "AUTOSAVED"
    && state.autosave.savedRevision === state.document.authoringRevision;
}

export function reconcileAnimationStudioSourceRevisionV1(
  state: AnimationStudioStateV1,
  currentSourceRevision: string,
): AnimationStudioStateV1 {
  const sourceStatus = currentSourceRevision === state.document.sourceRevision
    ? "CURRENT"
    : "STALE_SOURCE";
  return sourceStatus === state.sourceStatus
    ? state
    : {
        ...state,
        sourceStatus,
        lastCommitGroupId: null,
      };
}

export function markAnimationStudioAutosaveSavingV1(
  state: AnimationStudioStateV1,
): AnimationStudioStateV1 {
  return {
    ...state,
    autosave: {
      ...state.autosave,
      status: "SAVING",
      error: null,
    },
  };
}

export function markAnimationStudioAutosavedV1(
  state: AnimationStudioStateV1,
  revision: number,
  savedAt: string,
): AnimationStudioStateV1 {
  if (revision !== state.document.authoringRevision) {
    // A late write completion must not mark a newer edit as saved.
    return {
      ...state,
      autosave: {
        ...state.autosave,
        status: "IDLE",
        error: null,
      },
    };
  }
  return {
    ...state,
    autosave: {
      status: "AUTOSAVED",
      savedRevision: revision,
      savedAt,
      error: null,
    },
  };
}

export function markAnimationStudioAutosaveFailedV1(
  state: AnimationStudioStateV1,
  message: string,
): AnimationStudioStateV1 {
  return {
    ...state,
    autosave: {
      ...state.autosave,
      status: "ERROR",
      error: message,
    },
  };
}

function applyEditCommand(
  document: AnimationStudioDocumentV1,
  command: AnimationStudioEditCommandV1,
  revise: boolean,
): AnimationStudioDocumentV1 {
  let clips = document.authoredClips;
  switch (command.type) {
    case "AUTHORED_CLIP_CREATED":
      if (clips.some(({ id }) => id === command.clip.id)) {
        throw new RangeError(`Duplicate authored clip ID: ${command.clip.id}`);
      }
      clips = [...clips, structuredClone(command.clip)];
      break;
    case "AUTHORED_CLIP_UPDATED":
      clips = replaceClip(clips, command.clipId, (clip) => ({
        ...clip,
        ...structuredClone(command.patch),
        id: clip.id,
        revision: revise ? clip.revision + 1 : clip.revision,
      }));
      break;
    case "AUTHORED_CLIP_REMOVED": {
      requireClip(clips, command.clipId);
      const usages = command.referencedByCustomAnimationIds ?? [];
      if (usages.length > 0 && command.confirmed !== true) {
        throw new RangeError(
          `Authored clip ${command.clipId} is in use and requires confirmation`,
        );
      }
      clips = clips.filter(({ id }) => id !== command.clipId);
      break;
    }
    case "KEYFRAME_INSERTED":
      clips = replaceClip(clips, command.clipId, (clip) => {
        const track = requireTrack(clip, command.trackId);
        if (track.keyframes.some(({ id }) => id === command.keyframe.id)) {
          throw new RangeError(`Duplicate keyframe ID: ${command.keyframe.id}`);
        }
        return replaceTrack(clip, command.trackId, {
          ...track,
          keyframes: [...track.keyframes, structuredClone(command.keyframe)],
        }, revise);
      });
      break;
    case "KEYFRAME_UPDATED":
      clips = replaceClip(clips, command.clipId, (clip) => {
        const track = requireTrack(clip, command.trackId);
        if (!track.keyframes.some(({ id }) => id === command.keyframeId)) {
          throw new RangeError(`Unknown keyframe: ${command.keyframeId}`);
        }
        return replaceTrack(clip, command.trackId, {
          ...track,
          keyframes: track.keyframes.map((keyframe) => (
            keyframe.id === command.keyframeId
              ? { ...keyframe, ...structuredClone(command.patch), id: keyframe.id }
              : keyframe
          )),
        }, revise);
      });
      break;
    case "KEYFRAME_REMOVED":
      clips = replaceClip(clips, command.clipId, (clip) => {
        const track = requireTrack(clip, command.trackId);
        if (!track.keyframes.some(({ id }) => id === command.keyframeId)) {
          throw new RangeError(`Unknown keyframe: ${command.keyframeId}`);
        }
        return replaceTrack(clip, command.trackId, {
          ...track,
          keyframes: track.keyframes.filter(({ id }) => id !== command.keyframeId),
        }, revise);
      });
      break;
    case "EVENT_INSERTED":
      clips = replaceClip(clips, command.clipId, (clip) => {
        if (clip.events.some(({ id }) => id === command.event.id)) {
          throw new RangeError(`Duplicate event ID: ${command.event.id}`);
        }
        return {
          ...clip,
          events: [...clip.events, structuredClone(command.event)],
          revision: revise ? clip.revision + 1 : clip.revision,
        };
      });
      break;
    case "EVENT_UPDATED":
      clips = replaceClip(clips, command.clipId, (clip) => {
        if (!clip.events.some(({ id }) => id === command.eventId)) {
          throw new RangeError(`Unknown event: ${command.eventId}`);
        }
        return {
          ...clip,
          events: clip.events.map((event) => (
            event.id === command.eventId
              ? { ...event, ...structuredClone(command.patch), id: event.id }
              : event
          )),
          revision: revise ? clip.revision + 1 : clip.revision,
        };
      });
      break;
    case "EVENT_REMOVED":
      clips = replaceClip(clips, command.clipId, (clip) => {
        if (!clip.events.some(({ id }) => id === command.eventId)) {
          throw new RangeError(`Unknown event: ${command.eventId}`);
        }
        return {
          ...clip,
          events: clip.events.filter(({ id }) => id !== command.eventId),
          revision: revise ? clip.revision + 1 : clip.revision,
        };
      });
      break;
  }

  if (sameEditPayload(document.authoredClips, clips)) {
    return document;
  }

  return {
    ...document,
    status: "DRAFT",
    authoredClips: clips,
    authoringRevision: revise
      ? document.authoringRevision + 1
      : document.authoringRevision,
  };
}

function sameEditPayload(
  before: readonly AuthoredAnimationClipV1[],
  after: readonly AuthoredAnimationClipV1[],
): boolean {
  const withoutRevisions = (clips: readonly AuthoredAnimationClipV1[]) => (
    clips.map(({ revision: _revision, ...clip }) => clip)
  );
  return JSON.stringify(withoutRevisions(before))
    === JSON.stringify(withoutRevisions(after));
}

function replaceClip(
  clips: readonly AuthoredAnimationClipV1[],
  clipId: string,
  update: (clip: AuthoredAnimationClipV1) => AuthoredAnimationClipV1,
): AuthoredAnimationClipV1[] {
  requireClip(clips, clipId);
  return clips.map((clip) => clip.id === clipId ? update(clip) : clip);
}

function requireClip(
  clips: readonly AuthoredAnimationClipV1[],
  clipId: string,
): AuthoredAnimationClipV1 {
  const clip = clips.find(({ id }) => id === clipId);
  if (!clip) throw new RangeError(`Unknown authored clip: ${clipId}`);
  return clip;
}

function requireTrack(clip: AuthoredAnimationClipV1, trackId: string) {
  const track = clip.tracks.find(({ id }) => id === trackId);
  if (!track) throw new RangeError(`Unknown authored track: ${trackId}`);
  return track;
}

function replaceTrack(
  clip: AuthoredAnimationClipV1,
  trackId: string,
  replacement: AuthoredAnimationClipV1["tracks"][number],
  revise: boolean,
): AuthoredAnimationClipV1 {
  return {
    ...clip,
    tracks: clip.tracks.map((track) => (
      track.id === trackId ? replacement : track
    )),
    revision: revise ? clip.revision + 1 : clip.revision,
  };
}

function nextSelectedClipId(
  state: AnimationStudioStateV1,
  command: AnimationStudioEditCommandV1,
): string | null {
  if (command.type === "AUTHORED_CLIP_CREATED") return command.clip.id;
  if (
    command.type === "AUTHORED_CLIP_REMOVED"
    && state.selectedClipId === command.clipId
  ) return null;
  return state.selectedClipId;
}

function retainExistingClipSelection(
  document: AnimationStudioDocumentV1,
  selectedClipId: string | null,
): string | null {
  return selectedClipId !== null
    && document.authoredClips.some(({ id }) => id === selectedClipId)
    ? selectedClipId
    : null;
}

function parseHistoryDocument(json: string): AnimationStudioDocumentV1 {
  const parsed = parseAnimationStudioDocumentV1(json);
  if (parsed.kind === "INVALID") {
    throw new Error("Internal Animation Studio history is invalid");
  }
  return parsed.value;
}

function staleAutosave(
  autosave: AnimationStudioAutosaveV1,
): AnimationStudioAutosaveV1 {
  return {
    status: "IDLE",
    savedRevision: autosave.savedRevision,
    savedAt: autosave.savedAt,
    error: null,
  };
}
