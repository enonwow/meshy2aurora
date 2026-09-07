# TLC sandstorm mask B29 P15K V1 — offline quality failure

Date: 2026-09-04

## Exact candidate

- canonical source: `sample-3d/tlc-sandstorm-mask-b29-p15k-v1/source.glb`
- SHA-256: `9d6dac60b2cd495019f36326454c247b4e736286fc931b7bc4317ac78e996a46`
- size: `3,840,856` bytes
- measured triangles: `15,118`
- local Bridge run: `71f6237e-61b6-40b0-8cdd-acc01c595d1f`
- Meshy task: `01a06c8b-f6e6-763f-8923-63c4e2b465ec`
- source mode: `MULTI_IMAGE`, exact order `front, left, back, right`
- profile/model: `S1-static-prop/v1`, `standard`, `meshy-6`
- charged credits: `30`

The four input identities and the paid-run record are locked in
`sample-3d/tlc-sandstorm-mask-b29-p15k-v1/manifest.yaml` and
`meshy-run-provenance.json`. B30, B31 and concept regeneration were not used.

## Offline readback

The exact GLB imports as one mesh object and one material. After a `1e-5`
spatial weld used only for diagnosis, the surface is one connected component,
but it retains `86` boundary edges and `193` non-manifold edges. The raw GLB
contains `23,278` vertices and `15,118` triangle polygons.

Machine-readable report:
`artifacts/diagnostics/tlc-sandstorm-mask-b29-p15k-v1-offline-qa/renders/offline-qa.json`.

Rendered views:

- front: `artifacts/diagnostics/tlc-sandstorm-mask-b29-p15k-v1-offline-qa/renders/front.png`, SHA-256 `779da5c51aa3335e8f730c5ddf496fd23e9b398e9194e1d2a71656f03da9d1ad`
- left: `artifacts/diagnostics/tlc-sandstorm-mask-b29-p15k-v1-offline-qa/renders/left.png`, SHA-256 `eeaaab8cf2551ea3389a67970080c56d5824e0af341428caccb7e9d25e338983`
- back: `artifacts/diagnostics/tlc-sandstorm-mask-b29-p15k-v1-offline-qa/renders/back.png`, SHA-256 `4de4f8598524162049ec633e0bf55ad86055ed5c56f5d85bfc3548a8fedba6b2`
- right: `artifacts/diagnostics/tlc-sandstorm-mask-b29-p15k-v1-offline-qa/renders/right.png`, SHA-256 `90f4f7dfa9c2c2fb3937d345bc3f911738654f945b779ea44bb5548808d27618`
- three-quarter control: `artifacts/diagnostics/tlc-sandstorm-mask-b29-p15k-v1-offline-qa/renders/front-left-3q.png`, SHA-256 `90d8168ac2b1875b0046f9e4172f27d012fd9b5c28a3cb3b27989cf9817c555b`
- machine-readable QA: SHA-256 `a77ccd45ddec120e672212924830dd8894533814a6de9243254e780202e1b019`

## Candidate-bound verdict

`silhouette=failed` in offline QA.

- The front preserves the intended mask identity.
- Both profiles produce a large backward face-like protrusion instead of a
  clean rear cowl/head envelope.
- Cloth folds become long, sharp lamellar spikes and visible gaps.
- The generated shell covers the full head and neck rather than limiting the
  rigid armor shell to the front third ending before the ear.
- Eye recesses and many filter perforations are modeled as actual openings,
  contrary to the opaque-recess/surface-detail requirement.
- Boundary and non-manifold readback do not satisfy the closed-ish helmet
  requirement.

This is a visual source-quality failure, not proof that an Aurora/NWN model is
invisible. The independent proof axes remain:

- `modelVisibility=not_tested`
- `proofCompleteness=missing`

The current source is therefore not advanced into the Item/helmet materializer
or a demo MOD/HAK. Doing so would freeze a visibly wrong object and would not
repair the source silhouette.

## Minimal intended next delta and gate

The minimal useful delta is another Multi-Image generation from the same four
immutable B29 masters, keeping the same order and no concept prompt, while
changing only the provider-side stochastic result. No independent axis scaling,
manual design change, B30/B31 input or concept regeneration is proposed.

No second model was created. Under the repository model-iteration hard gate,
this offline failure does not by itself admit a new generated payload. A new
iteration requires a fresh exact-candidate authorization that explicitly
permits another Meshy run from this failed SHA-256, or the gate's required
candidate-bound Toolset/NWN result supplied by the human proof owner.
