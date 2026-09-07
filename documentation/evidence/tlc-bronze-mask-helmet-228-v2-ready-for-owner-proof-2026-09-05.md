# `m2bronpal228.mod`

Module: `Meshy2Aurora Bronze Mask Palette 228 V2`

Area: `Bronze Mask Item Palette 228 V2`

Status: `ready_for_owner_proof`

This is the single owner-authorized successor to the visually failed 227
candidate. Aurora Toolset and NWN were not started or controlled by the agent.
No creature resource and no placed object were added; the helmet remains a
custom Items-palette blueprint.

## Exact delta from 227

- source GLB, source hash, fitted transform and atlas UVs: unchanged;
- Metal 1, Cloth 1/2 and Leather 1/2 assignments: unchanged;
- Cloth 2 luminance correction from 227: unchanged;
- inventory icon and its transparent border: unchanged;
- compiler route: Item V1 -> Item V2;
- source `doubleSided=true` is now preserved as explicit reverse-wound faces
  with inverted normals and unchanged UVs;
- source triangles: 42,465;
- generated backface triangles: 42,465;
- compiled triangles: 84,930;
- output MDL segments: 4;
- shared 300,000-triangle budget: passed without warning.

The admitted failure and minimal-delta decision are recorded in
`documentation/evidence/tlc-bronze-mask-helmet-227-v1-owner-double-sided-eye-failure-2026-09-05.md`.

## Exact installed artifacts

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2bronpal228.mod`
  - SHA-256: `2adc12b798b18687ff818de3fdf6394601d803782d267a214e715e64da5d09b3`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2bronh228v2.hak`
  - SHA-256: `e7f9b0fa13e1f0585d60c6cb51e92bb74204c9e87e95968a13b2982024d7a640`

Both native targets were absent immediately before copying. Their post-copy
hashes are byte-identical to the immutable canonical sources.

Ordered HAK list:

1. `m2bronh228v2`
2. `lc_2da`

## Exact resources and hashes

- model: `helm_228.mdl`
  - SHA-256: `bdfc1b110cbc528a8f14d0c23ee61166e06c994fbd52d718a38130964af2a27b`
- model PLT: `helm_228.plt`
  - SHA-256: `f8b1978321b8b597347ac3d116356ca18e57b58ed9009a262aa55d5d68556cd4`
- inventory icon PLT: `ihelm_228.plt`
  - SHA-256: `1716bb4c1bee76d44cbea1b874f74482a1da1ee83bd1627d2330817ae86286bb`
- item blueprint: `m2bronmask228.uti`
  - SHA-256: `a373bf005e6cac2a0b3eb5a6868ff1ad984e88961a5cc53fab250f9bb3c2c91a`
- recipe SHA-256:
  `783ad6dc4d7d440ded872db46cf6c6563d28e64e2815c9eeab7af9e78eb2512f`
- appearance variant: `helm_228`

Canonical output root:
`proof-output/tlc-bronze-mask-helmet-five-material-228-v2-20260905`

## Offline verification

- compiler double-sided policy:
  `SOURCE_DOUBLE_SIDED_EXPLICIT_BACKFACES_V1`;
- MDL, HAK, MOD, UTI and both PLTs passed semantic readback;
- required PLT layers 2, 4, 5, 6 and 7 are non-empty;
- package contains no creature fixture;
- all ten outputs from a second independent build are byte-identical;
- `item_part`: 8 passed, 0 failed;
- `item_package`: 8 passed, 0 failed;
- `cargo fmt --all -- --check`: passed.

## Owner proof boundary

Current axes:

- `modelVisibility = not_tested`
- `proofCompleteness = missing`

Open `m2bronpal228.mod`, load `Bronze Mask Item Palette 228 V2`, create
`Brązowa Maska 228` from the custom Items palette and choose a vivid color for
`Cloth 2`. The recessed eye inserts should be visible from the camera-facing
side and only those inserts should react to Cloth 2.
