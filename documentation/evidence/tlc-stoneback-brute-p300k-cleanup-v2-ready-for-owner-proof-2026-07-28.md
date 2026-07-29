# TLC Stoneback Brute P300K Cleanup V2 — ready for owner proof

1. Test-module filename: `m2p3fd0eeb135c.mod`
2. Module name shown in Toolset: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Status: `ready_for_owner_proof`

## Iteration admission and exact delta

The owner visually verified the previous exact P300K candidate
`m2p3d0eeb135c.mod` in NWN and reported isolated white speckles on the rendered
Creature. The previous model was visible; the candidate-bound failure concerned
the base-color texture.

The diagnosed source was isolated bright and dark color outliers in the Meshy
base-color image. The intended V2 delta was:

- enable `textureArtifactCleanup=true`;
- preserve the exact Meshy source and geometry/animation pipeline;
- use a distinct deterministic `f` identity variant so the changed HAK cannot
  collide with the installed V1 HAK;
- keep the old V1 MOD/HAK unchanged.

The `f` marker means the first texture-artifact-fix identity variant. Builds
with the option disabled retain their previous deterministic names.

## Exact application route

The candidate was built and captured through:

`Studio Worker -> m2a-wasm -> m2a-core -> owned MDL/TGA/2DA/HAK/MOD writers -> canonical result projector`

The real Chromium Worker/WASM integration returned the exact artifact buffers.
The Vitest server command only persisted those returned bytes after contiguous
chunk assembly and SHA-256 verification. It did not implement or replace an
MDL, TGA, 2DA, HAK or MOD writer.

## Source and identity

- source:
  `sample-3d/tlc-stoneback-brute-h1-p300k-v1/source.glb`;
- source bytes: `20,256,180`;
- source SHA-256:
  `0eeb135c1077f2207b1f362222829a2493c003c64276db98633cf2af5c8385b3`;
- input `appearance.2da` SHA-256:
  `815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a`;
- model: `m2p3fm0eeb135c`;
- texture: `m2p3ft0eeb135c`;
- HAK: `m2p3fh0eeb135c`;
- MOD: `m2p3fd0eeb135c`;
- Area resref: `m2p3fa0eeb135c`;
- Creature resref: `m2p3fc0eeb135c`;
- Appearance row: `15100`.

## Texture cleanup result

- cleanup enabled: `true`;
- inspected interior pixels: `4,186,116`;
- repaired isolated color outliers: `1,050`;
- repaired one-pixel alpha holes: `0`;
- input pixel SHA-256:
  `fd05864b65c21dc98cc52131d0c6bc012ad546646a04940ef4135aaf9566f163`;
- output pixel SHA-256:
  `af28f6a5298ad460ffe10fff3d4cdbc11829ceeee80a59f9fb86043743e177cc`;
- output TGA SHA-256:
  `170d6ac6b1321fe8e5b12bab7e7fa3d804b8ed3d113ea53c05087ffdde339455`.

The filter remained conservative: it did not repair clusters, edges, larger
highlights or larger transparent regions.

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

`proof-output/tlc-stoneback-brute-p300k-cleanup-v2/studio-export`

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2p3fd0eeb135c.mod` | 15,193 | `759868b9d64eea5a13685323f516e0677bb7b65997d1accba6a0adb40c1cab42` |
| `m2p3fh0eeb135c.hak` | 47,420,702 | `92af04fb129d46379158f9d4351ef3931bfbd8b7d470f05c31af321fcfb2287a` |
| `m2p3fm0eeb135c.mdl` | 23,741,864 | `b9f0971e68381984e205b850b5b2882d65fc7af48d7d9ee9e4e4e6c5b17b7d64` |
| `m2p3ft0eeb135c.tga` | 16,777,260 | `170d6ac6b1321fe8e5b12bab7e7fa3d804b8ed3d113ea53c05087ffdde339455` |
| `inspection.json` | 1,243,555 | `dc1464e2f0ab7a783ae58df51e807c319c58228b51735f7b71769453c24bec1b` |
| `conversion-manifest.json` | 3,197 | `c7ef57b25655559deae4570f7fa1ed276ed64837c2c852e1c0b657755b380fd4` |
| `summary.json` | 1,735 | `48eaaa18c82243f67990a40a0493a02f183b66a581ea33053958b2aa92c58a8b` |

The exact HAK binds:

- `m2p3fm0eeb135c.mdl`, SHA-256
  `b9f0971e68381984e205b850b5b2882d65fc7af48d7d9ee9e4e4e6c5b17b7d64`;
- `m2p3ft0eeb135c.tga`, SHA-256
  `170d6ac6b1321fe8e5b12bab7e7fa3d804b8ed3d113ea53c05087ffdde339455`;
- appended `appearance.2da`, SHA-256
  `ea5655bb2bae2fd4180ea29107d401dafb4c4b5c738360f11c2184c5539ed401`.

## Native installation

Both exact destinations were absent before installation. They were copied
without overwrite and verified byte-identical afterward:

| Artifact | Native destination | Verified SHA-256 |
|---|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2p3fd0eeb135c.mod` | `759868b9d64eea5a13685323f516e0677bb7b65997d1accba6a0adb40c1cab42` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2p3fh0eeb135c.hak` | `92af04fb129d46379158f9d4351ef3931bfbd8b7d470f05c31af321fcfb2287a` |

## Owner test

Open exact `m2p3fd0eeb135c.mod`, then Area
`Meshy2Aurora M0 binary vertical-slice area`.

The Creature template is `m2p3fc0eeb135c`, Appearance row is `15100`, and the
fixture is placed directly in front of the player at `[10.0, 14.5, 0.0]`.

Compare the surface against the old V1 candidate, concentrating on isolated
white speckles. Record Toolset and NWN independently:

- `modelVisibility = visible | not_visible | not_tested`;
- `proofCompleteness = verified | failed | missing`;
- `textureArtifactResult = fixed | improved | unchanged | worse | not_tested`.

No agent-run Toolset or NWN session was started. The visual result remains
owner-owned and is not claimed by this offline packet.

## Owner NWN result: cleanup V2 failed

The owner tested the exact installed cleanup-V2 handoff and supplied a fresh
NWN runtime capture:

- capture:
  `C:\Users\enonw\AppData\Local\Temp\codex-clipboard-e741975e-d2ed-44b6-a715-9dc1faf58c60.png`;
- capture dimensions: `918 x 781`;
- capture bytes: `918,821`;
- capture SHA-256:
  `20875ac785e7f8736f0ee1f043761397d63157dd838f40ff19a4f3bb96a78fdf`;
- `modelVisibility = visible`;
- `proofCompleteness = verified`;
- `textureArtifactResult = unchanged`.

The capture is judgeable and still shows numerous isolated bright speckles on
the dark creature surface. This is a texture-cleanup failure, not a creature
visibility failure.

Offline diagnosis binds the failure to the V1 cleanup classifier used by this
V2 candidate. Replacement colors were never random: each repaired RGB channel
was the median of the opaque pixels in the local `3 x 3` neighborhood. The
defect is the classifier's raw neighbor-range gate. One additional bright or
dark neighbor expands the raw range above `30`, rejects the center pixel and
allows adjacent speckles to protect one another from repair.

This fresh candidate-bound failure admits one minimal successor iteration. Its
allowed delta is limited to a deterministic robust local-neighbor classifier
with bounded repeated passes; source GLB, geometry, skin, animations, module
layout and fixture placement remain unchanged.
