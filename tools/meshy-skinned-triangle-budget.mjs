import { createHash } from "node:crypto";
import { open, readFile } from "node:fs/promises";
import { resolve } from "node:path";

import { MeshoptSimplifier } from "../apps/studio-web/node_modules/meshoptimizer/meshopt_simplifier.js";

const COMPONENTS = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4 };
const CONSTRUCTORS = { 5121: Uint8Array, 5123: Uint16Array, 5125: Uint32Array, 5126: Float32Array };
const JSON_CHUNK = 0x4e4f534a;
const BIN_CHUNK = 0x004e4942;
const GEOMETRY_ATTRIBUTES = ["POSITION", "NORMAL", "TEXCOORD_0", "JOINTS_0", "WEIGHTS_0"];

function parseArguments(values) {
  const flags = new Map();
  for (let index = 0; index < values.length; index += 2) {
    const key = values[index];
    const value = values[index + 1];
    if (!key?.startsWith("--") || !value || value.startsWith("--") || flags.has(key)) throw new Error(`invalid or duplicate argument ${key ?? "<missing>"}`);
    flags.set(key, value);
  }
  for (const key of flags.keys()) if (!["--input", "--output", "--max-triangles"].includes(key)) throw new Error(`unsupported argument ${key}`);
  const maximumTriangles = Number(flags.get("--max-triangles"));
  if (!Number.isInteger(maximumTriangles) || maximumTriangles < 1) throw new Error("--max-triangles must be a positive integer");
  if (!flags.get("--input") || !flags.get("--output")) throw new Error("--input and --output are required");
  return { input: resolve(flags.get("--input")), output: resolve(flags.get("--output")), maximumTriangles };
}

function parseGlb(bytes) {
  if (bytes.length < 28 || bytes.toString("ascii", 0, 4) !== "glTF" || bytes.readUInt32LE(4) !== 2 || bytes.readUInt32LE(8) !== bytes.length) {
    throw new Error("input is not a canonical GLB 2.0 container");
  }
  let cursor = 12;
  let json;
  let bin;
  while (cursor < bytes.length) {
    const length = bytes.readUInt32LE(cursor);
    const type = bytes.readUInt32LE(cursor + 4);
    const payload = bytes.subarray(cursor + 8, cursor + 8 + length);
    if (payload.length !== length) throw new Error("GLB chunk exceeds the container");
    if (type === JSON_CHUNK && !json) json = JSON.parse(payload.toString("utf8").trimEnd());
    else if (type === BIN_CHUNK && !bin) bin = payload;
    else throw new Error("GLB must contain exactly one JSON and one BIN chunk");
    cursor += 8 + length;
  }
  if (!json || !bin || json.buffers?.length !== 1 || json.buffers[0].uri !== undefined) throw new Error("GLB must use one embedded buffer");
  return { json, bin: bin.subarray(0, json.buffers[0].byteLength) };
}

function typedAccessor(json, bin, accessorIndex) {
  const accessor = json.accessors?.[accessorIndex];
  const view = json.bufferViews?.[accessor?.bufferView];
  const components = COMPONENTS[accessor?.type];
  const Constructor = CONSTRUCTORS[accessor?.componentType];
  if (!accessor || !view || view.buffer !== 0 || !components || !Constructor || accessor.sparse || view.byteStride !== undefined) {
    throw new Error(`accessor ${accessorIndex} is not a supported tightly-packed geometry accessor`);
  }
  const offset = (view.byteOffset ?? 0) + (accessor.byteOffset ?? 0);
  if (offset % Constructor.BYTES_PER_ELEMENT !== 0) throw new Error(`accessor ${accessorIndex} is misaligned`);
  return new Constructor(bin.buffer, bin.byteOffset + offset, accessor.count * components);
}

function triangleAreaSquared(positions, a, b, c) {
  const abx = positions[b * 3] - positions[a * 3];
  const aby = positions[b * 3 + 1] - positions[a * 3 + 1];
  const abz = positions[b * 3 + 2] - positions[a * 3 + 2];
  const acx = positions[c * 3] - positions[a * 3];
  const acy = positions[c * 3 + 1] - positions[a * 3 + 1];
  const acz = positions[c * 3 + 2] - positions[a * 3 + 2];
  const x = aby * acz - abz * acy;
  const y = abz * acx - abx * acz;
  const z = abx * acy - aby * acx;
  return x * x + y * y + z * z;
}

function retainSafeTriangles(indices, positions) {
  const retained = [];
  for (let offset = 0; offset < indices.length; offset += 3) {
    const area = triangleAreaSquared(positions, indices[offset], indices[offset + 1], indices[offset + 2]);
    if (Number.isFinite(area) && area > 1.0e-10) retained.push(indices[offset], indices[offset + 1], indices[offset + 2]);
  }
  return new indices.constructor(retained);
}

function compactGeometry(indices, sourceAttributes, components, IndexArray) {
  const vertexCount = sourceAttributes.POSITION.length / 3;
  const remap = new Int32Array(vertexCount);
  remap.fill(-1);
  const referenced = [];
  const outputIndices = new IndexArray(indices.length);
  for (let index = 0; index < indices.length; index += 1) {
    const sourceVertex = indices[index];
    if (sourceVertex >= vertexCount) throw new Error(`index ${sourceVertex} exceeds the vertex buffer`);
    if (remap[sourceVertex] === -1) {
      remap[sourceVertex] = referenced.length;
      referenced.push(sourceVertex);
    }
    outputIndices[index] = remap[sourceVertex];
  }
  const outputAttributes = {};
  for (const semantic of GEOMETRY_ATTRIBUTES) {
    const source = sourceAttributes[semantic];
    const width = components[semantic];
    const output = new source.constructor(referenced.length * width);
    for (let vertex = 0; vertex < referenced.length; vertex += 1) {
      const sourceVertex = referenced[vertex];
      output.set(source.subarray(sourceVertex * width, sourceVertex * width + width), vertex * width);
    }
    outputAttributes[semantic] = output;
  }
  return { indices: outputIndices, attributes: outputAttributes, removedUnreferencedVertices: vertexCount - referenced.length };
}

function bounds(positions) {
  const min = [Infinity, Infinity, Infinity];
  const max = [-Infinity, -Infinity, -Infinity];
  for (let offset = 0; offset < positions.length; offset += 3) {
    for (let lane = 0; lane < 3; lane += 1) {
      min[lane] = Math.min(min[lane], positions[offset + lane]);
      max[lane] = Math.max(max[lane], positions[offset + lane]);
    }
  }
  return { min, max };
}

function appendAligned(parts, bytes, currentLength) {
  const padding = (4 - currentLength % 4) % 4;
  if (padding) parts.push(Buffer.alloc(padding));
  const offset = currentLength + padding;
  parts.push(Buffer.from(bytes.buffer, bytes.byteOffset, bytes.byteLength));
  return { offset, nextLength: offset + bytes.byteLength };
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

async function main() {
  const command = parseArguments(process.argv.slice(2));
  const source = await readFile(command.input);
  const { json, bin } = parseGlb(source);
  if (json.meshes?.length !== 1 || json.meshes[0].primitives?.length !== 1 || json.skins?.length !== 1 || !json.animations?.length) {
    throw new Error("skinned budget profile requires one mesh primitive, one skin and at least one animation");
  }
  const primitive = json.meshes[0].primitives[0];
  if ((primitive.mode ?? 4) !== 4 || Object.keys(primitive.attributes ?? {}).sort().join(",") !== [...GEOMETRY_ATTRIBUTES].sort().join(",")) {
    throw new Error("skinned budget profile requires indexed POSITION/NORMAL/TEXCOORD_0/JOINTS_0/WEIGHTS_0 triangles");
  }
  const sourceAttributes = {};
  const componentWidths = {};
  for (const semantic of GEOMETRY_ATTRIBUTES) {
    const accessorIndex = primitive.attributes[semantic];
    sourceAttributes[semantic] = typedAccessor(json, bin, accessorIndex);
    componentWidths[semantic] = COMPONENTS[json.accessors[accessorIndex].type];
  }
  const sourceIndices = typedAccessor(json, bin, primitive.indices);
  const sourceTriangles = sourceIndices.length / 3;
  if (sourceTriangles <= command.maximumTriangles) throw new Error(`source already satisfies the budget: ${sourceTriangles} <= ${command.maximumTriangles}`);
  const vertexCount = sourceAttributes.POSITION.length / 3;
  if (GEOMETRY_ATTRIBUTES.some((semantic) => sourceAttributes[semantic].length / componentWidths[semantic] !== vertexCount)) {
    throw new Error("skinned geometry attribute counts differ");
  }

  const simplifierAttributes = new Float32Array(vertexCount * 13);
  for (let vertex = 0; vertex < vertexCount; vertex += 1) {
    let cursor = vertex * 13;
    for (const semantic of ["NORMAL", "TEXCOORD_0", "WEIGHTS_0", "JOINTS_0"]) {
      const values = sourceAttributes[semantic];
      const width = componentWidths[semantic];
      for (let lane = 0; lane < width; lane += 1) simplifierAttributes[cursor++] = values[vertex * width + lane];
    }
  }
  await MeshoptSimplifier.ready;
  const [simplified, simplificationError] = MeshoptSimplifier.simplifyWithAttributes(
    sourceIndices,
    sourceAttributes.POSITION,
    3,
    simplifierAttributes,
    13,
    [0.25, 0.25, 0.25, 1, 1, 2, 2, 2, 2, 8, 8, 8, 8],
    null,
    command.maximumTriangles * 3,
    0.01,
    ["Permissive"],
  );
  const safeIndices = retainSafeTriangles(simplified, sourceAttributes.POSITION);
  if (!safeIndices.length || safeIndices.length / 3 > command.maximumTriangles) throw new Error("simplified geometry does not satisfy the Aurora budget");
  const IndexArray = json.accessors[primitive.indices].componentType === 5123 ? Uint16Array : Uint32Array;
  const compacted = compactGeometry(safeIndices, sourceAttributes, componentWidths, IndexArray);
  for (let offset = 0; offset < compacted.indices.length; offset += 3) {
    if (triangleAreaSquared(compacted.attributes.POSITION, compacted.indices[offset], compacted.indices[offset + 1], compacted.indices[offset + 2]) <= 1.0e-10) {
      throw new Error(`readback triangle ${offset / 3} is degenerate`);
    }
  }

  const parts = [Buffer.from(bin)];
  let length = bin.length;
  const appended = {};
  for (const semantic of GEOMETRY_ATTRIBUTES) {
    appended[semantic] = appendAligned(parts, compacted.attributes[semantic], length);
    length = appended[semantic].nextLength;
  }
  const indexAppend = appendAligned(parts, compacted.indices, length);
  length = indexAppend.nextLength;
  const outputBin = Buffer.concat(parts);
  if (outputBin.length !== length) throw new Error("internal BIN length mismatch");
  for (const semantic of GEOMETRY_ATTRIBUTES) {
    const accessor = json.accessors[primitive.attributes[semantic]];
    const view = json.bufferViews[accessor.bufferView];
    view.byteOffset = appended[semantic].offset;
    view.byteLength = compacted.attributes[semantic].byteLength;
    delete view.byteStride;
    accessor.byteOffset = 0;
    accessor.count = compacted.attributes[semantic].length / componentWidths[semantic];
  }
  const outputBounds = bounds(compacted.attributes.POSITION);
  json.accessors[primitive.attributes.POSITION].min = outputBounds.min;
  json.accessors[primitive.attributes.POSITION].max = outputBounds.max;
  const indexAccessor = json.accessors[primitive.indices];
  const indexView = json.bufferViews[indexAccessor.bufferView];
  indexView.byteOffset = indexAppend.offset;
  indexView.byteLength = compacted.indices.byteLength;
  delete indexView.byteStride;
  indexAccessor.byteOffset = 0;
  indexAccessor.count = compacted.indices.length;
  json.buffers[0].byteLength = outputBin.length;

  const output = encodeGlb(json, outputBin);
  const readback = parseGlb(output);
  const readbackPrimitive = readback.json.meshes[0].primitives[0];
  const readbackIndices = typedAccessor(readback.json, readback.bin, readbackPrimitive.indices);
  if (readbackIndices.length / 3 > command.maximumTriangles) throw new Error("encoded readback exceeds the triangle budget");
  const file = await open(command.output, "wx");
  try {
    await file.writeFile(output);
    await file.sync();
  } finally {
    await file.close();
  }
  process.stdout.write(`${JSON.stringify({
    schemaVersion: 1,
    input: command.input,
    output: command.output,
    sourceSha256: createHash("sha256").update(source).digest("hex"),
    outputSha256: createHash("sha256").update(output).digest("hex"),
    sourceTriangles,
    outputTriangles: readbackIndices.length / 3,
    maximumTriangles: command.maximumTriangles,
    sourceVertices: vertexCount,
    outputVertices: compacted.attributes.POSITION.length / 3,
    removedUnreferencedVertices: compacted.removedUnreferencedVertices,
    removedDegenerateAfterSimplification: simplified.length / 3 - safeIndices.length / 3,
    simplificationError,
    skinCount: readback.json.skins.length,
    animationNames: readback.json.animations.map((animation) => animation.name),
    preservesEmbeddedMaterialsImagesSkinAndAnimations: true,
  }, null, 2)}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
