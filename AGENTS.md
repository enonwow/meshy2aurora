# Meshy2Aurora agent rules

## Canonical workspace — HARD STOP

The only canonical repository and writable project workspace is:

`C:\Projects\meshy2aurora`

The following path is forbidden for every project operation:

`C:\Users\enonw\Documents\meshy2aurora`

Never create, edit, stage, copy, migrate, test, build, cache, or store temporary
project work there. It is not a fallback workspace, scratch directory, staging
area, clone, mirror, or migration source. The owner never authorized that path.

Before any implementation or documentation write, run:

`powershell -NoProfile -ExecutionPolicy Bypass -File assert-canonical-workspace.ps1`

The check must resolve the repository root.
If it is not exactly `C:\Projects\meshy2aurora`, stop. Do not work around the
problem by writing elsewhere and do not repeatedly request permissions for
out-of-workspace writes. Reopen or resume the task with the canonical repository
as its workspace root.

All durable project documentation belongs in
`C:\Projects\meshy2aurora\documentation`. Read
`documentation/PROJECT_RULES.md` and
`documentation/CANONICAL_WORKSPACE.md` before changing the project.

## Canonical Meshy asset layout — HARD STOP

The only canonical project root for owner-selected Meshy source models is:

`C:\Projects\meshy2aurora\sample-3d\<asset-id>\`

Every local sample directory must contain `manifest.yaml`. Model payloads such
as GLB remain local and Git-ignored, but their filenames, sizes, SHA-256 hashes,
roles and provenance are recorded in the tracked manifest. Product code and
tests must reference models through `sample-3d`; creating or restoring a
competing source root such as `test-assets\meshy` is forbidden.

`proof-output` is reserved for immutable proof lineages and `artifacts` for
generated deliverables. Neither is a source-model library, and their payloads
must not be silently promoted, duplicated or substituted for a `sample-3d`
source. After a canonical relocation, documentation may normalize a source
path only with a dated amendment confirming byte-identical SHA-256 identity;
the original immutable proof packet remains authoritative for historical
capture-time paths.

Before adding, moving or reconnecting a Meshy source model, read
`documentation/MESHY_ASSET_LAYOUT.md` and run:

`powershell -NoProfile -ExecutionPolicy Bypass -File assert-meshy-asset-layout.ps1`

Any second active asset-root policy, sample without a manifest, undeclared
payload, hash mismatch or product-code reference to `test-assets\meshy` is a
layout failure. Stop and repair the canonical structure instead of adding
another exception or storage location.

Aurora, NWN, Toolset, game installations, user configuration and
`nwtoolset.ini` remain read-only except for the mandatory, narrowly scoped
installation of exact proof MOD/HAK artifacts authorized below, or unless the
owner gives another separate, explicit instruction that changes that boundary.

## Shared render-model triangle budget - OWNER DECISION 2026-07-29

Creature, Placeable and every other render-model route share one product
triangle budget:

`AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000`

Exactly 300,000 triangles are accepted; totals above 300,000 are blocked. The
warning threshold is derived from that one value as 150,000. Product code must
not restore target-specific Creature or Placeable blocking thresholds.

The independent binary MDL boundary of 65,535 index entries, or 21,845
triangles for one triangle-list mesh stream, remains a format/writer gate. It
is not the product budget and must not be substituted for the shared 300,000
limit. Every product route must deterministically partition render geometry
that exceeds the per-stream boundary without deleting triangles or changing
material, hierarchy, deformation, UV, tangent, weight or surface metadata.

## Persistent Aurora/NWN live-control authorization — OWNER DECISION 2026-07-21

For the Meshy2Aurora proof goal, the owner grants the coordinating agents
persistent authority to start, inspect, reuse, operate, switch, test, and close
Aurora Toolset and NWN sessions that the agents started or explicitly adopted
for the current run. Routine in-scope actions do not require another question:
expected modal handling, exact module/Area navigation, object selection,
viewport/runtime capture, Test Module coordination, and clean process closure
may proceed autonomously through the shared canonical operator.

This standing authority also covers safe staging and installation of an exact,
project-generated MOD or HAK from the canonical workspace into the native NWN
user `modules` or `hak` directory for an approved proof lineage. Before any
copy, resolve and hash the exact source, resolve the exact destination, and
require that destination to be absent. After installation, hash the destination
and require byte-for-byte identity with the approved source before opening or
testing it. If the destination already exists, read it without mutation: reuse
it only when its hash is identical; when it differs, never overwrite, delete,
rename, or replace it. Select a fresh approved resref, filename, and destination
path consistent with the model-iteration gate, then repeat the absent-target
and before/after hash checks. This is a narrow MOD/HAK installation exception,
not permission to use a native NWN directory as project workspace or cache.

The owner also authorizes switching from an open module to another exact
approved module without another permission prompt. A proof-owned or explicitly
adopted responsive Toolset session may use `File > Open` when the current exact
module frame has no dirty `*`, readback confirms there are no unsaved changes,
no Save prompt or other modal is present, and the target is the exact installed,
hash-verified approved MOD. After the operation, resolve the windows again and
require the frame and module readback to match the target before continuing. An
unexpected Save prompt, dirty state, target mismatch, or missing post-switch
readback is fail-closed and must not be confirmed or bypassed. Clean close,
verified process settlement, and a fresh Toolset session remain an authorized
alternative when a same-session switch is unsuitable; they are not mandatory.
Never keep two competing Toolset processes.

This standing authorization removes per-step confirmation prompts; it does not
authorize global input, an unidentified destructive prompt, INI/MRU or user
configuration mutation, silent Save/repack of the frozen r27 lineage, a second
model iteration, or bypass of candidate/session/proof identity gates. Such
states remain fail-closed under the shared Aurora standards.

## Human-owned final proof — OWNER DECISION 2026-07-24

The project owner performs the final visual proof in Aurora Toolset and NWN.
This decision narrows and overrides the standing live-control authorization
above for candidate proof work.

- Agents implement and diagnose the model, run offline tests, freeze one exact
  candidate lineage, and prepare a concise handoff with its hashes and expected
  module, Area, object, HAK, Appearance row, and placement. Every handoff must
  lead with the exact test-module `.mod` filename, followed by the module name
  shown in Toolset and the exact Area name; none of these may be left implicit.
- Agents must not start, adopt, switch, control, capture, time, clean up, or
  complete an Aurora Toolset or NWN proof session. They must not invoke the
  proof skill, its coordinator, its wrappers, or any substitute proof workflow.
- When an agent creates or freezes proof artifacts as a MOD and one or more
  HAKs, it must immediately install those exact files into the native NWN user
  `modules` and `hak` directories. A MOD/HAK proof handoff is incomplete and
  must not receive `ready_for_owner_proof` until this installation and its
  byte-for-byte verification have succeeded.
- Before every proof-artifact copy, resolve and hash each exact canonical
  source, resolve the exact native destination, and require the destination to
  be absent. After copying, hash the destination and require equality with the
  source. If a destination already exists, reuse it only when the hash is
  identical. If it differs, never overwrite, delete, rename, or replace it;
  fail closed and report the exact collision. Do not silently allocate another
  candidate, resref, filename, HAK, or MOD to work around the collision.
- This mandatory installation is preparation for the human-owned proof, not
  permission to start, adopt, switch, control, capture, time, clean up, or
  complete Aurora Toolset/NWN, and not permission to use native NWN directories
  as a project workspace, cache, or staging area.
- The owner decides how to perform the visual proof. After the owner reports
  the result or supplies evidence, agents may record it, apply the model
  iteration gate, diagnose the failure, and continue implementation.
- A later exception requires a new, direct owner instruction that explicitly
  authorizes agent-run live proof for one exact candidate. General requests to
  continue implementation, finish the model, or prepare it for testing do not
  restore live-control authority.

The agent-side completion boundary is therefore `ready_for_owner_proof`, not a
claimed Toolset/NWN visual success. Only the owner's reported result can close
the visual proof stage while this decision remains active.

## Model iteration gate — HARD STOP

A new model iteration is forbidden until the current exact candidate has a
fresh, candidate-bound visual failure in Aurora Toolset or NWN runtime. A new
iteration includes a new `rNN` module, HAK name/copy, model or texture resref,
2DA mapping, fixture module, or regenerated asset payload used to repeat the
same candidate.

- Track two independent axes for Toolset and NWN:
  `modelVisibility = visible | not_visible | not_tested` and
  `proofCompleteness = verified | failed | missing`. Never use `missing` as a
  model-visibility result.
- A build, save/hash lock, HAK attachment, placement, geometry, proof-profile,
  capture, timeout, or automation failure is a lane failure, not a visual
  model failure.
- After the model is visible in Toolset, test that same module/HAK/model
  lineage in NWN before changing or copying any artifact in the lineage.
- A Toolset model verdict requires exact candidate/object identity, exact
  object selection/readback, and a fresh validated `TScrollBox` capture.
  A capture without exact object identity is observational evidence; it must
  be completed, not replaced by another iteration. A capture whose view does
  not permit a visual verdict is `proofCompleteness=missing` and
  `modelVisibility=not_tested`, never `modelVisibility=not_visible`.
- **Owner visual decision:** when the exact candidate-bound NWN session shows
  the player and the clear, unobstructed forward scene where the fixture was
  deliberately placed directly in front of the player, and the expected model
  is absent from that judgeable screen, the model does not work in NWN. Record
  `modelVisibility=not_visible`; do not downgrade it to `not_tested` merely
  because additional view telemetry, crop automation, packet assembly or
  another proof helper failed. Proof-lane completeness may be reported
  separately, but it must not erase or postpone the visible owner decision.
- In a multi-control scene, apply the same rule to every clearly expected
  on-screen control. If the screen is judgeable and none of the models placed
  in front of the player are drawn, record those visible absences instead of
  continuing to validate the proof machinery. Only a genuinely unreadable or
  unbound screen remains `not_tested`.
- Preserve successful evidence monotonically. A later run with missing proof
  cannot erase or invert an earlier verified observation for another exact
  module/hash.
- Admit a new iteration only from a fresh candidate-bound
  `modelVisibility=not_visible` result in Toolset or NWN. A
  `proofCompleteness=missing` result requires completing the same candidate.
- A new iteration requires a durable record of the failing capture/runtime
  packet, exact candidate hashes, diagnosed cause, and the minimal intended
  artifact delta. If the same candidate cannot be recovered for a non-model
  reason, stop that lane; do not silently allocate another `rNN` target.

## Shared Aurora operating skills

This section applies only if a later direct owner instruction restores
agent-run Aurora/NWN work for one exact candidate. While the human-owned final
proof decision above is active, agents do not invoke these live workflows.

Before every turn that touches Aurora Toolset or NWN, read and use the current
shared `$aurora-toolset-operate` skill.

### Skill-first execution — HARD STOP

Reading a skill is not sufficient. The coordinating agent and every delegated
live owner must execute Aurora/NWN work through the public workflow, runner and
state transitions named by the applicable shared skills. Before writing a new
runner, composing native leaf atoms, or inventing a recovery sequence, first
use the skill route exactly as published and inspect its returned state.

### No tool-building during Aurora/NWN execution — HARD STOP

When the current task is to operate, test, diagnose, capture, or prove Aurora
Toolset or NWN, agents must not create, modify, replace, or extend any wrapper,
runner, native atom, UI-control script, recovery path, skill, standard, route
manifest, or proof coordinator. The existing shared skills and their published
public entrypoints are the complete execution interface for that task.

- A failed, missing, timed-out, or rejected public transition is a lane
  blocker. It is not authorization to enter implementation mode, add a retry,
  switch transport, call a leaf atom directly, or compose a new wrapper.
- A proof skill may coordinate existing public operations, candidate identity,
  timing, capture, verdicts, and packets. It must not reimplement process,
  HWND, monitor, modal, button, module, Area, Test Module, NWN-window, or
  cleanup handling that belongs to the shared operating skills.
- Do not edit `C:\Projects\aurora-web`, user-scoped skills, installed game
  tooling, or shared Aurora standards from an active Meshy2Aurora execution or
  proof task.
- Shared-tooling repair is allowed only in a separate task after a direct,
  explicit owner instruction that names the tooling change or repair. That
  task must begin with zero live Toolset/NWN processes, reproduce the defect
  offline, add contract and negative tests, and finish before any proof task
  resumes.
- If no published skill transition can continue the exact state, preserve the
  immutable candidate and evidence, report the exact blocker and required
  resume condition, and stop that lane. Do not allocate another model
  iteration and do not repair tooling in place.

This section narrows every more general instruction below: a statement that a
broken shared transition must be repaired never authorizes that repair inside
the active Aurora/NWN execution or proof task.

- Add `$aurora-model-proof-120s` for every candidate-bound Toolset-to-NWN model
  visibility proof. Its public preparation, READY, timed-run, capture and proof
  transitions are mandatory; a hand-written substitute is forbidden.
- A skill created or modified for the current task is immediately mandatory for
  its author, the orchestrator and every delegated agent. Creating the skill
  does not exempt its author from using it, and familiar knowledge of its
  internals is never a reason to bypass its public commands or state machine.
- A missing or broken skill transition is a shared-tooling defect. Stop live
  work and record the exact blocker. Repair and contract-test it only in the
  separate, explicitly owner-authorized tooling task required above; then
  resume the same candidate through the published skill. Do not bypass it with
  project-local code or direct leaf calls.
- The coordinator must assign exactly one live proof owner. Other agents may do
  read-only review, but they must not operate Toolset/NWN or modify live proof
  state during that owner's run.
- Do not certify a proof standard from schema, mock or dry-run tests alone. Its
  real host-state certification must cover cold start, clean same-PID module
  switch through the native module list, exact Area loading when only a blank
  viewport is present, and adoption of an already exact module/Area session.
- A claimed time limit applies only after the standard has passed those real
  transitions. Report preparation time and timed-run time separately, and
  never describe an untested or lane-blocked route as a completed fast proof.
- When a skill result conflicts with an ad-hoc plan, the skill result controls.
  Correct the plan or the shared skill; never silently continue outside it.

- Add `$aurora-toolset-author` for module/Area/HAK/Appearance, object,
  inventory/equipment, script, build, or Test Module work.
- Add `$aurora-toolset-prove` for proof capture, runtime verification, audits,
  rollback evidence, or completion claims.
- Read the skill from the installed user-scoped path on every applicable turn;
  do not vendor or maintain a project-local copy.
- Treat `C:\Projects\aurora-web` product code, assets, fixtures, and product
  validators as reference-only under `documentation/PROJECT_RULES.md`. The
  shared Aurora operating skills, their canonical runner, and verified native
  atoms are mandatory shared operator tooling, not a Meshy2Aurora product
  dependency. Use them directly; do not create, qualify, or substitute a
  project-local Toolset runner or UI adapter.
- A shared skill may strengthen safety and proof requirements but cannot weaken
  the canonical-workspace, Aurora First, provenance, or reference-only rules.
