# M0 v10: binary runtime preflight

**Status:** `IN_PROGRESS` — binary package and its central profile are valid; no
NWN runtime observation has been made.

## Scope and non-interference boundary

This packet is an offline/central-tooling checkpoint for a new identity.  It
does not replace the already captured Aurora visual proof for `m2a_m0v6`, and
it must not be read as an NWN proof.  The live Toolset session remains frozen:
no camera move, Adjust Location transport, shared-runner edit, second Toolset
launch, File Open action, or `nwtoolset.ini`/MRU change was made for this
packet.

## Exact artifact binding

| Field | Value |
| --- | --- |
| Module resref / path | `m2a_bm0v10` / `C:\\Users\\enonw\\Documents\\Neverwinter Nights\\modules\\m2a_bm0v10.mod` |
| Module SHA-256 | `8f1e1a844d3cb40fc24de17129a218dfbd4067edd5e2060f03eb48bc74319ff5` |
| Ordered HAK entry / path | `m2a_m0v10` / `C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m0v10.hak` |
| HAK SHA-256 | `04afbb1dc1e2f5cbb601e5d005f81396a044fc7b16597022b104367c5ee74966` |
| Area | `m2a_bm0a10`, `2 x 2` `tdc01` tiles |
| Entry | Area `m2a_bm0a10`, position `[5.0, 5.0, 0.0]` |
| Fixture | `meshy-m0-v10`, template `nw_dwarfmerc001`, position `[10.0, 10.0, 0.0]` |
| Appearance | `848`, row label `M2A_M0_MESHY_RIGID`, model resref `m2a_m0p01` |
| Grounded MDL SHA-256 | `4cfa0ff8ef2ce54493256cc675de9236fb3045cc98d0f21dafaff22a0b0e4776` |
| Texture SHA-256 | `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b` |

The authoritative machine-readable profile is
[`binary-bootstrap-profile.json`](../../proof-output/m0-v10-binary-runtime-20260720/binary-bootstrap-profile.json).

## Reproducible preparation checks

The materializer was extended with caller-supplied module, area, and HAK
resrefs so that this experiment cannot overwrite the historic `m2a_bm0p1`
module or its HAK.  Targeted verification passed:

```text
rustfmt --check crates/m2a-core/src/proof_module.rs \
  crates/m2a-core/examples/materialize_m0_binary_vertical_slice.rs \
  crates/m2a-core/tests/binary_m0_vertical_slice_module.rs
cargo test -p m2a-core --test binary_m0_vertical_slice_module        # 2 passed
cargo test -p m2a-core --example materialize_m0_binary_vertical_slice # 2 passed
git diff --check                                                     # passed
```

The shared central scripts also accepted the exact profile in both modes:

```text
validate-aurora-toolset-binary-module-bootstrap.mjs --mode dry-run  # success
validate-aurora-toolset-binary-module-bootstrap.mjs --mode preflight # success
aurora-toolset-binary-module-native-geometry.mjs dry-run             # valid plan
```

## Appearance-row reconciliation

`15109` appears in older M0 v5 packets and in the historical example in the
shared vertical-slice standard.  It is not a hard-coded requirement of the
generic central route: its declaration validator accepts any non-negative
integer `appearanceType`, then its native GIT binding compares the saved value
to the value declared by that specific profile.  The current v10 HAK
`appearance.2da` has `M2A_M0_MESHY_RIGID` exactly at row `848`; the v10
binary profile declares `848`, and the binary bootstrap validator accepted it.

This also matches the existing saved `m2a_m0v6` native GIT readback and fresh
Toolset visual binding.  Therefore row `848` is the current artifact-bound
value.  Row `15109` is retained only as historical provenance and must not be
silently substituted into v10.

The local Toolset decompilation independently explains why the row must be
bound, rather than guessed: the creature resolver fetches `MODELTYPE` from the
selected appearance index and reports **Invalid Appearance / ModelType for
appearance not found** when that lookup fails.  The v10 binding supplies both
the row and its `m2a_m0p01` model resref.

The HAK-byte binding closes the remaining offline resource chain.  Its package
manifest records exactly three resources: `appearance` (type `2017`),
`m2a_m0p01` (binary MDL type `2002`, SHA-256
`4cfa0ff8ef2ce54493256cc675de9236fb3045cc98d0f21dafaff22a0b0e4776`), and
`m2a_m0t01` (TGA type `3`, SHA-256
`079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`).
The current copied HAK hash equals that package hash.  xoreos's NWN creature
loader independently follows the same relevant branch: it reads
`Appearance_Type`, selects the matching `appearance.2da` row, and, for a
non-`P` `MODELTYPE`, loads the model named by `RACE`.  Here that is `S` and
`m2a_m0p01`.  This is strong offline resolver evidence, but it is explicitly
not a substitute for the required NWN process/capture proof.

## Diagnostic sources rechecked for this decision

* Local Aurora Toolset decompilation:
  `C:\Projects\New Folder\export\decompiled_all.c`, resolver call near
  `FUN_00af3568` / lines 83218--83230.
* [xoreos NWN Area implementation](https://raw.githubusercontent.com/xoreos/xoreos/master/src/engines/nwn/area.cpp):
  the Area loads both ARE and GIT, loads creature entries from `Creature List`,
  and maps the `y * width + x` tiles on 10-by-10-unit centres.  This supports
  the area/fixture and walkability checks, but is not substituted for a real
  NWN observation.
* Shared central vertical-slice implementation:
  `C:\Projects\aurora-web\backend\scripts\aurora-toolset-vertical-slice.mjs`,
  whose generic declaration and native GIT binding compare the declared
  integer row rather than imposing `15109`.
* Current external cross-check (context only, not a substitute for the local
  binary evidence): [NWN Wiki's appearance.2da reference](https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da)
  describes the row-to-model and `MODELTYPE` contract; a
  [Neverwinter Vault discussion](https://forum.neverwintervault.org/t/some-creatures-npcs-wont-display-an-image-when-selected-from-the-palette/5133)
  independently describes the same deployment chain: module HAK, model, and
  an `appearance.2da` entry that points to the model.

## Live-route admission result

The next central native geometry stage correctly refused admission before it
could alter any live state:

```text
status: binary_native_mru_target_mismatch
current MRU[0]: C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0v6.mod
required target: C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_bm0v10.mod
exactTarget: false
```

The current responsive Toolset owns `m2a_m0v6.mod`.  The declared binary
route requires no pre-existing Toolset and an exact pre-existing MRU target
before it can start its single clean owned session.  Altering the MRU would
mean changing Toolset user configuration, which is expressly outside this
task's authorization.  Starting another Toolset or bypassing the route with a
manual File Open would break the route's one-session/provenance gate.

## Evidence verdict and next allowed stage

* **Package/structural binding:** verified.
* **Aurora viewport for v10:** not attempted.
* **NWN runtime for v10:** missing — no `nwmain` window, module/area binding,
  or runtime capture exists.
* **Current live `m2a_m0v6` Toolset visual M0:** separately verified by the
  binding packet, but it is not evidence for this v10 module and is not NWN
  proof.

Only after the declared native geometry route can be admitted should the
workflow produce a fresh saved-module/HAK/fixture binding and enter AUR-S07.
The AUR-S07 Test Module step remains user-owned unless the shared standard
introduces a verified native atom for it.

## Fresh live-state recheck

The following read-only recheck was made after this packet was first written:

| Check | Fresh result |
| --- | --- |
| Toolset processes | exactly one: PID `36888`, responsive |
| Toolset identity | `BioWare Aurora Neverwinter Nights Toolset v89.8193.37-17 - m2a_m0v6.mod`; no dirty `*` and no top-level modal |
| Current Area viewer | not established by this recheck: the top-level window scan found only the main frame and application window, not a `TfrmViewerArea` / `TScrollBox` capture target |
| NWN processes | none (`nwmain` absent) |
| Open M0 v6 hash | intentionally unavailable: the live Toolset holds an OS lock on the MOD |
| v10 MOD/HAK | unchanged; the structural preflight again returned `binary_bootstrap_structural_readback_valid` for hashes recorded above |
| Native-geometry profile | absent |
| AUR-S07 runtime profile, launch request, and runtime packet | absent |

This confirms a hard boundary between the two existing facts: the historical
`m2a_m0v6` visual packet proves a Toolset viewport only, while `m2a_bm0v10`
has a current immutable binary binding only.  Neither artifact can honestly be
used as an NWN runtime claim.  The current `m2a_m0v6` process was observed but
not reused, modified, closed, or treated as an input to the v10 route.

## Final current-route admission record

The current public binary-native-geometry entrypoint was re-run in its
read-only `preflight` phase against this exact v10 profile.  It returned:

```text
blockerCode: STALE_TOOLSET_SESSION_REJECTED
process: PID 36888, responsive
module frame: ... - m2a_m0v6.mod
message: This binary route never reuses a pre-existing Toolset session;
         preserve it and start no second process.
startsToolset: false
startsNwn: false
usesGlobalInput: false
```

Its durable central result is
`proof-output/m0-v10-binary-runtime-20260720/native-geometry/binary-native-geometry-manifest.json`.
This is the same external-state blocker recorded across the current goal
continuations.  The only resume condition is a clean Toolset ownership state
for the declared v10 route, followed by its native geometry/save and the
standard's required user-owned Test Module handoff.  Neither changing
`nwtoolset.ini`/MRU nor forcibly closing/reusing the current `m2a_m0v6`
process is an authorized workaround.

## Authorized restart and resulting gate

The owner then explicitly asked why Aurora was not restarted when it belonged
to this task and a restart was needed. The current clean `m2a_m0v6` frame was
therefore closed through the public canonical `close` command with that exact
authorization. It targeted only PID `36888`, used `WM_CLOSE` (no force kill,
global input, INI change, or save prompt), and verified zero Toolset processes
afterward.

The immediate v10 native-geometry `preflight` still failed without starting
Toolset or NWN, but now for the distinct deterministic gate:

```text
BINARY_NATIVE_MRU_TARGET_MISMATCH
current MRU0: ...\\modules\\m2a_m0v6.mod
required:     ...\\modules\\m2a_bm0v10.mod
```

This is not an untested workaround opportunity. The current shared route
contains this exact preflight condition and its contract test deliberately
asserts rejection of a nonmatching MRU. A temporary `--startupIniPath` is an
inspection input; it does not authorize changing the Toolset's real startup
configuration or bypassing the actual launch ownership gate. Consequently,
there is no approved no-INI/open-existing-module continuation for this
profile. Restart was necessary and successfully completed, but alone is not
sufficient to admit the current central route.

## Correction: configuration boundary and owned-process restart authority

This record corrects an earlier, incorrect implication that changing
`nwtoolset.ini`/`MRU0` might be a next step.  It is not.  The project rules and
the shared operating standard both make Toolset user configuration read-only.
No `nwtoolset.ini` or MRU value was changed during this work, and an MRU gate
inside a helper route is a limitation of that route, not a requirement of
Aurora or of the M0 model.

The owner has also given a standing, task-scoped authorization for the
Toolset process opened by this task: once its PID has been recorded and its
identity has been freshly verified, the operator may close and restart that
owned Toolset session when the workflow requires it, without asking again.
The restart remains bounded to the one recorded PID and uses the public
canonical close/restart path; it does not authorize a second Toolset, force
termination, global input, configuration writes, or closure of an unknown
process.

Before such a restart the operator must still read back PID, responsiveness,
module title/path, dirty state, and modals.  An identity conflict or an
unknown/destructive prompt remains read-only until it can be safely resolved.
For the active verified module, a known save prompt is handled through the
standard saved-close flow before restart.  This authority is distinct from,
and does not relax, the permanent prohibition on touching INI/user settings.

**Correct next-direction statement:** open or restart the verified owned
Toolset session as needed, then continue the normal module/viewport proof
workflow.  Do not propose, perform, or depend on an INI/MRU change.
