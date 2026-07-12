# M3 evidence - Creature Profile A conversion

Data startu: 2026-07-12
Attempt: `M3-20260712-01`
Status: `IN_PROGRESS`

## Contract lock

```yaml
contract:
  file: documentation/m3-profile-a-conversion-kontrakt-suplement-codex.md
  state: PROFILE_A_LOCKED_M3
  basis: "P(x,y,z)=(x,z,y)"
  determinant: -1
  winding: REVERSE_ONCE
  uv: "[u,v] -> [u,1-v]"
  scale: TARGET_BBOX_HEIGHT
  alignment: BOTTOM_CENTER_TO_PROFILE_ANCHOR
  materialGuardrail: 1
  engineFacingProof: OPEN_M6
  uvRuntimeProof: OPEN_M6
```

Niezalezny read-only audit M3 rozstrzygnal konflikt provenance i kontrakt
implementacyjny. Zewnetrzne skeleton/weights/animations sa zabronione w
eksporcie. Dozwolone rig provenance to `SYNTHETIC`, `OWNED` albo
`USER_PROVIDED`, zawsze z `exportAllowed=true` i zgodnym content hash.

P1/P2 contract review zostal rozstrzygniety dokumentacyjnie przed kodem:

- default scene only, reachable mesh instances i per-instance duplication;
- source-local -> source-world bake raz, global transform, potem rig-parent local;
- output hierarchy dokladnie z rig profile, bez source hierarchy;
- pelny Options/Report schema i material source binding z legalnym null;
- valid forbidden provenance jako blocking outcome, malformed/unknown JSON fatal;
- canonical typed hash helper oraz exact distance-evaluation accounting;
- target envelope/anchor/surface validation i pelny normal/tangent space chain;
- M4 input zablokowany na `AuroraCreatureIrV1`.
- public provenance YAML/Rust schema ma wszystkie piec kind i trzy identyczne
  attestations; report zachowuje je wraz z `allAttestationsSatisfied`;
- source IR/report identity i exact M2 coordinate contract sa fatalnie
  walidowane przed gates bez twierdzenia o pelnej kryptograficznej binding;
- severity domains, deterministic ordering i custom limit invariants sa exact.

## Canonical observations (read-only)

| packet | exact observation | status |
|---|---|---|
| R1 `c_kocrachn` | declared `66`, reachable `38`; 3 extended64 skins; map counts `38,38,38` | observation only |
| current own-reader `c_kocrachn` | 31 mesh nodes, 2009 vertices, 1458 triangles; 3 `extended64` skin nodes / 303 weighted vertices / max 2 influences | canonical read-only metrics only; not a fixture, oracle, or rig source |
| R5 `c_vampire_f` | reachable `28`; 2 legacy17 skins; map/q/t/constants `28,28` | observation only |

Zaden payload, skeleton, skin, keyframe ani host path z tych witnessow nie jest
czescia fixture lub outputu M3.

## Required proof matrix

| gate | required evidence | current result |
|---|---|---|
| basis/parity/winding | axis + asymmetric face fixtures | M3A PASS - basis/parity/reflection/winding covered |
| normals/tangents | inverse-transpose, tangent parity | M3A PASS |
| mixed tangent coverage | exact blocking `M3A-TANGENT-COVERAGE-MIXED`, no output | M3A PASS |
| UV | four-corner + double-flip | M3A PARTIAL - exact single flip and source immutability covered; expanded fixture remains before M3 DONE |
| scale/alignment | 1x/2x/zero-height/bottom-center | M3A PASS |
| material | 1 PASS, 2 BLOCKING | M3A PASS - blocked reports preserve exact count/bindings/diagnostics |
| segment assignment | distance sum + stable tie | PENDING_M3B |
| weights | barycentric, merge, stable top4, normalize | PENDING_M3B |
| provenance | allowed/forbidden/hash/exportAllowed | M3A PASS |
| limits/no-panic | exact/limit+1/overflow/truncation | M3A PASS for RIGID slice |
| traversal/amplification safety | iterative deep hierarchy; instances x primitives exact/limit+1 before allocation | M3A PASS - 4096 rig chain and 2048 source chain |
| creature hash finite domain | NaN/Inf returns `M3A-NONFINITE-FLOAT`, never JSON null/hash | M3A PASS |
| deterministic WASM | native/WASM same JSON | PENDING_M3 WASM adapter/parity |
| default scene/instances | unreachable ignored, shared mesh duplicated, bake once | M3A PASS |
| report/material schema | null/one/two source bindings, stable slots | M3A PASS |
| exact work accounting | increment before each point-triangle evaluation | PENDING_M3B distance transfer; M3A WorkBudget PASS |
| source immutability | bytes/IR/report unchanged | M3A PASS |
| M2 blocked-source correlation | exact code/path/severity pairs return `M3A-SOURCE-BLOCKED`; missing/wrong pair is fatal mismatch | M3A PASS |
| clean-room scan | no external asset/path | M3A PASS - synthetic fixtures only |
| final review | independent NO FINDINGS | M3A PASS P1=0/P2=0; full M3 review pending M3B/WASM |

## Gate commands

```yaml
commands:
  cargo_fmt: PASS
  cargo_clippy: "PASS --workspace --all-targets -- -D warnings"
  cargo_test_workspace: "PASS 118/118; profile_a 22/22"
  wasm_build: "PASS wasm32-unknown-unknown"
  wasm_pack_node: "PASS 12/12"
  git_diff_check: PASS
```

M3A RIGID single-segment slice ma implementacyjny PASS i niezalezny finalny
review `NO FINDINGS` (`P1=0`, `P2=0`). M3 pozostaje `IN_PROGRESS`: M3B
segmentacja/SKIN/nearest-surface/top-four weights oraz publiczny adapter/parity
WASM sa wymagane przed M3 DONE. `OPEN_M6` pozostaje bez zmian.

## M3A checkpoint 2026-07-12

- core: `crates/m2a-core/src/profile_a.rs`;
- tests: `crates/m2a-core/tests/profile_a.rs`;
- public export: `crates/m2a-core/src/lib.rs`;
- workspace tests: `118/118`, w tym `22/22` M3A;
- Node/WASM regression suite: `12/12`;
- final independent review: `NO FINDINGS`, `P1=0`, `P2=0`;
- external/CEP/retail payloads copied: `0`;
- M3B status: `PENDING`.

Docker build/test checkpoint (quality gate only, never Toolset/runtime proof):

- command: `docker build --no-cache -t m2a-quality:m3a .`;
- result: `PASS`, 162.5 s, build context 159.52 kB;
- image digest: `sha256:b346849fa471cbe460543295dc0afd8ebec76e3378a13de32b83126c23cd32c1`;
- image size: `1125042309` bytes;
- host/image `crates/m2a-core/src/profile_a.rs` SHA-256:
  `2eaad7293b66c608691a847310f173fa1ee60e397774cfec9a647f6a30ffa5be`;
- SHA match: `true`;
- image build ran fmt, clippy, 118 workspace tests, wasm32 build and 12 Node/WASM tests;
- Docker history scan for private host paths/reference names/secret labels: no matches.

## Next evidence append

Po kazdym slice dopisac bez nadpisywania historii:

- commit/branch i exact changed files;
- test names i exact pass counts;
- stable fatal/gate codes pokryte testami;
- deterministic output hash native/WASM;
- limity i provenance fixture;
- findings review oraz ich disposition.

## M3B checkpoint 2026-07-12

Status slice: `PASS`, independent review: `NO FINDINGS` (`P1=0`, `P2=0`).
Status calego M3: `IN_PROGRESS`.

M3B zaimplementowal i zweryfikowal:

- exhaustive triangle-to-segment assignment w target world, exact tie po
  rosnacym `segmentId` i nearest reference triangle tie po najnizszym ordinal;
- buckety `(segmentId, materialSlot)`, reuse vertexa w bucket oraz jawna
  duplikacje na granicy bucketow;
- nearest-surface barycentric `SKIN` transfer, merge duplicate bones, usuniecie
  merged values `<=0`, sort `(weight DESC, boneId ASC)`, top-four i normalizacje;
- `RIGID` bez transferu i bez weight lanes;
- exact `distanceEvaluations` z checked increment przed kazda evaluation oraz
  osobne assignment/transfer accounting;
- cumulative WorkBudget i fallible reserves dla assignment, bucket/output,
  mapping i weight scratch;
- stable blocking dla unreferenced source vertex bez silent drop;
- zachowanie provisional segment/weight/work report po pozniejszym blocking gate;
- target-world surface degeneracy/non-finite guard oraz duze finite weight sums
  w `f64` przed bezpieczna normalizacja do `f32`.

Zamkniete definicje raportu:

- `duplicatedVertexCount` liczy tylko cross-bucket copies ponad pierwsza emisje
  tego samego `(meshInstance, primitive, sourceVertex)`; instancing jest poza tym
  licznikiem;
- duplicate/zero/top-four/max-before/max-after counters maja exact raw/merged
  semantyke zapisana w aktywnym suplemencie;
- external profile reference bone spoza `allowedBoneNodeIds` jest malformed
  profile i musi skonczyc sie fatal `M3A-PROFILE-SEGMENT-INVALID`;
  `M3A-WEIGHT-BONE-FORBIDDEN` pozostaje tylko defensive post-validation/runtime
  invariant gate. Kod/test tej ostatniej granicy musi byc wyrownany przed
  finalnym M3 review; dokument normatywny jest rozstrzygajacy.

Exact gates uruchomione na checkpointcie:

```yaml
commands:
  cargo_fmt: PASS
  cargo_clippy: "PASS --workspace --all-targets -- -D warnings"
  cargo_test_workspace: "PASS 125/125; profile_a 29/29"
  wasm_build: "PASS wasm32-unknown-unknown"
  wasm_pack_node: "PASS 12/12"
  git_diff_check: PASS
review:
  result: NO_FINDINGS
  p1: 0
  p2: 0
```

Test matrix M3B pokrywa exact counts `6`, `12` i `15`, limit boundary/limit+1,
segment tie, barycentric transfer, duplicate/top-four, duze finite weights,
blocking weight outcomes, mixed `RIGID/SKIN`, cross-bucket `4 -> 6` vertexow z
`duplicatedVertexCount=2`, unreferenced vertex oraz degeneracje po world-transform
rounding. Source input pozostaje niezmieniony; external payloads skopiowane: `0`.

M3 nie przechodzi do `DONE`. Brakuje publicznego Profile A JSON/WASM adaptera i
realnego native/WASM byte-identical deterministic outcome proof z sekcji 11.
Obecne `wasm-pack 12/12` potwierdza brak regresji istniejacych adapterow, ale nie
jest tym brakujacym Profile A parity proof. M4 pozostaje `NOT_STARTED`.

## Final M3 checkpoint 2026-07-12

Ten wpis jest rozstrzygajacym stanem po M3A, M3B, wyrównaniu profile-fatal
boundary, publicznym adapterze i finalnym whole-M3 review. Wczesniejsze wpisy
`IN_PROGRESS`, `PENDING_M3B`, `125/125` i `12/12` zachowuja stan historyczny z
chwili checkpointu i nie opisuja juz stanu koncowego.

Status M3: `DONE`.

Final artifacts:

- core conversion: `crates/m2a-core/src/profile_a.rs`;
- core regression matrix: `crates/m2a-core/tests/profile_a.rs`;
- public native/WASM adapter and frozen byte proofs:
  `crates/m2a-wasm/src/lib.rs`;
- adapter dependencies: `crates/m2a-wasm/Cargo.toml`, `Cargo.lock`;
- locked contract:
  `documentation/m3-profile-a-conversion-kontrakt-suplement-codex.md`;
- append-only evidence: `documentation/evidence/M3-evidence.md`;
- live stage state: `documentation/orchestrator-state.yaml`.

Public API:

```text
Rust canonical: convert_profile_a_glb_json(bytes, rig_json, options_json)
JS canonical:   convertProfileAGlbJson(bytes, rigJson, optionsJson)
Rust alias:     convert_profile_a_json
JS alias:       convertProfileAJson
alias result:   byte-identical
```

Adapter wykonuje strict deserialize `CreatureRigProfileV1` i
`ProfileAOptionsV1` z nested `deny_unknown_fields`, nastepnie deleguje dokladnie
do `ingest_glb` i `convert_profile_a`. Malformed/unknown rig JSON, unknown options,
truncated GLB, core limit fatal i blocking provenance maja deterministic envelope
zgodny z core. Controlled fixtures maja provenance `SYNTHETIC`,
`exportAllowed=true`, komplet attestations i nie zawieraja external payloadu,
host path ani reference rig danych.

Frozen native/WASM exact-byte proofs:

| case | JSON length | SHA-256 |
|---|---:|---|
| RIGID success | 3186 | `bb1a7a8564be2938bc694b1ffb928e11b904aa62b61b2687bb8a0013bc6c10a1` |
| multi-segment SKIN success | 3562 | `e474aa01d1108e2278e6cfaf6c9d2b71b71e4e393b55dbc729b7c8c4c6a8d9dd` |
| malformed rig JSON fatal | 151 | `03bd6ebd5cdb45f738de87363d3ad6a95de9bbb5aa119a2fce3199255b8efa55` |
| distance-limit core fatal | 162 | `3bfb45cf36af0d4af174cea656ab669714a55c4c88a9661c7ea573be75bec4a2` |

Native i real Node/WASM assertuja te same length i pelne SHA-256. Work accounting
uzywa fixed conservative 64-bit contract charges, dlatego `workBytesPeak` i caly
success JSON sa identyczne na native64 i wasm32.

Final gates:

```yaml
commands:
  cargo_fmt: PASS
  cargo_clippy: "PASS --workspace --all-targets -- -D warnings"
  cargo_test_workspace: "PASS 127/127"
  profile_a_core: "PASS 29/29"
  profile_a_native_adapter: "PASS 2/2"
  wasm_build: "PASS wasm32-unknown-unknown"
  wasm_pack_node: "PASS 14/14"
  git_diff_check: PASS
review:
  scope: WHOLE_M3
  result: NO_FINDINGS
  p1: 0
  p2: 0
```

Final Docker build/test checkpoint (quality gate only, never Toolset/runtime proof):

- command: `docker build --no-cache -t m2a-quality:m3 .`;
- result: `PASS`, 116.6 s, build context 274.21 kB;
- image digest: `sha256:f3a68aac52cb59a0b5be0199b6b2112e9f1737c4f2047831eecccbf5ee360fe1`;
- image size: `1151988127` bytes;
- container gates: workspace `127/127`, Profile A `29/29`, native adapter
  `2/2`, Node/WASM `14/14`, fmt, clippy and wasm32 build `PASS`;
- host/image SHA-256 identity: `Cargo.lock`, core `profile_a.rs`, core
  `tests/profile_a.rs` and WASM `src/lib.rs` all `MATCH=true`;
- Docker history scan for private host paths/reference names/secret labels:
  `NO_MATCHES`.

External profile reference bone spoza `allowedBoneNodeIds` jest zgodnie z
kontraktem fatal `M3A-PROFILE-SEGMENT-INVALID`; defensive
`M3A-WEIGHT-BONE-FORBIDDEN` nie jest publiczna alternatywa walidacji. Wszystkie
M3A/M3B assignment, SKIN/RIGID, report, budget, strict JSON i parity gates sa
zamkniete. `OPEN_M6` dla engine-facing i wizualnego UV proof pozostaje
intencjonalnie przypisane do M6 i nie blokuje M3 DONE. M4 jest nastepnym etapem
gotowym do osobnego checkpointu aktywacyjnego; implementacja M4 nie rozpoczela sie
w tym wpisie.
