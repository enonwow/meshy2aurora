# audyt-parsera-blender-nwn-2026-07-09-codex.md

Data: 2026-07-09  
Autor: Codex  
Status: AUDYT + REKOMENDACJE  

## 0. Cel

Ten dokument odpowiada na pytanie: jak usprawnic parser/konwerter `meshy2aurora`, zeby przewidywal bledy przed eksportem do Aurory/NWN, oraz czy dodatek do Blendera moze pomoc w przeniesieniu modelu do NWN.

Aktywny kierunek projektu:

```text
Meshy GLB/FBX
  -> meshy2aurora
  -> native binary MDL + MDX policy + 2DA + HAK
  -> NWN EE Toolset/gra
```

`C:\Projects\aurora-web` zostaje tylko `reference-only`: mozna czytac implementacje, ale nie wolno uzywac go jako dependency, subprocess, oracle, validator, fixture source ani proof runnera.

## 1. Stan lokalny

```yaml
local_state:
  project:
    path: "C:\\Projects\\meshy2aurora"
    status: POTWIERDZONE
    contains:
      - ".git"
      - "documentation"
      - "sample-2d"
      - "sample-3d"
    code_parser_exists: false
    note: "Repo ma dokumentacje i puste/sample katalogi; nie ma jeszcze implementacji parsera."
  documentation:
    path: "C:\\Projects\\meshy2aurora\\documentation"
    status: POTWIERDZONE
  blender:
    installed:
      path: "C:\\Program Files\\Blender Foundation\\Blender 5.1\\blender.exe"
      version: "Blender 5.1.2"
      status: POTWIERDZONE
    neverblender_compatibility_with_5_1:
      status: NIE_WIEM
      risk: "Glowne opisy NeverBlender 2.8 wskazuja Blender 2.83/2.93/3.0-3.3; Vault ma link do wersji dla Blender 4.0, ale nie potwierdzono 5.1."
```

## 2. Zrodla internetowe

Status: POTWIERDZONE dla faktow z podanych stron; wdrozenie w `meshy2aurora` wymaga lokalnego testu.

```yaml
internet_sources:
  khronos_gltf_validator:
    url: "https://github.com/KhronosGroup/glTF-Validator"
    facts:
      - "Validator sprawdza glTF 2.0/GLBv2 i generuje JSON report z issue i statystykami."
      - "Wykrywa m.in. bledy JSON/GLB, referencje, accessors, NaN, invalid quaternions, animation inputs/outputs i ostrzega o non-power-of-two image dimensions."
    use_for_meshy2aurora: "preflight dla plikow GLB od Meshy przed naszym canonical importem"

  gltf_transform:
    url: "https://gltf-transform.dev/cli"
    facts:
      - "CLI/SDK wspiera inspect oraz operacje odczytu/edycji/zapisu glTF/GLB."
      - "Funkcja inspect zwraca JSON report dla zawartosci glTF."
    use_for_meshy2aurora: "statystyki wejscia: mesh/primitives/vertices/triangles/materials/textures/animations"

  xoreos_nwn1mdl_template:
    url: "https://github.com/xoreos/xoreos-docs/blob/master/templates/NWN1MDL.bt"
    facts:
      - "Repo xoreos-docs ma 010 Editor template NWN1MDL.bt dla NWN1 MDL."
      - "Template opisuje header_file z bin_mdl_id, p_start_mdx, size_mdx."
      - "Template opisuje node content flags, mesh, skin, animation, controllers i MDX data offsets."
    use_for_meshy2aurora: "specyfikacja layoutu binary MDL/MDX do testow parsera i writer/readback"

  neverblender_vault:
    url: "https://neverwintervault.org/project/nwn1/other/tool/neverblender-28"
    facts:
      - "NeverBlender dodaje support MDL NWN do Blendera."
      - "NeverBlender 2.8.051 wymaga Blender 2.83, 2.93, 3.0, 3.1, 3.2 lub 3.3; 3.4 ma znany problem z material import."
      - "Licencja: GPL v3."
    use_for_meshy2aurora: "manualny round-trip/inspekcja debug ASCII MDL, nie runtime dependency"

  neverblender_wiki:
    url: "https://nwn.wiki/display/NWN1/NeverBlender"
    facts:
      - "NeverBlender moze ladowac ASCII MDL; binary MDL wymaga zewnetrznego dekompilatora."
      - "Wspomina CleanModels EE jako najbardziej aktualny dekompilator wspierajacy cechy EE."
      - "Import Placement Line opisuje 10 units distance jako 10 meters in NWN."
      - "Mesh Validation w Blenderze moze usuwac dwustronne faces uzywajace tych samych vertexow."
    use_for_meshy2aurora: "narzedzie pomocnicze; uwazac na zmiane siatki przez mesh validation"

  neverblender_beamdog:
    url: "https://forums.beamdog.com/discussion/72134/neverblender"
    facts:
      - "Opisuje feature list: Trimesh, Danglymesh, Skinmesh, Animesh, walkmeshes, multiple uv maps, multiple textures, smoothgroups, animations, normals/tangents, vertex colors, MTR, basic emitters."
      - "Ma narzedzia do tworzenia Blender armature z pseudo-bones i pseudo-bones z armature."
    use_for_meshy2aurora: "lista rzeczy, ktore parser/writer powinien rozumiec przynajmniej jako unsupported/known node families"

  cleanmodels_ee:
    url: "https://neverwintervault.org/project/nwnee/other/tool/clean-modelsee"
    facts:
      - "CleanModels EE dekompiluje modele z najnowszego NWN:EE."
      - "Dekompiluje renderhint, materialname, normals, tangents i do 64 bones."
      - "Dostepne sa command line i UI buildy."
    use_for_meshy2aurora: "opcjonalne porownanie/debug, szczegolnie dla binary->ASCII; nie glowny oracle"

  borealis:
    url: "https://github.com/eryl/Borealis"
    facts:
      - "Stary Blender 2.5 addon do import/export NWN MDL."
      - "Import wymaga ASCII MDL."
    use_for_meshy2aurora: "historyczna referencja, nie rekomendowany workflow"

  rollnw:
    url: "https://rollnw.readthedocs.io/"
    facts:
      - "Dokumentacja deklaruje implementacje prawie wszystkich formatow NWN."
      - "Deklaruje binary i ASCII Model Parser oraz renderer validation path."
      - "Projekt jest w aktywnej zmianie; API i granice moga sie ruszac."
    use_for_meshy2aurora: "referencja porownawcza, ewentualnie przyszly external comparison, nie dependency"

  neverwinter_nim:
    url: "https://github.com/niv/neverwinter.nim"
    facts:
      - "Biblioteka i CLI dla NWN:EE: resman, ERF, 2DA, GFF, TLK, NWSync."
      - "Nie jest glownym zrodlem MDL parsera w tym audycie."
    use_for_meshy2aurora: "porownanie ERF/2DA/resource manager, nie parser MDL"
```

## 3. Najwazniejszy wniosek

NeverBlender moze pomoc czlowiekowi zobaczyc i naprawic model, ale nie rozwiazuje naszego glownego problemu.

```yaml
neverblender_decision:
  role:
    status: POTWIERDZONE
    value: "manual inspection / Blender round-trip / debug ASCII import-export"
  not_role:
    status: POTWIERDZONE
    value: "nie jest parserem produkcyjnym meshy2aurora, nie jest walidatorem koncowym, nie jest dowodem dzialania w NWN EE"
  blocker:
    status: POTWIERDZONE
    value: "NeverBlender laduje ASCII MDL; binary MDL wymaga zewnetrznego dekompilatora."
  local_risk:
    status: NIE_WIEM
    value: "zgodnosc z lokalnym Blender 5.1.2"
  recommendation:
    - "W meshy2aurora dodac opcjonalny eksport debug ASCII MDL dla NeverBlender."
    - "Nie opierac runtime outputu na ASCII."
    - "Dla pracy artystycznej przygotowac osobny, przenosny Blender zgodny z NeverBlender albo przetestowac wersje NeverBlender dla Blender 4.0/5.x, jesli istnieje."
```

## 4. Audyt luk parsera

Obecna dokumentacja dobrze opisuje target i ogolne moduly, ale brakuje osobnego kontraktu twardosci parsera.

```yaml
parser_gap_audit:
  G1_no_error_taxonomy:
    status: POTWIERDZONE
    problem: "Brak standardu kodow bledow, severity, source offsets i repair hints."
    consequence: "Parser moze raportowac bledy opisowo, ale trudno bedzie budowac UI, testy i automatyczne naprawy."

  G2_no_binary_bounds_policy:
    status: POTWIERDZONE
    problem: "Brak spisanej polityki pointer/offset bounds dla binary MDL/MDX."
    consequence: "Ryzyko blednego odczytu p_start_mdx, -1 sentinel, array_definition lub danych poza buforem."

  G3_no_readback_contract:
    status: CZESCIOWO_POTWIERDZONE
    problem: "Architektura wspomina readback, ale nie ma jeszcze formalnego readback AST/manifestu."
    consequence: "Writer moze generowac plik, ktory wyglada poprawnie na poziomie bajtow, ale semantycznie traci node/skin/animacje."

  G4_no_input_glb_profile:
    status: CZESCIOWO_POTWIERDZONE
    problem: "Jest plan viewport/walidacja, ale parser GLB nie ma jeszcze profilu dozwolonych features."
    consequence: "Meshy moze dac morph targets, wiele materialow, non-triangle primitives, brak UV, nietypowe extensions, za duze tekstury."

  G5_no_external_tool_policy:
    status: POTWIERDZONE
    problem: "Nie bylo jawnie zapisane, jak uzywac NeverBlender/CleanModels/rollNW."
    consequence: "Ryzyko, ze narzedzie pomocnicze zacznie pelnic role oracle albo dependency."

  G6_stale_docs_found:
    status: NAPRAWIONE_W_TYM_AUDYCIE
    changed_files:
      - "C:\\Projects\\meshy2aurora\\documentation\\aurora-mdl-format-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\aurora-animation-system-codex.md"
      - "C:\\Projects\\meshy2aurora\\documentation\\ekosystem-narzedzia-codex.md"
    problem: "Stare wpisy wskazywaly aurora-web validators albo ASCII MDL jako MVP output."
    fix: "Przestawiono na meshy2aurora readback + native binary MDL/MDX + NWN EE proof."
```

## 5. Proponowane funkcje parsera

### P0: funkcje wymagane przed pierwszym writerem

```yaml
parser_features_p0:
  input_guard:
    goal: "fail fast przed odczytem niepoprawnego pliku"
    checks:
      - "rozpoznaj binary MDL vs ASCII debug vs GLB/FBX po magic/header, nie po rozszerzeniu"
      - "sprawdz minimalny rozmiar pliku"
      - "sprawdz little-endian assumptions"
      - "dla binary MDL sprawdz bin_mdl_id == 0"
      - "sprawdz p_start_mdx i size_mdx mieszcza sie w buforze"

  offset_table:
    goal: "jeden centralny mechanizm pointerow"
    checks:
      - "kazdy pointer ma source_offset, target_offset, target_region"
      - "pointery MDL i MDX sa rozdzielone"
      - "wartosc -1 jest dozwolona tylko tam, gdzie format dopuszcza sentinel"
      - "array_definition: p_array_start + count * stride nie moze wyjsc poza bufor"
      - "recursion/cycles limit dla node tree"

  canonical_ast:
    goal: "parser zwraca stabilny model, nie losowe struktury z bufora"
    output:
      - "model header"
      - "node tree"
      - "mesh nodes"
      - "skin nodes"
      - "animation blocks"
      - "resource/material references"
      - "diagnostics"

  diagnostic_report:
    goal: "UI/testy dostaja maszynowy raport"
    format: "JSON"
    fields:
      - "severity: BLOCKER/WARN/INFO"
      - "code"
      - "message"
      - "file"
      - "byteOffset"
      - "nodeName"
      - "repairHint"
      - "source: parser|validator|writer|readback|external"

  readback_gate:
    goal: "kazdy wygenerowany MDL/MDX musi byc odczytany przez nasz parser"
    checks:
      - "write binary"
      - "readback binary"
      - "compare canonical manifest"
      - "compare node count, mesh count, triangle count, bone names, animation names"
```

### P1: funkcje, ktore bardzo zmniejsza liczbe poznych bledow

```yaml
parser_features_p1:
  glb_preflight:
    use:
      - "Khronos glTF Validator for conformance report"
      - "glTF Transform inspect for stats"
    checks:
      - "triangles/vertices/primitives/materials"
      - "POSITION/NORMAL/TEXCOORD_0 presence"
      - "JOINTS_0/WEIGHTS_0 presence for skinned models"
      - "animations count, clip names, durations"
      - "unsupported extensions"
      - "image dimensions and formats"

  geometry_validator:
    checks:
      - "non-triangle primitives -> require triangulation"
      - "degenerate triangles"
      - "zero-area faces"
      - "duplicate vertices over budget"
      - "too many material splits/primitives"
      - "bbox too small/too large"
      - "pivot/root offset suspicious"

  skin_validator:
    checks:
      - "max 4 influences per vertex for binary parity"
      - "weights sum ~= 1.0 after normalization"
      - "bone index maps to existing node"
      - "weighted bone exists in hierarchy"
      - "warn if bone count > 64 until proven safe"
      - "zero-weight influences stripped"

  animation_validator:
    checks:
      - "animroot exists"
      - "all animated node names exist"
      - "keyframe times monotonic"
      - "no key time < 0 or > length"
      - "orientation quaternions finite and normalized"
      - "required clip set present for chosen mode"
      - "events sorted and within length"

  texture_material_validator:
    checks:
      - "texture resref <= 16"
      - "base diffuse output exists"
      - "PBR maps either baked or explicitly discarded with warning"
      - "image size over policy threshold"
      - "missing TXI/MTR policy explicit"
```

### P2: funkcje narzedziowe

```yaml
parser_features_p2:
  debug_ascii_export:
    purpose: "manualny import do NeverBlender/CleanModels workflow"
    rule: "debug only; nie runtime output"

  external_comparison_adapter:
    purpose: "porownanie z CleanModels EE/NeverBlender/rollNW"
    rule: "optional, local, no oracle status"
    report_fields:
      - "tool"
      - "version"
      - "command"
      - "input"
      - "output"
      - "exitCode"
      - "differences"

  auto_repair_suggestions:
    safe_repairs:
      - "normalize weights"
      - "drop zero-weight influences"
      - "triangulate input GLB primitives"
      - "resize/bake textures"
      - "rename generated resrefs"
    unsafe_repairs_need_user_or_gate:
      - "merge/split skin nodes"
      - "retarget skeleton"
      - "delete animation tracks"
      - "change root pivot"
      - "reduce geometry aggressively"
```

## 6. Przewidywane bledy

```yaml
predicted_errors:
  binary_layout:
    - code: "M2A-MDL-0001"
      severity: "BLOCKER"
      message: "binary MDL header too short or bin_mdl_id != 0"
    - code: "M2A-MDL-0002"
      severity: "BLOCKER"
      message: "p_start_mdx/size_mdx outside file buffer"
    - code: "M2A-MDL-0003"
      severity: "BLOCKER"
      message: "array_definition points outside allowed region"
    - code: "M2A-MDL-0004"
      severity: "BLOCKER"
      message: "MDL pointer used as MDX pointer or reverse"

  node_tree:
    - code: "M2A-NODE-0001"
      severity: "BLOCKER"
      message: "node tree has cycle"
    - code: "M2A-NODE-0002"
      severity: "WARN"
      message: "duplicate node name; animation/skin lookup may become ambiguous"
    - code: "M2A-NODE-0003"
      severity: "WARN"
      message: "unsupported node content flag present"

  geometry:
    - code: "M2A-MESH-0001"
      severity: "BLOCKER"
      message: "mesh has no vertex positions"
    - code: "M2A-MESH-0002"
      severity: "BLOCKER"
      message: "face references vertex index outside vertex buffer"
    - code: "M2A-MESH-0003"
      severity: "WARN"
      message: "triangles exceed budget for target type"
    - code: "M2A-MESH-0004"
      severity: "WARN"
      message: "degenerate or zero-area triangles found"

  skin:
    - code: "M2A-SKIN-0001"
      severity: "BLOCKER"
      message: "skin weights vertex count != mesh vertex count"
    - code: "M2A-SKIN-0002"
      severity: "BLOCKER"
      message: "weighted bone missing from hierarchy"
    - code: "M2A-SKIN-0003"
      severity: "WARN"
      message: "weights do not sum to 1.0; normalize before writer"
    - code: "M2A-SKIN-0004"
      severity: "BLOCKER"
      message: "more than 4 influences per vertex after pruning"
    - code: "M2A-SKIN-0005"
      severity: "WARN"
      message: "bone count exceeds 64; CleanModels EE mentions decompile support up to 64 bones, runtime proof needed"

  animation:
    - code: "M2A-ANIM-0001"
      severity: "BLOCKER"
      message: "animation targets node that does not exist"
    - code: "M2A-ANIM-0002"
      severity: "BLOCKER"
      message: "keyframe time is outside animation length"
    - code: "M2A-ANIM-0003"
      severity: "WARN"
      message: "quaternion not normalized"
    - code: "M2A-ANIM-0004"
      severity: "WARN"
      message: "required NWN clip missing for selected creature mode"
    - code: "M2A-ANIM-0005"
      severity: "WARN"
      message: "event outside animation length or unsorted"

  textures:
    - code: "M2A-TEX-0001"
      severity: "BLOCKER"
      message: "missing diffuse/basecolor output"
    - code: "M2A-TEX-0002"
      severity: "WARN"
      message: "texture dimensions exceed policy"
    - code: "M2A-TEX-0003"
      severity: "WARN"
      message: "PBR maps present but no bake policy applied"

  blender_workflow:
    - code: "M2A-BLENDER-0001"
      severity: "WARN"
      message: "NeverBlender compatibility with installed Blender version not confirmed"
    - code: "M2A-BLENDER-0002"
      severity: "WARN"
      message: "NeverBlender import requires ASCII MDL or external decompiler"
    - code: "M2A-BLENDER-0003"
      severity: "WARN"
      message: "Blender mesh validation may remove two-sided duplicate faces"
```

## 7. Rekomendowana architektura parsera

```yaml
recommended_parser_architecture:
  packages:
    src/gltf:
      - "read Meshy GLB"
      - "run glTF validator/inspect adapter"
      - "convert to canonical source model"
    src/mdl/binary:
      - "safe binary reader"
      - "offset table"
      - "typed MDX accessors"
      - "writer"
      - "readback compare"
    src/mdl/ascii-debug:
      - "deterministic debug dump"
      - "optional NeverBlender export target"
    src/validation:
      - "geometry budget gates"
      - "skin gates"
      - "animation gates"
      - "texture/material gates"
      - "report JSON"
    src/tools/external:
      - "optional adapters for CleanModels/NeverBlender/rollNW"
      - "disabled by default"
      - "no oracle status"

  parser_modes:
    inspect_reference:
      strictness: "lenient but safe"
      unknown_fields: "preserve/report as WARN"
      use_case: "read retail/CEP/reference files"
    parse_generated_readback:
      strictness: "strict"
      unknown_fields: "BLOCKER unless explicitly allowed"
      use_case: "verify our output"
    parse_input_glb:
      strictness: "strict for export, lenient for preview"
      unknown_fields: "WARN for viewport, BLOCKER for export if they affect mesh/skin/animation"
```

## 8. TDD plan

```yaml
tdd_parser_plan:
  phase_0_report_schema:
    tests:
      - "diagnostic report serializes stable JSON"
      - "codes are unique"
      - "severity sorting BLOCKER > WARN > INFO"

  phase_1_glb_preflight:
    tests:
      - "valid minimal GLB returns stats"
      - "missing TEXCOORD_0 blocks textured export"
      - "non-triangle primitive requires triangulation"
      - "large texture triggers warning"

  phase_2_binary_reader_safe:
    tests:
      - "too short MDL blocks"
      - "bad p_start_mdx blocks"
      - "out-of-bounds array blocks"
      - "node cycle blocks"
      - "duplicate node names warn"

  phase_3_skin_animation_validation:
    tests:
      - "weights count mismatch blocks"
      - "bone missing blocks"
      - "weights normalize repair suggestion"
      - "animation target missing blocks"
      - "keyframe beyond length blocks"

  phase_4_writer_readback:
    tests:
      - "synthetic minimal creature writes binary MDL"
      - "readback has same resref, nodes, triangle count, bone names"
      - "readback detects corrupted generated MDL"
      - "debug ASCII dump generated but not treated as runtime output"

  phase_5_external_tool_optional:
    tests:
      - "when tool path missing, comparison is skipped"
      - "when CleanModels/NeverBlender available, report captures version/command/output"
      - "external mismatch is WARN unless NWN EE proof also fails"
```

## 9. Blender/NeverBlender workflow

Rekomendowany workflow pomocniczy:

```yaml
blender_workflow:
  goal: "pomoc artystyczna i diagnostyczna, nie implementacyjny core"
  steps:
    - "meshy2aurora importuje Meshy GLB"
    - "meshy2aurora generuje canonical model + validation-report.json"
    - "jesli trzeba recznie poprawic mesh/rig, eksportujemy GLB/FBX do Blendera"
    - "po edycji wracamy do meshy2aurora jako GLB/FBX"
    - "dla diagnostyki NWN generujemy debug ASCII MDL"
    - "debug ASCII MDL mozna otworzyc w NeverBlender zgodnej wersji"
    - "ostateczny output dalej idzie przez nasz binary writer + readback + HAK"
  do_not:
    - "nie generowac finalnego HAK-a z pliku wyeksportowanego przez NeverBlender"
    - "nie traktowac importu w Blenderze jako proofu gry"
    - "nie zakladac kompatybilnosci NeverBlender z Blender 5.1 bez testu"
```

## 10. Kolejne kroki

```yaml
next_steps:
  N1_parser_hardening_spec:
    priority: P0
    action: "Na podstawie tego dokumentu stworzyc testy report schema + binary input guard."
    output:
      - "src/validation/diagnostic.ts"
      - "tests/validation/diagnostic.test.ts"

  N2_gltf_preflight:
    priority: P0
    action: "Dodac inspect GLB i validation-report.json zanim powstanie writer."
    output:
      - "meshy2aurora inspect-glb --input sample.glb --out validation-report.json"

  N3_binary_mdl_input_guard:
    priority: P0
    action: "Zaczac parser binary MDL od header + bounds checks, nie od pelnej semantyki."
    output:
      - "src/mdl/binary/read-header.ts"
      - "tests/mdl/binary/read-header.test.ts"

  N4_debug_ascii_for_blender:
    priority: P1
    action: "Dodac deterministic ASCII/debug dump dopiero po canonical AST."
    output:
      - "dist/<resref>/<resref>.debug.ascii.mdl"

  N5_neverblender_local_test:
    priority: P1
    action: "Osobno sprawdzic, ktora wersja NeverBlender dziala lokalnie; obecny Blender 5.1.2 nie jest potwierdzony."
    output:
      - "documentation/neverblender-test-local-codex.md"

  N6_external_tools_policy:
    priority: P1
    action: "Dodac config env dla optional tools bez robienia z nich dependency."
    env:
      M2A_CLEANMODELS_CLI: "optional"
      M2A_NEVERBLENDER_BLENDER_EXE: "optional"
      M2A_ROLLNW_TOOL: "optional"
```

## 11. Decyzje do zatwierdzenia

```yaml
decisions_to_confirm:
  D_parser_first:
    recommendation: "Najpierw diagnostic report + GLB preflight + binary input guard, dopiero potem writer."
    status: REKOMENDACJA

  D_neverblender:
    recommendation: "NeverBlender jako manual/debug helper przez ASCII debug dump, nie jako glowny exporter."
    status: REKOMENDACJA

  D_blender_version:
    recommendation: "Nie opierac pipeline na lokalnym Blender 5.1.2, dopoki NeverBlender nie przejdzie lokalnego testu."
    status: REKOMENDACJA

  D_error_codes:
    recommendation: "Wprowadzic stabilne kody bledow M2A-* od pierwszego testu."
    status: REKOMENDACJA
```

## 12. Podsumowanie

Najwieksze usprawnienie parsera to nie "czyta wiecej plikow", tylko "nie pozwala wyeksportowac czegos podejrzanego".

Parser powinien byc od poczatku narzedziem diagnostycznym: odczytuje, liczy, mapuje offsety, sprawdza limity, nadaje kody bledow, proponuje naprawy i dopiero potem pozwala writerowi wygenerowac MDL/HAK. NeverBlender moze byc bardzo przydatny w pracy recznej, zwlaszcza przy ASCII MDL i armature/pseudo-bones, ale finalna sciezka projektu musi zostac: `our parser -> our writer -> readback -> HAK -> NWN EE proof`.
