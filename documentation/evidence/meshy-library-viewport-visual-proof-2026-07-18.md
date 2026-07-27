# Meshy library and viewport visual proof — 2026-07-18

## Scope

Proof of the Meshy Lab right-side model grid and the separate Meshy GLB
viewport, embedded below the primary Studio masthead and workflow. This is a
local, synthetic proof only; it does not create, charge or poll a real Meshy
task.

## Fixture and setup

- GLB fixture: `sample-3d/s1-static-prop-1500/source.glb`.
- Path amendment 2026-07-27: canonical relocation only; fixture bytes and
  SHA-256 used by this proof did not change.
- Synthetic loopback Bridge: `tools/meshy-local-bridge/visual-proof-server.mjs
  --visual-proof`.
- The synthetic record is a completed `text-to-3d-refine` task with one
  thumbnail and one GLB. Both fixture URLs include a synthetic private query
  parameter to verify that the browser never needs that URL.
- Studio ran with `VITE_MESHY_LAB=1` and
  `VITE_MESHY_BRIDGE_ORIGIN=http://127.0.0.1:43120`.

## Observed result

1. The right-side grid rendered a card for **Verified stone lantern proof
   asset**.
2. Clicking the card loaded its GLB through the local Bridge and opened the
   separate central Meshy viewport.
3. The viewport rendered the actual lantern fixture with Meshy-only orbit and
   zoom controls. It deliberately contains no Aurora grid, axes, debug drawer
   content, or generic Aurora viewport overlays.
4. No signed GLB or thumbnail URL appeared in the Studio UI or the Bridge's
   history JSON contract.

## Automated gates

- `npx vitest run src/features/meshy/bridge.test.ts src/features/meshy/MeshyLab.test.tsx`:
  12 tests passed, including opening a verified library asset in the Meshy
  viewport.
- `node --test bridge.test.mjs`: 13 tests passed, including local thumbnail
  proxying without exposing signed URLs.
- `npm run typecheck`: passed.
- `git diff --check`: passed.

## Remaining boundary

The visual proof uses a repository-owned GLB fixture and a synthetic Bridge;
it is not evidence of a paid Meshy generation. Real Meshy history thumbnails
are only served through the same local Bridge MIME/size gate and remain
unavailable when Meshy does not return a thumbnail.
