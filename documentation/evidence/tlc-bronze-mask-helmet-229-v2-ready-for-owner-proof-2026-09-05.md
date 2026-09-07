# `m2bronpal229.mod`

Module: `Meshy2Aurora Bronze Mask Palette 229 V2`

Area: `Bronze Mask Item Palette 229 V2`

Status: `owner_approved`

This is the owner-approved successor to the visually successful 228 candidate,
built from the updated five-material Tripo GLB. Aurora Toolset and NWN were not
started or controlled by the agent. No creature resource and no placed object
were added; the helmet remains a custom Items-palette blueprint.

## Exact delta from 228

- new owner GLB SHA-256:
  `1cd62de066d35f096f555aa3db40bb7ac16ed790570100b9c6c27aae48b02efd`;
- the source has five separate material primitives, including a dedicated
  430-triangle eye insert primitive;
- one exact degenerate triangle was removed before compilation: 42,466 source
  triangles became 42,465 admitted triangles;
- the model uses the unchanged Item V2 double-sided route, adding 42,465
  reverse-wound faces for 84,930 compiled triangles;
- donor-width fit and donor-top placement policy are unchanged;
- source-material assignment is now explicit and source-ID-bound:
  - material 0, red cowl -> `Cloth 1`;
  - material 4, eye inserts -> `Cloth 2`;
  - material 3, blue forehead wrap -> `Leather 1`;
  - material 1, yellow mask trim -> `Leather 2`;
  - material 2, cyan mask shell -> `Metal 1`;
- Cloth 2 keeps the accepted eye-luminance lift without modifying geometry,
  UVs or any other layer;
- the new native icon is generated from the same five-part semantic mask and
  has a fully transparent border.

## Exact installed artifacts

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2bronpal229.mod`
  - SHA-256: `644fbb2038ca804606331b66f60ff1f267f8f7b3e9a6715afe9dc08b34dadc75`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2bronh229v2.hak`
  - SHA-256: `cb316a183f5cdefe48d12f3a2f40b1e07c5ba5f2138f8d45edcc5bee47ca6350`

Both native targets were absent immediately before copying. Their post-copy
hashes are byte-identical to the immutable canonical sources.

Ordered HAK list:

1. `m2bronh229v2`
2. `lc_2da`

## Exact resources and hashes

- model: `helm_229.mdl`
  - SHA-256: `fffe6c70e23102a150034656657d46cd865db3a50ef9de5f2aec74aa99109dea`
- model PLT: `helm_229.plt`
  - SHA-256: `15f40561d4dbcba222d27cfba3784b459e5ae74334822b72f8754fe1888467c9`
- inventory icon PLT: `ihelm_229.plt`
  - SHA-256: `3aa2bd9a9ca907d68c8be7adb9dffaf453c677b2e67d0ee501a0670e6fb85268`
- item blueprint: `m2bronmask229.uti`
  - SHA-256: `aa78c4bbbfe9ec4f0f9a6a6082a98192b3d74a7b757edf2137876a3bfc07e6ec`
- recipe SHA-256:
  `64818bbf0f39048ed4755c7463f27d04e42a61c5e2dad394f90e3c3560e923f9`
- appearance variant: `helm_229`

Canonical output root:
`proof-output/tlc-bronze-mask-helmet-five-material-229-v2-20260905`

## Offline verification

- compiler double-sided policy:
  `SOURCE_DOUBLE_SIDED_EXPLICIT_BACKFACES_V1`;
- MDL, HAK, MOD, UTI and both PLTs passed semantic readback;
- model PLT has non-empty layers 2, 4, 5, 6 and 7;
- icon PLT has 571 Metal 1, 846 Cloth 1, 33 Cloth 2, 207 Leather 1
  and 97 Leather 2 pixels;
- package contains no creature fixture;
- all ten outputs from a second independent build are byte-identical;
- `item_part`: 8 passed, 0 failed;
- `item_package`: 8 passed, 0 failed;
- `item_icon`: 4 passed, 0 failed;
- `plt`: 3 passed, 0 failed;
- `cargo fmt --all -- --check`: passed;
- canonical Meshy asset layout guard: passed.

## Owner proof boundary

Final owner-reported axes:

- `modelVisibility = visible`
- `proofCompleteness = verified`

Open `m2bronpal229.mod`, load `Bronze Mask Item Palette 229 V2`, create
`Brązowa Maska 229` from the custom Items palette and assign visibly different
colors to all five selectors. Only the cowl should react to Cloth 1, only the
eye inserts to Cloth 2, the forehead wrap to Leather 1, the narrow mask trim to
Leather 2 and the main mask shell to Metal 1. Confirm the same partition in the
inventory icon and the equipped model.

## Owner result

On 2026-09-05 the owner reported: `super approved 229`.

At result-recording time the model-bearing HAK remained byte-identical to the
immutable candidate:

- `m2bronh229v2.hak` SHA-256:
  `cb316a183f5cdefe48d12f3a2f40b1e07c5ba5f2138f8d45edcc5bee47ca6350`.

The native module had been saved by Toolset after installation and therefore
had a new observed SHA-256
`614aaf39f360b8b703292f85712310500ed9a95339b3a3a9fce8d425170fd05a`
and byte length 32,806. The immutable canonical source MOD remains unchanged at
SHA-256
`644fbb2038ca804606331b66f60ff1f267f8f7b3e9a6715afe9dc08b34dadc75`.
No native file was overwritten while recording the result.
