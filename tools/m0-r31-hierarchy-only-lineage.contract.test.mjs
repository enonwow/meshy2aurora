#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const lineagePath = resolve(repoRoot, "proof-output/m0-r31-hierarchy-only-20260722/m0-r31-hierarchy-only-lineage-contract-v1.json");
const lineage = readJson(lineagePath);
const production = readJson(lineage.candidate.productionContract.path);
const binary = readJson(lineage.profiles.binaryBootstrap.path);
const toolset = readJson(lineage.profiles.toolsetProof.path);

validate(lineage, production, binary, toolset, true);
for (const [index, mutate] of [
  value => value.admission.modelVisibility = "visible",
  value => value.admission.runtimePacket.sha256 = "0".repeat(64),
  value => value.intendedDelta.rawMdxSha256 = "0".repeat(64),
  value => value.intendedDelta.r30ProtectedWriterFieldsSha256 = "0".repeat(64),
  value => value.candidate.sources.derived.geometrySha256 = "0".repeat(64),
  value => value.candidate.orderedHakList.push(structuredClone(value.candidate.orderedHakList[0])),
  value => value.candidate.fixture.appearanceRow = 848,
  value => value.candidate.productionContract.stateProjectionSha256 = "0".repeat(64),
  value => value.profiles.runtimeProfileMaterialized = true,
  value => value.materialization.materializationCount = 2,
].entries()) {
  const changed = structuredClone(lineage);
  mutate(changed);
  assert.throws(() => validate(changed, production, binary, toolset, false), `negative ${index}`);
}

console.log(JSON.stringify({
  ok: true,
  status: "m0_r31_hierarchy_only_lineage_valid",
  lineage: { path: lineagePath, sha256: sha256(lineagePath) },
  module: lineage.candidate.module,
  orderedHakList: lineage.candidate.orderedHakList,
  model: lineage.candidate.resources.model,
  profiles: lineage.profiles,
  materializationCount: lineage.materialization.materializationCount,
  negativeCases: 10,
  startsToolset: false,
  startsNwn: false,
}, null, 2));

function validate(value, contract, binaryProfile, toolsetProfile, verifyFiles) {
  assert.equal(value.version, "m2a-m0-hierarchy-only-lineage-contract/v1");
  assert.equal(value.id, "m2a-m0-r31-hierarchy-only");
  assert.equal(value.status, "offline_materialized_gate_b_pending");
  assert.equal(value.admission.sourceLineage, "m0-r30-retail-runtime-conformance-20260721");
  assert.equal(value.admission.modelVisibility, "not_visible");
  assert.equal(value.admission.proofCompleteness, "failed");
  assert.equal(value.admission.runtimePacket.sha256, "a3ff0ae6708b8235beee1161b06ea7654e999578be39a174ee08c1389176af21");
  assert.equal(value.admission.runtimeCapture.sha256, "a09fdaeea3f2e1c400e88cc332b3a6437ca535502d60ffed5be7be1f95aea9dc");
  assert.equal(value.admission.engineLog.sha256, "5063c46fcc90e577bb85234337c0ef03794271c6ecd561e72007599ca84ae9a8");
  assert.deepEqual([value.admission.cleanup.nwmain, value.admission.cleanup.nwtoolset], [0, 0]);

  assert.equal(value.intendedDelta.only, "SOURCE_DERIVED_IDENTITY_HIERARCHY");
  assert.deepEqual(value.intendedDelta.derivedNodes.map(node => [node.order, node.name, node.parent, node.mesh]), [
    [0, "m2a_m0p01", null, null], [1, "m2a_hier_1", 0, null], [2, "m2a_mesh_anchor", 1, 0],
  ]);
  assert.equal(value.intendedDelta.stateProjection, "RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1");
  assert.equal(value.intendedDelta.stateMeshPayload, false);
  assert.equal(value.intendedDelta.stateControllers, false);
  assert.equal(value.intendedDelta.cepProvenance, null);
  assert.equal(value.intendedDelta.rawMdxSha256, "731749c2e501305356a12939e519ee27a6bf6221bfbb925477d28e8f7fe58507");
  assert.equal(value.intendedDelta.protectedWriterFieldsSha256, value.intendedDelta.r30ProtectedWriterFieldsSha256);

  assert.equal(value.candidate.module.resref, "m2a_m0r31");
  assert.equal(value.candidate.areaResref, "m2a_m0a31");
  assert.equal(value.candidate.orderedHakList.length, 1);
  assert.equal(value.candidate.orderedHakList[0].resref, "m2a_m0r31");
  assert.equal(value.candidate.fixture.appearanceRow, 15100);
  assert.equal(value.candidate.resources.model.resref, "m2a_m0p01");
  assert.equal(value.candidate.resources.texture.resref, "m2a_m0t01");
  assert.equal(value.candidate.sources.original.sha256, "aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1");
  assert.equal(value.candidate.sources.derived.sha256, "88f1443aac1cf385222fc5c8940b6532b721de25e35b091492231ba88a4e2f16");
  assert.equal(value.candidate.sources.derived.geometrySha256, "42cd14c19edecc88907942e1933ff804929b9d9cd46a63c4392cbedee09577cc");
  assert.equal(value.candidate.sources.derived.binSha256, glbBinSha256(value.candidate.sources.original.path));
  assert.equal(value.candidate.sources.derived.binSha256, glbBinSha256(value.candidate.sources.derived.path));

  assert.equal(contract.schemaVersion, 4);
  assert.equal(contract.profile, "MESHY_HIERARCHY_RUNTIME_CANDIDATE_PACKAGE_CONTRACT_V4");
  assert.equal(contract.candidateAdmissible, true);
  assert.equal(contract.structuralVerdict, "STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED");
  assert.equal(contract.runtimeModelVisibility, "not_tested");
  assert.equal(contract.runtimeProofCompleteness, "missing");
  assert.equal(contract.materializationCount, 1);
  assert.equal(contract.runtimeProfileMaterialized, false);
  assert.deepEqual(contract.binaryScene.orderedHakResrefs, ["m2a_m0r31"]);
  assert.equal(contract.binaryScene.areaResref, "m2a_m0a31");
  assert.equal(contract.appearanceTable.scope, "FULL_RUNTIME_APPEND_V1");
  assert.equal(contract.appearanceTable.appendedPhysicalRow, 15100);
  assert.equal(contract.appearanceTable.outputPhysicalRows, contract.appearanceTable.inputPhysicalRows + 1);
  assert.equal(contract.modelContract.candidateRawMdxSha256, contract.modelContract.r30RawMdxSha256);
  assert.equal(contract.modelContract.candidateProtectedWriterFieldsSha256, contract.modelContract.r30ProtectedWriterFieldsSha256);
  assert.equal(contract.modelContract.candidateStateProjectionSha256, value.candidate.productionContract.stateProjectionSha256);
  assert.equal(contract.modelContract.outputTopologySha256, value.candidate.productionContract.outputTopologySha256);
  assert.equal(contract.modelContract.derivedSourceBindingSha256, value.candidate.productionContract.derivedSourceBindingSha256);
  assert.equal(contract.modelContract.candidateEngineEnvelope.baseNodes.length, 4);
  assert.equal(contract.modelContract.candidateEngineEnvelope.animations.length, 7);
  for (const animation of contract.modelContract.candidateEngineEnvelope.animations) {
    assert.equal(animation.animationType, 5);
    assert.equal(animation.topologyMatchesBase, true);
    assert.equal(animation.nodes.length, 4);
    assert.equal(animation.nodes.every(node => node.contentFlags === 1 && node.kind === "DUMMY" && node.controllers.length === 0), true);
  }

  if (verifyFiles) for (const artifact of [
    value.admission.runtimePacket, value.admission.runtimeCapture, value.admission.engineLog,
    value.admission.cleanup, value.admission.v3Evidence,
    value.candidate.sources.original, value.candidate.sources.derived,
    value.candidate.resources.model, value.candidate.resources.texture,
    value.candidate.resources.appearanceTwoDa, value.candidate.productionContract,
    value.profiles.binaryBootstrap, value.profiles.toolsetProof,
    { path: value.candidate.module.sourcePath, sha256: value.candidate.module.sha256 },
    { path: value.candidate.module.installedPath, sha256: value.candidate.module.sha256 },
    { path: value.candidate.orderedHakList[0].sourcePath, sha256: value.candidate.orderedHakList[0].sha256 },
    { path: value.candidate.orderedHakList[0].installedPath, sha256: value.candidate.orderedHakList[0].sha256 },
  ]) {
    assert.equal(existsSync(artifact.path), true, `missing ${artifact.path}`);
    assert.equal(sha256(artifact.path), artifact.sha256, `hash mismatch ${artifact.path}`);
  }
  if (verifyFiles) {
    assert.deepEqual(readFileSync(value.candidate.module.sourcePath), readFileSync(value.candidate.module.installedPath));
    assert.deepEqual(readFileSync(value.candidate.orderedHakList[0].sourcePath), readFileSync(value.candidate.orderedHakList[0].installedPath));
    assert.equal(sha256(value.candidate.resources.texture.path), "079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b");
  }

  assert.deepEqual(binaryProfile, {
    version: "aurora-toolset-binary-module-bootstrap-profile/v1",
    id: "m2a-m0-r31-hierarchy-only-binary-bootstrap-v1",
    module: { resref: "m2a_m0r31", path: value.candidate.module.sourcePath, sha256: value.candidate.module.sha256 },
    area: { resref: "m2a_m0a31", width: 2, height: 2 },
    entryPoint: { area: "m2a_m0a31", position: [10, 10, 0] },
    orderedHakList: [{ resref: "m2a_m0r31", path: value.candidate.orderedHakList[0].installedPath, sha256: value.candidate.orderedHakList[0].sha256 }],
    fixtures: [{ id: "m0_fixture", templateResRef: "nw_dwarfmerc001", appearanceType: 15100, position: [10, 14.5, 0] }],
    noLocalToolsetAdapter: true,
  });
  assert.deepEqual(toolsetProfile, {
    version: "aurora-toolset-binary-module-toolset-proof-profile/v1",
    id: "m2a-m0-r31-hierarchy-only-toolset-proof-v1",
    binaryProfile: { path: value.profiles.binaryBootstrap.path, sha256: value.profiles.binaryBootstrap.sha256 },
    fixtureSelection: { fixtureId: "m0_fixture", treeObjectText: "Meshy M0 binary vertical-slice fixture", occurrence: 0 },
    capture: { targetClass: "TScrollBox", allowUniformScene: false },
    noSave: true,
    noLocalToolsetAdapter: true,
  });
  assert.equal(value.profiles.runtimeProfileMaterialized, false);
  assert.equal(value.materialization.materializationCount, 1);
  assert.equal(value.materialization.startsToolset, false);
  assert.equal(value.materialization.startsNwn, false);
  assert.deepEqual(value.proofState.toolset, { modelVisibility: "not_tested", proofCompleteness: "missing" });
  assert.deepEqual(value.proofState.nwn, { modelVisibility: "not_tested", proofCompleteness: "missing" });
}

function glbBinSha256(path) {
  const bytes = readFileSync(path);
  const jsonLength = bytes.readUInt32LE(12);
  const binLengthOffset = 20 + jsonLength;
  const binLength = bytes.readUInt32LE(binLengthOffset);
  return sha256Bytes(bytes.subarray(binLengthOffset + 8, binLengthOffset + 8 + binLength));
}

function readJson(path) { return JSON.parse(readFileSync(path, "utf8")); }
function sha256(path) { return sha256Bytes(readFileSync(path)); }
function sha256Bytes(bytes) { return createHash("sha256").update(bytes).digest("hex"); }
