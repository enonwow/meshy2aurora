# TLC ship detailed materials V1 — ready for owner proof

## Owner handoff

- Test module: `m2a_tlcdm4_mod.mod`
- Module name in Aurora Toolset: `The Last City - Detailed Materials Ship Demo`
- Area: `The Last City Detailed Materials Shipyard`
- Ordered HAK: `m2a_tlcdm4_hak.hak`
- Status: `ready_for_owner_proof`

## Owner result amendment — 2026-08-04

The owner opened exact Area `m2a_tlcdm4_ar`. The model is visible and the
capture is judgeable, but visual acceptance is rejected because the uniformly
brown Toolset result does not match the light, differentiated Material
Separation viewport.

Offline diagnosis proves that the source hash and all `152,574` triangles are
preserved. It also proves that the web demo was not a final-texture parity
preview: it rendered `previewColor` Material ID overlays on the source GLB,
did not load the exact texture payload bytes and did not load the frozen
world-scale UV projection document. Aurora rendered the real packaged TGA and
MDL UV result. Therefore a different source model is rejected as the cause,
while an Aurora/NWN interpretation defect is not established.

Durable result:
`documentation/evidence/tlc-ship-detailed-materials-v1-owner-toolset-visible-rejected-2026-08-04.json`.

The exact MOD and HAK were installed into the native NWN user directories with
absent-target checks and byte-identical SHA-256 readback. The agent did not
start, adopt, switch, control or capture Aurora Toolset or NWN.

## Admission

The exact prior candidate `m2a_tlcsm3_mod.mod` is visible in the owner's Aurora
Toolset screenshots but visually rejected. The durable result is:

`documentation/evidence/tlc-ship-world-scale-uv-v1-owner-toolset-visible-rejected-2026-08-04.json`.

The owner then directly authorized one more detailed material-only iteration:
`postaraj sie podzielic na jeszcze bardziej szczegolowe materialy`. Geometry,
cleanup, scale, placement and collision changes remain unauthorized.

## Detailed Material Separation result

The former `wood` complement contained 141,183 of 152,574 triangles. Face Mode
V2 now retains the four prior non-wood groups and divides Wood into eight
source-bound semantic zones:

| Authored material | Triangles | Share |
|---|---:|---:|
| cloth | 3,633 | 2.381% |
| metal | 116 | 0.076% |
| rope | 1,178 | 0.772% |
| sail | 6,464 | 4.237% |
| wood_bow | 7,575 | 4.965% |
| wood_deck | 10,341 | 6.778% |
| wood_frame | 33,590 | 22.016% |
| wood_hull | 57,188 | 37.482% |
| wood_masts | 1,724 | 1.130% |
| wood_rails | 1,482 | 0.971% |
| wood_stern | 16,460 | 10.788% |
| wood_supports | 12,823 | 8.404% |

Exact recipe SHA-256:
`d606f15bf42d2d4d3a8f0ce1296fdb59d811b63f29064e9b2aa3787afff80fa0`.

The classifier is exact-source and deterministic, but its semantic labels are
still a position/normal heuristic rather than an AI guarantee. The Material ID
overlay is available at:

- `artifacts/material-separation/tlc-ship-under-construction-v2/recipe-detailed-v1/assignment-side-xy.png`;
- `artifacts/material-separation/tlc-ship-under-construction-v2/recipe-detailed-v1/assignment-top-xz.png`;
- `artifacts/material-separation/tlc-ship-under-construction-v2/recipe-detailed-v1/assignment-front-zy.png`.

## Textures and UV

Twelve authored materials resolve to seven exact TGA resources. Three timber
families are intentionally shared without duplicating their payload:

- `wood_medium`: bow, hull, stern;
- `wood_light`: deck, rails;
- `wood_dark`: frame, masts, construction supports.

Every wood group has an independent `MATERIAL_BOX_WORLD` density from 0.28 to
0.75 repeats per authored final metre. The projection SHA-256 is
`8474a14a8c1e4a0700991b81c66ef2a7cc4218acb48765b81c041dc1873a6c76`.

All eight wood bindings pass the neutral 16x16 mip-readability gate as
`READABLE`. Cloth, Metal, Rope and Sail remain `LOW_CONTRAST`, which is reported
without being hidden. Source `doubleSided` remains
`UNSUPPORTED_REPORT_ONLY` for the target writer.

Offline textured preview:

- `artifacts/material-separation/tlc-ship-under-construction-v2/offline-preview-v14-detailed-materials/retextured-side-xy.png`;
- `artifacts/material-separation/tlc-ship-under-construction-v2/offline-preview-v14-detailed-materials/retextured-top-xz.png`;
- `artifacts/material-separation/tlc-ship-under-construction-v2/offline-preview-v14-detailed-materials/retextured-front-zy.png`.

## Geometry and package invariants

- source triangles: `152,574`;
- output triangles: `152,574`;
- source vertices: `176,201`;
- projected vertices: `210,440` (UV seam splits only);
- geometry cleanup: disabled;
- dimensions: `15.173519 × 8.754520 × 6.288216` metres;
- placement: `10.0, 14.5, 0.0`, bearing `0.0`;
- semantic PWK vertices, faces, adjacency, surfaces, position and orientation:
  unchanged; only required fresh model/node resrefs differ;
- every MDL mesh stream stays within the independent binary format boundary.

## Exact artifact hashes

- MDL: `bf35c0fc593d52256da9f5c94699a2a12bdf2d0a0ddd58e35db30523866fb169`;
- PWK: `0319b5f3d96a6542c40894617265c3d07a0f12b22fa4c19046eb9989b52609ee`;
- HAK: `bd01dd793bf931372ac80080642c039db11cf0b201a7cd9f3a31d1abe5975674`;
- MOD: `96bc3fedc52c06b4cafd5122ca13889f3807971f07aee35d1a533cde9690d95f`;
- texture authoring: `4c2e58d0364f3cb8b158e9a4934fa6abf38ae6a21cb433e656ab41434434f1ee`.

Canonical handoff:
`proof-output/tlc-ship-detailed-materials-v1-20260804/ready-for-owner-proof.json`.

## Validation

- canonical workspace gate: PASS;
- canonical Meshy asset layout gate: PASS;
- Material Separation: 16/16 PASS;
- Placeable pipeline: 22 PASS, 1 ignored;
- detailed classifier example: 1/1 PASS;
- detailed texture preparation example: 1/1 PASS;
- materializer compile: PASS;
- deterministic double package build: PASS;
- source/native MOD and HAK hashes: byte-identical PASS.

The candidate is prepared for the owner's visual decision. Toolset and NWN are
still `modelVisibility=not_tested`, `proofCompleteness=missing` for this exact
new lineage until the owner tests it.
