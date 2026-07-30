import assert from "node:assert/strict";
import test from "node:test";

import { inspectGlbTriangleCount, mergeMeshyAnimationGlbs } from "./merge-animation-glbs.mjs";

const GLB_MAGIC = 0x46546c67;
const JSON_CHUNK = 0x4e4f534a;
const BIN_CHUNK = 0x004e4942;

function encode(document, bin) {
  document.buffers = [{ byteLength: bin.length }];
  const rawJson = Buffer.from(JSON.stringify(document));
  const json = Buffer.concat([rawJson, Buffer.alloc((4 - rawJson.length % 4) % 4, 0x20)]);
  const paddedBin = Buffer.concat([bin, Buffer.alloc((4 - bin.length % 4) % 4)]);
  const output = Buffer.alloc(12 + 8 + json.length + 8 + paddedBin.length);
  output.writeUInt32LE(GLB_MAGIC, 0);
  output.writeUInt32LE(2, 4);
  output.writeUInt32LE(output.length, 8);
  output.writeUInt32LE(json.length, 12);
  output.writeUInt32LE(JSON_CHUNK, 16);
  json.copy(output, 20);
  const binHeader = 20 + json.length;
  output.writeUInt32LE(paddedBin.length, binHeader);
  output.writeUInt32LE(BIN_CHUNK, binHeader + 4);
  paddedBin.copy(output, binHeader + 8);
  return output;
}

function animationGlb(delta, { positionX = 0, weight = 1, inverseBindTranslation = 0 } = {}) {
  const bin = Buffer.alloc(220);
  bin.writeFloatLE(0, 0);
  bin.writeFloatLE(1, 4);
  bin.writeFloatLE(0, 8);
  bin.writeFloatLE(0, 12);
  bin.writeFloatLE(0, 16);
  bin.writeFloatLE(1, 20);
  bin.writeFloatLE(0, 24);
  bin.writeFloatLE(delta, 28);
  bin.writeFloatLE(0, 32);
  bin.writeFloatLE(1, 36);
  const positions = [
    positionX, 0, 0,
    1, 0, 0,
    0, 1, 0,
  ];
  positions.forEach((value, index) => bin.writeFloatLE(value, 40 + index * 4));
  for (let vertex = 0; vertex < 3; vertex += 1) {
    bin.writeUInt16LE(0, 76 + vertex * 8);
    bin.writeUInt16LE(0, 76 + vertex * 8 + 2);
    bin.writeUInt16LE(0, 76 + vertex * 8 + 4);
    bin.writeUInt16LE(0, 76 + vertex * 8 + 6);
    bin.writeFloatLE(weight, 100 + vertex * 16);
    bin.writeFloatLE(0, 100 + vertex * 16 + 4);
    bin.writeFloatLE(0, 100 + vertex * 16 + 8);
    bin.writeFloatLE(0, 100 + vertex * 16 + 12);
  }
  bin.writeUInt16LE(0, 148);
  bin.writeUInt16LE(1, 150);
  bin.writeUInt16LE(2, 152);
  for (let index = 0; index < 16; index += 1) {
    bin.writeFloatLE(index % 5 === 0 ? 1 : 0, 156 + index * 4);
  }
  bin.writeFloatLE(inverseBindTranslation, 156 + 12 * 4);
  return encode({
    asset: { version: "2.0" },
    scene: 0,
    scenes: [{ nodes: [0] }],
    nodes: [
      { name: "Armature", children: [1, 2] },
      { name: "Hips" },
      { name: "Model", mesh: 0, skin: 0 },
    ],
    skins: [{ joints: [1], skeleton: 1, inverseBindMatrices: 6 }],
    materials: [{ name: "Body" }],
    meshes: [{
      primitives: [{
        attributes: { POSITION: 2, JOINTS_0: 3, WEIGHTS_0: 4 },
        indices: 5,
        material: 0,
        mode: 4,
      }],
    }],
    bufferViews: [
      { buffer: 0, byteOffset: 0, byteLength: 8 },
      { buffer: 0, byteOffset: 8, byteLength: 32 },
      { buffer: 0, byteOffset: 40, byteLength: 36 },
      { buffer: 0, byteOffset: 76, byteLength: 24 },
      { buffer: 0, byteOffset: 100, byteLength: 48 },
      { buffer: 0, byteOffset: 148, byteLength: 6 },
      { buffer: 0, byteOffset: 156, byteLength: 64 },
    ],
    accessors: [
      { bufferView: 0, componentType: 5126, count: 2, type: "SCALAR", min: [0], max: [1] },
      { bufferView: 1, componentType: 5126, count: 2, type: "VEC4" },
      { bufferView: 2, componentType: 5126, count: 3, type: "VEC3" },
      { bufferView: 3, componentType: 5123, count: 3, type: "VEC4" },
      { bufferView: 4, componentType: 5126, count: 3, type: "VEC4" },
      { bufferView: 5, componentType: 5123, count: 3, type: "SCALAR" },
      { bufferView: 6, componentType: 5126, count: 1, type: "MAT4" },
    ],
    animations: [{
      name: "MeshyAction",
      samplers: [{ input: 0, output: 1, interpolation: "LINEAR" }],
      channels: [{ sampler: 0, target: { node: 1, path: "rotation" } }],
    }],
  }, bin);
}

function parse(bytes) {
  const buffer = Buffer.from(bytes);
  const jsonLength = buffer.readUInt32LE(12);
  return JSON.parse(buffer.toString("utf8", 20, 20 + jsonLength).trimEnd());
}

test("merges same-rig Meshy action GLBs and assigns explicit NWN clip names", () => {
  assert.equal(inspectGlbTriangleCount(animationGlb(0.01), "fixture"), 1);
  const output = mergeMeshyAnimationGlbs([
    { clipName: "cpause1", label: "idle", bytes: animationGlb(0.01) },
    { clipName: "cwalk", label: "walk", bytes: animationGlb(0.2) },
    { clipName: "ca1slashl", label: "attack", bytes: animationGlb(0.5) },
  ]);
  const document = parse(output);
  assert.deepEqual(document.animations.map((animation) => animation.name), ["cpause1", "cwalk", "ca1slashl"]);
  assert.equal(document.animations[0].samplers[0].input >= 2, true);
  assert.equal(document.animations[1].samplers[0].input > document.animations[0].samplers[0].input, true);
  assert.equal(document.buffers[0].byteLength > 40, true);
});

test("fails closed when an action GLB rig differs from the base", () => {
  const donor = animationGlb(0.2);
  const jsonLength = donor.readUInt32LE(12);
  const document = JSON.parse(donor.toString("utf8", 20, 20 + jsonLength).trimEnd());
  document.nodes[1].name = "DifferentHips";
  const binHeader = 20 + jsonLength;
  const binLength = donor.readUInt32LE(binHeader);
  const bin = donor.subarray(binHeader + 8, binHeader + 8 + binLength);
  assert.throws(() => mergeMeshyAnimationGlbs([
    { clipName: "cpause1", label: "idle", bytes: animationGlb(0.01) },
    { clipName: "cwalk", label: "walk", bytes: encode(document, bin) },
  ]), /node 1 differs/);
});

test("fails closed when logical skin weights differ despite identical node topology", () => {
  assert.throws(() => mergeMeshyAnimationGlbs([
    { clipName: "cpause1", label: "idle", bytes: animationGlb(0.01) },
    { clipName: "cwalk", label: "walk", bytes: animationGlb(0.2, { weight: 0.75 }) },
  ]), /WEIGHTS_0.*differs/);
});

test("fails closed when inverse bind matrices differ despite identical joint topology", () => {
  assert.throws(() => mergeMeshyAnimationGlbs([
    { clipName: "cpause1", label: "idle", bytes: animationGlb(0.01) },
    {
      clipName: "cwalk",
      label: "walk",
      bytes: animationGlb(0.2, { inverseBindTranslation: 0.25 }),
    },
  ]), /inverseBindMatrices.*differs/);
});

test("fails closed when logical positions differ despite identical node topology", () => {
  assert.throws(() => mergeMeshyAnimationGlbs([
    { clipName: "cpause1", label: "idle", bytes: animationGlb(0.01) },
    { clipName: "cwalk", label: "walk", bytes: animationGlb(0.2, { positionX: 0.125 }) },
  ]), /POSITION.*differs/);
});
