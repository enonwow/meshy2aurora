# TLC ship — Face Mode V2 + component UV materials

## Owner handoff

- Test module: `m2a_tlcsm2_mod.mod`
- Module name in Aurora Toolset: `The Last City - Component UV Ship Demo`
- Area: `The Last City Component UV Shipyard`
- Ordered HAK: `m2a_tlcsm2_hak.hak`
- Status: `ready_for_owner_proof`
- Agent did not start or control Aurora Toolset or NWN.

The exact MOD and HAK were installed into the native NWN user directories with
absent-target checks and byte-identical readback. The project-side immutable
handoff is
`proof-output/tlc-ship-material-separation-v2-component-uv-20260802/ready-for-owner-proof.json`.

## Implemented result

The Placeable is now emitted through five independent material slots:

- Wood: 141,183 triangles;
- Rope: 1,178 triangles;
- Sail: 6,464 triangles;
- Cloth: 3,633 triangles;
- Metal: 116 triangles.

Face Mode V2 now splits triangles by their exact face assignment even when two
materials occur inside one connected component. This fixes the former behavior
where a whole component inherited the material of its first triangle.

Wood alone receives deterministic component-long-axis UV projection and a
dedicated plank texture. Rope, Sail, Cloth and Metal retain their source UV0.
The source geometry has 152,574 triangles and the output has 152,574 triangles.
Geometry cleanup is disabled. UV projection duplicates render vertices where
needed (176,201 source vertices, 179,319 vertices after exact material buckets,
436,847 final render vertices) but does not delete or add a triangle and does
not modify collision/PWK geometry.

## Texture source and offline preview

Wood uses the CC0 `Wood Planks` diffuse from Poly Haven, exact downloaded file
SHA-256
`3b0669f683e4bf10f5a55a381cfa9669a7b8dfd921901829daa3b35acc2bbdec`,
regraded to the NWN ship-reference midtones. The other four textures are
material-specific grades of the exact source atlas.

- Material UV projection SHA-256:
  `4cc440b988709537ff2404556212c184aec54311a0d15271f00af6af6e1a84d2`;
- texture authoring SHA-256:
  `560ccbd0f3ca7f34a2df1a2b1f50b3f3bc6c9cf2934680c6da8cf850f90a7fb1`;
- accepted offline side preview:
  `artifacts/material-separation/tlc-ship-under-construction-v2/offline-preview-v13-component-uv-large-grain/retextured-side-xy.png`;
- side preview SHA-256:
  `576650670ab2b1f3aa67067599d6d8637cf19ca7b0a44323e3e93000022aa613`;
- external-reference comparison board:
  `artifacts/material-separation/ship-wood-comparison/external-half-built-viking-vs-tlc-v13-component-uv.png`;
- board SHA-256:
  `f489eaa1355761ac2d2c16600d3bb4e03c5292101eb62661d08cad9c68845427`.

The earlier bake into the original 21,936-island atlas was rejected: it made
the timber flat and smeared because too little 1024px texture area remained per
island. It is preserved only as diagnostic V10 evidence and is not active in
this candidate.

## Exact candidate hashes

- source GLB:
  `61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278`;
- Material Separation recipe:
  `6eeb7877b24feb3f4a315a9ae68809dc5620ca4272bfb7702d3d159f67df4b3a`;
- MDL:
  `c5f2621aedc912b3479ae1eade5fc2ced84cef6debce5a26302255fa59afebb3`;
- PWK:
  `fd58bcb9b49d42008cd0c57e46de761c169c944346e08fde75b9b9c77843f4f5`;
- HAK:
  `b7577438c53039e78518f6b7d574cbb06a38216415641bf370a3945ba0a17493`;
- MOD:
  `30f0bfc99ea9a32e9fc83c241e7c843ca1bc85ccf01bb5134e44c170bd519bc5`.

## Proof boundary

Offline MDL/PWK/TGA/2DA/UTP/GIT/GIC/HAK/MOD readback passed. The candidate is
not yet visually approved: `modelVisibility=not_tested` and
`proofCompleteness=missing` for this exact lineage. The owner must open the
module and judge the ship in Toolset and NWN. Texture separation is now real;
remaining mismatch against the external reference may come from the generated
ship geometry itself, especially its fragmented hull and construction detail.
