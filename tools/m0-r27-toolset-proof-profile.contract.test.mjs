#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const profilePath = resolve(
  repoRoot,
  "proof-profiles/m0-r27-on-r25-toolset-proof-v1.json",
);
const binaryProfilePath = resolve(
  repoRoot,
  "proof-profiles/m0-r27-on-r25-binary-bootstrap-v1.json",
);
const profile = JSON.parse(readFileSync(profilePath, "utf8"));

const expected = {
  version: "aurora-toolset-binary-module-toolset-proof-profile/v1",
  id: "m2a-m0-r27-on-r25-toolset-proof-v1",
  binaryProfile: {
    path: binaryProfilePath,
    sha256: "759f9a95dad7b7d8de6d2ca494800bc8d9c65c25a9dcab3c7651df07c9c4aa6c",
  },
  fixtureSelection: {
    fixtureId: "m0_fixture",
    treeObjectText: "Dwarf Mercenary",
    occurrence: 0,
  },
  capture: {
    targetClass: "TScrollBox",
    allowUniformScene: false,
  },
  noSave: true,
  noLocalToolsetAdapter: true,
};

assert.deepEqual(profile, expected, "the exact r27 Toolset proof profile changed");
assert.equal(existsSync(profile.binaryProfile.path), true, "binary profile is missing");

const binaryProfileBytes = readFileSync(profile.binaryProfile.path);
const actualBinaryProfileSha256 = createHash("sha256")
  .update(binaryProfileBytes)
  .digest("hex");
assert.equal(
  actualBinaryProfileSha256,
  profile.binaryProfile.sha256,
  "binary profile hash changed",
);

const binaryProfile = JSON.parse(binaryProfileBytes.toString("utf8"));
const selectedFixtures = binaryProfile.fixtures.filter(
  ({ id }) => id === profile.fixtureSelection.fixtureId,
);
assert.equal(selectedFixtures.length, 1, "fixture selection is not unique in the binary profile");
assert.deepEqual(selectedFixtures[0], {
  id: "m0_fixture",
  templateResRef: "nw_dwarfmerc001",
  appearanceType: 848,
  position: [10, 14.5, 0],
});

console.log(JSON.stringify({
  ok: true,
  status: "m0_r27_toolset_proof_profile_contract_valid",
  profilePath,
  binaryProfile: {
    path: profile.binaryProfile.path,
    sha256: actualBinaryProfileSha256,
  },
  fixtureSelection: profile.fixtureSelection,
  capture: profile.capture,
  noSave: profile.noSave,
  noLocalToolsetAdapter: profile.noLocalToolsetAdapter,
  startsToolset: false,
  startsNwn: false,
}, null, 2));
