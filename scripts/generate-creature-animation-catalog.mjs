import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const contractPath = resolve(
  repositoryRoot,
  "contracts",
  "creature-animation-catalog-v1.json",
);
const result = spawnSync(
  process.platform === "win32" ? "cargo.exe" : "cargo",
  [
    "run",
    "--quiet",
    "-p",
    "m2a-core",
    "--example",
    "export_creature_animation_catalog_v1",
  ],
  {
    cwd: repositoryRoot,
    encoding: "utf8",
  },
);

if (result.status !== 0) {
  process.stderr.write(result.stderr || "Core catalog generation failed.\n");
  process.exit(result.status ?? 1);
}

const generated = `${result.stdout.trim()}\n`;
if (process.argv.includes("--check")) {
  const tracked = readFileSync(contractPath, "utf8").replace(/\r\n/g, "\n");
  if (tracked !== generated) {
    process.stderr.write(
      "contracts/creature-animation-catalog-v1.json is stale. Run npm run catalog:generate.\n",
    );
    process.exit(1);
  }
  process.stdout.write("creature-animation-catalog-v1: generated contract is current\n");
} else {
  writeFileSync(contractPath, generated, "utf8");
  process.stdout.write(`generated ${contractPath}\n`);
}
