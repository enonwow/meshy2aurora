# `m2aborzmod3.mod`

Toolset module name: `Meshy2Aurora Borzoi c_wolf Demo V3`\
Exact Area name: `Meshy2Aurora Borzoi Test Area V3`

Date: 2026-08-20

Status: owner NWN proof recorded. The model is visible and inherited `c_wolf`
combat behavior is active, but animated deformation quality failed.

## Owner NWN result — 2026-08-20

- NWN: `modelVisibility=visible`, `proofCompleteness=verified`;
- animation inheritance: `observed` in active combat;
- quality: `failed`;
- severe head/neck compression and twisting is visible;
- coat and limb regions stretch into spikes and sheet-like artifacts;
- the animated silhouette and anatomy are not acceptable for a finished dog.

Evidence:
`documentation/evidence/borzoi-c-wolf-demo-v3-owner-nwn-result-2026-08-20.png`,
SHA-256 `6d477f5eb0eb84a05e3ece4397ad8271bbb7667303697485fdb111d712736e80`.
The structured verdict is in
`documentation/evidence/borzoi-c-wolf-demo-v3-owner-nwn-result-2026-08-20.json`.

The V3 primary-region gate was therefore necessary but insufficient. It proves
bone coverage, not correct semantic ownership, bind-pose compatibility, bounded
edge stretch or the absence of triangle flips under inherited clips. The
catastrophic deformation points to the procedural rig/bind contract rather
than the shared 300,000-triangle product budget as the primary failure.

The visibility-only gate did not initially admit a new lineage. Later on
2026-08-20 the owner explicitly ordered correction and V4 after reviewing this
exact failure, thereby granting the required quality-remediation exception.
The exact V3 bytes and negative quality evidence remain immutable.

## Exact owner handoff

- Test module file: `m2aborzmod3.mod`
- Ordered HAK: `m2aborzhak3`
- Creature blueprint: `m2aborzutc3`
- Appearance row: `848`
- Model resref: `m2aborzcre3`
- Texture resref: `m2aborztex3`
- Fixture: `(10.0, 14.5, 0.0)`, facing `(0.0, -1.0)` toward the player entry point

The exact MOD and HAK are installed in the native NWN user directories. Both destination hashes are byte-identical to the canonical proof-output sources.

## New owner-supplied source

The canonical source is `sample-3d/borzoi-meshy-manual-p1997k-v1/source.glb`, SHA-256 `71949d52f0f9e68da0642511d786e144bfdd47491cc7bae9c0fe09bfbb3fac54`.

The source contains 1,997,064 triangles and cannot enter the product conversion route directly. The existing meshoptimizer stage created the explicit canonical variant `source-p300k.glb`, SHA-256 `f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda`, with exactly 300,000 triangles and 233,832 vertices. Materials and all three embedded images remain in the GLB. Superseded source geometry was compacted out of the result instead of remaining as unreachable payload.

## V3 rig remediation

The exact V2 owner result admitted this iteration with `qualityVerdict=failed`: both front upper-leg bones had zero primary deformation vertices and the previous source quality was rejected.

V3 uses the owner-supplied higher-quality surface and implements the declared minimum rig delta:

- source-fitted paw contact clusters;
- continuous shoulder-to-paw chain parameterization instead of nearest-two-node weighting;
- a blocking offline gate requiring a non-zero primary vertex region for every bone in all four leg chains;
- 16/16 leg bones passed the primary-region gate;
- left and right front upper-leg primary regions contain 1,617 and 1,668 vertices respectively.

The generated model contains the compatible 30-node hierarchy, zero local animation clips and inherits 42 clips from the base-game `c_wolf` supermodel. Creature Basis V3 maps glTF positive-Z to Aurora positive-Y with determinant `+1`.

## Immutable artifact identity

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2aborzmod3.mod` | 15,205 | `109dd1db637b1c744fd1a18b6daacfe5fc479e9cb1e04a5a7c44442a7f454c8e` |
| `m2aborzhak3.hak` | 76,222,468 | `60f5d7fa072b4375e8be95a7f995f0674d53a6847e067aa56ac3bea51e05e7ff` |
| `m2aborzcre3.mdl` | 25,502,368 | `c12087f62201fdc13c021e158194e58378912acb529508e0aca42e7b8bdba978` |
| `m2aborztex3.tga` | 50,331,692 | `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1` |

Canonical packet: `proof-output/borzoi-c-wolf-demo-v3-20260820`.

## Verification boundary

Offline model semantic readback, HAK resource readback, MOD scene readback and the retail `c_wolf` reference contract passed. The owner-owned axes now are:

- Toolset: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- NWN: `modelVisibility=visible`, `proofCompleteness=verified`, `qualityVerdict=failed`.

No NWN visual-success claim is made for V3: visibility and animation inheritance
are proven, while the model-quality acceptance criterion failed.
