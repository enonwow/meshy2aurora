#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const profilePath = resolve(
  repoRoot,
  "proof-profiles/m0-r27-on-r25-binary-bootstrap-v1.json",
);
const profile = JSON.parse(readFileSync(profilePath, "utf8"));

const expected = {
  version: "aurora-toolset-binary-module-bootstrap-profile/v1",
  id: "m2a-m0-r27-on-r25-binary-bootstrap-v1",
  module: {
    resref: "m2a_m0r25",
    path: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\modules\\m2a_m0r25.mod",
    sha256: "4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257",
  },
  area: { resref: "m2a_m0a25", width: 2, height: 2 },
  entryPoint: { area: "m2a_m0a25", position: [10, 10, 0] },
  orderedHakList: [
    {
      resref: "m2a_m0r27",
      path: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m0r27.hak",
      sha256: "8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd",
    },
    {
      resref: "m2a_m0r26",
      path: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m0r26.hak",
      sha256: "6f805873420b5279b45dadc26d913fa7692374df9e0582c23b42c8a4b42d6bb0",
    },
    {
      resref: "m2a_m0r21",
      path: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m0r21.hak",
      sha256: "25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6",
    },
  ],
  fixtures: [
    {
      id: "m0_fixture",
      templateResRef: "nw_dwarfmerc001",
      appearanceType: 848,
      position: [10, 14.5, 0],
    },
  ],
  noLocalToolsetAdapter: true,
};

assert.deepEqual(profile, expected, "the exact r27 profile contract changed");
assert.deepEqual(
  profile.orderedHakList.map(({ resref }) => resref),
  ["m2a_m0r27", "m2a_m0r26", "m2a_m0r21"],
  "ordered HAK precedence changed",
);
assert.equal(profile.noLocalToolsetAdapter, true);

for (const artifact of [profile.module, ...profile.orderedHakList]) {
  assert.equal(existsSync(artifact.path), true, `missing exact artifact: ${artifact.path}`);
  const actualSha256 = createHash("sha256")
    .update(readFileSync(artifact.path))
    .digest("hex");
  assert.equal(actualSha256, artifact.sha256, `hash mismatch: ${artifact.path}`);
}

console.log(JSON.stringify({
  ok: true,
  status: "m0_r27_binary_bootstrap_profile_contract_valid",
  profilePath,
  moduleSha256: profile.module.sha256,
  orderedHakList: profile.orderedHakList.map(({ resref, sha256 }) => ({ resref, sha256 })),
  area: profile.area,
  entryPoint: profile.entryPoint,
  fixtures: profile.fixtures,
  startsToolset: false,
  startsNwn: false,
  usesLocalUiRunner: false,
}, null, 2));
