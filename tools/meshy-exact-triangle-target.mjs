import { createHash } from "node:crypto";
import { open, readFile } from "node:fs/promises";
import { resolve } from "node:path";

import { MeshoptSimplifier } from "../apps/studio-web/node_modules/meshoptimizer/meshopt_simplifier.js";

const COMPONENTS = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4 };
const GLB_JSON_CHUNK = 0x4e4f534a;
const GLB_BIN_CHUNK = 0x004e4942;

function required(flags, name) {
  const value = flags.get(name);
  if (!value) throw new Error(`${name} is required`);
  return value;
}

function parseArguments(values) {
  const flags = new Map();
  for (let index = 0; index < values.length; index += 2) {
    const name = values[index];
    const value = values[index + 1];
    if (!name?.startsWith("--") || !value || value.startsWith("--") || flags.has(name)) {
      throw new Error(`invalid or duplicate argument: ${name ?? "<missing>"}`);
    }
    flags.set(name, value);
  }
  for (const name of flags.keys()) {
    if (!["--input", "--output", "--triangles", "--height"].includes(name)) {
      throw new Error(`unsupported argument: ${name}`);
    }
  }
  const triangles = Number(required(flags, "--triangles"));
  if (!Number.isInteger(triangles) || triangles < 1) {
    throw new Error("--triangles must be a positive integer");
  }
  const height = flags.has("--height") ? Number(flags.get("--height")) : undefined;
  if (height !== undefined && (!Number.isFinite(height) || height <= 0)) {
    throw new Error("--height must be a positive finite number");
  }
  return {
    input: resolve(required(flags, "--input")),
    output: resolve(required(flags, "--output")),
    triangles,
    height,
  };
}

function parseGlb(bytes) {
  if (
    bytes.length < 28
    || bytes.toString("ascii", 0, 4) !== "glTF"
    || bytes.readUInt32LE(4) !== 2
    || bytes.readUInt32LE(8) !== bytes.length
  ) {
    throw new Error("input is not a canonical GLB 2.0 container");
  }
  let cursor = 12;
  let json;
  let bin;
  while (cursor < bytes.length) {
    const length = bytes.readUInt32LE(cursor);
    const type = bytes.readUInt32LE(cursor + 4);
    const payload = bytes.subarray(cursor + 8, cursor + 8 + length);
    if (type === GLB_JSON_CHUNK) {
      if (json) throw new Error("GLB contains more than one JSON chunk");
      json = JSON.parse(payload.toString("utf8").replace(/\0+$/, "").trimEnd());
    } else if (type === GLB_BIN_CHUNK) {
      if (bin) throw new Error("GLB contains more than one BIN chunk");
      bin = payload;
    } else {
      throw new Error(`unsupported GLB chunk type ${type}`);
    }
    cursor += 8 + length;
  }
  if (!json || !bin || json.buffers?.length !== 1) {
    throw new Error("GLB must contain one JSON chunk and one embedded buffer");
  }
  return { json, bin: bin.subarray(0, json.buffers[0].byteLength) };
}

function typedAccessor(json, bin, accessorIndex) {
  const accessor = json.accessors?.[accessorIndex];
  const view = json.bufferViews?.[accessor?.bufferView];
  const components = COMPONENTS[accessor?.type];
  if (!accessor || !view || view.buffer !== 0 || !components || accessor.sparse) {
    throw new Error(`accessor ${accessorIndex} is unsupported`);
  }
  const Constructor = accessor.componentType === 5123
    ? Uint16Array
    : accessor.componentType === 5125
      ? Uint32Array
      : accessor.componentType === 5126
        ? Float32Array
        : undefined;
  if (!Constructor) throw new Error(`accessor ${accessorIndex} component type is unsupported`);
  const byteOffset = (view.byteOffset ?? 0) + (accessor.byteOffset ?? 0);
  const count = accessor.count * components;
  if (view.byteStride !== undefined || byteOffset % Constructor.BYTES_PER_ELEMENT !== 0) {
    throw new Error(`accessor ${accessorIndex} must be tightly packed and aligned`);
  }
  return new Constructor(bin.buffer, bin.byteOffset + byteOffset, count);
}

function appendAligned(parts, bytes, currentLength) {
  const padding = (4 - (currentLength % 4)) % 4;
  if (padding) parts.push(Buffer.alloc(padding));
  const offset = currentLength + padding;
  const payload = Buffer.from(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  parts.push(payload);
  return { offset, nextLength: offset + payload.length };
}

function normalize3(x, y, z) {
  const length = Math.hypot(x, y, z);
  if (!Number.isFinite(length) || length <= Number.EPSILON) {
    throw new Error("interpolated normal is invalid");
  }
  return [x / length, y / length, z / length];
}

function squaredDistance(positions, left, right) {
  const x = positions[left * 3] - positions[right * 3];
  const y = positions[left * 3 + 1] - positions[right * 3 + 1];
  const z = positions[left * 3 + 2] - positions[right * 3 + 2];
  return x * x + y * y + z * z;
}

function triangleCrossLengthSquared(positions, a, b, c) {
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

function retainAuroraSafeTriangles(indices, positions) {
  const retained = [];
  for (let offset = 0; offset < indices.length; offset += 3) {
    const areaSquared = triangleCrossLengthSquared(
      positions,
      indices[offset],
      indices[offset + 1],
      indices[offset + 2],
    );
    if (Number.isFinite(areaSquared) && areaSquared > 1.0e-10) {
      retained.push(indices[offset], indices[offset + 1], indices[offset + 2]);
    }
  }
  return {
    indices: new indices.constructor(retained),
    removed: indices.length / 3 - retained.length / 3,
  };
}

function splitDefinition(indices, offset, positions, newVertex) {
  const a = indices[offset];
  const b = indices[offset + 1];
  const c = indices[offset + 2];
  const edges = [
    { left: a, right: b, opposite: c, kind: "ab", length: squaredDistance(positions, a, b) },
    { left: b, right: c, opposite: a, kind: "bc", length: squaredDistance(positions, b, c) },
    { left: c, right: a, opposite: b, kind: "ca", length: squaredDistance(positions, c, a) },
  ].sort((left, right) => right.length - left.length);
  const edge = edges[0];
  if (!Number.isFinite(edge.length) || edge.length <= Number.EPSILON) {
    throw new Error("cannot split a zero-length triangle edge");
  }
  const position = new Float32Array(3);
  for (let lane = 0; lane < 3; lane += 1) {
    position[lane] =
      (positions[edge.left * 3 + lane] + positions[edge.right * 3 + lane]) * 0.5;
  }
  let triangles;
  if (edge.kind === "ab") {
    triangles = [a, newVertex, c, newVertex, b, c];
  } else if (edge.kind === "bc") {
    triangles = [a, b, newVertex, a, newVertex, c];
  } else {
    triangles = [a, b, newVertex, newVertex, b, c];
  }
  return { edge, position, triangles };
}

function subdivideLargestTrianglesToTarget(
  indices,
  positions,
  normals,
  uv,
  targetTriangles,
  IndexArray,
) {
  const currentTriangles = indices.length / 3;
  const deficit = targetTriangles - currentTriangles;
  if (deficit < 0) {
    throw new Error(`safe geometry exceeds exact target by ${-deficit} triangles`);
  }
  if (deficit === 0) {
    return { indices, positions, normals, uv, subdividedTriangles: 0 };
  }
  const candidates = [];
  for (let offset = 0; offset < indices.length; offset += 3) {
    candidates.push({
      offset,
      areaSquared: triangleCrossLengthSquared(
        positions,
        indices[offset],
        indices[offset + 1],
        indices[offset + 2],
      ),
    });
  }
  candidates.sort(
    (left, right) => right.areaSquared - left.areaSquared || left.offset - right.offset,
  );
  const selected = new Map();
  const appended = [];
  for (const candidate of candidates) {
    if (selected.size === deficit) break;
    const newVertex = positions.length / 3 + selected.size;
    if (IndexArray === Uint16Array && newVertex > 65_535) {
      throw new Error("subdivided vertex exceeds the existing UNSIGNED_SHORT index width");
    }
    const definition = splitDefinition(indices, candidate.offset, positions, newVertex);
    const scratch = new Float32Array(positions.length + 3);
    scratch.set(positions);
    scratch.set(definition.position, positions.length);
    const leftArea = triangleCrossLengthSquared(
      scratch,
      definition.triangles[0],
      definition.triangles[1],
      definition.triangles[2],
    );
    const rightArea = triangleCrossLengthSquared(
      scratch,
      definition.triangles[3],
      definition.triangles[4],
      definition.triangles[5],
    );
    if (leftArea <= 1.0e-10 || rightArea <= 1.0e-10) continue;
    selected.set(candidate.offset, definition);
    appended.push(definition);
  }
  if (selected.size !== deficit) {
    throw new Error(
      `only ${selected.size} triangles can be safely subdivided; ${deficit} are required`,
    );
  }

  const nextPositions = new Float32Array(positions.length + deficit * 3);
  const nextNormals = new Float32Array(normals.length + deficit * 3);
  const nextUv = new Float32Array(uv.length + deficit * 2);
  nextPositions.set(positions);
  nextNormals.set(normals);
  nextUv.set(uv);
  for (let index = 0; index < appended.length; index += 1) {
    const definition = appended[index];
    nextPositions.set(definition.position, positions.length + index * 3);
    const normal = normalize3(
      normals[definition.edge.left * 3] + normals[definition.edge.right * 3],
      normals[definition.edge.left * 3 + 1] + normals[definition.edge.right * 3 + 1],
      normals[definition.edge.left * 3 + 2] + normals[definition.edge.right * 3 + 2],
    );
    nextNormals.set(normal, normals.length + index * 3);
    nextUv[uv.length + index * 2] =
      (uv[definition.edge.left * 2] + uv[definition.edge.right * 2]) * 0.5;
    nextUv[uv.length + index * 2 + 1] =
      (uv[definition.edge.left * 2 + 1] + uv[definition.edge.right * 2 + 1]) * 0.5;
  }

  const nextIndices = new IndexArray(indices.length + deficit * 3);
  let outputOffset = 0;
  for (let offset = 0; offset < indices.length; offset += 3) {
    const definition = selected.get(offset);
    if (definition) {
      nextIndices.set(definition.triangles, outputOffset);
      outputOffset += 6;
    } else {
      nextIndices.set(indices.subarray(offset, offset + 3), outputOffset);
      outputOffset += 3;
    }
  }
  return {
    indices: nextIndices,
    positions: nextPositions,
    normals: nextNormals,
    uv: nextUv,
    subdividedTriangles: deficit,
  };
}

function compactReferencedGeometry(indices, positions, normals, uv, IndexArray) {
  const sourceVertexCount = positions.length / 3;
  const remap = new Int32Array(sourceVertexCount);
  remap.fill(-1);
  const referenced = [];
  const nextIndices = new IndexArray(indices.length);
  for (let offset = 0; offset < indices.length; offset += 1) {
    const sourceVertex = indices[offset];
    if (sourceVertex >= sourceVertexCount) {
      throw new Error(`index ${sourceVertex} is outside the source vertex buffer`);
    }
    let outputVertex = remap[sourceVertex];
    if (outputVertex === -1) {
      outputVertex = referenced.length;
      if (IndexArray === Uint16Array && outputVertex > 65_535) {
        throw new Error("compacted vertex exceeds the existing UNSIGNED_SHORT index width");
      }
      remap[sourceVertex] = outputVertex;
      referenced.push(sourceVertex);
    }
    nextIndices[offset] = outputVertex;
  }
  const nextPositions = new Float32Array(referenced.length * 3);
  const nextNormals = new Float32Array(referenced.length * 3);
  const nextUv = new Float32Array(referenced.length * 2);
  for (let outputVertex = 0; outputVertex < referenced.length; outputVertex += 1) {
    const sourceVertex = referenced[outputVertex];
    nextPositions.set(
      positions.subarray(sourceVertex * 3, sourceVertex * 3 + 3),
      outputVertex * 3,
    );
    nextNormals.set(
      normals.subarray(sourceVertex * 3, sourceVertex * 3 + 3),
      outputVertex * 3,
    );
    nextUv.set(uv.subarray(sourceVertex * 2, sourceVertex * 2 + 2), outputVertex * 2);
  }
  return {
    indices: nextIndices,
    positions: nextPositions,
    normals: nextNormals,
    uv: nextUv,
    removedUnreferencedVertices: sourceVertexCount - referenced.length,
  };
}

function requireNonDegenerate(indices, positions) {
  for (let offset = 0; offset < indices.length; offset += 3) {
    const areaSquared = triangleCrossLengthSquared(
      positions,
      indices[offset],
      indices[offset + 1],
      indices[offset + 2],
    );
    if (!Number.isFinite(areaSquared) || areaSquared <= 1.0e-10) {
      throw new Error(`output triangle ${offset / 3} is degenerate`);
    }
  }
}

function bounds3(positions) {
  const min = [Infinity, Infinity, Infinity];
  const max = [-Infinity, -Infinity, -Infinity];
  for (let offset = 0; offset < positions.length; offset += 3) {
    for (let lane = 0; lane < 3; lane += 1) {
      min[lane] = Math.min(min[lane], positions[offset + lane]);
      max[lane] = Math.max(max[lane], positions[offset + lane]);
    }
  }
  return { min, max, size: max.map((value, lane) => value - min[lane]) };
}

function encodeGlb(json, bin) {
  const jsonBytes = Buffer.from(JSON.stringify(json), "utf8");
  const jsonPadding = (4 - (jsonBytes.length % 4)) % 4;
  const paddedJson = Buffer.concat([jsonBytes, Buffer.alloc(jsonPadding, 0x20)]);
  const binPadding = (4 - (bin.length % 4)) % 4;
  const paddedBin = Buffer.concat([bin, Buffer.alloc(binPadding)]);
  const output = Buffer.alloc(12 + 8 + paddedJson.length + 8 + paddedBin.length);
  output.write("glTF", 0, 4, "ascii");
  output.writeUInt32LE(2, 4);
  output.writeUInt32LE(output.length, 8);
  output.writeUInt32LE(paddedJson.length, 12);
  output.writeUInt32LE(GLB_JSON_CHUNK, 16);
  paddedJson.copy(output, 20);
  const binHeader = 20 + paddedJson.length;
  output.writeUInt32LE(paddedBin.length, binHeader);
  output.writeUInt32LE(GLB_BIN_CHUNK, binHeader + 4);
  paddedBin.copy(output, binHeader + 8);
  return output;
}

async function main() {
  const command = parseArguments(process.argv.slice(2));
  const source = await readFile(command.input);
  const { json, bin } = parseGlb(source);
  if (
    json.meshes?.length !== 1
    || json.meshes[0].primitives?.length !== 1
    || json.nodes?.length !== 1
    || (json.skins?.length ?? 0) !== 0
    || (json.animations?.length ?? 0) !== 0
  ) {
    throw new Error("exact-target profile requires one static mesh primitive and no rig or animation");
  }
  const primitive = json.meshes[0].primitives[0];
  if (
    (primitive.mode ?? 4) !== 4
    || Object.keys(primitive.attributes ?? {}).sort().join(",") !== "NORMAL,POSITION,TEXCOORD_0"
  ) {
    throw new Error("exact-target profile requires indexed POSITION/NORMAL/TEXCOORD_0 triangles");
  }
  const indexAccessor = json.accessors[primitive.indices];
  const positionAccessor = json.accessors[primitive.attributes.POSITION];
  const normalAccessor = json.accessors[primitive.attributes.NORMAL];
  const uvAccessor = json.accessors[primitive.attributes.TEXCOORD_0];
  const sourceIndices = typedAccessor(json, bin, primitive.indices);
  const sourcePositions = typedAccessor(json, bin, primitive.attributes.POSITION);
  const sourceNormals = typedAccessor(json, bin, primitive.attributes.NORMAL);
  const sourceUv = typedAccessor(json, bin, primitive.attributes.TEXCOORD_0);
  const sourceTriangles = sourceIndices.length / 3;
  if (sourceTriangles <= command.triangles) {
    throw new Error(`source must exceed target; got ${sourceTriangles} <= ${command.triangles}`);
  }

  await MeshoptSimplifier.ready;
  const attributes = new Float32Array(positionAccessor.count * 5);
  for (let vertex = 0; vertex < positionAccessor.count; vertex += 1) {
    attributes.set(sourceNormals.subarray(vertex * 3, vertex * 3 + 3), vertex * 5);
    attributes.set(sourceUv.subarray(vertex * 2, vertex * 2 + 2), vertex * 5 + 3);
  }
  const [simplified, simplificationError] = MeshoptSimplifier.simplifyWithAttributes(
    sourceIndices,
    sourcePositions,
    3,
    attributes,
    5,
    [0.25, 0.25, 0.25, 1, 1],
    null,
    command.triangles * 3,
    0.01,
    ["Permissive"],
  );
  if (simplified.length > command.triangles * 3) {
    throw new Error(
      `simplifier returned ${simplified.length / 3} triangles above the exact target`,
    );
  }
  const IndexArray = indexAccessor.componentType === 5123 ? Uint16Array : Uint32Array;
  const safe = retainAuroraSafeTriangles(simplified, sourcePositions);
  let geometry = compactReferencedGeometry(
    safe.indices,
    sourcePositions,
    sourceNormals,
    sourceUv,
    IndexArray,
  );
  geometry = {
    ...subdivideLargestTrianglesToTarget(
      geometry.indices,
      geometry.positions,
      geometry.normals,
      geometry.uv,
      command.triangles,
      IndexArray,
    ),
    removedUnreferencedVertices: geometry.removedUnreferencedVertices,
    removedAuroraUnsafeTriangles: safe.removed,
  };
  const sourceBounds = bounds3(sourcePositions);
  let uniformScale = 1;
  if (command.height !== undefined) {
    if (!Number.isFinite(sourceBounds.size[1]) || sourceBounds.size[1] <= Number.EPSILON) {
      throw new Error("source height is invalid");
    }
    uniformScale = command.height / sourceBounds.size[1];
    for (let index = 0; index < geometry.positions.length; index += 1) {
      geometry.positions[index] *= uniformScale;
    }
  }
  const outputBounds = bounds3(geometry.positions);
  if (geometry.indices.length !== command.triangles * 3) {
    throw new Error("exact triangle target was not reached");
  }
  if (indexAccessor.componentType === 5123 && geometry.positions.length / 3 > 65_535) {
    throw new Error("new vertex exceeds the existing UNSIGNED_SHORT index width");
  }
  requireNonDegenerate(geometry.indices, geometry.positions);

  const parts = [Buffer.from(bin)];
  let length = bin.length;
  const positionAppend = appendAligned(parts, geometry.positions, length);
  length = positionAppend.nextLength;
  const normalAppend = appendAligned(parts, geometry.normals, length);
  length = normalAppend.nextLength;
  const uvAppend = appendAligned(parts, geometry.uv, length);
  length = uvAppend.nextLength;
  const indexAppend = appendAligned(parts, geometry.indices, length);
  length = indexAppend.nextLength;
  const outputBin = Buffer.concat(parts);
  if (outputBin.length !== length) throw new Error("internal BIN length mismatch");

  for (const [accessor, append, array] of [
    [positionAccessor, positionAppend, geometry.positions],
    [normalAccessor, normalAppend, geometry.normals],
    [uvAccessor, uvAppend, geometry.uv],
    [indexAccessor, indexAppend, geometry.indices],
  ]) {
    const view = json.bufferViews[accessor.bufferView];
    view.byteOffset = append.offset;
    view.byteLength = array.byteLength;
    delete view.byteStride;
    accessor.byteOffset = 0;
  }
  positionAccessor.count = geometry.positions.length / 3;
  positionAccessor.min = outputBounds.min;
  positionAccessor.max = outputBounds.max;
  normalAccessor.count = geometry.normals.length / 3;
  uvAccessor.count = geometry.uv.length / 2;
  indexAccessor.count = geometry.indices.length;
  json.buffers[0].byteLength = outputBin.length;
  const output = encodeGlb(json, outputBin);
  const final = parseGlb(output);
  const finalIndices = typedAccessor(final.json, final.bin, primitive.indices);
  const finalPositions = typedAccessor(final.json, final.bin, primitive.attributes.POSITION);
  requireNonDegenerate(finalIndices, finalPositions);
  if (finalIndices.length / 3 !== command.triangles) {
    throw new Error("encoded GLB readback differs from the exact triangle target");
  }

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
    outputTriangles: finalIndices.length / 3,
    sourceVertices: sourcePositions.length / 3,
    outputVertices: finalPositions.length / 3,
    removedUnreferencedVertices: geometry.removedUnreferencedVertices,
    removedAuroraUnsafeTriangles: geometry.removedAuroraUnsafeTriangles,
    subdividedTriangles: geometry.subdividedTriangles,
    sourceBounds,
    outputBounds,
    uniformScale,
    simplificationError,
    exactTarget: true,
    nonDegenerateReadback: true,
    preservesMeshyMaterialsAndEmbeddedImages: true,
  }, null, 2)}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
