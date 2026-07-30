import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const GLB_MAGIC = 0x46546c67;
const JSON_CHUNK = 0x4e4f534a;
const BIN_CHUNK = 0x004e4942;

function fail(message) {
  throw new Error(`MESHY-ANIMATION-MERGE: ${message}`);
}

function parseGlb(bytes, label) {
  const buffer = Buffer.from(bytes);
  if (buffer.length < 20 || buffer.readUInt32LE(0) !== GLB_MAGIC || buffer.readUInt32LE(4) !== 2) {
    fail(`${label} is not a GLB 2.0 file`);
  }
  if (buffer.readUInt32LE(8) !== buffer.length) fail(`${label} has a mismatched declared length`);
  const jsonLength = buffer.readUInt32LE(12);
  if (buffer.readUInt32LE(16) !== JSON_CHUNK || 20 + jsonLength + 8 > buffer.length) {
    fail(`${label} does not begin with a bounded JSON chunk`);
  }
  let json;
  try {
    json = JSON.parse(buffer.toString("utf8", 20, 20 + jsonLength).trimEnd());
  } catch {
    fail(`${label} contains invalid glTF JSON`);
  }
  const binHeader = 20 + jsonLength;
  const binLength = buffer.readUInt32LE(binHeader);
  if (buffer.readUInt32LE(binHeader + 4) !== BIN_CHUNK || binHeader + 8 + binLength !== buffer.length) {
    fail(`${label} must contain one terminal BIN chunk`);
  }
  if (!Array.isArray(json.buffers) || json.buffers.length !== 1 || json.buffers[0].uri !== undefined) {
    fail(`${label} must use exactly one embedded buffer`);
  }
  const declaredBinLength = json.buffers[0].byteLength;
  if (!Number.isInteger(declaredBinLength) || declaredBinLength < 0 || declaredBinLength > binLength || binLength - declaredBinLength > 3) {
    fail(`${label} declares an invalid embedded buffer length`);
  }
  return { json, bin: buffer.subarray(binHeader + 8, binHeader + 8 + declaredBinLength) };
}

export function inspectGlbTriangleCount(bytes, label = "GLB") {
  const parsed = parseGlb(bytes, label);
  let triangleCount = 0;
  for (let meshIndex = 0; meshIndex < (parsed.json.meshes ?? []).length; meshIndex += 1) {
    const primitives = parsed.json.meshes[meshIndex].primitives ?? [];
    for (let primitiveIndex = 0; primitiveIndex < primitives.length; primitiveIndex += 1) {
      const primitive = primitives[primitiveIndex];
      const path = `meshes[${meshIndex}].primitives[${primitiveIndex}]`;
      if ((primitive.mode ?? 4) !== 4) fail(`${label} ${path} is not a triangle list`);
      const accessorIndex = primitive.indices ?? primitive.attributes?.POSITION;
      if (!Number.isInteger(accessorIndex)) fail(`${label} ${path} has no indices or POSITION accessor`);
      accessorDigest(parsed, accessorIndex, label, primitive.indices === undefined ? `${path}.attributes.POSITION` : `${path}.indices`);
      const accessor = parsed.json.accessors[accessorIndex];
      if (accessor.count % 3 !== 0) fail(`${label} ${path} triangle-list accessor count is not divisible by three`);
      triangleCount += accessor.count / 3;
      if (!Number.isSafeInteger(triangleCount)) fail(`${label} triangle count exceeds the safe integer range`);
    }
  }
  return triangleCount;
}

function nodeSignature(node) {
  return JSON.stringify({
    name: node.name ?? null,
    children: node.children ?? [],
    mesh: node.mesh ?? null,
    skin: node.skin ?? null,
    matrix: node.matrix ?? null,
    translation: node.translation ?? null,
    rotation: node.rotation ?? null,
    scale: node.scale ?? null,
  });
}

const ACCESSOR_COMPONENT_BYTES = new Map([
  [5120, 1],
  [5121, 1],
  [5122, 2],
  [5123, 2],
  [5125, 4],
  [5126, 4],
]);
const ACCESSOR_TYPE_COMPONENTS = new Map([
  ["SCALAR", 1],
  ["VEC2", 2],
  ["VEC3", 3],
  ["VEC4", 4],
  ["MAT2", 4],
  ["MAT3", 9],
  ["MAT4", 16],
]);

function accessorDigest(parsed, accessorIndex, label, path) {
  const accessor = parsed.json.accessors?.[accessorIndex];
  if (!accessor || !Number.isInteger(accessorIndex) || accessorIndex < 0) {
    fail(`${label} ${path} references missing accessor ${accessorIndex}`);
  }
  if (accessor.sparse !== undefined) {
    fail(`${label} ${path} uses a sparse accessor, which the strict rig-identity check does not accept`);
  }
  const componentBytes = ACCESSOR_COMPONENT_BYTES.get(accessor.componentType);
  const componentCount = ACCESSOR_TYPE_COMPONENTS.get(accessor.type);
  if (!componentBytes || !componentCount || !Number.isInteger(accessor.count) || accessor.count < 0) {
    fail(`${label} ${path} has invalid accessor metadata`);
  }
  if (!Number.isInteger(accessor.bufferView) || accessor.bufferView < 0) {
    fail(`${label} ${path} must use a dense embedded bufferView`);
  }
  const view = parsed.json.bufferViews?.[accessor.bufferView];
  if (!view || view.buffer !== 0 || !Number.isInteger(view.byteLength) || view.byteLength < 0) {
    fail(`${label} ${path} references an invalid bufferView ${accessor.bufferView}`);
  }
  const viewStart = view.byteOffset ?? 0;
  const accessorOffset = accessor.byteOffset ?? 0;
  const elementBytes = componentBytes * componentCount;
  const stride = view.byteStride ?? elementBytes;
  if (
    !Number.isInteger(viewStart)
    || viewStart < 0
    || !Number.isInteger(accessorOffset)
    || accessorOffset < 0
    || !Number.isInteger(stride)
    || stride < elementBytes
  ) {
    fail(`${label} ${path} has an invalid byte range or stride`);
  }
  const first = viewStart + accessorOffset;
  const last = accessor.count === 0
    ? first
    : first + (accessor.count - 1) * stride + elementBytes;
  if (first < viewStart || last > viewStart + view.byteLength || last > parsed.bin.length) {
    fail(`${label} ${path} exceeds its embedded bufferView`);
  }
  const hash = createHash("sha256");
  hash.update(JSON.stringify({
    componentType: accessor.componentType,
    count: accessor.count,
    type: accessor.type,
    normalized: accessor.normalized === true,
  }));
  for (let element = 0; element < accessor.count; element += 1) {
    const start = first + element * stride;
    hash.update(parsed.bin.subarray(start, start + elementBytes));
  }
  return hash.digest("hex");
}

function requireAccessorMatch(base, donor, baseIndex, donorIndex, label, path) {
  const baseDigest = accessorDigest(base, baseIndex, "base animation GLB", path);
  const donorDigest = accessorDigest(donor, donorIndex, label, path);
  if (baseDigest !== donorDigest) fail(`${label} ${path} differs from the base animation GLB`);
}

function requireMatchingMeshes(base, donor, label) {
  const baseMeshes = base.json.meshes ?? [];
  const donorMeshes = donor.json.meshes ?? [];
  if (baseMeshes.length !== donorMeshes.length) fail(`${label} mesh count differs from the base animation GLB`);
  for (let meshIndex = 0; meshIndex < baseMeshes.length; meshIndex += 1) {
    const basePrimitives = baseMeshes[meshIndex].primitives ?? [];
    const donorPrimitives = donorMeshes[meshIndex].primitives ?? [];
    if (basePrimitives.length !== donorPrimitives.length) {
      fail(`${label} mesh ${meshIndex} primitive count differs from the base animation GLB`);
    }
    for (let primitiveIndex = 0; primitiveIndex < basePrimitives.length; primitiveIndex += 1) {
      const path = `meshes[${meshIndex}].primitives[${primitiveIndex}]`;
      const basePrimitive = basePrimitives[primitiveIndex];
      const donorPrimitive = donorPrimitives[primitiveIndex];
      if (
        (basePrimitive.mode ?? 4) !== (donorPrimitive.mode ?? 4)
        || (basePrimitive.material ?? null) !== (donorPrimitive.material ?? null)
      ) {
        fail(`${label} ${path} topology or material differs from the base animation GLB`);
      }
      const baseAttributes = basePrimitive.attributes ?? {};
      const donorAttributes = donorPrimitive.attributes ?? {};
      const attributeNames = Object.keys(baseAttributes).sort();
      if (JSON.stringify(attributeNames) !== JSON.stringify(Object.keys(donorAttributes).sort())) {
        fail(`${label} ${path} vertex attributes differ from the base animation GLB`);
      }
      for (const attributeName of attributeNames) {
        requireAccessorMatch(
          base,
          donor,
          baseAttributes[attributeName],
          donorAttributes[attributeName],
          label,
          `${path}.attributes.${attributeName}`,
        );
      }
      const baseHasIndices = basePrimitive.indices !== undefined;
      const donorHasIndices = donorPrimitive.indices !== undefined;
      if (baseHasIndices !== donorHasIndices) {
        fail(`${label} ${path}.indices differs from the base animation GLB`);
      }
      if (baseHasIndices) {
        requireAccessorMatch(
          base,
          donor,
          basePrimitive.indices,
          donorPrimitive.indices,
          label,
          `${path}.indices`,
        );
      }
      const baseTargets = basePrimitive.targets ?? [];
      const donorTargets = donorPrimitive.targets ?? [];
      if (baseTargets.length !== donorTargets.length) {
        fail(`${label} ${path}.targets differs from the base animation GLB`);
      }
      for (let targetIndex = 0; targetIndex < baseTargets.length; targetIndex += 1) {
        const baseTarget = baseTargets[targetIndex];
        const donorTarget = donorTargets[targetIndex] ?? {};
        const targetNames = Object.keys(baseTarget).sort();
        if (JSON.stringify(targetNames) !== JSON.stringify(Object.keys(donorTarget).sort())) {
          fail(`${label} ${path}.targets[${targetIndex}] differs from the base animation GLB`);
        }
        for (const attributeName of targetNames) {
          requireAccessorMatch(
            base,
            donor,
            baseTarget[attributeName],
            donorTarget[attributeName],
            label,
            `${path}.targets[${targetIndex}].${attributeName}`,
          );
        }
      }
    }
  }
}

function requireMatchingRig(base, donor, label) {
  const baseNodes = base.json.nodes ?? [];
  const donorNodes = donor.json.nodes ?? [];
  if (baseNodes.length !== donorNodes.length) fail(`${label} node count differs from the base animation GLB`);
  for (let index = 0; index < baseNodes.length; index += 1) {
    if (nodeSignature(baseNodes[index]) !== nodeSignature(donorNodes[index])) {
      fail(`${label} node ${index} differs from the base rig`);
    }
  }
  const baseSkins = base.json.skins ?? [];
  const donorSkins = donor.json.skins ?? [];
  if (baseSkins.length !== donorSkins.length) fail(`${label} skin count differs from the base animation GLB`);
  for (let skinIndex = 0; skinIndex < baseSkins.length; skinIndex += 1) {
    const baseSkin = baseSkins[skinIndex];
    const donorSkin = donorSkins[skinIndex];
    if (
      JSON.stringify(baseSkin.joints ?? []) !== JSON.stringify(donorSkin.joints ?? [])
      || (baseSkin.skeleton ?? null) !== (donorSkin.skeleton ?? null)
    ) {
      fail(`${label} skin ${skinIndex} joint topology differs from the base animation GLB`);
    }
    const baseHasInverseBind = baseSkin.inverseBindMatrices !== undefined;
    const donorHasInverseBind = donorSkin.inverseBindMatrices !== undefined;
    if (baseHasInverseBind !== donorHasInverseBind) {
      fail(`${label} skins[${skinIndex}].inverseBindMatrices differs from the base animation GLB`);
    }
    if (baseHasInverseBind) {
      requireAccessorMatch(
        base,
        donor,
        baseSkin.inverseBindMatrices,
        donorSkin.inverseBindMatrices,
        label,
        `skins[${skinIndex}].inverseBindMatrices`,
      );
    }
  }
  requireMatchingMeshes(base, donor, label);
}

function requireSingleAnimation(document, label) {
  if (!Array.isArray(document.animations) || document.animations.length !== 1) {
    fail(`${label} must contain exactly one animation`);
  }
  const animation = document.animations[0];
  if (!Array.isArray(animation.samplers) || !Array.isArray(animation.channels) || !animation.samplers.length || !animation.channels.length) {
    fail(`${label} animation must contain samplers and channels`);
  }
  return animation;
}

function encodeGlb(document, bin) {
  document.buffers[0].byteLength = bin.length;
  const rawJson = Buffer.from(JSON.stringify(document), "utf8");
  const jsonPadding = (4 - (rawJson.length % 4)) % 4;
  const json = Buffer.concat([rawJson, Buffer.alloc(jsonPadding, 0x20)]);
  const binPadding = (4 - (bin.length % 4)) % 4;
  const paddedBin = Buffer.concat([bin, Buffer.alloc(binPadding)]);
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

export function mergeMeshyAnimationGlbs(inputs) {
  if (!Array.isArray(inputs) || inputs.length < 1 || inputs.length > 10) {
    fail("provide between one and ten named animation GLBs");
  }
  const foldedNames = new Set();
  for (const input of inputs) {
    if (!input || typeof input.clipName !== "string" || !/^[a-z][a-z0-9_]{0,15}$/i.test(input.clipName)) {
      fail("every output clip name must be a 1-16 character ASCII resref-style name");
    }
    const folded = input.clipName.toLowerCase();
    if (foldedNames.has(folded)) fail(`duplicate output clip name ${input.clipName}`);
    foldedNames.add(folded);
  }

  const parsed = inputs.map((input, index) => ({
    ...input,
    parsed: parseGlb(input.bytes, input.label ?? `input[${index}]`),
  }));
  const base = parsed[0].parsed;
  requireSingleAnimation(base.json, parsed[0].label ?? "input[0]");
  const outputJson = structuredClone(base.json);
  outputJson.animations = [];
  outputJson.bufferViews ??= [];
  outputJson.accessors ??= [];
  let outputBin = Buffer.from(base.bin);

  for (let inputIndex = 0; inputIndex < parsed.length; inputIndex += 1) {
    const input = parsed[inputIndex];
    const { json: donorJson, bin: donorBin } = input.parsed;
    const label = input.label ?? `input[${inputIndex}]`;
    requireMatchingRig(base, input.parsed, label);
    const donorAnimation = requireSingleAnimation(donorJson, label);
    const viewMap = new Map();
    const accessorMap = new Map();

    const copyView = (viewIndex) => {
      if (viewMap.has(viewIndex)) return viewMap.get(viewIndex);
      const view = donorJson.bufferViews?.[viewIndex];
      if (!view || view.buffer !== 0 || !Number.isInteger(view.byteLength) || view.byteLength < 1) {
        fail(`${label} animation references an invalid bufferView ${viewIndex}`);
      }
      const start = view.byteOffset ?? 0;
      const end = start + view.byteLength;
      if (!Number.isInteger(start) || start < 0 || end > donorBin.length) {
        fail(`${label} bufferView ${viewIndex} exceeds the embedded BIN payload`);
      }
      const padding = (4 - (outputBin.length % 4)) % 4;
      const byteOffset = outputBin.length + padding;
      outputBin = Buffer.concat([outputBin, Buffer.alloc(padding), donorBin.subarray(start, end)]);
      const outputViewIndex = outputJson.bufferViews.length;
      outputJson.bufferViews.push({ ...structuredClone(view), buffer: 0, byteOffset });
      viewMap.set(viewIndex, outputViewIndex);
      return outputViewIndex;
    };

    const copyAccessor = (accessorIndex) => {
      if (accessorMap.has(accessorIndex)) return accessorMap.get(accessorIndex);
      const accessor = donorJson.accessors?.[accessorIndex];
      if (!accessor) fail(`${label} animation references missing accessor ${accessorIndex}`);
      const outputAccessor = structuredClone(accessor);
      if (outputAccessor.bufferView !== undefined) outputAccessor.bufferView = copyView(outputAccessor.bufferView);
      if (outputAccessor.sparse) {
        outputAccessor.sparse.indices.bufferView = copyView(outputAccessor.sparse.indices.bufferView);
        outputAccessor.sparse.values.bufferView = copyView(outputAccessor.sparse.values.bufferView);
      }
      const outputAccessorIndex = outputJson.accessors.length;
      outputJson.accessors.push(outputAccessor);
      accessorMap.set(accessorIndex, outputAccessorIndex);
      return outputAccessorIndex;
    };

    const outputAnimation = structuredClone(donorAnimation);
    outputAnimation.name = input.clipName;
    outputAnimation.samplers = outputAnimation.samplers.map((sampler) => ({
      ...sampler,
      input: copyAccessor(sampler.input),
      output: copyAccessor(sampler.output),
    }));
    for (const [channelIndex, channel] of outputAnimation.channels.entries()) {
      if (!Number.isInteger(channel.sampler) || channel.sampler < 0 || channel.sampler >= outputAnimation.samplers.length) {
        fail(`${label} animation channel ${channelIndex} references an invalid sampler`);
      }
      if (!channel.target || !Number.isInteger(channel.target.node) || channel.target.node < 0 || channel.target.node >= (donorJson.nodes ?? []).length) {
        fail(`${label} animation channel ${channelIndex} references an invalid target node`);
      }
    }
    outputJson.animations.push(outputAnimation);
  }

  return encodeGlb(outputJson, outputBin);
}

function parseArguments(argv) {
  let outputPath;
  const inputs = [];
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === "--out") outputPath = argv[++index];
    else if (argument === "--input") {
      const value = argv[++index] ?? "";
      const separator = value.indexOf("=");
      if (separator <= 0 || separator === value.length - 1) fail("--input must use clipName=path");
      inputs.push({ clipName: value.slice(0, separator), path: value.slice(separator + 1) });
    } else fail(`unknown argument ${argument}`);
  }
  if (!outputPath || !inputs.length) fail("usage: --out <combined.glb> --input cpause1=<idle.glb> [--input cwalk=<walk.glb> ...]");
  return { outputPath, inputs };
}

async function main() {
  const args = parseArguments(process.argv.slice(2));
  const inputs = await Promise.all(args.inputs.map(async (input) => ({
    clipName: input.clipName,
    label: resolve(input.path),
    bytes: await readFile(resolve(input.path)),
  })));
  const output = mergeMeshyAnimationGlbs(inputs);
  const absoluteOutput = resolve(args.outputPath);
  await mkdir(dirname(absoluteOutput), { recursive: true });
  await writeFile(absoluteOutput, output, { flag: "wx" });
  process.stdout.write(`${JSON.stringify({
    outputPath: absoluteOutput,
    byteLength: output.length,
    sha256: createHash("sha256").update(output).digest("hex"),
    clips: inputs.map((input) => input.clipName),
  }, null, 2)}\n`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  main().catch((error) => {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  });
}
