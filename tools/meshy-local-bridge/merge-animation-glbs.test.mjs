import assert from "node:assert/strict";
import test from "node:test";

import { mergeMeshyAnimationGlbs } from "./merge-animation-glbs.mjs";

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

function animationGlb(delta) {
  const bin = Buffer.alloc(40);
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
  return encode({
    asset: { version: "2.0" },
    scene: 0,
    scenes: [{ nodes: [0] }],
    nodes: [{ name: "Armature", children: [1] }, { name: "Hips" }],
    skins: [{ joints: [1], skeleton: 1 }],
    bufferViews: [{ buffer: 0, byteOffset: 0, byteLength: 8 }, { buffer: 0, byteOffset: 8, byteLength: 32 }],
    accessors: [
      { bufferView: 0, componentType: 5126, count: 2, type: "SCALAR", min: [0], max: [1] },
      { bufferView: 1, componentType: 5126, count: 2, type: "VEC4" },
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
