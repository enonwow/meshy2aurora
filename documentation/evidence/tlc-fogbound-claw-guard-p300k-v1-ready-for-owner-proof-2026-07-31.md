# TLC Fogbound Claw Guard P300K V1 — ready for owner proof

1. Test-module filename: `cd4cx2if4gswhd4d.mod`
2. Module name shown in Toolset: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora procedural humanoid proof area`

Status: `ready_for_owner_proof`

The agent did not start, adopt or control Aurora Toolset or NWN. The exact
MOD/HAK were installed for the human-owned visual test only.

## Meshy generation and provenance

The owner-selected concept was generated as a humanoid through Meshy 6 with a
300,000-triangle target, T-pose rig and ten purchased animation actions.

- canonical sample: `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1`;
- concept SHA-256:
  `96b784db771791391d5c57da5317762b5072448ac19382f1176e49dbf96a64f2`;
- exact image-to-3D task: `019fb8fe-4614-77e7-b22f-63eb5e460110`;
- exact remesh task: `019fb901-cef6-79a2-be40-9533fd8fe4ff`;
- exact rig task: `019fb904-d98c-7d88-b4ea-be88f06c63e9`;
- exact cost: `70` Meshy credits;
- no second model or paid recovery task was created.

All ten raw action GLBs remain in the canonical sample with task IDs, byte
lengths and SHA-256 values in `manifest.yaml` and
`meshy-run-provenance.json`.

## Continuous death-family correction before candidate freeze

The first Studio build attempt was correctly blocked before any candidate was
materialized:

`M6-ANIMATION-BEHAVIOR-INELIGIBLE: DEATH_FAMILY_BOUNDARY_DISCONTINUOUS:ckdbck->ckdbckdie`

The cause was an invalid composition of two independent Meshy actions:
`187 Knock Down` followed by `8 Dead`. Their boundary is not the same pose and
would recreate the previously owner-observed double fall/jump.

The final source follows the owner-proved
`MESHY_DEAD_CONTINUOUS_DEATH_FAMILY_V2` policy:

- full Meshy `Dead` is the sole moving death source;
- the pipeline moves it to runtime `ckdbck`;
- `ckdbckps`, `ckdbckdie` and `cdead` are continuous 1/30-second terminal
  holds derived from its last pose;
- raw Meshy `Knock Down` is preserved for audit but intentionally excluded
  from final 42-state routing;
- this correction cost zero Meshy credits.

Canonical final source:

- file: `source-death-continuous.glb`;
- bytes: `21,730,520`;
- SHA-256:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`;
- triangles: `297,190`;
- source clips: `9` (`cpause1`, `cwalk`, `crun`, `ctaunt`, `ca1slashl`,
  `ca1slashr`, `cdamagel`, `cguptokdb`, `cdead`).

## Exact application route

The successful candidate passed the real application boundary:

`Studio UI -> Web Worker -> public m2a-wasm -> m2a-core -> owned MDL/TGA/2DA/HAK/MOD writers -> binary readback`

Studio used the Creature `Product (300,000 triangle limit)` profile, `Auto`
detached-accessory stabilization and disabled texture repair so the source
base-color pixels remained unchanged. The Worker returned all exact artifact
buffers; the download surface validated byte lengths and SHA-256 values before
the immutable packet was copied to `proof-output`.

## Offline result

- source triangles: `297,190`;
- written/read-back triangles: `297,190`;
- written vertices: `224,786`;
- binary SkinMesh streams: `14`;
- active joints: `22`;
- deformation: `SKIN`;
- source clips: `9`;
- runtime states: `42/42`;
- behavior candidate: eligible;
- death-family boundary continuity: `PASS`;
- `ckdbck`: full `3.0 s` Meshy death motion;
- `ckdbckps`, `ckdbckdie`, `cdead`: continuous `0.03 s` terminal holds;
- gameplay events: `23/23 PASS`;
- semantic binary MDL diff: empty;
- demo MOD semantic readback: `PASS`.

The spatial-weld accessory audit found one connected primary-body component,
zero detached components and zero changed vertices. It did not rewrite skin
weights.

The safe classic material route preserves the base-color texture. Meshy
`emissiveFactor`, `emissiveTexture`, `doubleSided`, metallic and roughness are
reported as unsupported and are not silently translated to NWN material
features. The runtime model may consequently look less cyan/glowing than the
concept or Meshy viewer; this is a known material-fidelity limitation, not a
visibility or geometry failure.

## Frozen Studio artifacts

Canonical packet:

`proof-output/tlc-fogbound-claw-guard-h1-p300k-v1/studio-export`

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `cd4cx2if4gswhd4d.mod` | 15,210 | `2008a13a921dceeed884f745aad5f4996c42f60fe1f31927bb604e299c2de5ae` |
| `ch4cx2if4gswhd4d.hak` | 44,102,320 | `f3322e9649dae76c38b072dba25c6826acc47b2a65fd3ef5b12bf80b71dc8f1a` |
| `cm4cx2if4gswhd4d.mdl` | 24,617,748 | `7e568cda1a6323d4ef6a285ef5f0dad8a7bdcb2e5390c5a98aaf599f47bf6edc` |
| `ct4cx2if4gswhd4d.tga` | 12,582,956 | `7e24d51726355bf3a4ee2f1429ea85f43a604bc2ee3e079c9fd5a306d4b1426a` |
| `inspection.json` | 1,244,954 | `e88857d654c56180bc3299e255c58972e1bcc8a41f55ea039796afb10951445a` |
| `conversion-manifest.json` | 3,063 | `4c1ea870b99e7e46f801b8cf12d36caf9c9d6cad0a85aa2999273169ddc1f1c0` |
| `summary.json` | 1,730 | `5146b14e77ef8a53ba992d520da8bff2bbc844a618f6aecc30fd1381d666e900` |
| `demo-report.json` | 444 | `910a5d364cf75c0bb3fe2288a1c2cc00e129d7c31736ab1f66b328010eff4aaf` |

Identity:

- model: `cm4cx2if4gswhd4d`;
- texture: `ct4cx2if4gswhd4d`;
- HAK: `ch4cx2if4gswhd4d`;
- MOD: `cd4cx2if4gswhd4d`;
- Area resref: `ca4cx2if4gswhd4d`;
- Creature resref: `cc4cx2if4gswhd4d`;
- Appearance row: `15100`.

## Native installation

Both destinations were absent before installation. Copying used no-overwrite
semantics, after which both destinations were hashed and proved byte-identical
to the frozen Studio artifacts.

| Artifact | Native destination | Verified SHA-256 |
|---|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\cd4cx2if4gswhd4d.mod` | `2008a13a921dceeed884f745aad5f4996c42f60fe1f31927bb604e299c2de5ae` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\ch4cx2if4gswhd4d.hak` | `f3322e9649dae76c38b072dba25c6826acc47b2a65fd3ef5b12bf80b71dc8f1a` |

## Verification ledger

- canonical workspace and Meshy asset-layout guards: `PASS`;
- real Studio UI/Worker/WASM candidate build: `PASS`;
- `m2a-core` model pipeline: `29 passed`, `1` external-witness test ignored;
- binary MDL writer: `43/43 PASS`;
- `m2a-wasm`: `33/33 PASS`;
- Studio typecheck: `PASS`;
- Studio unit/component tests: `234/234 PASS`;
- Local Bridge and strict GLB animation merge: `26/26 PASS`;
- `cargo fmt --all -- --check`: `PASS`;
- `git diff --check`: `PASS`.

The only browser console error was an unrelated missing local `favicon.ico`
request. The build Worker, WASM and artifact downloads reported no error.

## Owner test

Open exact `cd4cx2if4gswhd4d.mod`, then exact Area
`Meshy2Aurora procedural humanoid proof area`.

The single Creature is named `Meshy procedural humanoid`, uses template
`cc4cx2if4gswhd4d`, Appearance row `15100`, and is placed directly in front
of the player at `[10.0, 14.5, 0.0]`.

Please verify separately in Aurora Toolset and NWN:

- `modelVisibility = visible | not_visible | not_tested`;
- `proofCompleteness = verified | failed | missing`;
- idle, walk, run/sprint and both attack states animate;
- the full death motion plays once without an entry jump or a second fall;
- the corpse remains in the terminal pose;
- the cape, claw and armor do not detach or stretch;
- whether the classic diffuse material is visually acceptable without Meshy
  emissive/double-sided features.
