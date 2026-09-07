# Reference-supermodel full-skeleton repair — offline result

Date: 2026-08-24

Status: `PIPELINE_REPAIRED / C_WOLF_PRODUCT_BLOCKED / NO_V9_CREATED`

## Immutable inputs

The exact Creature source used by the real `c_wolf` oracle was:

- `sample-3d/borzoi-meshy-manual-p1997k-v1/source-p300k.glb`;
- 29,889,104 bytes;
- 300,000 triangles and 233,832 vertices;
- SHA-256 `f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda`.

The selected retail reference was read without mutation through
`nwn_base.key` and its exact KEY/BIF lineage:

- resref `c_wolf`;
- 340,812 bytes;
- 30 reference nodes and 42 inherited animations;
- SHA-256 `a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726`.

Retail bytes were inspected read-only. They were not copied into an output,
HAK, artifact directory or proof lineage.

## Implemented contract

The shared route is selected-resref driven and has no product admission based
on a `c_wolf` literal or species whitelist. It now:

1. resolves and SHA-binds the exact selected-to-root binary/ASCII MDL chain;
2. builds the carrier inventory from every inherited clip, plus ancestor
   closure and passive structural/attachment/end nodes;
3. excludes render-only nodes, and passive duplicate terminal names shadowed
   by an indispensable controlled/structural carrier, with explicit reasons;
4. emits exact carrier names, parents and bind/rest transforms and verifies
   the output readback against that inventory;
5. separates carrier presence, skin relevance and legal passive-unweighted
   nodes;
6. maps the visible surface with hierarchy/chain distance, topology smoothing,
   coincident-vertex harmonization and dominant seam-stable joint clusters;
7. measures every controlling joint against every inherited clip and samples
   actual visible-surface displacement rather than controller/name presence;
8. blocks Review/export unless full carrier, required joint, active skin,
   inherited clip, visible motion, zero-seam and `motionQuality=PASS` gates all
   pass.

Diagnostic preview remains a separate runtime-readiness state and cannot enter
the Creature product path when any gate is false.

## Exact c_wolf offline oracle

The release oracle produced the following deterministic summary:

```json
{
  "sourceSha256": "f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda",
  "modelSha256": "7ab3a70b2e80b1f535a596c641faa468db3b31849a4246a340b1ec7770e7c62f",
  "carrierNodeCount": 30,
  "activeWeightedBoneCount": 23,
  "requiredClipCount": 42,
  "sampledClipCount": 42,
  "jointClipRequiredCount": 907,
  "jointClipPassCount": 907,
  "failedJointClipCells": [],
  "seamPairSampleCount": 8205960,
  "seamPairViolationCount": 0,
  "pawContactViolationCount": 10,
  "pawSideViolationCount": 4,
  "triangleAreaCollapseCount": 11855,
  "triangleAreaExpansionCount": 661023,
  "status": "BLOCKED"
}
```

Thus full carrier coverage, required joint coverage, active skin influence
coverage, inherited clip coverage, visible joint×clip response and the strict
zero-seam gate are satisfied. Product admission is nevertheless correctly
blocked: the current anatomical surface mapping still violates paw contact and
triangle-area deformation quality. The real env-gated test deliberately fails
its required `PASS` assertion. This is the remaining model-quality blocker and
must not be converted into a diagnostic exception or a weaker threshold.

## Real non-c_wolf proof

The same route was executed against exact retail `c_bat` data from KEY/BIF and
a structurally compatible owned offline surface fixture:

```json
{
  "selectedSupermodelResref": "c_bat",
  "referenceSha256": "0f25791e9e6dc28d87125772a108f12d787e89149f2c016e6f5a110ad9c33303",
  "modelSha256": "d8ace7fac2347344fb2318efec246b9337f0120307605c99b571c0d3a0f425e1",
  "carrierNodeCount": 10,
  "activeWeightedBoneCount": 6,
  "requiredClipCount": 39,
  "jointClipRequiredCount": 198,
  "jointClipPassCount": 198,
  "seamViolationCount": 0,
  "motionQualityStatus": "PASS",
  "exportAdmission": "REFERENCE_SUPERMODEL_EXPORT_ADMITTED"
}
```

This confirms a real non-wolf selected supermodel can complete selection,
analysis, application, preview, Review/product validation and in-memory
MDL/HAK packaging without a family profile or name whitelist. Synthetic tests
also cover structurally different branched, serpentine and winged chains plus
full ASCII-chain parsing.

## Verification

- core generic/motion/product integration: 26 passed;
- core motion unit coverage: 2 passed;
- WASM boundary with environment unset: 7 passed; real tests explicitly
  returned their env-gated skip branches;
- real `c_bat` env-gated release oracle: passed and export admitted;
- exact `c_wolf` env-gated release oracle: executed and failed as designed on
  `motionQualityStatus=BLOCKED`;
- Studio supermodel and Review tests: 19 passed;
- Rust core and WASM checks, including `legacy-c-wolf-demo`: passed;
- TypeScript typecheck: passed;
- production WASM/Vite build: passed;
- canonical-workspace and Meshy asset-layout guards: passed.

## Boundary and next blocker

No V9, new resref, HAK, MOD, installed proof artifact or product model
iteration was created. Aurora Toolset and NWN were not started. The exact
`c_wolf` output remains diagnostic and cannot be handed off as
`ready_for_owner_proof` until the paw-contact and triangle-deformation defects
are fixed offline and the same immutable source returns every admission gate
as PASS. Final Toolset/NWN proof remains human-owned.
