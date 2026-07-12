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
