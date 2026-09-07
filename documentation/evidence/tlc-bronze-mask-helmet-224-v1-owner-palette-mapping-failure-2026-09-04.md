# Bronze Mask 224 V1 — owner palette-mapping failure

Date: 2026-09-04

Exact candidate:

- module: `m2bronpal224.mod`
- module SHA-256: `95f0f66b47137f0cc9e83b72ff0696259a4431c7ff9848dd07c16c30e25ee79f`
- HAK: `m2bronh224v1.hak`
- HAK SHA-256: `00d51f7c9580e1134a4042291a7f73508b832c22e882850ff6dd023251ebd3ba`
- model: `helm_224`
- model SHA-256: `031445d669f27ac4353aeed6d04777fff6e4b85ff99a14252463f5d91fbf655f`

Owner report: the helmet is visible in Item Properties, but its authored parts
cannot be recolored through the Item color controls.

Evidence:

- `tlc-bronze-mask-helmet-224-v1-owner-palette-mapping-failure-2026-09-04.png`
- screenshot SHA-256:
  `127ef553a036578953a350015180fd5d55051f54a56226610ad7524b8b7c5d32`

The screenshot itself shows `Metal 2` selected. Candidate 224 intended to use
only `Metal 1` and `Cloth 1`, so that one selection alone is not a direct test
of either intended channel. The owner's functional verdict is nevertheless
consistent with the package defect found offline:

- 224 writes two unrelated model PLTs, `h224cloth.plt` and `h224metal.plt`;
- its MDL render meshes point to those two names;
- working R3 and the retail `helm_035` reference one model-named PLT;
- 224 therefore violates the ModelType-1 model texture contract even though
  both standalone PLT payloads contain valid layer indices.

Independent checks ruled out two tempting but incorrect fixes:

- `renderHint` is not the cause: working R3 uses `renderHint=0`, exactly like
  224;
- a `pltreplacement` TXI is not required for this direct PLT route: retail
  `helm_035` has no TXI resource in the donor HAK.

Recorded axes:

- `modelVisibility=visible`
- `proofCompleteness=verified` for the owner-visible Item Properties result
- `paletteControl=failed`

Minimal corrective delta admitted for 225:

1. pack the two authored source materials into one 2048×2048 UV atlas;
2. compile every render mesh against the exact model resref `helm_225`;
3. write one `helm_225.plt`, with the cloth atlas region on PLT layer 4 and
   the metal atlas region on PLT layer 2;
4. add a package regression gate rejecting any ModelType-1 render mesh whose
   texture0 differs from its model resref.
