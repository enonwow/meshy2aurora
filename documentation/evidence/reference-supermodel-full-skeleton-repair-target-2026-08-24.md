# Reference-supermodel full-skeleton repair — exact offline target

Date: 2026-08-24

Status: `offline_pipeline_repaired / exact_c_wolf_product_blocked_by_motion_quality`

## Exact target selected before implementation

The repair oracle for the next Borzoi product line is bound to:

- asset: `sample-3d/borzoi-meshy-manual-p1997k-v1`;
- role: `pipeline-product-budget-variant`;
- payload: `source-p300k.glb`;
- byte length: `29,889,104`;
- triangles: `300,000`;
- vertices: `233,832`;
- SHA-256:
  `f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda`.

This is the exact source used by the V3–V8 Borzoi line and explicitly selected
by the owner for the next `c_wolf`-inherited product line. It is therefore the
only source that can close the real repair oracle.

The historical
`sample-3d/borzoi-c-wolf-n1-p60k-v1/source.glb` (60,780 triangles, SHA-256
`44b5d3c63387ae7587b4de8dc9e4a6866948cd034cd1a1e3a0c0ce69af3e6678`)
remains valid provenance for V1/V2, but it is not admissible evidence for this
repair and must not be substituted or mixed into its result.

## Repair boundary

This work changes the generic offline pipeline and its tests only. It does not
allocate a V9 resref, materialize a new MDL/HAK/MOD lineage, install artifacts,
or run Aurora Toolset/NWN. Final runtime proof remains human-owned.

Completion is not established by this target declaration. It requires the
full carrier, skin influence, inherited joint×clip motion, zero-seam and product
admission gates described by the owner to pass on the exact payload above.

The current offline result and remaining fail-closed blockers are recorded in
`reference-supermodel-full-skeleton-repair-offline-result-2026-08-24.md`.
