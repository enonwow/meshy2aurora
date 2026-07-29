# TLC Stoneback Brute P300K Cleanup V3 — ready for owner proof

1. Test-module filename: `m2p3gd0eeb135c.mod`
2. Module name shown in Toolset: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Status: `ready_for_owner_proof`

## Iteration admission and exact delta

The owner tested exact cleanup V2 (`m2p3fd0eeb135c.mod`) in NWN and supplied a
fresh judgeable runtime capture. The Creature was visible, but numerous bright
speckles remained:

- capture SHA-256:
  `20875ac785e7f8736f0ee1f043761397d63157dd838f40ff19a4f3bb96a78fdf`;
- `modelVisibility = visible`;
- `proofCompleteness = verified`;
- `textureArtifactResult = unchanged`.

The V2 classifier required the raw luminance range of all neighbors to remain
at most `30`. One additional bright or dark neighbor therefore rejected the
center sample, allowing pairs and short runs of speckles to protect one
another.

The V3 delta is limited to:

- algorithm `ROBUST_LOCAL_NEIGHBOR_MEDIAN_V2`;
- two deterministic local passes;
- center classification against the second-lowest and second-highest opaque
  neighbor, so one other extreme neighbor cannot veto a repair;
- RGB replacement by the per-channel median of opaque pixels in the local
  `3 x 3` neighborhood;
- distinct `g` identity variant.

There is no random color generation. Source GLB, geometry, skin, animations,
module layout, Appearance row and fixture placement remain unchanged.

## Exact application route

The candidate was generated and captured through the production application
route:

`Studio Worker -> m2a-wasm -> m2a-core -> owned MDL/TGA/2DA/HAK/MOD writers -> canonical result projector`

The real headless Chromium Worker/WASM integration returned the exact artifact
buffers. The Vitest-side capture command persisted only complete, ordered
buffers whose SHA-256 matched the identities returned by the application. It
did not implement or replace an asset writer.

## Source and identity

- source:
  `sample-3d/tlc-stoneback-brute-h1-p300k-v1/source.glb`;
- source bytes: `20,256,180`;
- source SHA-256:
  `0eeb135c1077f2207b1f362222829a2493c003c64276db98633cf2af5c8385b3`;
- input `appearance.2da` SHA-256:
  `815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a`;
- model: `m2p3gm0eeb135c`;
- texture: `m2p3gt0eeb135c`;
- HAK: `m2p3gh0eeb135c`;
- MOD: `m2p3gd0eeb135c`;
- Area resref: `m2p3ga0eeb135c`;
- Creature resref: `m2p3gc0eeb135c`;
- Appearance row: `15100`.

## Texture cleanup result

- cleanup enabled: `true`;
- algorithm: `ROBUST_LOCAL_NEIGHBOR_MEDIAN_V2`;
- deterministic passes: `2`;
- inspected interior pixels: `4,186,116`;
- repaired color outliers: `3,682`;
- repaired one-pixel alpha holes: `0`;
- input pixel SHA-256:
  `fd05864b65c21dc98cc52131d0c6bc012ad546646a04940ef4135aaf9566f163`;
- output pixel SHA-256:
  `029b4191f10820fe4d7e1c6772529a7a2dbab4289144b02b425c223a510f111e`;
- output TGA SHA-256:
  `6623657cd8f7c6b875eb7cefe0fd8a18a4cbb306dd0e8abfd1c647cdb5c73d1a`.

Synthetic tests prove that isolated bright/dark samples, adjacent pairs and
three-pixel runs are filled from local neighbors. A separate negative test
preserves a material boundary and a multi-pixel transparent region.

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

`proof-output/tlc-stoneback-brute-p300k-cleanup-v3/studio-export`

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2p3gd0eeb135c.mod` | 15,193 | `d306942bdd93379d56fae6dd5fd47e4df6f80e7b7aea1a8f1a3544f93cb60a23` |
| `m2p3gh0eeb135c.hak` | 47,420,702 | `46bcd985c91a8ca81f7112a4e6b2b274fe0898aa9c9fb333793fca8e5e4e03f8` |
| `m2p3gm0eeb135c.mdl` | 23,741,864 | `4a5b55645cd6b0bed96c8025ab19feb439fe9a1041f33f224b54a0c385550eba` |
| `m2p3gt0eeb135c.tga` | 16,777,260 | `6623657cd8f7c6b875eb7cefe0fd8a18a4cbb306dd0e8abfd1c647cdb5c73d1a` |
| `inspection.json` | 1,243,627 | `c87fdc69943fec30f96d619413ccac5cdc935b59677aa190aa3f652edc9270a2` |
| `conversion-manifest.json` | 3,197 | `99cecfcf47d283979a16bc8a11bd6477e08d01c86d8a6994a712ae65abe950d4` |
| `summary.json` | 1,735 | `445e1aeed63f6d07e02e0346129776552424101ae8acdf12339b1b7c26d088e5` |

The appended `appearance.2da` SHA-256 is:

`07181f13425d6b0e4178e156d6df41f9c1fa8db26f23e382251ffea1dfe95caa`.

## Native installation

Both exact destinations were absent before installation. They were copied
without overwrite and verified byte-identical afterward:

| Artifact | Native destination | Verified SHA-256 |
|---|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2p3gd0eeb135c.mod` | `d306942bdd93379d56fae6dd5fd47e4df6f80e7b7aea1a8f1a3544f93cb60a23` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2p3gh0eeb135c.hak` | `46bcd985c91a8ca81f7112a4e6b2b274fe0898aa9c9fb333793fca8e5e4e03f8` |

## Owner test

Open exact `m2p3gd0eeb135c.mod`, then Area
`Meshy2Aurora M0 binary vertical-slice area`.

The Creature template is `m2p3gc0eeb135c`, Appearance row is `15100`, and the
fixture is placed directly in front of the player at `[10.0, 14.5, 0.0]`.

Compare the dark torso, shoulders and arms against cleanup V2. Record:

- `modelVisibility = visible | not_visible | not_tested`;
- `proofCompleteness = verified | failed | missing`;
- `textureArtifactResult = fixed | improved | unchanged | worse | not_tested`.

No agent-run Toolset or NWN session was started. The visual result remains
owner-owned and is not claimed by this offline packet.

## Owner NWN result: cleanup V3 failed

The owner tested the exact installed cleanup-V3 handoff and supplied a fresh,
judgeable NWN runtime capture:

- capture:
  `C:\Users\enonw\AppData\Local\Temp\codex-clipboard-6d95b356-b7e2-4797-bbb6-e3b4a6ae6ce0.png`;
- capture dimensions: `919 x 815`;
- capture bytes: `903,058`;
- capture SHA-256:
  `e28bcd3c3dfcc16e5b9b16da6f290f592f785991a3434b258a36deac8ebddbf8`;
- `modelVisibility = visible`;
- `proofCompleteness = verified`;
- `textureArtifactResult = unchanged`.

Bright point artifacts remain visible on the shoulders, head and torso.

Offline readback excludes a malformed material or normal stream:

- all `213,331` MDL vertex normals are finite;
- normal lengths are within
  `0.999999890783154 .. 1.0000001328953991`;
- no face/vertex-normal dot product is negative;
- the exact MDL material has `specular = [0, 0, 0]`.

The exact V3 TGA still contains thousands of locally bright residual samples.
The `3 x 3` rank classifier protects too much real Meshy microtexture around a
candidate pixel, even when that microtexture collapses to a bright point under
NWN minification.

This fresh candidate-bound failure admits one minimal successor iteration. Its
allowed delta is an optional edge-aware `5 x 5` Hampel/median impulse filter:

- robust median and median absolute deviation instead of a raw/rank range;
- minimum absolute contrast before any repair;
- same-color support test protecting coherent details and material borders;
- replacement RGB derived only from local neighboring texels;
- no global blur and no random color.

Source GLB, geometry, normals, skin, animations, module layout and placement
remain unchanged.
