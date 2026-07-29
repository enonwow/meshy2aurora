# Void Crystal Knight — Meshy generation evidence

Date: 2026-07-29
Status: `MATERIALIZED / PIPELINE-READY`

## Result

The owner-selected Void Crystal Knight concept was generated through Meshy
Image to 3D, remeshed to the shared product budget, rigged as a humanoid, and
materialized with Idle plus Meshy's basic Walking and Running animations.

Canonical local source:

`sample-3d/void-crystal-knight-h1-v1/source.glb`

The combined GLB contains:

- 19,704 triangles;
- 24,834 vertices;
- one mesh, one skin and 24 joints;
- `cpause1`, `cwalk` and `crun`;
- SHA-256
  `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`.

The source uses the front, weapon-free crop of the concept sheet. Meshy was
asked for a T-pose and a 20,000-triangle target. The resulting preview preserves
the black segmented armor, violet crystal core and floating crystal accents.

## Canonical Meshy lineage

- local run ID: `18d0254f-1190-4c3e-92a2-c37964da2d56`;
- Image to 3D task: `019fad5e-f4dc-772c-af98-22c52843fafb`;
- rig task: `019fad62-5540-7b53-a3fe-83c2b2517a1a`;
- Idle action 0 task: `019fad63-1f9f-7b83-974e-26042de8788b`;
- canonical cost: 38 credits;
- balance before the canonical run: 702 credits.

The rig task supplied Walking and Running GLBs without separate animation
tasks. The three same-rig animation GLBs were merged with the tested
`merge-animation-glbs.mjs` path and explicit NWN-facing clip names.

## Duplicate paid-run incident

The first background launcher call returned without process state or output,
but the detached runner continued. Treating that as a pre-request failure led
to one manual retry. The retry created a second complete paid lineage before it
failed closed on the existing local Idle file and did not overwrite the
canonical payload.

Duplicate task IDs:

- Image to 3D: `019fad60-09fb-777a-915b-7ebc4529290f`;
- rig: `019fad63-d9f4-7a44-9cda-3218479ea0ed`;
- Idle: `019fad64-aec0-752c-a494-0f90f3a34b5a`.

The duplicate consumed another 38 credits. Total cost was therefore 76 credits
and the observed balance after both lineages was 626. The duplicate remains in
Meshy history, is not selected as a local source, and was not deleted because
deletion is destructive and does not refund credits.

The operational correction is to run paid Meshy jobs only through a yielded
foreground command with an explicit cell handle. A launcher returning no
process state must not be interpreted as proof that a paid request was not
created; task history and balance must be checked before any retry.

## Verification

- Local Bridge tests: 19/19 passed.
- Studio unit/component tests: 210/210 passed.
- Studio TypeScript check: passed.
- Animation GLB merge tests: 2/2 passed.
- Canonical asset layout gate: passed after final manifest readback.
- Aurora Toolset/NWN visual proof: not started; final proof remains
  human-owned.
