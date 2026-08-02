# `m2a_ship3_mod.mod` — ready for owner proof

Toolset module name: `Meshy2Aurora Wooden Sailboat Preserve Demo`

Area name: `Meshy2Aurora Wooden Sailboat Preserve Harbor`

Status: `ready_for_owner_proof`

## Goal and result

The owner requested a simple, optional Placeable geometry-cleanup switch that
is disabled by default. With the option disabled, the pipeline now preserves
every finite, non-collinear microtriangle and removes only faces that cannot
form a valid Aurora face plane. The former fixed absolute-epsilon cleanup is an
explicit experimental opt-in with a UI warning that it may create holes.

The exact wooden-sailboat source passed the source-bound two-million-triangle
exception with aggressive cleanup disabled:

- source triangles: `1971350`;
- output triangles: `1971346`;
- truly invalid repeated-index/zero-area faces removed: `4`;
- valid microtriangles removed: `0`;
- aggressive cleanup enabled: `false`;
- binary-MDL render streams: `91`;
- largest stream: `21845` triangles;
- MDL semantic readback: passed.

## Exact candidate identity

- proof packet: `C:\Projects\meshy2aurora\proof-output\wooden-sailboat-deck-placeable-preserve-v2-20260731`
- source GLB SHA-256: `5085db9399eb7d30ddcb6c65b51748f4cabc5ed10cdf7ba912cdf2ef62e1dabe`
- MOD: `m2a_ship3_mod.mod`
- MOD SHA-256: `bddb7eb65c177a9e4b513dabf09fcf80cecd79f10921b3d08eeef9e6161238c3`
- HAK: `m2a_ship3_hak.hak`
- HAK SHA-256: `3b7968812dc94f2b511df904cf3fc1273d14ff540b594e4220f3e4179d73775f`
- HAK bytes: `140023461`
- model resref: `m2a_ship3_mdl`
- model SHA-256: `11794831dfc74821d7d3cde1c8dd6719ac92272ab4f673b4ff54b47922ca9d92`
- texture resref: `m2a_ship3_tex`
- blueprint resref: `m2a_ship3_utp`
- object tag: `m2a_ship3_wooden_sailboat`
- Appearance row: `16500`
- ordered HAK list: `m2a_ship3_hak.hak`
- placement: `(10.0, 14.5, 0.0)`, bearing `0.0`

## Geometry and scale

- uniform scale: `8.0`;
- dimensions: `15.211489 x 11.062224 x 6.844000 m`;
- grounded minimum Z: `0.0`;
- manual mesh or texture edits: `false`.

The different bounds relative to the legacy-cleaned candidate are expected:
the preserved microgeometry includes source extrema that the old cleanup had
deleted.

## Implementation

- `StaticPlaceableBuildOptionsV1.experimentalAggressiveGeometryCleanup`
  defaults to `false` and is strict-schema serialized through Worker/WASM.
- The default Placeable source sanitizer and MDL writer use the exact finite,
  non-collinear policy.
- The legacy fixed `1e-5` cross-length threshold runs only when the explicit
  experimental option is enabled.
- Studio shows the unchecked checkbox `Experimental aggressive geometry
  cleanup` and warns that the trial option may create holes or erase dense
  detail.
- The package report records the selected boolean.
- Oversized semantic readback remains source-bound and uses the exact approved
  readback byte ceiling; normal product parser and triangle limits are not
  widened.

## Verification

| Check | Result |
|---|---|
| exact vs aggressive sanitizer unit test | PASS |
| `m2a-core --lib` | PASS — 88 passed, 2 ignored |
| Placeable integration | PASS — 13 passed, 1 ignored |
| WASM V3 option parity test | PASS |
| Studio TypeScript typecheck | PASS |
| Studio source/App tests | PASS — 29 passed |
| real Worker/WASM integration | PASS — 9 passed, 2 skipped |
| `wasm-pack build --target web` | PASS |
| exact source materialization and MDL/HAK/MOD readback | PASS |

## Native installation

The new destinations were absent before installation. The exact generated
artifacts were copied without overwrite and verified byte-identical afterward:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_ship3_mod.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_ship3_hak.hak`.

No Aurora Toolset or NWN process was started, adopted, controlled or captured.
The owner performs the visual proof using the exact module, Area and object
identity above.

## Proof state

- Toolset `modelVisibility`: `not_tested`;
- Toolset `proofCompleteness`: `missing`;
- NWN `modelVisibility`: `not_tested`;
- NWN `proofCompleteness`: `missing`.

The previous `m2a_ship2_*` candidate and its owner screenshot remain immutable
historical evidence of the legacy aggressive-cleanup failure.
