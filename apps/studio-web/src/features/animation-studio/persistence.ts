import {
  fingerprintAnimationStudioDocumentV1,
  parseAnimationStudioDocumentV1,
  serializeAnimationStudioDocumentV1,
} from "./schema";
import type { AnimationStudioModeV1 } from "./state";
import type {
  AnimationStudioDiagnosticV1,
  AnimationStudioDocumentV1,
} from "./types";

const DATABASE_NAME = "m2a-animation-studio-v1";
const DATABASE_VERSION = 1;
const DOCUMENT_STORE = "documents";

export interface AnimationStudioStorageRecordV1 {
  storageSchemaVersion: 1;
  projectId: string;
  documentJson: string;
  documentFingerprintSha256: string;
  sourceRevision: string;
  authoringRevision: number;
  selectedMode: AnimationStudioModeV1;
  selectedClipId: string | null;
  savedAt: string;
}

export interface AnimationStudioStorageAdapterV1 {
  put(record: AnimationStudioStorageRecordV1): Promise<void>;
  get(projectId: string): Promise<unknown | undefined>;
  delete(projectId: string): Promise<void>;
  getAll(): Promise<unknown[]>;
  close?(): void;
}

export interface AnimationStudioDatabaseV1 {
  readonly schemaVersion: 1;
  readonly adapter: AnimationStudioStorageAdapterV1;
}

export interface AnimationStudioSessionMetadataV1 {
  selectedMode?: AnimationStudioModeV1;
  selectedClipId?: string | null;
}

export type AnimationStudioSaveResultV1 =
  | {
      kind: "SAVED";
      savedAt: string;
      revision: number;
      fingerprintSha256: string;
    }
  | { kind: "ERROR"; diagnostic: AnimationStudioDiagnosticV1 };

export type AnimationStudioLoadResultV1 =
  | { kind: "EMPTY" }
  | {
      kind: "LOADED";
      value: AnimationStudioDocumentV1;
      selectedMode: AnimationStudioModeV1;
      selectedClipId: string | null;
      savedAt: string;
      fingerprintSha256: string;
    }
  | { kind: "ERROR"; diagnostics: AnimationStudioDiagnosticV1[] };

export type AnimationStudioDeleteResultV1 =
  | { kind: "DELETED" }
  | { kind: "CONFIRMATION_REQUIRED" }
  | { kind: "ERROR"; diagnostic: AnimationStudioDiagnosticV1 };

export interface AnimationStudioRecoveryRecordV1 {
  projectId: string;
  sourceRevision: string | null;
  authoringRevision: number | null;
  documentStatus: AnimationStudioDocumentV1["status"] | "INVALID";
  selectedMode: AnimationStudioModeV1 | null;
  savedAt: string | null;
  recoverable: boolean;
  diagnostics: AnimationStudioDiagnosticV1[];
}

export type AnimationStudioStorageMigrationResultV1 =
  | { kind: "MIGRATED"; recordsChecked: number }
  | { kind: "ERROR"; diagnostics: AnimationStudioDiagnosticV1[] };

const databaseOpenings = new WeakMap<
  IDBFactory,
  Promise<AnimationStudioDatabaseV1>
>();

export function openAnimationStudioDatabaseV1(
  factory: IDBFactory | undefined = globalThis.indexedDB,
): Promise<AnimationStudioDatabaseV1> {
  if (factory === undefined) {
    return Promise.reject(new Error("IndexedDB is unavailable"));
  }
  const cached = databaseOpenings.get(factory);
  if (cached !== undefined) return cached;
  const opening = openIndexedDatabase(factory);
  databaseOpenings.set(factory, opening);
  return opening.catch((error: unknown) => {
    if (databaseOpenings.get(factory) === opening) {
      databaseOpenings.delete(factory);
    }
    throw error;
  });
}

export function createInMemoryAnimationStudioDatabaseV1(
  seed: readonly unknown[] = [],
): AnimationStudioDatabaseV1 {
  const records = new Map<string, unknown>();
  for (const record of seed) {
    if (isRecord(record) && typeof record.projectId === "string") {
      records.set(record.projectId, structuredClone(record));
    }
  }
  return {
    schemaVersion: 1,
    adapter: {
      async put(record) {
        records.set(record.projectId, structuredClone(record));
      },
      async get(projectId) {
        const record = records.get(projectId);
        return record === undefined ? undefined : structuredClone(record);
      },
      async delete(projectId) {
        records.delete(projectId);
      },
      async getAll() {
        return [...records.values()].map((record) => structuredClone(record));
      },
    },
  };
}

export async function saveAnimationStudioDocumentV1(
  projectId: string,
  document: AnimationStudioDocumentV1,
  database?: AnimationStudioDatabaseV1,
  metadata: AnimationStudioSessionMetadataV1 = {},
): Promise<AnimationStudioSaveResultV1> {
  try {
    const normalizedProjectId = requireProjectId(projectId);
    if (
      metadata.selectedClipId !== undefined
      && metadata.selectedClipId !== null
      && !document.authoredClips.some(({ id }) => id === metadata.selectedClipId)
    ) {
      throw new RangeError(
        `Selected authored clip does not exist: ${metadata.selectedClipId}`,
      );
    }
    const documentJson = serializeAnimationStudioDocumentV1(document);
    const fingerprintSha256 =
      await fingerprintAnimationStudioDocumentV1(document);
    const savedAt = new Date().toISOString();
    const target = database ?? await openAnimationStudioDatabaseV1();
    await target.adapter.put({
      storageSchemaVersion: 1,
      projectId: normalizedProjectId,
      documentJson,
      documentFingerprintSha256: fingerprintSha256,
      sourceRevision: document.sourceRevision,
      authoringRevision: document.authoringRevision,
      selectedMode: metadata.selectedMode ?? "MAP_BASE_42",
      selectedClipId: metadata.selectedClipId ?? null,
      savedAt,
    });
    return {
      kind: "SAVED",
      savedAt,
      revision: document.authoringRevision,
      fingerprintSha256,
    };
  } catch (error) {
    return {
      kind: "ERROR",
      diagnostic: storageDiagnostic(
        "M2A-ANIMATION-STUDIO-AUTOSAVE",
        `Animation Studio autosave failed: ${errorMessage(error)}`,
      ),
    };
  }
}

export async function loadAnimationStudioDocumentV1(
  projectId: string,
  database?: AnimationStudioDatabaseV1,
): Promise<AnimationStudioLoadResultV1> {
  try {
    const target = database ?? await openAnimationStudioDatabaseV1();
    const raw = await target.adapter.get(requireProjectId(projectId));
    if (raw === undefined) return { kind: "EMPTY" };
    const record = parseStorageRecord(raw);
    if (record.kind === "INVALID") return {
      kind: "ERROR",
      diagnostics: record.diagnostics,
    };
    const parsed = parseAnimationStudioDocumentV1(record.value.documentJson);
    if (parsed.kind === "INVALID") return {
      kind: "ERROR",
      diagnostics: parsed.diagnostics,
    };
    const fingerprint =
      await fingerprintAnimationStudioDocumentV1(parsed.value);
    if (fingerprint !== record.value.documentFingerprintSha256) {
      return {
        kind: "ERROR",
        diagnostics: [storageDiagnostic(
          "M2A-ANIMATION-STUDIO-RECOVERY-INTEGRITY",
          "Stored Animation Studio fingerprint does not match its document.",
        )],
      };
    }
    if (
      parsed.value.sourceRevision !== record.value.sourceRevision
      || parsed.value.authoringRevision !== record.value.authoringRevision
    ) {
      return {
        kind: "ERROR",
        diagnostics: [storageDiagnostic(
          "M2A-ANIMATION-STUDIO-RECOVERY-INTEGRITY",
          "Stored Animation Studio metadata does not match its document.",
        )],
      };
    }
    if (
      record.value.selectedClipId !== null
      && !parsed.value.authoredClips.some(
        ({ id }) => id === record.value.selectedClipId,
      )
    ) {
      return {
        kind: "ERROR",
        diagnostics: [storageDiagnostic(
          "M2A-ANIMATION-STUDIO-RECOVERY-INTEGRITY",
          "Stored Animation Studio selection references an unknown clip.",
        )],
      };
    }
    return {
      kind: "LOADED",
      value: parsed.value,
      selectedMode: record.value.selectedMode,
      selectedClipId: record.value.selectedClipId,
      savedAt: record.value.savedAt,
      fingerprintSha256: fingerprint,
    };
  } catch (error) {
    return {
      kind: "ERROR",
      diagnostics: [storageDiagnostic(
        "M2A-ANIMATION-STUDIO-LOAD",
        `Animation Studio load failed: ${errorMessage(error)}`,
      )],
    };
  }
}

export async function deleteAnimationStudioDocumentV1(
  projectId: string,
  confirmed: boolean,
  database?: AnimationStudioDatabaseV1,
): Promise<AnimationStudioDeleteResultV1> {
  if (!confirmed) return { kind: "CONFIRMATION_REQUIRED" };
  try {
    const target = database ?? await openAnimationStudioDatabaseV1();
    await target.adapter.delete(requireProjectId(projectId));
    return { kind: "DELETED" };
  } catch (error) {
    return {
      kind: "ERROR",
      diagnostic: storageDiagnostic(
        "M2A-ANIMATION-STUDIO-DELETE",
        `Animation Studio delete failed: ${errorMessage(error)}`,
      ),
    };
  }
}

export async function migrateAnimationStudioStorageV1(
  database: AnimationStudioDatabaseV1,
): Promise<AnimationStudioStorageMigrationResultV1> {
  const diagnostics: AnimationStudioDiagnosticV1[] = [];
  let records: unknown[];
  try {
    records = await database.adapter.getAll();
  } catch (error) {
    return {
      kind: "ERROR",
      diagnostics: [storageDiagnostic(
        "M2A-ANIMATION-STUDIO-MIGRATION",
        `Could not inspect Animation Studio storage: ${errorMessage(error)}`,
      )],
    };
  }
  for (const raw of records) {
    const parsed = parseStorageRecord(raw);
    if (parsed.kind === "INVALID") {
      diagnostics.push(...parsed.diagnostics);
      continue;
    }
    const document = parseAnimationStudioDocumentV1(parsed.value.documentJson);
    if (document.kind === "INVALID") diagnostics.push(...document.diagnostics);
  }
  // This V1 gate intentionally performs no writes. Unknown/old/new records are
  // reported and remain byte-for-byte untouched until an explicit future
  // migration is selected by the user.
  return diagnostics.length > 0
    ? { kind: "ERROR", diagnostics }
    : { kind: "MIGRATED", recordsChecked: records.length };
}

export async function listAnimationStudioRecoveryRecordsV1(
  database?: AnimationStudioDatabaseV1,
): Promise<AnimationStudioRecoveryRecordV1[]> {
  const target = database ?? await openAnimationStudioDatabaseV1();
  const records = await target.adapter.getAll();
  return Promise.all(records.map(async (raw): Promise<AnimationStudioRecoveryRecordV1> => {
    const record = parseStorageRecord(raw);
    if (record.kind === "INVALID") {
      return {
        projectId: isRecord(raw) && typeof raw.projectId === "string"
          ? raw.projectId
          : "<unknown>",
        sourceRevision: null,
        authoringRevision: null,
        documentStatus: "INVALID",
        selectedMode: null,
        savedAt: null,
        recoverable: false,
        diagnostics: record.diagnostics,
      };
    }
    const parsed = parseAnimationStudioDocumentV1(record.value.documentJson);
    const integrityDiagnostics: AnimationStudioDiagnosticV1[] = [];
    if (parsed.kind === "VALID") {
      const fingerprint =
        await fingerprintAnimationStudioDocumentV1(parsed.value);
      if (fingerprint !== record.value.documentFingerprintSha256) {
        integrityDiagnostics.push(storageDiagnostic(
          "M2A-ANIMATION-STUDIO-RECOVERY-INTEGRITY",
          "Stored Animation Studio fingerprint does not match its document.",
        ));
      }
      if (
        parsed.value.sourceRevision !== record.value.sourceRevision
        || parsed.value.authoringRevision !== record.value.authoringRevision
      ) {
        integrityDiagnostics.push(storageDiagnostic(
          "M2A-ANIMATION-STUDIO-RECOVERY-INTEGRITY",
          "Stored Animation Studio metadata does not match its document.",
        ));
      }
      if (
        record.value.selectedClipId !== null
        && !parsed.value.authoredClips.some(
          ({ id }) => id === record.value.selectedClipId,
        )
      ) {
        integrityDiagnostics.push(storageDiagnostic(
          "M2A-ANIMATION-STUDIO-RECOVERY-INTEGRITY",
          "Stored Animation Studio selection references an unknown clip.",
        ));
      }
    }
    const diagnostics = parsed.kind === "VALID"
      ? integrityDiagnostics
      : parsed.diagnostics;
    return {
      projectId: record.value.projectId,
      sourceRevision: record.value.sourceRevision,
      authoringRevision: record.value.authoringRevision,
      // Recovery preserves DRAFT/INVALID. It never promotes status to VALID.
      documentStatus: parsed.kind === "VALID" ? parsed.value.status : "INVALID",
      selectedMode: record.value.selectedMode,
      savedAt: record.value.savedAt,
      recoverable: parsed.kind === "VALID" && diagnostics.length === 0,
      diagnostics,
    };
  }));
}

function openIndexedDatabase(
  factory: IDBFactory,
): Promise<AnimationStudioDatabaseV1> {
  return new Promise((resolve, reject) => {
    const request = factory.open(DATABASE_NAME, DATABASE_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(DOCUMENT_STORE)) {
        db.createObjectStore(DOCUMENT_STORE, { keyPath: "projectId" });
      }
    };
    request.onerror = () => reject(
      request.error ?? new Error("IndexedDB open failed"),
    );
    request.onblocked = () => reject(
      new Error("IndexedDB upgrade is blocked by another Studio tab"),
    );
    request.onsuccess = () => {
      const db = request.result;
      resolve({
        schemaVersion: 1,
        adapter: new IndexedDbAnimationStudioAdapterV1(db),
      });
    };
  });
}

class IndexedDbAnimationStudioAdapterV1
implements AnimationStudioStorageAdapterV1 {
  constructor(private readonly database: IDBDatabase) {}

  put(record: AnimationStudioStorageRecordV1): Promise<void> {
    return this.request("readwrite", (store) => store.put(record)).then(() => undefined);
  }

  get(projectId: string): Promise<unknown | undefined> {
    return this.request("readonly", (store) => store.get(projectId));
  }

  delete(projectId: string): Promise<void> {
    return this.request("readwrite", (store) => store.delete(projectId))
      .then(() => undefined);
  }

  getAll(): Promise<unknown[]> {
    return this.request("readonly", (store) => store.getAll());
  }

  close(): void {
    this.database.close();
  }

  private request<T>(
    mode: IDBTransactionMode,
    makeRequest: (store: IDBObjectStore) => IDBRequest<T>,
  ): Promise<T> {
    return new Promise((resolve, reject) => {
      const transaction = this.database.transaction(DOCUMENT_STORE, mode);
      const request = makeRequest(transaction.objectStore(DOCUMENT_STORE));
      let result: T;
      request.onerror = () => reject(
        request.error ?? new Error("IndexedDB request failed"),
      );
      transaction.onabort = () => reject(
        transaction.error ?? new Error("IndexedDB transaction aborted"),
      );
      transaction.onerror = () => reject(
        transaction.error ?? new Error("IndexedDB transaction failed"),
      );
      request.onsuccess = () => {
        result = request.result;
      };
      // Autosave becomes visible as successful only after the transaction
      // commits, never merely after the put request has been queued.
      transaction.oncomplete = () => resolve(result);
    });
  }
}

type ParsedStorageRecordV1 =
  | { kind: "VALID"; value: AnimationStudioStorageRecordV1 }
  | { kind: "INVALID"; diagnostics: AnimationStudioDiagnosticV1[] };

function parseStorageRecord(value: unknown): ParsedStorageRecordV1 {
  if (!isRecord(value)) return invalidStorageRecord("Stored record is not an object.");
  const allowed = new Set([
    "storageSchemaVersion",
    "projectId",
    "documentJson",
    "documentFingerprintSha256",
    "sourceRevision",
    "authoringRevision",
    "selectedMode",
    "selectedClipId",
    "savedAt",
  ]);
  const unknown = Object.keys(value).find((key) => !allowed.has(key));
  if (unknown) return invalidStorageRecord(`Unknown stored field: ${unknown}.`);
  if (value.storageSchemaVersion !== 1) {
    return invalidStorageRecord(
      `Unsupported storage schemaVersion: ${String(value.storageSchemaVersion)}.`,
    );
  }
  if (
    typeof value.projectId !== "string"
    || value.projectId.trim().length === 0
    || typeof value.documentJson !== "string"
    || typeof value.documentFingerprintSha256 !== "string"
    || !/^[a-f0-9]{64}$/i.test(value.documentFingerprintSha256)
    || typeof value.sourceRevision !== "string"
    || !Number.isSafeInteger(value.authoringRevision)
    || (value.authoringRevision as number) < 1
    || (value.selectedMode !== "MAP_BASE_42" && value.selectedMode !== "CREATE_EDIT")
    || (value.selectedClipId !== null && typeof value.selectedClipId !== "string")
    || typeof value.savedAt !== "string"
    || !Number.isFinite(Date.parse(value.savedAt))
  ) {
    return invalidStorageRecord("Stored Animation Studio record has invalid fields.");
  }
  return { kind: "VALID", value: value as unknown as AnimationStudioStorageRecordV1 };
}

function invalidStorageRecord(message: string): ParsedStorageRecordV1 {
  return {
    kind: "INVALID",
    diagnostics: [storageDiagnostic("M2A-ANIMATION-STUDIO-STORAGE-SCHEMA", message)],
  };
}

function storageDiagnostic(
  code: string,
  message: string,
): AnimationStudioDiagnosticV1 {
  return {
    schemaVersion: 1,
    code,
    path: "$.storage",
    level: "BLOCKING",
    message,
    action: "Keep the current in-memory edit and retry or export a project backup.",
  };
}

function requireProjectId(projectId: string): string {
  const normalized = projectId.trim();
  if (!normalized) throw new RangeError("Animation Studio projectId is required");
  return normalized;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
