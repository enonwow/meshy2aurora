# M0 Area tileset correction — 2026-07-18

## Scope

The runtime target remains the last completed Meshy API asset:

- Meshy task: `019f7566-b572-7d4f-abe8-56383d01a016`
- source: `proof-output/m0-meshy-golem-20260718/generated/source.glb`
- source SHA-256: `aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1`

## Failed live gate

The installed, original `m2a_m0_proof.mod` opened its exact Area
`m2a_m0proof_area` in the Toolset. The `TfrmViewerArea` and its visible,
enabled `TScrollBox` were present on `\\.\DISPLAY1`, but the viewport was
uniformly blank. The capture is
`proof-output/m0-live-20260718/toolset-area-settled.png`.

This is `failed` for the Toolset viewport gate. NWN was not launched.

## Root cause (confirmed)

The failed module's `tdc01` `Tile_List` was:

| Index | Tile_ID | Orientation |
| ---: | ---: | ---: |
| 0 | 113 | 2 |
| 1 | 113 | 3 |
| 2 | 113 | 1 |
| 3 | 0 | 0 |

The native Toolset-created reference modules `m2a_test.mod` and
`m2a_test_tortoise.mod` use standalone `tdc01` tile `5`, rather than that
synthetic partial-tile sequence. The parsing shape (43 ARE fields, 2x2, four
tiles) was therefore insufficient evidence of a renderable Area.

## Correction prepared

`crates/m2a-core/src/proof_module.rs` now emits four `tdc01` tile-5 records at
orientation 0 and asserts that contract in its own readback test. The isolated
M0 materializer route was restored so it can regenerate the original proof
package shape without substituting an H1 module.

Fresh package, built from the exact Meshy GLB above:

- packet: `proof-output/m0-meshy-golem-20260718-v2`
- MOD SHA-256: `bbdbc8f2d741c1b877d5cd6eb63fe5eb0d91f4bf114cedff82a59192f7af3785`
- M0 model SHA-256: `971a15b0990fb0f308be929e23fc3fb70c8c7342c0c39246613687f6c6c84e66`
- tile readback: four entries `(Tile_ID=5, Tile_Orientation=0)`

Validation completed:

```text
cargo test -p m2a-core proof_module --lib
cargo test -p m2a-core --test model_pipeline static_m0_proof_packet_writes_only_its_own_runtime_names
node --test tools/m2a-aurora-proof.test.mjs
```

All passed.

## Remaining live action

The current Toolset process still owns and locks the failed original MOD. A
fresh Toolset session is required to load the corrected package and perform the
mandatory Toolset viewport gate before `Build -> Test Module` and NWN runtime
proof. It must not be replaced or reloaded through a second module-open action
in the existing session.
