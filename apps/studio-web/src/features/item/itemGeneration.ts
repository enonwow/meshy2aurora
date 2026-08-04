import type { ItemBaseItemRow } from "./types";

export type ItemGenerationRole = "MODEL" | "BOTTOM" | "MIDDLE" | "TOP";
export type ItemGenerationSlotStatus =
  | "EMPTY"
  | "CONCEPT_READY"
  | "REVIEWED"
  | "TASK_CREATED"
  | "RUNNING"
  | "RECOVERY_REQUIRED"
  | "ARTIFACT_VERIFIED"
  | "IMPORTED"
  | "FITTED"
  | "BUILD_ACCEPTED"
  | "FAILED"
  | "CANCELED";

export type ItemGenerationSessionStatus =
  | "DRAFT"
  | "CONCEPTS_READY"
  | "REVIEWED"
  | "RUNNING"
  | "PARTIAL"
  | "ARTIFACTS_VERIFIED"
  | "IMPORTED"
  | "FITTED"
  | "BUILD_ACCEPTED";

export interface ItemGenerationConceptV1 {
  readonly fileName: string;
  readonly mimeType: "image/png" | "image/jpeg";
  readonly byteLength: number;
  readonly sha256: string;
}

export interface ItemGenerationPreviewV1 {
  readonly previewId: string;
  readonly maximumCredits: number;
}

export interface ItemGenerationRunV1 {
  readonly runId: string;
  readonly taskId: string | null;
  readonly createdAt: string;
}

export interface ItemGenerationArtifactV1 {
  readonly sha256: string;
  readonly byteLength: number;
  readonly consumedCredits: number;
  readonly finishedAt: string;
}

export interface ItemGenerationSlotV1 {
  readonly field: string;
  readonly label: string;
  readonly token: string | null;
  readonly role: ItemGenerationRole;
  readonly profileId: "S1-static-prop/v1";
  readonly targetPolycount: number;
  readonly status: ItemGenerationSlotStatus;
  readonly concept: ItemGenerationConceptV1 | null;
  readonly preview: ItemGenerationPreviewV1 | null;
  readonly run: ItemGenerationRunV1 | null;
  readonly artifact: ItemGenerationArtifactV1 | null;
}

export interface ItemGenerationSessionV1 {
  readonly schemaVersion: 1;
  readonly sessionId: string;
  readonly createdAt: string;
  readonly baseitemsSha256: string;
  readonly baseItem: number;
  readonly itemClass: string;
  readonly modelType: 0 | 1 | 2 | 3;
  readonly status: ItemGenerationSessionStatus;
  readonly ownerCreditCap: number | null;
  readonly balanceAtReview: number | null;
  readonly maximumCredits: number;
  readonly slots: readonly ItemGenerationSlotV1[];
}

export interface ItemGenerationPlanEntryV1 {
  readonly field: string;
  readonly previewId: string;
  readonly maximumCredits: number;
  readonly targetPolycount: number;
}

const SHA256 = /^[a-f0-9]{64}$/;

function requireSha256(value: string, path: string) {
  if (!SHA256.test(value)) throw new Error(`${path} must be an exact lowercase SHA-256`);
}

function requireNonEmpty(value: string, path: string) {
  if (!value.trim()) throw new Error(`${path} must be non-empty`);
}

function requireNonNegativeInteger(value: number, path: string) {
  if (!Number.isInteger(value) || value < 0) throw new Error(`${path} must be a non-negative integer`);
}

function roleFor(row: ItemBaseItemRow, field: string): ItemGenerationRole {
  if (row.capability.compositionProfile !== "BOTTOM_MIDDLE_TOP") return "MODEL";
  if (field === "ModelPart1") return "BOTTOM";
  if (field === "ModelPart2") return "MIDDLE";
  if (field === "ModelPart3") return "TOP";
  throw new Error(`unsupported three-part Item field ${field}`);
}

function defaultTargetPolycount(role: ItemGenerationRole) {
  if (role === "BOTTOM") return 5_000;
  if (role === "MIDDLE") return 8_000;
  if (role === "TOP") return 3_000;
  return 20_000;
}

function aggregateStatus(slots: readonly ItemGenerationSlotV1[]): ItemGenerationSessionStatus {
  if (slots.every(({ status }) => status === "BUILD_ACCEPTED")) return "BUILD_ACCEPTED";
  if (slots.every(({ status }) => ["FITTED", "BUILD_ACCEPTED"].includes(status))) return "FITTED";
  if (slots.every(({ status }) => ["IMPORTED", "FITTED", "BUILD_ACCEPTED"].includes(status))) return "IMPORTED";
  if (slots.every(({ status }) => ["ARTIFACT_VERIFIED", "IMPORTED", "FITTED", "BUILD_ACCEPTED"].includes(status))) return "ARTIFACTS_VERIFIED";
  if (slots.some(({ status }) => ["TASK_CREATED", "RUNNING", "RECOVERY_REQUIRED"].includes(status))) return "RUNNING";
  if (slots.some(({ status }) => status === "ARTIFACT_VERIFIED")) return "PARTIAL";
  if (slots.length > 0 && slots.every(({ status }) => status === "REVIEWED")) return "REVIEWED";
  if (slots.length > 0 && slots.every(({ status }) => status === "CONCEPT_READY")) return "CONCEPTS_READY";
  return "DRAFT";
}

function replaceSlot(
  session: ItemGenerationSessionV1,
  field: string,
  update: (slot: ItemGenerationSlotV1) => ItemGenerationSlotV1,
): ItemGenerationSessionV1 {
  const matches = session.slots.filter((slot) => slot.field === field);
  if (matches.length !== 1) throw new Error(`generation field ${field} must resolve exactly once`);
  const slots = session.slots.map((slot) => slot.field === field ? update(slot) : slot);
  return { ...session, slots, status: aggregateStatus(slots) };
}

export function createItemGenerationSession(
  row: ItemBaseItemRow,
  baseitemsSha256: string,
  sessionId: string,
  createdAt: string,
): ItemGenerationSessionV1 {
  requireSha256(baseitemsSha256, "baseitemsSha256");
  requireNonEmpty(sessionId, "sessionId");
  requireNonEmpty(createdAt, "createdAt");
  const meshySlots = row.partSlots.filter(({ sourceKind }) => sourceKind === "MESHY_GLB");
  if (meshySlots.length !== row.capability.meshySourceCount) {
    throw new Error("BaseItem Meshy source count differs from its resolved slot schema");
  }
  const fields = new Set(meshySlots.map(({ field }) => field.toLowerCase()));
  if (fields.size !== meshySlots.length) throw new Error("BaseItem Meshy fields must be unique");
  const slots = meshySlots.map((slot): ItemGenerationSlotV1 => {
    const role = roleFor(row, slot.field);
    return {
      field: slot.field,
      label: slot.label,
      token: slot.token,
      role,
      profileId: "S1-static-prop/v1",
      targetPolycount: defaultTargetPolycount(role),
      status: "EMPTY",
      concept: null,
      preview: null,
      run: null,
      artifact: null,
    };
  });
  return {
    schemaVersion: 1,
    sessionId,
    createdAt,
    baseitemsSha256,
    baseItem: row.baseItem,
    itemClass: row.itemClass,
    modelType: row.modelType,
    status: "DRAFT",
    ownerCreditCap: null,
    balanceAtReview: null,
    maximumCredits: 0,
    slots,
  };
}

export function setItemGenerationConcept(
  session: ItemGenerationSessionV1,
  field: string,
  concept: ItemGenerationConceptV1,
): ItemGenerationSessionV1 {
  requireNonEmpty(concept.fileName, `${field}.concept.fileName`);
  if (!(["image/png", "image/jpeg"] as const).includes(concept.mimeType)) {
    throw new Error(`${field}.concept.mimeType is unsupported`);
  }
  if (!Number.isInteger(concept.byteLength) || concept.byteLength <= 0) {
    throw new Error(`${field}.concept.byteLength must be positive`);
  }
  requireSha256(concept.sha256, `${field}.concept.sha256`);
  return replaceSlot(session, field, (slot) => {
    if (!["EMPTY", "CONCEPT_READY"].includes(slot.status)) {
      throw new Error(`${field} concept cannot change after paid review`);
    }
    return { ...slot, concept, status: "CONCEPT_READY" };
  });
}

export function createItemGenerationPlan(
  session: ItemGenerationSessionV1,
  entries: readonly ItemGenerationPlanEntryV1[],
  ownerCreditCap: number,
  availableCredits: number,
): ItemGenerationSessionV1 {
  requireNonNegativeInteger(ownerCreditCap, "ownerCreditCap");
  requireNonNegativeInteger(availableCredits, "availableCredits");
  const pendingSlots = session.slots.filter((slot) => !slot.artifact);
  if (entries.length !== pendingSlots.length) throw new Error("generation plan must cover every unfinished slot exactly once");
  const byField = new Map<string, ItemGenerationPlanEntryV1>();
  for (const entry of entries) {
    requireNonEmpty(entry.previewId, `${entry.field}.previewId`);
    requireNonNegativeInteger(entry.maximumCredits, `${entry.field}.maximumCredits`);
    if (!Number.isInteger(entry.targetPolycount) || entry.targetPolycount < 100 || entry.targetPolycount > 300_000) {
      throw new Error(`${entry.field}.targetPolycount is outside 100..=300000`);
    }
    if (byField.has(entry.field)) throw new Error(`generation plan duplicates ${entry.field}`);
    byField.set(entry.field, entry);
  }
  const maximumCredits = entries.reduce((sum, entry) => sum + entry.maximumCredits, 0);
  if (maximumCredits > ownerCreditCap) throw new Error("generation plan exceeds the owner cap");
  if (maximumCredits > availableCredits) throw new Error("generation plan exceeds the available Meshy balance");
  const slots = session.slots.map((slot): ItemGenerationSlotV1 => {
    if (slot.artifact) return slot;
    if (!slot.concept) throw new Error(`${slot.field} requires a hash-bound concept before review`);
    if (!["CONCEPT_READY", "REVIEWED"].includes(slot.status)) {
      throw new Error(`${slot.field} cannot be included in a new paid review`);
    }
    const entry = byField.get(slot.field);
    if (!entry) throw new Error(`generation plan is missing ${slot.field}`);
    return {
      ...slot,
      targetPolycount: entry.targetPolycount,
      preview: { previewId: entry.previewId, maximumCredits: entry.maximumCredits },
      status: "REVIEWED",
    };
  });
  return {
    ...session,
    slots,
    status: aggregateStatus(slots),
    ownerCreditCap,
    balanceAtReview: availableCredits,
    maximumCredits,
  };
}

export function recordItemGenerationRun(
  session: ItemGenerationSessionV1,
  field: string,
  run: ItemGenerationRunV1,
): ItemGenerationSessionV1 {
  requireNonEmpty(run.runId, `${field}.runId`);
  if (run.taskId !== null) requireNonEmpty(run.taskId, `${field}.taskId`);
  requireNonEmpty(run.createdAt, `${field}.createdAt`);
  return replaceSlot(session, field, (slot) => {
    if (slot.run) throw new Error(`${field} is already bound to an exact run/task identity`);
    if (slot.status !== "REVIEWED") throw new Error(`${field} must be reviewed before task creation`);
    return { ...slot, run, status: "TASK_CREATED" };
  });
}

export function bindItemGenerationTask(
  session: ItemGenerationSessionV1,
  field: string,
  taskId: string,
): ItemGenerationSessionV1 {
  requireNonEmpty(taskId, `${field}.taskId`);
  return replaceSlot(session, field, (slot) => {
    if (!slot.run) throw new Error(`${field} requires a created run before task binding`);
    if (slot.run.taskId) throw new Error(`${field} is already bound to an exact task identity`);
    if (slot.status !== "TASK_CREATED") throw new Error(`${field} is not awaiting a task identity`);
    return { ...slot, run: { ...slot.run, taskId }, status: "RUNNING" };
  });
}

export function recordRecoveredItemGenerationArtifact(
  session: ItemGenerationSessionV1,
  field: string,
  input: {
    readonly taskId: string;
    readonly createdAt: string;
    readonly artifact: ItemGenerationArtifactV1;
  },
): ItemGenerationSessionV1 {
  requireNonEmpty(input.taskId, `${field}.taskId`);
  requireNonEmpty(input.createdAt, `${field}.createdAt`);
  requireSha256(input.artifact.sha256, `${field}.artifact.sha256`);
  if (!Number.isInteger(input.artifact.byteLength) || input.artifact.byteLength <= 0) {
    throw new Error(`${field}.artifact.byteLength must be positive`);
  }
  requireNonNegativeInteger(input.artifact.consumedCredits, `${field}.artifact.consumedCredits`);
  requireNonEmpty(input.artifact.finishedAt, `${field}.artifact.finishedAt`);
  return replaceSlot(session, field, (slot) => {
    if (!slot.concept) throw new Error(`${field} recovery requires its hash-bound concept`);
    if (slot.artifact) throw new Error(`${field} artifact is already bound`);
    if (slot.run) {
      if (slot.run.taskId !== input.taskId) {
        throw new Error(`${field} recovery task differs from its bound exact task identity`);
      }
      if (!["TASK_CREATED", "RUNNING", "RECOVERY_REQUIRED", "FAILED", "CANCELED"].includes(slot.status)) {
        throw new Error(`${field} is not awaiting recovery of its bound task`);
      }
      return { ...slot, artifact: input.artifact, status: "ARTIFACT_VERIFIED" };
    }
    if (slot.status !== "CONCEPT_READY") throw new Error(`${field} is not ready for exact task recovery`);
    return {
      ...slot,
      run: { runId: `recovered:${input.taskId}`, taskId: input.taskId, createdAt: input.createdAt },
      artifact: input.artifact,
      status: "ARTIFACT_VERIFIED",
    };
  });
}

export function recordItemGenerationArtifact(
  session: ItemGenerationSessionV1,
  field: string,
  artifact: ItemGenerationArtifactV1,
): ItemGenerationSessionV1 {
  requireSha256(artifact.sha256, `${field}.artifact.sha256`);
  if (!Number.isInteger(artifact.byteLength) || artifact.byteLength <= 0) {
    throw new Error(`${field}.artifact.byteLength must be positive`);
  }
  requireNonNegativeInteger(artifact.consumedCredits, `${field}.artifact.consumedCredits`);
  requireNonEmpty(artifact.finishedAt, `${field}.artifact.finishedAt`);
  return replaceSlot(session, field, (slot) => {
    if (!slot.run?.taskId) throw new Error(`${field} artifact requires an exact task ID`);
    if (slot.artifact) throw new Error(`${field} artifact is already bound`);
    if (!["TASK_CREATED", "RUNNING", "RECOVERY_REQUIRED"].includes(slot.status)) {
      throw new Error(`${field} is not awaiting an artifact`);
    }
    return { ...slot, artifact, status: "ARTIFACT_VERIFIED" };
  });
}

const ROOT_KEYS = [
  "schemaVersion", "sessionId", "createdAt", "baseitemsSha256", "baseItem", "itemClass",
  "modelType", "status", "ownerCreditCap", "balanceAtReview", "maximumCredits", "slots",
] as const;
const SLOT_KEYS = [
  "field", "label", "token", "role", "profileId", "targetPolycount", "status", "concept",
  "preview", "run", "artifact",
] as const;

function exactKeys(value: unknown, keys: readonly string[], path: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`${path} must be an object`);
  const record = value as Record<string, unknown>;
  const unknown = Object.keys(record).filter((key) => !keys.includes(key));
  if (unknown.length) throw new Error(`${path} has unknown field ${unknown[0]}`);
  if (keys.some((key) => !(key in record))) throw new Error(`${path} is missing a required field`);
  return record;
}

export function serializeItemGenerationSession(session: ItemGenerationSessionV1): string {
  return JSON.stringify(session);
}

export function deserializeItemGenerationSession(json: string): ItemGenerationSessionV1 {
  if (/https?:\/\/|authorization|api[_-]?key|signedUrl/i.test(json)) {
    throw new Error("session sidecar contains a URL or credential-like field");
  }
  const root = exactKeys(JSON.parse(json), ROOT_KEYS, "session");
  if (root.schemaVersion !== 1 || !Array.isArray(root.slots)) throw new Error("session schema is invalid");
  exactKeys(root, ROOT_KEYS, "session");
  for (const [index, rawSlot] of root.slots.entries()) {
    const slot = exactKeys(rawSlot, SLOT_KEYS, `session.slots[${index}]`);
    if (slot.concept !== null) exactKeys(slot.concept, ["fileName", "mimeType", "byteLength", "sha256"], `session.slots[${index}].concept`);
    if (slot.preview !== null) exactKeys(slot.preview, ["previewId", "maximumCredits"], `session.slots[${index}].preview`);
    if (slot.run !== null) exactKeys(slot.run, ["runId", "taskId", "createdAt"], `session.slots[${index}].run`);
    if (slot.artifact !== null) exactKeys(slot.artifact, ["sha256", "byteLength", "consumedCredits", "finishedAt"], `session.slots[${index}].artifact`);
  }
  const session = root as unknown as ItemGenerationSessionV1;
  const allowedStatuses: readonly ItemGenerationSessionStatus[] = [
    "DRAFT", "CONCEPTS_READY", "REVIEWED", "RUNNING", "PARTIAL", "ARTIFACTS_VERIFIED",
    "IMPORTED", "FITTED", "BUILD_ACCEPTED",
  ];
  if (!allowedStatuses.includes(session.status)) throw new Error("session status is invalid");
  requireNonEmpty(session.sessionId, "session.sessionId");
  requireNonEmpty(session.createdAt, "session.createdAt");
  requireNonEmpty(session.itemClass, "session.itemClass");
  requireSha256(session.baseitemsSha256, "session.baseitemsSha256");
  requireNonNegativeInteger(session.baseItem, "session.baseItem");
  requireNonNegativeInteger(session.maximumCredits, "session.maximumCredits");
  if (session.ownerCreditCap !== null) requireNonNegativeInteger(session.ownerCreditCap, "session.ownerCreditCap");
  if (session.balanceAtReview !== null) requireNonNegativeInteger(session.balanceAtReview, "session.balanceAtReview");
  const fields = new Set<string>();
  const allowedSlotStatuses: readonly ItemGenerationSlotStatus[] = [
    "EMPTY", "CONCEPT_READY", "REVIEWED", "TASK_CREATED", "RUNNING", "RECOVERY_REQUIRED",
    "ARTIFACT_VERIFIED", "IMPORTED", "FITTED", "BUILD_ACCEPTED", "FAILED", "CANCELED",
  ];
  for (const slot of session.slots) {
    requireNonEmpty(slot.field, "session.slot.field");
    requireNonEmpty(slot.label, `${slot.field}.label`);
    if (fields.has(slot.field)) throw new Error(`session duplicates slot ${slot.field}`);
    fields.add(slot.field);
    if (!(["MODEL", "BOTTOM", "MIDDLE", "TOP"] as const).includes(slot.role)) {
      throw new Error(`${slot.field}.role is invalid`);
    }
    if (slot.profileId !== "S1-static-prop/v1") throw new Error(`${slot.field}.profileId is invalid`);
    if (!Number.isInteger(slot.targetPolycount) || slot.targetPolycount < 100 || slot.targetPolycount > 300_000) {
      throw new Error(`${slot.field}.targetPolycount is outside 100..=300000`);
    }
    if (!allowedSlotStatuses.includes(slot.status)) throw new Error(`${slot.field}.status is invalid`);
    if (slot.concept) {
      requireNonEmpty(slot.concept.fileName, `${slot.field}.concept.fileName`);
      requireSha256(slot.concept.sha256, `${slot.field}.concept.sha256`);
      if (!["image/png", "image/jpeg"].includes(slot.concept.mimeType)
        || !Number.isInteger(slot.concept.byteLength) || slot.concept.byteLength <= 0) {
        throw new Error(`${slot.field}.concept is invalid`);
      }
    }
    if (slot.preview) {
      requireNonEmpty(slot.preview.previewId, `${slot.field}.preview.previewId`);
      requireNonNegativeInteger(slot.preview.maximumCredits, `${slot.field}.preview.maximumCredits`);
    }
    if (slot.run) {
      requireNonEmpty(slot.run.runId, `${slot.field}.run.runId`);
      requireNonEmpty(slot.run.createdAt, `${slot.field}.run.createdAt`);
      if (slot.run.taskId !== null) requireNonEmpty(slot.run.taskId, `${slot.field}.run.taskId`);
    }
    if (slot.artifact) {
      requireSha256(slot.artifact.sha256, `${slot.field}.artifact.sha256`);
      if (!Number.isInteger(slot.artifact.byteLength) || slot.artifact.byteLength <= 0) {
        throw new Error(`${slot.field}.artifact.byteLength must be positive`);
      }
      requireNonNegativeInteger(slot.artifact.consumedCredits, `${slot.field}.artifact.consumedCredits`);
      requireNonEmpty(slot.artifact.finishedAt, `${slot.field}.artifact.finishedAt`);
    }
  }
  if (aggregateStatus(session.slots) !== session.status) throw new Error("session aggregate status differs from its slots");
  return session;
}
