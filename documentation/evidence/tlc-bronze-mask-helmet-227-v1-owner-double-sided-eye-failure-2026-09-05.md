# Owner result: `helm_227` eye surface fails from the viewer side

Date: 2026-09-05

The owner confirmed that the last exact demo is a visual failure and directly
authorized one new helmet after the eye correction was implemented in the
pipeline. The earlier candidate-bound screenshot shows `Cloth 2` selected with
a vivid swatch while the intended eye inserts are absent or nearly absent from
the viewer-facing side.

## Exact failed lineage

- MOD: `m2bronpal227.mod`
  - SHA-256: `324fdf9cd273af7d508fc6ddd6e9f197368758b2d2b3858ae72917bf9e833c3f`
- HAK: `m2bronh227v1.hak`
  - SHA-256: `6c2f3473e7fea174da8599da2a67ad6312d96ec2293371cfdd26b0708bfabf96`
- model: `helm_227.mdl`
  - SHA-256: `3bb3471d90294b7a36ffba3eee5328d3c6ff81e4bd82ec680d8decec6e6118fc`
- model PLT: `helm_227.plt`
  - SHA-256: `f8b1978321b8b597347ac3d116356ca18e57b58ed9009a262aa55d5d68556cd4`
- icon PLT: `ihelm_227.plt`
  - SHA-256: `1716bb4c1bee76d44cbea1b874f74482a1da1ee83bd1627d2330817ae86286bb`

## Verdict

- `modelVisibility = visible`
- `proofCompleteness = verified` for the owner-supplied Toolset observation
- `eyeSurface.viewerSide = failed`

## Diagnosed cause and admitted delta

The source GLB material is explicitly `doubleSided=true`, but the classic Item
MDL route used by 227 did not serialize an equivalent two-sided mesh semantic.
The 430 eye triangles were retained and mapped to Cloth 2, but most source face
normals point away from the Aurora viewer. This is a facing failure, not a PLT
layer-removal or triangle-reduction failure.

One new iteration is admitted. Its minimal geometry delta is:

- use `compile_meshy_static_item_part_v2`;
- preserve source `doubleSided=true` as explicit reverse-wound faces with
  inverted normals and unchanged UVs;
- retain the exact 227 atlas, PLT layer mapping, Cloth 2 luminance correction,
  icon, transform, source hash and module-fixture policy;
- do not add a creature or a placed object.

Expected triangle count: `42,465 + 42,465 = 84,930`, below the shared 300,000
triangle budget.

## Evidence

- screenshot:
  `documentation/evidence/tlc-bronze-mask-helmet-227-v1-owner-double-sided-eye-failure-2026-09-05.png`
- screenshot SHA-256:
  `41c0605b9ba9f451c61d84319d4ca470b759bcc64a06016291586dd072c9f2ee`
