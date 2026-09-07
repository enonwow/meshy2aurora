# TLC ship wood - external reference comparison

Status: `external_reference_comparison_complete`

## Purpose

The earlier comparison against `wooden-sailboat-deck-s2-textured-v1` used a
second owner-provided Meshy asset from this project. It was useful as an
internal regression comparison, but it was not an independent visual
reference. This packet replaces it for the external-reference conclusion.

## External reference

- title: `Half Built Viking Boat`;
- author/vendor: Get Dead Entertainment;
- source page:
  `https://www.renderhub.com/get-dead-entertainment/half-built-viking-boat`;
- published preview reports 17,233 polygons, 11,244 vertices, UV mapping,
  materials, and high-resolution PBR texture maps;
- public render used for analysis only:
  `artifacts/material-separation/ship-wood-comparison/external-half-built-viking-boat/render-01.jpg`;
- render SHA-256:
  `ef97b92693f1a95c92cc654b57fd2e650260e5731653ebeb1f734da25de2a95f`.

The external model and texture payloads were not downloaded, copied, or used
by the Meshy2Aurora pipeline. Only the public marketplace preview is retained
as evidence of the visual comparison.

## Compared candidate

- asset: `tlc-ship-under-construction-s1-p150k-v1`;
- source SHA-256:
  `61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278`;
- 152,574 triangles;
- Material Separation V2: Wood/Rope/Sail/Cloth/Metal;
- geometry cleanup: disabled;
- source UV0: preserved;
- preview:
  `artifacts/material-separation/tlc-ship-under-construction-v2/offline-preview-v9-external-reference-midtones/retextured-side-xy.png`;
- wood transform: warm timber palette, dark-seam retention, and asymmetric
  bright-halo suppression;
- prepared texture-authoring SHA-256:
  `ff2f7533c8e782a12c2d36b3e59cf3df9df5f0ba947785876c63d8a9deb15f83`.

## Comparison board

Path:

`artifacts/material-separation/ship-wood-comparison/external-half-built-viking-vs-tlc-v9.png`

SHA-256:

`5d67410ebc6047d5e3e61eca9d512e1f292ffa3e619cbadba6d8eda8ddc30df4`

## Findings

The external reference remains readable with only 17,233 polygons because its
wood material carries a controlled hierarchy of detail:

1. broad warm-brown tonal variation across planks;
2. clear, dark plank seams without bright sharpening halos;
3. grain aligned with each beam or plank;
4. darker wear near the lower hull and joints;
5. sparse metal fasteners that break up large wood regions.

The superseded TLC v7 wood exposed much more source detail than v6, but it was
too grey and locally over-sharpened. Its bright edge halos made the surface
look carved or chalky instead of built from individual warm wooden members.

TLC v9 replaces that grade with a warm palette and asymmetric local contrast:
negative source detail can darken seams by at most eight levels, while positive
detail can brighten an edge by at most 1.5 levels. This removes the chalky halo
without erasing the original seams. An indicative warm-pixel measurement gives
the following result; renderers and lighting differ, so these values are a
calibration aid rather than a visual-proof claim.

| Render | Mean RGB | Luminance mean | Luminance std. | P10 | P90 |
|---|---:|---:|---:|---:|---:|
| External reference | 75.24 / 48.22 / 30.43 | 52.68 | 12.63 | 34.74 | 67.88 |
| TLC v7, superseded | 78.01 / 64.75 / 56.27 | 66.96 | 25.19 | 35.33 | 100.61 |
| TLC v9, active offline | 71.32 / 45.50 / 25.75 | 49.56 | 15.39 | 36.38 | 61.14 |

## Decision

The v9 grade is the active offline wood input. It restores warmer brown
midtones, suppresses bright halos, retains dark source seams, and keeps the
existing Wood/Rope/Sail/Cloth/Metal separation. It copies no external texture,
adds no synthetic plank layout, and changes no geometry, UV, or cleanup state.

The remaining gap cannot be closed by a global color transform alone: the
source atlas does not contain consistently aligned grain, authored fasteners,
or the broad per-plank variation visible in the external asset. Those features
would require a better source texture or controlled per-face/UV authoring, not
more sharpening.

This is an offline texture diagnosis. It does not admit a new MOD/HAK/model
lineage and does not claim Aurora Toolset or NWN visual proof.
