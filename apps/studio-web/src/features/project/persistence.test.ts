// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import {
  auditProjectStorageV1,
  createInMemoryProjectDatabaseV1,
  deleteProjectV1,
  listProjectRecoveryRecordsV1,
  loadProjectV1,
  saveProjectV1,
  type ProjectDatabaseV1,
} from "./persistence";
import { createMeshy2AuroraProjectV1 } from "./schema";

function project() {
  return createMeshy2AuroraProjectV1({
    projectId: "project-storage-01",
    name: "Stored creature",
    now: "2026-07-28T12:00:00.000Z",
  });
}

describe("project persistence", () => {
  it("saves, lists, loads and confirmation-gates deletion", async () => {
    const database = createInMemoryProjectDatabaseV1();
    const saved = await saveProjectV1(project(), database);
    expect(saved.kind).toBe("SAVED");

    const recovery = await listProjectRecoveryRecordsV1(database);
    expect(recovery).toEqual([expect.objectContaining({
      projectId: "project-storage-01",
      name: "Stored creature",
      recoverable: true,
    })]);

    const loaded = await loadProjectV1("project-storage-01", database);
    expect(loaded.kind).toBe("LOADED");
    if (loaded.kind === "LOADED") {
      expect(loaded.value.identity.name).toBe("Stored creature");
    }

    expect(await deleteProjectV1("project-storage-01", false, database))
      .toEqual({ kind: "CONFIRMATION_REQUIRED" });
    expect((await loadProjectV1("project-storage-01", database)).kind).toBe("LOADED");
    expect(await deleteProjectV1("project-storage-01", true, database))
      .toEqual({ kind: "DELETED" });
    expect((await loadProjectV1("project-storage-01", database)).kind).toBe("EMPTY");
  });

  it("reports corrupt and incompatible records without mutating them", async () => {
    const incompatible = {
      storageSchemaVersion: 2,
      projectId: "project-future-01",
      name: "Future",
    };
    const database = createInMemoryProjectDatabaseV1([incompatible]);
    const recovery = await listProjectRecoveryRecordsV1(database);

    expect(recovery[0]).toEqual(expect.objectContaining({
      projectId: "project-future-01",
      recoverable: false,
    }));
    expect(recovery[0]?.diagnostics[0]?.code).toBe("M2A-PROJECT-STORAGE-SCHEMA");
    expect((await auditProjectStorageV1(database)).kind).toBe("ERROR");
    expect(await database.adapter.get("project-future-01")).toEqual(incompatible);
  });

  it("turns quota exhaustion into an actionable diagnostic", async () => {
    const database: ProjectDatabaseV1 = {
      schemaVersion: 1,
      adapter: {
        async put() {
          throw new DOMException("full", "QuotaExceededError");
        },
        async get() {
          return undefined;
        },
        async delete() {},
        async getAll() {
          return [];
        },
      },
    };

    const result = await saveProjectV1(project(), database);
    expect(result.kind).toBe("ERROR");
    if (result.kind === "ERROR") {
      expect(result.diagnostic.code).toBe("M2A-PROJECT-STORAGE-QUOTA");
      expect(result.diagnostic.action).toContain("Export");
    }
  });
});
