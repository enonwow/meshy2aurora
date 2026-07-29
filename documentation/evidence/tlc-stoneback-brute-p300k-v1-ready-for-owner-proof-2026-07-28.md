# TLC Stoneback Brute P300K V1 — ready for owner proof

> **POLICY SUPERSEDED 2026-07-29:** tożsamość, hashe i wynik tego historycznego
> eksperymentu pozostają bez zmian. Bieżący wspólny limit produktu wynosi
> 300 000 trójkątów; opis 20K niżej jest capture-time policy.

1. Test-module filename: `m2p3d0eeb135c.mod`
2. Module name shown in Toolset: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Status: `ready_for_owner_proof`

## Scope

This is an isolated, explicitly selected P300K Creature experiment. It does
not change the shared product limit
`AURORA_MODEL_TRIANGLE_BUDGET_V1 = 20_000`.

The complete artifact build ran through the application boundary:

`Studio UI -> Web Worker -> m2a-wasm -> m2a-core -> owned MDL/TGA/2DA/HAK/MOD writers`

No JavaScript format writer, example CLI materializer, or ad-hoc packet writer
created this candidate.

## Meshy source

- canonical asset:
  `sample-3d/tlc-stoneback-brute-h1-p300k-v1`;
- combined source:
  `sample-3d/tlc-stoneback-brute-h1-p300k-v1/source.glb`;
- combined source SHA-256:
  `0eeb135c1077f2207b1f362222829a2493c003c64276db98633cf2af5c8385b3`;
- combined source bytes: `20,256,180`;
- Meshy model task:
  `019fa98b-2acc-72e3-96a6-0404d7985807`;
- final remesh task:
  `019fa999-54f5-7d42-bc0d-69cdd8b0cd5a`;
- rig task:
  `019fa99b-d13d-74e1-82ba-fbdc3be48201`;
- requested Meshy target: `300,000`;
- initial generated face count: `300,844`;
- riggable combined source: `296,276` raw triangles, `199,844`
  vertices, one Skin, 24 source joints and 10 Meshy animation clips;
- Meshy credits: `75`, within the owner cap of `250`.

Meshy target polycount is a target, not an exact output guarantee. The
300,844-face first result was above Meshy's 300,000-face rigging boundary.
The same exact source lineage was remeshed to a riggable result; no unrelated
model was substituted.

## Application build result

Studio used the explicit `EXPERIMENTAL_P300K` profile.

| Metric | Result |
|---|---:|
| Source triangles shown before sanitation | 296,276 |
| Source triangles after Aurora sanitation | 290,319 |
| Written/read-back MDL triangles | 290,318 |
| Written segmented vertices | 213,331 |
| Binary SkinMesh segments | 14 |
| Active output joints | 22 |
| Source Meshy clips | 10 |
| NWN Creature animation states | 42 |

One additional near-zero face was removed at the final binary-writer
face-plane threshold. The pipeline now uses one shared
`NWN_EE_BINARY_MDL_EPSILON_V1` value for source sanitation, final runtime
sanitation and the binary writer. Studio reports source and written geometry
separately instead of incorrectly requiring both totals to be identical.

The exact real-browser Worker/WASM integration completed successfully.
Canonical binary readback reported:

- one model root;
- 14 SkinMesh nodes;
- 42 animations;
- empty writer semantic diff;
- complete animation namespace;
- distinct walk/run behavior;
- continuous death-family boundary;
- complete kinematics, event and skin-deformation gates.

## Frozen Studio artifacts

Canonical packet:
`proof-output/tlc-stoneback-brute-p300k-v1/studio-export`.

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2p3d0eeb135c.mod` | 15,185 | `f35e0c796dc569af87dbdf253aa627ab4c0a73853a131bbf95b42766fb8d0907` |
| `m2p3h0eeb135c.hak` | 47,420,701 | `937a20ff748f1ea817d6d7b48f094292d7a3dd259c8fcab31795d0d358946ceb` |
| `m2p3m0eeb135c.mdl` | 23,741,864 | `499a0e4dc4d98317ed0b4bbe79686354c7bd7e445876d6ca758ca000039ef91e` |
| `inspection.json` | 1,243,169 | `cd88a55e4094cfb03cf28a1b4b4286872834d82ce7fb9e8e053736373687bd42` |
| `conversion-manifest.json` | 3,191 | `381713514bd7524b63a386cf7de7b0731cc89a3d0c2d507d96ec9a2f3ad2a52f` |
| `summary.json` | 1,733 | `14ffd141899098985005679e464e26b5d99ed10acf57656fc2dc7ef871195744` |

The exact HAK contains:

- `m2p3m0eeb135c.mdl`, SHA-256
  `499a0e4dc4d98317ed0b4bbe79686354c7bd7e445876d6ca758ca000039ef91e`;
- `m2p3t0eeb135c.tga`, SHA-256
  `f2942325b52e9340d2c006fce85a0292182350a4433c3f0e8317b78b9d8180a2`;
- appended `appearance.2da`, physical row `15100`, SHA-256
  `572ee5364b52e2d49a52bd86789ebf8d08a0184cd449895a858d0c5b870bcd26`.

## Native installation

Both destination files were absent before installation. The application
artifacts were copied without overwrite and verified byte-identical afterward:

| Artifact | Native destination | Verified SHA-256 |
|---|---|---|
| MOD | `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2p3d0eeb135c.mod` | `f35e0c796dc569af87dbdf253aa627ab4c0a73853a131bbf95b42766fb8d0907` |
| HAK | `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2p3h0eeb135c.hak` | `937a20ff748f1ea817d6d7b48f094292d7a3dd259c8fcab31795d0d358946ceb` |

## Owner test

Open exact `m2p3d0eeb135c.mod`, then Area
`Meshy2Aurora M0 binary vertical-slice area`.

The Creature template is `m2p3c0eeb135c`, Appearance row is `15100`, and the
fixture is placed directly in front of the player at
`[10.0, 14.5, 0.0]`.

Record Toolset and NWN independently:

- `modelVisibility = visible | not_visible | not_tested`;
- `proofCompleteness = verified | failed | missing`.

No agent-run Toolset or NWN session was started. Current state for both lanes
is `modelVisibility=not_tested`, `proofCompleteness=missing`; final visual
proof remains owner-owned.
