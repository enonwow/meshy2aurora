# ekosystem-narzedzia-codex.md

Status 2026-07-09: AKTYWNY INWENTARZ z korekta D7-D8. Narzedzia/repo z `aurora-web` sa reference-only; nie sa zaleznoscia ani proof runnerem `meshy2aurora`.
Data: 2026-07-08  
Status: POTWIERDZONE dla sciezek odnalezionych lokalnie; HIPOTEZA dla roli narzedzi nieuruchamianych w tej rundzie

## Zakres

Inwentarz repozytoriow, katalogow i narzedzi lokalnych przydatnych dla `meshy2aurora`.

## Katalogi projektu

```yaml
canonical_project:
  path: "C:\\Projects\\meshy2aurora"
  status: POTWIERDZONE
  current_contents:
    - ".git"
    - "documentation"
  role: "repo kontraktu/dokumentacji dla konwertera"
documentation:
  path: "C:\\Projects\\meshy2aurora\\documentation"
  status: POTWIERDZONE
  rule: "wszystkie dokumenty projektu sa tutaj"
wrong_documents_path:
  path: "C:\\Users\\enonw\\Documents\\meshy2aurora"
  status: "FORBIDDEN_NON_CANONICAL_SELF_CREATED_PATH"
  rule: "nie uzywac ani nie odtwarzac jako repo, workspace, staging, scratch, backup, worktree, migration source ani documentation target"
```

## Repozytoria lokalne

```yaml
repos:
  aurora_web:
    path: "C:\\Projects\\aurora-web"
    status: POTWIERDZONE
    role: "glowny projekt web: runtime settings, source layer, derived MDL->GLB, catalog, proof CDP"
    reference_only_for_meshy2aurora:
      - "mozna czytac implementacje ERF/HAK jako material porownawczy"
      - "mozna czytac implementacje MDL ASCII/binary parser/converter jako material porownawczy"
      - "mozna czytac runtime source conventions __aurora/sources jako kontekst historyczny"
      - "mozna czytac creature catalog appearance.2da parser jako kontekst"
      - "nie importowac, nie odpalac jako subprocess, nie uzywac jako oracle/proof/fixture source"
  aurora_decompilation:
    path: "C:\\Projects\\New Folder"
    status: POTWIERDZONE
    role: "glowne zrodlo wiedzy Aurora First"
    key_file: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
    reusable_for_meshy2aurora:
      - "parser keyword anchors"
      - "GFF field anchors"
      - "format/runtime behavior anchors"
  nwn:
    path: "C:\\Projects\\nwn"
    status: POTWIERDZONE
    role: "kolekcja feature/prototype NWN, zawiera VFX workspace"
    notable_subdirs:
      - "C:\\Projects\\nwn\\VFX"
    reusable_for_meshy2aurora:
      - "nwnmdlcomp binary found under VFX research"
      - "VFX/MDL reference artifacts"
  nwn_features:
    path: "C:\\Projects\\nwn-features"
    status: POTWIERDZONE
    role: "mniejsze feature packi NWN"
    reusable_for_meshy2aurora:
      - "raczej referencja domenowa; brak potwierdzonego core MDL/HAK kodu"
  nwn_conversation:
    path: "C:\\Projects\\nwn-conversation"
    status: POTWIERDZONE
    role: "Node/TypeScript dialogue builder i DLG/GFF tooling"
    reusable_for_meshy2aurora:
      - "wzorce GFF/locstring/testow TS"
      - "nie jest glownym zrodlem MDL/HAK"
  nwn_localization:
    path: "C:\\Projects\\nwn-localization"
    status: POTWIERDZONE
    role: "TLK/CSV localization workflow"
    reusable_for_meshy2aurora:
      - "jesli custom creature bedzie wymagal TLK/custom strings"
      - "nie blokuje MVP z ****/lokalnymi labelami"
  nwn_last_city:
    path: "C:\\Projects\\nwn-last-city"
    status: POTWIERDZONE
    role: "duzy workspace referencyjny: Areas Generator, web model viewer, reverse docs"
    reusable_for_meshy2aurora:
      - "C:\\Projects\\nwn-last-city\\Areas Generator 3d\\Web Model Viewer\\Common\\erf_reader.py"
      - "C:\\Projects\\nwn-last-city\\Areas Generator 3d\\Web Model Viewer\\Common\\mdl_parser.py"
      - "C:\\Projects\\nwn-last-city\\Areas Generator 3d\\Web Model Viewer\\Common\\mdl_ascii_parser.py"
      - "C:\\Projects\\nwn-last-city\\Areas Generator 3d\\Web Model Viewer\\Common\\twoda_reader.py"
    caveat: "referencja pomocnicza; nie zastepuje dekompilacji ani aurora-web"
  nwn_nui:
    path: "C:\\Projects\\nwn-nui"
    status: POTWIERDZONE
    role: "NUI builder"
    reusable_for_meshy2aurora:
      - "raczej brak dla MDL/creature MVP"
```

## Instalacja NWN / katalogi uzytkownika

```yaml
nwn_install:
  root: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights"
  status: POTWIERDZONE
  executable:
    game: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwmain.exe"
    toolset: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwtoolset.exe"
nwn_user_root:
  path: "C:\\Users\\enonw\\Documents\\Neverwinter Nights"
  status: POTWIERDZONE
  important_subdirs:
    hak: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak"
    modules: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\modules"
    override: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\override"
    erf: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\erf"
    tlk: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\tlk"
    development: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\development"
```

## Narzedzia

```yaml
tools:
  blender_5_1:
    path: "C:\\Program Files\\Blender Foundation\\Blender 5.1\\blender.exe"
    status: POTWIERDZONE
    in_path: false
    role: "inspekcja/naprawa mesh, rig, eksport GLB/FBX"
    reusable: true
  neverblender:
    path: null
    status: "POTWIERDZONE internetowo; NIE WIEM lokalnie"
    audit_doc: "C:\\Projects\\meshy2aurora\\documentation\\neverblender-audyt-2026-07-09-codex.md"
    internet_reference:
      - "https://neverwintervault.org/project/nwnee/other/tool/neverblender-40"
      - "https://neverwintervault.org/project/nwn1/other/tool/neverblender-28"
      - "https://nwn.wiki/display/NWN1/NeverBlender"
    role: "Blender addon do import/export NWN MDL, glownie ASCII MDL; pomocniczy round-trip/debug"
    caveat:
      - "nie jest dependency meshy2aurora"
      - "nie jest oracle/proof"
      - "zgodnosc z lokalnym Blender 5.1.2 niepotwierdzona"
      - "binary MDL wymaga zewnetrznego dekompilatora"
  cleanmodels_ee:
    path: null
    status: "POTWIERDZONE internetowo; NIE WIEM lokalnie"
    internet_reference:
      - "https://neverwintervault.org/project/nwnee/other/tool/clean-modelsee"
      - "https://github.com/plenarius/cleanmodels"
    role: "opcjonalny zewnetrzny dekompilator/czyszczenie MDL NWN:EE, pomocniczo dla NeverBlender/debug"
    caveat:
      - "nie jest dependency meshy2aurora"
      - "nie jest oracle/proof"
      - "lokalna instalacja i wersja niezweryfikowane"
  two_da_tlk_editor:
    path: "C:\\Program Files (x86)\\2DA & TLK Editor\\2DAEditor.exe"
    status: POTWIERDZONE
    role: "manualna inspekcja/edycja 2DA/TLK"
    reusable: true
    schema:
      appearance: "C:\\Program Files (x86)\\2DA & TLK Editor\\Schemas\\appearance.2daschema"
  nwnmdlcomp:
    path: "C:\\Projects\\nwn\\VFX\\source-assets\\loose-graphics-and-references\\nwn-vfx-research\\downloads\\tools\\nwn_model_compiler\\NWN Model Compiler\\nwnmdlcomp.exe"
    old_version: "C:\\Projects\\nwn\\VFX\\source-assets\\loose-graphics-and-references\\nwn-vfx-research\\downloads\\tools\\nwn_model_compiler\\NWN Model Compiler\\OldVersion\\nwnmdlcomp.exe"
    status: POTWIERDZONE
    in_path: false
    role: "kompilacja/dekompilacja MDL pomocniczo"
    caveat: "nie traktowac jako jedynego oracle; starsze narzedzia moga miec problemy z EE"
  aurora_toolset:
    path: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwtoolset.exe"
    status: POTWIERDZONE
    role: "manualna walidacja HAK/appearance/creature w module"
    caveat: "uzywac nisko-inwazyjnie; najlepiej proof przez pliki/screenshoty"
  xoreos_tools_erf:
    path: null
    status: NIE WIEM lokalnie
    internet_reference: "https://xoreos.org/"
    role: "potencjalny CLI paker ERF/HAK"
  nwntools_modpacker:
    path: null
    status: NIE WIEM lokalnie
    internet_reference: "https://nwntools.sourceforge.net/"
    role: "potencjalny zewnetrzny paker .mod/.hak/.erf"
```

## Co reuse'owac najpierw

Status: REKOMENDACJA.

```yaml
reuse_order:
  1_aurora_first:
    source: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
    use_for:
      - "format keyword facts"
      - "runtime/GFF fields"
  2_aurora_web:
    source: "C:\\Projects\\aurora-web"
    mode: "reference-only"
    use_for:
      - "porownanie decyzji implementacyjnych"
      - "odczyt historycznego ingest contract"
      - "nazywanie znanych problemow, bez reuzycia kodu/runtime/testow"
  3_local_tools:
    source:
      - "Blender 5.1"
      - "2DA & TLK Editor"
      - "nwnmdlcomp"
      - "nwtoolset"
    use_for:
      - "manual validation"
      - "asset inspection"
  4_reference_repos:
    source:
      - "C:\\Projects\\nwn-last-city"
      - "C:\\Projects\\nwn"
      - "C:\\Projects\\nwn-conversation"
      - "C:\\Projects\\nwn-localization"
    use_for:
      - "examples"
      - "helpers"
      - "past research"
  5_internet:
    use_for:
      - "missing public specs"
      - "tool download candidates"
    rule: "oznaczac jako HIPOTEZA lub internet supplement, dopoki nie zweryfikowane lokalnie"
```

## Minimalny toolchain dla MVP

Status: HIPOTEZA wdrozeniowa.

```yaml
mvp_toolchain:
  implementation:
    language: "Node.js + TypeScript"
    repo: "C:\\Projects\\meshy2aurora"
    tests: "TDD, unit + fixture binary readback"
  inputs:
    - "Meshy GLB/FBX"
    - "Aurora reference skeleton/model from decomp/aurora-web cache"
  outputs:
    - "native binary MDL"
    - "MDX wedlug zamknietej polityki embedded/separate"
    - "opcjonalny deterministic ASCII/debug dump tylko do snapshotow i Blender/NeverBlender"
    - "appearance.2da patch/merged table"
    - "HAK V1.0"
  validators:
    - "meshy2aurora binary MDL/MDX readback parser"
    - "meshy2aurora HAK/ERF readback parser"
    - "NWN EE Toolset/gra manual proof"
    - "optional external comparison: CleanModels EE / NeverBlender, bez statusu oracle"
```

## Otwarte braki

```yaml
open_gaps:
  local_retail_appearance_2da:
    status: NIE WIEM
    action: "odnalezc w NWN source/2dasource albo przez KEY/BIF reader"
  reusable_erf_writer_package:
    status: HIPOTEZA
    action: "wydzielic z aurora-web skryptow albo napisac TDD writer"
  full_animation_event_taxonomy:
    status: NIE WIEM
    action: "wydobyc z retail modeli/supermodeli i/lub dekompilacji runtime event handlers"
  nwtoolset_automated_proof:
    status: NIE WIEM
    action: "ustalic bezpieczny low-disruption sposob proofu"
```
