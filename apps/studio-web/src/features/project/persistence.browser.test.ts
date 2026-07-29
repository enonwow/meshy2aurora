import { describe, expect, it } from "vitest";
import {
  auditProjectStorageV1,
  deleteProjectV1,
  listProjectRecoveryRecordsV1,
  loadProjectV1,
  openProjectDatabaseV1,
  saveProjectV1,
  type ProjectStorageRecordV1,
} from "./persistence";
import { createMeshy2AuroraProjectV1 } from "./schema";

describe.runIf(typeof indexedDB !== "undefined")(
  "project native IndexedDB adapter",
  () => {
    it("round-trips, verifies integrity and preserves incompatible records", async () => {
      const database = await openProjectDatabaseV1();
      const suffix = crypto.randomUUID();
      const projectId = `browser-project-${suffix}`;
      const incompatibleId = `browser-project-future-${suffix}`;
      const project = createMeshy2AuroraProjectV1({
        projectId,
        name: "Browser recovery project",
        now: "2026-07-28T12:00:00.000Z",
      });
      const incompatible = {
        storageSchemaVersion: 2,
        projectId: incompatibleId,
        name: "Future project record",
        payload: "leave-untouched",
      };

      try {
        await expect(saveProjectV1(project, database)).resolves.toMatchObject({
          kind: "SAVED",
          projectId,
          revision: 1,
        });
        await expect(loadProjectV1(projectId, database)).resolves.toMatchObject({
          kind: "LOADED",
          value: project,
        });

        const stored = await database.adapter.get(projectId) as ProjectStorageRecordV1;
        await database.adapter.put({
          ...stored,
          projectFingerprintSha256: "0".repeat(64),
        });
        const recovery = await listProjectRecoveryRecordsV1(database);
        expect(recovery.find((record) => record.projectId === projectId))
          .toMatchObject({
            recoverable: false,
            diagnostics: [expect.objectContaining({
              code: "M2A-PROJECT-RECOVERY-INTEGRITY",
            })],
          });

        await database.adapter.put(incompatible as unknown as ProjectStorageRecordV1);
        const beforeAudit = await database.adapter.get(incompatibleId);
        await expect(auditProjectStorageV1(database)).resolves.toMatchObject({
          kind: "ERROR",
        });
        expect(await database.adapter.get(incompatibleId)).toEqual(beforeAudit);

        await expect(deleteProjectV1(projectId, false, database)).resolves.toEqual({
          kind: "CONFIRMATION_REQUIRED",
        });
        await expect(deleteProjectV1(projectId, true, database)).resolves.toEqual({
          kind: "DELETED",
        });
      } finally {
        await database.adapter.delete(projectId);
        await database.adapter.delete(incompatibleId);
      }
    });
  },
);
