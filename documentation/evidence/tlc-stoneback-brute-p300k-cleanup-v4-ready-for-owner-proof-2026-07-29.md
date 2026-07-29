# TLC Stoneback Brute P300K Cleanup V4 — ready for owner proof

1. Test-module filename: `m2p3hd0eeb135c.mod`
2. Module name shown in Toolset: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Status: `ready_for_owner_proof`

## Iteration admission

The owner tested exact cleanup V3 (`m2p3gd0eeb135c.mod`) in NWN and supplied a
fresh, judgeable runtime capture:

- capture SHA-256:
  `e28bcd3c3dfcc16e5b9b16da6f290f592f785991a3434b258a36deac8ebddbf8`;
- `modelVisibility = visible`;
- `proofCompleteness = verified`;
- `textureArtifactResult = unchanged`.

Bright point artifacts remained visible on the shoulders, head and torso.

## Root-cause audit

Exact MDL readback excluded malformed normals and specular lighting:

- `213,331` vertex normals are finite;
- normal lengths are within
  `0.999999890783154 .. 1.0000001328953991`;
- no face/vertex-normal dot product is negative;
- only `15 / 870,954` face/vertex pairs have a dot product below `0.5`;
- the material has `specular = [0, 0, 0]` and `shininess = 1`.

The remaining points originate in high-frequency Meshy base-color microdetail
that collapses to bright impulses when NWN minifies the dense UV atlas. Cleanup
V3 examined only `3 x 3` neighborhoods and therefore retained thousands of
locally bright residual samples.

## Exact V4 delta

The optional checkbox remains `Repair texture artifacts`
(`textureArtifactCleanup=true`). Its active algorithm is now:

`EDGE_AWARE_HAMPEL_MEDIAN_V3`

The algorithm:

1. examines a local `5 x 5` neighborhood;
2. uses the neighboring luminance median and median absolute deviation;
3. requires an absolute luminance difference of at least `24`;
4. requires the difference to be at least four local median deviations;
5. protects coherent detail and material borders by rejecting a center with
   more than two similar neighbors;
6. requires at least half of the opaque neighborhood to agree with its median;
7. repairs bright impulses only; dark albedo detail is not blurred;
8. writes the per-channel median of opaque neighboring texels;
9. performs two deterministic passes;
10. retains the separate conservative `3 x 3` alpha-hole repair.

There is no random color, global blur or model/material mutation.

Source GLB, geometry, UVs, normals, skin, animations, module layout, Appearance
row and fixture placement remain unchanged. The changed package uses the fresh
`h` identity variant.

## Exact application route

The candidate was generated and captured through:

`Studio Worker -> m2a-wasm -> m2a-core -> owned MDL/TGA/2DA/HAK/MOD writers -> canonical result projector`

The real headless Chromium Worker/WASM integration returned the exact artifact
buffers. The Vitest-side capture command persisted only complete ordered
buffers after SHA-256 verification; it did not implement an asset writer.

## Source and identity

- source:
  `sample-3d/tlc-stoneback-brute-h1-p300k-v1/source.glb`;
- source bytes: `20,256,180`;
- source SHA-256:
  `0eeb135c1077f2207b1f362222829a2493c003c64276db98633cf2af5c8385b3`;
- input `appearance.2da` SHA-256:
  `815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a`;
- model: `m2p3hm0eeb135c`;
- texture: `m2p3ht0eeb135c`;
- HAK: `m2p3hh0eeb135c`;
- MOD: `m2p3hd0eeb135c`;
- Area resref: `m2p3ha0eeb135c`;
- Creature resref: `m2p3hc0eeb135c`;
- Appearance row: `15100`.

## Texture cleanup result

- cleanup enabled: `true`;
- algorithm: `EDGE_AWARE_HAMPEL_MEDIAN_V3`;
- deterministic passes: `2`;
- inspected interior pixels: `4,177,936`;
- repaired bright color impulses: `4,588`;
- repaired one-pixel alpha holes: `0`;
- input pixel SHA-256:
  `fd05864b65c21dc98cc52131d0c6bc012ad546646a04940ef4135aaf9566f163`;
- output pixel SHA-256:
  `62ebd7a04eddcdb13ae43133c2aa78513cb426ce2ed056c59db11aa74be50937`;
- output TGA SHA-256:
  `ac6aceb3f2809c5ffe1170d2525df672fb077855cf8bc1d78f24241596f05fbc`.

Synthetic tests prove:

- isolated bright impulse repair;
- three-pixel run repair;
- one-pixel alpha-hole repair;
- dark-detail preservation;
- coherent `3 x 3` highlight preservation;
- material-border preservation;
- multi-pixel transparent-region preservation;
- deterministic output and immutable input;
- byte-exact disabled behavior.

## Geometry and animation result

- written/read-back triangles: `290,318`;
- written vertices: `213,331`;
- active joints: `22`;
- deformation: `SKIN`;
- required NWN Creature states: `42`;
- animation completeness: `true`;
- canonical result status: `M6_MODEL_PACKAGE_MATERIALIZED`.

## Frozen Studio artifacts

Canonical packet:

`proof-output/tlc-stoneback-brute-p300k-cleanup-v4/studio-export`

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2p3hd0eeb135c.mod` | 15,193 | `86c20ae34adab397344477ccde3670ef7e26e9617a5aa0dc417720a7fa6d79a2` |
| `m2p3hh0eeb135c.hak` | 47,420,702 | `eee444523d1f2fb170dadeb6b194d3d49e331dae30a21f7859400a092d57aec4` |
| `m2p3hm0eeb135c.mdl` | 23,741,864 | `781612741ab90f6e9d61ff60dd356d2b602e50e62e96d912f18715295f56b2d8` |
| `m2p3ht0eeb135c.tga` | 16,777,260 | `ac6aceb3f2809c5ffe1170d2525df672fb077855cf8bc1d78f24241596f05fbc` |
| `inspection.json` | 1,243,623 | `af2b1d970439056ff9cc58324fa87a4454106449bf989d744c1adb27a0a23bdb` |
| `conversion-manifest.json` | 3,197 | `6de80be9711d370006297e3745a48809e7f84f1ecee8ec11f8cd9ca2f800dcf2` |
| `summary.json` | 1,735 | `58423351cb65d65019205e27100ca92887ee0320f9fbb51cb5bb50dbc52a577a` |

The appended `appearance.2da` SHA-256 is:

`8ba7048a55937293081e784f0d9adb8b9a9c3e9155b0984ac187a359d09755d8`.

## Native installation

Both exact destinations were absent before installation. They were copied
without overwrite and verified byte-identical afterward:

| Artifact | Native destination | Verified SHA-256 |
|---|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2p3hd0eeb135c.mod` | `86c20ae34adab397344477ccde3670ef7e26e9617a5aa0dc417720a7fa6d79a2` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2p3hh0eeb135c.hak` | `eee444523d1f2fb170dadeb6b194d3d49e331dae30a21f7859400a092d57aec4` |

## Owner test

Open exact `m2p3hd0eeb135c.mod`, then Area
`Meshy2Aurora M0 binary vertical-slice area`.

The Creature template is `m2p3hc0eeb135c`, Appearance row is `15100`, and the
fixture is placed directly in front of the player at `[10.0, 14.5, 0.0]`.

Inspect the dark shoulders, head, chest and arms. Record:

- `modelVisibility = visible | not_visible | not_tested`;
- `proofCompleteness = verified | failed | missing`;
- `textureArtifactResult = fixed | improved | unchanged | worse | not_tested`.

No agent-run Toolset or NWN session was started. Offline evidence proves the
pipeline delta, but only the owner can close the visual result.
