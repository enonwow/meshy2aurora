# audyt-dokumentacji-plan-2026-07-09-codex.md

Data: 2026-07-09 | Autor: Codex | Status: PLAN NAPRAWCZY PO AUDYCIE DOKUMENTOW

## 0. Zakres audytu

Audyt obejmuje dokumenty w:

```yaml
repo: "C:\\Projects\\meshy2aurora"
documentation: "C:\\Projects\\meshy2aurora\\documentation"
markdown_files_count: 48
git_status: "documentation/ untracked; brak pierwszego commita"
code_scaffold:
  package_json: false
  src_dir: false
  tests_dir: false
wrong_docs_path:
  path: "C:\\Users\\enonw\\Documents\\meshy2aurora\\documentation"
  exists: false
```

Najwazniejszy wniosek: dokumentacja zawiera duzo wartosciowej wiedzy, ale nie ma jeszcze jednego obowiazujacego "source of truth" dla aktualnego kierunku. Stare dokumenty nadal opisuja `aurora-web` jako target/proof, a nowsze decyzje D6/D7 mowia juz o samodzielnym projekcie `meshy2aurora`.

Aktualizacja po poprawkach 2026-07-09: rozjazdy zostaly oznaczone w dokumentach statusami `REFERENCE-ONLY`, `SUPERSEDED` albo `HISTORYCZNE`; powstal dokument `architektura-meshy2aurora-codex.md`; `PROJECT_RULES.md`, `README.md`, `reguly-dokumentacji-cloud.md` i `decyzje-i-zadania-cloud.md` wskazuja teraz aktywny kierunek standalone.

## 1. Obowiazujace zasady po korekcie

Te zasady powinny byc traktowane jako nadrzedne wobec starszych dokumentow:

```yaml
rules:
  documentation_home:
    status: "POTWIERDZONE"
    path: "C:\\Projects\\meshy2aurora\\documentation"
    note: "Wszystkie dokumenty projektu trafiaja tutaj."
  aurora_first:
    status: "POTWIERDZONE"
    order:
      - "C:\\Projects\\New Folder"
      - "lokalne pliki gry/CEP/NWN EE jako read-only reference"
      - "C:\\Projects\\aurora-web jako read-only reference"
      - "Internet jako uzupelnienie"
    note: "Bez zgadywania. Hipoteza nie jest podstawa implementacji bez testu/proofu."
  standalone:
    status: "POTWIERDZONE"
    rule: "meshy2aurora nie uzywa aurora-web jako dependency, CLI, oracle, validator, fixture source ani runtime/test base."
    own_modules:
      - "MDL parser"
      - "binary MDL writer"
      - "optional ASCII/debug dump"
      - "2DA reader/writer"
      - "ERF/HAK reader/writer"
      - "proof HAK/module generator"
  tdd:
    status: "POTWIERDZONE"
    rule: "Najpierw test/gate, potem minimalna implementacja, potem refactor/proof."
  proof_policy:
    status: "POTWIERDZONE"
    primary: "NWN EE Toolset/gra"
    secondary_optional: "aurora-web jako zewnetrzny konsument standardowego HAK-a, poza testami meshy2aurora"
  c_kocrachn:
    status: "POTWIERDZONE"
    role: "techniczny proxy/crash-test dummy dla creature pipeline"
    not_role: "nie jest assetem The Last City i nie jest dowodem dzialania meshy2aurora"
```

## 2. Rozjazdy krytyczne

```yaml
rozjazdy:
  RZ1_aurora_web_target_vs_standalone:
    severity: "P0"
    active_rule: "D7 standalone"
    conflict_files:
      - "C:\\Projects\\meshy2aurora\\documentation\\wymagania-startowe-cloud.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\aurora-pipeline-odpowiedz-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\pliki-referencyjne-odpowiedz-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\implementacja-m1-odpowiedz-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\koncepcja-meshy-cloud.md"
    problem: "Starsze dokumenty zakladaja proof, CLI albo runtime przez aurora-web."
    decision_needed: "Oznaczyc stare sciezki jako HISTORYCZNE/SUPERSEDED. Nie kasowac wiedzy, ale nie traktowac jako planu implementacji."

  RZ2_oracle_i_proof:
    severity: "P0"
    active_rule: "proof primary = NWN EE Toolset/gra"
    conflict_files:
      - "C:\\Projects\\meshy2aurora\\documentation\\aurora-animacje-odpowiedz-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\c-kocrachn-kontrakt-wizualny-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\pliki-referencyjne-odpowiedz-codex.md"
    problem: "Czesc dowodow i screenshotow pochodzi z aurora-web. To dobre read-only reference, ale nie dowod dzialania meshy2aurora."
    decision_needed: "W kazdym dokumencie referencyjnym dopisac status: aurora-web = reference-only, nie oracle/proof."

  RZ3_binary_mdl_writer_contract:
    severity: "P0"
    source_file: "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md"
    problem: "Po korekcie 2026-07-09 nie uzywamy ASCII MDL jako runtime shortcut. Trzeba doprecyzowac minimalny binary MDL writer: sekcje, offsety, controllery, skin, animacje."
    blocking_question: "Q1 engine-mdl-pytania-cloud.md"

  RZ4_mdx_embedded_vs_separate_resource:
    severity: "P0"
    source_file: "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md"
    problem: "Runbook wymienia osobny MDX, ale binary c_kocrachn ma p_start_mdx/size_mdx w naglowku."
    blocking_question: "Q2 engine-mdl-pytania-cloud.md"

  RZ5_statusy_otwarte_po_odpowiedziach:
    severity: "P1"
    files:
      - "C:\\Projects\\meshy2aurora\\documentation\\mdl-2da-hak-pytania-cloud.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\zakres-dokumentacji-referencyjnej-cloud.md"
    problem: "Pliki maja Status: OTWARTE mimo istnienia odpowiedzi/dokumentow."
    fix: "Cloud powinien zamknac status albo dopisac nowe Q."

  RZ6_c_kocrachn_vs_last_city:
    severity: "P1"
    files:
      - "C:\\Projects\\meshy2aurora\\documentation\\c-kocrachn-kontrakt-wizualny-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\meshy-przygotowanie-modelu-cloud.md"
    problem: "c_kocrachn jest wzorcem technicznym, nie decyzja artystyczna The Last City."
    fix: "Rozdzielic sample techniczny od przyszlego oryginalnego assetu TLC."

  RZ7_repo_bez_wersjonowania:
    severity: "P1"
    problem: "46 plikow dokumentacji jest niezwersjonowane."
    fix: "Pierwszy commit dokumentacji przed wiekszymi zmianami."
```

## 3. Luki architektury i technologii

### 3.1 Architektura standalone

W audycie wykryto brak dokumentu, ktory opisuje architekture samego `meshy2aurora`. Po poprawkach 2026-07-09 powstal `architektura-meshy2aurora-codex.md`. Istniejacy `aurora-web-architektura-codex.md` mapuje osobny projekt i jest tylko referencja.

Potrzebny dokument:

```yaml
architecture_doc:
  path: "C:\\Projects\\meshy2aurora\\documentation\\architektura-meshy2aurora-codex.md"
  status: "UTWORZONE 2026-07-09"
  required_modules:
    - "cli entrypoint"
    - "config/env resolver"
    - "ERF/HAK reader"
    - "ERF/HAK writer"
    - "2DA reader"
    - "2DA writer/merger"
    - "binary MDL parser"
    - "binary MDL writer"
    - "optional ASCII/debug dump"
    - "GLB ingest"
    - "axis/scale normalizer"
    - "mesh decimation gate"
    - "texture baker/converter"
    - "animation mapper"
    - "proof HAK/module builder"
    - "manual NWN EE proof runbook"
    - "test fixtures/gates"
```

### 3.2 Stack technologiczny nie jest w pelni zamkniety

Dokumenty wskazuja Node.js + TypeScript, ale uzasadnienie czesciowo pochodzi ze starego zwiazku z `aurora-web`.

```yaml
technology_state:
  proposed:
    runtime: "Node.js >= 22"
    language: "TypeScript 5.9"
    package_manager: "npm"
    tests: "jest + ts-jest"
    glb_lib: "@gltf-transform/core"
  status: "REKOMENDACJA/DECYZJA D3, ale wymaga potwierdzenia po D7"
  gap: "Brak package.json, tsconfig, test runnera, struktury src/tests."
  decision_needed:
    id: "P-tech"
    question: "Czy akceptujemy Node.js + TypeScript + Jest jako stack standalone?"
```

### 3.3 Brak formalnego podzialu linii pracy

Obecnie mieszaja sie dwie linie:

```yaml
workstreams:
  technical_pipeline:
    purpose: "parser/emiter/2DA/HAK/proof"
    reference: "c_kocrachn jako proxy techniczny"
    proof: "NWN EE Toolset/gra"
  creative_sample:
    purpose: "obrazy 2D -> Meshy GLB -> pierwszy potwor testowy"
    reference: "screenshoty lokalne w sample-2d/_reference/c_kocrachn"
    proof: "dopiero po przejsciu przez technical_pipeline"
```

Potrzebna decyzja: nie robic promptow ani obietnic "100% dziala z animacjami", dopoki technical pipeline nie ma parsera/emittera i proofu w NWN EE.

## 4. Luki formatow i danych

```yaml
format_gaps:
  binary_mdl_writer_contract:
    status: "BLOKUJE M1"
    source: "engine-mdl-pytania-cloud.md Q1"
    consequence: "bez doprecyzowania writer moze wygenerowac MDL nieczytelny dla Toolset/gry"

  mdx_policy:
    status: "BLOKUJE M3"
    source: "engine-mdl-pytania-cloud.md Q2"
    consequence: "nie wiadomo, czy HAK ma zawierac osobny .mdx"

  bind_pose_reference:
    status: "BLOKUJE JAKOSC SAMPLE"
    source: "engine-mdl-pytania-cloud.md Q3"
    consequence: "screenshot z nwnexplorer moze byc bind pose albo klatka animacji"

  appearance_2da_retail:
    status: "NIEPELNE"
    source: "aurora-2da-creature-codex.md"
    gap: "Brak potwierdzonego retail appearance.2da / wzorcowego wiersza potwora."

  key_bif_reader:
    status: "DO ZAPLANOWANIA"
    gap: "Jesli potrzebujemy vanilla resources poza HAK, trzeba miec wlasny reader albo jawny export read-only."

  texture_pipeline:
    status: "CZESCIOWO USTALONE"
    confirmed_mvp: "TGA type 2, 24/32 bpp; opcjonalny TXI"
    gap: "Brak implementacji baker/converter i limitow dla finalnego runtime NWN EE."

  animation_contract:
    status: "CZESCIOWO USTALONE"
    confirmed: "klipy referencyjne c_kocrachn/c_Horror w dokumentacji"
    gap: "Brak ostatecznej mapy nazw animacji Meshy -> NWN oraz polityki eventow."
```

## 5. Luki implementacyjne

```yaml
implementation_gaps:
  repo_scaffold:
    status: "BRAK"
    missing:
      - "package.json"
      - "tsconfig.json"
      - ".gitignore"
      - "src/"
      - "tests/"
      - "README implementacyjny"

  core_modules:
    status: "BRAK"
    missing:
      - "HAK/ERF read"
      - "HAK/ERF write"
      - "binary MDL parse"
      - "binary MDL write"
      - "optional ASCII/debug dump"
      - "2DA read/write"
      - "GLB ingest"
      - "texture output"
      - "proof pack builder"

  tests:
    status: "BRAK"
    required_first:
      - "synthetic ERF/HAK fixture"
      - "synthetic 2DA fixture"
      - "synthetic minimal MDL fixture"
      - "env-gated integration test for C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\cep3_core1.hak"

  generated_assets:
    status: "BRAK"
    missing:
      - "sample-2d/_reference/c_kocrachn screenshots"
      - "sample-2d/koc01 generated images"
      - "sample-3d/m2a_koc01 Meshy GLB"
      - "generated m2a_*.hak"
      - "generated test module"
```

## 6. Dokumenty do oznaczenia jako aktywne/historical

### Aktywne teraz

```yaml
active_docs:
  rules:
    - "C:\\Projects\\meshy2aurora\\documentation\\PROJECT_RULES.md"
    - "C:\\Projects\\meshy2aurora\\documentation\\reguly-dokumentacji-cloud.md"
  decisions:
    - "C:\\Projects\\meshy2aurora\\documentation\\decyzje-i-zadania-cloud.md"
    - "C:\\Projects\\meshy2aurora\\documentation\\cel-projektu-cloud.md"
    - "C:\\Projects\\meshy2aurora\\documentation\\kierunek-implementacji-cloud.md"
  blockers:
    - "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md"
    - "C:\\Projects\\meshy2aurora\\documentation\\standalone-odpowiedz-codex.md"
  references:
    - "C:\\Projects\\meshy2aurora\\documentation\\aurora-mdl-format-codex.md"
    - "C:\\Projects\\meshy2aurora\\documentation\\aurora-animation-system-codex.md"
    - "C:\\Projects\\meshy2aurora\\documentation\\aurora-2da-creature-codex.md"
    - "C:\\Projects\\meshy2aurora\\documentation\\aurora-hak-erf-codex.md"
    - "C:\\Projects\\meshy2aurora\\documentation\\ekosystem-narzedzia-codex.md"
```

### Historyczne / do oznaczenia jako superseded

```yaml
superseded_or_reference_only:
  target_aurora_web_old:
    reason: "stary target/proof przed D7"
    files:
      - "C:\\Projects\\meshy2aurora\\documentation\\wymagania-startowe-cloud.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\aurora-pipeline-odpowiedz-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\pliki-referencyjne-odpowiedz-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\implementacja-m1-odpowiedz-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\koncepcja-meshy-cloud.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\koncepcja-meshy-odpowiedz-codex.md"
  aurora_web_architecture:
    reason: "wartosciowa mapa referencyjna, ale nie architektura meshy2aurora"
    files:
      - "C:\\Projects\\meshy2aurora\\documentation\\aurora-web-architektura-codex.md"
```

## 7. Plan naprawczy

### Faza 0: Zamrozenie i porzadkowanie dokumentacji

Cel: zatrzymac rozjazd przed pisaniem kodu.

```yaml
phase_0:
  owner: "Codex + Cloud"
  priority: "P0"
  tasks:
    - id: "F0.1"
      task: "Uznac ten audyt za aktualna mape rozjazdow."
      output: "audyt-dokumentacji-plan-2026-07-09-codex.md"
    - id: "F0.2"
      task: "Zaktualizowac PROJECT_RULES.md o doprecyzowanie: aurora-web read-only reference, nie dependency/oracle/proof."
      output: "PROJECT_RULES.md"
    - id: "F0.3"
      task: "Dopisac do decyzje-i-zadania-cloud.md sekcje 'aktywny plan po audycie'."
      output: "decyzje-i-zadania-cloud.md"
    - id: "F0.4"
      task: "Oznaczyc stare dokumenty aurora-web jako superseded/reference-only."
      output: "naglowki/statusy w plikach historycznych"
    - id: "F0.5"
      task: "Pierwszy commit dokumentacji."
      output: "git commit zawierajacy documentation/"
```

### Faza 1: Zamkniecie decyzji blokujacych

```yaml
phase_1:
  owner: "Codex"
  priority: "P0"
  tasks:
    - id: "F1.1"
      task: "Odpowiedziec na engine-mdl-pytania-cloud.md Q1-Q3."
      output: "engine-mdl-odpowiedz-codex.md"
    - id: "F1.2"
      task: "Dostarczyc retail appearance.2da albo udokumentowac twardy blocker."
      output: "aktualizacja aurora-2da-creature-codex.md albo osobna odpowiedz"
    - id: "F1.3"
      task: "Utrzymywac architektura-meshy2aurora-codex.md jako aktywny dokument architektury."
      output: "moduly, granice, formaty I/O, test gates"
    - id: "F1.4"
      task: "Potwierdzic stack P-tech."
      output: "Node/TS/Jest zatwierdzone albo alternatywa"
```

### Faza 2: Szkielet repo i TDD gates

```yaml
phase_2:
  owner: "implementacja"
  priority: "P1"
  prerequisites:
    - "P-tech"
    - "F0.2"
    - "F1.1 minimum Q1"
  tasks:
    - "package.json + tsconfig strict + test runner"
    - ".gitignore"
    - "src/cli"
    - "src/config"
    - "src/erf"
    - "src/two-da"
    - "src/mdl"
    - "tests/fixtures/synthetic"
    - "env-gated tests for external CEP/NWN files"
```

### Faza 3: Minimalna linia parsera

```yaml
phase_3:
  owner: "implementacja"
  priority: "P1"
  goal: "wczytac read-only c_kocrachn z CEP HAK bez kopiowania assetow do repo"
  tasks:
    - "ERF/HAK reader V1"
    - "resource directory lookup"
    - "binary MDL header parser"
    - "node tree parser"
    - "controllers parser"
    - "mesh/MDX data parser"
    - "skin weights parser"
    - "animation blocks parser"
  tests:
    - "synthetic HAK read test"
    - "synthetic binary MDL read test"
    - "integration skip-if-missing M2A_CEP_CORE1_HAK"
```

### Faza 4: Emisja i proof pack

```yaml
phase_4:
  owner: "implementacja"
  priority: "P1"
  branch_by_engine_answer:
    active_after_D9:
      M1_output: "binary MDL writer"
      debug_optional: "ASCII/debug dump only for readable snapshots"
      M3_hak_resources: ["MDL binary", "MDX policy per F1.1", "TGA/TXI", "appearance.2da"]
  tasks:
    - "2DA writer/merger"
    - "HAK writer"
    - "generated m2a_*.hak"
    - "generated or manual minimal module"
    - "NWN EE Toolset/game proof runbook"
```

### Faza 5: Linia sample 2D -> Meshy -> Aurora

```yaml
phase_5:
  owner: "Mateusz + Codex + implementacja"
  priority: "P2"
  prerequisites:
    - "local c_kocrachn screenshots or other accepted technical reference"
    - "phase_4 minimal proof pack path"
  tasks:
    - "sample-2d/_reference/c_kocrachn/front.png"
    - "sample-2d/_reference/c_kocrachn/side.png"
    - "sample-2d/_reference/c_kocrachn/quarter.png"
    - "sample-2d/_reference/c_kocrachn/manifest.yaml"
    - "sample-2d-prompty-codex.md"
    - "OpenAI images in sample-2d/koc01"
    - "Meshy GLB in sample-3d/m2a_koc01"
    - "conversion gates s1-s10"
```

## 8. Czego nie robic teraz

```yaml
do_not_now:
  - "Nie obiecywac '100% dzialajacy c_kocrachn z animacjami' w meshy2aurora, bo parser/emiter/HAK writer jeszcze nie istnieja."
  - "Nie uzywac aurora-web CLI/subprocess/importow w testach lub kodzie meshy2aurora."
  - "Nie traktowac screenshotow z aurora-web jako proofu meshy2aurora."
  - "Nie commitowac retail/CEP assetow do repo."
  - "Nie mieszac The Last City z c_kocrachn; TLC wymaga osobnego oryginalnego kontraktu wizualnego."
  - "Nie generowac finalnych promptow 2D bez lokalnych referencji albo jawnej decyzji, ze robimy tylko placeholder techniczny."
```

## 9. Decyzje do Mateusza

```yaml
mateusz_decisions:
  P4:
    question: "Akceptujesz kierunek M1-M5 po korekcie standalone?"
    recommended: "TAK, ale po F0/F1"
  P-tech:
    question: "Akceptujesz Node.js >=22 + TypeScript 5.9 + Jest/ts-jest dla standalone?"
    recommended: "TAK, bo repo i tak bedzie operowalo na binarnych plikach/GLB/CLI, a TS pasuje do dotychczasowych kompetencji projektu."
  P-proof:
    question: "Pierwszy twardy proof robimy na technicznym proxy c_kocrachn, a The Last City zostaje osobnym etapem artystycznym?"
    recommended: "TAK"
  P-git:
    question: "Robimy pierwszy commit dokumentacji przed kodem?"
    recommended: "TAK, pilne"
```

## 10. Najblizsze kroki

Kolejnosc zalecana:

1. Zaktualizowac `C:\Projects\meshy2aurora\documentation\PROJECT_RULES.md` o doprecyzowanie D7.
2. Dopisac do `C:\Projects\meshy2aurora\documentation\decyzje-i-zadania-cloud.md` aktywny plan po audycie.
3. Stworzyc `C:\Projects\meshy2aurora\documentation\engine-mdl-odpowiedz-codex.md`.
4. Utrzymywac `C:\Projects\meshy2aurora\documentation\architektura-meshy2aurora-codex.md` jako aktywna architekture standalone.
5. Zrobic pierwszy commit dokumentacji.
6. Dopiero potem scaffold kodu i pierwsze testy.

## 11. Krotka diagnoza

Rozjazd nie wynika z jednego bledu technicznego. Wynika z tego, ze projekt zmienil cel:

```yaml
old_goal: "uzyc/obejrzec pipeline przez aurora-web i GLB/proof w przegladarce"
new_goal: "samodzielny konwerter Meshy -> natywny content Aurora: MDL + 2DA + HAK"
```

Dlatego dokumenty `aurora-web` sa nadal przydatne, ale jako mapa porownawcza. Implementacyjny rdzen musi teraz isc osobno: wlasny parser, wlasny writer, wlasny proof w NWN EE.
