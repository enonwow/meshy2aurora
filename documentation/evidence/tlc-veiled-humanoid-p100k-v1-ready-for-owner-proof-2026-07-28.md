# TLC Veiled Humanoid P100K V1 — ready for owner proof

1. Test-module filename: `tlcv100demo1.mod`
2. Module name shown in Toolset: `Meshy2Aurora procedural humanoid proof`
3. Exact Area name: `Meshy2Aurora M0 binary vertical-slice area`

Status: `ready_for_owner_proof`

## Application-pipeline amendment — 2026-07-28

The original packet was frozen through the project example CLI calling the
canonical `m2a-core` writers. That was valid core materialization, but it was
not a Studio UI/Worker/WASM execution and must not be presented as such.

The missing explicit P100K Studio route has since been implemented and the
same source, full `appearance.2da`, and exact resource identity were replayed
through Studio -> Worker -> WASM -> core. MOD, HAK, MDL, TGA and appended 2DA
were byte-identical to this frozen packet. No packet or native installation was
rewritten. Full evidence:
[tlc-veiled-humanoid-p100k-studio-pipeline-replay-2026-07-28.md](tlc-veiled-humanoid-p100k-studio-pipeline-replay-2026-07-28.md).

## Exact candidate

- canonical packet:
  `proof-output/tlc-veiled-humanoid-p100k-v1`;
- source:
  `sample-3d/tlc-veiled-humanoid-h1-p100k-v1/source.glb`;
- source SHA-256:
  `941affd66d2afe803b2a63ba72c41c30592813d3f1deeae4600f24e0fc566b7d`;
- Meshy run:
  `1955bf8a-0362-4196-a713-68c15d59e518`;
- requested Meshy target: `100,000` triangles;
- safe source and written model triangles: `99,812`;
- binary MDL SkinMesh streams: `5`;
- source Meshy clips: `10`;
- emitted native Creature animation states: `42`;
- model resref: `tlcveil100_m1`;
- texture resref: `tlcveil100_t1`;
- HAK resref: `tlcv100hak1`;
- creature template resref: `tlcv100utc1`;
- Appearance physical row: `15100`;
- placement: `[10.0, 14.5, 0.0]`;
- player entry: `[10.0, 10.0, 0.0]`.

## Frozen artifact identity

| Artifact | Canonical source | Native destination | Bytes | SHA-256 |
|---|---|---|---:|---|
| MOD | `proof-output/tlc-veiled-humanoid-p100k-v1/generated/tlcv100demo1.mod` | `C:\Users\enonw\Documents\Neverwinter Nights\modules\tlcv100demo1.mod` | 15,174 | `91af75ead718d742045767c31617d560dad1bc3f05ae00644417149dea21215c` |
| HAK | `proof-output/tlc-veiled-humanoid-p100k-v1/generated/tlcv100hak1.hak` | `C:\Users\enonw\Documents\Neverwinter Nights\hak\tlcv100hak1.hak` | 29,667,213 | `6b8b74537fb6837cb6bb1981f1f417d931cebfb46b0d367965671cea0e39f8c3` |
| MDL | `proof-output/tlc-veiled-humanoid-p100k-v1/generated/tlcveil100_m1.mdl` | inside exact HAK | 10,182,680 | `876f6e92ec189b58ed9797333cde837edf3ca9dc57eba10fbf735d9c1f28bb70` |
| TGA | `proof-output/tlc-veiled-humanoid-p100k-v1/generated/tlcveil100_t1.tga` | inside exact HAK | 12,582,956 | `b581476d85906505845f113f3058eb42f5fb05a6dd6c35791bfbbe156b779f71` |
| appearance.2da | `proof-output/tlc-veiled-humanoid-p100k-v1/generated/appearance.2da` | inside exact HAK | 6,901,321 | `0a2aab57ebf621eff032a9d7eb119df1fbb037be435ada67d359cdb72eaebf74` |

Both native destinations were absent before installation. The exact MOD and
HAK were copied without overwrite and their destination hashes were verified
as byte-identical to the canonical packet.

## Offline verification

- module semantic readback: `PASS`;
- HAK resource inventory: `appearance.2da`, model and texture only;
- geometry: `99,812` source and written triangles;
- SkinMesh segmentation: `5` streams;
- animation namespace: `42 / 42`, complete;
- animation behavior candidate: eligible;
- transition continuity and locomotion kinematics: complete;
- animation-event pairs: `23 / 23`;
- SkinMesh deformation conformance: complete.

The host command exceeded its external four-minute invocation limit after the
packet files and reports had been written. Readback found no remaining build
process and the complete packet contains the materialization manifest, summary,
full semantic report, MOD, HAK, MDL, TGA and appended `appearance.2da`. No
second materialization was started and no artifact was overwritten.

## Owner test

Open exact `tlcv100demo1.mod`, then Area
`Meshy2Aurora M0 binary vertical-slice area`.

The Creature is deliberately placed directly in front of the player. Record
Toolset and NWN separately:

- `modelVisibility = visible | not_visible | not_tested`;
- `proofCompleteness = verified | failed | missing`.

No agent-run Toolset or NWN session was started. Final visual proof remains
owner-owned.
