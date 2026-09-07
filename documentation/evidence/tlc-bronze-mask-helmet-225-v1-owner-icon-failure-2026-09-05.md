# Owner result: `helm_225` inventory icon rejected

Date: 2026-09-05

The owner supplied a fresh NWN runtime screenshot for the exact `helm_225`
lineage. The helmet is visibly equipped on the player, but its inventory-slot
icon is cropped to the light mask and omits the dark cloth cowl. This is a
candidate-bound icon failure, not a model-visibility failure.

## Exact failed lineage

- MOD: `m2bronpal225.mod`
  - SHA-256: `61e02c8fbaffe3174ba8183e96d205078283812908dd6c5147e7c604a873803c`
- HAK: `m2bronh225v1.hak`
  - SHA-256: `241ec49399e4d9af05aec6df3b962cf6597a04d159e447f598f5ae2089e511b4`
- model: `helm_225.mdl`
  - SHA-256: `74b765b3b67ff9161d93879fd825b005aa4b7be9f325c7654daae721c1153b2a`
- inventory icon: `ihelm_225.plt`
  - SHA-256: `31ab58e121afb3bafc6c370e1a859c5aaaa2e5817832d42bc89fd6bab6d72a37`

## Verdict

- `modelVisibility = visible`
- `proofCompleteness = verified` for the supplied NWN screenshot
- `inventoryIcon = failed`

The diagnosed cause is the old beauty-image dark-background flood heuristic:
the black/dark cloth touched the image edge and was incorrectly classified as
background. The corrective product path composes PLT icons from a separate
material-ID render and uses only its alpha channel for silhouette membership.
Dark foreground color no longer controls transparency.

## Evidence

- screenshot:
  `documentation/evidence/tlc-bronze-mask-helmet-225-v1-owner-icon-failure-2026-09-05.png`
- screenshot SHA-256:
  `90b7dd580d7297d8e50240c63122497d8731fa7edfe4cb0d7edf2ad6cde77881`
