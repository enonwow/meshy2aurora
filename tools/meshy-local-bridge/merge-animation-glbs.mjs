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

function requireMatchingRig(base, donor, label) {
  const baseNodes = base.nodes ?? [];
  const donorNodes = donor.nodes ?? [];
  if (baseNodes.length !== donorNodes.length) fail(`${label} node count differs from the base animation GLB`);
  for (let index = 0; index < baseNodes.length; index += 1) {
    if (nodeSignature(baseNodes[index]) !== nodeSignature(donorNodes[index])) {
      fail(`${label} node ${index} differs from the base rig`);
    }
  }
  const baseSkins = (base.skins ?? []).map((skin) => JSON.stringify({ joints: skin.joints, skeleton: skin.skeleton ?? null }));
  const donorSkins = (donor.skins ?? []).map((skin) => JSON.stringify({ joints: skin.joints, skeleton: skin.skeleton ?? null }));
  if (JSON.stringify(baseSkins) !== JSON.stringify(donorSkins)) fail(`${label} skin joint topology differs from the base animation GLB`);
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
    requireMatchingRig(base.json, donorJson, label);
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
