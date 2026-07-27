#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const executionProfilePath = resolve(
  repoRoot,
  "proof-profiles/m0-r28-single-r27-runtime-v1.json",
);
const sidecarPath = resolve(
  repoRoot,
  "proof-output/m0-r28-single-r27-hak-20260721/m0-r28-runtime-v2-binding-sidecar-v1.json",
);

const executionProfile = readJson(executionProfilePath);
const sidecar = readJson(sidecarPath);
const binaryProfile = readJson(sidecar.sourceBinaryProfile.path);
const lineage = readJson(sidecar.lineageContract.path);
const gateB = readJson(sidecar.toolsetProof.path);

validateContract(executionProfile, sidecar, binaryProfile, lineage, gateB);

for (const mutate of [
  (profile) => profile.orderedHakList.push({ resref: "m2a_m0r26", sha256: "0".repeat(64) }),
  (_profile, binding) => { binding.toolsetProof.sha256 = "0".repeat(64); },
  (profile) => { profile.module.sha256 = "0".repeat(64); },
  (_profile, binding) => { binding.runtimeTarget.fixture.appearanceRow = 847; },
  (_profile, binding) => { binding.runtimeTarget.resources.model.sha256 = "0".repeat(64); },
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
  status: "m0_r28_runtime_profile_and_binding_contract_valid",
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
  negativeCases: 5,
  startsToolset: false,
  startsNwn: false,
}, null, 2));

function validateContract(profile, binding, source, lineageContract, packet) {
  assert.deepEqual(profile, {
    version: "aur-s07-runtime-profile/v1",
    id: "m2a-m0-r28-single-r27-runtime",
    module: {
      sha256: "08ac62b9b3decd33322657a25996557dcdf96e3db0a2d9f7f292f8b72be4cce2",
      area: "m2a_m0a28",
      entryPoint: [10, 10, 0],
    },
    orderedHakList: [{
      resref: "m2a_m0r27",
      sha256: "8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd",
    }],
    geometryGate: {
      status: "verified",
      evidence: resolve(
        repoRoot,
        "proof-output/m0-r28-single-r27-hak-20260721/m0-r28-single-r27-lineage-contract-v1.json",
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
  assert.equal(binding.id, "m2a-m0-r28-single-r27-runtime-v2-binding");
  assert.deepEqual(binding.executionProfile, {
    path: executionProfilePath,
    sha256: "3f8d2d56e907822a80315bbd3a1dc05aea6a28981110d760cf81663208cf65f8",
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
    ...binding.runtimeTarget.orderedHakList,
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
  assert.deepEqual(profile.orderedHakList, binding.runtimeTarget.orderedHakList.map(({ resref, sha256 }) => ({ resref, sha256 })));

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
  assert.equal(packet.fixture.sourceIdentity.appearanceBinding.modelSha256, binding.runtimeTarget.resources.model.sha256);
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
    implementsLocalRunner: false,
  });
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}
