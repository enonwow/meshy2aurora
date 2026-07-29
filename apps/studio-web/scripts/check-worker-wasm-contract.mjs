import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const [workerSource, sharedTypes, generatedWasmTypes] = await Promise.all([
  readFile(new URL("../src/worker/m2a.worker.ts", import.meta.url), "utf8"),
  readFile(new URL("../src/worker/types.ts", import.meta.url), "utf8"),
  readFile(
    new URL("../../../crates/m2a-wasm/pkg/m2a_wasm.d.ts", import.meta.url),
    "utf8",
  ),
]);

const requestSection = sharedTypes.match(
  /export type StudioWorkerRequest =([\s\S]*?)export interface WorkerArtifact/,
)?.[1];
if (!requestSection) {
  throw new Error("StudioWorkerRequest shared contract is missing.");
}

const expectedRequests = [
  "INITIALIZE",
  "INSPECT_SOURCE",
  "INSPECT_APPEARANCE",
  "VALIDATE_CREATURE_ANIMATION_MAPPING",
  "INSPECT_EDITABLE_ANIMATION_SOURCE",
  "VALIDATE_ANIMATION_STUDIO_DOCUMENT",
  "MATERIALIZE_ANIMATION_STUDIO_DOCUMENT",
  "PREVIEW_AUTHORED_ANIMATION_CLIP",
  "BUILD_MODEL_PACKAGE",
  "BUILD_PLACEABLE_PACKAGE",
  "BUILD_TILE_PACKAGE",
  "VALIDATE_M7_CORPUS",
  "INSPECT_M7_CORPUS_INTAKE",
  "BUILD_M7_CORPUS_BATCH",
].sort();
const requestTypes = [
  ...new Set(
    [...requestSection.matchAll(/type:\s*"([A-Z0-9_]+)"/g)]
      .map((match) => match[1]),
  ),
].sort();

const workerCases = [
  ...new Set(
    [...workerSource.matchAll(/request\.type\s*===\s*"([A-Z0-9_]+)"/g)]
      .map((match) => match[1]),
  ),
];
if (
  /switch\s*\(request\.packageLane\)/.test(workerSource)
  && /const unsupported:\s*never\s*=\s*request;/.test(workerSource)
) {
  workerCases.push("BUILD_MODEL_PACKAGE");
}
workerCases.sort();

const wasmImport = workerSource.match(
  /import init,\s*\{([\s\S]*?)\}\s*from "@m2a-wasm";/,
)?.[1];
if (!wasmImport) {
  throw new Error("The Worker does not use the generated typed WASM boundary.");
}
const wasmImports = wasmImport
  .split(",")
  .map((value) => value.trim())
  .filter(Boolean)
  .sort();
const generatedExports = new Set(
  [...generatedWasmTypes.matchAll(/export function\s+([A-Za-z0-9_]+)/g)]
    .map((match) => match[1]),
);
const missingWasmExports = wasmImports.filter(
  (name) => !generatedExports.has(name),
);

const failures = [];
if (JSON.stringify(requestTypes) !== JSON.stringify(expectedRequests)) {
  failures.push(
    `shared request discriminants differ:\nexpected ${expectedRequests.join(", ")}\nactual   ${requestTypes.join(", ")}`,
  );
}
if (JSON.stringify(workerCases) !== JSON.stringify(expectedRequests)) {
  failures.push(
    `Worker cases differ:\nexpected ${expectedRequests.join(", ")}\nactual   ${workerCases.join(", ")}`,
  );
}
if (missingWasmExports.length > 0) {
  failures.push(
    `generated m2a_wasm.d.ts misses Worker imports: ${missingWasmExports.join(", ")}`,
  );
}
if (!/export default function __wbg_init\b/.test(generatedWasmTypes)) {
  failures.push("generated m2a_wasm.d.ts misses the typed default initializer");
}
if (failures.length > 0) {
  throw new Error(
    `Worker/WASM contract gate failed in ${root}\n${failures.join("\n")}`,
  );
}

console.log(JSON.stringify({
  schemaVersion: 1,
  status: "PASS",
  sharedRequestCount: requestTypes.length,
  exhaustiveWorkerCaseCount: workerCases.length,
  generatedWasmImportCount: wasmImports.length + 1,
}, null, 2));
