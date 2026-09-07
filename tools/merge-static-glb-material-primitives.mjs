#!/usr/bin/env node

import { createHash } from "node:crypto";
import { closeSync, openSync, readFileSync, writeFileSync } from "node:fs";

const GLB_MAGIC = 0x46546c67;
const JSON_CHUNK_TYPE = 0x4e4f534a;

const [sourcePath, outputPath] = process.argv.slice(2);
if (!sourcePath || !outputPath) {
  throw new Error(
    "usage: merge-static-glb-material-primitives.mjs <source.glb> <output.glb>",
  );
}

const source = readFileSync(sourcePath);
if (source.readUInt32LE(0) !== GLB_MAGIC || source.readUInt32LE(4) !== 2) {
  throw new Error("source is not a GLB 2.0 container");
}
if (source.readUInt32LE(8) !== source.length) {
  throw new Error("source GLB declared length differs from file length");
}

const chunks = [];
for (let offset = 12; offset < source.length; ) {
  if (offset + 8 > source.length) {
    throw new Error("source GLB chunk header is truncated");
  }
  const byteLength = source.readUInt32LE(offset);
  const type = source.readUInt32LE(offset + 4);
  const end = offset + 8 + byteLength;
  if (end > source.length) {
    throw new Error("source GLB chunk payload is truncated");
  }
  chunks.push({ type, payload: source.subarray(offset + 8, end) });
  offset = end;
}
if (chunks.length < 2 || chunks[0].type !== JSON_CHUNK_TYPE) {
  throw new Error("source GLB must contain a leading JSON chunk and binary payload");
}

const document = JSON.parse(chunks[0].payload.toString("utf8").trim());
if (document.animations?.length || document.skins?.length) {
  throw new Error("only static, unskinned GLBs may merge mesh nodes");
}
if (document.scenes?.length !== 1 || document.scenes[0].nodes?.length !== 1) {
  throw new Error("source must have exactly one scene root");
}

const rootIndex = document.scenes[0].nodes[0];
const root = document.nodes?.[rootIndex];
const transformKeys = ["matrix", "translation", "rotation", "scale"];
if (!root || root.mesh !== undefined || transformKeys.some((key) => root[key] !== undefined)) {
  throw new Error("scene root must be an untransformed grouping node");
}
if (!Array.isArray(root.children) || root.children.length < 2) {
  throw new Error("scene root must group at least two mesh nodes");
}

const primitives = [];
for (const childIndex of root.children) {
  const node = document.nodes[childIndex];
  if (
    !node ||
    node.mesh === undefined ||
    node.skin !== undefined ||
    node.children?.length ||
    transformKeys.some((key) => node[key] !== undefined)
  ) {
    throw new Error(`child node ${childIndex} is not a plain static mesh node`);
  }
  const mesh = document.meshes?.[node.mesh];
  if (!mesh || !Array.isArray(mesh.primitives) || mesh.primitives.length === 0) {
    throw new Error(`child node ${childIndex} references an empty mesh`);
  }
  primitives.push(...mesh.primitives);
}

document.nodes = [{ name: root.name ?? "ROOT", mesh: 0 }];
document.meshes = [{ name: "merged_static_material_primitives", primitives }];
document.scenes[0].nodes = [0];

const jsonText = JSON.stringify(document);
const jsonPadding = (4 - (Buffer.byteLength(jsonText) % 4)) % 4;
const jsonPayload = Buffer.from(jsonText + " ".repeat(jsonPadding), "utf8");
const outputChunks = [{ type: JSON_CHUNK_TYPE, payload: jsonPayload }, ...chunks.slice(1)];
const totalLength =
  12 + outputChunks.reduce((sum, chunk) => sum + 8 + chunk.payload.length, 0);
const output = Buffer.allocUnsafe(totalLength);
output.writeUInt32LE(GLB_MAGIC, 0);
output.writeUInt32LE(2, 4);
output.writeUInt32LE(totalLength, 8);
let writeOffset = 12;
for (const chunk of outputChunks) {
  output.writeUInt32LE(chunk.payload.length, writeOffset);
  output.writeUInt32LE(chunk.type, writeOffset + 4);
  chunk.payload.copy(output, writeOffset + 8);
  writeOffset += 8 + chunk.payload.length;
}

const handle = openSync(outputPath, "wx");
try {
  writeFileSync(handle, output);
} finally {
  closeSync(handle);
}

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: 1,
      sourceSha256: sha256(source),
      outputSha256: sha256(output),
      outputByteLength: output.length,
      sourceMeshNodeCount: root.children.length,
      outputMeshNodeCount: 1,
      primitiveCount: primitives.length,
      materialIds: primitives.map((primitive) => primitive.material ?? null),
      binaryChunksPreservedByteIdentical: chunks
        .slice(1)
        .every((chunk, index) => chunk.payload.equals(outputChunks[index + 1].payload)),
    },
    null,
    2,
  )}\n`,
);
