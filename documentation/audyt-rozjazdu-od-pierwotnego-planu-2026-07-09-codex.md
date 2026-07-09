# audyt-rozjazdu-od-pierwotnego-planu-2026-07-09-codex.md

Data: 2026-07-09 | Autor: Codex | Status: AUDYT POROWNACZY PO KOREKCIE D9

## 0. Punkt odniesienia

Pierwotny plan roboczy, do ktorego porownuje dokumentacje:

```text
model z Meshy.ai
  -> nasz parser/konwerter
  -> natywne zasoby Aurory/NWN
  -> HAK + 2DA dla projektu w Aurorze
```

Po doprecyzowaniu D9 formatem docelowym nie jest ASCII MDL ani GLB dla `aurora-web`, tylko natywny output gry:

```yaml
target_output:
  model: "binary MDL"
  mdx: "embedded albo osobny zasob - do rozstrzygniecia w engine-mdl-pytania-cloud.md Q2"
  tables: "2DA, minimum appearance.2da"
  package: "HAK"
  proof: "NWN EE Toolset/gra"
  aurora_web: "read-only reference albo pozniejszy zewnetrzny konsument HAK-a"
```

## 1. Twardy stan repo

```yaml
repo: "C:\\Projects\\meshy2aurora"
documentation: "C:\\Projects\\meshy2aurora\\documentation"
markdown_files: 48
code_scaffold:
  package_json: false
  src: false
  tests: false
git_status: "documentation/ untracked"
```

Wniosek: rozjazd jest dokumentacyjny i decyzyjny, nie implementacyjny. Nie ma jeszcze kodu, ktory trzeba by odkrecać.

## 2. Jak daleko odjechalismy

Ocena:

```yaml
distance_score:
  active_plan_after_D9:
    score: "1/10 odjazdu"
    meaning: "aktywny kierunek jest zgodny z pierwotnym planem; zostaly tylko szczegoly techniczne binary MDL/MDX"
  full_documentation_corpus:
    score: "6/10 odjazdu"
    meaning: "stare dokumenty historyczne nadal zawieraja sciezki aurora-web/GLB/CDP/ASCII, ale aktywne dokumenty wejscia sa po D9"
  implementation_state:
    score: "0/10 odjazdu w kodzie"
    meaning: "brak kodu, wiec drift nie zdazyl wejsc w implementacje"
```

Najkrotsza diagnoza: rdzen po D9 wrocil na dobra trase, ale dokumentacja ma duzo osadu po trzech bocznych petlach:

1. `aurora-web` jako target/proof zamiast tylko reference,
2. ASCII MDL jako potencjalny runtime/proof zamiast debug dump,
3. `c_kocrachn`/CEP/CDP jako baza dowodu zamiast techniczny proxy i read-only reference.

## 3. Metryki sygnalow w dokumentach

Skan `documentation/`:

```yaml
signals:
  aurora_web_mentions: 398
  cdp_mentions: 24
  ascii_mentions: 73
  binary_mdl_or_mdx_policy_mentions: 49
  meshy_mentions: 420
  hak_2da_mdl_mentions: 325
  explicit_reference_only_or_superseded_markers: 35
  d9_or_native_binary_markers: 12
```

Interpretacja: stary temat `aurora-web` nadal dominuje tekstowo w dokumentach historycznych, ale aktywne markery D7-D9 sa juz obecne w najwazniejszych plikach wejscia: `README.md`, `PROJECT_RULES.md`, `cel-projektu-cloud.md`, `decyzje-i-zadania-cloud.md`, `architektura-meshy2aurora-codex.md`, `engine-mdl-pytania-cloud.md`.

## 4. Porownanie: plan pierwotny vs aktualny aktywny plan

```yaml
original_plan:
  input: "model z Meshy.ai"
  core: "nasz parser/konwerter"
  output: "HAK + 2DA dla Aurory"
  proof: "dzialanie w Aurorze/NWN"

active_plan_after_D9:
  input: "Meshy GLB/FBX"
  core:
    - "GLB ingest"
    - "normalizacja osi/skali"
    - "mapowanie szkieletu/skin/animacji"
    - "binary MDL writer"
    - "2DA writer"
    - "ERF/HAK writer"
  output:
    - "binary MDL"
    - "MDX embedded/separate per Q2"
    - "TGA/TXI"
    - "appearance.2da"
    - "HAK"
  proof: "NWN EE Toolset/gra"
  delta: "zgodne z pierwotnym planem, bardziej doprecyzowane technicznie"
```

Wniosek: aktywny plan po D9 nie jest juz odjechany. Jest tylko bardziej szczegolowy.

## 5. Gdzie naprawde odjechalismy

### 5.1 `aurora-web` zamiast Aurory/NWN

Rozjazd:

```yaml
wrong_direction:
  - "target to aurora-web"
  - "derived GLB"
  - "CDP proof"
  - "runtime settings / source layer"
  - "CLI/subprocess/import z aurora-web"
correct_direction:
  - "aurora-web tylko read-only reference"
  - "native binary MDL + 2DA + HAK"
  - "proof przez NWN EE"
```

Status: czesciowo naprawione przez D7/D8 i naglowki `REFERENCE-ONLY`, ale stary jezyk zostal w wielu dokumentach.

Najbardziej ryzykowne pliki historyczne:

```yaml
files:
  - "C:\\Projects\\meshy2aurora\\documentation\\wymagania-startowe-cloud.md"
  - "C:\\Projects\\meshy2aurora\\documentation\\koncepcja-meshy-cloud.md"
  - "C:\\Projects\\meshy2aurora\\documentation\\implementacja-m1-odpowiedz-codex.md"
  - "C:\\Projects\\meshy2aurora\\documentation\\aurora-pipeline-odpowiedz-codex.md"
  - "C:\\Projects\\meshy2aurora\\documentation\\pliki-referencyjne-odpowiedz-codex.md"
```

### 5.2 ASCII MDL jako runtime shortcut

Rozjazd:

```yaml
wrong_direction:
  - "czy NWN EE czyta ASCII MDL z HAK?"
  - "M1 moze skonczyc sie na emiterze ASCII"
  - "ASCII jako proof/output"
correct_direction:
  - "binary MDL writer jako docelowy output gry"
  - "ASCII tylko debug dump/golden snapshot"
```

Status: naprawione w aktywnych dokumentach przez D9.

Naprawione 2026-07-09:

```yaml
file: "C:\\Projects\\meshy2aurora\\documentation\\cel-projektu-cloud.md"
previous_problem: "mial status OBOWIAZUJACE, ale pipeline mowil 'emisja ASCII MDL'"
fix: "pipeline zmieniony na natywny binary MDL + MDX policy + 2DA + HAK; ASCII tylko debug dump"
```

### 5.3 `c_kocrachn` zaczal udawac cel

Rozjazd:

```yaml
wrong_direction:
  - "c_kocrachn jako prawie produkt"
  - "screenshoty/proof z aurora-web"
  - "sample podporzadkowany temu proxy"
correct_direction:
  - "c_kocrachn jako technical proxy/reference"
  - "wlasny wygenerowany model z Meshy jako cel"
  - "The Last City osobno, z oryginalnym kontraktem artystycznym"
```

Status: w duzej mierze naprawione przez `c-kocrachn-kontrakt-wizualny-codex.md` i decyzje P-proof, ale sample docs nadal mocno go eksponuja.

### 5.4 Zbyt duzo researchu przed kodem

Rozjazd:

```yaml
symptom:
  - "48 plikow markdown"
  - "brak package.json"
  - "brak src/"
  - "brak tests/"
  - "calosc documentation/ untracked"
impact: "projekt zaczal produkowac dokumenty szybciej niz decyzje i testy"
correct_direction:
  - "zamknac minimalny kontrakt M1"
  - "pierwszy commit dokumentacji"
  - "scaffold TDD"
```

Status: nadal otwarte.

## 6. Aktualne luki po powrocie na tor

To sa prawdziwe blokery, juz bez watku ASCII runtime:

```yaml
P0:
  binary_mdl_writer_contract:
    file: "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md"
    question: "jakie minimalne sekcje/pointery/offsety musi zapisac writer binary MDL dla creature?"
  mdx_policy:
    file: "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md"
    question: "MDX embedded w MDL payload czy osobny zasob 2003 w HAK?"
  first_commit:
    question: "documentation/ jest nadal untracked"

P1:
  cel_projektu_stale_fixed_2026_07_09:
    file: "C:\\Projects\\meshy2aurora\\documentation\\cel-projektu-cloud.md"
    status: "NAPRAWIONE"
    note: "pipeline zmieniony na binary MDL + MDX policy + 2DA + HAK"
  tech_stack:
    file: "C:\\Projects\\meshy2aurora\\documentation\\decyzje-i-zadania-cloud.md"
    question: "P-tech niepotwierdzone"
  appearance_2da:
    file: "C:\\Projects\\meshy2aurora\\documentation\\aurora-2da-creature-codex.md"
    question: "retail appearance.2da / wzorcowy wiersz nadal nie domkniety"
```

## 7. Rekomendowany reset operacyjny

Kolejne kroki bez dalszego rozjezdzania:

```yaml
reset_steps:
  1_fix_cel_projektu:
    status: "DONE 2026-07-09"
    action: "zaktualizowano cel-projektu-cloud.md do D9"
    reason: "to dokument oznaczony jako OBOWIAZUJACE, wiec nie moze mowic ASCII runtime"

  2_freeze_history:
    action: "nie edytowac juz masowo starych odpowiedzi; zostawic je jako reference-only"
    reason: "inaczej dokumentacja bedzie rosla bez konca"

  3_first_commit:
    action: "commit dokumentacji"
    reason: "48 plikow untracked to realne ryzyko utraty stanu"

  4_m1_contract:
    action: "odpowiedz na engine-mdl-pytania-cloud.md Q1-Q2"
    reason: "to definiuje binary MDL/MDX writer"

  5_scaffold:
    action: "package.json + src + tests + synthetic fixtures"
    reason: "przestac generowac tylko dokumenty"
```

## 8. Finalna ocena

```yaml
summary:
  czy_odjechalismy: "tak, mocno w dokumentach"
  czy_to_trwale_uszkodzilo_projekt: "nie, bo nie ma jeszcze kodu"
  czy_aktywny_plan_jest_zgodny_z_pierwotnym: "tak, po D9"
  najwieksze_ryzyko_teraz: "stare dokumenty historyczne nadal maja duzo fraz aurora-web/GLB/CDP, ale aktywne dokumenty wejscia sa juz po D9"
  najwazniejsza_korekta: "model z Meshy -> nasz konwerter -> binary MDL/MDX policy + 2DA + HAK"
```

Moja ocena po poprawkach: od pierwotnego planu odjechalismy najdalej w momencie, gdy `aurora-web`, GLB/CDP i ASCII MDL zaczely byc traktowane jako mozliwe sciezki wykonawcze. Po korektach D7-D9 i aktualizacji `cel-projektu-cloud.md` aktywny tor jest spójny. Nadal trzeba zamknac M1 kontraktem binary MDL/MDX i wykonac pierwszy scaffold kodu.
