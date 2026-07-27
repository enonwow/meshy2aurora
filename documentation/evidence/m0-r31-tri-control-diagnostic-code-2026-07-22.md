# M0 r31 tri-control diagnostic — code record (2026-07-22)

## Decision

The next runtime action is not another speculative M0 writer change. The
project now has an offline-only three-fixture diagnostic that holds the exact
r31 candidate bytes fixed and places these controls in one Area and one HAK:

1. stock `c_horror` at physical `appearance.2da` row 102;
2. exact r31 M0 `m2a_m0p01` at physical row 15100;
3. exact project-owned runtime-visible H1 v20 `m2a_m6p01`, with its complete
   historical row 15100 cloned to diagnostic row 15101.

The intended runtime positions are respectively `[5,18,0]`, `[10,14.5,0]`
and `[15,18,0]`, with the player entry kept at `[10,10,0]` facing `+Y`.
The output is diagnostic-only: it cannot claim resolver, renderer, Toolset or
NWN success and it cannot admit a model iteration by itself.

## Why this isolates a useful boundary

- Stock visible, H1 visible, M0 absent: the general creature renderer and the
  project custom HAK/2DA path work in the same run; the failure is specific to
  the M0 resource/load-or-render family.
- Stock visible, H1 absent, M0 absent: the common custom-resource path remains
  suspect; this does not justify another M0 writer delta.
- All three absent: the run is non-diagnostic.
- All three visible: the historical r31 session/container result must be
  investigated before changing M0 bytes.

No outcome is allowed to infer resource-load success from a silent log. A
fresh image verdict remains separate from resolver observability.

## Project implementation

- `crates/m2a-core/src/proof_module.rs` contains the public deterministic
  multi-fixture binary MOD builder and inspector. The inspector independently
  parses exact IFO/ARE/GIT/GIC/UTC GFF types, the full required ERF resource
  set, ordered HAK list, entry point, Area geometry and every fixture's archive
  key, UTC `TemplateResRef`, display name, Appearance, position and direction.
- `crates/m2a-core/src/tri_control_hak.rs` builds and replays one exact HAK with
  `appearance`, r31 MDL/TGA and H1 MDL/TGA. It pins input hashes and lengths,
  clones every H1 2DA cell and requires an exact `HAK V1.0` container plus
  ordered resource keys and payload hashes.
- `crates/m2a-core/src/tri_control_diagnostic.rs` composes the fixed three
  fixtures, MOD and HAK and requires deterministic replay. Its contract is
  permanently `diagnostic_only=true`, `runtime_admissible=false`,
  `resolver_verified=false`, and `renderer_verified=false`.
- `crates/m2a-core/examples/materialize_tri_control_diagnostic.rs` is the only
  project materialization entrypoint. It hard-pins the six base/r31/H1 input
  paths, lengths and hashes, requires a caller-owned absent output directory
  inside the canonical repository, writes through an atomic staging rename and
  then independently replays the four written outputs: MOD, singleton HAK,
  contract JSON and materialization-profile JSON. It never installs or launches
  anything.
- Exact r31/H1 artifact tests are explicitly ignored in a normal suite and
  require `M2A_REQUIRE_RUNTIME_WITNESSES=1`; a routine green suite therefore
  cannot be reported as an exact witness run.

## Verification completed

- multi-fixture MOD contract: 4/4 PASS, including actual-byte GFF header,
  internal UTC `TemplateResRef` and same-count resource-key attacks;
- normal tri-control HAK suite: 1 self-contained PASS, 2 explicit ignored;
- normal composite diagnostic suite: 2 explicit ignored;
- forced exact HAK witness suite: 2/2 PASS;
- forced exact MOD+HAK composite suite: 2/2 PASS;
- offline materializer contract: 2/2 PASS in 35.21 s test time (37.5 s wall),
  including absent/no-overwrite, wrong path/hash/resref and exact four-output
  post-write replay;
- `cargo check -p m2a-core`, `cargo fmt --all -- --check`, and
  `git diff --check`: PASS.

## Live boundary

This record does not materialize, install or launch the diagnostic. It creates
no r32 path and performs no Toolset/NWN action. Before one diagnostic run, the
offline materializer/profile and central 120-second multi-fixture proof binding
must both pass independent adversarial review. The run must then use the shared
operator and one immutable output root, with zero Save/Build/repack/retry.
