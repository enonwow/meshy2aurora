import assert from "node:assert/strict";
import test from "node:test";

import {
  buildM0ProofPlan,
  parseCliArguments,
  resolveExactModuleListItem,
  resolveOpenMenuItem,
  summarizeDoctor,
  validateOpenOptions,
} from "./m2a-aurora-proof.mjs";

test("parses the narrow read-only M0 proof commands", () => {
  assert.deepEqual(
    parseCliArguments(["--json", "doctor"]),
    { json: true, command: "doctor" },
  );
  assert.deepEqual(
    parseCliArguments(["plan"]),
    { json: false, command: "plan" },
  );
  assert.deepEqual(
    parseCliArguments(["open", "--live"]),
    { json: false, command: "open", live: true },
  );
  assert.deepEqual(
    parseCliArguments(["open", "--live", "--reuse-existing-session"]),
    { json: false, command: "open", live: true, reuseExistingSession: true },
  );
  assert.deepEqual(
    parseCliArguments(["inspect-open-dialog", "--out", "proof-output/m0/open-dialog.json"]),
    { json: false, command: "inspect-open-dialog", out: "proof-output/m0/open-dialog.json" },
  );
  assert.deepEqual(
    parseCliArguments(["complete-open-dialog", "--live", "--out", "proof-output/m0/module-open.json"]),
    { json: false, command: "complete-open-dialog", live: true, out: "proof-output/m0/module-open.json" },
  );
  assert.throws(
    () => parseCliArguments(["complete-open-dialog", "--out", "proof-output/m0/module-open.json"]),
    /--live is required/,
  );
});

test("requires one exact M0 module entry before selecting the current Open dialog list", () => {
  assert.deepEqual(
    resolveExactModuleListItem(["other", "m2a_m0_proof", "third"], "m2a_m0_proof"),
    { item: "m2a_m0_proof", index: 1 },
  );
  assert.throws(
    () => resolveExactModuleListItem(["m2a_m0_proof", "m2a_m0_proof"], "m2a_m0_proof"),
    /expected_one_exact_module_list_item:2/,
  );
});

test("accepts the narrow owner-drawn File/Open fallback only after current menu readback", () => {
  const fallback = resolveOpenMenuItem([
    { position: 0, id: 43, label: "", enabled: true },
    { position: 1, id: 44, label: "", enabled: true },
    { position: 2, id: 45, label: "", enabled: false },
  ]);
  assert.deepEqual(fallback, {
    item: { position: 1, id: 44, label: "", enabled: true },
    identity: "current-file-menu-position-1-id-44-label-unavailable",
  });
  assert.throws(
    () => resolveOpenMenuItem([
      { position: 0, id: 43, label: "", enabled: true },
      { position: 1, id: 44, label: "", enabled: false },
    ]),
    /not_uniquely_enabled/,
  );
});

test("requires an explicit live flag and a project-local evidence path before Toolset open", () => {
  const parsed = parseCliArguments(["--json", "open", "--live", "--out", "proof-output/m0/live/open.json"]);
  assert.deepEqual(parsed, {
    json: true,
    command: "open",
    live: true,
    out: "proof-output/m0/live/open.json",
  });
  assert.throws(() => validateOpenOptions({ live: false, out: "proof-output/m0/live/open.json" }), /live required/i);
  assert.throws(() => validateOpenOptions({ live: true, out: "C:/outside/evidence.json" }), /project-local/i);
});

test("keeps the Toolset -> NWN plan fail-closed around the viewport gate", () => {
  const plan = buildM0ProofPlan();
  assert.equal(plan.module.resref, "m2a_m0_proof");
  assert.equal(plan.module.area, "m2a_m0proof_area");
  assert.deepEqual(plan.module.hakList, ["znd_tortoise", "m2a_m0_proof"]);
  assert.equal(plan.stages.at(-1).id, "nwn-proof");
  assert.equal(plan.stages.find((stage) => stage.id === "toolset-viewport").requiredBeforeNext, true);
  assert.equal(plan.stages.find((stage) => stage.id === "nwn-proof").requiresEngineLoadLog, true);
});

test("doctor reports missing requirements without exposing process environment", () => {
  const summary = summarizeDoctor({
    paths: {
      toolset: { exists: true, sha256: "toolset" },
      nwmain: { exists: true, sha256: "nwmain" },
      referenceHak: { exists: true, sha256: "reference" },
      m0Hak: { exists: false, sha256: null },
      m0Module: { exists: false, sha256: null },
    },
    processes: { toolset: [], nwmain: [] },
    displays: [{ deviceName: "\\\\.\\DISPLAY1", primary: false }],
  });

  assert.equal(summary.ok, false);
  assert.deepEqual(summary.missing, ["m0Hak", "m0Module"]);
  assert.equal("environment" in summary, false);
  assert.equal(summary.proofDisplay.ready, true);
});
