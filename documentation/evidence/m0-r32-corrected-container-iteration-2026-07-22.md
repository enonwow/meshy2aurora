# M0 r32 corrected-container iteration — 2026-07-22

## Decision and scope

R32 is a fresh MOD/Area identity that corrects the runtime creature container rejected in frozen r31. It reuses the exact installed `m2a_m0r31.hak` bytes and therefore changes no HAK, MDL, TGA, `appearance.2da`, model resref, fixture semantics, or model payload.

The admitted delta is exactly:

- `MOD_AREA_IDENTITY`: `m2a_m0r32` / `m2a_m0a32`;
- `RUNTIME_COMPLETE_CREATURE_ENVELOPE`: the generic creature-module builder emits the complete UTC/GIT container required by the independent generic parser.

R31 remains immutable. The independent parser rejects its sparse creature at `area.git.Creature List[0].MaxHitPoints`; the exact rejection requires value `13`. The same parser accepts the r32 container and confirms `MaxHitPoints=13` plus all 28 zero-rank skills.

## Test-first implementation

The forced test `crates/m2a-core/tests/m0_r32_corrected_container.rs` was added first and produced the expected RED compile failure because `build_meshy_r31_corrected_container_module_v1` and `M0_R31_HAK_SHA256` did not exist. The product API was then added to `crates/m2a-core/src/hierarchy_candidate.rs` and the same test passed GREEN.

The new API invokes `build_binary_creature_multi_fixture_module_v1`, then independently reparses its exact output through `inspect_binary_creature_multi_fixture_module_v1`. It is fail-closed on module, Area, singleton HAK, entry, fixture identity, display text, Appearance row, position, and orientation. It neither reads nor emits a replacement HAK.

## Exact artifacts

- generated MOD: `C:\Projects\meshy2aurora\proof-output\m0-r32-corrected-container-20260722\generated\m2a_m0r32.mod`
  - length: 14,743 bytes
  - SHA-256: `a952e05746710c8461e74b38dbf93e6cc5c86314771330d68724579c11693573`
- installed MOD: `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0r32.mod`
  - absent-target installation completed once;
  - byte-identical to the generated MOD;
  - SHA-256: `a952e05746710c8461e74b38dbf93e6cc5c86314771330d68724579c11693573`.
- reused source HAK: `C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\generated\m2a_m0r31.hak`
- reused installed HAK: `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r31.hak`
  - both length: 19,635,900 bytes;
  - both SHA-256: `ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12`.
- no generated or installed `m2a_m0r32.hak` exists.
- lineage contract: `C:\Projects\meshy2aurora\proof-output\m0-r32-corrected-container-20260722\m0-r32-corrected-container-lineage-contract-v1.json`
  - SHA-256: `3ba2e572291fc971e972c839960d894ff4856170de480ab1a23467dcddc5f5b3`.

The single fixture is `m0_fixture`, template `nw_dwarfmerc001`, display text `Meshy M0 binary vertical-slice fixture`, Appearance `15100`, position `[10,14.5,0]`, orientation `[1,0]`. The Area is 2x2 and entry is `[10,10,0]`.

## Proof profiles

- binary bootstrap: `C:\Projects\meshy2aurora\proof-profiles\m0-r32-corrected-container-binary-bootstrap-v1.json`
  - SHA-256: `912de1684ef274e6914620ffc4995b0cc22e50fd7066bc09e781ad8aa13920f8`;
- no-Save Toolset proof: `C:\Projects\meshy2aurora\proof-profiles\m0-r32-corrected-container-toolset-proof-v1.json`
  - SHA-256: `1fe6fabfce4843f56b994ceec684112db09acbf540c19c3810afb4ec0df313a5`.

No r32 runtime profile was created. Toolset and NWN proof axes remain independently `modelVisibility=not_tested`, `proofCompleteness=missing`.

## Verification

Project checks:

- forced `m0_r32_corrected_container` test: PASS;
- `m0-r32-corrected-container-lineage.contract.test.mjs`: PASS with nine negative mutations;
- generic binary creature multi-fixture tests: PASS;
- frozen hierarchy-candidate regression: PASS;
- workspace check, format check, and `git diff --check`: PASS.

Central read-only checks from `C:\Projects\aurora-web`:

- binary bootstrap schema dry-run: `binary_bootstrap_profile_schema_valid`;
- binary bootstrap structural preflight: `binary_bootstrap_structural_readback_valid`;
- binary native geometry dry-run: `binary_native_geometry_plan_valid`.

The central dry-run reports `startsToolset=false`, `startsNwn=false`, `usesGlobalInput=false`, and `savesModule=false`. It did not create the designated live output. Final process state is `nwtoolset=0`, `nwmain=0`.

## Next boundary

Only the exact r32 no-Save Toolset Gate B is next: exact module/HAK/Area/object readback followed by a fresh validated `TScrollBox` capture. Do not run geometry observe/finalize, Save, Build, repack, materialize another lineage, or create a runtime profile before an accepted r32 Gate B packet exists.

## State transition — 2026-07-24

Status: IN_PROGRESS

The boundary above was subsequently crossed by the exact r32 model-proof run.
The immutable candidate bytes did not change:

- MOD SHA-256 remains `a952e05746710c8461e74b38dbf93e6cc5c86314771330d68724579c11693573`;
- ordered HAK SHA-256 remains `ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12`;
- MDL SHA-256 remains `fcfbe7e329d51aef6ccb3ae87b4bfe7db7c5395cd5e8adf24159e73e556b5ab6`.

The fresh exact-object `TScrollBox` proof records Toolset
`modelVisibility=visible`, `proofCompleteness=verified`:

- capture:
  `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-1/toolset/capture.png`,
  SHA-256 `77f1d3fa3324cfd8e3c2c14275e57bec6f15090a7a35d5626750673ad8f1a41d`;
- verdict:
  `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-1/toolset/verdict.json`,
  SHA-256 `ef6a3d5cf0eeda4323f3ea5ab2c1cfb1054b5bf0c3267cab38f86bf03516b62f`.

The same run launched exact r32 in NWN but the capture lane failed before a
runtime PNG. Its preserved blocker has SHA-256
`42ff4967fb99735c94f9345041bbe0ca688edaa8d4c1d2d8de35589c02f1cfd2`.
Therefore NWN remains `modelVisibility=not_tested`,
`proofCompleteness=missing`; this is not a negative model result and does not
admit r33.

The existing route-fix profile was maintenance-pinned, without changing its
ID, output root, or any candidate resource, to the current shared route
manifest SHA-256
`65ae5aa66e81061a1c70df04497be07a3088c8c7c6fad0d3f1ecd34ffb4956e5`.
Its profile SHA-256 is
`2a60ed1f4905c40d6b50780e725edd0b71e01d448029f77efc6d36a38100f90b`;
the public coordinator `preflight` returned
`MODEL_PROOF_120S_PREFLIGHT_VALID`.

The lineage contract was updated to bind the materialized runtime/profile
files and the monotonic Toolset proof without changing the candidate. Its new
SHA-256 is
`02602d06c9437f0d946e6846ee18f0ced327a4b876eb17902406fd197c0e91a2`.

Verification:

- `node tools/m0-r32-corrected-container-lineage.contract.test.mjs` — PASS,
  including 12 negative mutations;
- public model-proof `preflight` — PASS;
- final live state — `nwtoolset=0`, `nwmain=0`;
- native `modules\temp0` remains present and unmodified.

## 2026-07-24 exact recovery live result

The shared public preparation route admitted the immutable r32 candidate on
the same responsive Toolset PID `31700`. Its attempt-2 dry run passed all
profile, MOD, HAK, cache, process, frame, and modal identity gates. The live
route then resolved the exact recovery prompt, the unique localized
non-destructive button `&Tak`, and submitted its one allowed targeted
`PostMessage(BM_CLICK)`.

The prompt remained visible after the bounded readback. The public route
therefore failed closed with
`toolset_verified_temp0_recovery_prompt_did_not_close`; it did not repeat the
action, use global input, Save, Build, change INI/MRU, mutate `temp0`, start
NWN, or allocate r33. Durable evidence:

- `continuation-plan-v2.json`: SHA-256
  `c26801003df36c909c2abb6af9aeac0d7f54e4151aac209ab39b51769ff2ff11`;
- `continuation-blocker-v2.json`: SHA-256
  `f80d0351d16d5ac555f0fe08a76bf8e4ec603a24fb9bb6a2c17bcd9b9a2f0f58`;
- current binary-native manifest: SHA-256
  `651f1515e82d0ae0596dbeb37e4ede7fa9bf2c2ee9e187a6b80968573b58e095`.

Current live state is one responsive, clean, blank Toolset frame on PID
`31700`, the unchanged exact `Confirmation` modal, and zero `nwmain`
processes. Toolset visibility remains the earlier monotonic
`visible/verified` result; NWN remains `not_tested/missing`. This lane failure
does not admit a model iteration.

The current public preparation wrapper has no identity-bound route for
adopting that orphaned `temp0` with zero Toolset processes. Repeating the
unchanged cold `prepare` is forbidden after its existing
`toolset_temp0_requires_clean_start` failure. Resume condition: one clean,
responsive exact `m2a_m0r32.mod` / `m2a_m0a32` Toolset session that the public
wrapper can adopt, after which run exactly one
`prepare → preflight → ready → toolset → runtime → finalize` sequence.

## 2026-07-24 accepted 120-second Toolset-to-NWN proof

Status: ACCEPTED

This section supersedes the earlier in-progress and recovery states above. The
proof was completed through the public `aurora-model-proof-120s` coordinator
after a new attempt started from `nwtoolset=0`, `nwmain=0`. Public preparation
reached formal READY for the immutable r32 candidate and retained the same
Toolset PID `13336`; the timed run was then executed exactly once.

The immutable candidate identity remained:

- proof profile:
  `proof-profiles/m0-r32-corrected-container-model-proof-120s-route-fix-v1.json`,
  SHA-256
  `2a60ed1f4905c40d6b50780e725edd0b71e01d448029f77efc6d36a38100f90b`;
- installed MOD:
  `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0r32.mod`,
  SHA-256
  `a952e05746710c8461e74b38dbf93e6cc5c86314771330d68724579c11693573`;
- installed ordered HAK:
  `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r31.hak`,
  SHA-256
  `ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12`.

The exact selected fixture in the validated Toolset `TScrollBox` produced:

- `modelVisibility=visible`;
- `proofCompleteness=verified`;
- capture:
  `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-2/toolset/capture.png`,
  SHA-256
  `77f1d3fa3324cfd8e3c2c14275e57bec6f15090a7a35d5626750673ad8f1a41d`;
- verdict:
  `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-2/toolset/verdict.json`,
  SHA-256
  `712884564df564c20620273687f67b38bca905a77b36f180ef2f2b79a101d96e`.

The same exact module, HAK, and model lineage was then tested in NWN on runtime
PID `36868`. The engine log contains
`[Fri Jul 24 11:49:47] Loading Module: m2a_m0r32`. The fresh runtime capture
shows the player and the clear, unobstructed forward ground where the fixture
at `[10,14.5,0]` was deliberately placed, but the expected model is absent.
The candidate-bound runtime result is therefore:

- `modelVisibility=not_visible`;
- `proofCompleteness=verified`;
- capture:
  `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-2/runtime/capture.png`,
  SHA-256
  `e976d849456bbeb5f24d98ad9e95c742508f43d3c6afee3f68fa84eebaec5f76`;
- verdict:
  `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-2/runtime/verdict.json`,
  SHA-256
  `e682a0f4c1a268bd8977e0f40c8d11ee65c04d0f878952847b5e90b96a46064e`.

The accepted proof is
`proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-2/final-model-proof.accepted-at-543958529728.json`,
SHA-256
`1c787faf0780a44637264051ab4e049b02266d512db15145789113cabad53345`.
The QPC start counter was `543301196929`, the accepted publish counter was
`543958529728`, and the frequency was `10000000`. Thus publication completed
after `(543958529728 - 543301196929) / 10000000 = 65.7332799` seconds, within
the 120-second live budget. The provisional `final-model-proof.json` remains
the intentionally pending pre-acceptance packet; the separately named and
hashed accepted file is authoritative.

The shared proof skills were repaired centrally where public transitions were
missing: exact installed MRU continuation and hidden-frame post-Test Module
cleanup. No project-local UI wrapper, runner, or native leaf sequence was
added. Public cleanup closed NWN and Toolset through the skill route, including
Toolset `WM_CLOSE` without a force fallback:

- cleanup record:
  `proof-output/m0-r32-corrected-container-20260722/live/model-proof-120s-run-2/cleanup.json`,
  SHA-256
  `6a9af95a7783a8e67d385ab9afc7b20e761f69e83205c05cd4e755778200cac2`;
- independently confirmed final state: `nwtoolset=0`, `nwmain=0`.

Conclusion: r32 is visible and verified in Aurora Toolset but not visible and
verified in NWN runtime. This is the fresh exact-candidate visual failure
required by the model-iteration gate. It satisfies the visual prerequisite for
a possible r33; r33 has not been created. Before any r33 artifact is allocated,
the diagnosed runtime cause and the minimal intended artifact delta must be
recorded durably as required by the remaining iteration-gate conditions.
