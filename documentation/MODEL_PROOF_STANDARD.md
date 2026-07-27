# Meshy2Aurora model-proof standard — 120 seconds

This is the project contract for proving one already admitted, materialized,
installed, immutable model candidate in Aurora Toolset and NWN.

Use the shared `$aurora-model-proof-120s` skill together with
`$aurora-toolset-operate`, `$aurora-toolset-author`, and
`$aurora-toolset-prove`. The only timed-run entrypoint is:

```powershell
node C:\Projects\aurora-web\backend\scripts\aurora-model-proof-120s.mjs <command>
```

Do not recreate the workflow from native leaf atoms or a project-local runner.

## What the 120 seconds measure

The SLO starts with an exact clean proof-ready Toolset already showing the
approved module and Area. Candidate generation, installation, hashing,
structural preflight, cold Toolset startup, module switching, and Area opening
are preparation. Their elapsed time is reported separately and is never hidden
inside the proof result.

Preparation has one public central entrypoint and runs before the clock:

```powershell
node C:\Projects\aurora-web\backend\scripts\prepare-aurora-model-proof-120s.mjs prepare --profile <profile> --prepOutDir <owned-prep-dir>
```

It resolves the current host state into exactly one tested decision:

- `cold`: zero Toolset processes, so open the exact installed/hash-verified MOD;
- `switch`: one clean responsive Toolset has another module, so use `File > Open`
  and select/read back the exact target from the native module list;
- `load-area`: the exact module is loaded but its exact Area/`TScrollBox` is
  absent, including the gray-viewport state, so open the exact Area from the
  profile-owned queue;
- `adopt`: the exact clean module, Area, and `TScrollBox` are already present.

Every active route must end with the same Toolset PID and a new readback of the
exact clean module title, exact Area viewer, and `TScrollBox`. Dirty state,
unexpected modal, second Toolset, any `nwmain`, module/HAK hash drift, wrong
module, wrong Area, or a missing viewport fails before the timed run. The prep
wrapper never saves, builds, writes INI/MRU, uses global input, or changes the
candidate.

The measured run owns:

1. exact Objects-mode selection and independent caret readback;
2. one fresh validated `TScrollBox` capture and Toolset verdict;
3. one nonblocking Test Module dispatch when Toolset is visible;
4. one request-bound `nwmain`, exact placement on the proof monitor without
   activation or z-order change, fresh NWN PNG, and a fresh exact module-load
   log window: appended after a hash-stable prefix, or appended only after the
   coordinator first observes a post-`arm` truncation/rotation and establishes
   a new hash baseline; a historic token before that boundary is never accepted;
5. one runtime verdict and final immutable packet.

Historical r29/r30 runs do not support a 120-second cold-start promise. This
standard removes their manual handoffs and packet-construction delays from the
proof-ready path; it does not falsify the old timings.

## READY — outside the clock

`ready` is read-only and must pass immediately before `toolset`. Require:

- the versioned `aurora-model-proof-120s-profile/v1` and exact hashes of its
  binary, Toolset, runtime, and central route-manifest profiles;
- byte-identical source/installed MOD and exact ordered installed HAK list;
- central structural readback of module, Area, entry, fixture, Appearance, and
  HAK order;
- exactly one responsive clean Toolset on the exact module and Area, with no
  dirty `*`, modal, second Toolset, or `nwmain`;
- an absent profile-owned output root;
- all public coordinator children and contract tests present;
- no Save, Build, repack, `geometry-observe`, retry, Focus, Properties,
  framing, or camera action in the plan.

If READY fails, do not start the clock. Prepare or repair the same candidate;
never allocate another model iteration for a proof-lane failure.

## Timed commands and budget

The same proof owner executes, without an agent handoff:

```powershell
node C:\Projects\aurora-web\backend\scripts\aurora-model-proof-120s.mjs toolset --profile <profile>
node C:\Projects\aurora-web\backend\scripts\aurora-model-proof-120s.mjs record-toolset-visibility --profile <profile> --modelVisibility visible|not_visible --inspection <text>
# only after Toolset visible
node C:\Projects\aurora-web\backend\scripts\aurora-model-proof-120s.mjs runtime --profile <profile>
node C:\Projects\aurora-web\backend\scripts\aurora-model-proof-120s.mjs record-runtime-visibility --profile <profile> --modelVisibility visible|not_visible --inspection <text>
node C:\Projects\aurora-web\backend\scripts\aurora-model-proof-120s.mjs finalize --profile <profile>
```

Budget from the monotonic start counter:

| Window | Required result |
|---|---|
| T+0–20 s | exact selection/readback, `TScrollBox`, Toolset verdict |
| T+20–25 s | runtime arm and one nonblocking Test Module |
| T+25–85 s | one new `nwmain`, NWN PNG, appended exact load-log window |
| T+85–120 s | runtime verdict, integrity validation, final packet |

The clock is Windows `QueryPerformanceCounter`. UTC and file times are
evidence metadata only. One run has one output root, one dispatch per phase,
and zero retry. At 120,000 monotonic milliseconds, no new live action is
dispatched.

## Evidence and verdicts

The first fresh, judgeable, identity-bound image closes each environment.
Focus, Properties, framing, camera movement, and recapture are not acceptance
requirements.

Record independent axes for Toolset and NWN:

- `modelVisibility = visible | not_visible | not_tested`;
- `proofCompleteness = verified | failed | missing`.

Only a judgeable bound absence is `not_visible`. A timeout, capture error,
automation error, stale log, or infrastructure failure is never a visual
failure. Preserve every earlier accepted image and verdict monotonically.

The output contains the QPC clock, exact candidate/profile/resource hashes,
selection and caret readbacks, Toolset and NWN PNG/metadata, request/PID/start
binding, appended log bytes and identity, both verdicts, and either
provisional `final-model-proof.json` plus an atomically published
`final-model-proof.accepted-at-<QPC>.json`, or one `blocker.json`. The
provisional file alone is never an accepted proof.

## SLO acceptance

Schema, fake-clock, route, and dry-run tests establish code readiness but do
not establish elapsed-time performance. The shared skill is now
`SLO=verified`: a cold-prep
run completed in 65,743 ms and a same-PID clean-switch run from r30 to r31
completed in 76,474 ms. Both produced accepted Toolset and NWN packets under
the 120,000 ms QPC deadline with zero retry. The immutable certification is
`proof-output/m0-r31-hierarchy-only-20260722/model-proof-120s-slo-certification-v1.json`.
