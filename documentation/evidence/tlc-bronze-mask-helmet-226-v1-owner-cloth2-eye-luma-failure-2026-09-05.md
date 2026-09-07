# Owner result: `helm_226` Cloth 2 eye layer is functionally black

Date: 2026-09-05

The owner supplied a fresh Aurora Toolset screenshot for the exact `helm_226`
lineage. The helmet is visible and the Item Properties color dialog has
`Cloth 2` selected with a vivid green swatch, but the eye inserts remain
visually near-black.

## Exact failed lineage

- MOD: `m2bronpal226.mod`
  - SHA-256: `d77f659c650ba78a28c065a2ea7f73005c3d00fe8d077480b5375a07faeadc33`
- HAK: `m2bronh226v1.hak`
  - SHA-256: `2b0cdc44e21118d34a9fb8710b8c5b5e18d5102f852e4b8c2f0fb01b21506168`
- model: `helm_226.mdl`
  - SHA-256: `77341494b852debd4768be2d7998678b9538204c16050b055971698f0e1b61c0`
- model PLT: `helm_226.plt`
  - SHA-256: `f17a9fee7d29e98aebb99d8564556e4c1bf7bbbeff2c35094203985e42798fe1`
- icon PLT: `ihelm_226.plt`
  - SHA-256: `c4b51b381c9c7b928ae33273fb0342a83018990bc4e22a00a71d5b0d983ad2c3`

## Offline diagnosis

The eye geometry was not removed. Binary MDL readback found 430 eye triangles,
and all 430 triangle-centroid UV samples resolve to PLT layer 5 (`Cloth 2`).
The failure is the retained source albedo luminance: the sampled PLT color
indices are too close to the bottom of Aurora's 0..=174 palette ramp.

- minimum: 1
- p10: 9
- median: 18
- mean: 18.05
- p90: 24
- maximum: 61

Thus even a vivid selected color is multiplied by a nearly black shade. The
owner's observation is accepted as a palette-control failure.

## Verdict and admitted correction

- `modelVisibility = visible`
- `proofCompleteness = verified` for the supplied Toolset screenshot
- `paletteControl.Cloth2 = failed`

After the iteration rule was explained, the owner directly ordered generation
of a corrected version. This is recorded as an explicit owner exception for
one `helm_227` iteration. The allowed delta is limited to a Cloth 2 luminance
lift in model and icon PLTs; geometry, UVs, source atlas, transform and all
other palette layers must remain unchanged.

Correction formula:
`min(174, originalColorIndex * 3 + 32)`.

## Evidence

- screenshot:
  `documentation/evidence/tlc-bronze-mask-helmet-226-v1-owner-cloth2-eye-luma-failure-2026-09-05.png`
- screenshot SHA-256:
  `1a1630644f2daa6fc613fa2718ffb7bc5589c9dfdb183be0c9c94c4b8f7ba7ce`
