# Material world-scale UV and quality gate V1 — offline implementation

Date: 2026-08-04

## Outcome

The shared Material Separation pipeline now supports a world-scale box UV mode
for large fragmented Placeables. Texture scale is authored per material group
as repeats per final authored metre. Disconnected faces in one group share the
same origin and physical scale; only projection-plane seams duplicate vertices.
The previous normalized `MATERIAL_BOX` mode remains available so existing
recipes keep their meaning.

Studio can reuse one uploaded override texture across several independently
assigned material groups. The payload descriptor is emitted once and the core
continues to deduplicate byte-identical TGA resources by SHA-256.

The package report now contains deterministic texture readability diagnostics:
base luminance deviation, an approximate 16x16-mip deviation, retained contrast
and `READABLE`, `LOW_CONTRAST` or `FLAT` status. Material Separation also warns
when source topology is extremely fragmented. Source `doubleSided` is reported
explicitly as `UNSUPPORTED_REPORT_ONLY`; the pipeline does not claim that Aurora
preserves this glTF state.

Studio Review displays these diagnostics for every neutral material binding.
The texture report identifies `MATERIAL_UV_PROJECTION_V1` when projection was
actually applied instead of claiming that source UV0 was unchanged everywhere.

## Offline verification

- `cargo test -p m2a-core --lib`: 114 passed, 3 ignored because they require
  local Git-ignored proof assets.
- `cargo test -p m2a-core --test model_material_separation`: 16 passed.
- `cargo test -p m2a-core --test placeable_pipeline`: 22 passed, 1 ignored
  because it requires an owner-selected local GLB environment variable.
- `cargo check -p m2a-wasm`: passed.
- Studio: 48 test files, 276 tests passed.
- Studio TypeScript typecheck and production WASM/Vite build: passed.

## Candidate boundary

No MOD, HAK, model resref, texture resref, Appearance row or demo lineage was
created or changed by this implementation. The currently visible Toolset
candidate still requires the owner's NWN result. A new visual candidate may be
materialized only after the model-iteration gate admits it.
