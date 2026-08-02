import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const inputIndex = process.argv.indexOf("--input");
const input = inputIndex >= 0 ? process.argv[inputIndex + 1] : undefined;
if (!input) throw new Error("Usage: node scripts/import-animation-contribution.mjs --input <contribution.json>");

const result = spawnSync("cargo", [
  "run",
  "--quiet",
  "-p",
  "m2a-core",
  "--example",
  "community_animation_library",
  "--",
  "install-contribution",
  "--input",
  path.resolve(input),
  "--root",
  path.join(repositoryRoot, "animation-library"),
], {
  cwd: repositoryRoot,
  encoding: "utf8",
  stdio: "inherit",
});
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
