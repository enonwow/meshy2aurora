# status-dokumentacji-web-2026-07-10-codex.md

Data: 2026-07-10 | Autor: Codex | Status: AKTYWNY INDEKS CALOSCI DOKUMENTACJI

## 1. Cel

Ten indeks klasyfikuje wszystkie 69 plikow z `C:\Projects\meshy2aurora\documentation` po decyzjach D11-D14. Ma usunac rozjazd miedzy aktualnym produktem webowym a starszymi planami Node/TypeScript/CLI lub dawnym targetem `aurora-web`.

```yaml
active_product:
  delivery: "browser web application, local-first"
  core: "Rust 1.96.1 compiled to WebAssembly"
  ui: "React + TypeScript + Vite + Three.js"
  mvp_backend: false
  initial_hosting: "GitHub Pages static hosting"
  final_proof: "NWN EE Toolset/game"
authority_order:
  - "PROJECT_RULES.md section 6"
  - "decyzje-i-zadania-cloud.md D11 through D14"
  - "architektura-meshy2aurora-codex.md"
  - "architektura-web-wasm-codex.md"
  - "audyt-gotowosci-startowej-2026-07-10-codex.md"
  - "macierz-gotowosci-wiedzy-codex.md"
  - "mdl-binary-crosswalk-codex.md"
  - "mdx-polityka-codex.md"
  - "animacje-kontrakt-profil-a-codex.md"
  - "hak-2da-gff-crosswalk-codex.md"
  - "korpus-referencyjny-mdl-codex.md"
  - "status-dokumentacji-web-2026-07-10-codex.md"
  - "plan-implementacji-orkiestrator-codex.md"
  - "orchestrator-state.yaml"
rule: "A file below marked HISTORYCZNE or REFERENCE never overrides active_product."
```

`*-cloud.md` and `*-pytania-cloud.md` are not rewritten by Codex under the collaboration rules. Their historical status is recorded here instead.

## 2. AKTYWNE: product contract and live blockers

```yaml
active:
  - "README.md"
  - "PROJECT_RULES.md"
  - "audyt-gotowosci-startowej-2026-07-10-codex.md"
  - "macierz-gotowosci-wiedzy-codex.md"
  - "mdl-binary-crosswalk-codex.md"
  - "mdx-polityka-codex.md"
  - "animacje-kontrakt-profil-a-codex.md"
  - "hak-2da-gff-crosswalk-codex.md"
  - "korpus-referencyjny-mdl-codex.md"
  - "reguly-dokumentacji-cloud.md"
  - "CLOUD_SUPPLEMENT_FORMAT.md"
  - "decyzje-i-zadania-cloud.md"
  - "architektura-meshy2aurora-codex.md"
  - "architektura-web-wasm-codex.md"
  - "status-dokumentacji-web-2026-07-10-codex.md"
  - "plan-implementacji-orkiestrator-codex.md"
  - "orchestrator-state.yaml"
  - "evidence/README.md"
  - "prompt-dla-claude-prototyp-parsera.md"
  - "viewport-walidacja-animacje-plan-codex.md"
  - "przyszle-featurey-studio-codex.md"
  - "engine-mdl-pytania-cloud.md"
  - "engine-mdl-odpowiedz-codex.md"
  - "mdl-2da-hak-pytania-cloud.md"
  - "standalone-pytania-cloud.md"
```

Question files remain active only for unresolved Aurora format facts. They do not choose product technology.

## 3. REFERENCE: facts may be used, product decisions may not

```yaml
reference:
  - "audyt-parsera-blender-nwn-2026-07-09-codex.md"
  - "audyt-repozytoriow-pomocniczych-2026-07-10-codex.md"
  - "aurora-2da-creature-codex.md"
  - "aurora-animacje-odpowiedz-codex.md"
  - "aurora-animation-system-codex.md"
  - "aurora-hak-erf-codex.md"
  - "aurora-mdl-format-codex.md"
  - "aurora-models-animations-audit-2026-07-08.md"
  - "aurora-pipeline-odpowiedz-codex.md"
  - "aurora-web-architektura-codex.md"
  - "c-kocrachn-kontrakt-wizualny-codex.md"
  - "ekosystem-narzedzia-codex.md"
  - "konwersja-meshy-analiza-cloud.md"
  - "konwersja-meshy-odpowiedz-codex.md"
  - "mdl-2da-hak-odpowiedz-codex.md"
  - "meshy-api-cloud.md"
  - "meshy-input-odpowiedz-codex.md"
  - "meshy-przygotowanie-modelu-cloud.md"
  - "neverblender-audyt-2026-07-09-codex.md"
  - "pliki-referencyjne-odpowiedz-codex.md"
  - "repozytoria-pomocnicze-2026-07-09-codex.md"
  - "standalone-odpowiedz-codex.md"
```

## 4. HISTORYCZNE: preserve context, do not implement from them

```yaml
historical:
  - "audyt-dokumentacji-plan-2026-07-09-codex.md"
  - "audyt-rozjazdu-od-pierwotnego-planu-2026-07-09-codex.md"
  - "audyt-stanu-2026-07-08-cloud.md"
  - "aurora-animacje-pytania-cloud.md"
  - "aurora-pipeline-pytania-cloud.md"
  - "cel-projektu-cloud.md"
  - "implementacja-m1-odpowiedz-codex.md"
  - "implementacja-m1-pytania-cloud.md"
  - "kierunek-implementacji-cloud.md"
  - "koncepcja-meshy-cloud.md"
  - "koncepcja-meshy-odpowiedz-codex.md"
  - "koncepcja-meshy-pytania-cloud.md"
  - "konwersja-meshy-pytania-cloud.md"
  - "korpus-testowy-cloud.md"
  - "meshy-input-pytania-cloud.md"
  - "pliki-referencyjne-pytania-cloud.md"
  - "sample-2d-generacja-cloud.md"
  - "sample-foldery-cloud.md"
  - "srodowisko-zakres-odpowiedz-codex.md"
  - "srodowisko-zakres-pytania-cloud.md"
  - "wymagania-startowe-cloud.md"
  - "zakres-dokumentacji-referencyjnej-cloud.md"
```

## 5. Verification

```yaml
inventory:
  audited_files: 69
  active: 25
  reference: 22
  historical: 22
  unclassified: 0
verification_rule:
  - "Before changing product architecture, read this index and the active product contract."
  - "Before using a reference fact, apply Aurora First and record provenance."
  - "Before reopening a historical technology choice, create a new D-numbered decision; do not revive it implicitly."
```
