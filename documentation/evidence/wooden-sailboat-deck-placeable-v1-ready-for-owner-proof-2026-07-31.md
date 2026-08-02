# `m2a_ship2_mod.mod` — ready for owner proof

Toolset module name: `Meshy2Aurora Wooden Sailboat Demo`

Area name: `Meshy2Aurora Wooden Sailboat Harbor`

Status: `owner_nwn_proof_received_visual_quality_failed`

## Exact candidate identity

- proof packet: `C:\Projects\meshy2aurora\proof-output\wooden-sailboat-deck-placeable-v1-20260731`
- source GLB SHA-256: `5085db9399eb7d30ddcb6c65b51748f4cabc5ed10cdf7ba912cdf2ef62e1dabe`
- MOD: `m2a_ship2_mod.mod`
- MOD SHA-256: `72c7c552f3cf46ba676419d06adb6d0b5ae1f59e90e9f1da5293ae3f8f9f7c48`
- HAK: `m2a_ship2_hak.hak`
- HAK SHA-256: `2665dd0155a09ea53d0e7b9c27a38e123820a6a00ae0dea5a409c2a91efbb3b3`
- model resref: `m2a_ship2_mdl`
- texture resref: `m2a_ship2_tex`
- blueprint resref: `m2a_ship2_utp`
- object tag: `m2a_ship2_wooden_sailboat`
- Appearance row: `16500`
- ordered HAK list: `m2a_ship2_hak.hak`
- placement: `(10.0, 14.5, 0.0)`, bearing `0.0`

## Geometry and scale

- Meshy-declared source triangles: `1971350`
- triangles removed by the legacy absolute-epsilon policy: `1751244`
- truly invalid repeated-index/zero-area triangles: `4`
- valid microtriangles incorrectly removed by that policy: `1751240`
- Aurora-safe output triangles: `220106`
- binary-MDL render streams: `11`
- largest render stream: `21845` triangles
- uniform demo scale: `8.0`
- final dimensions: `15.193872 x 11.049480 x 6.841168 m`
- grounded minimum Z: `0.0`
- manual mesh or texture edits: `false`

The source-bound exception admitted up to 128 MiB and 2 million raw triangles
for this exact SHA only. Normal product limits remain unchanged. Sanitization,
grounding, scale application, stream partitioning and package generation were
performed by the pipeline.

## Offline verification

- MDL: passed
- PWK: passed; ASCII readback passed
- `placeables.2da`: passed
- UTP and custom palette ITP: passed
- Area GIT/GIC: passed
- HAK/MOD package readback: passed
- default core tests: `87 passed`, `2 ignored`, `0 failed`
- Placeable integration tests: `13 passed`, `1 ignored`, `0 failed`
- `modelVisibility`: `not_tested`
- `proofCompleteness`: `missing`

## Native installation

The exact generated artifacts were installed without overwrite and then
independently hash-verified byte-identical:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_ship2_mod.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_ship2_hak.hak`

No Aurora Toolset or NWN process was started, adopted, controlled or captured.
The owner performs the visual proof using the exact module, Area and object
identity above.

## Owner NWN visual result — 2026-07-31

- owner-supplied screenshot: `documentation/evidence/wooden-sailboat-deck-placeable-v1-nwn-owner-proof-2026-07-31.png`
- screenshot SHA-256: `6f9f081faa7a45ca931b9bb5ecfc7fae084e37c27837be73f949e013e1509ca3`
- NWN `modelVisibility`: `visible`
- NWN `proofCompleteness`: `verified`
- Toolset `modelVisibility`: `not_tested`
- Toolset `proofCompleteness`: `missing`
- separate visual-quality verdict: `failed`

The ship is clearly rendered at the expected large scale, so this is not a
missing HAK, Appearance mapping, placement or model-visibility failure. The
owner-visible runtime image has extensive holes, fragmented strips and surface
speckling.

The initial diagnosis that the source contained `1751244` genuinely degenerate
triangles was disproved by a direct A/B measurement on the immutable source.
All `1971350` face cross products are finite. Only `4` faces have exactly zero
area, all four through repeated indices. The remaining `1751240` faces removed
by the Placeable route are valid, finite, non-collinear microtriangles whose
cross-product length is positive but at most the legacy fixed `1e-5` cutoff.

The primary diagnosed cause of the fragmented runtime image is therefore the
Placeable route's scale-dependent legacy sanitizer, not defective Meshy
topology and not the raw polygon count by itself. The sanitizer retained only
`220106` faces and punched away `88.83%` of the valid source surface. This is
the same policy defect previously corrected for active Creature routes: a
fixed world-unit epsilon is not a valid definition of geometric degeneracy for
dense meshes. Source textures are embedded RGB JPEG images without alpha, so
texture transparency is not the primary explanation. The retained output was
correctly split into `11` binary-MDL streams with no stream above `21845`
triangles, ruling out the per-stream index boundary as the direct cause.

Correct sanitation should remove only non-finite, exactly zero-area or exactly
collinear faces. For this source that would retain `1971346` faces. That exact
result exceeds the shared `300000`-triangle product budget and must be rejected
or passed through a distinct topology-preserving remesh/decimation stage. A
sanitizer must never act as an accidental polygon-budget reducer.
