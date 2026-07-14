"use strict";

const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { createHash } = require("node:crypto");
const { mkdtempSync, readFileSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");

const packagePath = process.argv[2];
if (!packagePath) {
  throw new Error("usage: node node_boundary.cjs <generated-node-package>");
}
const wasm = require(path.resolve(packagePath));

const sha256 = (bytes) =>
  createHash("sha256").update(Buffer.from(bytes)).digest("hex");

const withoutRigAndAnimations = (glb) => {
  const input = Buffer.from(glb);
  const jsonLength = input.readUInt32LE(12);
  const jsonEnd = 20 + jsonLength;
  const root = JSON.parse(input.subarray(20, jsonEnd).toString("utf8"));
  delete root.skins;
  delete root.animations;
  for (const node of root.nodes) delete node.skin;
  const json = Buffer.from(JSON.stringify(root), "utf8");
  const paddedJson = Buffer.alloc((json.length + 3) & ~3, 0x20);
  json.copy(paddedJson);
  const result = Buffer.concat([
    Buffer.from("glTF", "ascii"),
    Buffer.from([2, 0, 0, 0]),
    Buffer.alloc(4),
    Buffer.from([
      paddedJson.length & 0xff,
      (paddedJson.length >>> 8) & 0xff,
      (paddedJson.length >>> 16) & 0xff,
      (paddedJson.length >>> 24) & 0xff,
    ]),
    Buffer.from("JSON", "ascii"),
    paddedJson,
    input.subarray(jsonEnd),
  ]);
  result.writeUInt32LE(result.length, 8);
  return result;
};

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
const M7_READY_BATCH_JSON_SHA256 =
  "ee04ebfcdbb3e1265913de8f88d3c05f9277d18c7d0c75bdbcecc8139046c808";

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

const m7Manifest = JSON.stringify({
  schemaVersion: 1,
  corpusId: "browser_corpus",
  artDirectionApprovalId: null,
  samples: [
    {
      role: "RIGGED_HUMANOID_SOURCE_CLIPS",
      sampleId: "humanoid",
      source: null,
      requiredSourceClipNames: ["walk"],
    },
    {
      role: "NON_HUMANOID_REFERENCE_SUPERMODEL",
      sampleId: "creature",
      source: null,
      referenceSupermodel: "c_dog",
    },
    {
      role: "STATIC_PLACEABLE_OR_ITEM",
      sampleId: "placeable",
      source: null,
      resourceKind: "PLACEABLE",
    },
  ],
});
const emptyM7Descriptors = JSON.stringify({ schemaVersion: 1, payloads: [] });
const deferredM7Blob = new Uint8Array();
const deferredM7Before = deferredM7Blob.slice();
const validatedM7 = JSON.parse(
  wasm.validateM7CorpusManifestV1Json(m7Manifest),
);
assert.equal(validatedM7.corpusId, "browser_corpus");
const m7Intake = JSON.parse(
  wasm.inspectM7CorpusIntakeV1Json(
    m7Manifest,
    deferredM7Blob,
    emptyM7Descriptors,
  ),
);
assert.equal(m7Intake.status, "INPUT_DEFERRED");
assert.equal(m7Intake.requiredSourceCount, 3);
const m7BatchJson = wasm.buildM7CorpusBatchV1(
  m7Manifest,
  deferredM7Blob,
  emptyM7Descriptors,
);
const m7Batch = JSON.parse(m7BatchJson);
assert.equal(m7Batch.report.packetCount, 3);
assert.equal(m7Batch.packets.length, 3);
assert.deepEqual(
  m7Batch.packets.map((packet) => packet.sampleId),
  ["humanoid", "creature", "placeable"],
);
assert.equal(m7Batch.report.m7DoneClaimAllowed, false);
assert.ok(!JSON.stringify(m7Batch).toLowerCase().includes("base64"));
assert.equal(
  wasm.buildM7CorpusBatchV1(
    m7Manifest,
    deferredM7Blob,
    emptyM7Descriptors,
  ),
  m7BatchJson,
);
assert.deepEqual(deferredM7Blob, deferredM7Before);

for (const [invalidManifest, code] of [
  ["{", "M7-MANIFEST-JSON-INVALID"],
  [
    JSON.stringify({
      schemaVersion: 2,
      corpusId: "bad_version",
      artDirectionApprovalId: null,
      samples: [],
    }),
    "M7-MANIFEST-SCHEMA-UNSUPPORTED",
  ],
]) {
  assert.equal(
    JSON.parse(wasm.validateM7CorpusManifestV1Json(invalidManifest)).code,
    code,
  );
}

const malformedM7Descriptors = JSON.stringify({
  schemaVersion: 1,
  payloads: [
    {
      role: "SOURCE",
      relativePath: "source.glb",
      payloadOffset: 0,
      payloadSize: 2,
    },
  ],
});
const malformedM7Blob = new Uint8Array([1]);
const malformedM7Before = malformedM7Blob.slice();
try {
  wasm.buildM7CorpusBatchV1(
    m7Manifest,
    malformedM7Blob,
    malformedM7Descriptors,
  );
  assert.fail("out-of-bounds M7 descriptor must throw");
} catch (error) {
  assert.equal(typeof error, "string");
  assert.equal(JSON.parse(error).code, "M7-WASM-PAYLOAD-RANGE-OOB");
}
assert.deepEqual(malformedM7Blob, malformedM7Before);

const wasm32OverflowDescriptors = JSON.stringify({
  schemaVersion: 1,
  payloads: [
    {
      role: "SOURCE",
      relativePath: "source.glb",
      payloadOffset: 0xffffffff,
      payloadSize: 1,
    },
  ],
});
try {
  wasm.buildM7CorpusBatchV1(
    m7Manifest,
    new Uint8Array([1]),
    wasm32OverflowDescriptors,
  );
  assert.fail("wasm32 payload range overflow must throw");
} catch (error) {
  assert.equal(typeof error, "string");
  assert.equal(JSON.parse(error).code, "M7-WASM-PAYLOAD-RANGE-OVERFLOW");
}

const repoRoot = path.resolve(__dirname, "../../..");
const readyFixtureDirectory = mkdtempSync(
  path.join(tmpdir(), "m2a-wasm-m7-ready-"),
);
try {
  const generatedOutputDirectory = path.join(
    readyFixtureDirectory,
    "owned-package",
  );
  const appearancePath = path.join(
    repoRoot,
    "apps/studio-web/tests/fixtures/appearance.2da",
  );
  const cargo = process.platform === "win32" ? "cargo.exe" : "cargo";
  const generated = spawnSync(
    cargo,
    [
      "run",
      "--quiet",
      "--manifest-path",
      path.join(repoRoot, "Cargo.toml"),
      "-p",
      "m2a-core",
      "--example",
      "materialize_m6",
      "--",
      "--synthetic-owned",
      "--appearance-2da",
      appearancePath,
      "--output-dir",
      generatedOutputDirectory,
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  if (generated.status !== 0) {
    process.stderr.write(generated.stderr);
    throw new Error(`M7 owned fixture generator failed: ${generated.status}`);
  }

  const humanoid = readFileSync(
    path.join(generatedOutputDirectory, "generated/source-owned.glb"),
  );
  const staticGlb = withoutRigAndAnimations(humanoid);
  const appearance = readFileSync(appearancePath);
  const sourceIdentity = (bytes) => ({
    byteLength: bytes.length,
    sha256: sha256(bytes),
  });
  const source = (relativePath, bytes, providerTaskId) => ({
    relativePath,
    identity: sourceIdentity(bytes),
    provenance: {
      provider: "MESHY",
      providerTaskId,
      originalExportAttested: true,
      rightsConfirmed: true,
      notSyntheticFixtureAttested: true,
    },
  });
  const readyManifest = JSON.stringify({
    schemaVersion: 1,
    corpusId: "m7-wasm-ready-fixture",
    artDirectionApprovalId: "owned-test-approval",
    samples: [
      {
        role: "RIGGED_HUMANOID_SOURCE_CLIPS",
        sampleId: "humanoid",
        source: source("models/humanoid.glb", humanoid, "task-h"),
        requiredSourceClipNames: ["owned-linear-pause"],
      },
      {
        role: "NON_HUMANOID_REFERENCE_SUPERMODEL",
        sampleId: "creature",
        source: source("models/creature.glb", staticGlb, "task-c"),
        referenceSupermodel: "c_horror",
      },
      {
        role: "STATIC_PLACEABLE_OR_ITEM",
        sampleId: "static-prop",
        source: source("models/static.glb", staticGlb, "task-s"),
        resourceKind: "PLACEABLE",
      },
    ],
  });
  const chunks = [humanoid, staticGlb, staticGlb, appearance];
  const readyBlob = new Uint8Array(Buffer.concat(chunks));
  const readyBefore = readyBlob.slice();
  let offset = 0;
  const readyPayloads = [
    ["SOURCE", "models/humanoid.glb", humanoid],
    ["SOURCE", "models/creature.glb", staticGlb],
    ["SOURCE", "models/static.glb", staticGlb],
  ].map(([role, relativePath, bytes]) => {
    const descriptor = {
      role,
      relativePath,
      payloadOffset: offset,
      payloadSize: bytes.length,
    };
    offset += bytes.length;
    return descriptor;
  });
  readyPayloads.push({
    role: "RIGGED_HUMANOID_APPEARANCE_2DA",
    sampleId: "humanoid",
    payloadOffset: offset,
    payloadSize: appearance.length,
  });
  const readyDescriptors = JSON.stringify({
    schemaVersion: 1,
    payloads: readyPayloads,
  });

  const readyIntake = JSON.parse(
    wasm.inspectM7CorpusIntakeV1Json(
      readyManifest,
      readyBlob,
      readyDescriptors,
    ),
  );
  assert.equal(readyIntake.status, "READY_FOR_M7_V5");
  const firstReadyJson = wasm.buildM7CorpusBatchV1(
    readyManifest,
    readyBlob,
    readyDescriptors,
  );
  const secondReadyJson = wasm.buildM7CorpusBatchV1(
    readyManifest,
    readyBlob,
    readyDescriptors,
  );
  assert.equal(firstReadyJson, secondReadyJson);
  const readyBatch = JSON.parse(firstReadyJson);
  assert.equal(
    sha256(Buffer.from(firstReadyJson)),
    M7_READY_BATCH_JSON_SHA256,
  );
  assert.equal(readyBatch.report.materializedPacketCount, 1);
  assert.equal(readyBatch.report.deferredPacketCount, 2);
  assert.deepEqual(
    readyBatch.packets.map(({ sampleId, status }) => [sampleId, status]),
    [
      ["humanoid", "CANONICAL_PACKAGE_MATERIALIZED"],
      ["creature", "INPUT_DEFERRED"],
      ["static-prop", "INPUT_DEFERRED"],
    ],
  );
  assert.deepEqual(
    readyBatch.report.packetIdentities.map(({ sampleId }) => sampleId),
    ["humanoid", "creature", "static-prop"],
  );
  for (const identity of readyBatch.report.packetIdentities) {
    assert.match(identity.identity.sha256, /^[0-9a-f]{64}$/);
    assert.ok(identity.identity.byteLength > 0);
  }
  assert.ok(!firstReadyJson.toLowerCase().includes("base64"));
  assert.deepEqual(readyBlob, readyBefore);
} finally {
  rmSync(readyFixtureDirectory, { force: true, recursive: true });
}

console.log("M5/M7 Node boundary PASS");
