# Studio viewport grounded-grid fix — 2026-08-04

## Symptom

The Studio grid was always rendered at global `Y=0`. Meshy GLBs commonly keep
their origin near the centre of the asset, so part of a model could appear
below the viewport grid. The exact TLC ship source
`61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278`
has source bounds `Y=-0.549749..0.544566`, causing the old grid to cut through
the model.

## Cause

Both the static `SceneViewport` grid and the debug-overlay grid were created at
the Three.js default position `Y=0`. Camera fitting used the model bounds, but
grid placement did not.

## Resolution

- The grid plane is placed just below `bounds.min.y` with a scale-relative
  epsilon to prevent visible intersection and z-fighting.
- The source model transform is not changed.
- Pivot axes remain at the real origin `0,0,0`.
- The same grounding rule is shared by the static viewport and
  `SceneOverlayRuntime` debug overlays.

## Verification

- Regression test proves that a model below `Y=0` is not moved, its pivot axes
  remain at the origin, and the grid is below its world-space bounds.
- `sceneOverlays.test.ts` and `MaterialSeparationEditor.test.tsx`: 6/6 passed.
- Studio TypeScript typecheck passed.
- Live local Studio reload of the exact TLC ship shows the construction
  supports resting above the grid.

Visual capture:
`artifacts/material-separation/tlc-ship-under-construction-v2/recipe-detailed-v1/studio-detailed-material-separation-grounded-grid.png`
