# `m2bronpal226.mod`

Module: `Meshy2Aurora Bronze Mask Palette 226 V1`

Area: `Bronze Mask Item Palette 226 V1`

Status: `ready_for_owner_proof`

This is a new owner-selected five-material Tripo source lineage. No creature
resource or creature placement was added. The module exposes the generated
helmet item in the custom item palette.

## Material mapping

- cloth cowl -> `Cloth 1` (PLT layer 4)
- eye inserts -> `Cloth 2` (PLT layer 5)
- main mask shell -> `Metal 1` (PLT layer 2)
- viewer-left mask strip -> `Leather 1` (PLT layer 6)
- viewer-right mask strip -> `Leather 2` (PLT layer 7)

All five source material regions were UV-remapped into one 2048x2048 atlas and
compiled against one model-named `helm_226.plt`. The inventory icon is
`ihelm_226.plt`; its silhouette comes from material-ID alpha, so dark cloth is
preserved instead of being removed as background.

## Exact installed artifacts

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2bronpal226.mod`
  - SHA-256: `d77f659c650ba78a28c065a2ea7f73005c3d00fe8d077480b5375a07faeadc33`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2bronh226v1.hak`
  - SHA-256: `2b0cdc44e21118d34a9fb8710b8c5b5e18d5102f852e4b8c2f0fb01b21506168`

Both destinations were absent before installation. The installed files were
hashed after copying and are byte-for-byte identical to their immutable
canonical sources.

Ordered HAK list:

1. `m2bronh226v1`
2. `lc_2da`

## Exact resources and hashes

- model: `helm_226.mdl`
  - SHA-256: `77341494b852debd4768be2d7998678b9538204c16050b055971698f0e1b61c0`
- model PLT: `helm_226.plt`
  - SHA-256: `f17a9fee7d29e98aebb99d8564556e4c1bf7bbbeff2c35094203985e42798fe1`
- inventory icon PLT: `ihelm_226.plt`
  - SHA-256: `c4b51b381c9c7b928ae33273fb0342a83018990bc4e22a00a71d5b0d983ad2c3`
- item blueprint: `m2bronmask226.uti`
  - SHA-256: `48d9fc4dc6d36e99d9ddbad0952f423d741064038cce5b9fac71dc931d5d1aff`
- appearance variant: `helm_226`

Canonical output root:
`proof-output/tlc-bronze-mask-helmet-five-material-226-v1-20260905`

## Offline verification

- 42,465 compiled triangles; shared 300,000-triangle budget passed.
- Exact one-mesh/one-material atlas input passed static Item Profile A.
- Binary stream partitioning produced two output segments without removing
  render triangles after the one documented exact-degenerate source triangle.
- MDL, HAK, MOD, UTI and both PLTs passed semantic readback.
- Required PLT layers 2, 4, 5, 6 and 7 are non-empty in both model and icon.
- Inventory icon has a transparent border and 5-pixel padding.
- A second clean build matched every canonical output byte-for-byte.
- Relevant Rust tests: 19 passed, 0 failed.
- `cargo fmt --all --check`: passed.
- `cargo check --target wasm32-unknown-unknown -p m2a-core`: passed.
- Canonical-workspace and Meshy asset-layout guards: passed.

Offline render:
`artifacts/diagnostics/tlc-bronze-mask-tripo-20260905-v2-offline-qa/atlas-render-v2/front-left-3q.png`

Icon layer preview:
`artifacts/diagnostics/tlc-bronze-mask-tripo-20260905-v2-offline-qa/ihelm_226-layer-preview.png`

## Owner proof boundary

Toolset and NWN were not started or controlled. The owner performs the visual
proof. Current axes are:

- `modelVisibility = not_tested`
- `proofCompleteness = missing`

Open `m2bronpal226.mod`, load `Bronze Mask Item Palette 226 V1`, create the
helmet from the custom Items palette, and verify the Appearance controls for
`Cloth 1`, `Cloth 2`, `Metal 1`, `Leather 1` and `Leather 2`, plus the inventory
icon and on-head fit.
