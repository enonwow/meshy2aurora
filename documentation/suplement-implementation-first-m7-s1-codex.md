# Suplement implementation-first dla M7 i S1

Data: 2026-07-14 | Autor: Codex | Status: AKTYWNY | Decyzja wlasciciela

## 1. Cel i pierwszenstwo

Ten suplement zmienia kolejnosc pracy dla aktywnej fali M7/S1. Najpierw
powstaje pierwsza, spojna implementacja wiekszosci pionowych slice'ow, a
dopiero potem rozpoczyna sie wspolna faza testow, napraw, niezaleznego review
i proof gates.

Suplement ma pierwszenstwo przed dotychczasowym wymaganiem uruchamiania pelnej
bramki testowej po kazdym malym slice oraz przed harmonogramem TDD z sekcji 3
`documentation/PROJECT_RULES.md`. Nie znosi wymaganej jakosci koncowej ani
Definition of Done. Zmienia tylko moment tworzenia i uruchamiania testow.

## 2. Twarde granice

- Implementacja i artefakty robocze powstaja tylko w
  `C:\Projects\meshy2aurora`.
- `C:\Users\enonw\Documents\meshy2aurora` jest bezwzglednie zakazane dla
  implementacji, stagingu, scratcha, testow, buildow, cache i plikow
  tymczasowych. Wlasciciel nigdy nie wskazal tej sciezki. Agent uruchomiony z
  takim workspace'em wykonuje HARD STOP bez zapisu i bez kopiowania pozniej.
- `C:\Projects\aurora-web`, Aurora/NWN EE, konfiguracje, katalogi uzytkownika,
  Toolset i gra sa w tej fali read-only i nie sa uruchamiane ani modyfikowane.
- Brak trzech oryginalnych modeli Meshy jest odroczonym input gate, a nie
  biezacym blockerem implementacji. Nie blokuje M7-V1--M7-V4, S1-V1--S1-V5,
  wspolnej fazy testow kodu ani pierwszego checkpointu implementacyjnego.
- Syntetyczne fixture nie moga zostac przedstawione jako trzy profile Meshy.
- Nie implementujemy klas gameplay, pelnego generatora modulu, S2 ani F1-F10.

## 3. Zamrozony inventory vertical slices

Pierwsza fala obejmuje dziesiec slice'ow:

| ID | Slice | Pierwsza implementacja oznacza |
|---|---|---|
| M7-V1 | Corpus contract | wersjonowany manifest trzech profili, provenance, hash i wymagane role |
| M7-V2 | Corpus intake | resolver lokalnych plikow, walidacja kompletności i diagnostyka odroczonych wejsc; realna kompletnosc nalezy do M7-V5 |
| M7-V3 | Canonical batch runner | runner i kanoniczne wywolania API bez alternatywnego konwertera; trzy realne wykonania naleza do M7-V5 |
| M7-V4 | Per-profile proof packet | builder i schema deterministycznego packetu ze statusem `INPUT_DEFERRED`; trzy realne packety naleza do M7-V5 |
| M7-V5 | Real three-profile execution | humanoid animated, non-humanoid oraz static model na oryginalnych eksportach Meshy |
| S1-V1 | Studio shell and local files | React/TypeScript shell, lokalny file picker i jawny stan sesji bez uploadu |
| S1-V2 | Canonical WASM Worker | Worker laduje publiczny adapter `m2a-wasm`; brak drugiego parsera lub mock convertera |
| S1-V3 | Source viewport | Three.js renderuje dane source/canonical IR z widoczna provenance |
| S1-V4 | Aurora/readback and validation | podglad output/readback, diagnostyka i powiazanie zaznaczenia z czescia modelu |
| S1-V5 | Artifact downloads | pobranie wygenerowanego HAK i raportow z bajtow zwroconych przez Worker |

`M7-V5` jest odroczony do fazy po pierwszej implementacji i integracji. Zaczyna
sie dopiero po dostarczeniu trzech oryginalnych modeli Meshy. Nie zmniejsza to
inventory i nie pozwala oznaczyc M7 jako DONE, ale nie nadaje aktywnej fali
statusu BLOCKED.

### 3.2 M7-V4W - publiczny most WASM

`M7-V4W` jest completion sub-slice'em M7-V4 i nie zmienia dziesiecioelementowego
inventory. Domyka odroczona granice core -> browser bez otwierania M7-V5.

Definition of Done:

- publiczny adapter `m2a-wasm` przyjmuje strict manifest JSON oraz jeden blob
  lokalnych payloadow z wersjonowanymi deskryptorami offset/length;
- manifest validation, intake i batch deleguja do istniejacych API
  `m2a-core`; adapter nie zawiera drugiego parsera ani konwertera;
- obecna canonical humanoid route moze byc materializowana tylko przez
  `M7CanonicalPipelineArtifactV1::build_rigged_humanoid_m6`;
- non-humanoid i static pozostaja jawnymi `DEFERRED_M7_V5`, bez synthetic
  substitution i bez `M7 DONE` claim;
- payloady binarne nie sa kodowane jako base64 i adapter nie korzysta z
  filesystemu, DOM, Aurory/NWN ani konfiguracji uzytkownika;
- strict boundary errors, native/WASM JSON parity i deterministycznosc maja
  bezposrednie testy.

Wynik 2026-07-14: `DONE_REVIEW_CLEAN`. Native adapter ma 8/8 testow PASS,
real generated Node/WASM przechodzi READY i deferred flow, a obie strony
potwierdzaja ten sam frozen hash pelnego batch JSON. Dwa niezalezne finalne
review zakonczyly sie P1=0/P2=0. To domyka tylko granice core -> browser;
oryginalne modele Meshy i M7-V5 pozostaja odroczonym input gate.

### 3.3 M7-V4S - Studio M7 corpus Worker flow

`M7-V4S` jest completion sub-slice'em M7-V4/S1-V2. Laczy publiczny most
`M7-V4W` z istniejacym lokalnym Studio Workerem, nie zmienia inventory i nie
otwiera M7-V5.

Definition of Done:

- manifest JSON i payloady sa wybierane lokalnie; nie ma uploadu ani network;
- Studio Worker wywoluje publiczne M7 validate/intake/batch API bez drugiego
  walidatora lub konwertera w TypeScript;
- payloady sa skladane w jeden transferable `ArrayBuffer` z strict,
  wersjonowanymi deskryptorami offset/size;
- UI pokazuje jawne role i zadeklarowane `source.relativePath`, nie zgaduje
  plikow ani provenance;
- intake i batch pozostaja exact JSON z WASM i sa dostepne jako lokalne
  `JSON_REPORT` artifacts;
- deferred/invalid sa jawnymi wynikami, non-humanoid/static pozostaja
  `DEFERRED_M7_V5`, a UI nie oglasza M7 DONE;
- realny browser Worker test obejmuje local File -> Worker -> public web-WASM
  -> M7 JSON, transfer, deterministycznosc i brak base64;
- synthetic-owned fixture jest tylko testem kodu, nie Meshy acceptance.

Wynik 2026-07-14: `DONE_REVIEW_CLEAN`. Studio ma typowane validate/intake/batch,
kanoniczny WASM validation gate, jeden transferable blob, exact hash-verified
JSON artifacts i lokalny panel bez M7 DONE claim. Testy obejmuja 16/16 unit/App
oraz 3/3 real Chrome Worker/web-WASM, w tym READY 1+2, deferred, invalid,
determinizm, stale-response guards i izolacje statusu M6. Dwa niezalezne
finalne review zakonczyly sie P1=0/P2=0.

### 3.4 S1-V5R - canonical result workspace

`S1-V5R` jest completion sub-slice'em S1-V5. Pokazuje czlowiekowi realny wynik
istniejacego pipeline'u bez zmiany konwertera, inventory ani gate M7-V5.

Definition of Done:

- po `MODEL_PACKAGE_BUILT` UI pokazuje exact presentation fields z
  `reportJson`, `summaryJson` i `manifestJson` zwroconych przez Worker;
- widoczne sa co najmniej: canonical status, geometria, animacja, tekstura,
  resrefy, appended 2DA row oraz rozmiary/inventory artefaktow;
- brak lub zly typ wymaganego pola jest jawnym bledem; UI nie wstawia
  fikcyjnego zera, `UNKNOWN` ani inferred PASS;
- `WORKING` jest indeterminate, bo Worker nie raportuje etapow ani procentow;
- zmiana inputu usuwa snapshot, readback i artefakty, a stale inspect/build
  response nie moze odtworzyc starego wyniku;
- glowny wynik pojedynczego modelu jest widoczny przed zaawansowanym panelem
  korpusu M7;
- etykieta rozroznia canonical structural output od pozniejszego Aurora/NWN
  runtime acceptance;
- testy uzywaja tylko realnych JSON-ow/bytes zwracanych przez Worker; fixture
  synthetic-owned pozostaje wylacznie testem kodu.

Wynik 2026-07-14: `DONE_REVIEW_CLEAN`. Produkcyjny App pokazuje strict,
atomowy snapshot exact Worker/WASM outputu: geometrie, animacje, teksture,
resrefy, appended 2DA row, HAK resources i byte identities. Widok jawnie
utrzymuje `OPEN_M6`, wynik jest przed viewportami i M7, a zmiana inputu usuwa
snapshot oraz downloady. Testy obejmuja 55/55 unit/App oraz 4/4 real Chrome
App+Worker/web-WASM. Dwa niezalezne finalne review: P1=0/P2=0.

### 3.1 Odroczony input gate modeli Meshy

Modele Meshy nie sa teraz wymagane do rozpoczecia ani kontynuowania kodu.
Orkiestrator nie szuka ich, nie generuje, nie pobiera i nie zastepuje
syntetykami podczas first pass. Gate zostaje otwarty dopiero po:

1. pierwszej implementacji aktualnie implementowalnych slice'ow M7/S1;
2. wspolnej fazie testow i stabilizacji ich kontraktow;
3. gotowym interfejsie local-file -> Worker -> canonical WASM -> artifact;
4. jawnym dostarczeniu i zatwierdzeniu przez wlasciciela trzech oryginalnych
   eksportow Meshy.

Po otwarciu gate modele sluza do M7-V5, realnego browser E2E i finalnego
acceptance. Do tego czasu ich brak raportujemy jako `DEFERRED_INPUT_GATE`, nie jako
`BLOCKED`, `current_problem` ani powod do zatrzymania implementacji.

```yaml
ready_first_pass_slices:
  - M7-V1
  - M7-V2
  - M7-V3
  - M7-V4
  - S1-V1
  - S1-V2
  - S1-V3
  - S1-V4
  - S1-V5
deferred_input_gate:
  applies_to: [M7-V5, M7_REAL_E2E, M7_FINAL_ACCEPTANCE]
  requires: [three_original_meshy_glbs, approved_art_direction]
```

## 4. Prog rozpoczecia fazy testowej

`wiekszosc vertical slices` oznacza co najmniej `6/10` slice'ow z tabeli,
ktore maja pierwsza implementacje polaczona w jednym worktree. Do progu musza
nalezec co najmniej:

- `M7-V1` i `M7-V3`;
- `S1-V1`, `S1-V2` i `S1-V5`;
- jeden z `S1-V3` albo `S1-V4`.

Pelna faza testowa moze zaczac sie po osiagnieciu tego progu. Preferowany
moment to zakonczenie wszystkich aktualnie implementowalnych slice'ow, ale
testow nie wolno przesuwac poza pierwszy spojny przebieg implementacyjny.

## 5. Co robimy podczas implementation-first

Podczas pierwszej implementacji:

- piszemy kod pionowo od wejscia do rzeczywistego outputu;
- integrujemy slice z istniejacymi publicznymi API zamiast budowac atrapy;
- zapisujemy przy kazdym slice odroczony ledger testow: happy path, bledy,
  granice i oczekiwane artefakty;
- dopuszczamy tylko minimalne kontrole techniczne potrzebne do dalszego
  skladania: parser TypeScript, `cargo check`, typecheck albo pojedynczy smoke
  uruchamiany wtedy, gdy bez niego nie da sie bezpiecznie kontynuowac;
- nie uruchamiamy po kazdym slice pelnego `cargo test --workspace`, clippy,
  wasm-pack, macierzy browserowej, niezaleznego review ani proof gates;
- nie zatrzymujemy nastepnego slice tylko po to, aby rozbudowac pokrycie
  poprzedniego.

Minimalna kontrola nie jest dowodem zakonczenia slice i nie moze zmienic jego
statusu na DONE.

## 6. Odroczona faza integracji i testow

Po osiagnieciu progu orkiestrator wykonuje jedna wspolna fale:

1. domkniecie ledgerow testowych dla zaimplementowanych slice'ow;
2. testy jednostkowe i negatywne Rust core/WASM;
3. testy Worker protocol, transferu bajtow i obslugi bledow;
4. testy UI dla file pickera, stanow, provenance, walidacji i downloadow;
5. integracyjny local-file -> Worker -> WASM -> HAK/report flow;
6. `cargo fmt --all --check`, clippy, workspace tests i build wasm32;
7. frontend typecheck/build oraz wymagane testy komponentow/integracji;
8. niezalezne review implementacji i poprawki P1/P2;
9. dopiero po realnych wejsciach Meshy: trzy proof packety M7;
10. finalna akceptacja zewnetrzna w osobnej pozniejszej fazie, bez
    modyfikowania Aurory/NWN ani ich konfiguracji.

Nie wolno uznac zielonego waskiego smoke'a za pelna bramke. Nie wolno tez
oznaczyc M7/S1 jako DONE przed wykonaniem ich koncowych testow i wymaganych
artefaktow.

## 7. Zasady commitow i review

- Przed progiem `6/10` dopuszczalne sa robocze, niespublikowane zmiany w
  worktree; nie tworzymy fikcyjnych commitow `DONE`.
- Pierwszy checkpoint powstaje po spojnej implementacji wiekszosci i przejsciu
  wspolnej fazy testowej.
- Review jest odroczone razem z pelna bramka testowa, aby oceniac polaczone
  pionowe przeplywy zamiast izolowanych fragmentow.
- Koncowy commit i push wymagaja zielonych bramek dla calego objetego zakresu.

## 8. Raportowanie stanu

Przed progiem raportujemy osobno:

```yaml
implementation_wave:
  inventory_total: 10
  first_pass_complete: 0
  test_phase_threshold: 6
  full_test_phase: DEFERRED
  review_phase: DEFERRED
  real_meshy_execution: DEFERRED_INPUT_GATE
```

Status `first_pass_complete` oznacza tylko obecna implementacje w worktree.
Nie oznacza PASS, DONE ani runtime acceptance.
