#!/usr/bin/env node

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const standardPath = resolve(repoRoot, "documentation/MODEL_PROOF_STANDARD.md");
const standard = readFileSync(standardPath, "utf8");

for (const required of [
  "prepare-aurora-model-proof-120s.mjs prepare",
  "`cold`:",
  "`switch`:",
  "`load-area`:",
  "`adopt`:",
  "File > Open",
  "gray-viewport state",
  "same Toolset PID",
  "one nonblocking Test Module",
  "QueryPerformanceCounter",
  "120,000 monotonic milliseconds",
  "zero retry",
  "final-model-proof.accepted-at-<QPC>.json",
  "SLO=verified",
  "65,743 ms",
  "76,474 ms",
]) {
  assert.ok(standard.includes(required), `model-proof standard contract missing: ${required}`);
}

assert.equal(prepDecision({ toolsetCount: 0, nwmainCount: 0 }), "cold");
assert.equal(prepDecision({ toolsetCount: 1, nwmainCount: 0, clean: true, exactModule: false }), "switch");
assert.equal(prepDecision({ toolsetCount: 1, nwmainCount: 0, clean: true, exactModule: true, exactAreaViewportCount: 0 }), "load-area");
assert.equal(prepDecision({ toolsetCount: 1, nwmainCount: 0, clean: true, exactModule: true, exactAreaViewportCount: 1 }), "adopt");
assert.throws(() => prepDecision({ toolsetCount: 1, nwmainCount: 0, clean: false, exactModule: false }), /dirty Toolset/);
assert.throws(() => prepDecision({ toolsetCount: 1, nwmainCount: 0, clean: true, exactModule: true, exactAreaViewportCount: 2 }), /ambiguous Area viewport/);
assert.throws(() => prepDecision({ toolsetCount: 2, nwmainCount: 0 }), /exactly zero or one Toolset/);
assert.throws(() => prepDecision({ toolsetCount: 0, nwmainCount: 1 }), /nwmain must be absent/);

const startedAtUtc = "2026-07-22T08:45:37.4148413Z";
const runtimeArmUtc = "2026-07-22T08:48:07.866Z";
const logLastWriteTimeUtc = "2026-07-22T08:48:34.6540318Z";

assert.equal(
  freshUtc({ actionStartedAtUtc: runtimeArmUtc, lastWriteTimeUtc: logLastWriteTimeUtc }),
  true,
  "the exact r31 engine log must be classified as fresh",
);
assert.ok(
  utcEpoch(logLastWriteTimeUtc, "lastWriteTimeUtc") - utcEpoch(startedAtUtc, "startedAtUtc") > 177_000,
  "the exact r31 log must be later than the timed-run start",
);

// Reproduce the failed PowerShell shape: casting the Z instant to local time
// created 10:45 wall-clock ticks, then comparing them with 08:48 UTC ticks.
// Such a mixed-kind comparison reports false even though the instants are in
// the opposite order.
const mutatedLocalWallClockTicks = Date.UTC(2026, 6, 22, 10, 45, 37, 414);
const utcLogTicks = utcEpoch(logLastWriteTimeUtc, "lastWriteTimeUtc");
assert.equal(utcLogTicks > mutatedLocalWallClockTicks, false, "regression setup must reproduce false stale");
const observedNowUtcTicks = Date.UTC(2026, 6, 22, 8, 46, 27, 126);
assert.ok(
  observedNowUtcTicks - mutatedLocalWallClockTicks < -7_150_000,
  "regression setup must reproduce the negative elapsed duration",
);
assert.equal(monotonicElapsed(0, 49_712), 49_712, "deadline elapsed time must remain monotonic");
assert.throws(() => monotonicElapsed(50_000, 49_712), /monotonic counter moved backwards/);

assert.throws(
  () => freshUtc({
    actionStartedAtUtc: runtimeArmUtc,
    lastWriteTimeUtc: "2026-07-22T10:48:34.6540318+02:00",
  }),
  /canonical UTC instant ending in Z/,
  "a local-offset LastWriteTime mutation must be rejected",
);
assert.throws(
  () => freshUtc({ actionStartedAtUtc: runtimeArmUtc, lastWriteTime: logLastWriteTimeUtc }),
  /LastWriteTimeUtc is required/,
  "the local LastWriteTime field must never substitute for LastWriteTimeUtc",
);

const requiredRuntimeProducers = ["dispatch", "handoff", "observe", "capture", "log", "packet"];
assert.deepEqual(
  admitRuntimePlan({
    producers: requiredRuntimeProducers,
    dispatchMode: "nonblocking",
    dispatchReturnsAtMs: 4_000,
    runtimeReadyAtMs: 5_000,
    observerStartsAtMs: 5_000,
    combinesDispatchAndPoll: false,
    postDispatchToolsetHwndReadback: false,
  }),
  { admitted: true },
);

assert.throws(
  () => admitRuntimePlan({
    producers: requiredRuntimeProducers,
    dispatchMode: "synchronous",
    dispatchReturnsAtMs: 70_000,
    runtimeReadyAtMs: 5_000,
    observerStartsAtMs: 70_000,
    combinesDispatchAndPoll: true,
    postDispatchToolsetHwndReadback: true,
  }),
  /runtime observer is blocked behind Test Module dispatch/,
  "run-2 regression: NWN ready at T+5 must not wait for a dispatch returning at T+70",
);

assert.throws(
  () => admitRuntimePlan({
    producers: requiredRuntimeProducers.filter((name) => name !== "capture"),
    dispatchMode: "nonblocking",
    dispatchReturnsAtMs: 4_000,
    runtimeReadyAtMs: 5_000,
    observerStartsAtMs: 5_000,
    combinesDispatchAndPoll: false,
    postDispatchToolsetHwndReadback: false,
  }),
  /runtime producer missing: capture/,
  "every post-launch producer must exist before T+0",
);

const exactCommandPlan = {
  stages: [
    "toolset-open",
    "area-open",
    "selection",
    "toolset-capture",
    "dispatch",
    "handoff",
    "observe",
    "runtime-capture",
    "log",
    "packet",
  ].map((name) => ({
    name,
    canonicalEntrypoint: `public:${name}`,
    exactArgvEnvPinned: true,
    dryRunPassed: true,
    helpAcceptedExactArgv: true,
    contractPassed: true,
    policyAccepted: true,
    unsupportedCliFlags: [],
  })),
  observedState: "ownership-unverified",
  recovery: {
    count: 1,
    stateFingerprintExact: true,
    samePid: true,
    canonicalEntrypoint: "public:same-pid-ownership-recovery",
    exactArgvEnvPinned: true,
    dryRunPassed: true,
    contractPassed: true,
    plannedBeforeT0: true,
  },
};

assert.deepEqual(admitExactCommandPlan(exactCommandPlan), { admitted: true });

assert.throws(
  () => admitExactCommandPlan(mutateCommandPlan(exactCommandPlan, (plan) => {
    plan.stages.find((stage) => stage.name === "toolset-open").policyAccepted = false;
  })),
  /planned route rejected by current policy: toolset-open/,
  "a noncanonical direct opener must be rejected before T+0",
);

assert.throws(
  () => admitExactCommandPlan(mutateCommandPlan(exactCommandPlan, (plan) => {
    plan.stages.find((stage) => stage.name === "area-open").unsupportedCliFlags.push("--area");
  })),
  /unsupported CLI flag: area-open:--area/,
  "an Area command with an unsupported flag must be rejected before T+0",
);

assert.throws(
  () => admitExactCommandPlan(mutateCommandPlan(exactCommandPlan, (plan) => {
    plan.recovery.plannedBeforeT0 = false;
  })),
  /unplanned live recovery: ownership-unverified/,
  "ownership-unverified without a predeclared same-PID route must reject START",
);

const certificationPath = resolve(
  repoRoot,
  "proof-output/m0-r31-hierarchy-only-20260722/model-proof-120s-slo-certification-v1.json",
);
const certification = JSON.parse(readFileSync(certificationPath, "utf8"));
assert.equal(certification.version, "aurora-model-proof-120s-slo-certification/v1");
assert.equal(certification.status, "verified");
assert.equal(certification.acceptancePolicy.requiresColdPreparation, true);
assert.equal(certification.acceptancePolicy.requiresCleanSwitchPreparation, true);
assert.equal(certification.runs.length, 2);
assert.deepEqual(certification.runs.map((run) => run.preparation.route).sort(), ["cold", "switch"]);

for (const run of certification.runs) {
  assert.equal(sha256File(run.acceptancePath), run.acceptanceSha256);
  const acceptance = JSON.parse(readFileSync(run.acceptancePath, "utf8"));
  assert.equal(acceptance.profileId, run.profileId);
  assert.equal(acceptance.profileSha256, run.profileSha256);
  assert.equal(acceptance.finalPacket.sha256, run.finalPacketSha256);
  assert.equal(sha256File(acceptance.finalPacket.path), run.finalPacketSha256);
  const match = basename(run.acceptancePath).match(/accepted-at-(\d+)\.json$/u);
  assert.ok(match, `accepted filename lacks QPC counter: ${run.id}`);
  const elapsedMs = Number(
    ((BigInt(match[1]) - BigInt(acceptance.clock.startCounter)) * 1000n)
      / BigInt(acceptance.clock.frequency),
  );
  assert.equal(elapsedMs, run.publishCompletedElapsedMs);
  assert.ok(elapsedMs <= certification.hardLimitMs, `${run.id} exceeded 120-second SLO`);
  assert.equal(run.toolset.modelVisibility, "visible");
  assert.equal(run.toolset.proofCompleteness, "verified");
  assert.equal(run.runtime.modelVisibility, "not_visible");
  assert.equal(run.runtime.proofCompleteness, "verified");

  for (const [key, path] of Object.entries(run.preparation)) {
    if (!key.endsWith("Path")) continue;
    const hashKey = `${key.slice(0, -"Path".length)}Sha256`;
    assert.match(run.preparation[hashKey] ?? "", /^[0-9a-f]{64}$/u, `${run.id}:${hashKey}`);
    assert.equal(sha256File(path), run.preparation[hashKey], `${run.id}:${key}`);
  }
}

assert.equal(
  certification.result.maximumElapsedMs,
  Math.max(...certification.runs.map((run) => run.publishCompletedElapsedMs)),
);
assert.equal(certification.result.sloVerified, true);

console.log(JSON.stringify({
  ok: true,
  status: "model_proof_standard_timing_and_runtime_action_contract_valid",
  standardPath,
  regression: {
    startedAtUtc,
    runtimeArmUtc,
    logLastWriteTimeUtc,
    freshAfterArmMs: utcEpoch(logLastWriteTimeUtc, "lastWriteTimeUtc") - utcEpoch(runtimeArmUtc, "runtimeArmUtc"),
    mixedKindMutationRejected: true,
    blockingDispatchRegressionRejected: {
      runtimeReadyAtMs: 5_000,
      dispatchReturnsAtMs: 70_000,
    },
    requiredRuntimeProducers,
    exactCommandStages: exactCommandPlan.stages.map((stage) => stage.name),
    preT0RegressionsRejected: [
      "planned-route-rejected-by-policy",
      "unsupported-cli-flag",
      "unplanned-live-recovery",
    ],
    sloCertification: {
      path: certificationPath,
      acceptedRuns: certification.runs.map((run) => ({
        id: run.id,
        preparation: run.preparation.route,
        elapsedMs: run.publishCompletedElapsedMs,
      })),
    },
  },
}, null, 2));

function freshUtc(input) {
  assert.ok(Object.hasOwn(input, "lastWriteTimeUtc"), "LastWriteTimeUtc is required");
  return utcEpoch(input.lastWriteTimeUtc, "lastWriteTimeUtc") >= utcEpoch(input.actionStartedAtUtc, "actionStartedAtUtc");
}

function prepDecision(state) {
  assert.equal(state.nwmainCount, 0, "nwmain must be absent during preparation");
  assert.ok(state.toolsetCount === 0 || state.toolsetCount === 1, "exactly zero or one Toolset is allowed");
  if (state.toolsetCount === 0) return "cold";
  assert.equal(state.clean, true, "dirty Toolset cannot be prepared");
  if (!state.exactModule) return "switch";
  const viewportCount = state.exactAreaViewportCount ?? 0;
  assert.ok(viewportCount === 0 || viewportCount === 1, "ambiguous Area viewport");
  return viewportCount === 0 ? "load-area" : "adopt";
}

function utcEpoch(value, field) {
  assert.match(value ?? "", /Z$/u, `${field} must be a canonical UTC instant ending in Z`);
  const parsed = Date.parse(value);
  assert.ok(Number.isFinite(parsed), `${field} must be a valid UTC instant`);
  return parsed;
}

function monotonicElapsed(startTick, currentTick) {
  assert.ok(currentTick >= startTick, "monotonic counter moved backwards");
  return currentTick - startTick;
}

function admitRuntimePlan(plan) {
  for (const producer of ["dispatch", "handoff", "observe", "capture", "log", "packet"]) {
    assert.ok(plan.producers.includes(producer), `runtime producer missing: ${producer}`);
  }
  assert.ok(
    plan.observerStartsAtMs <= plan.runtimeReadyAtMs,
    "runtime observer is blocked behind Test Module dispatch",
  );
  assert.equal(plan.combinesDispatchAndPoll, false, "dispatch and poll must be separate calls");
  assert.equal(plan.postDispatchToolsetHwndReadback, false, "post-dispatch Toolset HWND readback is forbidden");
  assert.equal(plan.dispatchMode, "nonblocking", "Test Module dispatch must be nonblocking");
  assert.ok(plan.dispatchReturnsAtMs <= 10_000, "Test Module dispatch exceeded its bounded return interval");
  return { admitted: true };
}

function admitExactCommandPlan(plan) {
  const requiredStages = [
    "toolset-open",
    "area-open",
    "selection",
    "toolset-capture",
    "dispatch",
    "handoff",
    "observe",
    "runtime-capture",
    "log",
    "packet",
  ];
  for (const name of requiredStages) {
    const matches = plan.stages.filter((stage) => stage.name === name);
    assert.equal(matches.length, 1, `exact command stage missing or ambiguous: ${name}`);
    const stage = matches[0];
    assert.ok(stage.canonicalEntrypoint, `canonical entrypoint missing: ${name}`);
    assert.equal(stage.policyAccepted, true, `planned route rejected by current policy: ${name}`);
    assert.equal(stage.exactArgvEnvPinned, true, `exact argv/env not pinned: ${name}`);
    assert.equal(stage.dryRunPassed, true, `public dry-run missing: ${name}`);
    assert.equal(stage.helpAcceptedExactArgv, true, `CLI help rejected exact argv: ${name}`);
    assert.equal(stage.contractPassed, true, `public contract missing: ${name}`);
    assert.equal(stage.unsupportedCliFlags.length, 0, `unsupported CLI flag: ${name}:${stage.unsupportedCliFlags[0]}`);
  }
  if (plan.observedState === "ownership-unverified") {
    assert.equal(plan.recovery?.plannedBeforeT0, true, "unplanned live recovery: ownership-unverified");
    assert.equal(plan.recovery.count, 1, "ownership-unverified requires exactly one recovery");
    assert.equal(plan.recovery.stateFingerprintExact, true, "recovery state fingerprint mismatch");
    assert.equal(plan.recovery.samePid, true, "recovery is not same-PID");
    assert.ok(plan.recovery.canonicalEntrypoint, "recovery canonical entrypoint missing");
    assert.equal(plan.recovery.exactArgvEnvPinned, true, "recovery argv/env not pinned");
    assert.equal(plan.recovery.dryRunPassed, true, "recovery dry-run missing");
    assert.equal(plan.recovery.contractPassed, true, "recovery contract missing");
  }
  return { admitted: true };
}

function mutateCommandPlan(source, mutate) {
  const changed = structuredClone(source);
  mutate(changed);
  return changed;
}

function sha256File(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}
