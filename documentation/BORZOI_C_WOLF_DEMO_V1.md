# Borzoi `c_wolf` demo V1

Date: 2026-08-20

Status: `ready_for_owner_proof`. This record makes no Toolset or NWN visual-success claim.

## Exact owner handoff

- Test module file: `m2aborzmod1.mod`
- Toolset module name: `Meshy2Aurora Borzoi c_wolf Demo V1`
- Area: `Meshy2Aurora Borzoi Test Area V1`
- Ordered HAK: `m2aborzhak1`
- Creature blueprint: `m2aborzutc1`
- Appearance row: `848`
- Model resref: `m2aborzcre1`
- Texture resref: `m2aborztex1`

The exact module and HAK were installed into the native NWN user `modules` and `hak` directories only after absent-target checks. Destination SHA-256 values were then required to equal their canonical proof-output sources byte for byte.

## Immutable artifact identity

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2aborzmod1.mod` | 15,205 | `abe99cc1fdd9caa79869317e53e68e5475e174f1147ecdf644a2fe75024128e3` |
| `m2aborzhak1.hak` | 20,661,840 | `0833e31c0dcee182d7725c130aa9dff357a02dc6380c63dad634e102b9c26b86` |
| `m2aborzcre1.mdl` | 7,690,476 | `fda66a93b58a13c839cd0a177bf3dc2fcf9ba663655b887e56aeda42d30202bc` |
| `m2aborztex1.tga` | 12,582,956 | `8b8285e273d3079a6651a4a86c5e83493d6a8fa2fb6cdd8ce35fb4b5c6fec893` |

Canonical packet: `proof-output/borzoi-c-wolf-demo-v1-20260820`.

## Source and clean-room boundary

The canonical Meshy source is `sample-3d/borzoi-c-wolf-n1-p60k-v1/source.glb`, SHA-256 `44b5d3c63387ae7587b4de8dc9e4a6866948cd034cd1a1e3a0c0ce69af3e6678`, with 60,780 triangles. Meshy task `01a01fbf-fdb8-77b8-b381-c6480727cd3d` cost 30 credits. No second generated 3D model was requested.

The base-game `c_wolf` resource was inspected read-only from the retail KEY/BIF pair. Its immutable fingerprint is SHA-256 `a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726`, 340,812 bytes. The compatibility facts are 30 ordered node names/parents, classification 4, animation scale 1.0 and 42 animation clips. No retail geometry, bind transform, skin weight, controller or animation payload was persisted or copied into the generated asset. The generated MDL contains zero local animation clips and names `c_wolf` as its supermodel.

All bone placement and skin weights are computed from the owned borzoi surface. The product route preserves all 60,780 source triangles and partitions the weighted geometry into three binary-MDL-safe streams.

## Facing and runtime fixture

The source was visually and structurally classified as glTF positive-Z forward. Creature Basis V3 maps it to Aurora positive-Y forward with determinant `+1` (`CREATURE_BASIS_V3_RESOLVED`). The demo creature is placed at `[10.0, 14.5, 0.0]`, directly in front of the player start, and oriented `[0.0, -1.0]` to face the player. Its active-monster fixture is intended to exercise idle, movement and combat inheritance from `c_wolf`.

Until the owner reports the result, the two proof axes remain:

- `modelVisibility = not_tested`
- `proofCompleteness = missing`

Owner NWN result recorded later on 2026-08-20:

- `modelVisibility = visible`
- `proofCompleteness = verified`
- rig quality verdict: `failed`
- visible defects: front paws not spread, model levitates, inherited movement looks artificial

The exact candidate-bound result and offline diagnosis are recorded in `documentation/evidence/borzoi-c-wolf-demo-v1-owner-nwn-result-2026-08-20.json`.

## Offline verification

The exact source fingerprint, retail reference proof packet, generated MDL semantic readback, HAK resource readback, MOD scene readback, 30-node topology, V3 basis contract, weighted-stream partition and package tests all passed. The owner performs the final visual proof in Toolset and NWN.
