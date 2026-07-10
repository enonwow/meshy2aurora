# architektura-meshy2aurora-codex.md

Data: 2026-07-10 | Autor: Codex | Status: AKTYWNY SZKIELET ARCHITEKTURY

## Cel

`meshy2aurora` jest samodzielnym konwerterem:

```text
Meshy GLB (FBX deferred after MVP)
  -> aplikacja webowa Meshy2Aurora Studio
  -> Rust/WASM converter
  -> Aurora/NWN MDL + 2DA + HAK
  -> pobrany plik HAK + report
  -> NWN EE Toolset/gra
```

`C:\Projects\aurora-web` moze byc czytany jako reference-only, ale nie jest czescia runtime, testow ani procesu walidacji `meshy2aurora`.

## Granice projektu

```yaml
in_scope:
  - "wlasny parser MDL"
  - "wlasny binary MDL writer jako format docelowy gry"
  - "opcjonalny ASCII MDL/debug dump tylko do czytelnych snapshotow i diagnostyki"
  - "wlasny 2DA reader/writer that automatically updates a user-selected existing table"
  - "wlasny ERF/HAK reader/writer"
  - "GLB-only ingest z Meshy for MVP"
  - "normalizacja osi/skali"
  - "mapowanie szkieletu, skin weights i animacji"
  - "bake/konwersja tekstur do TGA/TXI"
  - "niedestrukcyjna inspekcja i edycja materialow/tekstur przed eksportem"
  - "wygenerowany HAK i modul/proof path dla NWN EE"
out_of_scope_for_dependency:
  - "importy z C:\\Projects\\aurora-web"
  - "subprocess CLI z C:\\Projects\\aurora-web"
  - "aurora-web jako oracle albo validator"
  - "commitowanie retail/CEP assetow do repo"
```

## Stack rdzenia

Status: POTWIERDZONE decyzje `D11` i `D12` z 2026-07-10.

```yaml
core_language: "Rust 1.96.1 (pinned toolchain)"
package_manager: "Cargo"
tests: "cargo test --workspace"
build: "cargo build --workspace"
product_delivery: "web application, local-first in a browser"
web_ui: "React + TypeScript + Vite"
viewport: "Three.js"
wasm_adapter: "wasm-bindgen over a Rust core with no DOM dependency"
file_io: "user-selected File/Blob input and Blob download output"
background_work: "Web Worker for conversion and packaging"
server_mvp: "none; a backend is a separate later decision for protected Meshy API credentials or collaboration"
initial_hosting: "GitHub Pages static files only"
supported_browsers: "current desktop Chromium, Firefox and Safari; capability fallbacks are required"
binary_io: "Rust standard library; checked little-endian reader owned by meshy2aurora"
glb_library: "Rust crate gltf; exact version locked with Cargo.lock in M2"
studio_ui: "S1 after M6 native proof; the WASM boundary is established in M1A"
```

Uzasadnienie: rdzen pracuje na niezaufanych, binarnych zasobach MDL/MDX/ERF i musi dawac identyczny wynik w przegladarkach na kazdej wspieranej platformie. Rust daje kontrolowane granice pamieci i jeden kontrakt TDD; WebAssembly wystawia ten sam rdzen aplikacji webowej bez uzalezniania implementacji od `aurora-web`.

MVP nie wysyla modelu ani HAK-a na serwer. Przegladarka moze czytac tylko pliki wybrane przez uzytkownika i musi miec fallback do standardowego wyboru pliku; wynikowy HAK i raport powstaja jako `Blob` do pobrania. Sekret Meshy API nie moze trafic do JavaScript bundle - ewentualny backend dla API jest osobnym, pozniejszym etapem.

## Moduly

```yaml
modules:
  wasm_adapter:
    responsibility:
      - "accept Uint8Array/ArrayBuffer from JavaScript"
      - "call pure m2a-core operations"
      - "return versioned JSON reports and output bytes"
      - "never access DOM, browser filesystem or network"
    proposed_path: "crates/m2a-wasm/src"

  web_app:
    responsibility:
      - "file selection, drag/drop and download UX"
      - "run conversion in a Web Worker"
      - "render Source Preview and Aurora Preview"
      - "show diagnostics and generated reports"
    proposed_path: "apps/studio-web/src"

  config:
    responsibility:
      - "validate user-selected files and project settings"
      - "keep browser configuration serializable"
      - "never require retail/CEP assets for unit tests"
    proposed_path: "crates/m2a-core/src/config"

  erf_hak:
    responsibility:
      - "read ERF/HAK V1.0"
      - "write generated HAK"
      - "lookup resources by resref/type"
    proposed_path: "crates/m2a-core/src/erf"

  two_da:
    responsibility:
      - "parse 2DA"
      - "write 2DA"
      - "automatically apply the confirmed row edit to a user-selected existing appearance.2da"
      - "never ship a retail base table in the repository"
      - "never assume HAK table merge without Aurora First and runtime proof"
    proposed_path: "crates/m2a-core/src/two_da"

  mdl:
    responsibility:
      - "parse binary MDL enough for reference inspection"
      - "write binary MDL as native game output"
      - "write/read MDX data according to resolved Q2 policy"
      - "optionally emit deterministic ASCII/debug dump for snapshots"
    proposed_path: "crates/m2a-core/src/mdl"

  glb:
    responsibility:
      - "read Meshy GLB"
      - "extract mesh, materials, textures, bones, skin, animations"
      - "normalize axes and scale"
    proposed_path: "crates/m2a-core/src/glb"

  conversion:
    responsibility:
      - "map Meshy data to Aurora node/model contract"
      - "apply geometry budget gates"
      - "prepare animation names/events"
    proposed_path: "crates/m2a-core/src/conversion"

  textures:
    responsibility:
      - "inspect material -> primitive -> texture -> image and UV links"
      - "keep source PBR state separate from Aurora target state"
      - "apply declared non-destructive texture recipes from m2a.project.json"
      - "bake PBR/diffuse into Aurora-friendly TGA"
      - "emit optional TXI"
    proposed_path: "crates/m2a-core/src/textures"

  project_manifest:
    responsibility:
      - "store versioned conversion settings without modifying the source GLB"
      - "store material/texture recipes, output resrefs and validation policy"
      - "record source hashes so stale derived textures are detectable"
    proposed_path: "crates/m2a-core/src/project"

  proof:
    responsibility:
      - "build generated HAK bytes"
      - "produce a downloadable proof manifest and manual NWN EE runbook"
      - "for creature proof: automatically generate or losslessly update a target-specific UTC blueprint"
      - "for creature proof: automatically place the UTC through a generated module/GIT instance"
      - "never treat one fixed UTC as universal for every creature or asset type"
      - "use UTP/UTI or another confirmed template family for non-creature asset types"
    proposed_path: "crates/m2a-core/src/proof"
```

## Przeplywy aplikacji webowej

```yaml
web_actions:
  inspect_hak:
    input: "user-selected .hak File"
    purpose: "read-only reference inspection"

  inspect_mdl:
    input: "user-selected .mdl File"
    purpose: "parser gate and debug"

  convert:
    input: "user-selected .glb File + project settings"
    output: "in-memory native binary MDL/MDX policy/2DA/textures/manifest"

  pack_hak:
    input: "in-memory converted asset"
    output: "downloadable .hak Blob"

  proof_pack:
    output: "downloadable proof manifest + manual NWN EE runbook"
```

## TDD gates

```yaml
test_layers:
  unit:
    - "synthetic ERF/HAK read/write fixture"
    - "synthetic 2DA parse/write fixture"
    - "synthetic minimal MDL parser fixture"
    - "GLB axis/UV probe fixtures"
    - "synthetic GLB material -> primitive -> image inspection fixture"
    - "texture recipe -> deterministic TGA output fixture"

  integration_optional:
    - name: "CEP c_kocrachn read-only inspection"
      env_required: "M2A_CEP_CORE1_HAK"
      rule: "skip if env missing; never commit CEP asset"

  manual_proof:
    - "install generated HAK into Documents\\Neverwinter Nights\\hak"
    - "open generated/minimal test module in NWN EE Toolset"
    - "place generated creature"
    - "capture screenshot and proof notes"
```

## Otwarte i profilowo rozstrzygniete decyzje formatu

```yaml
blocking_questions:
  L1_binary_mdl_writer_contract:
    source: "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md Q1"
    impact: "decides minimal binary MDL fields/sections needed for M1 writer"
resolved_for_profile_A:
  L2_mdx_policy:
    source: "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md Q2"
    decision: "one resource type 2002 containing binary MDL followed by the appended MDX block; no separate type 2003 resource"
    evidence: "cep3_core1.hak has 3517 type-2002 entries and zero type-2003 entries; c_kocrachn MDL size equals 12 + p_start_mdx + size_mdx"
    final_gate: "own readback plus NWN EE Toolset/game proof"
  L3_bind_pose_preview:
    source: "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md Q3"
    decision: "the audited NwnExplorer version renders the base/rest pose at time 0.0 and does not auto-play cpause1"
    caveat: "this confirms that viewer version, not the NWN EE runtime"

```

## Aktualny proof baseline

```yaml
proof_baseline:
  primary: "NWN EE Toolset/gra"
  secondary_optional: "aurora-web only as external consumer of a standard generated HAK"
  reference_creature:
    resref: "c_kocrachn"
    role: "technical reference/proxy"
    source_policy: "read-only, env-gated, not committed"
  generated_content:
    required: true
    includes:
      - "m2a_*.mdl as native binary MDL"
      - "m2a_*.mdx only if Q2 requires separate resource"
      - "m2a_*.tga or .txi"
      - "appearance.2da row/full file"
      - "m2a_*.hak"
      - "m2a_*.utc for creature blueprint proof"
      - "generated module GIT creature instance for placement proof"
```

## Status

Ten dokument zastapil potrzebe traktowania `aurora-web-architektura-codex.md` jako architektury `meshy2aurora`. `aurora-web-architektura-codex.md` zostaje tylko mapa referencyjna osobnego projektu.
