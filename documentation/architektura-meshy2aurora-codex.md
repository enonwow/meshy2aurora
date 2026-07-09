# architektura-meshy2aurora-codex.md

Data: 2026-07-09 | Autor: Codex | Status: AKTYWNY SZKIELET ARCHITEKTURY

## Cel

`meshy2aurora` jest samodzielnym konwerterem:

```text
Meshy GLB/FBX
  -> meshy2aurora
  -> Aurora/NWN MDL + 2DA + HAK
  -> NWN EE Toolset/gra
```

`C:\Projects\aurora-web` moze byc czytany jako reference-only, ale nie jest czescia runtime, testow ani procesu walidacji `meshy2aurora`.

## Granice projektu

```yaml
in_scope:
  - "wlasny parser MDL"
  - "wlasny binary MDL writer jako format docelowy gry"
  - "opcjonalny ASCII MDL/debug dump tylko do czytelnych snapshotow i diagnostyki"
  - "wlasny 2DA reader/writer/merger"
  - "wlasny ERF/HAK reader/writer"
  - "GLB ingest z Meshy"
  - "normalizacja osi/skali"
  - "mapowanie szkieletu, skin weights i animacji"
  - "bake/konwersja tekstur do TGA/TXI"
  - "wygenerowany HAK i modul/proof path dla NWN EE"
out_of_scope_for_dependency:
  - "importy z C:\\Projects\\aurora-web"
  - "subprocess CLI z C:\\Projects\\aurora-web"
  - "aurora-web jako oracle albo validator"
  - "commitowanie retail/CEP assetow do repo"
```

## Proponowany stack

Status: DO POTWIERDZENIA jako decyzja `P-tech`.

```yaml
runtime: "Node.js >= 22"
language: "TypeScript 5.9"
package_manager: "npm"
tests: "Jest + ts-jest"
glb_library: "@gltf-transform/core"
style: "CLI batch-first"
```

Uzasadnienie: kod bedzie operowal na binarnych zasobach, GLB, manifestach i testach TDD. TypeScript pasuje do istniejacej wiedzy zespolu, ale po D7 nie oznacza to zaleznosci od `aurora-web`.

## Moduly

```yaml
modules:
  cli:
    responsibility:
      - "parse command line"
      - "route convert/inspect/pack/proof commands"
    proposed_path: "src/cli"

  config:
    responsibility:
      - "read env paths"
      - "validate optional local references"
      - "never require retail/CEP assets for unit tests"
    env:
      M2A_NWN_ROOT: "optional local NWN EE root"
      M2A_CEP_CORE1_HAK: "optional read-only CEP HAK path"
    proposed_path: "src/config"

  erf_hak:
    responsibility:
      - "read ERF/HAK V1.0"
      - "write generated HAK"
      - "lookup resources by resref/type"
    proposed_path: "src/erf"

  two_da:
    responsibility:
      - "parse 2DA"
      - "write 2DA"
      - "merge/add generated appearance.2da rows"
    proposed_path: "src/two-da"

  mdl:
    responsibility:
      - "parse binary MDL enough for reference inspection"
      - "write binary MDL as native game output"
      - "write/read MDX data according to resolved Q2 policy"
      - "optionally emit deterministic ASCII/debug dump for snapshots"
    proposed_path: "src/mdl"

  glb:
    responsibility:
      - "read Meshy GLB"
      - "extract mesh, materials, textures, bones, skin, animations"
      - "normalize axes and scale"
    proposed_path: "src/glb"

  conversion:
    responsibility:
      - "map Meshy data to Aurora node/model contract"
      - "apply geometry budget gates"
      - "prepare animation names/events"
    proposed_path: "src/conversion"

  textures:
    responsibility:
      - "bake PBR/diffuse into Aurora-friendly TGA"
      - "emit optional TXI"
    proposed_path: "src/textures"

  proof:
    responsibility:
      - "build generated HAK"
      - "prepare proof module instructions/assets"
      - "produce manifest for manual NWN EE Toolset/game proof"
    proposed_path: "src/proof"
```

## Przeplywy CLI

```yaml
commands:
  inspect_hak:
    example: "meshy2aurora inspect-hak --hak <path> --resref c_kocrachn"
    purpose: "read-only reference inspection"

  inspect_mdl:
    example: "meshy2aurora inspect-mdl --input <model.mdl>"
    purpose: "parser gate and debug"

  convert:
    example: "meshy2aurora convert --input sample.glb --type creature --resref m2a_koc01 --out dist\\m2a_koc01"
    purpose: "Meshy GLB -> generated native binary MDL/MDX policy/2DA/textures/manifest"

  pack_hak:
    example: "meshy2aurora pack-hak --input dist\\m2a_koc01 --out dist\\m2a_koc01.hak"
    purpose: "generated content -> HAK"

  proof_pack:
    example: "meshy2aurora proof-pack --resref m2a_koc01 --hak dist\\m2a_koc01.hak --out proof\\m2a_koc01"
    purpose: "manual NWN EE proof package"
```

## TDD gates

```yaml
test_layers:
  unit:
    - "synthetic ERF/HAK read/write fixture"
    - "synthetic 2DA parse/write fixture"
    - "synthetic minimal MDL parser fixture"
    - "GLB axis/UV probe fixtures"

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

## Otwarte decyzje blokujace architekture

```yaml
blocking_questions:
  L1_binary_mdl_writer_contract:
    source: "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md Q1"
    impact: "decides minimal binary MDL fields/sections needed for M1 writer"

  L2_mdx_policy:
    source: "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md Q2"
    impact: "decides expected resources in generated HAK"

  L3_bind_pose_preview:
    source: "C:\\Projects\\meshy2aurora\\documentation\\engine-mdl-pytania-cloud.md Q3"
    impact: "decides how to trust nwnexplorer screenshots for sample-2d"

  P_tech:
    source: "C:\\Projects\\meshy2aurora\\documentation\\decyzje-i-zadania-cloud.md"
    impact: "final tech stack before repo scaffold"
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
```

## Status

Ten dokument zastapil potrzebe traktowania `aurora-web-architektura-codex.md` jako architektury `meshy2aurora`. `aurora-web-architektura-codex.md` zostaje tylko mapa referencyjna osobnego projektu.
