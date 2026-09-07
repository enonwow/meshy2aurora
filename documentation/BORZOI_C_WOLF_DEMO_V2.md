# Borzoi `c_wolf` demo V2

Date: 2026-08-20

Status: `ready_for_owner_proof`. This record makes no Toolset or NWN visual-success claim.

## Exact owner handoff

- Test module file: `m2aborzmod2.mod`
- Toolset module name: `Meshy2Aurora Borzoi c_wolf Demo V2`
- Area: `Meshy2Aurora Borzoi Test Area V2`
- Ordered HAK: `m2aborzhak2`
- Creature blueprint: `m2aborzutc2`
- Appearance row: `848`
- Model resref: `m2aborzcre2`
- Texture resref: `m2aborztex2`

The exact module and HAK were installed into the native NWN user `modules` and `hak` directories after absent-target checks. Destination SHA-256 values equal the canonical proof-output sources byte for byte.

## V1 defect remediation

V2 uses the same owned Meshy source mesh, texture and all 60,780 triangles. No new Meshy task was created. Only the clean-room rig construction changed:

- front paw nodes are fitted to source contact centroids at X `+0.10065819` and `-0.10013413` instead of the narrower V1 procedural positions;
- back paw nodes are fitted to X `+0.12633894` and `-0.12671168`;
- all four paw nodes use source contact heights in the `0.035083484` to `0.042655107` range;
- `Wolf_rootdummy` bind world position is `[0.0, -0.38856345, 0.7881385]`, calibrated to the neutral inherited `c_wolf` root-motion frame so animation does not apply the V1 start-height jump;
- leg weights use distance to the fitted bone chain and a torso/fur guard; low central chest and long-fur vertices no longer enter broad left/right leg boxes.

The exact V1 owner result and diagnosis remain in `documentation/evidence/borzoi-c-wolf-demo-v1-owner-nwn-result-2026-08-20.json`. The V2 source-fit evidence is `proof-output/borzoi-c-wolf-demo-v2-20260820/source-fit-report.json`.

## Immutable artifact identity

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2aborzmod2.mod` | 15,205 | `19764078dd962da1541810d1837b8805f4f7cd14242296c7b7e0c8a61b1dd55d` |
| `m2aborzhak2.hak` | 20,661,840 | `2611ffc9ca68cca91aabd96dd76ef0e484bb40f4f13b43d10fd4249e3c21cc37` |
| `m2aborzcre2.mdl` | 7,690,476 | `36c30f6c2ced53031a1938a66b8055d30ab0a969250b2c7c795db8acbf09282d` |
| `m2aborztex2.tga` | 12,582,956 | `8b8285e273d3079a6651a4a86c5e83493d6a8fa2fb6cdd8ce35fb4b5c6fec893` |

Canonical packet: `proof-output/borzoi-c-wolf-demo-v2-20260820`.

## Animation and geometry contract

The generated model names base-game `c_wolf` as its supermodel, contains the exact compatible 30-node hierarchy, zero local animation clips and inherits the 42 inspected `c_wolf` clips. It preserves all 60,780 source triangles in three binary-MDL-safe weighted streams. Creature Basis V3 still maps glTF positive-Z to Aurora positive-Y with determinant `+1`, so this remediation does not change the proven facing convention.

## Verification boundary

The exact source fingerprint, retail-reference packet, source-fit report, generated MDL semantic readback, HAK resource readback, MOD scene readback, targeted V2 regression tests and the complete `m2a-core` test suite passed. The remaining owner-owned visual axes are:

- `modelVisibility = not_tested`
- `proofCompleteness = missing`
- `qualityVerdict = not_tested`

The owner performs the final Toolset/NWN check of grounding, front-paw separation and inherited movement quality.

## Owner NWN result

The owner later reported the exact V2 candidate as visible but rejected its quality:

- `modelVisibility = visible`
- `proofCompleteness = verified`
- `qualityVerdict = failed`
- front legs remain visibly rigid;
- overall dog-model quality is too low.

Offline weight inspection confirmed that `Wolf_Lfrontupperleg` and `Wolf_Rfrontupperleg` each have zero vertices for which they are the primary influence. The V2 nearest-two-chain weighting therefore failed to give either shoulder/upper-leg joint its own deformation region. This rig defect is pipeline-remediable.

The source still contains 60,780 triangles and a 2048×2048, 24-bit texture. V2 did not reduce those assets; it preserved the generated Meshy surface and texture exactly. A material improvement to anatomy, silhouette and authored detail requires a new or manually improved source rather than another weight-only adjustment.

Exact candidate-bound result: `documentation/evidence/borzoi-c-wolf-demo-v2-owner-nwn-result-2026-08-20.json`.
