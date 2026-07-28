import type { DirectCreatureBaseSlotV1 } from "../animation-mapping/types";
import { canonicalFloat32ForWireV1 } from "../animation-studio/schema";
import type {
  AnimationKeyframeV1,
  AnimationStudioDiagnosticV1,
  AnimationStudioDocumentV1,
  AuthoredAnimationClipV1,
  AuthoredAnimationEventV1,
  AuthoredAnimationTrackPathV1,
  AuthoredAnimationTrackV1,
  CreatureAnimationAuthoringV2,
  CustomAnimationClipReferenceV2,
  CustomAnimationDefinitionV2,
  CustomAnimationLibraryItemV1,
} from "../animation-studio/types";
import type { AnimationRigNodeV1 } from "./AnimationBoneTree";

export interface AnimationStudioLibrarySourceV1 {
  readonly clipId: string;
  readonly name: string;
  readonly durationSeconds: number;
  readonly trackCount: number;
}

export interface AnimationStudioLibraryItemV1 {
  readonly id: string;
  readonly name: string;
  readonly origin: "SOURCE" | "EDITED" | "GENERATED";
  readonly status: "SOURCE" | "DRAFT" | "VALID" | "INVALID";
  readonly durationSeconds: number;
  readonly keyframeCount: number;
  readonly authoredClipId: string | null;
  readonly diagnosticCodes: readonly string[];
}

export interface ProjectedCustomAnimationLibraryItemV1
  extends CustomAnimationLibraryItemV1 {
  readonly id: string;
  readonly playback: CustomAnimationDefinitionV2["playback"];
  readonly authoredClipIds: string[];
  readonly openableInEditor: boolean;
  readonly sourceClipCount: number;
  readonly durationSeconds: number;
  readonly keyframeCount: number;
  readonly assignmentCount: number;
  readonly assignedSlots: DirectCreatureBaseSlotV1[];
  readonly assignable: boolean;
  readonly diagnosticCodes: string[];
}

export interface CreateBlankPoseClipInputV1 {
  readonly id: string;
  readonly name: string;
  readonly sourceRevision: string;
  readonly animationRoot: string;
  readonly lengthSeconds?: number;
  readonly rig?: readonly AnimationRigNodeV1[];
}

export type ProceduralAnimationTemplateV1 =
  | "BIND_POSE"
  | "ROOT_TRANSLATION_PULSE";

export function createBlankPoseClipV1(
  input: CreateBlankPoseClipInputV1,
): AuthoredAnimationClipV1 {
  return {
    id: input.id,
    name: input.name,
    kind: "STATIC_POSE",
    status: "DRAFT",
    source: {
      kind: "BLANK_POSE",
      sourceRevision: input.sourceRevision,
      sourceClipName: null,
      sourceClipFingerprint: null,
      proceduralTemplate: null,
    },
    lengthSeconds: input.lengthSeconds ?? 1,
    transitionSeconds: 0.1,
    animationRoot: input.animationRoot,
    tracks: (input.rig ?? []).flatMap((node): AuthoredAnimationTrackV1[] => [
      {
        id: `track-${node.id}-translation`,
        targetNodeId: node.id,
        path: "TRANSLATION",
        interpolation: "LINEAR",
        keyframes: [{
          id: "key-0000",
          timeSeconds: 0,
          value: [...node.translation],
        }],
      },
      {
        id: `track-${node.id}-rotation`,
        targetNodeId: node.id,
        path: "ROTATION",
        interpolation: "LINEAR",
        keyframes: [{
          id: "key-0000",
          timeSeconds: 0,
          value: normalizeQuaternionV1(node.rotation),
        }],
      },
    ]),
    events: [],
    revision: 1,
  };
}

/**
 * UI-side projection of the canonical core templates. The resulting document
 * still crosses the worker/core validation boundary before it can be built.
 */
export function createProceduralTemplateClipV1(
  input: CreateBlankPoseClipInputV1,
  template: ProceduralAnimationTemplateV1,
): AuthoredAnimationClipV1 {
  const clip = createBlankPoseClipV1(input);
  const source = {
    ...clip.source,
    kind: "PROCEDURAL_TEMPLATE" as const,
    proceduralTemplate: template,
  };
  if (template === "BIND_POSE") return { ...clip, source };

  const rig = input.rig ?? [];
  const root = rig.find(({ name }) => name === input.animationRoot) ?? rig[0];
  if (!root) {
    throw new Error("Procedural template requires an output-rig root node.");
  }
  const rootTranslation = clip.tracks.find((track) => (
    track.targetNodeId === root.id && track.path === "TRANSLATION"
  ));
  if (!rootTranslation) {
    throw new Error("Procedural template requires a root translation track.");
  }
  const base = rootTranslation.keyframes[0]?.value;
  if (!base || base.length !== 3) {
    throw new Error("Procedural template root translation must have three components.");
  }
  return {
    ...clip,
    kind: "MOTION",
    source,
    tracks: clip.tracks.map((track) => track.id === rootTranslation.id
      ? {
          ...track,
          interpolation: "LINEAR",
          keyframes: [{
            id: "key-0000",
            timeSeconds: 0,
            value: [...base],
          }, {
            id: "key-0001",
            timeSeconds: clip.lengthSeconds * 0.5,
            value: [base[0]! + 0.1, base[1]!, base[2]!],
          }, {
            id: "key-0002",
            timeSeconds: clip.lengthSeconds,
            value: [...base],
          }],
        }
      : track),
  };
}

export function cloneSourceClipForEditingV1(
  source: AuthoredAnimationClipV1,
  input: { readonly id: string; readonly name: string },
): AuthoredAnimationClipV1 {
  return {
    ...structuredClone(source),
    id: input.id,
    name: input.name,
    status: "DRAFT",
    source: {
      ...source.source,
      kind: "SOURCE_CLIP_COPY",
    },
    revision: 1,
  };
}

export function projectAnimationStudioLibraryV1(
  document: AnimationStudioDocumentV1,
  sourceInventory: readonly AnimationStudioLibrarySourceV1[],
): AnimationStudioLibraryItemV1[] {
  return sortAnimationStudioLibraryV1([
    ...sourceInventory.map((source): AnimationStudioLibraryItemV1 => ({
      id: `source:${source.clipId}`,
      name: source.name,
      origin: "SOURCE",
      status: "SOURCE",
      durationSeconds: source.durationSeconds,
      keyframeCount: source.trackCount,
      authoredClipId: null,
      diagnosticCodes: [],
    })),
    ...document.authoredClips.map((clip): AnimationStudioLibraryItemV1 => ({
      id: clip.id,
      name: clip.name,
      origin: clip.source.kind === "PROCEDURAL_TEMPLATE" ? "GENERATED" : "EDITED",
      status: clip.status,
      durationSeconds: clip.lengthSeconds,
      keyframeCount: clip.tracks.reduce(
        (sum, track) => sum + track.keyframes.length,
        0,
      ),
      authoredClipId: clip.id,
      diagnosticCodes: clip.status === "INVALID"
        ? ["M2A-ANIMATION-EDIT-INVALID"]
        : [],
    })),
  ]);
}

export function filterAnimationStudioLibraryV1(
  items: readonly AnimationStudioLibraryItemV1[],
  query: string,
  filter: "ALL" | "BASE_42" | "CUSTOM",
): AnimationStudioLibraryItemV1[] {
  const normalized = query.trim().toLocaleLowerCase("en-US");
  return items.filter((item) => (
    (filter === "ALL"
      || (filter === "BASE_42" && item.origin === "SOURCE")
      || (filter === "CUSTOM" && item.origin !== "SOURCE"))
    && (!normalized || item.name.toLocaleLowerCase("en-US").includes(normalized))
  ));
}

export function sortAnimationStudioLibraryV1(
  items: readonly AnimationStudioLibraryItemV1[],
): AnimationStudioLibraryItemV1[] {
  return [...items].sort((left, right) => (
    left.origin.localeCompare(right.origin)
    || left.name.localeCompare(right.name, "en-US")
    || left.id.localeCompare(right.id)
  ));
}

export function getAnimationStudioClipActionsV1(
  item: AnimationStudioLibraryItemV1,
  usage: readonly DirectCreatureBaseSlotV1[],
): readonly ("EDIT_COPY" | "DUPLICATE" | "REMOVE" | "OPEN")[] {
  if (item.origin === "SOURCE") return ["EDIT_COPY"];
  return usage.length > 0
    ? ["OPEN", "DUPLICATE", "REMOVE"]
    : ["OPEN", "DUPLICATE", "REMOVE"];
}

export function getAuthoredClipUsageV1(
  clipId: string,
  authoring: CreatureAnimationAuthoringV2,
): DirectCreatureBaseSlotV1[] {
  const customIds = new Set(authoring.customAnimations
    .filter((custom) => customReferencesClip(custom, clipId))
    .map(({ id }) => id));
  return authoring.assignments
    .filter((assignment) => (
      assignment.sourceKind === "CUSTOM"
      && assignment.customAnimationId !== null
      && customIds.has(assignment.customAnimationId)
    ))
    .map(({ targetSlot }) => targetSlot);
}

export function normalizeQuaternionV1(
  value: readonly number[],
): [number, number, number, number] {
  if (value.length !== 4 || value.some((component) => !Number.isFinite(component))) {
    throw new Error("Quaternion must contain four finite components.");
  }
  const projected = value.map(Math.fround);
  const squaredNorm = projected.reduce((sum, component) => (
    Math.fround(sum + Math.fround(component * component))
  ), 0);
  const length = Math.fround(Math.sqrt(squaredNorm));
  if (length <= 1e-6) {
    throw new Error("Quaternion must have a norm greater than 1e-6.");
  }
  const normalized = projected.map((component) => (
    canonicalFloat32ForWireV1(Math.fround(component / length))
  )) as [
    number,
    number,
    number,
    number,
  ];
  const firstVectorNonZero = normalized
    .slice(0, 3)
    .find((component) => component !== 0);
  const flip = normalized[3] < 0
    || (normalized[3] === 0
      && firstVectorNonZero !== undefined
      && firstVectorNonZero < 0);
  const canonical = flip
    ? normalized.map((component) => -component)
    : normalized;
  return canonical.map((component) => (
    Object.is(component, -0) ? 0 : component
  )) as [number, number, number, number];
}

export function normalizeAnimationQuaternionsV1(
  clip: AuthoredAnimationClipV1,
  trackId: string,
): AuthoredAnimationClipV1 {
  const found = clip.tracks.find(({ id }) => id === trackId);
  if (!found) throw new Error(`Animation track does not exist: ${trackId}`);
  if (found.path !== "ROTATION") {
    throw new Error("Quaternion normalization requires a rotation track.");
  }
  return committedClip(clip, {
    tracks: clip.tracks.map((track) => (
      track.id === trackId
        ? {
            ...track,
            keyframes: track.keyframes.map((keyframe) => ({
              ...keyframe,
              value: normalizeQuaternionV1(keyframe.value),
            })),
          }
        : track
    )),
  });
}

export function sampleAnimationTrackLinearV1(
  track: AuthoredAnimationTrackV1,
  timeSeconds: number,
): number[] {
  const keys = track.keyframes;
  if (keys.length === 0) throw new Error("Cannot sample an empty animation track.");
  const previous = [...keys].reverse()
    .find((keyframe) => keyframe.timeSeconds <= timeSeconds) ?? keys[0]!;
  const next = keys.find((keyframe) => keyframe.timeSeconds >= timeSeconds)
    ?? keys[keys.length - 1]!;
  if (previous.id === next.id || previous.timeSeconds === next.timeSeconds) {
    return [...previous.value];
  }
  const sampledTime = Math.fround(timeSeconds);
  const span = Math.fround(next.timeSeconds - previous.timeSeconds);
  const alpha = Math.fround(
    Math.fround(sampledTime - previous.timeSeconds) / span,
  );
  const sampled = previous.value.map((component, index) => (
    canonicalFloat32ForWireV1(Math.fround(
      Math.fround(component)
      + Math.fround(
        Math.fround((next.value[index] ?? component) - component) * alpha,
      ),
    ))
  ));
  return track.path === "ROTATION" ? normalizeQuaternionV1(sampled) : sampled;
}

export function insertKeyAtPlayheadV1(
  clip: AuthoredAnimationClipV1,
  selection: {
    readonly targetNodeId: number;
    readonly path: AuthoredAnimationTrackPathV1;
    readonly value: readonly number[];
  },
  timeSeconds: number,
  keyId = uniqueId("key"),
  collisionPolicy: "FAIL" | "REPLACE_EXISTING" = "FAIL",
): AuthoredAnimationClipV1 {
  const projectedTime = canonicalFloat32ForWireV1(timeSeconds);
  assertClipTime(projectedTime, clip.lengthSeconds);
  const value = selection.path === "ROTATION"
    ? normalizeQuaternionV1(selection.value)
    : selection.value.map(canonicalFloat32ForWireV1);
  const existingIndex = clip.tracks.findIndex((track) => (
    track.targetNodeId === selection.targetNodeId && track.path === selection.path
  ));
  const tracks = structuredClone(clip.tracks);
  const track: AuthoredAnimationTrackV1 = existingIndex >= 0
    ? tracks[existingIndex]!
    : {
        id: uniqueId("track"),
        targetNodeId: selection.targetNodeId,
        path: selection.path,
        interpolation: "LINEAR",
        keyframes: [],
      };
  const duplicateTime = track.keyframes.findIndex(
    (keyframe) => Math.abs(keyframe.timeSeconds - projectedTime) < 1e-7,
  );
  const keyframe = { id: keyId, timeSeconds: projectedTime, value };
  if (duplicateTime >= 0) {
    if (collisionPolicy === "FAIL") {
      throw new Error(
        "A keyframe already exists at this time. Choose an explicit replace policy.",
      );
    }
    track.keyframes[duplicateTime] = {
      ...keyframe,
      id: track.keyframes[duplicateTime]!.id,
    };
  } else {
    track.keyframes.push(keyframe);
  }
  track.keyframes.sort((left, right) => left.timeSeconds - right.timeSeconds);
  if (existingIndex < 0) tracks.push(track);
  return committedClip(clip, { tracks, kind: "MOTION", status: "DRAFT" });
}

export function moveSelectedKeysV1(
  clip: AuthoredAnimationClipV1,
  keyIds: readonly string[],
  delta: number,
): AuthoredAnimationClipV1 {
  const selected = new Set(keyIds);
  const projectedDelta = Math.fround(delta);
  const tracks = clip.tracks.map((track) => {
    const keyframes = track.keyframes.map((keyframe) => (
      isAnimationKeySelectedV1(selected, track.id, keyframe.id)
        ? {
            ...keyframe,
            timeSeconds: canonicalFloat32ForWireV1(Math.fround(clamp(
              Math.fround(keyframe.timeSeconds + projectedDelta),
              0,
              clip.lengthSeconds,
            ))),
          }
        : { ...keyframe }
    )).sort((left, right) => left.timeSeconds - right.timeSeconds);
    assertStrictTimes(keyframes);
    return { ...track, keyframes };
  });
  return committedClip(clip, { tracks });
}

export function deleteSelectedKeysV1(
  clip: AuthoredAnimationClipV1,
  keyIds: readonly string[],
): AuthoredAnimationClipV1 {
  const selected = new Set(keyIds);
  return committedClip(clip, {
    tracks: clip.tracks
      .map((track) => ({
        ...track,
        keyframes: track.keyframes.filter(({ id }) => (
          !isAnimationKeySelectedV1(selected, track.id, id)
        )),
      }))
      .filter(({ keyframes }) => keyframes.length > 0),
    status: "DRAFT",
  });
}

export function selectKeysInRangeV1(
  clip: AuthoredAnimationClipV1,
  range: { readonly start: number; readonly end: number },
): string[] {
  const start = Math.min(range.start, range.end);
  const end = Math.max(range.start, range.end);
  return clip.tracks.flatMap((track) => track.keyframes
    .filter(({ timeSeconds }) => timeSeconds >= start && timeSeconds <= end)
    .map(({ id }) => animationKeySelectionIdV1(track.id, id)));
}

export function animationKeySelectionIdV1(
  trackId: string,
  keyframeId: string,
) {
  return `${encodeURIComponent(trackId)}/${encodeURIComponent(keyframeId)}`;
}

function isAnimationKeySelectedV1(
  selected: ReadonlySet<string>,
  trackId: string,
  keyframeId: string,
) {
  return selected.has(animationKeySelectionIdV1(trackId, keyframeId))
    // Backwards-compatible command payloads with globally unique key IDs.
    || selected.has(keyframeId);
}

export function collectAnimationKeyTimesV1(
  clip: AuthoredAnimationClipV1,
): number[] {
  return [...new Set([
    ...clip.tracks.flatMap((track) => track.keyframes.map(({ timeSeconds }) => timeSeconds)),
    ...clip.events.map(({ timeSeconds }) => timeSeconds),
  ])].sort((left, right) => left - right);
}

export function trimAnimationEditorSelectionV1(
  clip: AuthoredAnimationClipV1,
  start: number,
  end: number,
): AuthoredAnimationClipV1 {
  const projectedStart = canonicalFloat32ForWireV1(start);
  const projectedEnd = canonicalFloat32ForWireV1(end);
  if (!Number.isFinite(projectedStart) || !Number.isFinite(projectedEnd) || projectedStart < 0 || projectedEnd > clip.lengthSeconds || projectedStart >= projectedEnd) {
    throw new Error("Trim range must be finite, ordered and inside the clip.");
  }
  const newLength = canonicalFloat32ForWireV1(
    Math.fround(projectedEnd - projectedStart),
  );
  const tracks = clip.tracks.map((track) => {
    const boundaryStart = sampleAnimationTrackLinearV1(track, projectedStart);
    const boundaryEnd = sampleAnimationTrackLinearV1(track, projectedEnd);
    const existingStart = track.keyframes.find(
      ({ timeSeconds }) => Math.abs(timeSeconds - projectedStart) < 1e-7,
    );
    const existingEnd = track.keyframes.find(
      ({ timeSeconds }) => Math.abs(timeSeconds - projectedEnd) < 1e-7,
    );
    const keyframes = [
      {
        id: existingStart?.id ?? "trim-start",
        timeSeconds: 0,
        value: boundaryStart,
      },
      ...track.keyframes
        .filter(({ timeSeconds }) => (
          timeSeconds > projectedStart && timeSeconds < projectedEnd
        ))
        .map((keyframe) => ({
          ...keyframe,
          timeSeconds: canonicalFloat32ForWireV1(
            Math.fround(keyframe.timeSeconds - projectedStart),
          ),
        })),
      {
        id: existingEnd?.id ?? "trim-end",
        timeSeconds: newLength,
        value: boundaryEnd,
      },
    ];
    return { ...track, keyframes: deduplicateTimes(keyframes) };
  });
  const events = clip.events
    .filter(({ timeSeconds }) => (
      timeSeconds >= projectedStart && timeSeconds <= projectedEnd
    ))
    .map((event) => ({
      ...event,
      timeSeconds: canonicalFloat32ForWireV1(
        Math.fround(event.timeSeconds - projectedStart),
      ),
    }));
  return committedClip(clip, {
    lengthSeconds: newLength,
    tracks,
    events,
    status: "DRAFT",
  });
}

export function retimeAnimationEditorSelectionV1(
  clip: AuthoredAnimationClipV1,
  newLength: number,
): AuthoredAnimationClipV1 {
  const projectedLength = canonicalFloat32ForWireV1(newLength);
  if (!Number.isFinite(projectedLength) || projectedLength <= 0) {
    throw new Error("Animation length must be finite and positive.");
  }
  const scale = clip.lengthSeconds <= Number.EPSILON
    ? 1
    : Math.fround(projectedLength / clip.lengthSeconds);
  return committedClip(clip, {
    lengthSeconds: projectedLength,
    tracks: clip.tracks.map((track) => ({
      ...track,
      keyframes: track.keyframes.map((keyframe) => ({
        ...keyframe,
        timeSeconds: canonicalFloat32ForWireV1(
          Math.fround(keyframe.timeSeconds * scale),
        ),
      })),
    })),
    events: clip.events.map((event) => ({
      ...event,
      timeSeconds: canonicalFloat32ForWireV1(
        Math.fround(event.timeSeconds * scale),
      ),
    })),
    status: "DRAFT",
  });
}

export function snapAnimationTimeV1(
  time: number,
  snapPolicy: "NONE" | "FRAME_30",
): number {
  return snapPolicy === "FRAME_30" ? Math.round(time * 30) / 30 : time;
}

export function zoomAnimationTimelineV1(
  view: { readonly pixelsPerSecond: number; readonly offsetSeconds: number },
  delta: number,
) {
  return {
    ...view,
    pixelsPerSecond: clamp(view.pixelsPerSecond * (delta > 0 ? 1.1 : 0.9), 24, 400),
  };
}

export function panAnimationTimelineV1(
  view: { readonly pixelsPerSecond: number; readonly offsetSeconds: number },
  delta: number,
) {
  return { ...view, offsetSeconds: Math.max(0, view.offsetSeconds + delta) };
}

export function createAnimationEventDraftV1(
  timeSeconds: number,
): AuthoredAnimationEventV1 {
  return { id: uniqueId("event"), timeSeconds, name: "" };
}

export function validateAnimationEventDraftV1(
  event: AuthoredAnimationEventV1,
  clip: AuthoredAnimationClipV1,
): AnimationStudioDiagnosticV1[] {
  const validName = event.name.length > 0
    && event.name.length <= 31
    && /^[\x20-\x7e]+$/.test(event.name)
    && !event.name.includes("\0");
  const validTime = Number.isFinite(event.timeSeconds)
    && event.timeSeconds >= 0
    && event.timeSeconds <= clip.lengthSeconds;
  return [
    ...(validName ? [] : [diagnostic(
      "M2A-ANIMATION-EDIT-EVENT",
      `events.${event.id}.name`,
      "Event name must be 1-31 printable ASCII characters.",
      "Enter a short ASCII callback name.",
    )]),
    ...(validTime ? [] : [diagnostic(
      "M2A-ANIMATION-EDIT-TIME-OOB",
      `events.${event.id}.timeSeconds`,
      "Event time must be inside the clip.",
      "Move the event between 0 and clip length.",
    )]),
  ];
}

export function projectAnimationEventMarkersV1(
  events: readonly AuthoredAnimationEventV1[],
) {
  return events.map((event, order) => ({ ...event, order }));
}

export function projectCustomAnimationLibraryV1(
  authoring: CreatureAnimationAuthoringV2,
  studio: AnimationStudioDocumentV1,
  sourceInventory: readonly AnimationStudioLibrarySourceV1[] = [],
): ProjectedCustomAnimationLibraryItemV1[] {
  const clips = new Map(studio.authoredClips.map((clip) => [clip.id, clip]));
  const sourceClips = new Map(sourceInventory.map((clip) => [
    clip.name.toLocaleLowerCase("en-US"),
    clip,
  ]));
  return authoring.customAnimations.map((custom) => {
    const references = customClipReferences(custom);
    const authoredClipIds = customAuthoredClipIds(custom);
    const statuses = references.map((reference) => (
      reference.sourceKind === "AUTHORED_CLIP"
        ? clips.get(reference.authoredClipId ?? "")?.status ?? "INVALID"
        : sourceClips.has(
            (reference.sourceClipName ?? "").toLocaleLowerCase("en-US"),
          )
          ? "VALID"
          : "INVALID"
    ));
    const status = statuses.includes("INVALID")
      ? "INVALID"
      : statuses.includes("DRAFT") || references.length === 0
        ? "DRAFT"
        : "VALID";
    const assignmentCount = authoring.assignments
      .filter(({ customAnimationId }) => customAnimationId === custom.id)
      .length;
    const assignedSlots = authoring.assignments
      .filter(({ customAnimationId }) => customAnimationId === custom.id)
      .map(({ targetSlot }) => targetSlot);
    const referencedClips = authoredClipIds.flatMap((id) => {
      const clip = clips.get(id);
      return clip ? [clip] : [];
    });
    const referencedSourceClips = references.flatMap((reference) => {
      if (reference.sourceKind !== "SOURCE_CLIP") return [];
      const clip = sourceClips.get(
        (reference.sourceClipName ?? "").toLocaleLowerCase("en-US"),
      );
      return clip ? [clip] : [];
    });
    return {
      authoredClipId: authoredClipIds[0] ?? "",
      revision: authoredClipIds.reduce(
        (maximum, id) => Math.max(maximum, clips.get(id)?.revision ?? 0),
        0,
      ),
      id: custom.id,
      name: custom.name,
      playback: custom.playback,
      authoredClipIds,
      openableInEditor: authoredClipIds.some((id) => clips.has(id)),
      sourceClipCount: referencedSourceClips.length,
      durationSeconds: referencedClips.reduce(
        (total, clip) => total + clip.lengthSeconds,
        referencedSourceClips.reduce(
          (total, clip) => total + clip.durationSeconds,
          0,
        ),
      ),
      keyframeCount: referencedClips.reduce(
        (total, clip) => total + clip.tracks.reduce(
          (clipTotal, track) => clipTotal + track.keyframes.length,
          0,
        ),
        0,
      ),
      status,
      assignmentCount,
      assignedSlots,
      assignable: status === "VALID",
      diagnosticCodes: status === "VALID" ? [] : ["M2A-ANIMATION-EDIT-NOT-VALID"],
    };
  });
}

export function getAssignableCustomAnimationsV1(
  items: readonly ProjectedCustomAnimationLibraryItemV1[],
) {
  return items.filter(({ assignable }) => assignable);
}

export function getUnassignableCustomAnimationsV1(
  items: readonly ProjectedCustomAnimationLibraryItemV1[],
) {
  return items.filter(({ assignable }) => !assignable);
}

export function filterCustomAnimationPickerV1(
  items: readonly ProjectedCustomAnimationLibraryItemV1[],
  query: string,
) {
  const normalized = query.trim().toLocaleLowerCase("en-US");
  return items.filter(({ name }) => (
    !normalized || name.toLocaleLowerCase("en-US").includes(normalized)
  ));
}

export function validateCustomAnimationAssignmentV2(
  slot: DirectCreatureBaseSlotV1,
  customId: string,
  authoring: CreatureAnimationAuthoringV2,
  studio: AnimationStudioDocumentV1,
  sourceInventory: readonly AnimationStudioLibrarySourceV1[] = [],
): AnimationStudioDiagnosticV1[] {
  const item = projectCustomAnimationLibraryV1(
    authoring,
    studio,
    sourceInventory,
  )
    .find(({ id }) => id === customId);
  if (!item) return [diagnostic(
    "M2A-ANIMATION-EDIT-CUSTOM-MISSING",
    `assignments.${slot}`,
    "The selected Custom animation does not exist.",
    "Choose an item from the current Custom library.",
  )];
  if (!item.assignable) return [diagnostic(
    "M2A-ANIMATION-EDIT-CUSTOM-NOT-VALID",
    `customAnimations.${customId}`,
    "Draft or invalid Custom animations cannot be assigned.",
    "Open the animation in Create & edit and resolve its diagnostics.",
  )];
  return [];
}

export function assignCustomAnimationToBaseSlotV2(
  authoring: CreatureAnimationAuthoringV2,
  slot: DirectCreatureBaseSlotV1,
  customId: string,
  studio: AnimationStudioDocumentV1,
  sourceInventory: readonly AnimationStudioLibrarySourceV1[] = [],
): CreatureAnimationAuthoringV2 {
  const diagnostics = validateCustomAnimationAssignmentV2(
    slot,
    customId,
    authoring,
    studio,
    sourceInventory,
  );
  if (diagnostics.length > 0) throw new Error(diagnostics[0]!.message);
  return {
    ...authoring,
    authoringRevision: authoring.authoringRevision + 1,
    assignments: [
      ...authoring.assignments.filter(({ targetSlot }) => targetSlot !== slot),
      {
        targetSlot: slot,
        sourceKind: "CUSTOM",
        sourceClipName: null,
        customAnimationId: customId,
        provenance: {
          provider: "USER_CUSTOM",
          assetId: customId,
          ownership: "USER_OWNED",
        },
      },
    ],
  };
}

export function clearCustomAnimationFromBaseSlotV2(
  authoring: CreatureAnimationAuthoringV2,
  slot: DirectCreatureBaseSlotV1,
): CreatureAnimationAuthoringV2 {
  const assignments = authoring.assignments.filter(
    ({ targetSlot }) => targetSlot !== slot,
  );
  return assignments.length === authoring.assignments.length
    ? authoring
    : {
        ...authoring,
        authoringRevision: authoring.authoringRevision + 1,
        assignments,
      };
}

export function openCustomAnimationInEditorV1(
  customId: string,
  authoring: CreatureAnimationAuthoringV2,
  studio: AnimationStudioDocumentV1,
): string {
  const custom = authoring.customAnimations.find(({ id }) => id === customId);
  if (!custom) throw new Error("Unknown Custom animation ID.");
  const clipId = customAuthoredClipIds(custom)[0];
  if (!clipId || !studio.authoredClips.some(({ id }) => id === clipId)) {
    throw new Error("Custom animation does not reference an authored clip.");
  }
  return clipId;
}

export function createCustomDefinitionFromAuthoredClipV1(
  clipId: string,
  input: { readonly id: string; readonly name: string },
): CustomAnimationDefinitionV2 {
  return {
    id: input.id,
    name: input.name,
    playback: "ONE_SHOT",
    clipReference: {
      sourceKind: "AUTHORED_CLIP",
      sourceClipName: null,
      authoredClipId: clipId,
    },
    phases: [],
    provenance: {
      provider: "USER_CUSTOM",
      assetId: clipId,
      ownership: "USER_OWNED",
    },
  };
}

export function createPhasedCustomDefinitionFromAuthoredClipsV1(
  studio: AnimationStudioDocumentV1,
  input: {
    readonly name?: string;
    readonly startAuthoredClipId: string;
    readonly loopAuthoredClipId: string;
    readonly endAuthoredClipId: string;
    readonly existingIds?: readonly string[];
    readonly existingNames?: readonly string[];
  },
): CustomAnimationDefinitionV2 {
  const phaseClipIds = [
    input.startAuthoredClipId,
    input.loopAuthoredClipId,
    input.endAuthoredClipId,
  ];
  if (phaseClipIds.some((id) => !id.trim()) || new Set(phaseClipIds).size !== 3) {
    throw new RangeError(
      "Phased Custom requires three distinct authored clips for START, LOOP and END.",
    );
  }
  const clips = new Map(studio.authoredClips.map((clip) => [clip.id, clip]));
  for (const clipId of phaseClipIds) {
    const clip = clips.get(clipId);
    if (!clip) throw new RangeError(`Unknown authored clip ID: ${clipId}`);
    if (clip.status !== "VALID") {
      throw new RangeError(
        `Authored clip ${clip.name} must be Valid before phased assignment.`,
      );
    }
  }
  const existingNames = input.existingNames ?? [];
  const name = (input.name?.trim() || nextPhasedCustomAnimationNameV1(existingNames))
    .toLocaleLowerCase("en-US");
  if (!/^[a-z][a-z0-9_]{0,13}$/.test(name)) {
    throw new RangeError(
      "Phased Custom name must be an ASCII resref-like identifier with at most 14 characters.",
    );
  }
  if (existingNames.some(
    (candidate) => candidate.toLocaleLowerCase("en-US") === name,
  )) {
    throw new RangeError(`Custom animation name ${name} is already used.`);
  }
  const existingIds = new Set(input.existingIds ?? []);
  let id = uniqueId("custom");
  while (existingIds.has(id)) id = uniqueId("custom");
  const clipReference = (authoredClipId: string) => ({
    sourceKind: "AUTHORED_CLIP" as const,
    sourceClipName: null,
    authoredClipId,
  });
  return {
    id,
    name,
    playback: "LOOPING_PHASED",
    clipReference: null,
    phases: [
      {
        phase: "START",
        clipReference: clipReference(input.startAuthoredClipId),
      },
      {
        phase: "LOOP",
        clipReference: clipReference(input.loopAuthoredClipId),
      },
      {
        phase: "END",
        clipReference: clipReference(input.endAuthoredClipId),
      },
    ],
    provenance: {
      provider: "USER_CUSTOM",
      assetId: id,
      ownership: "USER_OWNED",
    },
  };
}

export function nextPhasedCustomAnimationNameV1(
  existingNames: readonly string[],
) {
  const used = new Set(
    existingNames.map((name) => name.toLocaleLowerCase("en-US")),
  );
  if (!used.has("custom_loop")) return "custom_loop";
  let suffix = 2;
  while (used.has(`custom_loop_${suffix}`)) suffix += 1;
  return `custom_loop_${suffix}`;
}

function customReferencesClip(
  custom: CustomAnimationDefinitionV2,
  clipId: string,
) {
  return customAuthoredClipIds(custom).includes(clipId);
}

function customClipReferences(custom: CustomAnimationDefinitionV2) {
  return [
    custom.clipReference,
    ...custom.phases.map(({ clipReference }) => clipReference),
  ].filter((reference): reference is CustomAnimationClipReferenceV2 => (
    reference !== null
  ));
}

function customAuthoredClipIds(custom: CustomAnimationDefinitionV2): string[] {
  return customClipReferences(custom).flatMap((reference) => (
    reference?.sourceKind === "AUTHORED_CLIP" && reference.authoredClipId
      ? [reference.authoredClipId]
      : []
  ));
}

function committedClip(
  clip: AuthoredAnimationClipV1,
  patch: Partial<AuthoredAnimationClipV1>,
): AuthoredAnimationClipV1 {
  return { ...clip, ...patch, revision: clip.revision + 1 };
}

function assertClipTime(timeSeconds: number, clipLength: number) {
  if (!Number.isFinite(timeSeconds) || timeSeconds < 0 || timeSeconds > clipLength) {
    throw new Error("Keyframe time must be finite and inside the clip.");
  }
}

function assertStrictTimes(keys: readonly AnimationKeyframeV1[]) {
  for (let index = 1; index < keys.length; index += 1) {
    if (keys[index]!.timeSeconds <= keys[index - 1]!.timeSeconds) {
      throw new Error("Keyframe times must remain strictly increasing.");
    }
  }
}

function deduplicateTimes(keys: readonly AnimationKeyframeV1[]) {
  const byTime = new Map<string, AnimationKeyframeV1>();
  keys.forEach((keyframe) => byTime.set(keyframe.timeSeconds.toFixed(7), keyframe));
  return [...byTime.values()].sort((left, right) => left.timeSeconds - right.timeSeconds);
}

function diagnostic(
  code: string,
  path: string,
  message: string,
  action: string,
): AnimationStudioDiagnosticV1 {
  return { schemaVersion: 1, code, path, level: "BLOCKING", message, action };
}

function clamp(value: number, minimum: number, maximum: number) {
  return Math.min(maximum, Math.max(minimum, value));
}

function uniqueId(prefix: string) {
  return typeof crypto.randomUUID === "function"
    ? `${prefix}-${crypto.randomUUID()}`
    : `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}
