# M4 evidence - binary MDL/MDX writer and semantic readback

Data startu: 2026-07-12
Attempt: `M4-20260712-01`
Status: `RIGID_SLICE_DONE_M4_IN_PROGRESS_SKIN`

Branch: `codex/m4-mdl-writer-readback`

Feature commit: `0f7b2f2 feat: add deterministic rigid MDL writer readback`

## Activation checkpoint

M4 jest aktywne w bramce `AURORA_FIRST_CONTRACT_LOCK`. Implementacja writera
nie moze rozpoczac sie przed sprawdzeniem dokladnych anchorow dekompilacji
Aurory, lokalnych struktur own-readera i rozstrzygnieciem decyzji zapisanych
ponizej.

```yaml
activation:
  input: AuroraCreatureIrV1
  source_glb_retraversal: FORBIDDEN
  authority_order:
    - "Aurora decompilation"
    - "local read-only canonical MDL observations"
    - "external read-only cross-checks"
    - "own reader and self-generated fixtures"
    - "NWN EE Toolset/game final acceptance"
  implementation_code: RIGID_SLICE_DONE
```

## Activation snapshot (resolved by the contract-lock result below)

| id | decision needed before code | current boundary |
|---|---|---|
| M4-D01 | M4 structural model versus M4A animation ownership | `RESOLVED`: M4 structure/bind pose; M4A own clips/events. |
| M4-D02 | Emitted Profile-A skin format | `RESOLVED_DIRECTION`: explicit extended64; emission waits for named skin mapping/inverse-bind gate. |
| M4-D03 | M4 versus M6 acceptance ownership | `RESOLVED`: M4 own readback; M6 Toolset/game runtime and visual acceptance. |

## Required contract-lock evidence

- exact authority anchors: decompilation for type/node-family dispatch and
  local binary/P-REF for file/core/raw pointer origins and serialized layout;
- explicit writer format profile, including skin header variant;
- exact structural payload owned by M4 and animation payload deferred to M4A;
- deterministic layout/planning and checked arithmetic policy;
- own-reader semantic comparison fields and stable diagnostics;
- explicit `OPEN_M6` runtime/visual boundary;
- clean-room statement: no external writer code or reference payload copied.

## Gate status

| gate | current result |
|---|---|
| Aurora First anchor audit | PASS; decompiled type/node-family anchors plus M1B binary/P-REF layout evidence |
| M4 contract supplement | PASS for rigid slice; skin mapping/inverse-bind gate remains named OPEN |
| Independent contract review | PASS; final rereview P1=0, P2=0 |
| Writer implementation | PASS_RIGID; skin emission remains OPEN |
| Deterministic byte proof | PASS_RIGID; len `1188`, core `1072`, raw `104`, SHA-256 `e100130d1dfbd18657413cdb7a701396d466cee081683591fc9836bf0c11b4b2` |
| Own-reader semantic readback | PASS_RIGID; semantic diff `0`, exact EOF, core faces and raw index path equal |
| P-REF invariant conformance | PASS_RIGID with explicit OPEN_M6 deviations; extended64 skin invariants remain OPEN |
| NWN EE runtime/visual proof | OPEN_M6 |

## Contract-lock result

Decisions M4-D01..M4-D03 sa zamkniete w
`documentation/m4-binary-writer-kontrakt-suplement-codex.md`:

- M4 emituje strukture i bind-pose controllers, ale zero wlasnych klipow;
- pierwszy jawny skin profile to extended64, lecz emission czeka na osobny
  mapping/inverse-bind gate;
- M4 structural closure opiera sie na own readbacku, a M6 posiada runtime i
  visual acceptance.

Audyt wykazal, ze `decompiled_all.c` potwierdza type 2002 i runtime node-family
dispatch, lecz nie zawiera nazwanego serialized writera. Offsety/core/raw i
skin boundary pochodza z own-reader canonical binary/P-REF evidence, zgodnie z
Aurora First fallback order.

## Rigid implementation result

Publiczne core API:

- `write_binary_mdl(&AuroraCreatureIrV1, &MdlWriterOptionsV1)`;
- jawny `M4_DIRECT_CREATURE_EXTENDED64_V1`; request legacy17 jest stabilnie
  odrzucany, a skin IR czeka na osobny gate;
- deterministic two-region planner przed alokacja, checked conversions i
  fallible exact-size allocation;
- model/dummy/trimesh hierarchy, bind position/orientation controllers,
  face planes, bounds, texture resref, core faces, raw positions/UV0/normals i
  raw `u16` index path;
- writer uruchamia own reader i nie zwraca artifactu przy semantic diff.

Own reader zostal rozszerzony o exact EOF, model geometry type, mesh
bounds/material defaults i raw index count/offset/list inventory. Non-finite
average w canonical empty mesh jest zachowane jako `None`, aby P-REF pozostawal
deterministyczny i porownywalny.

Frozen syntetyczny proof `m2a_test`:

```yaml
payload_length: 1188
core_length: 1072
raw_length: 104
sha256: e100130d1dfbd18657413cdb7a701396d466cee081683591fc9836bf0c11b4b2
semantic_diff_count: 0
own_animation_count: 0
unexpected_diagnostics: 0
```

## P-REF invariant conformance - rigid slice

| invariant | local authority | emitted result |
|---|---|---|
| one type-2002 payload is `12 + core + appended MDX` | R1/M1B | PASS; exact EOF |
| core pointers are relative to byte 12; raw pointers to appended MDX | M1B | PASS; reader range validation |
| model geometry type `2` and dummy/trimesh flags `0x01/0x21` | R1 canonical observation | PASS |
| bind controllers use types `8/20` and frozen row/time/data/column layout | R1/R4 canonical observation | PASS, including exact quaternion sign tests |
| faces and raw index path describe identical triangles | R1 direct mesh observation | PASS |
| zero own model animations in structural direct creature | R1 `c_kocrachn` | PASS |
| runtime routines, face adjacency/surface defaults and tail fields | not proven by structural readback | explicit artifact deviations; `OPEN_M6` |
| extended64 node-to-bone/inverse-bind/weights | R1 skin witness | OPEN next M4 slice; no false rigid claim |

Read-only local canonical regression po rozszerzeniu readera:
`M2A_REFERENCE_CEP_HAK` -> 6 wymaganych P-REF packets, `1/1 PASS`, payloady
czytane in-place i niekopiowane do repo.

## Verification

| gate | exact result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --locked --workspace` | `137/137 PASS` (core `135`, WASM native `2`) |
| `cargo build --locked -p m2a-wasm --target wasm32-unknown-unknown` | PASS |
| `wasm-pack test --node crates/m2a-wasm` | `14/14 PASS` |
| canonical CEP P-REF integration | `1/1 PASS`; 6 packets |
| independent implementation rereview | `NO FINDINGS`; P1=`0`, P2=`0` |
| Docker no-cache quality target | PASS, `124.5 s` |

Docker image `m2a-quality:m4-rigid`:

```yaml
digest: sha256:739def9dde4f2c3b8f06903aa3a198b04d839d4fdd2522f64b3edb9cf1c76459
size_bytes: 1179330207
retail_or_reference_assets_in_context: false
```

Clean-room: implementacja nie kopiuje zewnetrznego writera ani payloadu
retail/CEP. Canonical HAK jest tylko env-gated read-only proofem.

## Next action

Zamknac extended64 skin mapping/inverse-bind contract z Aurora First/P-REF,
nastepnie zaimplementowac skin emission i semantic bone readback. Runtime i
visual acceptance pozostaja `OPEN_M6`.

## Evidence append policy

Po kazdym slice dopisywac bez nadpisywania historii:

- exact changed files i commit/branch;
- anchor audytu albo jawne odchylenie wraz z uzasadnieniem;
- test names, exact pass counts i deterministyczne SHA/length;
- semantic readback diff i P-REF invariant report;
- review findings oraz ich disposition;
- potwierdzenie, ze runtime/visual acceptance pozostaje `OPEN_M6`.
