# `m2atgls3.mod`

Module name shown in Toolset: `Meshy2Aurora TLC Guard Longsword item v3`

Exact Area name: `TLC Guard Longsword Item Model Color V3`

## Owner proof handoff

The exact V3 candidate is
`tlc-guard-longsword-item-model-color-v3-20260803`. It supersedes the
owner-rejected V2 fixture and is the first frozen candidate produced with the
complete ModelType 2 model/color pipeline.

- MOD: `m2atgls3.mod`
  - SHA-256: `7ca0760645d88660eaec96c524adf0d3c7dbe823aa68c4280c2e43487efe7748`
  - canonical: `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-model-color-v3-20260803\m2atgls3.mod`
  - installed: `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2atgls3.mod`
- HAK: `m2atglh3.hak`
  - SHA-256: `00f40b6585c0942e95b03feb0ec6f588427ca2420e19f46e1ac23b02c0325c2d`
  - canonical: `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-model-color-v3-20260803\m2atglh3.hak`
  - installed: `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2atglh3.hak`
- UTI: `m2atglu3`, SHA-256
  `5456b41c32e410231d7a5810981e3e02d9fc4f09262e144c3648e987f26af9a0`
- BaseItem: existing retail row `1`, `longsword`, `ItemClass=WSwLs`,
  `ModelType=2`; no `baseitems.2da` mutation was made.
- UTI readback: `Identified=1`, `ModelPart1=23`, `ModelPart2=63`,
  `ModelPart3=23`, semantic status `PASS`.
- Decoded selectors:
  - Bottom: model `2`, selected color `3`;
  - Middle: model `6`, selected color `3`;
  - Top: model `2`, selected color `3`.
- HAK coverage: `12/12` concrete colorways, comprising four MDLs, four
  direct-color TGA textures and four TGA icon layers for each of the three
  ordered parts. The HAK has 37 resources: 12 MDLs, 12 textures, 12 icons and
  the exact UTI.
- Geometry provenance: one canonical Meshy GLB per part; the same geometry is
  reused for colors `1..4` without counting alternatives as simultaneously
  rendered geometry.
- Proportion contract: Bottom/Middle/Top axial lengths
  `0.22 / 0.08 / 0.90`; fit status `PASSED`, solution SHA-256
  `ab335afb5b1b6c73611ffd7243201455e07b29e6fe256d0344bfc9aec829e15a`.
- Geometry budget: 16,356 source triangles, 16,356 selected-render triangles,
  zero removed triangles, shared budget 300,000.
- Semantic readback: MOD `PASS`, HAK `PASS`, UTI `PASS`.
- Native MOD and HAK destinations were absent before installation. Their
  post-installation SHA-256 values are byte-identical to the canonical files.

## What is an Item and what is only the render witness

The Area contains one real placed Item instance of the exact `m2atglu3` UTI at
`(6.0, 9.0, 0.0)`. It also contains one separate humanoid fixture,
`m2atgln3`, at `(8.5, 9.0, 0.0)`. The fixture carries the same byte-identical
UTI in native right-hand `Equip_ItemList` slot `16`; it exists only to exercise
Aurora's equipped ModelType 2 renderer. The Item is not authored as a creature.

## Expected owner check

Open the exact MOD and Area named above. Confirm that the placed object is an
Item blueprint named `Last City Bronze Guard Longsword`, is identified, and
uses part values `23/63/23`. The separate render witness should hold one
assembled longsword with the slimmer `0.22/0.08` hilt and `0.90` blade
proportions. The selected concrete color is `3` for all three parts.

This packet is `ready_for_owner_proof`. The agent did not start or control
Aurora Toolset or NWN. Current proof axes remain
`modelVisibility=not_tested` and `proofCompleteness=missing` until the owner
reports the exact candidate-bound visual result.

Machine-readable packet:
`C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-model-color-v3-20260803\ready-for-owner-proof.json`

## Canonical source-path amendment

The immutable V3 packet records the active worktree source paths used at
capture time. Immediately after the freeze, the three local GLBs were moved to
the required canonical source root
`C:\Projects\meshy2aurora\sample-3d\tlc-guard-longsword-parts-v1`. The exact
Bottom/Middle/Top SHA-256 values remain the values declared above in the
tracked manifest. No MOD, HAK, UTI, MDL, TGA, icon, fit-report or immutable
packet byte was changed. Future product reads use the canonical root.
