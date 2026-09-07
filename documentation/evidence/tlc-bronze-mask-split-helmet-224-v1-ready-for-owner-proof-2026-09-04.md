# `m2bronpal224.mod`

Toolset module name: `Meshy2Aurora Bronze Mask Split Palette 224`

Area name: `Bronze Mask Split Item Palette 224`

Status: `ready_for_owner_proof`

## What changed

This candidate uses the owner's new two-material GLB. The pipeline now gives
authored source-material mappings precedence over automatic pixel inference:

- source material `0`, node `tripo_part_0` (red/material-ID cloth) ->
  `h224cloth.plt` -> PLT layer `4`, `Cloth 1`;
- source material `1`, node `tripo_part_11` (yellow/material-ID metal) ->
  `h224metal.plt` -> PLT layer `2`, `Metal 1`;
- an unmapped source material still uses the compatibility automatic
  Cloth/Metal classifier.

The two model textures are independent PLTs. Every one of the `4,194,304`
cloth pixels uses only layer `4`, and every one of the `1,048,576` metal pixels
uses only layer `2`. This removes the former same-texture layer bleed.

The inventory icon is also generated from an explicit rendered material-ID
mask. It contains `1,937` Metal 1 pixels, `209` Cloth 1 pixels, `1,950`
transparent pixels and an exact transparent border.

## Exact owner test target

- test module: `m2bronpal224.mod`
- module resref: `m2bronpal224`
- module display name: `Meshy2Aurora Bronze Mask Split Palette 224`
- Area resref: `m2assmarea221`
- Area display name: `Bronze Mask Split Item Palette 224`
- ordered HAKs: `m2bronh224v1`, then `lc_2da`
- item palette blueprint: `m2bronmask224`
- item name: `Brązowa Maska 224`
- Appearance/model row: `helm_224`
- icon: `ihelm_224`
- creature resources: none
- placed creatures: none

## Geometry and fit

- owner source SHA-256:
  `8014dfc4a600d8dcd4ad251862e4d20db6900103dd98383e5ea9077d74aa4f0d`;
- canonical compiled source SHA-256:
  `e21998acceb551e756c789b053059e2880bc847fb7fbc17c3cc12aed3ee297bc`;
- input/output triangles: `36,365` / `36,365`;
- output MDL segments: `2`;
- transform policy: restored 222 donor-top-fit, not 223 global
  ground-clearance;
- uniform scale: `0.3429127633571625`;
- translation: `[0.0, 0.0033032000064849854, -0.12338735163211823]`;
- fitted bounds min:
  `[-0.1211988553404808, -0.13096104562282562, -0.12338735163211823]`;
- fitted bounds max:
  `[0.1211988553404808, 0.1375674456357956, 0.21952541172504425]`.

The source differs slightly in bounds from 222, so the calculated scale differs
by about `0.05%`; the placement policy and equipped top alignment are the same.

## Frozen hashes

- MDL `helm_224.mdl`:
  `031445d669f27ac4353aeed6d04777fff6e4b85ff99a14252463f5d91fbf655f`;
- Cloth 1 PLT `h224cloth.plt`:
  `4f3b63a457283b50c7b6f6428a5c51c2c38d35b53df3ecbc3687482e34b91a27`;
- Metal 1 PLT `h224metal.plt`:
  `0381a38554aa48495b8981183ea75eac7b94cb86cd092cc44d3c7c1b35346cc7`;
- icon PLT `ihelm_224.plt`:
  `a6f1d8e71b9d475b8a2dcb323e3ec2e4f1e7843faabc1cbc8515c344352ee336`;
- UTI `m2bronmask224.uti`:
  `435db29d89e4d280d76ed2dea3580a0e2f95e3d97a3ee8b30adafa71fcee7037`;
- HAK `m2bronh224v1.hak`:
  `00d51f7c9580e1134a4042291a7f73508b832c22e882850ff6dd023251ebd3ba`;
- MOD `m2bronpal224.mod`:
  `95f0f66b47137f0cc9e83b72ff0696259a4431c7ff9848dd07c16c30e25ee79f`.

## Offline verification

- exact material mapping unit tests: `2/2 PASS`;
- Item package tests: `7/7 PASS`;
- Item part tests: `5/5 PASS`;
- WebAssembly boundary: `cargo check -p m2a-wasm` PASS;
- binary MDL semantic readback: PASS;
- package semantic readback: PASS;
- triangle budget: PASS (`36,365 <= 300,000`);
- no creature blueprint in the fixture: PASS;
- canonical workspace guard: PASS;
- canonical Meshy asset layout guard: PASS.

## Native installation

The exact frozen proof artifacts were copied only after absent-target checks
and then verified byte-for-byte:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2bronpal224.mod`
  -> MOD hash above;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2bronh224v1.hak`
  -> HAK hash above.

No Toolset or NWN process was started, adopted, switched, captured or closed.
Final visual proof remains owner-owned.

## Owner check

Open `m2bronpal224.mod`, load `Bronze Mask Split Item Palette 224`, and create
or edit a helmet item. Select `helm_224`. Verify that Cloth 1 changes only the
cowl and Metal 1 changes only the face shell, that the inventory icon is
visible, and that equipped fit matches the earlier 222 placement behavior.
