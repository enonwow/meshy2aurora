# srodowisko-zakres-odpowiedz-codex.md

Status 2026-07-10: HISTORYCZNE / CALKOWICIE SUPERSEDED przez D11-D12. Rekomendacja Node.js + TypeScript, CLI batch-first i integracja z backendem `aurora-web` nie sa aktywne. Produkt jest aplikacja webowa local-first z rdzeniem Rust/WASM; szczegoly sa w `architektura-web-wasm-codex.md`.
Data: 2026-07-08 | Odpowiada na: srodowisko-zakres-pytania-cloud.md

## Q1: Stack konwertera
Status: HIPOTEZA (decyzja projektowa); POTWIERDZONE uzasadnienie w `C:\Projects\aurora-web`

Decyzja Codexa dla MVP: Node/TypeScript, nie Python. Powod: aurora-web ma juz parser/konwerter MDL->GLB, GLB loader, testy i proof tooling w TypeScript/Node. Python moze byc pomocniczy pozniej do analizy mesh, ale pierwszy dzialajacy przeplyw powinien siedziec blisko aurora-web.

```yaml
decision:
  mvp_stack: "Node.js + TypeScript"
  reason:
    - "reuse aurora-web converter/loader contracts"
    - "same ecosystem as backend scripts and CDP proof tooling"
    - "less impedance mismatch for GLB metadata and tests"
confirmed_reuse_paths:
  converter_cli: "C:\\Projects\\aurora-web\\backend\\scripts\\aurora-mdl-to-glb.ts"
  converter_core: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
  runtime_loader: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
python:
  status: "deferred"
  possible_use: "offline geometry analysis, simplification experiments"
```

## Q2: Forma narzedzia
Status: HIPOTEZA

MVP powinien byc CLI batch-first. Integracja z backendiem aurora-web powinna przyjsc po potwierdzeniu 2-3 realnych eksportow Meshy i proofow runtime.

```yaml
tool_shape:
  mvp:
    interface: "CLI"
    mode: "single file plus batch folder"
    outputs:
      - "normalized.glb"
      - "manifest.json"
      - "proof-request.json"
  later:
    interface: "aurora-web backend integration"
    reason: "requires stable catalog/blob registration contract"
cli_commands_target:
  examples:
    - "meshy2aurora convert --input sample.glb --type placeable --resref zcp_sample --out dist\\zcp_sample"
    - "meshy2aurora convert --input sample.glb --type creature --supermodel c_horror --animation cpause1 --out dist\\cre_sample"
    - "meshy2aurora batch --input samples --out dist"
```

## Q3: Kryteria akceptacji
Status: HIPOTEZA (MVP criteria); POTWIERDZONE dla rodzaju wymaganych proofow w aurora-web

```yaml
definition_of_working_model:
  common:
    - "GLB parses through aurora-web GLTFLoader"
    - "model is visible on screenshot, not placeholder"
    - "scale and axis are stable in area/model preview"
    - "textures/materials render or missing textures are explicitly reported"
    - "proof artifacts are written: summary.json, state.json, PNG"
  static_mvp:
    - "one Meshy static prop/placeable loads as GLB"
    - "no animation required"
    - "expected bounds are sane: non-empty mesh, nonzero pixel area"
  creature_mvp:
    - "one direct creature retargets to existing Aurora clip"
    - "selected animation plays in runtime state"
    - "frame samples show pose change or accepted idle motion"
    - "no visible duplicate driver mesh"
  not_mvp:
    - "full retail parity"
    - "all 1693 creature templates"
    - "all humanoid equipment/weapon animation cases"
    - "placeable light/shadow/VFX parity"
```

## Q4: Podzial pracy
Status: HIPOTEZA

```yaml
roles:
  Codex:
    - "utrzymuje zasade Aurora First"
    - "czyta dekompilacje i aurora-web przed odpowiedzia"
    - "pisze/aktualizuje pliki *-odpowiedz-codex.md"
    - "implementuje kod konwertera i testy TDD, jesli dostanie zadanie implementacyjne"
    - "uruchamia proofy w aurora-web"
  Claude:
    - "pisze *-cloud.md i *-pytania-cloud.md"
    - "zamyka pytania po przeczytaniu odpowiedzi"
    - "utrzymuje spec/plan implementacji po stronie Cloud"
  Mateusz:
    - "dostarcza realne eksporty Meshy"
    - "zatwierdza priorytet MVP"
    - "ocenia wizualna akceptacje screenshot/MP4"
blocked_until:
  - "brak 2-3 lokalnych eksportow Meshy"
  - "brak decyzji: static placeable first vs creature first"
```
