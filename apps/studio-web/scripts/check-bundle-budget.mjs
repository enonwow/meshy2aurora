import { readdir, readFile } from "node:fs/promises";
import { basename, extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const assetsDirectory = new URL("../dist/assets/", import.meta.url);
const assetsPath = fileURLToPath(assetsDirectory);
const entries = await readdir(assetsDirectory, { withFileTypes: true });
const files = await Promise.all(entries
  .filter((entry) => entry.isFile())
  .map(async (entry) => {
    const bytes = await readFile(join(assetsPath, entry.name));
    return {
      name: entry.name,
      extension: extname(entry.name),
      bytes: bytes.byteLength,
      gzipBytes: gzipSync(bytes).byteLength,
    };
  }));

const js = files.filter(({ extension }) => extension === ".js");
const css = files.filter(({ extension }) => extension === ".css");
const wasm = files.filter(({ extension }) => extension === ".wasm");
const main = js.filter(({ name }) => /^index-[^.]+\.js$/.test(name));

const budgets = {
  mainJsBytes: 460_000,
  mainJsGzipBytes: 130_000,
  maximumJsChunkBytes: 620_000,
  totalJsBytes: 1_450_000,
  totalCssBytes: 150_000,
  wasmBytes: 4_000_000,
  wasmGzipBytes: 1_420_000,
};

const sum = (items, field) => items.reduce((total, item) => total + item[field], 0);
const maximum = (items, field) => Math.max(0, ...items.map((item) => item[field]));
const measured = {
  mainJsBytes: sum(main, "bytes"),
  mainJsGzipBytes: sum(main, "gzipBytes"),
  maximumJsChunkBytes: maximum(js, "bytes"),
  totalJsBytes: sum(js, "bytes"),
  totalCssBytes: sum(css, "bytes"),
  wasmBytes: sum(wasm, "bytes"),
  wasmGzipBytes: sum(wasm, "gzipBytes"),
};

if (main.length !== 1 || wasm.length !== 1) {
  throw new Error("Bundle budget requires exactly one initial index JS chunk and one WASM binary");
}
for (const prefix of [
  "AnimationStudioWorkspace-",
  "PlaceableAuthoringEditor-",
  "MeshyLab-",
  "TileReview-",
]) {
  if (!files.some(({ name }) => name.startsWith(prefix))) {
    throw new Error(`Required lazy feature chunk is missing: ${prefix}`);
  }
}

const violations = Object.entries(budgets)
  .filter(([key, limit]) => measured[key] > limit)
  .map(([key, limit]) => `${key}: ${measured[key]} > ${limit}`);
if (violations.length) {
  throw new Error(`Studio bundle budget exceeded:\n${violations.join("\n")}`);
}

console.log(JSON.stringify({
  schemaVersion: 1,
  status: "PASS",
  artifactRoot: basename(fileURLToPath(new URL("../dist/", import.meta.url))),
  measured,
  budgets,
}, null, 2));
