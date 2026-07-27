#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const binaryProfilePath = resolve(
  repoRoot,
  "proof-profiles/m0-r28-single-r27-binary-bootstrap-v1.json",
);
const toolsetProfilePath = resolve(
  repoRoot,
  "proof-profiles/m0-r28-single-r27-toolset-proof-v1.json",
);
const lineagePath = resolve(
  repoRoot,
  "proof-output/m0-r28-single-r27-hak-20260721/m0-r28-single-r27-lineage-contract-v1.json",
);
const materializationManifestPath = resolve(
  repoRoot,
  "proof-output/m0-r27-animation-type5-20260721/reports/materialization-manifest.json",
);

const binaryProfile = readJson(binaryProfilePath);
const toolsetProfile = readJson(toolsetProfilePath);
const lineage = readJson(lineagePath);
const materializationManifest = readJson(materializationManifestPath);

assert.deepEqual(binaryProfile, {
  version: "aurora-toolset-binary-module-bootstrap-profile/v1",
  id: "m2a-m0-r28-single-r27-binary-bootstrap-v1",
  module: {
    resref: "m2a_m0r28",
    path: resolve(
      repoRoot,
      "proof-output/m0-r28-single-r27-hak-20260721/generated/m2a_m0r28.mod",
    ),
    sha256: "08ac62b9b3decd33322657a25996557dcdf96e3db0a2d9f7f292f8b72be4cce2",
  },
  area: { resref: "m2a_m0a28", width: 2, height: 2 },
  entryPoint: { area: "m2a_m0a28", position: [10, 10, 0] },
  orderedHakList: [
    {
      resref: "m2a_m0r27",
      path: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m0r27.hak",
      sha256: "8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd",
    },
  ],
  fixtures: [
    {
      id: "m0_fixture",
      templateResRef: "nw_dwarfmerc001",
      appearanceType: 848,
      position: [10, 14.5, 0],
    },
  ],
  noLocalToolsetAdapter: true,
});

assert.deepEqual(toolsetProfile, {
  version: "aurora-toolset-binary-module-toolset-proof-profile/v1",
  id: "m2a-m0-r28-single-r27-toolset-proof-v1",
  binaryProfile: {
    path: binaryProfilePath,
    sha256: "0d2311b4929e5768249e9e20e5b327efd14e6f12cb73c6785380ef5cfb1b43e3",
  },
  fixtureSelection: {
    fixtureId: "m0_fixture",
    treeObjectText: "Meshy M0 binary vertical-slice fixture",
    occurrence: 0,
  },
  capture: { targetClass: "TScrollBox", allowUniformScene: false },
  noSave: true,
  noLocalToolsetAdapter: true,
});

assert.equal(lineage.version, "m2a-m0-iteration-lineage-contract/v1");
assert.deepEqual(lineage.intendedDelta.orderedHakListBefore, [
  "m2a_m0r27",
  "m2a_m0r26",
  "m2a_m0r21",
]);
assert.deepEqual(lineage.intendedDelta.orderedHakListAfter, ["m2a_m0r27"]);
assert.equal(lineage.intendedDelta.newHakCreated, false);
assert.equal(lineage.intendedDelta.newHakCopied, false);
assert.deepEqual(lineage.intendedDelta.payloadChanges, []);
assert.deepEqual(lineage.candidate.orderedHakList, binaryProfile.orderedHakList);
assert.equal(lineage.candidate.fixture.treeObjectText, "Meshy M0 binary vertical-slice fixture");
assert.equal(lineage.profiles.runtimeProfileMaterialized, false);

for (const binding of [
  lineage.admission.sourceModule,
  lineage.admission.runtimePacket,
  lineage.admission.runtimeCapture,
  binaryProfile.module,
  ...binaryProfile.orderedHakList,
  lineage.profiles.binaryBootstrap,
  lineage.profiles.toolsetProof,
]) {
  assert.equal(existsSync(binding.path), true, `missing bound artifact: ${binding.path}`);
  assert.equal(sha256(binding.path), binding.sha256, `hash mismatch: ${binding.path}`);
}

assert.equal(
  existsSync(resolve(repoRoot, "proof-output/m0-r28-single-r27-hak-20260721/generated/m2a_m0r28.hak")),
  false,
  "r28 must not create or copy a HAK",
);
assert.equal(
  existsSync("C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m0r28.hak"),
  false,
  "r28 must not install a new HAK",
);

const expectedResources = {
  MODEL: lineage.candidate.resources.model,
  TEXTURE: lineage.candidate.resources.texture,
  APPEARANCE_TABLE: lineage.candidate.resources.appearanceTwoDa,
};
for (const resource of materializationManifest.packageManifest.resources) {
  const expected = expectedResources[resource.role];
  assert.equal(resource.resref, expected.resref, `${resource.role} resref changed`);
  assert.equal(resource.byteLength, expected.byteLength, `${resource.role} byte length changed`);
  assert.equal(resource.sha256, expected.sha256, `${resource.role} hash changed`);
}

console.log(JSON.stringify({
  ok: true,
  status: "m0_r28_single_r27_lineage_contract_valid",
  module: binaryProfile.module,
  orderedHakList: binaryProfile.orderedHakList,
  binaryProfile: { path: binaryProfilePath, sha256: sha256(binaryProfilePath) },
  toolsetProfile: { path: toolsetProfilePath, sha256: sha256(toolsetProfilePath) },
  lineage: { path: lineagePath, sha256: sha256(lineagePath) },
  fixtureSelection: toolsetProfile.fixtureSelection,
  startsToolset: false,
  startsNwn: false,
}, null, 2));

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}
