# M1B Evidence

Ten plik jest append-only dziennikiem dowodow dla deep binary MDL reference readera. Biezacy stan pozostaje w `documentation/orchestrator-state.yaml`.

## M1B-20260711-01 - 2026-07-11

Status: IN_PROGRESS
Owner: Codex orchestrator + M1B implementation subagents
Stage: M1B

### Cel proby

Rozszerzyc wlasny reader o potwierdzone sekcje mesh, skin, controllerow i animacji oraz utworzyc env-gated wielomodelowy przeplyw P-REF bez kopiowania zewnetrznych payloadow.

### Aurora First / provenance

- `C:\Projects\New Folder\export\decompiled_all.c` - runtime families `CResMDL` oraz `MdlNode*`; brak nazwanej struktury writera.
- `documentation/mdl-binary-crosswalk-codex.md` - aktywny layout i jawny konflikt skin 17/64.
- `documentation/korpus-referencyjny-mdl-codex.md` - polityka R0-R6 i P-REF.
- Syntetyczne fixture pozostaja jedynymi commitowanymi payloadami testowymi.

### Zmienione pliki

- `documentation/orchestrator-state.yaml` - aktywowano M1B.
- `documentation/evidence/M1B-evidence.md` - utworzono dziennik etapu.

### Weryfikacja

| Command or action | Expected | Actual | Status |
|---|---|---|---|
| `git status --short --branch` | czysty checkpoint M1A | czysta galaz bazowa przed startem | PASS |
| `git switch -c codex/m1b-deep-mdl-reader` | osobna galaz M1B | utworzono | PASS |

### Problemy i bledy

```yaml
current_problems:
  - "Dokladny supported/deferred layout M1B jest w trakcie audytu."
  - "GB-001-SKIN 17/64 wymaga lokalnego boundary evidence."
bugs: []
```

### Evidence artifacts

- `documentation/orchestrator-state.yaml` - aktywny stage i attempt id.

### Nastepny krok

Zamknac suplement kontraktu M1B z dokladnymi polami, testami i stop conditions, potem implementowac w TDD.

## M1B-20260711-01 - real-binary correction

Status: IN_PROGRESS
Owner: Codex orchestrator
Stage: M1B

### Cel proby

Skorygowac M1A tam, gdzie pierwszy realny read-only binary ujawnil rozjazd miedzy syntetycznym zalozeniem a runtime headerem.

### Aurora First / provenance

- Realny read-only `c_kocrachn` smoke: root runtime `parent_ptr` mial wartosc `7733349`, nie serialized parent offset.
- `C:\Projects\Claude\rollnw\lib\nw\model\MdlBinaryParser.hpp:118-122` klasyfikuje `geometry_ptr` i `parent_ptr` jako ignore, a relacje drzewa wynikaja z children array.

### Zmienione pliki

- `crates/m2a-core/src/mdl/parse_binary_mdl.rs` - parent relationship jest wyprowadzana z traversalu; runtime fields nie sa file pointerami.
- `crates/m2a-core/tests/mdl/parser.rs` - regresja dla niezerowych runtime values.
- `documentation/prototyp-parsera-m1a-claude.md` - poprawiona klasyfikacja provenance.

### Weryfikacja

| Command or action | Expected | Actual | Status |
|---|---|---|---|
| real read-only smoke before fix | parser nie interpretuje runtime parent jako core pointer | `M2A-MDL-HEADER-INVALID` offset 312 ujawnil bledne zalozenie | FAIL |
| synthetic runtime-field regression | arbitrary runtime values nie zmieniaja relacji tree | test dodany; final gate pending | VERIFYING |

### Problemy i bledy

```yaml
current_problems:
  - "Ponowic canonical real-model smoke po integracji deep readera i M1C locatora."
bugs:
  - id: "M1B-BUG-001"
    severity: "P1"
    status: "FIXED"
    repro: "root node zawiera niezerowy runtime parent_ptr"
    expected: "parent relationship derived from children traversal"
    actual: "M1A porownywal runtime value z serialized parent offset"
    next_action: "run full gates and canonical P-REF"
```

### Evidence artifacts

- `crates/m2a-core/tests/mdl/parser.rs` - runtime-field regression.

### Nastepny krok

Uruchomic native/WASM regression, potem kontynuowac deep-section parser.

## M1B-20260711-01 - contract, corpus inventory and P-REF review

Status: IN_PROGRESS
Owner: Codex orchestrator + M1B implementation/review subagents
Stage: M1B

### Cel proby

Zamknac testowalny kontrakt deep readera i pure-bytes P-REF przed canonical runem, bez kopiowania payloadow i bez promowania container inventory do statusu own-reader proof.

### Canonical corpus identity - container-level observation

Ponizsze wartosci sa obserwacja read-only bajtow zasobu na poziomie kontenera. Nie sa jeszcze canonical P-REF ani wynikiem locatora M1C. Identity nie zawiera host paths.

| ID | Source class | Container identity | Resref | Type | SHA-256 observed | Status |
|---|---|---|---|---:|---|---|
| R1 | `named_hak` | `cep3_core1` | `c_kocrachn` | 2002 | `f16426310f826ae2ab15034ac979c65f812ee8bda0d13ee459bf2b293d7db270` | `PENDING_CANONICAL_M1C_P-REF` |
| R2 | `base_nwn` | `models_01.bif` | `c_horror` | 2002 | `2faf553a0665da200b232bd52d03c0e1d79b88959cabdbe840f35f16e5878c8e` | `PENDING_CANONICAL_M1C_P-REF`; adapter KEY/BIF optional |
| R3a | `named_hak` | `cep3_core1` | `c_phod_horror_b` | 2002 | `62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a` | `PENDING_CANONICAL_M1C_P-REF` |
| R3b | `named_hak` | `cep3_core1` | `c_phod_horror_p` | 2002 | `09e43ee9493d2fe2bbf9cbeb44f24dcb999e5f38e651bdc79eefdd5e1f19722f` | `PENDING_CANONICAL_M1C_P-REF` |

### Test contract review

Status: IN_PROGRESS

Required happy fixtures: metadata/ranges, all common controllers, trimesh, `legacy17`, `extended64`, multiple animation roots/events, combined deep fixture i native/WASM deterministic JSON.

Required negative matrix: core/raw OOB per field, `used > allocated`, declared-vs-reachable, controller layout/index, unknown flags/types, skin boundary/ambiguity/bone-ref oraz cycles/truncation pod base i animation roots. Stabilna taksonomia jest zapisana w `documentation/m1b-kontrakt-suplement-codex.md`.

### P-REF review findings

Wszystkie ponizsze poprawki maja status `FIXED_PENDING_GATES`: logika zostala skierowana do poprawy, ale nie ma jeszcze zakonczonego wspolnego native/WASM gate ani canonical runu.

| Finding | Status | Required behavior |
|---|---|---|
| Cryptographic binding | `FIXED_PENDING_GATES` | Builder sam uruchamia own reader oraz SHA-256 na tym samym `&[u8]`; same-length/different-content ma test regresyjny. |
| Manifest completeness | `FIXED_PENDING_GATES` | `expectedInput` obejmuje SHA-256 i byte length; expected capabilities/invariants sa non-empty, unique i dokladnie pokryte wynikami. |
| Provenance safety | `FIXED_PENDING_GATES` | Packet ma logiczny `commandLabel` i walidowany UTC timestamp; reference id/source/resref odrzucaja host paths i separatory. |
| Schema binding | `FIXED_PENDING_GATES` | Akceptowany jest manifest schema 1 i own-reader report schema 1. |
| No-payload boundary | `FIXED_PENDING_GATES` | Rekurencyjna kontrola JSON keys blokuje `payload`/`bytes`; filesystem/env pozostaja tylko w env-gated integration test. |

### Real-binary runtime-parent correction - status clarification

`M1B-BUG-001` ma status `FIXED_PENDING`, nie finalne `FIXED`: syntetyczny test potwierdza wyprowadzanie parentage z children traversal, ale canonical own-reader P-REF po M1C i pelne gates pozostaja niewykonane. Poprzedni wpis `FIXED` opisywal stan implementacji poprawki, nie zamkniety gate M1B.

### Otwarte granice

- Skin classifier ma jeden `node-to-bone map` pointer; brak dopasowania do `node + 0x2d4` i `node + 0x330` jest ambiguous. Dawny przypadek "oba" nie jest konstruowalny przez equality jednego pointera z dwoma roznymi offsetami.
- Znaczenie bone ref `0xffff` przy zerowej wadze pozostaje `OPEN`; brak zgody na wymyslenie sentinela przed canonical Aurora/M1C evidence.
- Container hashes nie zamykaja R1-R3; M1B ma przejsc `VERIFYING -> M1C -> M1B IN_PROGRESS -> final VERIFYING`.

### Nastepny krok

Zakonczyc synthetic/native/WASM gates, ustawic M1B na `VERIFYING`, uruchomic M1C locator, a nastepnie wrocic do M1B po canonical P-REF R1/R3 i wybor R4-R6. Nie deklarowac `DONE` na podstawie samego inventory.

## M1B-20260711-01 - synthetic checkpoint final gates

Status: VERIFYING, NIE DONE
Owner: Codex orchestrator + M1B implementation/review subagents
Stage: M1B

### Cel proby

Zamknac synthetic/native/WASM checkpoint deep readera i P-REF contract, naprawic wszystkie review findings oraz przekazac aktywny etap do M1C bez udawania canonical corpus proofu.

### Weryfikacja

| Command or action | Expected | Actual | Status |
|---|---|---|---|
| `cargo test --workspace` | wszystkie native tests zielone | 49 tests: 2 unit + 34 MDL + 13 P-REF; 0 failed | PASS |
| `wasm-pack test --node crates/m2a-wasm` | publiczny adapter obejmuje M1A i deep M1B | 4 tests; 0 failed, w tym deep report i oba skin variants | PASS |
| `cargo fmt --all -- --check` | brak format diff | brak output/error | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | zero warnings | zakonczono bez warnings | PASS |
| `cargo build --workspace` | oba crates buduja sie | `m2a-core` i `m2a-wasm` zbudowane | PASS |
| `git diff --check` | brak whitespace errors | brak output/error | PASS |
| P-REF final review | brak P1/P2 findings | brak findings | PASS |
| deep reader final code/test re-review | brak pozostalych findings | brak findings | PASS |

### Review findings - final status

Pierwsze osiem findings deep readera ma status `FIXED` i testy regresyjne:

1. common controller column layout;
2. skin map count przekraczajacy profil;
3. nierozstrzygniety `0xffff` bone ref zachowany bez wymyslonej semantyki;
4. addytywne trailing families zachowuja common mesh prefix;
5. overlap typed core ranges;
6. `unsupportedFamilies` nie zawiera supported header/mesh;
7. signed face adjacency;
8. deep deterministic JSON i oba skin variants przez publiczny WASM.

P-REF findings `cryptographic binding`, `manifest completeness`, `provenance safety`, `schema binding` i `no-payload boundary` maja status `FIXED`; finalny review nie znalazl nowych problemow.

`M1B-BUG-001` real runtime-parent correction ma status `FIXED`: parentage wynika z children traversal, arbitrary runtime parent/geometry values nie sa interpretowane jako serialized pointers, a regression jest czescia zielonych 34 MDL tests.

### Payload/provenance assertion

- Brak retail/CEP payloadow, extracted MDL/MDX, tekstur, animacji i szkieletow w Git.
- Testy CI/native/WASM uzywaja wlasnych synthetic builders.
- R1/R2/R3 hashes nadal sa container-level inventory o statusie `PENDING_CANONICAL_M1C_P-REF`.

### Pozostale problemy

```yaml
current_problems:
  - "M1C own HAK/ERF locator nie jest jeszcze zaimplementowany ani zweryfikowany."
  - "Canonical own-reader P-REF R1/R3 nie zostal jeszcze wykonany."
  - "R4-R6 nie sa jeszcze wybrane przez canonical inventory."
  - "GB-001-SKIN wymaga canonical boundary evidence albo jawnego nazwanego corpus gap."
bugs: []
```

### Nastepny krok

Ustawic M1B na `VERIFYING`, aktywowac M1C jako `IN_PROGRESS` z attempt `M1C-20260711-01`, a po locatorze wrocic do M1B dla canonical P-REF. M1B nie jest `DONE`.

## M1B-20260711-02 - canonical M1C handback

Status: IN_PROGRESS, NIE DONE
Owner: Codex orchestrator + M1B implementation/review subagents
Stage: M1B

### Cel proby

Przyjac zweryfikowany locator M1C i canonical own-reader P-REF R1/R3, a nastepnie wybrac R4-R6 i dostarczyc brakujace GB-001-SKIN boundary evidence.

### Handback z M1C

- M1C ma status `DONE`: 18 synthetic ERF tests i 1 canonical env test sa zielone, piec findings ma status `FIXED`, finalny re-review nie znalazl findings.
- R1/R3 sa odczytane in-place przez own HAK locator, przekazane jako borrowed slices do own MDL readera i zwiazane z P-REF bez extraction.
- Exact resource ID, container offset/size, SHA-256 oraz MDL core/raw ranges sa zapisane w `documentation/evidence/M1C-evidence.md`.
- Canonical packets nie zawieraja host path, `payload`/`bytes`, retail/CEP payloadu ani binaries.

### Canonical R1/R3 status

| Ref | Resref/type | Resource ID | Container range | SHA-256 | Status |
|---|---|---:|---|---|---|
| R1 | `c_kocrachn` / 2002 | 724 | `[179725952,179889144)` | `f16426310f826ae2ab15034ac979c65f812ee8bda0d13ee459bf2b293d7db270` | CANONICAL_OWN_READER_PASS |
| R3a | `c_phod_horror_b` / 2002 | 1026 | `[264142176,264988240)` | `62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a` | CANONICAL_OWN_READER_PASS |
| R3b | `c_phod_horror_p` / 2002 | 1027 | `[264988240,265834304)` | `09e43ee9493d2fe2bbf9cbeb44f24dcb999e5f38e651bdc79eefdd5e1f19722f` | CANONICAL_OWN_READER_PASS |

`M1B-BUG-001` pozostaje `FIXED`: canonical packets przechodza own reader, a runtime `parent_ptr`/`geometry_ptr` nie sa interpretowane jako serialized parentage. R1/R3 nie sa juz `PENDING_CANONICAL_M1C_P-REF`.

### Pozostale problemy

```yaml
current_problems:
  - "R4-R6 musza zostac wybrane z canonical inventory i uruchomione przez own locator/reader."
  - "GB-001-SKIN wymaga canonical legacy17/extended64 boundary evidence, w tym rozstrzygniecia lub jawnej klasyfikacji 0xffff przy zerowej wadze."
bugs: []
```

### Nastepny krok

Wybrac R4-R6 z canonical inventory, preferujac kandydatow zamykajacych oba warianty skin i bind-pose/bone-ref boundary, potem uruchomic P-REF. M1B pozostaje `IN_PROGRESS`, nie `DONE`.

## M1B-20260711-02 - canonical corpus implementation gate

Status: VERIFYING, NIE DONE
Owner: Codex orchestrator + M1B implementation/review subagents
Stage: M1B

### Wynik i gates

Implementation i canonical corpus gate sa zielone. Final review wykryl dodatkowe P1 w explicit sentinel/boundary/R0/R6/nonzero-bind proofie; ich fix i clean re-review sa wymagane przed `DONE`.

| Command or action | Actual | Status |
|---|---|---|
| `cargo test --workspace` | 68 native: 2 unit + 18 ERF + 1 env integration clean-skip + 34 MDL + 13 P-REF; 0 failed | PASS |
| real `M2A_REFERENCE_CEP_HAK` integration | 1 test, 6 P-REF packets; 0 failed | PASS |
| `wasm-pack test --node crates/m2a-wasm` | 4 Node/WASM tests; 0 failed | PASS |
| fmt/clippy/WASM build/diff gates | zielone | PASS |
| independent final review | explicit proof findings wymagaja fixu i re-review | VERIFYING |

### Canonical six-packet matrix

| Ref | Resref/type | Resource ID | Container range | SHA-256 | Role result |
|---|---|---:|---|---|---|
| R1 | `c_kocrachn` / 2002 | 724 | `[179725952,179889144)` | `f16426310f826ae2ab15034ac979c65f812ee8bda0d13ee459bf2b293d7db270` | 3 extended64 skins, count `38,38,38`, zero-weight `0xffff`; PASS |
| R3a | `c_phod_horror_b` / 2002 | 1026 | `[264142176,264988240)` | `62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a` | 42 animations, 41 events; PASS |
| R3b | `c_phod_horror_p` / 2002 | 1027 | `[264988240,265834304)` | `09e43ee9493d2fe2bbf9cbeb44f24dcb999e5f38e651bdc79eefdd5e1f19722f` | 42 animations, 41 events; PASS |
| R4 | `c_nulltail` / 2002 | 6390 | `[1420456988,1420458032)` | `b51542cc752421a41ff605d4c348794fff15ebfdb8973572d51a3a06fc7f8b76` | mesh-only, zero MDX/skin/animation/unsupported; PASS |
| R5 | `c_vampire_f` / 2002 | 6240 | `[1161251268,1161482260)` | `964b015298743216a0d78fa0ddf2dedc9fb6ad45c39f54457767fd60cd96c5d4` | 2 legacy17 skins, map/q/t/constants `28,28`; PASS |
| R6 | `c_eye` / 2002 | 552 | `[136663067,136686127)` | `401672fa00074c34b6c68e982242d5f0499ec657978826f15921f69200d719ea` | dangly diagnostic, 6 preserved mesh prefixes; PASS |

Own locator zwrocil borrowed slices, own reader zbudowal reports, a P-REF zwiazal te same bytes z exact fingerprintem. Packet/evidence nie zawiera host paths, `payload`/`bytes` ani zewnetrznych binaries.

### GB-001-SKIN i bug

`GB-001-SKIN` jest strukturalnie `CLOSED`: `legacy17`/`extended64` wynika z boundary `node + 0x2d4/0x330`; 17/64 nie jest capacity limitem; R1 count `38` i R5 count `28` przechodza; `0xffff` wystepuje w zero-weight lanes.

```yaml
bugs:
  - id: "M1B-BUG-002"
    severity: "P1"
    status: "FIXED"
    repro: "c_vampire_f legacy17 ma map/bind count 28, a stary parser odrzucal count > 17."
    expected: "17/64 klasyfikuje header boundary; counts waliduja arrays/ranges."
    actual: "Synthetic count28 i canonical R5 przechodza; R1 count38 i zero-weight 0xffff rowniez przechodza."
    next_action: "Uzupelnic explicit final-review invariants i wykonac clean re-review."
```

Globalny `GB-001` writer/readback/runtime pozostaje `DIRECTION_DEFINED_EVIDENCE_OPEN` dla M4+.

### Full-scan context

Scan wszystkich `3517` type-2002 entries dal `2146` parser successes, `1255` null-UV0 gaps, `96` header-invalid (`87` text MDL i `9` starych skin failures) oraz `20` diagnostic-limit cases. Brak camera/reference/animmesh/aabb w sukcesach jest named gapem. Te fakty nie blokuja role DoD wybranych R4/R5/R6.

M1B pozostaje `VERIFYING`, nie `DONE`. Nastepna akcja: naprawa final-review findings, full gates i clean re-review.

### Final-review P1 fix handoff

Status: FIXED_PENDING_REVIEW, M1B nadal VERIFYING

- R0 ma komplet 9 capability results i zgodny manifest: Header/CoreRanges/NodeTree `PASS`, pozostale `NOT_PRESENT`, plus core/raw coverage invariant.
- `SkinReport` raportuje generic `nodeOffset`, `headerSize` i `nodeToBonePointer`; canonical R1/R5 obliczaja exact boundary oraz `skin-map-count-observed` z own reportu.
- Sentinel classification jest obliczana z weights/bone refs: R1 `zero=881;nonzero=0`, R5 `zero=3208;nonzero=0`.
- R5 ma explicit `skin-nonzero-bind-pose-observed`; canonical run potwierdza wynik dodatni.
- R6 packet porownuje exact diagnostic: code `M2A-MDL-UNSUPPORTED-NODE-FAMILY`, severity `warning`, offset `476`, context `node \"Bat_body\" uses deferred node family dangly`.
- Clean-env 68 native, 4 WASM, fmt, clippy, WASM build i diff checks przechodza; real-env six-packet test przechodzi.

Niezalezny clean re-review pozostaje wymagany przed `DONE`.

## M1B-20260711-02 - final independent re-review

Status: DONE
Owner: Codex orchestrator + independent review agent
Stage: M1B

### Final result

- Independent final re-review po wszystkich P1 fixes: `NO FINDINGS`.
- `cargo test --workspace`: 68 native tests, w tym 2 unit + 18 ERF + 1 clean-skip integration + 34 MDL + 13 P-REF; 0 failed.
- `wasm-pack test --node crates/m2a-wasm`: 4 tests; 0 failed.
- Real `M2A_REFERENCE_CEP_HAK` run: 6 canonical packets R1/R3a/R3b/R4/R5/R6; 0 failed.
- fmt, clippy `-D warnings`, WASM build i `git diff --check`: PASS.
- `M1B-BUG-001` runtime pointer interpretation: `FIXED`.
- `M1B-BUG-002` legacy17 false capacity limit: `FIXED`.
- Structural `GB-001-SKIN`: `CLOSED`; 17/64 jest header boundary, nie capacity, a zero-weight `0xffff` lanes sa jawnie klasyfikowane.
- Full-scan named gaps nie blokuja wybranych R4/R5/R6 role DoD.
- Repo nie zawiera retail/CEP binaries, extracted payloadow, prywatnych host paths ani embedded `payload`/`bytes` w P-REF/evidence.

Globalny `GB-001` pozostaje `DIRECTION_DEFINED_EVIDENCE_OPEN`: writer emission, semantic readback, HAK round-trip i NWN EE runtime proof naleza do M4+.

### Handoff

M1B spelnia Definition of Done i ma status `DONE`. Jedyny aktywny etap przechodzi do M2 jako `IN_PROGRESS`, attempt `M2-20260711-01`; pierwsza akcja to AuroraAssetIR schema oraz synthetic GLB axis/UV fixtures. Ten checkpoint nie implementuje M2.
