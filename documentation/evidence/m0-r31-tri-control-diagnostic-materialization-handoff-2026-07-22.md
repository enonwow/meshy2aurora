# M0 r31 tri-control diagnostic — materialization and proof handoff (2026-07-22)

## Status and scope

The exact project-owned tri-control diagnostic was materialized once, replayed,
installed by create-new copy, and admitted by the central offline preflights.
No Toolset or NWN process was started. The future live proof remains unrun:

- `modelVisibility=not_tested` for Toolset and NWN;
- `proofCompleteness=missing` for Toolset and NWN;
- `nwtoolset=0`, `nwmain=0` at the end of preparation;
- `proof-output/m0-r31-tri-control-diagnostic-20260722/live` is absent.

This diagnostic is not a new model iteration. It preserves the exact r31 M0
MDL/TGA bytes and adds the already project-owned H1 v20 positive control plus
the stock appearance row 102. Its generated contracts remain
`diagnosticOnly=true`, `runtimeAdmissible=false`, `resolverVerified=false`, and
`rendererVerified=false` until candidate-bound visual evidence exists.

## Canonical guard and exact materialization

The required guard passed before the first write:

```powershell
Set-Location C:\Projects\meshy2aurora
powershell -NoProfile -ExecutionPolicy Bypass -File .\assert-canonical-workspace.ps1
# canonical-workspace-ok: C:\Projects\meshy2aurora
```

The output directory was required absent before the one materializer call:

`C:\Projects\meshy2aurora\proof-output\m0-r31-tri-control-diagnostic-20260722\generated`

Exact command:

```powershell
$generated = 'C:\Projects\meshy2aurora\proof-output\m0-r31-tri-control-diagnostic-20260722\generated'

cargo run -p m2a-core --quiet --example materialize_tri_control_diagnostic -- `
  --outDir $generated `
  --module-resref m2a_diag01 `
  --area-resref m2a_diaga01 `
  --hak-resref m2a_diagh01 `
  --base-appearance 'C:\Projects\meshy2aurora\local-reference-assets\appearance.2da' `
  --base-appearance-sha256 815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a `
  --r31-model 'C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\generated\m2a_m0p01.mdl' `
  --r31-model-sha256 fcfbe7e329d51aef6ccb3ae87b4bfe7db7c5395cd5e8adf24159e73e556b5ab6 `
  --r31-texture 'C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\generated\m2a_m0t01.tga' `
  --r31-texture-sha256 079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b `
  --h1-appearance 'C:\Projects\meshy2aurora\proof-output\meshy-h1-nwn-runtime-rigid-isolation-v20\generated\appearance.2da' `
  --h1-appearance-sha256 e50e2f8c5fc42771f5c433dd3984b0f8e740938d6f722adf838942f1f6342f8e `
  --h1-model 'C:\Projects\meshy2aurora\proof-output\meshy-h1-nwn-runtime-rigid-isolation-v20\generated\m2a_m6p01.mdl' `
  --h1-model-sha256 6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd `
  --h1-texture 'C:\Projects\meshy2aurora\proof-output\meshy-h1-nwn-runtime-rigid-isolation-v20\generated\m2a_m6t01.tga' `
  --h1-texture-sha256 ab8f11c7f448801d905594802a223a1ed5969bc5d09ce06e6588cbe83b6a86b4
```

The materializer returned
`TRI_CONTROL_DIAGNOSTIC_MATERIALIZED_OFFLINE_ONLY` and published exactly four
files only after its deterministic project-owned replay passed:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `m2a_diag01.mod` | 21,053 | `40282440091c8d778b8b21f715ad9f4c5f9bff998c4605a1769f5fd156a00ff3` |
| `m2a_diagh01.hak` | 32,776,373 | `f538734743a76d20ef1fc7c294f4029940073d76d50ded174ac3b30a6fc74d6d` |
| `tri-control-diagnostic-contract-v1.json` | 5,670 | `6349591b83f3d699020b5aaea43fb25e2ca08d39b756923ab62c93546d22f468` |
| `tri-control-diagnostic-profile-v1.json` | 5,114 | `2ed479e24d461a495824a0ee2e93b648b448dd165527dc796775715104e777d3` |

Independent central ERF/GFF readback then confirmed:

- module `m2a_diag01`, Area `m2a_diaga01`, 2x2 / 4 tiles;
- singleton ordered HAK `[m2a_diagh01]`;
- entry `[10,10,0]` facing `+Y`;
- exact three fixtures in the MOD and their UTC resources;
- exact five HAK resources in this order:
  `appearance:2017`, `m2a_m0p01:2002`, `m2a_m0t01:3`,
  `m2a_m6p01:2002`, `m2a_m6t01:3`.

Resource identities:

| Resource | Bytes | SHA-256 |
| --- | ---: | --- |
| `appearance.2da` | 6,901,545 | `3f7e7f6115e3f2a9917aadccb7e16b9708efb9d87b372cc84b4c56dc991d0448` |
| `m2a_m0p01.mdl` | 151,328 | `fcfbe7e329d51aef6ccb3ae87b4bfe7db7c5395cd5e8adf24159e73e556b5ab6` |
| `m2a_m0t01.tga` | 12,582,956 | `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b` |
| `m2a_m6p01.mdl` | 557,268 | `6e34e7f7a57ac5ea33897cf64e1f37798e47f9fd632e48827037bb328d193fbd` |
| `m2a_m6t01.tga` | 12,582,956 | `ab8f11c7f448801d905594802a223a1ed5969bc5d09ce06e6588cbe83b6a86b4` |

## Safe native installation

Both native destinations were absent immediately before copying. The copy used
the .NET `File.Copy(source, destination, false)` no-overwrite contract. The
installed bytes were rehashed after copying:

| Artifact | Installed path | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_diag01.mod` | 21,053 | `40282440091c8d778b8b21f715ad9f4c5f9bff998c4605a1769f5fd156a00ff3` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_diagh01.hak` | 32,776,373 | `f538734743a76d20ef1fc7c294f4029940073d76d50ded174ac3b30a6fc74d6d` |

Source and installed byte identities match exactly. Never overwrite or replace
these destinations in a later run.

## Declarative proof profiles

| Profile | Bytes | SHA-256 |
| --- | ---: | --- |
| `proof-profiles/m0-r31-tri-control-diagnostic-binary-bootstrap-v1.json` | 1,253 | `6844c41b1cf790da850d8f3233b4603555a08b6880d27b136c65276fe3c4e150` |
| `proof-profiles/m0-r31-tri-control-diagnostic-toolset-proof-v1.json` | 618 | `6f1130ab29755718dfbc268f4ea73cf4613d9437b087b7bf6d09e78e520c4522` |
| `proof-profiles/m0-r31-tri-control-diagnostic-runtime-v1.json` | 1,255 | `786cd2b4ccc8d0a794f1d66416289b4dd4d32d1bb15250792f8789c1fa1a0752` |
| `proof-profiles/m0-r31-tri-control-diagnostic-model-proof-120s-v1.json` | 2,765 | `41627822c46ca04b4d4fdeefd3e346a05f890cdf3c4b0de2be3e032ff7193ced` |
| `proof-profiles/m0-r31-tri-control-diagnostic-multi-fixture-post-run-template-v1.json` | 5,903 | `d6c25d873d7c55d05690febff58d21fe92a1eb163c47ddd65ec67dd1408c1111` |

The executable base profile pins the current central route manifest:

`C:\Projects\aurora-web\skills\aurora-model-proof-120s\references\route-manifest-v1.json`

SHA-256:
`bfb3e67c876a989db367e0d9cc72b12339f0892394c304620ae5bd023d80bcce`.

The Toolset target is `candidate_m0`, native tree text
`Exact r31 M0 candidate`, occurrence `0`, with `TScrollBox`, `noSave=true`.
The runtime profile declares all three fixtures, but it is only an input to the
central coordinator; it is not independent runtime admission or proof.

The multi-fixture file is deliberately a non-executable post-run template:

- version `m2a-multi-fixture-diagnostic-post-run-template/v1`;
- `status=awaiting_accepted_base_run` and `executable=false`;
- `publishedAcceptanceTrustRootId=null`;
- `acceptedBaseRun.clock=null`;
- `acceptedBaseRun.finalPacket=null`;
- `acceptedBaseRun.acceptance=null`.

These values must not be guessed. After a successful atomically accepted base
run, a central owner must publish a code-owned trust root for that exact graph.
Only then derive a new immutable executable
`aurora-model-proof-120s-multi-fixture-diagnostic-profile/v1` from the template,
filling the three exact artifact identities and the published trust-root ID.

The post-run runtime ROIs follow the exact spatial contract rather than fixture
array order. With entry `[10,10,0]`, direction `+Y`, and lateral offsets
`-5 / 0 / +5`, the immutable image order is stock control on the left, M0 in
the center, and H1 on the right. The contract test derives this ordering from
the binary diagnostic readback and rejects both ROI swaps and fixture-ID swaps:

```powershell
node tools\m0-r31-tri-control-diagnostic-post-run-template.contract.test.mjs
# m0_r31_tri_control_diagnostic_post_run_template_contract_valid
```

Test SHA-256:
`81e3850080f8b4f91456b4101655ce3fc7008fe921a4e3627eb9364d5bb1c448`.

## Offline central results

The following commands all passed and declared no Toolset/NWN/global-input
action:

```powershell
Set-Location C:\Projects\aurora-web
$binary = 'C:\Projects\meshy2aurora\proof-profiles\m0-r31-tri-control-diagnostic-binary-bootstrap-v1.json'
$runtime = 'C:\Projects\meshy2aurora\proof-profiles\m0-r31-tri-control-diagnostic-runtime-v1.json'
$model = 'C:\Projects\meshy2aurora\proof-profiles\m0-r31-tri-control-diagnostic-model-proof-120s-v1.json'
$live = 'C:\Projects\meshy2aurora\proof-output\m0-r31-tri-control-diagnostic-20260722\live'

node backend\scripts\validate-aurora-toolset-binary-module-bootstrap.mjs --mode dry-run --profile $binary
# binary_bootstrap_profile_schema_valid

node backend\scripts\validate-aurora-toolset-binary-module-bootstrap.mjs --mode preflight --profile $binary
# binary_bootstrap_structural_readback_valid

node backend\scripts\aurora-toolset-binary-module-native-geometry.mjs dry-run --profile $binary --outDir (Join-Path $live 'binary-native-geometry')
# binary_native_geometry_plan_valid; no mutation performed

node backend\scripts\aur-s07-runtime-execution.mjs dry-run --profile $runtime --outDir (Join-Path $live 'runtime')
# dry-run PASS; startsNwn=false, usesGlobalInput=false

node backend\scripts\aurora-model-proof-120s.mjs preflight --profile $model
# MODEL_PROOF_120S_PREFLIGHT_VALID; startsClock=false, startsToolset=false, startsNwn=false
```

The native-geometry dry-run prints `geometry-observe` as a generic next step.
It is not authorized for this immutable no-Save diagnostic and must not be run.

Exact project-owned replay tests:

```powershell
Set-Location C:\Projects\meshy2aurora
$env:M2A_REQUIRE_RUNTIME_WITNESSES = '1'
cargo test -p m2a-core --test tri_control_diagnostic -- --ignored --nocapture
# 2 passed
cargo test -p m2a-core --test tri_control_hak -- --ignored --nocapture
# 2 passed
Remove-Item Env:M2A_REQUIRE_RUNTIME_WITNESSES
```

## Future live handoff

Do not use leaf atoms, `geometry-observe`, Save, Build, repack, Focus,
Properties, camera work, retry, or a second Toolset. Use the current shared
`aurora-model-proof-120s` skill and only its public preparation wrapper and
coordinator.

Before live work, rehash the installed MOD/HAK, all five resources, every
profile, and the current route manifest. Rerun model-proof `preflight`. If the
central route manifest changed, refresh the declarative model-proof profile and
its dependent template hash after the canonical guard; do not run a stale
profile.

Preparation remains outside the 120-second SLO:

```powershell
Set-Location C:\Projects\aurora-web
$model = 'C:\Projects\meshy2aurora\proof-profiles\m0-r31-tri-control-diagnostic-model-proof-120s-v1.json'
$prep = 'C:\Projects\meshy2aurora\proof-output\m0-r31-tri-control-diagnostic-20260722\live\model-proof-120s-prep-1'

node backend\scripts\prepare-aurora-model-proof-120s.mjs prepare --profile $model --prepOutDir $prep
node backend\scripts\aurora-model-proof-120s.mjs preflight --profile $model
node backend\scripts\aurora-model-proof-120s.mjs ready --profile $model
```

After `MODEL_PROOF_120S_READY`, one proof owner follows the skill's exact
measured sequence: `toolset`, one image verdict, immediate same-lineage
`runtime` only if Toolset is visible, one runtime verdict, and `finalize`.
The final multi-fixture diagnostic profile and verdict are post-run work and
must bind the one accepted runtime PNG; they are not prerequisites for the base
run and cannot be fabricated in advance.
