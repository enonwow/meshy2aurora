# TLC Veiled Humanoid P100K — Studio pipeline replay

> **POLICY SUPERSEDED 2026-07-29:** byte-identical replay i jego hashe pozostają
> ważne. Bieżący domyślny profil to `PRODUCT_300K`; opis `PRODUCT_20K` niżej
> dokumentuje stan aplikacji w chwili tego historycznego replay.

Date: 2026-07-28
Status: `PASS / APP_PIPELINE_BYTE_IDENTICAL`

## Correction

The first frozen P100K packet was materialized by the project example CLI
calling `m2a-core`. It used the canonical format writers, but it did **not**
pass through the Studio application boundary. Therefore it was not sufficient
evidence for the owner's requirement that the conversion be performed by the
application.

The missing application route has now been implemented:

`Studio UI -> Web Worker -> m2a-wasm -> m2a-core -> MDL/TGA/2DA/HAK/MOD writers`

There is no JavaScript format writer and no ad-hoc packet writer on this route.
The project's own writers live in `m2a-core`; Studio invokes them through the
Worker/WASM boundary. The earlier CLI is only another adapter over the same
core and is no longer used as evidence that Studio itself performed a build.

## Explicit experiment profile

The ordinary product profile remains unchanged:

- profile: `PRODUCT_20K`;
- shared Creature/Placeable product budget: 20,000 triangles;
- the exact 102,335-raw-triangle source reports
  `conversionEligible = false`.

The elevated route is available only after explicitly selecting:

- profile: `EXPERIMENTAL_P100K`;
- admitted raw Meshy envelope: up to 110,000 faces so degenerate faces can be
  removed;
- required post-sanitize range: 20,001–100,000 triangles;
- the exact source reports `conversionEligible = true` at inspection and
  materializes 99,812 safe triangles;
- binary-MDL segmentation remains subject to the independent per-stream index
  boundary.

This profile does not change `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 20_000`.

## Exact replay identity

- source:
  `sample-3d/tlc-veiled-humanoid-h1-p100k-v1/source.glb`;
- source SHA-256:
  `941affd66d2afe803b2a63ba72c41c30592813d3f1deeae4600f24e0fc566b7d`;
- input `appearance.2da` SHA-256:
  `815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a`;
- model: `tlcveil100_m1`;
- texture: `tlcveil100_t1`;
- HAK: `tlcv100hak1`;
- MOD: `tlcv100demo1`;
- Area: `tlcv100area1`;
- UTC: `tlcv100utc1`.

The replay ran in memory. It did not create another candidate, allocate new
resrefs, rewrite `proof-output`, or touch the already installed native
MOD/HAK.

## Byte-identical result

The native Studio/WASM boundary test compared all generated resource bytes
directly with the frozen packet:

| Resource | Bytes | Frozen and replay SHA-256 | Result |
|---|---:|---|---|
| MOD | 15,174 | `91af75ead718d742045767c31617d560dad1bc3f05ae00644417149dea21215c` | exact |
| HAK | 29,667,213 | `6b8b74537fb6837cb6bb1981f1f417d931cebfb46b0d367965671cea0e39f8c3` | exact |
| MDL | 10,182,680 | `876f6e92ec189b58ed9797333cde837edf3ca9dc57eba10fbf735d9c1f28bb70` | exact |
| TGA | 12,582,956 | `b581476d85906505845f113f3058eb42f5fb05a6dd6c35791bfbbe156b779f71` | exact |
| appended `appearance.2da` | 6,901,321 | `0a2aab57ebf621eff032a9d7eb119df1fbb037be435ada67d359cdb72eaebf74` | exact |

The real browser Worker/WASM integration independently reproduced the exact
MOD, HAK and MDL hashes. The byte-identical HAK binds the exact TGA and appended
2DA as packaged runtime resources. The application result projector accepted
the package and reconciled:

- source triangles: 99,812;
- converted triangles: 99,812;
- model resref: `tlcveil100_m1`;
- texture resref: `tlcveil100_t1`;
- status: `M6_MODEL_PACKAGE_MATERIALIZED`.

## Verification ledger

- UI routing test: explicit profile selects the P100K inspection and package
  lane with deterministic caller-owned model/texture/HAK/MOD/Area/UTC identity;
- exact Studio/WASM replay:
  `1 passed`, 250.63 s, all five payloads compared byte for byte;
- explicit inspection gate test:
  default 20K `false`, P100K experiment `true`, raw count 102,335;
- real browser Worker/WASM replay:
  `1 passed`, exact MOD/HAK/MDL hashes and successful canonical result
  projection;
- Studio unit suite: `202 passed`;
- `m2a-wasm`: `33 passed`;
- core P100K/model-pipeline tests: `4 passed`;
- production Studio build: passed;
- Rust Clippy for `m2a-core` and `m2a-wasm`, all targets, warnings denied:
  passed;
- canonical workspace and Meshy asset-layout guards: passed.

## Remaining proof boundary

This correction proves the application conversion route and deterministic
artifact identity. It does not replace visual proof in Aurora Toolset or NWN.
The installed frozen candidate remains unchanged and
`ready_for_owner_proof`; final visual verification remains owner-owned.
