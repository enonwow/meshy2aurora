#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const lineagePath = resolve(repoRoot, "proof-output/m0-r32-corrected-container-20260722/m0-r32-corrected-container-lineage-contract-v1.json");
const binaryPath = resolve(repoRoot, "proof-profiles/m0-r32-corrected-container-binary-bootstrap-v1.json");
const toolsetPath = resolve(repoRoot, "proof-profiles/m0-r32-corrected-container-toolset-proof-v1.json");
const runtimePath = resolve(repoRoot, "proof-profiles/m0-r32-corrected-container-runtime-v1.json");
const modelProofInitialPath = resolve(repoRoot, "proof-profiles/m0-r32-corrected-container-model-proof-120s-v1.json");
const modelProofRouteFixPath = resolve(repoRoot, "proof-profiles/m0-r32-corrected-container-model-proof-120s-route-fix-v1.json");
const toolsetVerdictPath = resolve(repoRoot, "proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-1/toolset/verdict.json");
const toolsetCapturePath = resolve(repoRoot, "proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-1/toolset/capture.png");
const runtimeBlockerPath = resolve(repoRoot, "proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-1/blocker.json");
const installedModulePath = "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\modules\\m2a_m0r32.mod";
const generatedR32HakPath = resolve(repoRoot, "proof-output/m0-r32-corrected-container-20260722/generated/m2a_m0r32.hak");
const installedR32HakPath = "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m0r32.hak";

const lineage = readJson(lineagePath);
const binary = readJson(binaryPath);
const toolset = readJson(toolsetPath);
const runtime = readJson(runtimePath);

validate(lineage, binary, toolset, runtime, true);
for (const [index, mutate] of [
  value => value.correctedModule.sha256 = "0".repeat(64),
  value => value.orderedHakList[0].sha256 = "0".repeat(64),
  value => value.orderedHakList.push(structuredClone(value.orderedHakList[0])),
  value => value.fixture.appearanceRow = 848,
  value => value.fixture.position = [10, 15, 0],
  value => value.intendedDelta.modelDelta = true,
  value => value.materialization.materializationCount = 2,
  value => value.materialization.runtimeProfileMaterialized = false,
  value => value.proofState.toolset.modelVisibility = "not_tested",
  value => value.profiles.runtime.sha256 = "0".repeat(64),
  value => value.proofEvidence.toolsetCapture.sha256 = "0".repeat(64),
  value => value.proofState.nwn.modelVisibility = "visible",
].entries()) {
  const changed = structuredClone(lineage);
  mutate(changed);
  assert.throws(() => validate(changed, binary, toolset, runtime, false), `negative ${index}`);
}

console.log(JSON.stringify({
  ok: true,
  status: "m0_r32_corrected_container_lineage_contract_valid",
  lineage: { path: lineagePath, sha256: sha256(lineagePath) },
  module: { path: lineage.correctedModule.path, sha256: lineage.correctedModule.sha256 },
  installedModule: { path: installedModulePath, sha256: sha256(installedModulePath) },
  orderedHakList: lineage.orderedHakList,
  profiles: {
    binary: { path: binaryPath, sha256: sha256(binaryPath) },
    toolset: { path: toolsetPath, sha256: sha256(toolsetPath) },
    runtime: { path: runtimePath, sha256: sha256(runtimePath) },
    modelProofInitial: { path: modelProofInitialPath, sha256: sha256(modelProofInitialPath) },
    modelProofRouteFix: { path: modelProofRouteFixPath, sha256: sha256(modelProofRouteFixPath) },
    runtimeProfileMaterialized: true,
  },
  proofState: lineage.proofState,
  negativeCases: 12,
  startsToolset: false,
  startsNwn: false,
}, null, 2));

function validate(value, binaryProfile, toolsetProfile, runtimeProfile, verifyFiles) {
  assert.equal(value.version, "m2a-m0-r32-corrected-container-lineage-contract/v1");
  assert.equal(value.id, "m2a-m0-r32-corrected-container");
  assert.equal(value.status, "toolset_visible_runtime_pending");
  assert.equal(value.sourceR31Module.sha256, "8575465ef683256a54c101a3f4445a4e22803194c9b899a0e43e3fd069fdeafd");
  assert.equal(value.sourceR31Module.byteLength, 13173);
  assert.equal(value.correctedModule.sha256, "a952e05746710c8461e74b38dbf93e6cc5c86314771330d68724579c11693573");
  assert.equal(value.correctedModule.byteLength, 14743);

  assert.equal(value.orderedHakList.length, 1);
  assert.deepEqual(value.orderedHakList[0], {
    resref: "m2a_m0r31",
    sourcePath: resolve(repoRoot, "proof-output/m0-r31-hierarchy-only-20260722/generated/m2a_m0r31.hak"),
    installedPath: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m0r31.hak",
    byteLength: 19635900,
    sha256: "ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12",
    reusedByteIdentical: true,
    copiedOrRebuilt: false,
  });
  assert.deepEqual(value.area, { resref: "m2a_m0a32", width: 2, height: 2, entryPoint: [10, 10, 0] });
  assert.deepEqual(value.fixture, {
    id: "m0_fixture",
    templateResref: "nw_dwarfmerc001",
    treeObjectText: "Meshy M0 binary vertical-slice fixture",
    appearanceRow: 15100,
    position: [10, 14.5, 0],
    orientation: [1, 0],
  });
  assert.deepEqual(value.intendedDelta.only, ["MOD_AREA_IDENTITY", "RUNTIME_COMPLETE_CREATURE_ENVELOPE"]);
  assert.equal(value.intendedDelta.modelDelta, false);
  assert.equal(value.intendedDelta.textureDelta, false);
  assert.equal(value.intendedDelta.appearanceTwoDaDelta, false);
  assert.equal(value.intendedDelta.hakDelta, false);
  assert.match(value.intendedDelta.historicalR31SparseRejection, /MaxHitPoints requires 13/);
  assert.equal(value.intendedDelta.correctedContainerReadback, "independent generic parser PASS");
  assert.deepEqual(value.materialization, {
    materializationCount: 1,
    startsToolset: false,
    startsNwn: false,
    runtimeProfileMaterialized: true,
    generatedHak: false,
  });
  assert.deepEqual(value.profiles, {
    runtime: {
      path: runtimePath,
      sha256: "9c1ab9b47bf88f77eb122ef1301ed74c3aab2a5da9e66e7dab6dfb72a77ecda4",
    },
    modelProofInitial: {
      path: modelProofInitialPath,
      sha256: "59d342ad5dbfef5b5fea6c002512b85b3bf7d1434908efa997365691b3e349e2",
    },
    modelProofRouteFix: {
      path: modelProofRouteFixPath,
      sha256: "2a60ed1f4905c40d6b50780e725edd0b71e01d448029f77efc6d36a38100f90b",
    },
  });
  assert.deepEqual(value.proofEvidence, {
    toolsetCapture: {
      path: toolsetCapturePath,
      sha256: "77f1d3fa3324cfd8e3c2c14275e57bec6f15090a7a35d5626750673ad8f1a41d",
    },
    toolsetVerdict: {
      path: toolsetVerdictPath,
      sha256: "ef6a3d5cf0eeda4323f3ea5ab2c1cfb1054b5bf0c3267cab38f86bf03516b62f",
    },
    runtimeBlocker: {
      path: runtimeBlockerPath,
      sha256: "42ff4967fb99735c94f9345041bbe0ca688edaa8d4c1d2d8de35589c02f1cfd2",
    },
  });
  assert.deepEqual(value.proofState.toolset, { modelVisibility: "visible", proofCompleteness: "verified" });
  assert.deepEqual(value.proofState.nwn, { modelVisibility: "not_tested", proofCompleteness: "missing" });

  assert.deepEqual(binaryProfile, {
    version: "aurora-toolset-binary-module-bootstrap-profile/v1",
    id: "m2a-m0-r32-corrected-container-binary-bootstrap-v1",
    module: { resref: "m2a_m0r32", path: value.correctedModule.path, sha256: value.correctedModule.sha256 },
    area: { resref: "m2a_m0a32", width: 2, height: 2 },
    entryPoint: { area: "m2a_m0a32", position: [10, 10, 0] },
    orderedHakList: [{ resref: "m2a_m0r31", path: value.orderedHakList[0].installedPath, sha256: value.orderedHakList[0].sha256 }],
    fixtures: [{ id: "m0_fixture", templateResRef: "nw_dwarfmerc001", appearanceType: 15100, position: [10, 14.5, 0] }],
    noLocalToolsetAdapter: true,
  });
  assert.deepEqual(toolsetProfile, {
    version: "aurora-toolset-binary-module-toolset-proof-profile/v1",
    id: "m2a-m0-r32-corrected-container-toolset-proof-v1",
    binaryProfile: { path: binaryPath, sha256: sha256(binaryPath) },
    fixtureSelection: { fixtureId: "m0_fixture", treeObjectText: "Meshy M0 binary vertical-slice fixture", occurrence: 0 },
    capture: { targetClass: "TScrollBox", allowUniformScene: false },
    noSave: true,
    noLocalToolsetAdapter: true,
  });
  assert.deepEqual(runtimeProfile, {
    version: "aur-s07-runtime-profile/v1",
    id: "m2a-m0-r32-corrected-container-runtime",
    module: {
      sha256: value.correctedModule.sha256,
      area: value.area.resref,
      entryPoint: value.area.entryPoint,
    },
    orderedHakList: [{ resref: value.orderedHakList[0].resref, sha256: value.orderedHakList[0].sha256 }],
    geometryGate: { status: "verified", evidence: lineagePath },
    fixtures: [{
      id: value.fixture.id,
      resref: "m2a_m0p01",
      templateResRef: "nw_dwarfmerc001",
      appearanceRow: value.fixture.appearanceRow,
      position: value.fixture.position,
    }],
    launch: { mode: "user_owned_test_module_action", processName: "nwmain" },
    noLocalSubstitute: true,
  });

  if (!verifyFiles) return;
  for (const artifact of [
    value.sourceR31Module,
    value.correctedModule,
    { path: installedModulePath, sha256: value.correctedModule.sha256 },
    { path: value.orderedHakList[0].sourcePath, sha256: value.orderedHakList[0].sha256 },
    { path: value.orderedHakList[0].installedPath, sha256: value.orderedHakList[0].sha256 },
    { path: binaryPath, sha256: "912de1684ef274e6914620ffc4995b0cc22e50fd7066bc09e781ad8aa13920f8" },
    { path: toolsetPath, sha256: "1fe6fabfce4843f56b994ceec684112db09acbf540c19c3810afb4ec0df313a5" },
    value.profiles.runtime,
    value.profiles.modelProofInitial,
    value.profiles.modelProofRouteFix,
    value.proofEvidence.toolsetCapture,
    value.proofEvidence.toolsetVerdict,
    value.proofEvidence.runtimeBlocker,
  ]) {
    assert.equal(existsSync(artifact.path), true, `missing ${artifact.path}`);
    assert.equal(sha256(artifact.path), artifact.sha256, `hash mismatch ${artifact.path}`);
  }
  assert.deepEqual(readFileSync(value.correctedModule.path), readFileSync(installedModulePath));
  assert.deepEqual(readFileSync(value.orderedHakList[0].sourcePath), readFileSync(value.orderedHakList[0].installedPath));
  assert.equal(existsSync(generatedR32HakPath), false);
  assert.equal(existsSync(installedR32HakPath), false);
  assert.equal(existsSync(runtimePath), true);
}

function readJson(path) { return JSON.parse(readFileSync(path, "utf8")); }
function sha256(path) { return createHash("sha256").update(readFileSync(path)).digest("hex"); }
