import { describe, expect, it } from "vitest";
import {
  createInMemoryAnimationStudioDatabaseV1,
  deleteAnimationStudioDocumentV1,
  listAnimationStudioRecoveryRecordsV1,
  loadAnimationStudioDocumentV1,
  migrateAnimationStudioStorageV1,
  openAnimationStudioDatabaseV1,
  saveAnimationStudioDocumentV1,
  type AnimationStudioDatabaseV1,
} from "./persistence";
import { animationStudioDocumentFixtureV1 } from "./testFixtures";

describe("Animation Studio IndexedDB persistence contract V1", () => {
  it("restores exact clips and selected mode after refresh", async () => {
    const database = createInMemoryAnimationStudioDatabaseV1();
    const document = animationStudioDocumentFixtureV1();
    const saved = await saveAnimationStudioDocumentV1(
      "project-1",
      document,
      database,
      {
        selectedMode: "CREATE_EDIT",
        selectedClipId: "authored_attack_01",
      },
    );
    expect(saved).toMatchObject({ kind: "SAVED", revision: 1 });

    await expect(loadAnimationStudioDocumentV1("project-1", database))
      .resolves.toEqual({
        kind: "LOADED",
        value: document,
        selectedMode: "CREATE_EDIT",
        selectedClipId: "authored_attack_01",
        savedAt: expect.any(String),
        fingerprintSha256: expect.stringMatching(/^[a-f0-9]{64}$/),
      });
  });

  it("does not promote a recovered draft to Valid", async () => {
    const database = createInMemoryAnimationStudioDatabaseV1();
    await saveAnimationStudioDocumentV1(
      "project-draft",
      animationStudioDocumentFixtureV1({ status: "DRAFT" }),
      database,
    );
    await expect(listAnimationStudioRecoveryRecordsV1(database)).resolves.toEqual([
      expect.objectContaining({
        projectId: "project-draft",
        documentStatus: "DRAFT",
        recoverable: true,
      }),
    ]);
  });

  it("reports an IndexedDB blocker and never claims autosave", async () => {
    const unavailable: AnimationStudioDatabaseV1 = {
      schemaVersion: 1,
      adapter: {
        put: async () => {
          throw new DOMException("quota", "QuotaExceededError");
        },
        get: async () => undefined,
        delete: async () => undefined,
        getAll: async () => [],
      },
    };
    await expect(saveAnimationStudioDocumentV1(
      "project-1",
      animationStudioDocumentFixtureV1(),
      unavailable,
    )).resolves.toMatchObject({
      kind: "ERROR",
      diagnostic: {
        code: "M2A-ANIMATION-STUDIO-AUTOSAVE",
        level: "BLOCKING",
        message: expect.stringMatching(/quota/i),
      },
    });
  });

  it("does not silently overwrite an old storage schema", async () => {
    const oldRecord = {
      storageSchemaVersion: 0,
      projectId: "legacy",
      payload: "leave-me-exact",
    };
    const database = createInMemoryAnimationStudioDatabaseV1([oldRecord]);
    const before = await database.adapter.get("legacy");
    await expect(migrateAnimationStudioStorageV1(database)).resolves.toMatchObject({
      kind: "ERROR",
      diagnostics: [expect.objectContaining({
        code: "M2A-ANIMATION-STUDIO-STORAGE-SCHEMA",
      })],
    });
    expect(await database.adapter.get("legacy")).toEqual(before);
  });

  it("requires confirmation for deletion", async () => {
    const database = createInMemoryAnimationStudioDatabaseV1();
    await saveAnimationStudioDocumentV1(
      "project-1",
      animationStudioDocumentFixtureV1(),
      database,
    );
    await expect(deleteAnimationStudioDocumentV1("project-1", false, database))
      .resolves.toEqual({ kind: "CONFIRMATION_REQUIRED" });
    await expect(loadAnimationStudioDocumentV1("project-1", database))
      .resolves.toMatchObject({ kind: "LOADED" });
    await expect(deleteAnimationStudioDocumentV1("project-1", true, database))
      .resolves.toEqual({ kind: "DELETED" });
    await expect(loadAnimationStudioDocumentV1("project-1", database))
      .resolves.toEqual({ kind: "EMPTY" });
  });

  it("retries IndexedDB opening after a cached open rejection", async () => {
    const originalIndexedDb = globalThis.indexedDB;
    let openCount = 0;
    const fakeDatabase = {
      objectStoreNames: { contains: () => false },
      createObjectStore: () => undefined,
      close: () => undefined,
    } as unknown as IDBDatabase;
    const factory = {
      open() {
        openCount += 1;
        const attempt = openCount;
        const request = {
          error: attempt === 1
            ? new DOMException("temporary open failure", "UnknownError")
            : null,
          result: fakeDatabase,
          onblocked: null as (() => void) | null,
          onerror: null as (() => void) | null,
          onsuccess: null as (() => void) | null,
          onupgradeneeded: null as (() => void) | null,
        };
        queueMicrotask(() => {
          if (attempt === 1) {
            request.onerror?.();
            return;
          }
          request.onupgradeneeded?.();
          request.onsuccess?.();
        });
        return request as unknown as IDBOpenDBRequest;
      },
    } as unknown as IDBFactory;

    Object.defineProperty(globalThis, "indexedDB", {
      configurable: true,
      value: factory,
    });
    try {
      await expect(openAnimationStudioDatabaseV1()).rejects.toThrow(
        /temporary open failure/,
      );
      await expect(openAnimationStudioDatabaseV1()).resolves.toMatchObject({
        schemaVersion: 1,
      });
      expect(openCount).toBe(2);
    } finally {
      Object.defineProperty(globalThis, "indexedDB", {
        configurable: true,
        value: originalIndexedDb,
      });
    }
  });
});
