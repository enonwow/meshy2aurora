import {
  parseMeshy2AuroraProjectV1,
  serializeMeshy2AuroraProjectV1,
  type Meshy2AuroraProjectV1,
  type ProjectDiagnosticV1,
} from "./schema";

const DATABASE_NAME = "m2a-projects-v1";
const DATABASE_VERSION = 1;
const PROJECT_STORE = "projects";

export interface ProjectStorageRecordV1 {
  readonly storageSchemaVersion: 1;
  readonly projectId: string;
  readonly name: string;
  readonly target: Meshy2AuroraProjectV1["target"];
  readonly revision: number;
  readonly updatedAt: string;
  readonly projectJson: string;
  readonly projectFingerprintSha256: string;
}

export interface ProjectStorageAdapterV1 {
  put(record: ProjectStorageRecordV1): Promise<void>;
  get(projectId: string): Promise<unknown | undefined>;
  delete(projectId: string): Promise<void>;
  getAll(): Promise<unknown[]>;
  close?(): void;
}

export interface ProjectDatabaseV1 {
  readonly schemaVersion: 1;
  readonly adapter: ProjectStorageAdapterV1;
}

export type ProjectSaveResultV1 =
  | {
      readonly kind: "SAVED";
      readonly projectId: string;
      readonly revision: number;
      readonly savedAt: string;
      readonly fingerprintSha256: string;
    }
  | { readonly kind: "ERROR"; readonly diagnostic: ProjectDiagnosticV1 };

export type ProjectLoadResultV1 =
  | { readonly kind: "EMPTY" }
  | {
      readonly kind: "LOADED";
      readonly value: Meshy2AuroraProjectV1;
      readonly fingerprintSha256: string;
    }
  | { readonly kind: "ERROR"; readonly diagnostics: readonly ProjectDiagnosticV1[] };

export type ProjectDeleteResultV1 =
  | { readonly kind: "DELETED" }
  | { readonly kind: "CONFIRMATION_REQUIRED" }
  | { readonly kind: "ERROR"; readonly diagnostic: ProjectDiagnosticV1 };

export interface ProjectRecoveryRecordV1 {
  readonly projectId: string;
  readonly name: string;
  readonly target: Meshy2AuroraProjectV1["target"] | null;
  readonly revision: number | null;
  readonly updatedAt: string | null;
  readonly sourceSha256: string | null;
  readonly recoverable: boolean;
  readonly diagnostics: readonly ProjectDiagnosticV1[];
}

export type ProjectStorageAuditResultV1 =
  | { readonly kind: "VALID"; readonly recordsChecked: number }
  | { readonly kind: "ERROR"; readonly diagnostics: readonly ProjectDiagnosticV1[] };

const databaseOpenings = new WeakMap<IDBFactory, Promise<ProjectDatabaseV1>>();

export function openProjectDatabaseV1(
  factory: IDBFactory | undefined = globalThis.indexedDB,
): Promise<ProjectDatabaseV1> {
  if (!factory) return Promise.reject(new Error("IndexedDB is unavailable"));
  const cached = databaseOpenings.get(factory);
  if (cached) return cached;
  const opening = openIndexedDatabase(factory);
  databaseOpenings.set(factory, opening);
  return opening.catch((error: unknown) => {
    if (databaseOpenings.get(factory) === opening) databaseOpenings.delete(factory);
    throw error;
  });
}

export function createInMemoryProjectDatabaseV1(
  seed: readonly unknown[] = [],
): ProjectDatabaseV1 {
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

export async function saveProjectV1(
  project: Meshy2AuroraProjectV1,
  database?: ProjectDatabaseV1,
): Promise<ProjectSaveResultV1> {
  try {
    const projectJson = serializeMeshy2AuroraProjectV1(project);
    const fingerprintSha256 = await sha256(projectJson);
    const target = database ?? await openProjectDatabaseV1();
    await target.adapter.put({
      storageSchemaVersion: 1,
      projectId: project.identity.projectId,
      name: project.identity.name,
      target: project.target,
      revision: project.revision,
      updatedAt: project.identity.updatedAt,
      projectJson,
      projectFingerprintSha256: fingerprintSha256,
    });
    return {
      kind: "SAVED",
      projectId: project.identity.projectId,
      revision: project.revision,
      savedAt: new Date().toISOString(),
      fingerprintSha256,
    };
  } catch (error) {
    return {
      kind: "ERROR",
      diagnostic: storageDiagnostic(
        isQuotaError(error) ? "M2A-PROJECT-STORAGE-QUOTA" : "M2A-PROJECT-SAVE",
        isQuotaError(error)
          ? "Browser storage quota was exceeded. The in-memory project remains unchanged."
          : `Project save failed: ${errorMessage(error)}`,
        isQuotaError(error)
          ? "Export a backup, remove an older local project, then retry."
          : "Keep this tab open, export a backup and retry local save.",
      ),
    };
  }
}

export async function loadProjectV1(
  projectId: string,
  database?: ProjectDatabaseV1,
): Promise<ProjectLoadResultV1> {
  try {
    const target = database ?? await openProjectDatabaseV1();
    const raw = await target.adapter.get(requireProjectId(projectId));
    if (raw === undefined) return { kind: "EMPTY" };
    return parseStoredProject(raw);
  } catch (error) {
    return {
      kind: "ERROR",
      diagnostics: [storageDiagnostic(
        "M2A-PROJECT-LOAD",
        `Project load failed: ${errorMessage(error)}`,
        "Keep the current project open or import a project backup.",
      )],
    };
  }
}

export async function deleteProjectV1(
  projectId: string,
  confirmed: boolean,
  database?: ProjectDatabaseV1,
): Promise<ProjectDeleteResultV1> {
  if (!confirmed) return { kind: "CONFIRMATION_REQUIRED" };
  try {
    const target = database ?? await openProjectDatabaseV1();
    await target.adapter.delete(requireProjectId(projectId));
    return { kind: "DELETED" };
  } catch (error) {
    return {
      kind: "ERROR",
      diagnostic: storageDiagnostic(
        "M2A-PROJECT-DELETE",
        `Project delete failed: ${errorMessage(error)}`,
        "Retry deletion or leave the local recovery record unchanged.",
      ),
    };
  }
}

export async function listProjectRecoveryRecordsV1(
  database?: ProjectDatabaseV1,
): Promise<ProjectRecoveryRecordV1[]> {
  const target = database ?? await openProjectDatabaseV1();
  const records = await target.adapter.getAll();
  return Promise.all(records.map(async (raw): Promise<ProjectRecoveryRecordV1> => {
    const parsed = await parseStoredProject(raw);
    if (parsed.kind === "LOADED") {
      return {
        projectId: parsed.value.identity.projectId,
        name: parsed.value.identity.name,
        target: parsed.value.target,
        revision: parsed.value.revision,
        updatedAt: parsed.value.identity.updatedAt,
        sourceSha256: parsed.value.files.sourceGlb?.sha256 ?? null,
        recoverable: true,
        diagnostics: [],
      };
    }
    const rawProjectId = isRecord(raw) && typeof raw.projectId === "string"
      ? raw.projectId
      : "<unknown>";
    const rawName = isRecord(raw) && typeof raw.name === "string"
      ? raw.name
      : "Invalid project record";
    return {
      projectId: rawProjectId,
      name: rawName,
      target: null,
      revision: null,
      updatedAt: null,
      sourceSha256: null,
      recoverable: false,
      diagnostics: parsed.kind === "ERROR"
        ? parsed.diagnostics
        : [storageDiagnostic(
            "M2A-PROJECT-RECOVERY-EMPTY",
            "Stored recovery record is empty.",
            "Delete the empty local record.",
          )],
    };
  }));
}

export async function auditProjectStorageV1(
  database: ProjectDatabaseV1,
): Promise<ProjectStorageAuditResultV1> {
  let records: unknown[];
  try {
    records = await database.adapter.getAll();
  } catch (error) {
    return {
      kind: "ERROR",
      diagnostics: [storageDiagnostic(
        "M2A-PROJECT-STORAGE-AUDIT",
        `Could not inspect project storage: ${errorMessage(error)}`,
        "Retry after browser storage becomes available.",
      )],
    };
  }
  const diagnostics: ProjectDiagnosticV1[] = [];
  for (const raw of records) {
    const parsed = await parseStoredProject(raw);
    if (parsed.kind === "ERROR") diagnostics.push(...parsed.diagnostics);
  }
  // V1 deliberately audits without writes. A future migration must be an
  // explicit transform and may not silently discard unknown data.
  return diagnostics.length > 0
    ? { kind: "ERROR", diagnostics }
    : { kind: "VALID", recordsChecked: records.length };
}

async function parseStoredProject(raw: unknown): Promise<ProjectLoadResultV1> {
  const record = parseStorageRecord(raw);
  if (record.kind === "INVALID") {
    return { kind: "ERROR", diagnostics: record.diagnostics };
  }
  const parsed = parseMeshy2AuroraProjectV1(record.value.projectJson);
  if (parsed.kind === "INVALID") {
    return { kind: "ERROR", diagnostics: parsed.diagnostics };
  }
  const fingerprint = await sha256(record.value.projectJson);
  if (fingerprint !== record.value.projectFingerprintSha256) {
    return {
      kind: "ERROR",
      diagnostics: [storageDiagnostic(
        "M2A-PROJECT-RECOVERY-INTEGRITY",
        "Stored project fingerprint does not match its JSON payload.",
        "Keep the record for diagnosis or delete it and import a known-good backup.",
      )],
    };
  }
  if (
    record.value.projectId !== parsed.value.identity.projectId
    || record.value.name !== parsed.value.identity.name
    || record.value.target !== parsed.value.target
    || record.value.revision !== parsed.value.revision
    || record.value.updatedAt !== parsed.value.identity.updatedAt
  ) {
    return {
      kind: "ERROR",
      diagnostics: [storageDiagnostic(
        "M2A-PROJECT-RECOVERY-INTEGRITY",
        "Stored project index metadata does not match its JSON payload.",
        "Keep the record for diagnosis or import a known-good backup.",
      )],
    };
  }
  return {
    kind: "LOADED",
    value: parsed.value,
    fingerprintSha256: fingerprint,
  };
}

type StorageRecordParseResult =
  | { readonly kind: "VALID"; readonly value: ProjectStorageRecordV1 }
  | { readonly kind: "INVALID"; readonly diagnostics: readonly ProjectDiagnosticV1[] };

function parseStorageRecord(value: unknown): StorageRecordParseResult {
  if (!isRecord(value)) return invalidStorage("Stored project record is not an object.");
  const allowed = new Set([
    "storageSchemaVersion",
    "projectId",
    "name",
    "target",
    "revision",
    "updatedAt",
    "projectJson",
    "projectFingerprintSha256",
  ]);
  const unknown = Object.keys(value).find((key) => !allowed.has(key));
  if (unknown) return invalidStorage(`Unknown stored project field: ${unknown}.`);
  if (value.storageSchemaVersion !== 1) {
    return invalidStorage(
      `Unsupported project storage schemaVersion: ${String(value.storageSchemaVersion)}.`,
    );
  }
  if (
    typeof value.projectId !== "string"
    || value.projectId.trim().length === 0
    || typeof value.name !== "string"
    || value.name.trim().length === 0
    || !["CREATURE", "PLACEABLE", "TILE"].includes(String(value.target))
    || !Number.isSafeInteger(value.revision)
    || Number(value.revision) < 1
    || typeof value.updatedAt !== "string"
    || !Number.isFinite(Date.parse(value.updatedAt))
    || typeof value.projectJson !== "string"
    || typeof value.projectFingerprintSha256 !== "string"
    || !/^[a-f0-9]{64}$/i.test(value.projectFingerprintSha256)
  ) {
    return invalidStorage("Stored project record has invalid fields.");
  }
  return { kind: "VALID", value: value as unknown as ProjectStorageRecordV1 };
}

function invalidStorage(message: string): StorageRecordParseResult {
  return {
    kind: "INVALID",
    diagnostics: [storageDiagnostic(
      "M2A-PROJECT-STORAGE-SCHEMA",
      message,
      "Use a compatible Studio version; the record was left unchanged.",
    )],
  };
}

function openIndexedDatabase(factory: IDBFactory): Promise<ProjectDatabaseV1> {
  return new Promise((resolve, reject) => {
    const request = factory.open(DATABASE_NAME, DATABASE_VERSION);
    request.onupgradeneeded = () => {
      const database = request.result;
      if (!database.objectStoreNames.contains(PROJECT_STORE)) {
        database.createObjectStore(PROJECT_STORE, { keyPath: "projectId" });
      }
    };
    request.onerror = () => reject(request.error ?? new Error("IndexedDB open failed"));
    request.onblocked = () => reject(
      new Error("IndexedDB upgrade is blocked by another Studio tab"),
    );
    request.onsuccess = () => resolve({
      schemaVersion: 1,
      adapter: new IndexedDbProjectAdapterV1(request.result),
    });
  });
}

class IndexedDbProjectAdapterV1 implements ProjectStorageAdapterV1 {
  constructor(private readonly database: IDBDatabase) {}

  put(record: ProjectStorageRecordV1): Promise<void> {
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
    createRequest: (store: IDBObjectStore) => IDBRequest<T>,
  ): Promise<T> {
    return new Promise((resolve, reject) => {
      const transaction = this.database.transaction(PROJECT_STORE, mode);
      const request = createRequest(transaction.objectStore(PROJECT_STORE));
      let result: T;
      request.onerror = () => reject(request.error ?? new Error("IndexedDB request failed"));
      transaction.onabort = () => reject(
        transaction.error ?? new Error("IndexedDB transaction aborted"),
      );
      transaction.onerror = () => reject(
        transaction.error ?? new Error("IndexedDB transaction failed"),
      );
      request.onsuccess = () => {
        result = request.result;
      };
      transaction.oncomplete = () => resolve(result);
    });
  }
}

async function sha256(value: string): Promise<string> {
  const bytes = new TextEncoder().encode(value);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function requireProjectId(projectId: string): string {
  const normalized = projectId.trim();
  if (!normalized) throw new RangeError("projectId is required");
  return normalized;
}

function storageDiagnostic(
  code: string,
  message: string,
  action: string,
): ProjectDiagnosticV1 {
  return {
    schemaVersion: 1,
    code,
    path: "$.storage",
    message,
    action,
  };
}

function isQuotaError(error: unknown): boolean {
  return error instanceof DOMException
    ? error.name === "QuotaExceededError"
    : error instanceof Error && error.name === "QuotaExceededError";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
