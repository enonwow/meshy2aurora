# `m2bronpal227.mod`

Module: `Meshy2Aurora Bronze Mask Palette 227 V1`

Area: `Bronze Mask Item Palette 227 V1`

Status: `ready_for_owner_proof`

This is the owner-authorized correction of the visible `helm_226` Cloth 2 eye
luminance failure. No creature resource or placement was added.

## Exact isolated delta from 226

- geometry, UVs, atlas input and transform: unchanged;
- PLT layer assignments: unchanged;
- only `Cloth 2` (layer 5) color indices were lifted;
- model PLT pixels changed: 262,144, all on layer 5;
- inventory icon PLT pixels changed: 46, all on layer 5;
- pixels changed on Metal 1, Cloth 1 or Leather 1/2: zero;
- layer-index bytes changed: zero.

The independently serialized vertex, UV0 and triangle-index streams of 226 and
227 have the same SHA-256 digest:
`1a8de6e3c5ab7e6ea4a5c57b3394fa8a5c50f8e2190e3e474496692845d7344b`.

Correction formula:
`min(174, originalColorIndex * 3 + 32)`.

The 430 eye-triangle centroid samples still resolve exclusively to layer 5.
Their luminance distribution changed from median 18 to median 86:

| Statistic | 226 | 227 |
|---|---:|---:|
| minimum | 1 | 35 |
| p10 | 9 | 59 |
| median | 18 | 86 |
| mean | 18.05 | 85.71 |
| p90 | 24 | 104 |
| maximum | 61 | 174 |

## Material mapping

- cloth cowl -> `Cloth 1` (PLT layer 4)
- eye inserts -> `Cloth 2` (PLT layer 5)
- main mask shell -> `Metal 1` (PLT layer 2)
- viewer-left mask strip -> `Leather 1` (PLT layer 6)
- viewer-right mask strip -> `Leather 2` (PLT layer 7)

## Exact installed artifacts

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2bronpal227.mod`
  - SHA-256: `324fdf9cd273af7d508fc6ddd6e9f197368758b2d2b3858ae72917bf9e833c3f`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2bronh227v1.hak`
  - SHA-256: `6c2f3473e7fea174da8599da2a67ad6312d96ec2293371cfdd26b0708bfabf96`

Both targets were absent before installation and were verified byte-identical
to the immutable canonical sources after copying.

Ordered HAK list:

1. `m2bronh227v1`
2. `lc_2da`

## Exact resources and hashes

- model: `helm_227.mdl`
  - SHA-256: `3bb3471d90294b7a36ffba3eee5328d3c6ff81e4bd82ec680d8decec6e6118fc`
- model PLT: `helm_227.plt`
  - SHA-256: `f8b1978321b8b597347ac3d116356ca18e57b58ed9009a262aa55d5d68556cd4`
- inventory icon PLT: `ihelm_227.plt`
  - SHA-256: `1716bb4c1bee76d44cbea1b874f74482a1da1ee83bd1627d2330817ae86286bb`
- item blueprint: `m2bronmask227.uti`
  - SHA-256: `a912f3ae87dcc9a9cf7127a1b9d78b47c21d491a52bd123a90a1b685826f6766`
- recipe SHA-256:
  `228e01747e544ed87ca2c53d258eb5db84b1065aed3038f51b93f43a80ee0bf1`
- appearance variant: `helm_227`

Canonical output root:
`proof-output/tlc-bronze-mask-helmet-five-material-227-v1-20260905`

## Offline verification

- 42,465 triangles; shared 300,000-triangle budget passed.
- MDL, HAK, MOD, UTI and both PLTs passed semantic readback.
- Required PLT layers 2, 4, 5, 6 and 7 remain non-empty.
- The inventory icon retains its transparent border and material-ID-alpha
  silhouette policy.
- A second clean build matched all ten canonical outputs byte-for-byte.
- Relevant Rust tests: 20 passed, 0 failed.
- `cargo fmt --all --check`: passed.
- `cargo check --target wasm32-unknown-unknown -p m2a-core`: passed.
- Canonical-workspace and Meshy asset-layout guards: passed.

Eye-visible icon-layer preview:
`artifacts/diagnostics/tlc-bronze-mask-tripo-20260905-v2-offline-qa/ihelm_227-layer-preview-8x.png`

## Owner proof boundary

Toolset and NWN were not started or controlled. Current axes are:

- `modelVisibility = not_tested`
- `proofCompleteness = missing`

Open `m2bronpal227.mod`, load `Bronze Mask Item Palette 227 V1`, create the
helmet from the custom Items palette, select `Cloth 2`, and choose a vivid
swatch. Only the recessed eye inserts should react, and they should now remain
bright enough to show the selected hue.
