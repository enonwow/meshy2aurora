"use strict";

const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const path = require("node:path");

const packagePath = process.argv[2];
if (!packagePath) {
  throw new Error("usage: node node_boundary.cjs <generated-node-package>");
}
const wasm = require(path.resolve(packagePath));

const sha256 = (bytes) =>
  createHash("sha256").update(Buffer.from(bytes)).digest("hex");

// Frozen by crates/m2a-core/tests/tga_writer.rs.
const TGA_BYTE_LENGTH = 60;
const TGA_SHA256 =
  "ab5365a31f1ef4d57b33943ae01735a33e5337d4d0d6b9eba5b715a3fb360c79";
// Frozen by crates/m2a-core/tests/two_da.rs.
const TWO_DA_BYTE_LENGTH = 36;
const TWO_DA_SHA256 =
  "fed4b73584a864c1a5532b1dfea78f07a603116fa59ef4b6f5f70b84fc96cb67";
// Frozen by crates/m2a-core/tests/package_manifest.rs and the native WASM parity test.
const HAK_BYTE_LENGTH = 265;
const HAK_SHA256 =
  "494862f6a12f91d5a269519d0579a05ace5bb50fd8f72b5711fcae7445444477";

const tgaImage = JSON.stringify({
  schemaVersion: 1,
  width: 2,
  height: 2,
  pixelFormat: "RGBA8",
  pixels: [
    255, 0, 0, 1, 0, 255, 0, 2, 0, 0, 255, 3, 255, 255, 255, 4,
  ],
});
const tgaOptions = JSON.stringify({
  schemaVersion: 1,
  limits: { maxOutputBytes: 67108864 },
});
const tga = wasm.writeTgaV1(tgaImage, tgaOptions);
assert.ok(tga instanceof Uint8Array);
const tgaReport = wasm.writeTgaV1ReportJson(tgaImage, tgaOptions);
assert.equal(typeof tgaReport, "string");
assert.equal(JSON.parse(tgaReport).byteLength, tga.byteLength);
assert.equal(tga.byteLength, TGA_BYTE_LENGTH);
assert.equal(sha256(tga), TGA_SHA256);
assert.equal(JSON.parse(tgaReport).outputSha256, TGA_SHA256);

const twoDaSource = new Uint8Array(Buffer.from("2DA V2.0\n\nA B\n0 old ****\n", "ascii"));
const twoDaBefore = twoDaSource.slice();
const twoDaLimits = JSON.stringify({
  maxInputBytes: 16777216,
  maxColumns: 4096,
  maxRows: 65536,
  maxTokenBytes: 1048576,
  maxDiagnostics: 2048,
});
const twoDaRequest = JSON.stringify({
  schemaVersion: 1,
  cells: [{ columnName: "A", value: { kind: "TEXT", value: "new" } }],
});
const inspection = wasm.inspectTwoDaV2Json(twoDaSource, twoDaLimits);
assert.equal(typeof inspection, "string");
const appended = wasm.appendTwoDaRowV1(twoDaSource, twoDaRequest, twoDaLimits);
assert.ok(appended instanceof Uint8Array);
const appendReport = wasm.appendTwoDaRowV1ReportJson(
  twoDaSource,
  twoDaRequest,
  twoDaLimits,
);
assert.equal(typeof appendReport, "string");
assert.equal(appended.byteLength, TWO_DA_BYTE_LENGTH);
assert.equal(sha256(appended), TWO_DA_SHA256);
assert.equal(JSON.parse(appendReport).outputSha256, TWO_DA_SHA256);
assert.deepEqual(twoDaSource, twoDaBefore);
try {
  wasm.appendTwoDaRowV1(twoDaSource, "{", twoDaLimits);
  assert.fail("malformed request must throw");
} catch (error) {
  assert.equal(typeof error, "string");
  assert.equal(JSON.parse(error).code, "M5-2DA-REQUEST-JSON-INVALID");
}
assert.deepEqual(twoDaSource, twoDaBefore);

const hakBlob = new Uint8Array(Buffer.from("tga2damdl", "ascii"));
const hakBefore = hakBlob.slice();
const hakResources = JSON.stringify({
  schemaVersion: 1,
  resources: [
    { resref: "texture", resourceType: 3, payloadOffset: 0, payloadSize: 3 },
    {
      resref: "appearance",
      resourceType: 2017,
      payloadOffset: 3,
      payloadSize: 3,
    },
    { resref: "model", resourceType: 2002, payloadOffset: 6, payloadSize: 3 },
  ],
});
const hakOptions = JSON.stringify({
  schemaVersion: 1,
  limits: { maxEntryCount: 262144, maxOutputBytes: 268435456 },
});
const hak = wasm.writeHakV1(hakBlob, hakResources, hakOptions);
assert.ok(hak instanceof Uint8Array);
const hakReport = wasm.writeHakV1ReportJson(hakBlob, hakResources, hakOptions);
const manifest = wasm.writePackageManifestV1Json(hakBlob, hakResources, hakOptions);
const modelPackage = wasm.writeModelPackageV1(hakBlob, hakResources, hakOptions);
const modelPackageHak = modelPackage.takeHakBytes();
assert.equal(typeof hakReport, "string");
assert.equal(typeof manifest, "string");
assert.ok(modelPackageHak instanceof Uint8Array);
assert.equal(modelPackage.takeHakBytes().byteLength, 0);
assert.equal(modelPackage.reportJson, hakReport);
assert.equal(modelPackage.manifestJson, manifest);
assert.equal(sha256(modelPackageHak), HAK_SHA256);
assert.equal(JSON.parse(hakReport).archiveSha256, JSON.parse(manifest).packageSha256);
assert.equal(hak.byteLength, HAK_BYTE_LENGTH);
assert.equal(sha256(hak), HAK_SHA256);
assert.equal(JSON.parse(hakReport).archiveSha256, HAK_SHA256);
assert.deepEqual(hakBlob, hakBefore);
try {
  wasm.writeHakV1(hakBlob, "{", hakOptions);
  assert.fail("malformed resources must throw");
} catch (error) {
  assert.equal(typeof error, "string");
  assert.equal(JSON.parse(error).code, "M5-HAK-RESOURCES-JSON-INVALID");
}
assert.deepEqual(hakBlob, hakBefore);

for (const value of [
  tgaImage,
  tgaOptions,
  tgaReport,
  twoDaLimits,
  twoDaRequest,
  inspection,
  appendReport,
  hakResources,
  hakOptions,
  hakReport,
  manifest,
  modelPackage.reportJson,
  modelPackage.manifestJson,
]) {
  assert.equal(typeof value, "string");
  assert.ok(!value.toLowerCase().includes("base64"));
}

console.log("M5 Node boundary PASS");
