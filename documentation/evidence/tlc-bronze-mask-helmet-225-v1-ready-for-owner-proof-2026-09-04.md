# `m2bronpal225.mod` — corrected single-atlas helmet

Toolset module name: `Meshy2Aurora Bronze Mask Palette 225 V1`

Exact Area name: `Bronze Mask Item Palette 225 V1`

Status: `ready_for_owner_proof`. Aurora Toolset and NWN were not started or
controlled by the agent.

## What changed

Candidate 224 incorrectly referenced two independent model PLTs. Candidate
225 packs both authored source materials into one UV atlas and uses the native
ModelType-1 resource relationship:

- MDL: `helm_225.mdl`;
- every render mesh texture0: `helm_225`;
- one model palette texture: `helm_225.plt`, resource type 6;
- source material 0 / red material region: `Cloth 1`, PLT layer 4;
- source material 1 / yellow material region: `Metal 1`, PLT layer 2;
- inventory icon: `ihelm_225.plt`, resource type 6.

The model PLT readback contains:

- 1,048,576 `Cloth 1` pixels;
- 1,048,576 `Metal 1` pixels;
- 2,097,152 transparent, unused pixels in the lower atlas half.

The geometry remains exactly 36,365 triangles in two legal MDL streams. The
accepted 222/224 donor-top-fit transform is preserved.

## Installed exact files

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2bronpal225.mod`
  - SHA-256: `61e02c8fbaffe3174ba8183e96d205078283812908dd6c5147e7c604a873803c`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2bronh225v1.hak`
  - SHA-256: `241ec49399e4d9af05aec6df3b962cf6597a04d159e447f598f5ae2089e511b4`

Both destinations were absent before installation and match their canonical
sources byte-for-byte afterward.

Other locked hashes:

- MDL: `74b765b3b67ff9161d93879fd825b005aa4b7be9f325c7654daae721c1153b2a`
- model PLT: `a865f54a8898d689bf563b4bc900032fdfb8033c12e6b9815302cde74fdbef62`
- icon PLT: `31ab58e121afb3bafc6c370e1a859c5aaaa2e5817832d42bc89fd6bab6d72a37`
- UTI: `c609966a2be11e41f0eb31f49be81957a665f721e87284fe287ace91ed801012`
- atlas GLB: `bd664ae455fb960e33c216cc4e41235b53ab05a927a73a1752d802cc90ea77c2`

## Owner test

1. Open `m2bronpal225.mod`.
2. Open Area `Bronze Mask Item Palette 225 V1`.
3. In `Items → Custom → Armor → Helmets`, open `Brązowa Maska 225`.
4. On Appearance choose `helm_225` if needed.
5. Change `Cloth 1`: only the cowl/cloth region should change.
6. Change `Metal 1`: only the face-shell/metal region should change.

`Metal 2`, `Cloth 2`, `Leather 1` and `Leather 2` are deliberately unused and
should not change this candidate.

No creature blueprint or placed creature is included.
