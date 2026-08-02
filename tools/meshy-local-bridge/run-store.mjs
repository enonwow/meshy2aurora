import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";

const RUN_ID = /^[0-9a-f-]{36}$/i;

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function runDirectory(root, runId) {
  if (!RUN_ID.test(runId)) throw new Error("MESHY-RUN-STORE: invalid run identity");
  return join(root, runId);
}

export function persistReadyRunV1(root, run) {
  if (!root || run.status !== "READY" || !run.artifactBytes || !run.provenance) return;
  const directory = runDirectory(root, run.id);
  mkdirSync(directory, { recursive: true });
  const artifactPath = join(directory, "artifact.glb");
  const artifactTemporary = `${artifactPath}.tmp`;
  writeFileSync(artifactTemporary, run.artifactBytes);
  renameSync(artifactTemporary, artifactPath);
  for (const artifact of run.artifacts ?? []) {
    const path = join(directory, `action-${artifact.actionId}.glb`);
    const temporary = `${path}.tmp`;
    writeFileSync(temporary, artifact.bytes);
    renameSync(temporary, path);
  }
  const metadata = {
    schemaVersion: 1,
    run: {
      id: run.id,
      profile: run.profile,
      prompt: run.prompt,
      source: run.source,
      geometryTarget: run.geometryTarget,
      status: run.status,
      progress: run.progress,
      taskIds: run.taskIds,
      createdAt: run.createdAt,
      updatedAt: run.updatedAt,
      provenance: run.provenance,
      ...(run.geometryAdmission ? { geometryAdmission: run.geometryAdmission } : {}),
      artifacts: (run.artifacts ?? []).map(({ actionId, clipName, sha256: digest, byteLength }) => ({
        actionId,
        clipName,
        sha256: digest,
        byteLength,
      })),
    },
  };
  const metadataPath = join(directory, "run.json");
  const metadataTemporary = `${metadataPath}.tmp`;
  writeFileSync(metadataTemporary, `${JSON.stringify(metadata, null, 2)}\n`, "utf8");
  renameSync(metadataTemporary, metadataPath);
}

export function loadReadyRunsV1(root) {
  if (!root || !existsSync(root)) return [];
  const recovered = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    if (!entry.isDirectory() || !RUN_ID.test(entry.name)) continue;
    try {
      const directory = runDirectory(root, entry.name);
      const payload = JSON.parse(readFileSync(join(directory, "run.json"), "utf8"));
      const run = payload?.schemaVersion === 1 ? payload.run : undefined;
      if (!run || run.id !== entry.name || run.status !== "READY" || !run.provenance) continue;
      const artifactBytes = readFileSync(join(directory, "artifact.glb"));
      if (
        artifactBytes.byteLength !== run.provenance.byteLength
        || sha256(artifactBytes) !== run.provenance.sha256
      ) continue;
      const artifacts = (run.artifacts ?? []).map((artifact) => {
        const bytes = readFileSync(join(directory, `action-${artifact.actionId}.glb`));
        if (bytes.byteLength !== artifact.byteLength || sha256(bytes) !== artifact.sha256) {
          throw new Error("stored action artifact identity mismatch");
        }
        return { ...artifact, bytes };
      });
      recovered.push({ ...run, artifactBytes, artifacts });
    } catch {
      // One corrupt/incomplete journal entry cannot hide other immutable runs.
    }
  }
  return recovered.sort((left, right) => right.updatedAt.localeCompare(left.updatedAt));
}
