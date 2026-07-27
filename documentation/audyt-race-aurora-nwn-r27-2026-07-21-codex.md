# Audyt race do widocznego modelu w Aurora Toolset i NWN

Data: 2026-07-21  
Snapshot live: 2026-07-21T16:00:45+02:00  
Zakres: aplikacja Meshy2Aurora, bieżący kandydat M0 r27, centralny tor
operatora Aurora/NWN i bramki proofu.  
Tryb: audyt read-only; bez sterowania Toolsetem/NWN, bez zmiany konfiguracji i
bez utworzenia nowej iteracji modelu.

## Werdykt wykonawczy

**Nie trzeba teraz zmieniać modelu. Trzeba domknąć proof dokładnego r27.**

Kandydat r27 ma zielony writer/readback, siedem poprawionych nagłówków
local-animation `type=5`, zgodne hashe zainstalowanych artefaktów i
`saved_verified` dla MOD/ordered HAK list. Nie istnieje świeży, związany z r27
obraz Toolsetu ani NWN. Jego poprawny status to:

| Środowisko | `modelVisibility` | `proofCompleteness` |
| --- | --- | --- |
| Aurora Toolset, r27 | `not_tested` | `missing` |
| NWN, r27 | `not_tested` | `missing` |

Kolejne `r28`, nowy MOD, HAK, resref, Appearance, placement albo regeneracja
payloadu są zabronione do czasu świeżego `not_visible` dla dokładnego r27.

Najkrótsza ścieżka jest operacyjna:

1. zatrzymać wszystkie poboczne prace produktowe;
2. dodać do **centralnego** operatora immutable Toolset profile dla dokładnego
   r27 oraz domknąć jego walidator proofu;
3. po zwolnieniu istniejącej sesji wykonać jeden Toolset run:
   exact object selection/readback -> świeży `TScrollBox`; `Focus on Object`
   i kadrowanie są opcjonalne;
4. jeśli r27 jest widoczny, bez zmiany linii artefaktów wykonać `Test Module`,
   capture NWN i engine log;
5. dopiero wynik tego runu decyduje o następnym kroku.

## Stan dowodów

### Dokładny kandydat r27

Źródło packetu:
`proof-output/m0-r27-animation-type5-20260721/`.

| Artefakt | SHA-256 / fakt |
| --- | --- |
| MDL `m2a_m0p01` | `bf7864f0ed541ed93c4075d167f4a2392eeb6ef37b4cdf444cbb74efd5731578` |
| HAK `m2a_m0r27` | `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd` |
| MOD `m2a_m0r25.mod` | `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257` |
| Area | `m2a_m0a25` |
| Entry | `(10,10,0)` |
| Fixture | `m2a_m0p01`, Appearance `848`, `(10,14.5,0)` |
| Ordered HAK list | `m2a_m0r27`, `m2a_m0r26`, `m2a_m0r21` |

Hashe zainstalowanego MOD i wszystkich trzech HAK-ów są zgodne z profilem
r27. Centralny AUR-S07 `dry-run` przechodzi, lecz `live/` packetu r27 jest
pusty.

### Poprzedni wynik, który dopuścił r27

R26 na tym samym module/fixture ma:

- Toolset: `modelVisibility=visible`; historyczny capture został wykonany po
  opcjonalnym `Focus on Object` i pokazuje model;
- NWN: `modelVisibility=not_visible`; świeży obraz nie pokazuje fixture'a, a
  engine log wiąże ładowanie `m2a_m0r25`;
- runtime `proofCompleteness=missing`, ponieważ nie istnieje pełny packet
  AUR-S07.

Ten wynik dopuścił dokładnie jedną minimalną zmianę r26 -> r27: siedem bajtów
`00 -> 05` pod `GeometryHeader + 0x6c`. Nie dopuszcza kolejnej zmiany przed
testem r27.

## Wyniki audytu kodu i testów

### Zielone bramki bieżącego M0

- `cargo test -p m2a-core --test mdl_writer`: 31/31;
- `cargo test -p m2a-core --test model_pipeline`: 14/14;
- `cargo test -p m2a-core --test binary_m0_vertical_slice_module`: 2/2;
- łączny czas powyższych testów na bieżącym snapshotcie: 3,61 s;
- Studio `typecheck` i `vitest`: 28 plików, 172/172 testy, 7,55 s;
- `cargo fmt --all -- --check`: PASS;
- `git diff --check`: PASS.

### Czerwona bramka repo

`cargo test --workspace` kończy się po 7,62 s dwoma istniejącymi błędami GFF:

1. `phase_nine_field_indices_layout_precedes_phase_ten_oversized_locstring_limit`;
2. `swapped_field_indices_are_in_bounds_but_violate_canonical_encounter_order`.

To nie jest blocker testu r27 w Toolsecie/NWN, ale jest blockerem czystego
merge/release. Nie należy mieszać naprawy GFF z kandydatem r27.

## Findings

### P0-1 — brak centralnego profilu Toolsetu dla dokładnego r27

Centralny `aurora-toolset-session.mjs` ma zatwierdzony profil M0 wyłącznie dla
historycznego `m0-static-rigid-v5`. Profil, validator, queue contract i packet
validator są związane na stałe z `m2a_m0_proof.mod` /
`m2a_m0proof_area`. Próba użycia r27 poprawnie kończy się
`UNAPPROVED_MODULE_PATH`.

Skutek: r27 nie może uzyskać akceptowanego Toolset packetu mimo gotowego
MOD/HAK/readbacku.

Minimalna poprawka należy do centralnego operatora, nie do Meshy2Aurora:

- immutable profile `m0-r27-on-r25-toolset-v1`;
- exact path i SHA MOD;
- Area resref oraz dokładny native tree display text;
- fixture `m2a_m0p01`, Appearance 848, `(10,14.5,0)`;
- ordered HAK resrefy i SHA-256;
- capture target `TScrollBox`, caller-owned outDir i exact packet validator;
- test kontraktu odrzucający inny MOD, HAK order/hash, Area, fixture i capture.

Nie tworzyć project-local runnera ani parametrów omijających profil.

### P0-2 — centralny AUR-S07 nie spełnia jeszcze własnego kontraktu akceptacji

Bieżący `aur-s07-runtime-execution.mjs` w `dry-run` sprawdza głównie kształt
JSON. Nie odczytuje zainstalowanego MOD/HAK, nie hashuje pliku geometry gate,
nie sprawdza treści/freshness engine logu i przy `validate` ufa polu
`modelVisibility.status` dostarczonemu przez packet. Test kontraktu używa
zwykłego tekstu jako `runtime.png` i `nwn.log`, a mimo to uzyskuje `verified`.

Skutek: aktualne `dry-run=ok` nie jest dowodem gotowości ani nie pozwala
oznaczyć runtime r27 jako `verified`.

Minimalne centralne hardening przed finalnym claimem:

- profile wiąże ścieżki i hashe zainstalowanego MOD oraz ordered HAK-ów;
- geometry gate jest odczytany, zahashowany i semantycznie związany;
- observation wymaga świeżego PID i one-time request/handoff;
- engine log zawiera świeże `Loading Module: m2a_m0r25`;
- capture jest rzeczywistym PNG z właściwego procesu/okna, świeżym po starcie;
- wizualny werdykt jest zapisany po inspekcji, nie tylko przepisany z JSON;
- negatywne testy odrzucają tekst podszywający się pod PNG/log, stale artifacts,
  wrong HAK order/hash i wrong module/Area/fixture.

### P0-3 — project-local runner proofu jest zabroniony i nieaktualny

`tools/m2a-aurora-proof.mjs` ma 981 linii własnego transportu Toolsetu,
uruchamiania procesu i adapterów UI. Jest związany ze starym
`m2a_m0_proof.mod`, nie z r27. Narusza regułę projektu wymagającą bezpośredniego
użycia centralnego runnera/atomów i zakazującą lokalnego substytutu.

Skutek: utrzymujemy drugi workflow, drugi zestaw założeń i historyczny target;
to zwiększa czas oraz ryzyko błędnego proofu.

Decyzja: natychmiast usunąć go z critical path. Po zachowaniu provenance zmian
usunąć/deprecjonować ten runner i testy w osobnym cleanupie. Nie używać go do
r27.

### P0-4 — plan projektu nie opisuje realnego active stage

`documentation/orchestrator-state.yaml` nadal ma `last_updated=2026-07-14`,
`active_stage=M7_V5_S1_REAL_E2E_INPUT_GATE` i next action dotyczący Meshy/browser
E2E. Nie ma w nim r27 ani bieżącej bramki Aurora -> NWN.

Repo ma 125 pozycji dirty: 45 zmodyfikowanych i 80 untracked; sam tracked diff
to 7 369 insercji i 618 usunięć. Jednocześnie zmieniane są Rust MDL/GFF,
WASM/web, Meshy bridge, Docker, proof tooling i duża liczba dokumentów.

Skutek: brak jednego active stage powoduje wykonywanie poprawnej pracy w złej
kolejności oraz zwiększa ryzyko nieodtwarzalnego r27.

Decyzja organizacyjna dla race:

- jedyny active stage: `M0_R27_AURORA_NWN_LIVE_PROOF`;
- zamrozić M7/S1/UI/Docker/bridge i M4/H1 do wyniku r27;
- nie porządkować całego worktree przed proofem, ale nie dodawać dalszych zmian;
- po proofie rozdzielić commity: r27 writer/readback, proof/evidence, niezależne
  UI/Meshy/Docker oraz GFF fix.

### P0-5 — proof candidate nie jest dokładnym artefaktem pobieranym z aplikacji

Studio/WASM dla lane M0 wywołuje
`build_meshy_m0_static_rigid_package_v1()` i emituje pojedynczy HAK
`m2a_m0_proof.hak` oraz MOD `m2a_bm0p1.mod`. Bieżący live candidate używa
modelu/HAK r27 dołączonego natywnie do `m2a_m0r25.mod` z trzema HAK-ami.

Skutek: sukces r27 na r25 zamknie zgodność binary MDL/HAK z rendererem, ale nie
zamknie jeszcze finalnego one-click artefaktu pobranego ze Studio.

To rozdzielenie jest korzystne dla szybkości:

1. najpierw zamknąć „czy nasz model może być widoczny w obu silnikach” na r27;
2. dopiero potem przenieść potwierdzone invariants do exact Studio output i
   zrobić osobny proof wygenerowanego single-HAK MOD.

Nie wolno mieszać tych dwóch celów w następnym live A/B.

### P1-1 — `M0RuntimeFixtureContractV1` nie obsługuje r27 multi-HAK

Builder i verifier wymagają dokładnie jednego wpisu `Mod_HakList`. R27 ma trzy
warstwy, wszystkie z tymi samymi kluczami `appearance`, `m2a_m0p01` i
`m2a_m0t01`, lecz różnymi MDL.

Przed claimem pełnej gotowości aplikacji potrzebny jest kompatybilny V2:

`ordered_hak_layers = [{resref, sha256, bytes}, ...]`

z jawnie nazwaną regułą priorytetu i testami order/hash/resource winner.
Nie zmieniać r27 ani nie scalać HAK-ów przed jego live testem.

### P1-2 — „canonical runtime” nadal emituje skróconą `appearance.2da`

Oba warianty M0 wykonują `retain_two_da_row_prefix_v1(..., 848, ...)`, więc
funkcja nazwana canonical runtime emituje tabelę 0..848 zamiast bezpiecznego
pełnego appendu. Dla dokładnego fixture'a 848 nie jest to potwierdzona przyczyna
braku modelu; Toolset już pokazał tę mapę.

Po pierwszym proofie trzeba rozdzielić typowane scope:

- `IsolatedToolsetVerticalSlice`;
- `FullRuntimeAppend` zachowujący pełny input i zapisujący row counts/scope w
  manifeście.

To jest blocker dystrybucji ogólnego HAK-a, nie blocker bieżącego live r27.

## Snapshot żywej sesji

W chwili audytu istnieją:

- responsywny `nwtoolset.exe`, PID `51048`;
- responsywny `nwmain.exe`, PID `9400`;
- Toolset posiada moduł `m2a_now01.mod`, nie r27;
- Toolset jest zminimalizowany na proof monitorze;
- nie stwierdzono widocznego modalu r27.

To jest obca/inna aktywna lane. Audyt jej nie zamyka, nie przełącza i nie
wykorzystuje. R27 może wystartować dopiero po zwolnieniu tej sesji albo po
jednoznacznym przekazaniu jej własności zgodnie z centralnym profilem.
Konflikt procesu jest tymczasowym blockerem wykonania, nie wizualnym błędem
modelu.

## Sekwencja dowiezienia

### Gate A — freeze i profile

1. Nie zmieniać r27 i nie tworzyć r28.
2. Dokończyć albo przekazać istniejącą lane `m2a_now01`; bez kill/restartu z
   toru r27.
3. W centralnym `aurora-web` dodać exact immutable Toolset profile r27 oraz
   test kontraktu.
4. W centralnym AUR-S07 domknąć binding/freshness/log/capture gates opisane w
   P0-2.
5. Centralne dry-runs muszą przejść na dokładnych hashach r27 i caller-owned
   `proof-output/m0-r27-animation-type5-20260721/live`.

### Gate B — Aurora Toolset

1. Jedna sesja i jeden module owner.
2. Otworzyć dokładnie MOD SHA
   `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257`.
3. Odczytać ordered HAK list, Area i fixture.
4. Wybrać dokładny obiekt `m2a_m0p01`/Appearance 848 i zachować native
   readback jego tożsamości.
5. Opcjonalnie użyć `Focus on Object` lub kadrowania, jeżeli poprawia to
   czytelność; nie jest to bramka.
6. Świeży, zwalidowany capture `TScrollBox` i packet validator.
7. Zapisać osobno `modelVisibility` i `proofCompleteness`.

Jeśli model jest widoczny, nie zmieniać niczego i przejść do Gate C. Jeśli nie
jest widoczny, zachować packet, zdiagnozować dokładną resource chain i dopiero
wtedy rozważyć minimalną kolejną iterację.

### Gate C — NWN

1. Ten sam Toolset/MOD/HAK/Area/fixture.
2. Uzbroić one-time AUR-S07 request.
3. User-owned `Test Module` dokładnie raz.
4. Obserwować dokładnie jeden świeży `nwmain`.
5. Zebrać runtime capture i engine log z
   `Loading Module: m2a_m0r25`.
6. Obejrzeć capture i zwalidować pełny packet.
7. Zapisać osobno `modelVisibility` i `proofCompleteness`.

### Decyzja po Gate C

- **Aurora visible + NWN visible:** zamknąć pierwszy engine-compatibility
  milestone; potem productyzować exact single-HAK Studio output.
- **Aurora visible + NWN not_visible:** zachować świeży packet; diagnozować
  runtime. Pierwszym podejrzanym do izolacji jest ambiguity multi-HAK/resolver,
  nie losowa zmiana geometrii. Nowa iteracja wymaga zapisanej minimalnej delty.
- **Aurora not_visible:** diagnozować ordered HAK -> row 848 -> MDL -> fixture;
  nie przechodzić do NWN i nie zgadywać nowego eksportu.

## Czego nie robić

- nie tworzyć r28 bez świeżego `not_visible` dla r27;
- nie używać `tools/m2a-aurora-proof.mjs`;
- nie dotykać `nwtoolset.ini`, MRU ani ustawień gry;
- nie zabijać aktywnego Toolsetu/NWN należącego do innej lane;
- nie mieszać GFF, M4/H1, UI, Docker ani Meshy bridge z live A/B r27;
- nie uznawać parsera, builda, `dry-run=ok`, logu bez capture albo screenshotu
  bez dokładnego object bindingu i zwalidowanego `TScrollBox` za proof
  widoczności; brak opcjonalnego Focusu nie unieważnia proofu;
- nie traktować sukcesu r27 na r25 jako finalnego proofu exact Studio output.

## Definition of Done dla bieżącego race

Race jest dowieziony dopiero, gdy jeden immutable packet r27 zawiera:

- exact MOD i ordered HAK hashe;
- saved/native readback Area i fixture;
- świeży, związany z dokładnym obiektem, zwalidowany `TScrollBox` z
  `modelVisibility=visible`; Focus i kadrowanie są opcjonalne;
- świeży NWN capture z `modelVisibility=visible`;
- świeży engine log wiążący właściwy moduł;
- przechodzące centralne validators;
- brak placeholdera, fallbacku, blank modelu i nieoczekiwanych modali;
- osobne, poprawne wartości `proofCompleteness` dla Toolsetu i NWN.

Do tego czasu stan projektu pozostaje `R27 SAVED_VERIFIED / LIVE PROOF MISSING`.
