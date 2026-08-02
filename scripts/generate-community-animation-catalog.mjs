import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "..");
const check = process.argv.includes("--check");
const explicitWrite = process.argv.includes("--write");
const args = [
  "run",
  "--quiet",
  "-p",
  "m2a-core",
  "--example",
  "community_animation_library",
  "--",
  "catalog",
  "--root",
  path.join(repositoryRoot, "animation-library"),
  "--output",
  path.join(repositoryRoot, "contracts", "community-animation-catalog-v1.json"),
  check ? "--check" : "--write",
];

if (check && explicitWrite) {
  throw new Error("Choose only --check or --write.");
}

const result = spawnSync("cargo", args, {
  cwd: repositoryRoot,
  encoding: "utf8",
  stdio: "inherit",
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
