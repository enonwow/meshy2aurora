# M4 evidence - binary MDL/MDX writer and semantic readback

Data startu: 2026-07-12
Attempt: `M4-20260712-01`
Status: `DONE_STRUCTURAL_M4`; active stage moved to `M4A`

Branch: `codex/m4-mdl-writer-readback`

Feature commit: `0f7b2f2 feat: add deterministic rigid MDL writer readback`

## Activation checkpoint

Na checkpointcie aktywacyjnym M4 bylo aktywne w bramce
`AURORA_FIRST_CONTRACT_LOCK`. Implementacja writera nie mogla rozpoczac sie
przed sprawdzeniem dokladnych anchorow dekompilacji Aurory, lokalnych struktur
own-readera i rozstrzygnieciem decyzji zapisanych ponizej.

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
  implementation_code: RIGID_AND_EXTENDED64_SKIN_DONE
```

## Activation snapshot (resolved by the contract-lock result below)

| id | decision needed before code | current boundary |
|---|---|---|
| M4-D01 | M4 structural model versus M4A animation ownership | `RESOLVED`: M4 structure/bind pose; M4A own clips/events. |
| M4-D02 | Emitted Profile-A skin format | `RESOLVED`: explicit extended64 structural emission and own semantic readback pass. |
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
| M4 contract supplement | PASS for rigid and extended64 structural skin slices; runtime deformation OPEN_M6 |
| Independent rigid implementation review | PASS; final rereview P1=0, P2=0 |
| Extended64 contract and implementation review | PASS; two final independent rereviews P1=0, P2=0 |
| Writer implementation | PASS_STRUCTURAL; rigid and extended64 skin emission complete |
| Deterministic byte proof | PASS_STRUCTURAL; skin deterministic tests pass and rigid remains len `1188`, core `1072`, raw `104`, SHA-256 `e100130d1dfbd18657413cdb7a701396d466cee081683591fc9836bf0c11b4b2` |
| Own-reader semantic readback | PASS_STRUCTURAL; rigid plus skin map/slot/q/t/constants/weights/refs semantic diff `0` |
| P-REF invariant conformance | PASS_STRUCTURAL; rigid and extended64 emission/readback pass, runtime remains OPEN_M6 |
| NWN EE runtime/visual proof | OPEN_M6 |

## Contract-lock result

Decisions M4-D01..M4-D03 sa zamkniete w
`documentation/m4-binary-writer-kontrakt-suplement-codex.md`:

- M4 emituje strukture i bind-pose controllers, ale zero wlasnych klipow;
- pierwszy jawny skin profile to extended64; tree-ordinal mapping, inverse-bind
  WXYZ/XYZ, exact header layout, emission i readback sa zamkniete;
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
  odrzucany; w rigid checkpoint skin IR byl jeszcze odrzucany, a obecny
  zamkniety kontrakt autoryzuje kolejna implementacje;
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
| extended64 node-to-bone/inverse-bind/weights | R1 skin witness | PASS_STRUCTURAL; emission/readback complete, runtime OPEN_M6 |

## Extended64 skin contract audit - 2026-07-12

Own `ErfArchive` plus `inspect_binary_mdl` probed `c_kocrachn` in-place from the
env-selected CEP HAK. Temporary probe source was removed; worktree returned
clean and no payload was copied.

Canonical results:

- three skin nodes, each `extended64`, header boundary `node+0x330`;
- map/q/t/constants counts `38/38/38/38`, equal reachable base tree nodes;
- forward map uses deterministic tree ordinal, not raw node number;
- active map pairs: `{7:0,8:1}`, `{11:0,12:1}`, `{4:0,13:1}`;
- first reverse entries: `[7,8]`, `[11,12]`, `[4,13]`;
- active vertex refs are dense local slots `{0,1}`;
- zero lanes use `weight=0/ref=0xffff`; no active `0xffff`;
- all `114` constants are `[0,0]`; weights metadata headers are `0/0/0`;
- inverse bind is `inverse(bindWorld(binaryNode_i))*bindWorld(skinMeshNode)`;
- raw inverse quaternion matches WXYZ exactly: max matrix error `0`, versus
  up to `2` for XYZW.

Decompilation anchors:

- `decompiled_all.c:864063-864077`: weights `16 B/vertex`, refs `8 B/vertex`;
- `:864079-864083`: forward map `count*2`;
- `:864085-864088`: quaternion arrays `count*16`;
- `:864090-864093`: translation arrays `count*12`;
- `:864095-864098`: constants `count*4`;
- `:890461-890475`: runtime reuse of refs/weights.

Independent supplement cross-check confirms `vertex ref -> local slot -> node`
from Borealis and Rollnw, four weight/ref lanes from three sources, and the
4/3/4-byte q/t/constants widths. The WXYZ/XYZW supplement conflict is resolved
for this profile by the local canonical reconstruction, zgodnie z Aurora First.

Contract review status: implementation may start. Runtime-only deviations are
recorded for unused inline `-1`, slot boundary `64`, constants meaning, WXYZ
deformation and final visible skin behavior; M6 owns those proofs.

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

M4 jest `DONE_STRUCTURAL`. Aktywny M4A zaczyna od Aurora First contract lock
dla animation headers, roots, tracks, transitions i events. Nie wolno
przenosic strukturalnych wnioskow M4 na animation emission przez zgadywanie.
Runtime/controller deformation pozostaje `OPEN_M6`.

## Evidence append policy

Po kazdym slice dopisywac bez nadpisywania historii:

- exact changed files i commit/branch;
- anchor audytu albo jawne odchylenie wraz z uzasadnieniem;
- test names, exact pass counts i deterministyczne SHA/length;
- semantic readback diff i P-REF invariant report;
- review findings oraz ich disposition;
- potwierdzenie, ze runtime/visual acceptance pozostaje `OPEN_M6`.

## Final extended64 skin implementation checkpoint - 2026-07-12

Status calego M4: `DONE_STRUCTURAL`. Active stage po tym checkpointcie: `M4A`.

Finalna implementacja emituje i odczytuje:

- extended64 `0x330` header plus forward map indexed by deterministic preorder;
- dense active slots, inline reverse map i exact `+0.0/0xffff` zero lanes;
- q/t/constants rows dla wszystkich reachable binary nodes;
- raw weights/refs z bitwise weight readback i semantic
  `slot -> tree ordinal -> IR node id`;
- mixed rigid/skin oraz wiele skin segmentow bez zmiany zamrozonego rigid path.

### Aurora First RAW controller boundary

Decompilation anchors uzyte do finalnej decyzji:

- `decompiled_all.c:875324-875339` - konstruktor stanu noda kopiuje cztery
  komponenty quaternionu bez transformacji;
- `:875429-875464` - local bez parenta jest kopiowany, a world composition
  uzywa przechowywanego quaternionu bez widocznej normalizacji;
- `:1020337-1020365` i `:1020438-1020485` - odpowiednio surowe quaternion
  multiply i quaternion-to-rotated-vector, bez dzielenia przez norme;
- helper normalizacji `:1020523-1020546` istnieje, ale nie jest wywolywany w
  zakotwiczonej downstream node composition. Polaczenie serialized controller
  type `20` z zapisem pola quaternionu noda pozostaje nierozstrzygniete.

M4 downstream `bindWorld` komponuje dokladnie wyemitowane `f32 qx,qy,qz,qw`
bez ponownej normalizacji. Niezalezny test odbudowuje swiaty tylko z
`artifact.inspection` controllers, bez production/expected helpers. Gleboka
adversarialna kompozycja RAW, ktora przestaje byc proper rigid, jest stabilnie
odrzucana `M4-SKIN-INVERSE-BIND-UNSUPPORTED`. Czy quaternion zostal
znormalizowany przed controller type-20 write pozostaje nierozstrzygniete i
`OPEN_M6`; M4 nie twierdzi runtime deformation parity.

### Final findings disposition

Wszystkie cztery implementation-review P2 sa `FIXED`:

1. inactive `-0.0` jest odrzucane przed emisja, a payload mutation jest
   wykrywana przez bitwise semantic weight readback;
2. circular inverse-bind proof zastapiono inspection-only RAW controller
   reconstruction oraz adversarial deep stable-reject testem;
3. signed `i32` map-count/forward-offset conversions sa sprawdzane przed
   alokacja i maja test stabilnego `M4-LAYOUT-OVERFLOW`;
4. wszystkie runtime-only skin assumptions maja jawne artifact deviations:
   unused inline, slot boundary, constants meaning, WXYZ deformation i visual
   deformation.

Dwa finalne niezalezne rereview po disposition: `P1=0`, `P2=0`.

### Final exact gates

| gate | exact result |
|---|---|
| core tests | `145 PASS` |
| workspace tests | `147/147 PASS` (`145 core + 2 WASM native`) |
| M4 writer tests | `16/16 PASS` |
| Node/WASM | `14/14 PASS` |
| canonical CEP P-REF | `1/1 PASS`; 6 packets |
| Docker no-cache | PASS, `130.9 s`; tag `m2a-quality:m4-skin-final` |
| independent final rereviews | two PASS; `P1=0`, `P2=0` |

```yaml
docker:
  tag: m2a-quality:m4-skin-final
  digest: sha256:959664f411828099eb3ec7618c9f5cbcc511f7b4206c41d535f98f7e6da815a5
  size_bytes: 1195004606
  no_cache_seconds: 130.9
rigid_frozen_proof:
  payload_length: 1188
  core_length: 1072
  raw_length: 104
  sha256: e100130d1dfbd18657413cdb7a701396d466cee081683591fc9836bf0c11b4b2
  unchanged: true
```

Clean-room pozostaje zachowany: zero retail/CEP payloadu w repo, canonical HAK
wylacznie jako env-gated read-only proof. Runtime i visual acceptance sa
`OPEN_M6`; M4A ma osobny Aurora First contract-lock przed implementacja.
