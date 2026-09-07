# TLC ship world-scale UV V1 — ready for owner proof

## Owner handoff

- Test module: `m2a_tlcsm3_mod.mod`
- Module name in Aurora Toolset: `The Last City - World Scale UV Ship Demo`
- Area: `The Last City World Scale UV Shipyard`
- Ordered HAK: `m2a_tlcsm3_hak.hak`
- Status: `ready_for_owner_proof`

The exact MOD and HAK were installed into the native NWN user directories with
absent-target checks and byte-identical SHA-256 readback. The agent did not
start, adopt or control Aurora Toolset or NWN.

## Admission and minimal delta

The exact prior candidate `m2a_tlcsm2_mod.mod` was reported by the owner as
visible in Aurora Toolset but visually rejected. The owner's 2026-08-04
clarification admits one material-only iteration and explicitly forbids a
geometry change.

The new candidate keeps the same source, Material Separation recipe, physical
scale, placement, triangle count and semantic PWK surface. Wood changes from
per-component atlas-band projection to full-texture `MATERIAL_BOX_WORLD` at
`0.5` repeats per authored metre. Neutral texture mip-readability and source
`doubleSided` diagnostics are included in the package report.

## Offline invariants

- source and output triangles: `152,574`;
- source vertices: `176,201`;
- projected vertices: `186,436`;
- geometry cleanup: disabled;
- dimensions: `15.173519 × 8.754520 × 6.288216` metres;
- placement: `10.0, 14.5, 0.0`, bearing `0.0`;
- previous and current PWK vertices, faces, adjacency, surfaces, position and
  orientation: semantically identical; only required model/node resrefs differ;
- material UV projection SHA-256:
  `adcb1ecb5cc52edca36523149f8df2d1f858631e17af32a49da02b4fd255db2b`;
- texture authoring SHA-256:
  `560ccbd0f3ca7f34a2df1a2b1f50b3f3bc6c9cf2934680c6da8cf850f90a7fb1`.

The quality gate classifies all five retained bitmaps as `LOW_CONTRAST` at the
16x16-mip approximation. This does not block the demo: it makes the remaining
bitmap limitation explicit. V1 tests the world-scale UV correction without
silently claiming that the diffuse textures themselves are accepted.

## Exact artifact hashes

- MDL: `a4679a97c7c5b6eb010b003c433428bf5bcff0be9e68add14d7c089e982eb0a7`;
- PWK: `fd4dbaea322a8feb30ba023c89b0fc0a81770303b337a67eb0e957e90210a9bd`;
- HAK: `9a04b028e605cd2cfb79f38bd0e08a565cc1fc3da362b5c18186df1292e6f340`;
- MOD: `ce3872e6d6bf24496ebdc44ec225a11356f200d3f89a8d1caa740cf88405ba7e`.

Canonical handoff:
`proof-output/tlc-ship-world-scale-uv-v1-20260804/ready-for-owner-proof.json`.

## Proof boundary

Offline package determinism, MDL stream limits, triangle preservation, semantic
PWK equality, TGA hashes and native installation passed. Toolset and NWN remain
`modelVisibility=not_tested`, `proofCompleteness=missing` for this new exact
lineage until the owner performs the visual proof.
