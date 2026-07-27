#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const lineagePath = resolve(
  repoRoot,
  "proof-output/m0-r30-retail-runtime-conformance-20260721/m0-r30-retail-runtime-conformance-lineage-contract-v1.json",
);
const lineage = readJson(lineagePath);
const summary = readJson(lineage.materialization.summary.path);
const binaryProfile = readJson(lineage.profiles.binaryBootstrap.path);
const toolsetProfile = readJson(lineage.profiles.toolsetProof.path);

validateContract(lineage, summary, binaryProfile, toolsetProfile);

for (const mutate of [
  (contract) => contract.admission.nwnFailure.packet.sha256 = "0".repeat(64),
  (contract) => contract.candidate.orderedHakList.push(structuredClone(contract.candidate.orderedHakList[0])),
  (contract) => contract.candidate.runtimeFixtureContract.stateProjectionProfile = "CEP_RIGID_PLACEHOLDER_V1",
  (contract) => contract.candidate.runtimeFixtureContract.stateProjectionProvenance = {},
  (contract) => contract.candidate.runtimeFixtureContract.stateProjectionSummarySha256 = "0".repeat(64),
  (contract) => contract.candidate.runtimeFixtureContract.appearanceTable.scope = "ISOLATED_TOOLSET_VERTICAL_SLICE",
  (contract) => contract.candidate.runtimeFixtureContract.appearanceTable.inputSha256 = "0".repeat(64),
  (contract) => contract.profiles.runtimeProfileMaterialized = true,
]) {
  const contract = structuredClone(lineage);
  mutate(contract);
  assert.throws(
    () => validateContract(contract, summary, binaryProfile, toolsetProfile),
    "a mutated r30 integration lineage must fail closed",
  );
}

console.log(JSON.stringify({
  ok: true,
  status: "m0_r30_retail_runtime_conformance_lineage_valid",
  lineage: { path: lineagePath, sha256: sha256(lineagePath) },
  module: lineage.candidate.module,
  orderedHakList: lineage.candidate.orderedHakList,
  model: lineage.candidate.resources.model,
  texture: lineage.candidate.resources.texture,
  appearanceTwoDa: lineage.candidate.resources.appearanceTwoDa,
  stateProjection: {
    profile: lineage.candidate.runtimeFixtureContract.stateProjectionProfile,
    provenance: lineage.candidate.runtimeFixtureContract.stateProjectionProvenance,
    summary: lineage.candidate.runtimeFixtureContract.stateProjectionSummary,
    summarySha256: lineage.candidate.runtimeFixtureContract.stateProjectionSummarySha256,
  },
  appearanceTable: lineage.candidate.runtimeFixtureContract.appearanceTable,
  profiles: lineage.profiles,
  negativeCases: 8,
  startsToolset: false,
  startsNwn: false,
}, null, 2));

function validateContract(contract, materializationSummary, binary, toolset) {
  assert.equal(contract.version, "m2a-m0-runtime-conformance-lineage-contract/v1");
  assert.equal(contract.id, "m2a-m0-r30-retail-runtime-conformance");
  assert.equal(contract.status, "offline_materialized_gate_b_pending");
  assert.equal(contract.admission.nwnFailure.modelVisibility, "not_visible");
  assert.equal(contract.admission.nwnFailure.proofCompleteness, "failed");
  assert.equal(
    contract.admission.nwnFailure.packet.sha256,
    "4539b88024fb1b8977cc2a2f92554f6a78cdf0234570bbc01423bc55488b6fb6",
  );
  assert.equal(
    contract.admission.nwnFailure.capture.sha256,
    "5836a5e43dd9d29c57522cbbd7a7d1f87808f476e5d4da2dda64d0456634e10d",
  );
  assert.equal(
    contract.admission.codeFixEvidence.sha256,
    "dd16cc4b113d578a70b0bdc4a38e5ff75b8865fe673c6888ebb33a26004a9f40",
  );
  assert.deepEqual(
    [contract.admission.cleanup.nwmain, contract.admission.cleanup.nwtoolset],
    [0, 0],
  );

  assert.equal(contract.intendedDelta.stateProjectionProfile, "RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1");
  assert.equal(contract.intendedDelta.stateProjectionProvenance, null);
  assert.equal(contract.intendedDelta.animationStateTree.contentFlags, "0x01");
  assert.equal(contract.intendedDelta.animationStateTree.clipCount, 7);
  assert.equal(contract.intendedDelta.animationStateTree.meshPayload, false);
  assert.equal(contract.intendedDelta.animationStateTree.skinPayload, false);
  assert.equal(contract.intendedDelta.animationStateTree.rawMdxPayload, false);
  assert.equal(contract.intendedDelta.appearancePolicy, "FULL_RUNTIME_APPEND_V1");
  assert.deepEqual(contract.intendedDelta.lineageIdentity.orderedHakResrefs, ["m2a_m0r30"]);

  assert.equal(contract.candidate.module.resref, "m2a_m0r30");
  assert.equal(contract.candidate.areaResref, "m2a_m0a30");
  assert.equal(contract.candidate.orderedHakList.length, 1);
  assert.equal(contract.candidate.orderedHakList[0].resref, "m2a_m0r30");
  assert.equal(contract.candidate.fixture.appearanceRow, 15100);
  assert.equal(contract.candidate.resources.model.resref, "m2a_m0p01");
  assert.equal(contract.candidate.resources.texture.resref, "m2a_m0t01");

  const runtime = contract.candidate.runtimeFixtureContract;
  assert.equal(runtime.stateProjectionProfile, "RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1");
  assert.equal(runtime.stateProjectionProvenance, null);
  assert.deepEqual(runtime.stateProjectionSummary, {
    schemaVersion: 1,
    modelName: "m2a_m0p01",
    byteLength: 150024,
    baseNodeCount: 2,
    baseMeshNodeCount: 1,
    baseSkinNodeCount: 0,
    baseMaxDepth: 1,
    animationCount: 7,
    animationTypeHistogram: { "5": 7 },
    fullBaseTopologyProjectionCount: 7,
    allGenericDummyProjectionCount: 7,
    cepRigidPlaceholderProjectionCount: 0,
  });
  assert.equal(
    sha256Bytes(Buffer.from(JSON.stringify(runtime.stateProjectionSummary))),
    runtime.stateProjectionSummarySha256,
  );
  assert.equal(runtime.appearanceTable.scope, "FULL_RUNTIME_APPEND_V1");
  assert.equal(runtime.appearanceTable.outputPhysicalRows, runtime.appearanceTable.inputPhysicalRows + 1);
  assert.equal(runtime.appearanceTable.appendedPhysicalRow, runtime.appearanceTable.inputPhysicalRows);
  assert.equal(runtime.appearanceTable.appendedPhysicalRow, contract.candidate.fixture.appearanceRow);
  assert.equal(runtime.appearanceTable.sourcePrefixPreserved, true);

  const materializedRuntime = materializationSummary.m0RuntimeFixtureContract;
  assert.equal(materializedRuntime.stateProjectionProfile, runtime.stateProjectionProfile);
  assert.equal(Object.hasOwn(materializedRuntime, "stateProjectionProvenance"), false);
  assert.deepEqual(materializedRuntime.stateProjectionSummary, runtime.stateProjectionSummary);
  assert.equal(materializedRuntime.stateProjectionSummarySha256, runtime.stateProjectionSummarySha256);
  assert.deepEqual(materializedRuntime.appearanceTable, runtime.appearanceTable);
  assert.deepEqual(materializedRuntime.binaryScene.orderedHakResrefs, ["m2a_m0r30"]);
  assert.equal(materializedRuntime.binaryScene.areaResref, "m2a_m0a30");
  assert.equal(materializedRuntime.appearance.physicalRow, 15100);
  assert.equal(materializedRuntime.meshEligibility.eligible, true);

  for (const artifact of [
    contract.admission.sourceCandidate.module,
    contract.admission.sourceCandidate.hak,
    contract.admission.sourceCandidate.model,
    contract.admission.nwnFailure.packet,
    contract.admission.nwnFailure.capture,
    contract.admission.cleanup,
    contract.admission.codeFixEvidence,
    contract.candidate.sources.glb,
    contract.candidate.sources.fullAppearanceTwoDa,
    contract.candidate.resources.model,
    contract.candidate.resources.texture,
    contract.candidate.resources.appearanceTwoDa,
    contract.materialization.summary,
    contract.materialization.report,
    contract.materialization.manifest,
    contract.profiles.binaryBootstrap,
    contract.profiles.toolsetProof,
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
  ]) {
    assert.equal(existsSync(artifact.path), true, `missing bound artifact: ${artifact.path}`);
    assert.equal(sha256(artifact.path), artifact.sha256, `hash mismatch: ${artifact.path}`);
  }

  assert.deepEqual(
    readFileSync(contract.candidate.module.sourcePath),
    readFileSync(contract.candidate.module.installedPath),
  );
  assert.deepEqual(
    readFileSync(contract.candidate.orderedHakList[0].sourcePath),
    readFileSync(contract.candidate.orderedHakList[0].installedPath),
  );
  assert.deepEqual(
    readFileSync(contract.candidate.resources.texture.path),
    readFileSync(resolve(repoRoot, "proof-output/m0-r29-all-state-identity-20260721/generated/m2a_m0t01.tga")),
  );

  const currentRawMdx = rawMdx(readFileSync(contract.candidate.resources.model.path));
  const previousRawMdx = rawMdx(readFileSync(contract.admission.sourceCandidate.model.path));
  assert.deepEqual(currentRawMdx, previousRawMdx);
  assert.equal(currentRawMdx.length, contract.candidate.resources.model.rawMdxByteLength);
  assert.equal(sha256Bytes(currentRawMdx), contract.candidate.resources.model.rawMdxSha256);
  assert.equal(
    contract.candidate.resources.model.baseSemanticDigestSha256,
    contract.intendedDelta.preserved.baseSemanticDigestSha256,
  );

  const fullAppearance = readFileSync(contract.candidate.sources.fullAppearanceTwoDa.path);
  const emittedAppearance = readFileSync(contract.candidate.resources.appearanceTwoDa.path);
  assert.equal(fullAppearance.length, runtime.appearanceTable.inputByteLength);
  assert.equal(emittedAppearance.length, runtime.appearanceTable.outputByteLength);
  assert.deepEqual(emittedAppearance.subarray(0, fullAppearance.length), fullAppearance);
  assert.equal(sha256Bytes(fullAppearance), runtime.appearanceTable.inputSha256);
  assert.equal(sha256Bytes(emittedAppearance), runtime.appearanceTable.outputSha256);

  const hakResources = readErfResources(contract.candidate.orderedHakList[0].sourcePath);
  assert.deepEqual([...hakResources.keys()].sort(), ["appearance/2017", "m2a_m0p01/2002", "m2a_m0t01/3"]);
  assert.equal(hakResources.get("m2a_m0p01/2002").sha256, contract.candidate.resources.model.sha256);
  assert.equal(hakResources.get("m2a_m0t01/3").sha256, contract.candidate.resources.texture.sha256);
  assert.equal(hakResources.get("appearance/2017").sha256, contract.candidate.resources.appearanceTwoDa.sha256);

  assert.deepEqual(binary, {
    version: "aurora-toolset-binary-module-bootstrap-profile/v1",
    id: "m2a-m0-r30-retail-runtime-conformance-binary-bootstrap-v1",
    module: {
      resref: "m2a_m0r30",
      path: contract.candidate.module.sourcePath,
      sha256: contract.candidate.module.sha256,
    },
    area: { resref: "m2a_m0a30", width: 2, height: 2 },
    entryPoint: { area: "m2a_m0a30", position: [10, 10, 0] },
    orderedHakList: [{
      resref: "m2a_m0r30",
      path: contract.candidate.orderedHakList[0].installedPath,
      sha256: contract.candidate.orderedHakList[0].sha256,
    }],
    fixtures: [{
      id: "m0_fixture",
      templateResRef: "nw_dwarfmerc001",
      appearanceType: 15100,
      position: [10, 14.5, 0],
    }],
    noLocalToolsetAdapter: true,
  });
  assert.deepEqual(toolset, {
    version: "aurora-toolset-binary-module-toolset-proof-profile/v1",
    id: "m2a-m0-r30-retail-runtime-conformance-toolset-proof-v1",
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

  assert.equal(contract.materialization.runtimeContractVerified, true);
  assert.equal(contract.materialization.materializationCount, 1);
  assert.equal(contract.profiles.runtimeProfileMaterialized, false);
  assert.deepEqual(contract.proofState.toolset, {
    modelVisibility: "not_tested",
    proofCompleteness: "missing",
  });
  assert.deepEqual(contract.proofState.nwn, {
    modelVisibility: "not_tested",
    proofCompleteness: "missing",
  });
  assert.deepEqual(contract.safety, {
    startsToolset: false,
    startsNwn: false,
    savesModule: false,
    usesLocalToolsetAdapter: false,
    nativeInstallAbsentTargetOnly: true,
    installedBytesMatchSource: true,
    modifiesR29OrEarlier: false,
    runtimeProfileDeferredUntilGateB: true,
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
      sha256: sha256Bytes(payload),
    });
  }
  return resources;
}

function rawMdx(bytes) {
  const start = 12 + bytes.readUInt32LE(4);
  const length = bytes.readUInt32LE(8);
  return bytes.subarray(start, start + length);
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
