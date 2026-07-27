# M6 — NeverBlender reference comparison v1 (2026-07-18)

## Status

`PARTIAL / REFERENCE-SEMANTICS-PASS / NOT-RUNTIME-PROOF`

This record covers a deliberately isolated comparison export. It is not a
replacement for Meshy2Aurora's clean-room binary writer, and it does not prove
that the reference output is visible in NWN.

## Symptom and scope

Meshy H1 is visible in Aurora Toolset but the prior `SKIN` binary output has
not yet been proven visible in the NWN client. The purpose of this test was to
separate a loss of source geometry/weights from a binary `skin`-layout issue.

Input GLB:

- path: `test-assets/meshy/incoming/h1-humanoid-1500.glb`;
- SHA-256: `3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f`.

## Reference environment and provenance

NeverBlender was used only as a locally isolated, reference-only exporter and
importer. Its GPL source was not copied into product code and it is not a
Meshy2Aurora build/runtime dependency.

| Item | Value |
| --- | --- |
| Blender | official portable Blender 4.0.2 |
| Blender archive SHA-256 | `BDDB7AB6880BBB80297B93731506A48BBEBDCE214EE5EDD17DBEF8380DAECDED` |
| NeverBlender | 4.1.0 (`neverblender_40-010.zip`) |
| NeverBlender archive SHA-256 | `4C94DADA3C3AB6FDD3CF34C8563B6F2355AF156B6E850D416D7994D64FB78C5A` |
| Compatibility fact | NeverBlender release declares Blender 4.0.x; it explicitly excludes Blender 4.1+ |
| Isolation | `BLENDER_USER_SCRIPTS` and `BLENDER_USER_CONFIG` pointed beneath `proof-output/third-party-tools/neverblender-reference`; the system Blender 5.1 and user configuration were untouched |

The add-on was loaded successfully in batch mode (`Neverblender: Ready`).

## Method

1. Import the exact H1 GLB in Blender 4.0.2.
2. Create one synthetic Aurora root dummy plus one dummy per GLB joint, named
   exactly like the GLB joint. This is an adapter required by NeverBlender's
   documented data model; it is not a source or product skeleton.
3. Mark the imported `char1` mesh as `skin` and export one ASCII MDL with
   NeverBlender's public export operator.
4. Import that emitted ASCII MDL back with NeverBlender's public import
   operator and compare the recovered mesh semantics.

The GLB carries a uniform `0.01` armature-container scale. An initial export
lost that object-scale and emitted coordinates approximately 100x too large.
The reference adapter therefore bakes the positive uniform scale into mesh
coordinates and dummy translations before the export. This matches the
explicit Meshy H1 container-scale treatment in
`crates/m2a-core/src/profile_a.rs`; it is not a change to the product
pipeline.

Reference scripts and emitted file (all non-product proof artifacts):

- `proof-output/third-party-tools/neverblender-reference/export_meshy_h1_reference.py`;
- `proof-output/third-party-tools/neverblender-reference/roundtrip_meshy_h1_reference.py`;
- `proof-output/third-party-tools/neverblender-reference/neverblender-meshy-h1-reference.mdl`;
- reference MDL SHA-256: `931AE25EECE35FF3723685684BDC1E2685E1A8BEA2DFD10500E8E687B26336E1`.

## Results

| Semantic property | NeverBlender export → import | Meshy2Aurora v19 report |
| --- | ---: | ---: |
| Mesh type | 1 `skin` | `SKIN` |
| Vertices | 1334 | 1334 |
| Triangles | 1556 | 1556 |
| Source joint names | 24 dummies | 24 source joints |
| Non-empty skin influence groups | 22 | 22 active joints |
| Weight block | 1334 entries, maximum four influences | emitted and own-readback passed |

The round-trip importer recovered exactly one `skin` mesh, 1334 vertices,
1556 triangles and the 22 non-empty vertex groups. The two source joints that
have no non-zero vertex weights (`head_end`, `headfront`) are naturally absent
from the re-imported skin groups. This is consistent with the 22 active joints
reported by Meshy2Aurora; no evidence points to a missing geometry, a missing
weight domain, or a source-joint-count mismatch.

## What the test does and does not establish

Facts established by this comparison:

- a conventional NWN authoring add-on accepts the H1 mesh and produces a
  syntactically re-importable `skin` with matching geometry and active
  influence count;
- the Meshy H1 `0.01` container scale must be handled explicitly by any
  reference conversion; and
- our current source ingestion/profile does not appear to discard the two
  main prerequisites of `skin` export: topology or non-zero weight groups.

This does **not** establish that either ASCII export or our binary output is
visible in NWN. NeverBlender writes ASCII; binary `skin` fields such as inverse
bind records, forward/reverse bone maps and bone constants are materialized by
the subsequent compiler. Aurora decompilation identifies those binary fields
as consumed by the engine loader, so they remain the primary suspect class for
the invisible `SKIN` variant.

## Compiler attempt and remaining risk

The locally available `nwnmdlcomp.exe` was invoked only as a reference compiler
with input/output paths under this proof directory. It stopped before parsing
the reference MDL with `Unable to locate or open Neverwinter Night`, because it
requires an NWN installation location in the Windows registry. No registry,
game configuration or installation change was made. Therefore no reference
binary was produced and this record intentionally makes no runtime claim.

Next evidence-driven action: compare the compiler-generated binary `skin`
tables with our writer only if a non-invasive, owner-approved compiler context
is available; otherwise continue the existing direct NWN `RIGID`/`SKIN` A/B
runtime proof. In either case, Aurora decompilation remains the format
contract.
