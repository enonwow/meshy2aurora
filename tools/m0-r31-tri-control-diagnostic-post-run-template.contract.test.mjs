#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repo = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const templatePath = resolve(
  repo,
  "proof-profiles/m0-r31-tri-control-diagnostic-multi-fixture-post-run-template-v1.json",
);
const template = readJson(templatePath);
const contractPath = resolve(template.diagnosticBinaryProfile.path);
const contractBytes = readFileSync(contractPath);
assert.equal(sha256(contractBytes), template.diagnosticBinaryProfile.sha256);
assert.equal(contractBytes.length, template.diagnosticBinaryProfile.byteLength);
const contract = JSON.parse(contractBytes.toString("utf8"));

verifyTemplate(template, contract);

const swappedRois = structuredClone(template);
[swappedRois.fixtures[1].runtimeRoi, swappedRois.fixtures[2].runtimeRoi] = [
  swappedRois.fixtures[2].runtimeRoi,
  swappedRois.fixtures[1].runtimeRoi,
];
assert.throws(() => verifyTemplate(swappedRois, contract), /runtime_roi_order_mismatch/);

const swappedIds = structuredClone(template);
[swappedIds.fixtures[0].id, swappedIds.fixtures[2].id] = [
  swappedIds.fixtures[2].id,
  swappedIds.fixtures[0].id,
];
assert.throws(() => verifyTemplate(swappedIds, contract), /fixture_binding_mismatch/);

console.log("m0_r31_tri_control_diagnostic_post_run_template_contract_valid");

function verifyTemplate(value, binaryContract) {
  assert.equal(value.version, "m2a-multi-fixture-diagnostic-post-run-template/v1");
  assert.equal(value.status, "awaiting_accepted_base_run");
  assert.equal(value.executable, false);
  assert.equal(value.publishedAcceptanceTrustRootId, null);
  assert.deepEqual(value.acceptedBaseRun, { clock: null, finalPacket: null, acceptance: null });

  const readback = binaryContract.moduleReadback;
  const entry = vector(readback.entryPosition);
  const direction = vector(readback.entryDirection);
  assert.deepEqual(entry, value.baseBinding.area.entryPoint);
  assert.deepEqual(direction, [0, 1, 0]);
  assert.equal(value.fixtures.length, readback.fixtures.length);

  const contractById = new Map(readback.fixtures.map((fixture) => [fixture.id, fixture]));
  for (const fixture of value.fixtures) {
    const expected = contractById.get(fixture.id);
    assert.ok(expected, `fixture_binding_mismatch:${fixture.id}`);
    assert.deepEqual(fixture.position, vector(expected.position), `fixture_binding_mismatch:${fixture.id}:position`);
    assert.equal(fixture.templateResRef, expected.templateResref, `fixture_binding_mismatch:${fixture.id}:template`);
    assert.equal(fixture.treeObjectText, expected.displayName, `fixture_binding_mismatch:${fixture.id}:name`);
    assert.equal(fixture.appearanceRow, expected.appearanceRow, `fixture_binding_mismatch:${fixture.id}:appearance`);
  }

  const expectedOrder = [...readback.fixtures]
    .map((fixture) => ({
      id: fixture.id,
      lateral: lateralOffset(entry, direction, vector(fixture.position)),
    }))
    .sort((left, right) => left.lateral - right.lateral)
    .map(({ id }) => id);
  assert.deepEqual(expectedOrder, ["stock_control", "candidate_m0", "custom_control_h1"]);

  const actualOrder = [...value.fixtures]
    .map((fixture) => ({ id: fixture.id, centerX: fixture.runtimeRoi.x + fixture.runtimeRoi.width / 2 }))
    .sort((left, right) => left.centerX - right.centerX)
    .map(({ id }) => id);
  assert.deepEqual(actualOrder, expectedOrder, "runtime_roi_order_mismatch");
}

function lateralOffset(entry, direction, position) {
  const deltaX = position[0] - entry[0];
  const deltaY = position[1] - entry[1];
  return deltaX * direction[1] - deltaY * direction[0];
}

function vector(value) {
  if (Array.isArray(value)) return value;
  return [value.x, value.y, value.z ?? 0];
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
