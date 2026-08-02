import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const catalogPath = "contracts/community-animation-catalog-v1.json";
const current = JSON.parse(readFileSync(path.join(repositoryRoot, catalogPath), "utf8"));
const baseRef = process.env.GITHUB_BASE_REF?.trim();
if (!baseRef) {
  console.log("animation-library-immutability-ok: local run has no GitHub base ref");
  process.exit(0);
}

let previous;
try {
  const json = execFileSync("git", ["show", `origin/${baseRef}:${catalogPath}`], {
    cwd: repositoryRoot,
    encoding: "utf8",
  });
  previous = JSON.parse(json);
} catch (error) {
  throw new Error(`Cannot read base animation catalog from origin/${baseRef}: ${error}`);
}

const identity = (entry) => `${entry.presetId}@${entry.presetVersion}`;
const currentByIdentity = new Map(current.entries.map((entry) => [identity(entry), entry]));
for (const entry of previous.entries) {
  const candidate = currentByIdentity.get(identity(entry));
  if (!candidate) {
    throw new Error(`Published animation preset was removed: ${identity(entry)}. Add a new version instead.`);
  }
  if (JSON.stringify(candidate) !== JSON.stringify(entry)) {
    throw new Error(`Published animation preset was mutated: ${identity(entry)}. Restore it and publish a new version.`);
  }
}
console.log(`animation-library-immutability-ok: ${previous.entries.length} published versions preserved`);
