#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const executionProfilePath = resolve(
  repoRoot,
  "proof-profiles/m0-r30-retail-runtime-conformance-runtime-v1.json",
);
const sidecarPath = resolve(
  repoRoot,
  "proof-output/m0-r30-retail-runtime-conformance-20260721/m0-r30-runtime-v2-binding-sidecar-v1.json",
);

const executionProfile = readJson(executionProfilePath);
const sidecar = readJson(sidecarPath);
const binaryProfile = readJson(sidecar.sourceBinaryProfile.path);
const lineage = readJson(sidecar.lineageContract.path);
const gateB = readJson(sidecar.toolsetProof.path);
const materializationSummary = readJson(sidecar.runtimeConformance.productionContract.summary.path);

validateContract(executionProfile, sidecar, binaryProfile, lineage, gateB, materializationSummary);

for (const mutate of [
  (profile) => profile.orderedHakList.push({ resref: "m2a_m0r29", sha256: "0".repeat(64) }),
  (_profile, binding) => { binding.toolsetProof.sha256 = "0".repeat(64); },
  (profile) => { profile.module.sha256 = "0".repeat(64); },
  (_profile, binding) => { binding.runtimeTarget.fixture.appearanceRow = 848; },
  (_profile, binding) => { binding.runtimeTarget.resources.model.sha256 = "0".repeat(64); },
  (_profile, binding) => { binding.toolsetProof.capture.sha256 = "0".repeat(64); },
  (profile) => { profile.module.area = "m2a_m0a29"; },
  (_profile, binding) => { binding.toolsetProof.modelVisibility = "not_visible"; },
  (_profile, binding) => {
    binding.runtimeConformance.stateProjection.profile = "CEP_RIGID_PLACEHOLDER_V1";
  },
  (_profile, binding) => {
    binding.runtimeConformance.stateProjection.provenance = { family: "CEP" };
  },
  (_profile, binding) => {
    binding.runtimeConformance.appearanceTable.scope = "ISOLATED_TOOLSET_VERTICAL_SLICE";
  },
  (_profile, binding) => {
    binding.runtimeConformance.appearanceTable.source.sha256 = "0".repeat(64);
  },
  (_profile, binding) => {
    binding.runtimeConformance.appearanceTable.source.path =
      binding.runtimeConformance.appearanceTable.output.path;
  },
  (_profile, binding) => {
    binding.runtimeConformance.appearanceTable.sourcePrefixPreserved = false;
  },
]) {
  const profileCopy = structuredClone(executionProfile);
  const sidecarCopy = structuredClone(sidecar);
  mutate(profileCopy, sidecarCopy);
  assert.throws(
    () => validateContract(
      profileCopy,
      sidecarCopy,
      binaryProfile,
      lineage,
      gateB,
      materializationSummary,
    ),
    "mutated r30 runtime identity or provenance must fail closed",
  );
}

console.log(JSON.stringify({
  ok: true,
  status: "m0_r30_runtime_profile_and_binding_contract_valid",
  executionProfile: { path: executionProfilePath, sha256: sha256(executionProfilePath) },
  bindingSidecar: { path: sidecarPath, sha256: sha256(sidecarPath) },
  module: sidecar.runtimeTarget.module,
  area: sidecar.runtimeTarget.area,
  orderedHakList: sidecar.runtimeTarget.orderedHakList,
  fixture: sidecar.runtimeTarget.fixture,
  stateProjection: sidecar.runtimeConformance.stateProjection,
  appearanceTable: sidecar.runtimeConformance.appearanceTable,
  gateB: {
    packet: sidecar.toolsetProof.path,
    packetSha256: sidecar.toolsetProof.sha256,
    capture: sidecar.toolsetProof.capture,
    modelVisibility: sidecar.toolsetProof.modelVisibility,
    proofCompleteness: sidecar.toolsetProof.proofCompleteness,
  },
  negativeCases: 14,
  startsToolset: false,
  startsNwn: false,
}, null, 2));

function validateContract(profile, binding, source, lineageContract, packet, summary) {
  assert.deepEqual(profile, {
    version: "aur-s07-runtime-profile/v1",
    id: "m2a-m0-r30-retail-runtime-conformance-runtime",
    module: {
      sha256: "8c87317956053cecdf40d939c9061a174943b982a8db2c94d35aa2fba44043df",
      area: "m2a_m0a30",
      entryPoint: [10, 10, 0],
    },
    orderedHakList: [{
      resref: "m2a_m0r30",
      sha256: "6694c0fb7f16665025d312dfd35c4bcbb293fccdb0760c110492a5eb2d576d09",
    }],
    geometryGate: {
      status: "verified",
      evidence: resolve(
        repoRoot,
        "proof-output/m0-r30-retail-runtime-conformance-20260721/m0-r30-retail-runtime-conformance-lineage-contract-v1.json",
      ),
    },
    fixtures: [{
      id: "m0_fixture",
      resref: "m2a_m0p01",
      templateResRef: "nw_dwarfmerc001",
      appearanceRow: 15100,
      position: [10, 14.5, 0],
    }],
    launch: { mode: "user_owned_test_module_action", processName: "nwmain" },
    noLocalSubstitute: true,
  });

  assert.equal(binding.version, "m2a-aur-s07-runtime-v2-binding-sidecar/v1");
  assert.equal(binding.id, "m2a-m0-r30-retail-runtime-conformance-runtime-v2-binding");
  assert.deepEqual(binding.executionProfile, {
    path: executionProfilePath,
    sha256: sha256(executionProfilePath),
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
    binding.runtimeConformance.productionContract.summary,
    { path: binding.runtimeTarget.module.sourcePath, sha256: binding.runtimeTarget.module.sha256 },
    { path: binding.runtimeTarget.module.installedPath, sha256: binding.runtimeTarget.module.sha256 },
    ...binding.runtimeTarget.orderedHakList,
    binding.runtimeTarget.resources.model,
    binding.runtimeTarget.resources.texture,
    binding.runtimeTarget.resources.appearanceTwoDa,
    binding.runtimeConformance.appearanceTable.source,
    binding.runtimeConformance.appearanceTable.output,
  ]) {
    assert.equal(existsSync(artifact.path), true, `missing bound artifact: ${artifact.path}`);
    assert.equal(sha256(artifact.path), artifact.sha256, `hash mismatch: ${artifact.path}`);
  }

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
  assert.equal(packet.capture.pngPath, binding.toolsetProof.capture.path);
  assert.equal(packet.capture.pngSha256, binding.toolsetProof.capture.sha256);
  assert.equal(packet.capture.captureResultPath, binding.toolsetProof.captureResult.path);
  assert.equal(packet.capture.captureResultSha256, binding.toolsetProof.captureResult.sha256);
  assert.equal(packet.visualInspection.inspectedCaptureSha256, binding.toolsetProof.capture.sha256);
  assert.equal(binding.toolsetProof.modelVisibility, packet.verdict.modelVisibility);
  assert.equal(binding.toolsetProof.proofCompleteness, packet.verdict.proofCompleteness);

  assert.equal(binding.runtimeTarget.module.resref, source.module.resref);
  assert.equal(binding.runtimeTarget.module.sourcePath, source.module.path);
  assert.equal(binding.runtimeTarget.module.sha256, source.module.sha256);
  assert.equal(binding.runtimeTarget.module.resref, packet.module.resref);
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
    binding.runtimeTarget.orderedHakList.map(({ resref, sha256: hash }) => ({ resref, sha256: hash })),
  );

  const fixture = binding.runtimeTarget.fixture;
  assert.deepEqual(fixture, {
    id: "m0_fixture",
    treeObjectText: "Meshy M0 binary vertical-slice fixture",
    modelResRef: "m2a_m0p01",
    templateResRef: "nw_dwarfmerc001",
    appearanceRow: 15100,
    position: [10, 14.5, 0],
    orientation: [1, 0],
    scale: 1,
  });
  assert.equal(packet.fixture.sourceIdentity.id, fixture.id);
  assert.equal(packet.fixture.sourceIdentity.templateResRef, fixture.templateResRef);
  assert.equal(packet.fixture.sourceIdentity.appearanceType, fixture.appearanceRow);
  assert.deepEqual(packet.fixture.sourceIdentity.position, fixture.position);
  assert.deepEqual(packet.fixture.sourceIdentity.orientation, fixture.orientation);
  assert.equal(packet.fixture.sourceIdentity.scale, fixture.scale);
  assert.equal(packet.fixture.selector.treeObjectText, fixture.treeObjectText);
  assert.equal(packet.fixture.selectionResult.matchCount, 1);

  assert.deepEqual(binding.runtimeTarget.resources.model, lineageContract.candidate.resources.model);
  assert.deepEqual(binding.runtimeTarget.resources.texture, lineageContract.candidate.resources.texture);
  assert.deepEqual(
    binding.runtimeTarget.resources.appearanceTwoDa,
    lineageContract.candidate.resources.appearanceTwoDa,
  );
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

  const runtime = binding.runtimeConformance;
  const lineageRuntime = lineageContract.candidate.runtimeFixtureContract;
  assert.equal(runtime.productionContract.runtimeContractVerified, true);
  assert.equal(lineageContract.materialization.runtimeContractVerified, true);
  assert.equal(runtime.productionContract.lineagePath, binding.lineageContract.path);
  assert.equal(runtime.productionContract.lineageSha256, binding.lineageContract.sha256);
  assert.equal(runtime.stateProjection.profile, "RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1");
  assert.equal(runtime.stateProjection.provenance, null);
  assert.deepEqual(runtime.stateProjection.summary, lineageRuntime.stateProjectionSummary);
  assert.equal(runtime.stateProjection.summarySha256, lineageRuntime.stateProjectionSummarySha256);
  assert.equal(
    sha256Bytes(Buffer.from(JSON.stringify(runtime.stateProjection.summary))),
    runtime.stateProjection.summarySha256,
  );
  assert.deepEqual(summary.m0RuntimeFixtureContract.stateProjectionSummary, runtime.stateProjection.summary);
  assert.equal(
    summary.m0RuntimeFixtureContract.stateProjectionSummarySha256,
    runtime.stateProjection.summarySha256,
  );

  const table = runtime.appearanceTable;
  assert.equal(table.scope, "FULL_RUNTIME_APPEND_V1");
  assert.equal(table.schemaVersion, 1);
  assert.equal(table.source.physicalRows, 15100);
  assert.equal(table.output.physicalRows, 15101);
  assert.equal(table.appendCount, 1);
  assert.equal(table.appendedPhysicalRow, 15100);
  assert.equal(table.appendedPhysicalRow, fixture.appearanceRow);
  assert.equal(table.sourcePrefixPreserved, true);
  assert.equal(table.source.path, lineageContract.candidate.sources.fullAppearanceTwoDa.path);
  assert.equal(table.source.sha256, lineageRuntime.appearanceTable.inputSha256);
  assert.equal(table.source.byteLength, lineageRuntime.appearanceTable.inputByteLength);
  assert.equal(table.output.path, lineageContract.candidate.resources.appearanceTwoDa.path);
  assert.equal(table.output.sha256, lineageRuntime.appearanceTable.outputSha256);
  assert.equal(table.output.byteLength, lineageRuntime.appearanceTable.outputByteLength);
  const sourceAppearance = readFileSync(table.source.path);
  const outputAppearance = readFileSync(table.output.path);
  assert.deepEqual(outputAppearance.subarray(0, sourceAppearance.length), sourceAppearance);
  assert.deepEqual(summary.m0RuntimeFixtureContract.appearanceTable, {
    schemaVersion: table.schemaVersion,
    scope: table.scope,
    inputPhysicalRows: table.source.physicalRows,
    outputPhysicalRows: table.output.physicalRows,
    appendedPhysicalRow: table.appendedPhysicalRow,
    inputByteLength: table.source.byteLength,
    outputByteLength: table.output.byteLength,
    inputSha256: table.source.sha256,
    outputSha256: table.output.sha256,
    sourcePrefixPreserved: table.sourcePrefixPreserved,
  });

  assert.deepEqual(binding.fixtureTargets, [{ fixtureId: fixture.id, modelResRef: fixture.modelResRef }]);
  assert.deepEqual(binding.launch, profile.launch);
  assert.equal(binding.engineLog.loadingModuleResref, binding.runtimeTarget.module.resref);
  assert.equal(binding.noLocalSubstitute, true);
  assert.deepEqual(binding.safety, {
    startsToolset: false,
    startsNwn: false,
    usesGlobalInput: false,
    savesModule: false,
    modifiesR29OrEarlier: false,
    modifiesR30Artifacts: false,
    implementsLocalRunner: false,
  });
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function sha256(path) {
  return sha256Bytes(readFileSync(path));
}

function sha256Bytes(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
