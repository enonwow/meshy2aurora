import { describe, expect, it } from "vitest";
import {
  deleteAnimationStudioDocumentV1,
  listAnimationStudioRecoveryRecordsV1,
  loadAnimationStudioDocumentV1,
  migrateAnimationStudioStorageV1,
  openAnimationStudioDatabaseV1,
  saveAnimationStudioDocumentV1,
  type AnimationStudioStorageRecordV1,
} from "./persistence";
import { animationStudioDocumentFixtureV1 } from "./testFixtures";

describe.runIf(typeof indexedDB !== "undefined")(
  "Animation Studio native IndexedDB adapter",
  () => {
    it("round-trips, detects damaged recovery, and leaves legacy data untouched", async () => {
      const database = await openAnimationStudioDatabaseV1();
      const suffix = crypto.randomUUID();
      const projectId = `browser-${suffix}`;
      const legacyProjectId = `browser-legacy-${suffix}`;
      const document = animationStudioDocumentFixtureV1();
      const legacyRecord = {
        storageSchemaVersion: 0,
        projectId: legacyProjectId,
        payload: "leave-native-indexeddb-byte-equivalent",
      };

      try {
        await expect(saveAnimationStudioDocumentV1(
          projectId,
          document,
          database,
          {
            selectedMode: "CREATE_EDIT",
            selectedClipId: "authored_attack_01",
          },
        )).resolves.toMatchObject({ kind: "SAVED" });
        await expect(loadAnimationStudioDocumentV1(projectId, database))
          .resolves.toMatchObject({
            kind: "LOADED",
            value: document,
            selectedMode: "CREATE_EDIT",
            selectedClipId: "authored_attack_01",
          });

        const stored = await database.adapter.get(projectId) as
          AnimationStudioStorageRecordV1;
        await database.adapter.put({
          ...stored,
          documentFingerprintSha256: "0".repeat(64),
        });
        const recovery = await listAnimationStudioRecoveryRecordsV1(database);
        expect(recovery.find((record) => record.projectId === projectId))
          .toMatchObject({
            recoverable: false,
            diagnostics: [expect.objectContaining({
              code: "M2A-ANIMATION-STUDIO-RECOVERY-INTEGRITY",
            })],
          });

        await database.adapter.put(
          legacyRecord as unknown as AnimationStudioStorageRecordV1,
        );
        const beforeMigration = await database.adapter.get(legacyProjectId);
        await expect(migrateAnimationStudioStorageV1(database))
          .resolves.toMatchObject({
            kind: "ERROR",
            diagnostics: [expect.objectContaining({
              code: "M2A-ANIMATION-STUDIO-STORAGE-SCHEMA",
            })],
          });
        expect(await database.adapter.get(legacyProjectId))
          .toEqual(beforeMigration);

        await expect(deleteAnimationStudioDocumentV1(
          projectId,
          true,
          database,
        )).resolves.toEqual({ kind: "DELETED" });
        await expect(loadAnimationStudioDocumentV1(projectId, database))
          .resolves.toEqual({ kind: "EMPTY" });
      } finally {
        await database.adapter.delete(projectId);
        await database.adapter.delete(legacyProjectId);
        database.adapter.close?.();
      }
    });
  },
);
