import assert from "node:assert/strict";
import { createHash, randomUUID } from "node:crypto";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { loadReadyRunsV1, persistReadyRunV1 } from "./run-store.mjs";

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

test("persists and recovers one exact READY run without secrets or signed URLs", () => {
  const directory = mkdtempSync(join(tmpdir(), "m2a-run-store-"));
  try {
    const runId = randomUUID();
    const artifactBytes = Buffer.from("glTF exact merged artifact");
    const actionBytes = Buffer.from("glTF exact action artifact");
    persistReadyRunV1(directory, {
      id: runId,
      profile: { id: "H1-humanoid-animated/v1" },
      prompt: "durable recovery",
      source: "IMAGE",
      geometryTarget: "BALANCED",
      status: "READY",
      progress: 100,
      taskIds: { RIG: "rig-task", ANIMATE_0: "animation-task" },
      createdAt: "2026-07-31T20:00:00.000Z",
      updatedAt: "2026-07-31T20:01:00.000Z",
      apiOptions: { secretSignedUrl: "https://signed.invalid/must-not-persist" },
      artifactBytes,
      artifacts: [{
        actionId: 0,
        clipName: "cpause1",
        bytes: actionBytes,
        sha256: digest(actionBytes),
        byteLength: actionBytes.byteLength,
      }],
      provenance: {
        sha256: digest(artifactBytes),
        byteLength: artifactBytes.byteLength,
      },
    });

    const [recovered] = loadReadyRunsV1(directory);
    assert.equal(recovered.id, runId);
    assert.deepEqual(recovered.artifactBytes, artifactBytes);
    assert.deepEqual(recovered.artifacts[0].bytes, actionBytes);
    assert.equal(JSON.stringify(recovered).includes("signed.invalid"), false);
    assert.equal(recovered.apiOptions, undefined);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("ignores a journal whose stored artifact no longer matches provenance", () => {
  const directory = mkdtempSync(join(tmpdir(), "m2a-run-store-corrupt-"));
  try {
    const runId = randomUUID();
    const artifactBytes = Buffer.from("glTF valid");
    persistReadyRunV1(directory, {
      id: runId,
      profile: { id: "S1-static-prop/v1" },
      prompt: "corruption gate",
      source: "TEXT",
      geometryTarget: "BALANCED",
      status: "READY",
      progress: 100,
      taskIds: {},
      createdAt: "2026-07-31T20:00:00.000Z",
      updatedAt: "2026-07-31T20:01:00.000Z",
      artifactBytes,
      artifacts: [],
      provenance: { sha256: digest(artifactBytes), byteLength: artifactBytes.byteLength },
    });
    writeFileSync(join(directory, runId, "artifact.glb"), "corrupt");
    assert.deepEqual(loadReadyRunsV1(directory), []);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
