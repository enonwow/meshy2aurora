# Reference-relative Item part scaling

Date: 2026-08-04
Status: implemented and offline-verified

## Decision

Item part size editing is a reference-fit operation, not an unrestricted mesh transform.
The Studio derives the ordered Bottom/Middle/Top slot frames from the selected Aurora
reference appearance and lets the user change axial length only where the extracted
slot contract permits extension. A locked attachment or connector end never moves.

For the current ModelType 2 longsword profile:

- Bottom and Middle remain at 100% because both axial ends are reference-locked.
- Top supports 50–200% axial scaling and stays anchored at its connector-side minimum.
- every manual change invalidates the previous fit report and blocks Build;
- `Validate fit` runs the authoritative worker/core fitter with explicit ordered scale
  factors and restores Build only after a passing report;
- `Discard changes` restores the last validated transforms, while `Reset part` returns
  the selected extensible part to 100% and requires validation again.

The viewport can show the 100% Aurora slot frames together with the transformed Meshy
parts. This overlay is diagnostic and is included in camera framing, but it does not
replace the worker-produced fit report.

## Aurora Item Properties presentation

The composed viewport is an acceptance view, so its initial camera follows the Aurora
Item Properties coordinate presentation instead of a generic Three.js engineering
scene. After the product basis conversion `P(x,y,z)=(x,z,y)`, Aurora depth is Three X,
item width is Three Y, and axial length is Three Z. The viewport therefore:

- uses an orthographic camera looking straight down model depth;
- uses Three Z as screen-up, placing the blade above the grip;
- frames Three Z vertically and Three Y horizontally from the composed/reference bounds;
- removes the world grid, which previously made the correctly authored Item look as if
  it were lying on the ground;
- separates exploded parts along screen-horizontal Three Y.

This is only a camera and viewport-layout contract. It never adds a compensating model
rotation and never changes the validated controller transforms written to the MDLs.

## Contract

`FIT_ITEM_PARTS.targetAxialScaleFactors` is optional and ordered exactly like the
resolved ModelType 2 part sources. Missing values preserve the existing 100% behavior.
Each supplied factor must be finite and in the inclusive `0.5..=2.0` range. A value
other than `1.0` for a slot without an extensible axial end fails closed with
`ITEM-FIT-REFERENCE-SCALE-NOT-ALLOWED`.

## Verification

- `cargo test -p m2a-core --test item_reference_profile`
- `cargo test -p m2a-wasm --lib --no-run`
- `npm run typecheck`
- `npm test -- --run src/features/item src/worker`
- `npm run build`
- browser smoke test with the canonical three Meshy GLBs and exact retail
  `WSwLs_b_023`, `WSwLs_m_063`, and `WSwLs_t_023` BIF payloads: Top 150% validates,
  keeps the connector anchor, and re-enables Build; browser console errors: none.

This is offline application verification only. It does not claim Aurora Toolset or NWN
visual acceptance, which remains owner-owned under the project rules.
