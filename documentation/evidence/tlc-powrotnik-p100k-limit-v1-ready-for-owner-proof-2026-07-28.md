# TLC Powrotnik P100K limit V1 — ready for owner proof

> **POLICY SUPERSEDED 2026-07-29:** tożsamość, hashe i wynik tego historycznego
> eksperymentu pozostają bez zmian. Bieżący wspólny limit produktu wynosi
> 300 000 trójkątów; opis 20K niżej jest capture-time policy.

1. Test-module filename: `tlcp100demo1.mod`
2. Module name shown in Toolset: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Status: `ready_for_owner_proof`

## Purpose

This is an isolated NWN render-boundary experiment requested by the owner. It
does not change the production policy
`AURORA_MODEL_TRIANGLE_BUDGET_V1 = 20_000`.

The test asks whether one animated Creature with approximately 100K
renderable triangles can be visible in Aurora/NWN when the binary MDL
per-mesh `u16`/65,535-index boundary is satisfied by deterministic
multi-SkinMesh segmentation.

## Meshy provenance

- provider: Meshy;
- source image:
  `documentation/mockups/the-last-city/creatures/tlc-creature-powrotnik-tpose-front-v1.png`;
- Meshy run: `b4b66a10-b167-47db-854c-c9e8f7b6f277`;
- requested `target_polycount`: `100000`;
- pose: T-pose;
- rig height: `1.85 m`;
- explicit Meshy animations: `10`;
- credits spent: `65`;
- combined source:
  `sample-3d/tlc-powrotnik-h1-p100k-limit-v1/source.glb`;
- combined source SHA-256:
  `ad840cb78566967ecfbc9767625867e502edef8c1e5bc0c65caac1688c258c70`.

Meshy emitted `103,290` raw faces. The shared Aurora-safe sanitation removed
`5,657` zero-area/degenerate faces and compacted `4,255` now-unreferenced
vertices. The exact candidate therefore contains `97,633` renderable
triangles and `82,815` source vertices. This is the immutable result being
tested; it must not be described as exactly 100,000 renderable triangles.

## Offline materialization

Packet:

`proof-output/tlc-powrotnik-p100k-limit-v1`

The binary writer preserved all `97,633` safe triangles and emitted five
SkinMesh streams. The deterministic stream triangle counts are:

`21,845 + 21,845 + 21,845 + 21,845 + 10,253 = 97,633`.

Every stream is independently below the binary MDL boundary of `65,535`
index entries. The model exposes:

- 42 native Creature animation states;
- 10 source-derived Meshy animations;
- complete offline behavior oracle;
- complete kinematics oracle;
- complete animation-event conformance;
- complete SkinMesh deformation conformance;
- Appearance physical row `15100`;
- creature template resref `tlcp100utc1`;
- model resref `tlcpow100_m1`;
- texture resref `tlcpow100_t1`.

Fixture placement is `[10.0, 14.5, 0.0]`, directly in front of the player
entry `[10.0, 10.0, 0.0]`.

## Frozen artifact identity

| Artifact | Canonical source | Native destination | Bytes | SHA-256 |
|---|---|---|---:|---|
| MOD | `proof-output/tlc-powrotnik-p100k-limit-v1/generated/tlcp100demo1.mod` | `C:\Users\enonw\Documents\Neverwinter Nights\modules\tlcp100demo1.mod` | 15,174 | `570b35eac41d7a0323ad65cbd8c7511c5216bc9a78d79f629515cfe11bc0164b` |
| HAK | `proof-output/tlc-powrotnik-p100k-limit-v1/generated/tlcp100hak1.hak` | `C:\Users\enonw\Documents\Neverwinter Nights\hak\tlcp100hak1.hak` | 28,717,724 | `69327838a7242528470e693e5f6525ef5d04642734e1f9c32d9dfd87ce48d8e6` |
| MDL | `proof-output/tlc-powrotnik-p100k-limit-v1/generated/tlcpow100_m1.mdl` | inside exact HAK | 9,233,192 | `3ff329e6357630c6dc97c9fc2e83535b644726aca439be78250563eda3d9177f` |
| TGA | `proof-output/tlc-powrotnik-p100k-limit-v1/generated/tlcpow100_t1.tga` | inside exact HAK | 12,582,956 | `4ba85e147b341a924ead5680b6deb94514861ad001ad0248fcf7ee6b23ae9566` |
| appearance.2da | `proof-output/tlc-powrotnik-p100k-limit-v1/generated/appearance.2da` | inside exact HAK | 6,901,320 | `255a6a99751ccb4b7ee37a4e3797e7ef4942a512fb03ba251fe339df7b9abe81` |

The MOD and HAK destinations were absent before installation. They were
copied without overwrite and their native SHA-256 values were verified as
byte-identical to the canonical packet.

## Owner test

Open exact `tlcp100demo1.mod`, then Area
`Meshy2Aurora M0 binary vertical-slice area`.

Record the two axes independently:

- Toolset `modelVisibility = visible | not_visible | not_tested`;
- Toolset `proofCompleteness = verified | failed | missing`;
- NWN `modelVisibility = visible | not_visible | not_tested`;
- NWN `proofCompleteness = verified | failed | missing`.

For the triangle-limit question, the decisive observation is NWN visibility
of the exact fixture directly in front of the player. If it is visible, the
test disproves a universal 10K or 20K NWN engine limit for one Creature and
demonstrates at least `97,633` rendered triangles across five SkinMesh
streams. It does not by itself prove that one unsplit stream can exceed
21,845 triangles or that arbitrary higher totals are safe.

No agent-run Toolset or NWN session was started. Final visual proof remains
owner-owned.
