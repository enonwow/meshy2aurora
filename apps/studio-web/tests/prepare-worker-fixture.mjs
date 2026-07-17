import { rmSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const testsDirectory = dirname(fileURLToPath(import.meta.url));
const generatedDirectory = resolve(testsDirectory, ".generated");
const expectedGeneratedDirectory = join(testsDirectory, ".generated");

if (generatedDirectory !== expectedGeneratedDirectory) {
  throw new Error(`refusing to clear unexpected fixture path: ${generatedDirectory}`);
}

rmSync(generatedDirectory, { force: true, recursive: true });

const cargo = process.platform === "win32" ? "cargo.exe" : "cargo";
const result = spawnSync(cargo, [
  "run",
  "--quiet",
  "--manifest-path",
  "../../Cargo.toml",
  "-p",
  "m2a-core",
  "--example",
  "materialize_m6",
  "--",
  "--synthetic-owned-h1",
  "--appearance-2da",
  "tests/fixtures/appearance.2da",
  "--output-dir",
  "tests/.generated/owned-package",
], {
  cwd: resolve(testsDirectory, ".."),
  encoding: "utf8",
  stdio: ["ignore", "pipe", "pipe"],
});

if (result.status !== 0) {
  process.stderr.write(result.stderr);
  throw new Error(`owned fixture generator exited with status ${result.status}`);
}

process.stdout.write("generated repo-owned synthetic Studio integration fixture\n");
