#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const executionProfilePath = resolve(
  repoRoot,
  "proof-profiles/m0-r29-all-state-identity-runtime-v1.json",
);
const sidecarPath = resolve(
  repoRoot,
  "proof-output/m0-r29-all-state-identity-20260721/m0-r29-runtime-v2-binding-sidecar-v1.json",
);

const executionProfile = readJson(executionProfilePath);
const sidecar = readJson(sidecarPath);
const binaryProfile = readJson(sidecar.sourceBinaryProfile.path);
const lineage = readJson(sidecar.lineageContract.path);
const gateB = readJson(sidecar.toolsetProof.path);

validateContract(executionProfile, sidecar, binaryProfile, lineage, gateB);

for (const mutate of [
  (profile) => profile.orderedHakList.push({ resref: "m2a_m0r28", sha256: "0".repeat(64) }),
  (_profile, binding) => { binding.toolsetProof.sha256 = "0".repeat(64); },
  (profile) => { profile.module.sha256 = "0".repeat(64); },
  (_profile, binding) => { binding.runtimeTarget.fixture.appearanceRow = 847; },
  (_profile, binding) => { binding.runtimeTarget.resources.model.sha256 = "0".repeat(64); },
  (_profile, binding) => { binding.toolsetProof.capture.sha256 = "0".repeat(64); },
  (profile) => { profile.module.area = "m2a_m0a28"; },
  (_profile, binding) => { binding.toolsetProof.modelVisibility = "not_visible"; },
]) {
  const profileCopy = structuredClone(executionProfile);
  const sidecarCopy = structuredClone(sidecar);
  mutate(profileCopy, sidecarCopy);
  assert.throws(
    () => validateContract(profileCopy, sidecarCopy, binaryProfile, lineage, gateB),
    "mutated runtime identity must fail closed",
  );
}

console.log(JSON.stringify({
  ok: true,
  status: "m0_r29_runtime_profile_and_binding_contract_valid",
  executionProfile: {
    path: executionProfilePath,
    sha256: sha256(executionProfilePath),
  },
  bindingSidecar: {
    path: sidecarPath,
    sha256: sha256(sidecarPath),
  },
  module: sidecar.runtimeTarget.module,
  area: sidecar.runtimeTarget.area,
  orderedHakList: sidecar.runtimeTarget.orderedHakList,
  fixture: sidecar.runtimeTarget.fixture,
  gateB: {
    packet: sidecar.toolsetProof.path,
    packetSha256: sidecar.toolsetProof.sha256,
    capture: sidecar.toolsetProof.capture,
    modelVisibility: sidecar.toolsetProof.modelVisibility,
    proofCompleteness: sidecar.toolsetProof.proofCompleteness,
  },
  negativeCases: 8,
  startsToolset: false,
  startsNwn: false,
}, null, 2));

function validateContract(profile, binding, source, lineageContract, packet) {
  assert.deepEqual(profile, {
    version: "aur-s07-runtime-profile/v1",
    id: "m2a-m0-r29-all-state-identity-runtime",
    module: {
      sha256: "c28af8aff97be895d80c9ac01d5aa5ec909ec6b2a96f8e6ffd0317ffb16931d4",
      area: "m2a_m0a29",
      entryPoint: [10, 10, 0],
    },
    orderedHakList: [{
      resref: "m2a_m0r29",
      sha256: "e8c007faace65255cca2e76845db435eadf19c06c1455dedada133a95ddaf9a3",
    }],
    geometryGate: {
      status: "verified",
      evidence: resolve(
        repoRoot,
        "proof-output/m0-r29-all-state-identity-20260721/m0-r29-all-state-identity-lineage-contract-v1.json",
      ),
    },
    fixtures: [{
      id: "m0_fixture",
      resref: "m2a_m0p01",
      templateResRef: "nw_dwarfmerc001",
      appearanceRow: 848,
      position: [10, 14.5, 0],
    }],
    launch: { mode: "user_owned_test_module_action", processName: "nwmain" },
    noLocalSubstitute: true,
  });

  assert.equal(binding.version, "m2a-aur-s07-runtime-v2-binding-sidecar/v1");
  assert.equal(binding.id, "m2a-m0-r29-all-state-identity-runtime-v2-binding");
  assert.deepEqual(binding.executionProfile, {
    path: executionProfilePath,
    sha256: "926af303858c86ecb6f923b862527d25034c2b1aef34725e79f47575b6e88814",
    version: "aur-s07-runtime-profile/v1",
    publicRunner: "C:\\Projects\\aurora-web\\backend\\scripts\\aur-s07-runtime-execution.mjs",
  });

  for (const artifact of [
    binding.executionProfile,
    binding.sourceBinaryProfile,
    binding.lineageContract,
    binding.toolsetProof,
    binding.toolsetProof.profile,
    binding.toolsetProof.capture,
    binding.toolsetProof.captureResult,
    { path: binding.runtimeTarget.module.sourcePath, sha256: binding.runtimeTarget.module.sha256 },
    { path: binding.runtimeTarget.module.installedPath, sha256: binding.runtimeTarget.module.sha256 },
    ...binding.runtimeTarget.orderedHakList,
    lineageContract.candidate.resources.model,
    lineageContract.candidate.resources.texture,
    lineageContract.candidate.resources.appearanceTwoDa,
  ]) {
    assert.equal(existsSync(artifact.path), true, `missing bound artifact: ${artifact.path}`);
    assert.equal(sha256(artifact.path), artifact.sha256, `hash mismatch: ${artifact.path}`);
  }

  assert.equal(binding.executionProfile.sha256, sha256(executionProfilePath));
  assert.equal(binding.sourceBinaryProfile.sha256, sha256(binding.sourceBinaryProfile.path));
  assert.equal(binding.lineageContract.sha256, sha256(binding.lineageContract.path));
  assert.equal(binding.toolsetProof.sha256, sha256(binding.toolsetProof.path));

  assert.equal(packet.version, "aurora-toolset-binary-module-toolset-proof-packet/v1");
  assert.equal(packet.status, "verified");
  assert.equal(packet.gate, "B");
  assert.equal(packet.verdict.modelVisibility, "visible");
  assert.equal(packet.verdict.proofCompleteness, "verified");
  assert.equal(packet.profiles.binaryBootstrap.path, binding.sourceBinaryProfile.path);
  assert.equal(packet.profiles.binaryBootstrap.sha256, binding.sourceBinaryProfile.sha256);
  assert.equal(packet.profiles.toolsetProof.path, binding.toolsetProof.profile.path);
  assert.equal(packet.profiles.toolsetProof.sha256, binding.toolsetProof.profile.sha256);
  assert.equal(packet.profiles.lineageContract.path, binding.lineageContract.path);
  assert.equal(packet.profiles.lineageContract.sha256, binding.lineageContract.sha256);
  assert.equal(packet.visualInspection.inspectedCaptureSha256, binding.toolsetProof.capture.sha256);
  assert.equal(packet.capture.pngPath, binding.toolsetProof.capture.path);
  assert.equal(packet.capture.pngSha256, binding.toolsetProof.capture.sha256);
  assert.equal(packet.capture.captureResultPath, binding.toolsetProof.captureResult.path);
  assert.equal(packet.capture.captureResultSha256, binding.toolsetProof.captureResult.sha256);
  assert.equal(binding.toolsetProof.modelVisibility, packet.verdict.modelVisibility);
  assert.equal(binding.toolsetProof.proofCompleteness, packet.verdict.proofCompleteness);

  assert.equal(binding.runtimeTarget.module.resref, source.module.resref);
  assert.equal(binding.runtimeTarget.module.sourcePath, source.module.path);
  assert.equal(binding.runtimeTarget.module.sha256, source.module.sha256);
  assert.equal(binding.runtimeTarget.module.resref, packet.module.resref);
  assert.equal(binding.runtimeTarget.module.sourcePath, packet.module.sourcePath);
  assert.equal(binding.runtimeTarget.module.installedPath, packet.module.installedPath);
  assert.equal(binding.runtimeTarget.module.sha256, packet.module.expectedSha256);
  assert.equal(binding.runtimeTarget.area, source.area.resref);
  assert.equal(binding.runtimeTarget.area, packet.area.resref);
  assert.deepEqual(binding.runtimeTarget.entryPoint, source.entryPoint.position);
  assert.deepEqual(profile.module, {
    sha256: binding.runtimeTarget.module.sha256,
    area: binding.runtimeTarget.area,
    entryPoint: binding.runtimeTarget.entryPoint,
  });

  assert.equal(binding.runtimeTarget.orderedHakList.length, 1);
  assert.deepEqual(binding.runtimeTarget.orderedHakList, source.orderedHakList);
  assert.deepEqual(binding.runtimeTarget.orderedHakList, packet.orderedHakList.beforeCapture);
  assert.deepEqual(packet.orderedHakList.afterCapture, packet.orderedHakList.beforeCapture);
  assert.deepEqual(
    profile.orderedHakList,
    binding.runtimeTarget.orderedHakList.map(({ resref, sha256 }) => ({ resref, sha256 })),
  );

  const fixture = binding.runtimeTarget.fixture;
  assert.deepEqual(fixture, {
    id: "m0_fixture",
    treeObjectText: "Meshy M0 binary vertical-slice fixture",
    modelResRef: "m2a_m0p01",
    templateResRef: "nw_dwarfmerc001",
    appearanceRow: 848,
    position: [10, 14.5, 0],
    orientation: [1, 0],
    scale: 1,
  });
  assert.equal(source.fixtures.length, 1);
  assert.equal(source.fixtures[0].id, fixture.id);
  assert.equal(source.fixtures[0].templateResRef, fixture.templateResRef);
  assert.equal(source.fixtures[0].appearanceType, fixture.appearanceRow);
  assert.deepEqual(source.fixtures[0].position, fixture.position);
  assert.equal(packet.fixture.sourceIdentity.id, fixture.id);
  assert.equal(packet.fixture.sourceIdentity.templateResRef, fixture.templateResRef);
  assert.equal(packet.fixture.sourceIdentity.appearanceType, fixture.appearanceRow);
  assert.deepEqual(packet.fixture.sourceIdentity.position, fixture.position);
  assert.deepEqual(packet.fixture.sourceIdentity.orientation, fixture.orientation);
  assert.equal(packet.fixture.sourceIdentity.scale, fixture.scale);
  assert.equal(packet.fixture.selector.treeObjectText, fixture.treeObjectText);
  assert.equal(packet.fixture.selectionResult.matchCount, 1);

  assert.deepEqual(binding.runtimeTarget.resources.model, {
    resref: lineageContract.candidate.resources.model.resref,
    sha256: lineageContract.candidate.resources.model.sha256,
  });
  assert.deepEqual(binding.runtimeTarget.resources.texture, {
    resref: lineageContract.candidate.resources.texture.resref,
    sha256: lineageContract.candidate.resources.texture.sha256,
  });
  assert.deepEqual(binding.runtimeTarget.resources.appearanceTwoDa, {
    resref: lineageContract.candidate.resources.appearanceTwoDa.resref,
    physicalRow: lineageContract.candidate.resources.appearanceTwoDa.physicalRow,
    label: lineageContract.candidate.resources.appearanceTwoDa.label,
    modelType: lineageContract.candidate.resources.appearanceTwoDa.modelType,
    race: lineageContract.candidate.resources.appearanceTwoDa.race,
    sha256: lineageContract.candidate.resources.appearanceTwoDa.sha256,
  });
  assert.equal(
    packet.fixture.sourceIdentity.appearanceBinding.modelSha256,
    binding.runtimeTarget.resources.model.sha256,
  );
  assert.equal(
    packet.fixture.sourceIdentity.appearanceBinding.textureSha256,
    binding.runtimeTarget.resources.texture.sha256,
  );
  assert.equal(
    packet.fixture.sourceIdentity.appearanceBinding.appearanceTwoDaSha256,
    binding.runtimeTarget.resources.appearanceTwoDa.sha256,
  );
  assert.deepEqual(binding.fixtureTargets, [{ fixtureId: fixture.id, modelResRef: fixture.modelResRef }]);
  assert.deepEqual(binding.launch, profile.launch);
  assert.equal(binding.engineLog.loadingModuleResref, binding.runtimeTarget.module.resref);
  assert.equal(binding.noLocalSubstitute, true);
  assert.deepEqual(binding.safety, {
    startsToolset: false,
    startsNwn: false,
    usesGlobalInput: false,
    savesModule: false,
    modifiesR27: false,
    modifiesR28: false,
    modifiesR29: false,
    implementsLocalRunner: false,
  });
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}
