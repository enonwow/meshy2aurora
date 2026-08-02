import { copyFileSync, mkdirSync, rmSync } from "node:fs";
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
for (const [sourceArguments, outputDirectory] of [
  [["--synthetic-owned-h1"], "tests/.generated/owned-package"],
  [["--synthetic-owned-h1-full-42"], "tests/.generated/owned-full42-package"],
  [[
    "--synthetic-owned-h1-full-42-rig-profile",
    "../../animation-library/rig-profiles/m2a-humanoid-strict-v1.json",
  ], "tests/.generated/owned-library-humanoid-full42-package"],
]) {
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
    ...sourceArguments,
    "--appearance-2da",
    "tests/fixtures/appearance.2da",
    "--output-dir",
    outputDirectory,
  ], {
    cwd: resolve(testsDirectory, ".."),
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });

  if (result.status !== 0) {
    process.stderr.write(result.stderr);
    throw new Error(`owned fixture generator exited with status ${result.status}`);
  }
}

const p100kFixtureDirectory = join(generatedDirectory, "p100k-studio-replay");
mkdirSync(p100kFixtureDirectory, { recursive: true });
const requireP100kReplay = process.env.VITE_M2A_RUN_P100K_STUDIO_REPLAY === "1";
copyFileSync(
  requireP100kReplay
    ? resolve(testsDirectory, "../../../sample-3d/tlc-veiled-humanoid-h1-p100k-v1/source.glb")
    : join(generatedDirectory, "owned-package/generated/source.glb"),
  join(p100kFixtureDirectory, "source.glb"),
);
copyFileSync(
  requireP100kReplay
    ? resolve(testsDirectory, "../../../local-reference-assets/appearance.2da")
    : resolve(testsDirectory, "fixtures/appearance.2da"),
  join(p100kFixtureDirectory, "appearance.2da"),
);

const p300kFixtureDirectory = join(generatedDirectory, "p300k-studio-replay");
mkdirSync(p300kFixtureDirectory, { recursive: true });
const requireP300kReplay = process.env.VITE_M2A_RUN_P300K_STUDIO_REPLAY === "1";
copyFileSync(
  requireP300kReplay
    ? resolve(testsDirectory, "../../../sample-3d/tlc-stoneback-brute-h1-p300k-v1/source.glb")
    : join(generatedDirectory, "owned-package/generated/source.glb"),
  join(p300kFixtureDirectory, "source.glb"),
);
copyFileSync(
  requireP300kReplay
    ? resolve(testsDirectory, "../../../local-reference-assets/appearance.2da")
    : resolve(testsDirectory, "fixtures/appearance.2da"),
  join(p300kFixtureDirectory, "appearance.2da"),
);

process.stdout.write("generated repo-owned synthetic Studio integration fixtures\n");
