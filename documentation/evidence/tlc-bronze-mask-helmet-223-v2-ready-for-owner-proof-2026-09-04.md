# `helm_223` V2 — ready for owner proof

Test-module file: `m2bronpal223.mod`

Module name in Toolset: `Meshy2Aurora Bronze Mask Palette 223 V2`

Exact Area: `Bronze Mask Item Palette 223 V2`

Status: `ready_for_owner_proof` (offline implementation and native installation complete; no agent-run Toolset/NWN session).

## What changed from failed `helm_222`

- The PLT layer assignment is now a deterministic, hash-locked geometry-space mask instead of a per-pixel warm-colour guess.
- The complete front face shell is `Metal 1` and the cowl on the front, sides and rear is `Cloth 1`.
- Fitted minimum Z changed from `-0.1233827621 m` to `+0.0020000001 m`; the uniform donor-width scale remains `0.3430756927`.
- Source topology is unchanged: 36,365 triangles, two Aurora triangle-list segments, no creature resource and no fallback TGA.

Offline semantic-layer views:

- `artifacts/diagnostics/tlc-bronze-mask-tripo-20260904-v2-layer-mask/semantic-layer-front.png`
- `artifacts/diagnostics/tlc-bronze-mask-tripo-20260904-v2-layer-mask/semantic-layer-back.png`
- `artifacts/diagnostics/tlc-bronze-mask-tripo-20260904-v2-layer-mask/semantic-layer-left.png`
- `artifacts/diagnostics/tlc-bronze-mask-tripo-20260904-v2-layer-mask/semantic-layer-right.png`

The preview is diagnostic only: yellow marks `Metal 1`, red marks `Cloth 1`. It is not a Toolset/NWN success claim.

## Exact candidate

- model / Appearance: `helm_223`
- icon: `ihelm_223`
- item blueprint: `m2bronmask223` (`Brązowa Maska 223`)
- custom palette: `Custom → Armor → Helmets → Brązowa Maska 223`
- ordered HAKs: `m2bronh223v2`, then `lc_2da`
- output: `C:\Projects\meshy2aurora\proof-output\tlc-bronze-mask-helmet-palette-only-223-v2-20260904`

Hashes:

- MOD: `f3cfa445646faa8e2fdc513775731cb0d5cb3dad0b8eb1a4ac3074b69ad1cb37`
- HAK: `a3f02239e5f6e54f61058de5a9ef7bc255e63015e5bb0dae3d1b79e50b902ace`
- MDL: `c72bb32aa702e596564c01dfcf8044971a93c4fe0a31a37569bf2f8aac5d9981`
- model PLT: `2032735cdf7e39304f425b775a878208ade4d1824867a37a6b58ac3b27027e64`
- icon PLT: `131461a30a41aa2e45363471aa2b4658210af333cf0db105a779b0da23b86852`
- UTI: `36b5f8fa0e5544c9bf879611b876d6706cddbc90a759556a2ec930ff78b7d5af`
- semantic mask: `ff5716185cbd534654289d8278afab94916b4d80a88fdeaa3985e2cc73fa1cb7`

PLT layer counts:

- `Metal 1`: 623,831 pixels
- `Cloth 1`: 3,570,473 pixels
- all other model layers: 0 pixels

## Native installation receipt

The exact files were installed only after absent-target checks and were hashed again at destination:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2bronpal223.mod`
  - SHA-256 `f3cfa445646faa8e2fdc513775731cb0d5cb3dad0b8eb1a4ac3074b69ad1cb37`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2bronh223v2.hak`
  - SHA-256 `a3f02239e5f6e54f61058de5a9ef7bc255e63015e5bb0dae3d1b79e50b902ace`

## Owner proof checklist

1. Open `m2bronpal223.mod`, then Area `Bronze Mask Item Palette 223 V2`.
2. Select `Brązowa Maska 223` from `Custom → Armor → Helmets` or choose Appearance `helm_223`.
3. Set `Cloth 1` to a saturated colour and `Metal 1` to a contrasting metal; verify that the cowl has no metal flecks.
4. Place the item on level terrain; verify that its bottom is above the terrain instead of intersecting it.
5. Equip the same item and check its vertical position on the head. Aurora uses the same variant transform in both contexts, so this last visual check is mandatory after the ground correction.

## Offline verification

- materializer example compilation: PASS
- exact package generation and semantic readback: PASS
- PLT, item-icon and item-package tests: 13 PASS
- `cargo check -p m2a-wasm --locked`: PASS
- Rust formatting check: PASS
- semantic-mask reproducibility: PASS, byte-identical SHA-256
- canonical-workspace and canonical Meshy-layout guards: PASS
