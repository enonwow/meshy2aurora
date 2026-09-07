import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";

const JSON_CHUNK = 0x4e4f534a;
const BIN_CHUNK = 0x004e4942;

function appendAligned(parts, bytes) {
  const length = parts.reduce((sum, part) => sum + part.length, 0);
  const padding = (4 - (length % 4)) % 4;
  if (padding) parts.push(Buffer.alloc(padding));
  const offset = length + padding;
  const payload = Buffer.from(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  parts.push(payload);
  return { offset, byteLength: payload.length };
}

function encodeGlb(json, bin) {
  const rawJson = Buffer.from(JSON.stringify(json));
  const paddedJson = Buffer.concat([rawJson, Buffer.alloc((4 - rawJson.length % 4) % 4, 0x20)]);
  const paddedBin = Buffer.concat([bin, Buffer.alloc((4 - bin.length % 4) % 4)]);
  const output = Buffer.alloc(12 + 8 + paddedJson.length + 8 + paddedBin.length);
  output.write("glTF", 0, 4, "ascii");
  output.writeUInt32LE(2, 4);
  output.writeUInt32LE(output.length, 8);
  output.writeUInt32LE(paddedJson.length, 12);
  output.writeUInt32LE(JSON_CHUNK, 16);
  paddedJson.copy(output, 20);
  const binHeader = 20 + paddedJson.length;
  output.writeUInt32LE(paddedBin.length, binHeader);
  output.writeUInt32LE(BIN_CHUNK, binHeader + 4);
  paddedBin.copy(output, binHeader + 8);
  return output;
}

function parseGlb(bytes) {
  const jsonLength = bytes.readUInt32LE(12);
  const json = JSON.parse(bytes.subarray(20, 20 + jsonLength).toString("utf8").trimEnd());
  const binHeader = 20 + jsonLength;
  const binLength = bytes.readUInt32LE(binHeader);
  return {
    json,
    bin: bytes.subarray(binHeader + 8, binHeader + 8 + binLength),
  };
}

function staticTexturedGridGlb() {
  const side = 11;
  const positions = new Float32Array(side * side * 3);
  const normals = new Float32Array(side * side * 3);
  const uv = new Float32Array(side * side * 2);
  for (let z = 0; z < side; z += 1) {
    for (let x = 0; x < side; x += 1) {
      const vertex = z * side + x;
      positions.set([x / 5 - 1, 0, z / 5 - 1], vertex * 3);
      normals.set([0, 1, 0], vertex * 3);
      uv.set([x / (side - 1), z / (side - 1)], vertex * 2);
    }
  }
  const indexValues = [];
  for (let z = 0; z < side - 1; z += 1) {
    for (let x = 0; x < side - 1; x += 1) {
      const topLeft = z * side + x;
      const topRight = topLeft + 1;
      const bottomLeft = topLeft + side;
      const bottomRight = bottomLeft + 1;
      indexValues.push(topLeft, topRight, bottomRight, topLeft, bottomRight, bottomLeft);
    }
  }
  const indices = new Uint32Array(indexValues);
  const imageA = Buffer.alloc(4096, 0xa5);
  const imageB = Buffer.alloc(2048, 0x5a);
  const parts = [];
  const views = [
    appendAligned(parts, indices),
    appendAligned(parts, positions),
    appendAligned(parts, uv),
    appendAligned(parts, normals),
    appendAligned(parts, imageA),
    appendAligned(parts, imageB),
  ];
  const bin = Buffer.concat(parts);
  const json = {
    asset: { version: "2.0", generator: "meshy2aurora-test" },
    scene: 0,
    scenes: [{ nodes: [0] }],
    nodes: [{ mesh: 0 }],
    meshes: [{ primitives: [{
      attributes: { POSITION: 0, NORMAL: 1, TEXCOORD_0: 2 },
      indices: 3,
      material: 0,
      mode: 4,
    }] }],
    materials: [{ pbrMetallicRoughness: {
      baseColorTexture: { index: 0 },
      metallicRoughnessTexture: { index: 1 },
    } }],
    textures: [{ source: 0 }, { source: 1 }],
    images: [
      { mimeType: "image/jpeg", bufferView: 4 },
      { mimeType: "image/jpeg", bufferView: 5 },
    ],
    buffers: [{ byteLength: bin.length }],
    bufferViews: views.map((view, index) => ({
      buffer: 0,
      byteOffset: view.offset,
      byteLength: view.byteLength,
      ...(index < 4 ? { target: index === 0 ? 34963 : 34962 } : {}),
    })),
    accessors: [
      { bufferView: 1, componentType: 5126, count: side * side, type: "VEC3", min: [-1, 0, -1], max: [1, 0, 1] },
      { bufferView: 3, componentType: 5126, count: side * side, type: "VEC3" },
      { bufferView: 2, componentType: 5126, count: side * side, type: "VEC2" },
      { bufferView: 0, componentType: 5125, count: indices.length, type: "SCALAR" },
    ],
  };
  return { payload: encodeGlb(json, bin), imageA, imageB };
}

test("exact-target output compacts superseded geometry and preserves embedded image bytes", async () => {
  const directory = await mkdtemp(join(tmpdir(), "m2a-exact-target-"));
  try {
    const inputPath = join(directory, "source.glb");
    const outputPath = join(directory, "target.glb");
    const fixture = staticTexturedGridGlb();
    await writeFile(inputPath, fixture.payload, { flag: "wx" });
    const run = spawnSync(
      process.execPath,
      [
        resolve("tools/meshy-exact-triangle-target.mjs"),
        "--input", inputPath,
        "--output", outputPath,
        "--triangles", "100",
      ],
      { cwd: resolve("."), encoding: "utf8" },
    );
    assert.equal(run.status, 0, run.stderr);
    const output = await readFile(outputPath);
    const source = parseGlb(fixture.payload);
    const result = parseGlb(output);

    assert.ok(
      result.json.buffers[0].byteLength < source.json.buffers[0].byteLength,
      "output BIN must omit superseded source geometry",
    );
    for (const [imageIndex, expected] of [fixture.imageA, fixture.imageB].entries()) {
      const view = result.json.bufferViews[result.json.images[imageIndex].bufferView];
      const actual = result.bin.subarray(view.byteOffset ?? 0, (view.byteOffset ?? 0) + view.byteLength);
      assert.equal(createHash("sha256").update(actual).digest("hex"), createHash("sha256").update(expected).digest("hex"));
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
