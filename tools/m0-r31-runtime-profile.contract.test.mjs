#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const profilePath = resolve(repoRoot, "proof-profiles/m0-r31-hierarchy-only-runtime-v1.json");
const sidecarPath = resolve(
  repoRoot,
  "proof-output/m0-r31-hierarchy-only-20260722/m0-r31-runtime-v2-binding-sidecar-v1.json",
);

const profile = readJson(profilePath);
const sidecar = readJson(sidecarPath);
const binary = readJson(sidecar.sourceBinaryProfile.path);
const lineage = readJson(sidecar.lineageContract.path);
const production = readJson(sidecar.productionCandidateContract.path);
const packet = readJson(sidecar.toolsetProof.path);
const capture = readJson(sidecar.toolsetProof.captureResult.path);

validate(profile, sidecar, binary, lineage, production, packet, capture);

const attacks = [
  (p) => p.orderedHakList.push({ resref: "m2a_m0r30", sha256: "0".repeat(64) }),
  (_p, s) => { s.runtimeTarget.module.sha256 = "0".repeat(64); },
  (_p, s) => { s.runtimeTarget.orderedHakList[0].sha256 = "0".repeat(64); },
  (_p, s) => { s.toolsetProof.sha256 = "0".repeat(64); },
  (_p, s) => { s.toolsetProof.capture.sha256 = "0".repeat(64); },
  (_p, s) => { s.toolsetProof.captureResult.sha256 = "0".repeat(64); },
  (_p, s) => { s.toolsetProof.modelVisibility = "not_visible"; },
  (_p, s) => { s.toolsetProof.selectionHandle = 0; },
  (_p, s) => { s.runtimeTarget.fixture.appearanceRow = 848; },
  (_p, s) => { s.runtimeTarget.resources.model.sha256 = "0".repeat(64); },
  (_p, s) => { s.runtimeConformance.stateProjection.profile = "CEP_RIGID_PLACEHOLDER_V1"; },
  (_p, s) => { s.runtimeConformance.stateProjection.summaryDigestSha256 = "0".repeat(64); },
  (_p, s) => { s.runtimeConformance.sourceTopology.outputTopologySha256 = "0".repeat(64); },
  (_p, s) => { s.runtimeConformance.engineEnvelope.digestSha256 = "0".repeat(64); },
  (_p, s) => { s.runtimeConformance.appearanceTable.scope = "ISOLATED_TOOLSET_VERTICAL_SLICE"; },
  (_p, s) => { s.runtimeConformance.appearanceTable.sourcePrefixPreserved = false; },
  (_p, s) => { s.runtimeConformance.appearanceTable.source.sha256 = "0".repeat(64); },
];

for (const attack of attacks) {
  const mutatedProfile = structuredClone(profile);
  const mutatedSidecar = structuredClone(sidecar);
  attack(mutatedProfile, mutatedSidecar);
  assert.throws(
    () => validate(mutatedProfile, mutatedSidecar, binary, lineage, production, packet, capture),
    "mutated r31 identity or conformance binding must fail closed",
  );
}

console.log(JSON.stringify({
  ok: true,
  status: "m0_r31_runtime_profile_and_binding_contract_valid",
  executionProfile: { path: profilePath, sha256: sha256(profilePath) },
  bindingSidecar: { path: sidecarPath, sha256: sha256(sidecarPath) },
  gateB: {
    packet: sidecar.toolsetProof.path,
    packetSha256: sidecar.toolsetProof.sha256,
    capture: sidecar.toolsetProof.capture,
    captureResult: sidecar.toolsetProof.captureResult,
    modelVisibility: sidecar.toolsetProof.modelVisibility,
    proofCompleteness: sidecar.toolsetProof.proofCompleteness,
  },
  module: sidecar.runtimeTarget.module,
  orderedHakList: sidecar.runtimeTarget.orderedHakList,
  negativeCases: attacks.length,
  startsToolset: false,
  startsNwn: false,
}, null, 2));

function validate(p, s, b, l, c, gateB, captureResult) {
  assert.deepEqual(p, {
    version: "aur-s07-runtime-profile/v1",
    id: "m2a-m0-r31-hierarchy-only-runtime",
    module: {
      sha256: "8575465ef683256a54c101a3f4445a4e22803194c9b899a0e43e3fd069fdeafd",
      area: "m2a_m0a31",
      entryPoint: [10, 10, 0],
    },
    orderedHakList: [{
      resref: "m2a_m0r31",
      sha256: "ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12",
    }],
    geometryGate: {
      status: "verified",
      evidence: resolve(
        repoRoot,
        "proof-output/m0-r31-hierarchy-only-20260722/m0-r31-hierarchy-only-lineage-contract-v1.json",
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

  assert.equal(s.version, "m2a-aur-s07-runtime-v2-binding-sidecar/v1");
  assert.equal(s.id, "m2a-m0-r31-hierarchy-only-runtime-v2-binding");
  assert.deepEqual(s.executionProfile, {
    path: profilePath,
    sha256: sha256(profilePath),
    version: "aur-s07-runtime-profile/v1",
    publicRunner: "C:\\Projects\\aurora-web\\backend\\scripts\\aur-s07-runtime-execution.mjs",
  });

  for (const artifact of [
    s.executionProfile,
    s.sourceBinaryProfile,
    s.lineageContract,
    s.productionCandidateContract,
    s.toolsetProof,
    s.toolsetProof.profile,
    s.toolsetProof.capture,
    s.toolsetProof.captureResult,
    { path: s.runtimeTarget.module.sourcePath, sha256: s.runtimeTarget.module.sha256 },
    ...s.runtimeTarget.orderedHakList.map((hak) => ({ path: hak.sourcePath, sha256: hak.sha256 })),
    s.runtimeTarget.resources.model,
    s.runtimeTarget.resources.texture,
    s.runtimeTarget.resources.appearanceTwoDa,
    s.runtimeConformance.appearanceTable.source,
    s.runtimeConformance.appearanceTable.output,
  ]) {
    assert.equal(existsSync(artifact.path), true, `missing bound artifact: ${artifact.path}`);
    assert.equal(sha256(artifact.path), artifact.sha256, `hash mismatch: ${artifact.path}`);
  }

  assert.equal(gateB.version, "meshy2aurora-toolset-gate-b-proof/v1");
  assert.equal(gateB.status, "verified");
  assert.deepEqual(gateB.toolset, { modelVisibility: "visible", proofCompleteness: "verified" });
  assert.equal(gateB.profile.binaryPath, s.sourceBinaryProfile.path);
  assert.equal(gateB.profile.binarySha256, s.sourceBinaryProfile.sha256);
  assert.equal(gateB.profile.toolsetPath, s.toolsetProof.profile.path);
  assert.equal(gateB.profile.toolsetSha256, s.toolsetProof.profile.sha256);
  assert.equal(gateB.acceptedCapture.pngPath, s.toolsetProof.capture.path);
  assert.equal(gateB.acceptedCapture.pngSha256, s.toolsetProof.capture.sha256);
  assert.equal(gateB.acceptedCapture.captureMetadataPath, s.toolsetProof.captureResult.path);
  assert.equal(gateB.acceptedCapture.captureMetadataSha256, s.toolsetProof.captureResult.sha256);
  assert.equal(gateB.session.processId, s.toolsetProof.processId);
  assert.equal(gateB.selection.selectedHandle, s.toolsetProof.selectionHandle);
  assert.equal(s.toolsetProof.modelVisibility, gateB.toolset.modelVisibility);
  assert.equal(s.toolsetProof.proofCompleteness, gateB.toolset.proofCompleteness);
  assert.equal(captureResult.status, "verified");
  assert.equal(captureResult.pngPath, s.toolsetProof.capture.path);
  assert.equal(captureResult.moduleTitle.includes("m2a_m0r31.mod"), true);
  assert.equal(captureResult.areaViewerTitle.includes("m2a_m0a31"), true);

  assert.equal(b.module.resref, s.runtimeTarget.module.resref);
  assert.equal(b.module.path, s.runtimeTarget.module.sourcePath);
  assert.equal(b.module.sha256, s.runtimeTarget.module.sha256);
  assert.equal(gateB.lineage.moduleResref, s.runtimeTarget.module.resref);
  assert.equal(gateB.lineage.moduleSha256PreOpen, s.runtimeTarget.module.sha256);
  assert.equal(gateB.lineage.areaResref, s.runtimeTarget.area);
  assert.deepEqual(s.runtimeTarget.entryPoint, b.entryPoint.position);
  assert.deepEqual(p.module, {
    sha256: s.runtimeTarget.module.sha256,
    area: s.runtimeTarget.area,
    entryPoint: s.runtimeTarget.entryPoint,
  });
  assert.equal(s.runtimeTarget.orderedHakList.length, 1);
  assert.deepEqual(
    p.orderedHakList,
    s.runtimeTarget.orderedHakList.map(({ resref, sha256: hash }) => ({ resref, sha256: hash })),
  );
  assert.equal(gateB.lineage.orderedHakList.length, 1);
  assert.equal(gateB.lineage.orderedHakList[0].resref, s.runtimeTarget.orderedHakList[0].resref);
  assert.equal(gateB.lineage.orderedHakList[0].sha256, s.runtimeTarget.orderedHakList[0].sha256);

  const fixture = s.runtimeTarget.fixture;
  assert.equal(fixture.id, "m0_fixture");
  assert.equal(fixture.treeObjectText, "Meshy M0 binary vertical-slice fixture");
  assert.equal(fixture.occurrence, 0);
  assert.equal(fixture.selectionHandle, 105874560);
  assert.equal(fixture.modelResRef, "m2a_m0p01");
  assert.equal(fixture.templateResRef, "nw_dwarfmerc001");
  assert.equal(fixture.appearanceRow, 15100);
  assert.deepEqual(fixture.position, [10, 14.5, 0]);
  assert.deepEqual(fixture.orientation, [1, 0]);
  assert.equal(fixture.scale, 1);
  assert.equal(gateB.lineage.fixture.id, fixture.id);
  assert.equal(gateB.lineage.fixture.treeObjectText, fixture.treeObjectText);
  assert.equal(gateB.lineage.fixture.occurrence, fixture.occurrence);
  assert.equal(gateB.lineage.fixture.templateResRef, fixture.templateResRef);
  assert.equal(gateB.lineage.fixture.appearanceType, fixture.appearanceRow);
  assert.deepEqual(gateB.lineage.fixture.position, fixture.position);

  assert.deepEqual(s.runtimeTarget.resources.model, l.candidate.resources.model);
  assert.deepEqual(s.runtimeTarget.resources.texture, l.candidate.resources.texture);
  assert.deepEqual(s.runtimeTarget.resources.appearanceTwoDa, {
    resref: "appearance",
    ...l.candidate.resources.appearanceTwoDa,
  });
  assert.equal(c.profile, s.productionCandidateContract.profile);
  assert.equal(c.candidateAdmissible, s.productionCandidateContract.candidateAdmissible);
  assert.equal(c.structuralVerdict, s.productionCandidateContract.structuralVerdict);
  assert.equal(c.model.sha256, s.runtimeTarget.resources.model.sha256);
  assert.equal(c.texture.sha256, s.runtimeTarget.resources.texture.sha256);
  assert.equal(c.appearanceTwoDa.sha256, s.runtimeTarget.resources.appearanceTwoDa.sha256);

  const runtime = s.runtimeConformance;
  assert.equal(runtime.sourceTopology.profile, c.modelContract.derivedSourceBinding.topologySummary.profile);
  assert.equal(runtime.sourceTopology.derivationProfile, c.modelContract.derivedSourceBinding.profile);
  assert.equal(runtime.sourceTopology.originalSourceSha256, c.modelContract.derivedSourceBinding.originalSourceSha256);
  assert.equal(runtime.sourceTopology.derivedSourceSha256, c.modelContract.derivedSourceBinding.derivedSourceSha256);
  assert.equal(runtime.sourceTopology.derivedSourceBindingSha256, c.modelContract.derivedSourceBindingSha256);
  assert.equal(runtime.sourceTopology.topologySummarySha256, c.modelContract.derivedSourceBinding.topologySummarySha256);
  assert.equal(runtime.sourceTopology.outputTopologySha256, c.modelContract.outputTopologySha256);
  assert.equal(runtime.stateProjection.profile, "RETAIL_DIRECT_CREATURE_TYPE5_DUMMY_V1");
  assert.equal(runtime.stateProjection.provenance, null);
  assert.equal(runtime.stateProjection.summary.baseNodeCount, c.modelContract.candidateEngineEnvelope.baseNodeCount);
  assert.equal(runtime.stateProjection.summary.animationCount, c.modelContract.candidateEngineEnvelope.animations.length);
  assert.equal(runtime.stateProjection.summary.fullBaseTopologyProjectionCount, 7);
  assert.equal(runtime.stateProjection.summary.allGenericDummyProjectionCount, 7);
  assert.equal(runtime.stateProjection.summary.cepRigidPlaceholderProjectionCount, 0);
  assert.equal(runtime.stateProjection.summaryDigestSha256, c.modelContract.candidateStateProjectionSha256);
  assert.equal(runtime.engineEnvelope.profile, c.modelContract.candidateEngineEnvelope.profile);
  assert.equal(runtime.engineEnvelope.digestSha256, c.modelContract.candidateEngineEnvelopeSha256);
  assert.equal(runtime.engineEnvelope.rawMdxSha256, c.modelContract.candidateRawMdxSha256);
  assert.equal(runtime.engineEnvelope.protectedWriterFieldsSha256, c.modelContract.candidateProtectedWriterFieldsSha256);
  assert.equal(runtime.engineEnvelope.structuralVerdict, "STRUCTURAL_PASS_RUNTIME_NOT_WITNESSED");

  const table = runtime.appearanceTable;
  assert.equal(table.scope, "FULL_RUNTIME_APPEND_V1");
  assert.equal(table.source.physicalRows, 15100);
  assert.equal(table.output.physicalRows, 15101);
  assert.equal(table.appendCount, 1);
  assert.equal(table.appendedPhysicalRow, fixture.appearanceRow);
  assert.equal(table.sourcePrefixPreserved, true);
  assert.equal(table.source.sha256, c.appearanceTable.inputSha256);
  assert.equal(table.output.sha256, c.appearanceTable.outputSha256);
  assert.deepEqual(readFileSync(table.output.path).subarray(0, table.source.byteLength), readFileSync(table.source.path));

  assert.equal(s.engineLog.loadingModuleResref, s.runtimeTarget.module.resref);
  assert.deepEqual(s.launch, p.launch);
  assert.equal(s.noLocalSubstitute, true);
  assert.deepEqual(s.safety, {
    startsToolset: false,
    startsNwn: false,
    usesGlobalInput: false,
    savesModule: false,
    modifiesEarlierLineages: false,
    implementsLocalRunner: false,
  });
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}
