import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { dirname, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(scriptDirectory, "..", "..", "..");
const evidenceRoot = resolve(repositoryRoot, "documentation", "evidence");
const registryPath = resolve(
  repositoryRoot,
  "contracts",
  "owner-animation-playback-proofs-v1.json",
);
const catalogPath = resolve(
  repositoryRoot,
  "contracts",
  "community-animation-catalog-v1.json",
);
const registry = JSON.parse(await readFile(registryPath, "utf8"));
const catalog = JSON.parse(await readFile(catalogPath, "utf8"));

function fail(message) {
  throw new Error(`Owner animation playback proof registry: ${message}`);
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function animationPresetProof(value, path) {
  if (
    typeof value !== "object"
    || value === null
    || typeof value.presetId !== "string"
    || !/^[A-Za-z0-9_-]{1,64}$/.test(value.presetId)
    || !Number.isSafeInteger(value.presetVersion)
    || value.presetVersion < 1
    || !isSha256(value.motionSha256)
    || !isSha256(value.rigSignatureSha256)
  ) {
    fail(`${path} must bind one exact preset id/version/motion/rig identity.`);
  }
  return value;
}

function validateAnimationPresetProofClaims(catalogValue, registryEntries) {
  if (
    catalogValue?.schemaVersion !== 1
    || !Array.isArray(catalogValue.entries)
  ) {
    fail("community animation catalog must be schemaVersion 1 with entries.");
  }
  const presetProofs = registryEntries
    .map((entry, index) => (
      entry.animationPreset === undefined
        ? null
        : animationPresetProof(entry.animationPreset, `entries[${index}].animationPreset`)
    ))
    .filter(Boolean);
  const catalogByIdentity = new Map(catalogValue.entries.map((entry) => [
    `${entry.presetId}@${entry.presetVersion}`,
    entry,
  ]));
  for (const entry of catalogValue.entries) {
    if (entry.validationStatus !== "OWNER_NWN_VERIFIED") continue;
    const proof = presetProofs.find((candidate) => (
      candidate.presetId === entry.presetId
      && candidate.presetVersion === entry.presetVersion
      && candidate.motionSha256 === entry.motionSha256
      && candidate.rigSignatureSha256 === entry.rigSignatureSha256
    ));
    if (!proof) {
      fail(
        `catalog preset ${entry.presetId}@${entry.presetVersion} claims OWNER_NWN_VERIFIED without exact registry evidence.`,
      );
    }
  }
  for (const proof of presetProofs) {
    const identity = `${proof.presetId}@${proof.presetVersion}`;
    const entry = catalogByIdentity.get(identity);
    if (
      !entry
      || entry.validationStatus !== "OWNER_NWN_VERIFIED"
      || entry.motionSha256 !== proof.motionSha256
      || entry.rigSignatureSha256 !== proof.rigSignatureSha256
    ) {
      fail(`registry animation preset proof ${identity} is stale or not promoted in the catalog.`);
    }
  }
}

function selfTestAnimationPresetProofClaims() {
  const pipeline = {
    schemaVersion: 1,
    entries: [{
      presetId: "m2a_test",
      presetVersion: 1,
      motionSha256: "1".repeat(64),
      rigSignatureSha256: "2".repeat(64),
      validationStatus: "PIPELINE_VERIFIED",
    }],
  };
  validateAnimationPresetProofClaims(pipeline, []);
  const owner = {
    ...pipeline,
    entries: [{ ...pipeline.entries[0], validationStatus: "OWNER_NWN_VERIFIED" }],
  };
  assert.throws(() => validateAnimationPresetProofClaims(owner, []));
  const proof = {
    animationPreset: {
      presetId: "m2a_test",
      presetVersion: 1,
      motionSha256: "1".repeat(64),
      rigSignatureSha256: "2".repeat(64),
    },
  };
  validateAnimationPresetProofClaims(owner, [proof]);
  assert.throws(() => validateAnimationPresetProofClaims(pipeline, [proof]));
  assert.throws(() => validateAnimationPresetProofClaims(owner, [{
    animationPreset: { ...proof.animationPreset, motionSha256: "3".repeat(64) },
  }]));
}

selfTestAnimationPresetProofClaims();

if (
  registry?.schemaVersion !== 1
  || !Array.isArray(registry.entries)
) {
  fail("schemaVersion 1 with an entries array is required.");
}

const evidenceIds = new Set();
for (const [index, entry] of registry.entries.entries()) {
  const path = `entries[${index}]`;
  if (entry?.schemaVersion !== 1) fail(`${path}.schemaVersion must be 1.`);
  if (
    typeof entry.evidenceId !== "string"
    || !entry.evidenceId.trim()
    || evidenceIds.has(entry.evidenceId)
  ) {
    fail(`${path}.evidenceId must be non-empty and unique.`);
  }
  evidenceIds.add(entry.evidenceId);
  if (
    typeof entry.evidencePath !== "string"
    || !/^documentation\/evidence\/[A-Za-z0-9._/-]+\.md$/.test(
      entry.evidencePath,
    )
    || entry.evidencePath.includes("..")
  ) {
    fail(`${path}.evidencePath must name a tracked Markdown evidence file.`);
  }
  if (
    !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$/.test(
      entry.verifiedAt,
    )
    || !Number.isFinite(Date.parse(entry.verifiedAt))
  ) {
    fail(`${path}.verifiedAt must be an ISO-8601 UTC timestamp.`);
  }
  if (
    !isSha256(entry.evidenceSha256)
    || !isSha256(entry.animationStudioFingerprintSha256)
    || !isSha256(entry.outputs?.model)
    || !isSha256(entry.outputs?.hak)
    || !isSha256(entry.outputs?.proofModule)
  ) {
    fail(`${path} contains an invalid SHA-256 identity.`);
  }
  if (
    entry.toolset?.modelVisibility !== "visible"
    || entry.toolset?.proofCompleteness !== "verified"
    || entry.nwn?.modelVisibility !== "visible"
    || entry.nwn?.proofCompleteness !== "verified"
    || entry.nwn?.animationPlayback !== "verified"
  ) {
    fail(`${path} must record verified visible Toolset and NWN playback axes.`);
  }

  const absoluteEvidencePath = resolve(repositoryRoot, entry.evidencePath);
  const relativeEvidencePath = relative(evidenceRoot, absoluteEvidencePath);
  if (
    relativeEvidencePath.startsWith(`..${sep}`)
    || relativeEvidencePath === ".."
    || relativeEvidencePath.startsWith(sep)
  ) {
    fail(`${path}.evidencePath escapes documentation/evidence.`);
  }
  let evidenceBytes;
  try {
    evidenceBytes = await readFile(absoluteEvidencePath);
  } catch (error) {
    fail(
      `${path}.evidencePath cannot be read: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }
  const evidenceSha256 = createHash("sha256")
    .update(evidenceBytes)
    .digest("hex");
  if (evidenceSha256 !== entry.evidenceSha256) {
    fail(`${path}.evidenceSha256 does not match the exact evidence file.`);
  }
}

validateAnimationPresetProofClaims(catalog, registry.entries);

console.log(
  `owner-animation-playback-proofs-ok: ${registry.entries.length} entr${
    registry.entries.length === 1 ? "y" : "ies"
  }`,
);
