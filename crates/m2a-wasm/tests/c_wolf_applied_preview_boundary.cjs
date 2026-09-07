"use strict";

const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const {
  closeSync,
  openSync,
  readFileSync,
  readSync,
} = require("node:fs");
const path = require("node:path");

const packagePath = process.argv[2];
const keyPath = process.env.M2A_REFERENCE_NWN_KEY;
const sourcePath = process.env.M2A_CWOLF_PREVIEW_SOURCE_GLB;
if (!packagePath || !keyPath || !sourcePath) {
  throw new Error(
    "usage: set M2A_REFERENCE_NWN_KEY and M2A_CWOLF_PREVIEW_SOURCE_GLB, then run node c_wolf_applied_preview_boundary.cjs <generated-node-package>",
  );
}

const wasm = require(path.resolve(packagePath));
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const keyBytes = readFileSync(keyPath);
const keyIndex = JSON.parse(wasm.indexNwnKeyModelsV1Json(keyBytes));
const locator = keyIndex.models.find((model) => model.resref.toLowerCase() === "c_wolf");
assert.ok(locator, "exact c_wolf locator must exist in the selected KEY");
const bif = keyIndex.bifs.find((item) => item.index === locator.bifIndex);
assert.ok(bif, "c_wolf BIF must resolve from the selected KEY");
const relativeBif = bif.logicalName.replace(/[\\/]/g, path.sep);
const keyParent = path.dirname(keyPath);
const candidates = [
  path.join(path.dirname(keyParent), relativeBif),
  path.join(keyParent, relativeBif),
];
const bifPath = candidates.find((candidate) => {
  try {
    const handle = openSync(candidate, "r");
    closeSync(handle);
    return true;
  } catch {
    return false;
  }
});
assert.ok(bifPath, `exact BIF must exist: ${bif.logicalName}`);

const handle = openSync(bifPath, "r");
let referenceMdl;
try {
  const header = Buffer.alloc(20);
  assert.equal(readSync(handle, header, 0, header.length, 0), header.length);
  const plan = JSON.parse(wasm.planNwnBifIndexV1Json(header));
  const table = Buffer.alloc(plan.tableByteLength);
  assert.equal(
    readSync(handle, table, 0, table.length, plan.tableOffset),
    table.length,
  );
  const index = JSON.parse(wasm.indexNwnBifTableV1Json(header, table));
  const resource = index.resources.find(
    (item) => item.resourceIndex === locator.resourceIndex,
  );
  assert.ok(resource, "exact c_wolf resource must resolve from the BIF table");
  assert.equal(resource.resourceType, 2002);
  referenceMdl = Buffer.alloc(resource.payloadSize);
  assert.equal(
    readSync(handle, referenceMdl, 0, referenceMdl.length, resource.payloadOffset),
    referenceMdl.length,
  );
} finally {
  closeSync(handle);
}

const sourceGlb = readFileSync(sourcePath);
const referenceSha256 = sha256(referenceMdl);
assert.equal(referenceMdl.length, 340_812);
assert.equal(
  referenceSha256,
  "a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726",
);
assert.equal(sourceGlb.length, 6_847_588);
assert.equal(
  sha256(sourceGlb),
  "44b5d3c63387ae7587b4de8dc9e4a6866948cd034cd1a1e3a0c0ce69af3e6678",
);

const referenceChainJson = JSON.stringify([{
  resref: "c_wolf",
  supermodelResref: "NULL",
  format: "BINARY",
  sha256: referenceSha256,
  byteOffset: 0,
  byteLength: referenceMdl.length,
}]);
const result = wasm.buildReferenceSupermodelAppliedPreviewV2(
  "c_wolf",
  sourceGlb,
  referenceMdl,
  referenceChainJson,
  "POSITIVE_Z",
);
try {
  const report = JSON.parse(result.applyReportJson);
  const readback = JSON.parse(result.readbackJson);
  const modelBytes = result.takeModelBytes();
  assert.equal(readback.model.supermodelName, "c_wolf");
  assert.equal(readback.animations.length, 0);
  assert.equal(report.inheritedAnimationCount, 42);
  assert.equal(report.referenceFormat, "BINARY");
  assert.equal(report.retailPayloadCopied, false);
  assert.deepEqual(report.exactChain.map((resource) => ({
    resref: resource.resref,
    supermodelResref: resource.supermodelResref,
    format: resource.format,
    sha256: resource.sha256,
    byteLength: resource.byteLength,
  })), [{
    resref: "c_wolf",
    supermodelResref: "NULL",
    format: "BINARY",
    sha256: referenceSha256,
    byteLength: 340_812,
  }]);
  assert.equal(report.status, "APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY");
  assert.equal(report.motionCompatible, false);
  assert.equal(report.bindPoseCompatible, true);
  assert.equal(report.skinBindCompatible, true);
  assert.equal(report.motionQualityStatus, "BLOCKED");
  assert.equal(report.motionQuality.surfaceSeamGate.failOnAnySeamViolation, true);
  assert.equal(report.motionQuality.surfaceSeamGate.status, "BLOCKED_VISIBLE_SEAM");
  for (const clipName of ["cpause1", "cwalk", "crun"]) {
    const clip = report.motionQuality.clips.find((item) => item.clipName === clipName);
    assert.ok(clip, `quality report must include ${clipName}`);
    assert.equal(clip.visibleAnchorTrajectories.length, 2);
  }
  assert.ok(report.motionQuality.seamPairViolationCount > 0);
  assert.ok(report.motionQuality.visibleAnchorMotionViolationCount > 0);
  assert.equal(sha256(modelBytes), report.modelSha256);
  process.stdout.write(`${JSON.stringify({
    api: "buildReferenceSupermodelAppliedPreviewV2",
    status: report.status,
    supermodelResref: report.supermodelResref,
    inheritedAnimationCount: report.inheritedAnimationCount,
    bindPoseCompatible: report.bindPoseCompatible,
    skinBindCompatible: report.skinBindCompatible,
    motionCompatible: report.motionCompatible,
    motionQualityStatus: report.motionQualityStatus,
    modelSha256: report.modelSha256,
    modelByteLength: modelBytes.byteLength,
  })}\n`);
} finally {
  result.free();
}

assert.throws(
  () => wasm.buildReferenceSupermodelCreatureProductV2(
    "c_wolf",
    sourceGlb,
    Buffer.from("2DA V2.0\n\nLABEL MODELTYPE RACE\n0 dummy S c_dog\n"),
    referenceMdl,
    referenceChainJson,
    JSON.stringify({
      modelResref: "m2acwprod",
      textureResref: "m2acwtex",
      materialResref: "m2acwmtr",
      hakResref: "m2acwhak",
      appearanceLabel: "M2A_CWOLF_PRODUCT",
      appearanceDonorResrefs: ["c_dog"],
      rejectedBaseline: {
        modelSha256: "a2749e97a35dd6c41d9dd0cefbb3c20301927d453271b6d90c10ece44bcdfa5f",
        visibleSurfaceSemanticSha256: "ff75e44d8c903e68fdded68061c594013374cd4255b9b31b356a9c77e64ab15a",
      },
      semanticControllerNames: ["Wolf_tail", "Wolf_tailend"],
    }),
    "POSITIVE_Z",
  ),
  /MOTION-QUALITY-BLOCKED|BLOCKED_SEMANTIC_DELTA_MISSING/,
  "the current exact c_wolf candidate must fail closed before product packaging",
);
