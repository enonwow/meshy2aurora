import { isDirectCreatureBaseSlotV1 } from "./catalog";
import {
  CREATURE_ANIMATION_AUTHORING_PROFILE_V1,
  type AnimationMappingDiagnosticV1,
  type AnimationMappingProvenanceV1,
  type AnimationSourceAssignmentV1,
  type CreatureAnimationAuthoringV1,
  type CustomAnimationDefinitionV1,
} from "./types";

export type CreatureAnimationDraftParseResultV1 =
  | { kind: "VALID"; value: CreatureAnimationAuthoringV1 }
  | { kind: "INVALID"; diagnostics: AnimationMappingDiagnosticV1[] };

export type CreatureAnimationDraftSaveResultV1 =
  | { kind: "SAVED"; savedAt: string }
  | { kind: "ERROR"; message: string };

export type CreatureAnimationDraftLoadResultV1 =
  | { kind: "EMPTY" }
  | { kind: "LOADED"; value: CreatureAnimationAuthoringV1 }
  | { kind: "ERROR"; diagnostics: AnimationMappingDiagnosticV1[] };

const storagePrefix = "m2a:creature-animation-draft:v1:";

export function serializeCreatureAnimationAuthoringV1(
  authoring: CreatureAnimationAuthoringV1,
): string {
  return JSON.stringify({
    schemaVersion: authoring.schemaVersion,
    profile: authoring.profile,
    modelType: authoring.modelType,
    sourceRevision: authoring.sourceRevision,
    authoringRevision: authoring.authoringRevision,
    assignments: authoring.assignments.map((assignment) => ({
      targetSlot: assignment.targetSlot,
      sourceKind: assignment.sourceKind,
      sourceClipName: assignment.sourceClipName,
      customAnimationId: assignment.customAnimationId,
      provenance: canonicalProvenance(assignment.provenance),
    })),
    fallbacks: authoring.fallbacks.map((fallback) => ({
      id: fallback.id,
      targetSlot: fallback.targetSlot,
      sourceSlot: fallback.sourceSlot,
      reason: fallback.reason,
      review: fallback.review,
    })),
    customAnimations: authoring.customAnimations.map((custom) => ({
      id: custom.id,
      name: custom.name,
      playback: custom.playback,
      sourceClipName: custom.sourceClipName,
      phases: custom.phases.map((phase) => ({
        phase: phase.phase,
        sourceClipName: phase.sourceClipName,
      })),
      provenance: canonicalProvenance(custom.provenance),
    })),
  });
}

export function parseCreatureAnimationAuthoringV1(
  json: string,
): CreatureAnimationDraftParseResultV1 {
  try {
    return migrateCreatureAnimationDraftV1(JSON.parse(json));
  } catch (error) {
    return invalid(
      "M2A-ANIMATION-DRAFT-JSON",
      "draft",
      `Animation draft is not valid JSON: ${errorMessage(error)}`,
      "Discard the corrupt draft or restore a valid project export.",
    );
  }
}

export function migrateCreatureAnimationDraftV1(
  document: unknown,
): CreatureAnimationDraftParseResultV1 {
  if (!isRecord(document)) {
    return invalidDocument("Animation draft must be a JSON object.");
  }
  if (
    typeof document.schemaVersion === "number"
    && document.schemaVersion > 1
  ) {
    return invalid(
      "M2A-ANIMATION-DRAFT-NEWER-SCHEMA",
      "schemaVersion",
      `Animation draft schemaVersion ${document.schemaVersion} is newer than supported version 1.`,
      "Open the draft with a newer Studio version.",
    );
  }
  if (
    document.schemaVersion !== 1
    || document.profile !== CREATURE_ANIMATION_AUTHORING_PROFILE_V1
    || (document.modelType !== "S" && document.modelType !== "L")
    || typeof document.sourceRevision !== "string"
    || !Number.isSafeInteger(document.authoringRevision)
    || (document.authoringRevision as number) < 1
    || !Array.isArray(document.assignments)
    || !Array.isArray(document.fallbacks)
    || !Array.isArray(document.customAnimations)
  ) {
    return invalidDocument("Animation draft has an invalid v1 document header.");
  }

  const assignments = document.assignments.every(isAssignment)
    ? document.assignments as AnimationSourceAssignmentV1[]
    : null;
  const fallbacks = document.fallbacks.every((fallback) => (
    isRecord(fallback)
    && typeof fallback.id === "string"
    && isDirectCreatureBaseSlotV1(stringValue(fallback.targetSlot))
    && isDirectCreatureBaseSlotV1(stringValue(fallback.sourceSlot))
    && typeof fallback.reason === "string"
    && ["PENDING", "ACCEPTED", "REJECTED"].includes(stringValue(fallback.review))
  ))
    ? document.fallbacks as CreatureAnimationAuthoringV1["fallbacks"]
    : null;
  const customAnimations = document.customAnimations.every(isCustomAnimation)
    ? document.customAnimations as CustomAnimationDefinitionV1[]
    : null;
  if (!assignments || !fallbacks || !customAnimations) {
    return invalidDocument("Animation draft contains an invalid assignment, fallback or custom animation.");
  }

  return {
    kind: "VALID",
    value: {
      schemaVersion: 1,
      profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1,
      modelType: document.modelType,
      sourceRevision: document.sourceRevision,
      authoringRevision: document.authoringRevision as number,
      assignments,
      fallbacks,
      customAnimations,
    },
  };
}

export function saveCreatureAnimationDraftV1(
  projectId: string,
  authoring: CreatureAnimationAuthoringV1,
  storage: Storage = window.localStorage,
): CreatureAnimationDraftSaveResultV1 {
  try {
    storage.setItem(storageKey(projectId), serializeCreatureAnimationAuthoringV1(authoring));
    return { kind: "SAVED", savedAt: new Date().toISOString() };
  } catch (error) {
    return { kind: "ERROR", message: errorMessage(error) };
  }
}

export function loadCreatureAnimationDraftV1(
  projectId: string,
  storage: Storage = window.localStorage,
): CreatureAnimationDraftLoadResultV1 {
  try {
    const json = storage.getItem(storageKey(projectId));
    if (json === null) return { kind: "EMPTY" };
    const parsed = parseCreatureAnimationAuthoringV1(json);
    return parsed.kind === "VALID"
      ? { kind: "LOADED", value: parsed.value }
      : { kind: "ERROR", diagnostics: parsed.diagnostics };
  } catch (error) {
    return {
      kind: "ERROR",
      diagnostics: [diagnostic(
        "M2A-ANIMATION-DRAFT-STORAGE",
        "draft",
        `Animation draft storage failed: ${errorMessage(error)}`,
        "Check browser storage permissions and available space.",
      )],
    };
  }
}

function isAssignment(value: unknown): value is AnimationSourceAssignmentV1 {
  return isRecord(value)
    && isDirectCreatureBaseSlotV1(stringValue(value.targetSlot))
    && ["SOURCE_CLIP", "INHERITED_SUPERMODEL", "PROCEDURAL", "CUSTOM"]
      .includes(stringValue(value.sourceKind))
    && isNullableString(value.sourceClipName)
    && isNullableString(value.customAnimationId)
    && isProvenance(value.provenance);
}

function isCustomAnimation(value: unknown): value is CustomAnimationDefinitionV1 {
  return isRecord(value)
    && typeof value.id === "string"
    && typeof value.name === "string"
    && (value.playback === "ONE_SHOT" || value.playback === "LOOPING_PHASED")
    && isNullableString(value.sourceClipName)
    && Array.isArray(value.phases)
    && value.phases.every((phase) => (
      isRecord(phase)
      && ["START", "LOOP", "END"].includes(stringValue(phase.phase))
      && typeof phase.sourceClipName === "string"
    ))
    && isProvenance(value.provenance);
}

function isProvenance(value: unknown): value is AnimationMappingProvenanceV1 {
  return isRecord(value)
    && ["SOURCE_GLB", "COMPATIBLE_SUPERMODEL", "PROCEDURAL_GENERATOR", "USER_CUSTOM"]
      .includes(stringValue(value.provider))
    && typeof value.assetId === "string"
    && ["USER_OWNED", "ENGINE_INHERITED", "PROJECT_GENERATED"]
      .includes(stringValue(value.ownership));
}

function canonicalProvenance(
  provenance: AnimationMappingProvenanceV1,
): AnimationMappingProvenanceV1 {
  return {
    provider: provenance.provider,
    assetId: provenance.assetId,
    ownership: provenance.ownership,
  };
}

function storageKey(projectId: string): string {
  if (!projectId.trim()) throw new RangeError("Animation draft projectId is required");
  return `${storagePrefix}${encodeURIComponent(projectId)}`;
}

function invalidDocument(message: string): CreatureAnimationDraftParseResultV1 {
  return invalid(
    "M2A-ANIMATION-DRAFT-SCHEMA",
    "draft",
    message,
    "Discard the corrupt draft or restore a valid v1 project export.",
  );
}

function invalid(
  code: string,
  path: string,
  message: string,
  action: string,
): CreatureAnimationDraftParseResultV1 {
  return { kind: "INVALID", diagnostics: [diagnostic(code, path, message, action)] };
}

function diagnostic(
  code: string,
  path: string,
  message: string,
  action: string,
): AnimationMappingDiagnosticV1 {
  return {
    schemaVersion: 1,
    code,
    path,
    level: "BLOCKING",
    message,
    action,
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isNullableString(value: unknown): value is string | null {
  return value === null || typeof value === "string";
}

function stringValue(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
