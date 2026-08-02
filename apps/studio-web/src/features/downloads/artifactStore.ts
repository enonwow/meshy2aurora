import type { WorkerArtifact } from "../../worker/types";
import { verifyWorkerArtifactV1 } from "./ArtifactDownloads";

const DATABASE_NAME = "m2a-studio-artifacts-v1";
const STORE_NAME = "artifact-sets";
const LATEST_KEY = "latest";

interface StoredArtifactSetV1 {
  readonly key: typeof LATEST_KEY;
  readonly schemaVersion: 1;
  readonly savedAt: string;
  readonly artifacts: WorkerArtifact[];
}

function openDatabase(): Promise<IDBDatabase | undefined> {
  if (typeof indexedDB === "undefined") return Promise.resolve(undefined);
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DATABASE_NAME, 1);
    request.onupgradeneeded = () => {
      if (!request.result.objectStoreNames.contains(STORE_NAME)) {
        request.result.createObjectStore(STORE_NAME, { keyPath: "key" });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("Could not open artifact storage"));
  });
}

export async function persistLatestWorkerArtifactsV1(artifacts: WorkerArtifact[]) {
  if (!artifacts.length) return;
  await Promise.all(artifacts.map(verifyWorkerArtifactV1));
  const database = await openDatabase();
  if (!database) return;
  try {
    await new Promise<void>((resolve, reject) => {
      const transaction = database.transaction(STORE_NAME, "readwrite");
      transaction.objectStore(STORE_NAME).put({
        key: LATEST_KEY,
        schemaVersion: 1,
        savedAt: new Date().toISOString(),
        artifacts,
      } satisfies StoredArtifactSetV1);
      transaction.oncomplete = () => resolve();
      transaction.onerror = () => reject(transaction.error ?? new Error("Could not persist artifacts"));
      transaction.onabort = () => reject(transaction.error ?? new Error("Artifact persistence was aborted"));
    });
  } finally {
    database.close();
  }
}

export async function loadLatestWorkerArtifactsV1(): Promise<WorkerArtifact[]> {
  const database = await openDatabase();
  if (!database) return [];
  try {
    const stored = await new Promise<StoredArtifactSetV1 | undefined>((resolve, reject) => {
      const request = database.transaction(STORE_NAME, "readonly")
        .objectStore(STORE_NAME)
        .get(LATEST_KEY);
      request.onsuccess = () => resolve(request.result as StoredArtifactSetV1 | undefined);
      request.onerror = () => reject(request.error ?? new Error("Could not restore artifacts"));
    });
    if (!stored || stored.schemaVersion !== 1 || !Array.isArray(stored.artifacts)) return [];
    await Promise.all(stored.artifacts.map(verifyWorkerArtifactV1));
    return stored.artifacts;
  } finally {
    database.close();
  }
}
