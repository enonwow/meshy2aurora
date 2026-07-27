#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const lineagePath = resolve(
  repoRoot,
  "proof-output/m0-r29-all-state-identity-20260721/m0-r29-all-state-identity-lineage-contract-v1.json",
);
const lineage = readJson(lineagePath);
const binaryProfile = readJson(lineage.profiles.binaryBootstrap.path);
const toolsetProfile = readJson(lineage.profiles.toolsetProof.path);

validateContract(lineage, binaryProfile, toolsetProfile);

for (const mutate of [
  (contract) => contract.admission.nwn.packet.sha256 = "0".repeat(64),
  (contract) => contract.intendedDelta.addedStreams.streamCount = 11,
  (contract) => contract.candidate.orderedHakList.push(structuredClone(contract.candidate.orderedHakList[0])),
  (contract) => contract.candidate.resources.model.baseSemanticDigestSha256 = "0".repeat(64),
]) {
  const contract = structuredClone(lineage);
  mutate(contract);
  assert.throws(
    () => validateContract(contract, binaryProfile, toolsetProfile),
    "a mutated r29 lineage must fail closed",
  );
}

console.log(JSON.stringify({
  ok: true,
  status: "m0_r29_all_state_identity_lineage_valid",
  lineage: { path: lineagePath, sha256: sha256(lineagePath) },
  binaryProfile: {
    path: lineage.profiles.binaryBootstrap.path,
    sha256: lineage.profiles.binaryBootstrap.sha256,
  },
  toolsetProfile: {
    path: lineage.profiles.toolsetProof.path,
    sha256: lineage.profiles.toolsetProof.sha256,
  },
  module: lineage.candidate.module,
  orderedHakList: lineage.candidate.orderedHakList,
  model: lineage.candidate.resources.model,
  addedStreams: lineage.intendedDelta.addedStreams,
  negativeCases: 4,
  startsToolset: false,
  startsNwn: false,
}, null, 2));

function validateContract(contract, binary, toolset) {
  assert.equal(contract.version, "m2a-m0-iteration-lineage-contract/v1");
  assert.equal(contract.id, "m2a-m0-r29-all-state-identity");
  assert.equal(contract.admission.toolset.modelVisibility, "visible");
  assert.equal(contract.admission.toolset.proofCompleteness, "verified");
  assert.equal(contract.admission.nwn.modelVisibility, "not_visible");
  assert.equal(contract.admission.nwn.proofCompleteness, "failed");
  assert.deepEqual(contract.admission.cleanup.nwmain, 0);
  assert.deepEqual(contract.admission.cleanup.nwtoolset, 0);
  assert.deepEqual(contract.admission.sourceOrderedHakList.map(({ resref }) => resref), ["m2a_m0r27"]);

  assert.equal(contract.intendedDelta.sourcePath, "crates/m2a-core/src/model_pipeline.rs");
  assert.equal(contract.intendedDelta.codePath, "static_direct_creature_runtime_clips");
  assert.equal(contract.intendedDelta.animationTypeBefore, 5);
  assert.equal(contract.intendedDelta.animationTypeAfter, 5);
  assert.equal(contract.intendedDelta.cpause1Changed, false);
  assert.equal(contract.intendedDelta.addedStreams.clipCount, 6);
  assert.equal(contract.intendedDelta.addedStreams.streamCount, 12);
  assert.deepEqual(
    contract.intendedDelta.addedStreams.perClip.map(({ controllerType }) => controllerType),
    [8, 20],
  );
  assert.equal(contract.candidate.orderedHakList.length, 1);
  assert.equal(contract.candidate.orderedHakList[0].resref, "m2a_m0r29");
  assert.equal(
    contract.candidate.resources.model.baseSemanticDigestSha256,
    "f4172130a98ff6b3b4f3c9e67e9b915d0b9da72f7dff6293eabed51e369e7b6c",
  );
  assert.equal(
    contract.candidate.resources.previousModel.baseSemanticDigestSha256,
    contract.candidate.resources.model.baseSemanticDigestSha256,
  );
  assert.equal(
    contract.candidate.resources.model.byteLength - contract.candidate.resources.previousModel.byteLength,
    576,
  );

  assert.deepEqual(binary, {
    version: "aurora-toolset-binary-module-bootstrap-profile/v1",
    id: "m2a-m0-r29-all-state-identity-binary-bootstrap-v1",
    module: {
      resref: "m2a_m0r29",
      path: contract.candidate.module.sourcePath,
      sha256: contract.candidate.module.sha256,
    },
    area: { resref: "m2a_m0a29", width: 2, height: 2 },
    entryPoint: { area: "m2a_m0a29", position: [10, 10, 0] },
    orderedHakList: [{
      resref: "m2a_m0r29",
      path: contract.candidate.orderedHakList[0].installedPath,
      sha256: contract.candidate.orderedHakList[0].sha256,
    }],
    fixtures: [{
      id: "m0_fixture",
      templateResRef: "nw_dwarfmerc001",
      appearanceType: 848,
      position: [10, 14.5, 0],
    }],
    noLocalToolsetAdapter: true,
  });
  assert.deepEqual(toolset, {
    version: "aurora-toolset-binary-module-toolset-proof-profile/v1",
    id: "m2a-m0-r29-all-state-identity-toolset-proof-v1",
    binaryProfile: {
      path: contract.profiles.binaryBootstrap.path,
      sha256: contract.profiles.binaryBootstrap.sha256,
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

  for (const artifact of [
    contract.admission.sourceModule,
    contract.admission.toolset.packet,
    contract.admission.nwn.packet,
    contract.admission.nwn.capture,
    contract.admission.nwn.engineLog,
    contract.admission.cleanup,
    { path: contract.candidate.module.sourcePath, sha256: contract.candidate.module.sha256 },
    { path: contract.candidate.module.installedPath, sha256: contract.candidate.module.sha256 },
    {
      path: contract.candidate.orderedHakList[0].sourcePath,
      sha256: contract.candidate.orderedHakList[0].sha256,
    },
    {
      path: contract.candidate.orderedHakList[0].installedPath,
      sha256: contract.candidate.orderedHakList[0].sha256,
    },
    contract.candidate.resources.model,
    contract.candidate.resources.previousModel,
    contract.candidate.resources.texture,
    contract.candidate.resources.appearanceTwoDa,
    contract.profiles.binaryBootstrap,
    contract.profiles.toolsetProof,
  ]) {
    assert.equal(existsSync(artifact.path), true, `missing bound artifact: ${artifact.path}`);
    assert.equal(sha256(artifact.path), artifact.sha256, `hash mismatch: ${artifact.path}`);
  }

  assert.equal(sha256(contract.profiles.binaryBootstrap.path), contract.profiles.binaryBootstrap.sha256);
  assert.equal(sha256(contract.profiles.toolsetProof.path), contract.profiles.toolsetProof.sha256);
  assert.deepEqual(
    readFileSync(contract.candidate.resources.texture.path),
    readFileSync(resolve(repoRoot, "proof-output/m0-r27-animation-type5-20260721/generated/m2a_m0t01.tga")),
  );
  assert.deepEqual(
    readFileSync(contract.candidate.resources.appearanceTwoDa.path),
    readFileSync(resolve(repoRoot, "proof-output/m0-r27-animation-type5-20260721/generated/appearance.2da")),
  );

  const hakResources = readErfResources(contract.candidate.orderedHakList[0].sourcePath);
  assert.deepEqual([...hakResources.keys()].sort(), ["appearance/2017", "m2a_m0p01/2002", "m2a_m0t01/3"]);
  assert.equal(hakResources.get("m2a_m0p01/2002").sha256, contract.candidate.resources.model.sha256);
  assert.equal(hakResources.get("m2a_m0t01/3").sha256, contract.candidate.resources.texture.sha256);
  assert.equal(hakResources.get("appearance/2017").sha256, contract.candidate.resources.appearanceTwoDa.sha256);
  // This immutable lineage record predates Gate B.  Its false value is a
  // point-in-time fact, not a permanent assertion that later monotonic proof
  // artifacts must remain absent.  The later runtime profile/sidecar have
  // their own hash-bound contract test.
  assert.equal(contract.profiles.runtimeProfileMaterialized, false);
  assert.deepEqual(contract.safety, {
    startsToolset: false,
    startsNwn: false,
    savesModule: false,
    modifiesR27: false,
    modifiesR28: false,
    usesLocalToolsetAdapter: false,
    nativeInstallAbsentTargetOnly: true,
    installedBytesMatchSource: true,
  });
}

function readErfResources(path) {
  const bytes = readFileSync(path);
  assert.equal(bytes.subarray(4, 8).toString("ascii"), "V1.0");
  const count = bytes.readUInt32LE(16);
  const keyOffset = bytes.readUInt32LE(24);
  const resourceOffset = bytes.readUInt32LE(28);
  const resources = new Map();
  for (let index = 0; index < count; index += 1) {
    const key = keyOffset + index * 24;
    const resrefBytes = bytes.subarray(key, key + 16);
    const nul = resrefBytes.indexOf(0);
    const resref = resrefBytes.subarray(0, nul < 0 ? 16 : nul).toString("ascii");
    const resourceId = bytes.readUInt32LE(key + 16);
    const type = bytes.readUInt16LE(key + 20);
    assert.equal(resourceId, index);
    const descriptor = resourceOffset + index * 8;
    const payloadOffset = bytes.readUInt32LE(descriptor);
    const payloadSize = bytes.readUInt32LE(descriptor + 4);
    const payload = bytes.subarray(payloadOffset, payloadOffset + payloadSize);
    resources.set(`${resref.toLowerCase()}/${type}`, {
      byteLength: payload.length,
      sha256: createHash("sha256").update(payload).digest("hex"),
    });
  }
  return resources;
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}
