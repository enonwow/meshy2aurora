# M1B - suplement canonical corpus capability/invariant matrix

Data: 2026-07-11 | Autor: Codex | Status: ZAMKNIETY SUPLEMENT, M1B DONE

## 1. Cel i aktualny punkt startu

Ten suplement doprecyzowuje finalny canonical corpus gate M1B. Nie rozszerza M1B na writer ani runtime acceptance.

- M1C jest `DONE`; own HAK/ERF locator przekazal canonical R1/R3 jako borrowed type-2002 slices do own MDL readera.
- R1/R3 maja potwierdzone identity, resource ID, container range, SHA-256, core/raw ranges oraz role capabilities/invariants.
- Powtarzalny selector nazwal R4/R5/R6 i zwiazal je z exact manifest identity ponizej. Real-env integration potwierdzil own locator -> borrowed slice -> own reader -> P-REF dla wszystkich szesciu packetow.
- R2 KEY/BIF pozostaje opcjonalnym adapterem; brak uruchomienia R2 nie blokuje M1B, jesli wymagane rodziny pokrywaja canonical R1/R3/R4-R6.
- M1B ma status `DONE`; niezalezny final re-review nie znalazl findings.

### Inventory context, nie rozszerzenie scope

Read-only, bounded scan calego `cep3_core1.hak` objal wszystkie `3517` wpisow type 2002. Own locator i binary reader daly:

| Wynik | Liczba |
|---|---:|
| parser success | 2146 |
| `M2A-MDL-POINTER-OOB` | 1255 |
| `M2A-MDL-HEADER-INVALID` | 96 |
| `M2A-LIMIT-EXCEEDED` | 20 |

To jest kontekst inventory do wyboru swiadkow, nie nowy DoD M1B. Named gaps pozostaja jawne:

- wszystkie `1255` pointer failures maja ten sam powod: `required uv0 pointer is null`; obsluga canonical mesh bez UV0 jest osobnym parser gapem;
- `20` limit failures przekracza guardrail `1024` diagnostics;
- `87` header failures to tekstowe MDL zaczynajace sie od `#Exp`, `#MAX` albo `# Ex`, mimo type 2002; binary-only reader odrzuca je celowo;
- pozostale `9` header failures ujawnily map-count/profile-boundary skin gap, w tym R5 `c_vampire_f`;
- wsrod `2146` sukcesow bylo `128` modeli `extended64` i zero `legacy17`; canonical `legacy17` pochodzi z R5 po poprawce classifiera/count semantics, a nie z falszywej reklasyfikacji sukcesow;
- successful-family inventory pokrywa `header`, `mesh`, `skin`, `dangly`, `emitter`, `light`; brak `camera`, `reference`, `animmesh` i `aabb` w sukcesach nie rozszerza obecnego korpusu.

Te named gaps sa kontekstem pelnego inventory, nie blockerami role DoD: wybrane R4/R5/R6 maja exact canonical packety i wymagane invariants.

## 2. Canonical bug evidence - skin boundary

Aktualny read-only selector/parser repro raportuje:

| Model | Observed header boundary | Observed `map count` | Dodatkowa obserwacja | Status |
|---|---|---:|---|---|
| `c_kocrachn` | `extended64`, node-to-bone payload przy `node + 0x330` | `38` | `0xffff` wystepuje w lanes o zerowej wadze | `CANONICAL_PACKET_PASS`; parser fix `FIXED`, re-review pending |
| `c_vampire_f` | `legacy17`, node-to-bone payload przy `node + 0x2d4` | `28` | canonical legacy witness | `CANONICAL_PACKET_PASS`; parser fix `FIXED`, re-review pending |

Wniosek kontraktowy: `17` i `64` sa szerokosciami inline mappingu w headerze i rozmiarami granicy `0x2d4`/`0x330`. Nie sa capacity limitami `map count`, bind arrays ani indeksem kosci. Reader nie moze odrzucac `c_vampire_f` tylko dlatego, ze `map count = 28 > 17`. Bone reference moze byc uznany za OOB tylko wobec potwierdzonej domeny node-to-bone/bone inventory, nie wobec samej szerokosci headera.

Obserwacja `0xffff` przy zerowej wadze jest canonical evidence, ze reader ma zachowac taki lane bez wymyslania aktywnego bone lookup. Nie dowodzi jeszcze, ze writer moze emitowac dowolny sentinel. Decyzja emission/readback i NWN EE acceptance pozostaje w M4.

## 3. Dokladne role canonical corpus

### R4 - mesh-only

Finalny R4 to `c_nulltail`, type 2002. Selector i own reader potwierdzily:

- co najmniej jeden trimesh z faces, vertices, UV0 i normals w prawidlowych core/raw ranges;
- zero skin nodes;
- zero wlasnych model animations i events;
- zero unsupported node families.

Exact manifest identity:

```yaml
reference_id: R4
source: { kind: named_hak, name: cep3_core1 }
resref: c_nulltail
resource_type: 2002
resource_id: 6390
container_offset: 1420456988
container_end: 1420458032
payload_length: 1044
sha256: b51542cc752421a41ff605d4c348794fff15ebfdb8973572d51a3a06fc7f8b76
core_range: { start: 12, length: 1032, end: 1044 }
raw_range: { start: 1044, length: 0, end: 1044 }
status: CANONICAL_PACKET_PASS
```

Observed role counts: `2` nodes, `1` mesh, `0` skin nodes, `0` animations, `0` events, controller types `{8,20}`, puste `unsupportedFamilies` i puste diagnostics. Zerowy MDX jest poprawna granica `[1044,1044)`, nie brak zasobu.

### R5 - skin boundary i bind data

R5 jest pokryciem dwoch realnych boundary, nie jedna magiczna stala:

- `extended64`: rozszerzony P-REF R1 `c_kocrachn` potwierdza Mesh/Skin i ponizsze invarianty;
- `legacy17`: `c_vampire_f` z exact identity ponizej.

Kazdy przypadek ma niepuste weights/bone refs oraz common inverse-bind arrays. Canonical R5 potwierdza dwa skin nodes, q/t/constants counts `28,28` oraz explicit `skin-nonzero-bind-pose-observed=PASS`. M1B pozostaje `VERIFYING` do clean re-review.

```yaml
reference_id: R5
source: { kind: named_hak, name: cep3_core1 }
resref: c_vampire_f
resource_type: 2002
resource_id: 6240
container_offset: 1161251268
container_end: 1161482260
payload_length: 230992
sha256: 964b015298743216a0d78fa0ddf2dedc9fb6ad45c39f54457767fd60cd96c5d4
core_range: { start: 12, length: 91128, end: 91140 }
raw_range: { start: 91140, length: 139852, end: 230992 }
skin_header_boundary: 0x2d4
skin_map_count_observed: 28
status: CANONICAL_PACKET_PASS
```

### R6 - structured unsupported family

Finalny R6 to `c_eye`, type 2002. Own reader konczy go nonfatalnie, zachowuje `6` mesh common prefixes i raportuje konkretna deferred family `dangly` przez `M2A-MDL-UNSUPPORTED-NODE-FAMILY`. Nie ma dodatkowego szumu `M2A-MDL-CONTROLLER-TYPE-UNKNOWN`, dlatego jest lepszym izolowanym swiadkiem niz `c_lok_skyel`.

```yaml
reference_id: R6
source: { kind: named_hak, name: cep3_core1 }
resref: c_eye
resource_type: 2002
resource_id: 552
container_offset: 136663067
container_end: 136686127
payload_length: 23060
sha256: 401672fa00074c34b6c68e982242d5f0499ec657978826f15921f69200d719ea
core_range: { start: 12, length: 11952, end: 11964 }
raw_range: { start: 11964, length: 11096, end: 23060 }
status: CANONICAL_PACKET_PASS
```

Observed role counts: `10` nodes, `6` meshes, `0` skin nodes, `0` animations, `0` events, controller types `{8,20}`, `unsupportedFamilies={dangly}` i diagnostic codes `{M2A-MDL-UNSUPPORTED-NODE-FAMILY}`.

## 4. Capability matrix

Kazdy wykonany P-REF zapisuje kazda capability jako `PASS`, `UNSUPPORTED` albo `NOT_PRESENT`. `OBSERVED` w ponizszej tabeli jest instrukcja budowy packetu, nie czwartym statusem: finalny packet rozstrzyga status z own-reader reportu. `NOT_PRESENT` oznacza potwierdzona nieobecnosc, nie brak sprawdzenia.

| Role | Header | CoreRanges | NodeTree | Mesh | Skin | Controllers | Animations | Events | UnsupportedNodeFamily |
|---|---|---|---|---|---|---|---|---|---|
| R0 synthetic | PASS | PASS | PASS | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT |
| R1 `c_kocrachn` | PASS | PASS | PASS | PASS | PASS | PASS | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT |
| R2 base NWN | opcjonalny run | opcjonalny run | opcjonalny run | opcjonalny run | opcjonalny run | opcjonalny run | opcjonalny run | opcjonalny run | opcjonalny run |
| R3a/R3b | PASS | PASS | PASS | PASS | NOT_PRESENT | PASS | PASS | PASS | NOT_PRESENT |
| R4 `c_nulltail` | PASS | PASS | PASS | PASS | NOT_PRESENT | PASS | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT |
| R5 `c_vampire_f` | PASS | PASS | PASS | PASS | PASS | PASS | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT |
| R6 `c_eye` | PASS | PASS | PASS | PASS | NOT_PRESENT | PASS | NOT_PRESENT | NOT_PRESENT | PASS |

Dla R6 `UnsupportedNodeFamily=PASS` oznacza, ze capability stabilnej detekcji zadzialala; exact nieobslugiwana rodzina pozostaje `UNSUPPORTED` semantycznie i musi miec diagnostic w packet/report. R2 ma osobny execution status `OPTIONAL_NOT_RUN`, gdy adapter KEY/BIF nie jest uruchomiony; nie tworzy sie dla niego fikcyjnych capability results.

## 5. Minimalne invariant results

Kazdy canonical P-REF ma co najmniej:

- `binary-mdl-id-is-zero`;
- `payload-byte-length-exact`;
- `core-byte-length-exact`;
- `raw-byte-length-exact`;
- `file-header-core-raw-cover-payload`.

Role dodaja:

```yaml
R1_extended64:
  - "skin-node-count-positive"
  - "skin-header-boundary-exact: expected 0x330,0x330,0x330"
  - "skin-map-count-observed: expected 38,38,38"
  - "skin-bind-arrays-nonempty"
  - "skin-raw-weights-bone-refs-in-bounds"
  - "skin-ffff-lanes-classified: expected zero=881;nonzero=0"
R3a_R3b:
  - "own-animation-count-exact: expected 42"
  - "animation-root-trees-in-bounds"
  - "animation-events-in-bounds"
  - "animation-controllers-in-bounds"
R4:
  - "mesh-node-count-positive"
  - "skin-node-count-zero"
  - "own-animation-count-zero"
  - "mesh-required-arrays-in-bounds"
  - "unsupported-node-family-count-zero"
R5_legacy17:
  - "skin-node-count-positive"
  - "skin-header-boundary-exact: expected 0x2d4,0x2d4"
  - "skin-map-count-observed: expected 28,28"
  - "skin-bind-arrays-nonempty"
  - "skin-raw-weights-bone-refs-in-bounds"
  - "skin-ffff-lanes-classified: expected zero=3208;nonzero=0"
  - "skin-nonzero-bind-pose-observed: expected positive"
R6:
  - "unsupported-node-family-present"
  - "unsupported-node-family-diagnostic-exact"
  - "supported-common-prefix-preserved"
```

Exact invariant names zapisane w manifeście nie zawieraja `expected` value po dwukropku; expected/actual sa osobnymi polami resultu. Powyższy zapis pokazuje wymagana wartosc evidence.

## 6. `NOT_EVALUATED` policy

- `FAIL` na common albo role-defining invariant blokuje M1B `DONE`.
- `NOT_EVALUATED` nie moze zastepowac invariantow definiujacych R4, R5 lub R6.
- `NOT_EVALUATED` jest dopuszczalne tylko dla jawnie nazwanego corpus gap, gdy aktywny kontrakt tej roli pozwala na taki gap. Result musi miec `expected`, `actual` i stable diagnostic nazywajacy brak.
- Brak R4 albo R6 nie ma obecnie wyjatku corpus-gap i blokuje `DONE`.
- Dla `0xffff` canonical zero-weight observation zamyka obowiazek readera: preserve/classify. Semantyka writera pozostaje jawnie nieewaluowana do M4 i nie blokuje M1B.

## 7. Env/CI i status gate

Env-gated canonical test moze cleanly skipowac w CI i na hostach bez reference source. Jest jednak obowiazkowym lokalnym evidence runem przed M1B `DONE`; skip CI nie jest evidence zgodnosci canonical corpus.

M1B ma status `DONE`: parser fix count `28`/`38`, explicit boundary fields, R0 full capability row, sentinel classification, R5 nonzero bind pose i exact R6 diagnostic przeszly full gates oraz real-env six-packet run. `GB-001-SKIN` jest strukturalnie `CLOSED`.

Final-review findings maja status `FIXED`, a niezalezny clean re-review nie znalazl findings. M1B `DONE` nie zamyka globalnego GB-001: writer, semantic readback, HAK round-trip oraz Toolset/game acceptance pozostaja M4+.
