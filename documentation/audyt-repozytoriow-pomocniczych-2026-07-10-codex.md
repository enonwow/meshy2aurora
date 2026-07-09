# audyt-repozytoriow-pomocniczych-2026-07-10-codex.md

Data: 2026-07-10  
Autor: Codex  
Status: POTWIERDZONE LOKALNIE  
Zakres: audyt repozytoriow z `C:\Projects\Claude` jako suplementu po `Aurora First`

## 0. Werdykt

Mamy na czym bazowac, ale nie jako gotowy fundament do kopiowania.

Repozytoria w `C:\Projects\Claude` tworza bardzo dobra druga linie dla:

- binary/ascii `MDL`,
- animacji, skinningu, kosci, supermodeli,
- texture/material/TGA/DDS/PLT/TXI,
- `2DA`,
- `GFF`,
- `ERF/HAK/MOD/SAV`,
- KEY/BIF/resource manager,
- viewportu/debug renderingu.

Nie zmienia to zasad projektu:

```yaml
project_rule:
  source_of_truth: "Aurora First"
  primary_reference: "C:\\Projects\\New Folder"
  external_repositories_role:
    - "suplement"
    - "cross-check"
    - "material do testow i przewidywania bledow"
    - "debug/reference-only"
  forbidden_by_default:
    - "kopiowanie kodu do core bez decyzji licencyjnej"
    - "robienie z repo oracle zamiast NWN EE proof"
    - "uzywanie repo jako produkcyjnej zaleznosci bez decyzji architektonicznej"
```

## 1. Metoda audytu

Status: POTWIERDZONE.

Zrobione lokalnie:

```yaml
audit_method:
  root: "C:\\Projects\\Claude"
  repositories_scanned: 27
  measured_total_bytes: 1217890023
  scan_sources:
    - "git remote get-url origin"
    - "git rev-parse --abbrev-ref HEAD"
    - "git rev-parse --short HEAD"
    - "rg --files z pominieciem .git"
    - "rg -i -l dla domenowych slow kluczowych"
    - "README / LICENSE / COPYING"
  keyword_buckets:
    - "mdl"
    - "mdx"
    - "animation"
    - "skin"
    - "2DA"
    - "GFF"
    - "ERF/HAK"
    - "resman/KEY/BIF/TLK"
    - "texture/material"
    - "viewport/render"
```

Uwaga: liczniki keywordowe sa wskaznikiem sygnalu, nie formalna metryka kompletnej obslugi formatu.

## 2. Najkrotsza klasyfikacja

Status: POTWIERDZONE + rekomendacja Codexa.

```yaml
tiers:
  A_core_reference:
    - "borealis_nwn_mdl"
    - "rollnw"
    - "Radoub"
    - "xoreos-docs"
    - "neverwinter.nim"
    - "borealis_nwn_resman"
  B_strong_support:
    - "cleanmodels"
    - "nwn_mdl_webviewer"
    - "borealis_nwn_model_viewer"
    - "NWNFileFormats"
    - "nwnexplorer"
    - "nwn-lib-d"
    - "xoreos-tools"
    - "xoreos"
  C_narrow_or_historical:
    - "nwn-lib"
    - "Alia"
    - "Moneo"
    - "NWNT"
    - "serde-gff"
    - "nwn_sqlite"
    - "nwn-mcp"
    - "mudl"
    - "nwn_mdl_viewer_tauri"
  D_static_data_or_low_priority:
    - "NWN1EE_2DAs"
    - "nwn-2da"
    - "NWNEEGameData"
    - "nwn_minimap"
```

Najwazniejsze repo na start M1/M2:

```yaml
recommended_reading_order_for_m1:
  mdl_binary_layout:
    - "C:\\Projects\\Claude\\xoreos-docs\\templates\\NWN1MDL.bt"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\docs\\BinaryMdlFormat.dox"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\BinaryParser.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\BinaryWriter.cpp"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\model\\MdlBinaryParser.cpp"
  animation_skinning:
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\AnimationPlayer.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\docs\\SupermodelSystem.dox"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\render\\model_instance_animation.cpp"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\render\\viewer\\preview_model_animation.cpp"
  hak_2da_gff:
    - "C:\\Projects\\Claude\\borealis_nwn_resman"
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats"
    - "C:\\Projects\\Claude\\neverwinter.nim"
    - "C:\\Projects\\Claude\\NWNFileFormats"
```

## 3. Inwentarz lokalny

Status: POTWIERDZONE.

| repo | commit | shallow | files | MB | domena glowna | priorytet |
|---|---:|---:|---:|---:|---|---|
| `Alia` | `42459a0` | false | 7 | 0.26 | GFF legacy | C |
| `borealis_nwn_mdl` | `bdc889c` | false | 44 | 0.58 | MDL/animation/texture | A |
| `borealis_nwn_model_viewer` | `16c8739` | false | 37 | 0.47 | viewport MDL | B |
| `borealis_nwn_resman` | `03f4d28` | true | 48 | 0.44 | resource manager/GFF/ERF/2DA | A |
| `cleanmodels` | `990e9a0` | false | 11 | 0.52 | binary MDL decompile/check | B |
| `Moneo` | `12de240` | false | 18 | 0.61 | GFF batch edit legacy | C |
| `mudl` | `5d6a9ab` | true | 246 | 29.96 | model viewer PoC | C |
| `neverwinter.nim` | `db755db` | true | 267 | 12.63 | NWN EE files/resman | A |
| `nwn_mdl_viewer_tauri` | `a1becb9` | true | 71 | 6.75 | wrapper viewer | C |
| `nwn_mdl_webviewer` | `dfb219c` | false | 57 | 13.35 | browser MDL viewer | B |
| `nwn_minimap` | `e0bfb53` | false | 3 | 0.01 | module minimap | D |
| `nwn_sqlite` | `d29a864` | false | 20 | 0.28 | module indexing | C |
| `NWN1EE_2DAs` | `417bf2b` | true | 601 | 21.80 | 2DA snapshot | D |
| `nwn-2da` | `4b071f6` | true | 577 | 5.46 | 2DA 1.69 snapshot | D |
| `NWNEEGameData` | `10f77144` | true | 34252 | 101.14 | scripts + 2DA snapshot | D |
| `nwnexplorer` | `56da6dc` | false | 490 | 15.06 | explorer/nwnmdlcomp/model view | B |
| `NWNFileFormats` | `a13236d` | false | 64 | 0.20 | C++ GFF/ERF/KEY/BIF/TLK | B |
| `nwn-lib` | `7041a0f` | true | 61 | 0.24 | Ruby GFF/ERF/2DA/TLK | C |
| `nwn-lib-d` | `7bbb614` | true | 163 | 55.62 | D tools GFF/ERF/2DA/TLK | B |
| `nwn-mcp` | `f83be47` | false | 78 | 150.15 | MCP/module automation | C |
| `NWNT` | `692bd26` | true | 21 | 0.37 | GFF text diff | C |
| `Radoub` | `d31cf86` | false | 1512 | 42.39 | GFF/2DA/ERF/HAK + MDL readers | A |
| `rollnw` | `b3512c1` | true | 10495 | 386.91 | broad NWN formats/render/resman | A |
| `serde-gff` | `2bfacbb` | true | 24 | 0.26 | Rust GFF | C |
| `xoreos` | `89c99d2` | true | 2294 | 26.65 | Aurora engine reimplementation | B |
| `xoreos-docs` | `4e1c197` | false | 54 | 2.03 | specs/templates | A |
| `xoreos-tools` | `b2ebf4f` | true | 526 | 26.14 | format tools | B |

## 4. Domenowy heatmap

Status: POTWIERDZONE.

Wartosci to liczba plikow trafionych przez `rg -i -l` w danym bucketcie.

```yaml
domain_signal_top:
  mdl:
    strongest:
      rollnw: 115
      nwnexplorer: 89
      Radoub: 57
      borealis_nwn_mdl: 43
      xoreos: 43
      nwn_mdl_webviewer: 38
  mdx:
    strongest:
      rollnw: 10
      xoreos: 7
      xoreos_docs: 3
      nwn_mdl_webviewer: 2
      xoreos_tools: 2
    note: "MDX ma mniej jawnych trafien niz MDL; polityka MDX nadal wymaga osobnego testu/proofu"
  animation:
    strongest:
      rollnw: 525
      NWNEEGameData: 2156
      xoreos: 108
      Radoub: 74
      borealis_nwn_mdl: 28
      nwn_mdl_webviewer: 22
  skin:
    strongest:
      rollnw: 348
      Radoub: 170
      xoreos: 89
      NWN1EE_2DAs: 80
      nwn_2da: 76
      nwnexplorer: 49
  two_da:
    strongest:
      NWNEEGameData: 1257
      NWN1EE_2DAs: 601
      nwn_2da: 577
      Radoub: 164
      rollnw: 171
      xoreos: 143
  gff:
    strongest:
      Radoub: 230
      xoreos: 228
      rollnw: 122
      xoreos_tools: 58
      nwn_mcp: 46
      nwn_lib: 37
  erf_hak:
    strongest:
      rollnw: 1327
      Radoub: 305
      xoreos: 292
      xoreos_tools: 123
      NWN1EE_2DAs: 120
      nwn_2da: 114
  resman:
    strongest:
      rollnw: 1228
      Radoub: 550
      xoreos: 357
      nwnexplorer: 142
      xoreos_tools: 119
      neverwinter_nim: 76
  texture_material:
    strongest:
      rollnw: 1413
      xoreos: 404
      NWNEEGameData: 429
      Radoub: 176
      xoreos_tools: 139
      mudl: 101
  viewport_render:
    strongest:
      rollnw: 958
      xoreos: 324
      Radoub: 176
      mudl: 147
      nwn_mdl_webviewer: 42
      nwnexplorer: 37
```

## 5. Audyt A: repozytoria core-reference

### 5.1 `borealis_nwn_mdl`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\borealis_nwn_mdl`  
Remote: `https://github.com/varenx/borealis_nwn_mdl.git`  
Commit: `bdc889c`

Najlepsze zastosowanie:

```yaml
borealis_nwn_mdl:
  role:
    - "druga linia dla binary/ascii MDL"
    - "porownanie animacji i skinningu"
    - "porownanie texture/material/TGA/DDS/PLT/TXI"
    - "kontrola decompile-to-ASCII"
  key_files:
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\docs\\BinaryMdlFormat.dox"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\docs\\SupermodelSystem.dox"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\BinaryParser.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\BinaryWriter.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\AsciiParser.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\AsciiWriter.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\AnimationPlayer.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\internal\\MdlStructures.hpp"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\include\\borealis\\nwn\\mdl\\Mesh.hpp"
    - "C:\\Projects\\Claude\\borealis_nwn_mdl\\include\\borealis\\nwn\\mdl\\Animation.hpp"
  risk:
    - "GPL/COPYING; reference-only"
    - "C++23; nie przyjmowac jako dependency bez decyzji"
  verdict: "A: najwazniejsze repo do porownania MDL i animacji"
```

### 5.2 `rollnw`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\rollnw`  
Remote: `https://github.com/jd28/rollnw.git`  
Commit: `b3512c1`

Najlepsze zastosowanie:

```yaml
rollnw:
  role:
    - "nowoczesny, szeroki stack NWN"
    - "MDL parser, render, resman, GFF, ERF, texture"
    - "headless/render validation ideas"
    - "fuzz corpus dla formatow"
  key_files:
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\model\\MdlBinaryParser.cpp"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\model\\MdlTextParser.cpp"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\model\\Mdl.cpp"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\resources\\ResourceManager.cpp"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\resources\\Erf.cpp"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\resources\\StaticErf.cpp"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\serialization\\Gff.cpp"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\formats\\Plt.cpp"
    - "C:\\Projects\\Claude\\rollnw\\lib\\nw\\render\\viewer\\preview_model_animation.cpp"
    - "C:\\Projects\\Claude\\rollnw\\fuzz\\corpus\\format_parsers\\mdl_binary"
    - "C:\\Projects\\Claude\\rollnw\\fuzz\\corpus\\format_parsers\\erf"
    - "C:\\Projects\\Claude\\rollnw\\fuzz\\corpus\\format_parsers\\gff"
  caveat:
    - "repo aktywnie idzie w nowoczesny runtime/toolset, nie w silnik Aurora 1:1"
    - "MIT jest korzystniejszy, ale nadal nie kopiowac automatycznie"
  verdict: "A: najlepszy szeroki cross-check, zwlaszcza gdy Borealis i xoreos-docs nie wystarcza"
```

### 5.3 `Radoub`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\Radoub`  
Remote: `https://github.com/LordOfMyatar/Radoub.git`  
Commit: `d31cf86`

Najlepsze zastosowanie:

```yaml
radoub:
  role:
    - "praktyczny stack C# dla GFF/2DA/ERF/HAK/resource resolver"
    - "reader/writer blueprintow i modulow"
    - "dodatkowe MDL ASCII/Binary readers"
    - "testy corrupted/round-trip"
  key_files:
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Erf\\ErfReader.cs"
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Erf\\ErfWriter.cs"
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Gff\\GffReader.cs"
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Gff\\GffWriter.cs"
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\TwoDA\\TwoDAReader.cs"
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Resolver\\GameResourceResolver.cs"
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Resolver\\ModuleHakResolver.cs"
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Mdl\\MdlBinaryReader.cs"
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Mdl\\MdlAsciiReader.cs"
    - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats.Tests\\CorruptedFileTests.cs"
  caveat:
    - "duzy projekt narzedziowy, nie model converter"
    - "traktowac jako reference-only"
  verdict: "A: najlepszy praktyczny cross-check dla GFF/2DA/ERF/HAK i testow odpornosci"
```

### 5.4 `xoreos-docs`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\xoreos-docs`  
Remote: `https://github.com/xoreos/xoreos-docs.git`  
Commit: `4e1c197`

Najlepsze zastosowanie:

```yaml
xoreos_docs:
  role:
    - "specyfikacje i 010 Editor templates"
    - "formalny punkt odniesienia dla binary layout"
    - "BioWare/Torlack docs"
  key_files:
    - "C:\\Projects\\Claude\\xoreos-docs\\templates\\NWN1MDL.bt"
    - "C:\\Projects\\Claude\\xoreos-docs\\specs\\torlack\\binmdl.html"
    - "C:\\Projects\\Claude\\xoreos-docs\\specs\\bioware"
  caveat:
    - "spec/template nie jest runtime proof"
    - "mozliwe roznice NWN 1.69 vs NWN EE"
  verdict: "A: pierwsza zewnetrzna kotwica dla layoutu MDL"
```

### 5.5 `neverwinter.nim`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\neverwinter.nim`  
Remote: `https://github.com/niv/neverwinter.nim.git`  
Commit: `db755db`

Najlepsze zastosowanie:

```yaml
neverwinter_nim:
  role:
    - "NWN EE resource file library and CLI collection"
    - "resman, key/bif, erf, gff, tlk"
    - "praktyczne narzedzia z stdout/stderr"
    - "dokumenty BioWare PDF w repo"
  key_files:
    - "C:\\Projects\\Claude\\neverwinter.nim\\neverwinter\\resman.nim"
    - "C:\\Projects\\Claude\\neverwinter.nim\\neverwinter\\erf.nim"
    - "C:\\Projects\\Claude\\neverwinter.nim\\neverwinter\\gff.nim"
    - "C:\\Projects\\Claude\\neverwinter.nim\\neverwinter\\key.nim"
    - "C:\\Projects\\Claude\\neverwinter.nim\\neverwinter\\tlk.nim"
    - "C:\\Projects\\Claude\\neverwinter.nim\\nwn_resman_extract.nim"
    - "C:\\Projects\\Claude\\neverwinter.nim\\nwn_erf.nim"
    - "C:\\Projects\\Claude\\neverwinter.nim\\nwn_gff.nim"
    - "C:\\Projects\\Claude\\neverwinter.nim\\docs\\Bioware_Aurora_ERF_Format.pdf"
    - "C:\\Projects\\Claude\\neverwinter.nim\\docs\\Bioware_Aurora_GFF_Format.pdf"
    - "C:\\Projects\\Claude\\neverwinter.nim\\docs\\Bioware_Aurora_2DA_Format.pdf"
  caveat:
    - "nie jest MDL-first"
    - "CLI moze byc external validator, ale nie core dependency bez decyzji"
  verdict: "A: najlepszy praktyczny suplement dla resman/ERF/GFF/KEY/TLK"
```

### 5.6 `borealis_nwn_resman`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\borealis_nwn_resman`  
Remote: `https://github.com/varenx/borealis_nwn_resman.git`  
Commit: `03f4d28`

Najlepsze zastosowanie:

```yaml
borealis_nwn_resman:
  role:
    - "C++23 resource manager library"
    - "KEY/BIF, ERF/MOD/HAK/SAV, GFF, 2DA, TLK, MTR, ITP"
    - "resource enumeration and module management"
  key_files:
    - "C:\\Projects\\Claude\\borealis_nwn_resman\\src\\ResourceManager.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_resman\\src\\ErfParser.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_resman\\src\\ErfWriter.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_resman\\src\\GffParser.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_resman\\src\\GffWriter.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_resman\\src\\TwoDAParser.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_resman\\src\\KeyBifParser.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_resman\\src\\MtrParser.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_resman\\include\\borealis\\nwn\\resman\\ResourceTypes.hpp"
  caveat:
    - "GPL/COPYING; reference-only"
    - "bardziej resource pipeline niz Meshy/model transform"
  verdict: "A: bardzo przydatne przy HAK/ERF/GFF/2DA i resman"
```

## 6. Audyt B: silne repozytoria pomocnicze

### 6.1 `cleanmodels`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\cleanmodels`

```yaml
cleanmodels:
  role:
    - "historyczne/narzedziowe sanity-check dla modeli"
    - "binary loader/decompiler flow"
  key_files:
    - "C:\\Projects\\Claude\\cleanmodels\\load_binary.pl"
    - "C:\\Projects\\Claude\\cleanmodels\\load_models.pl"
    - "C:\\Projects\\Claude\\cleanmodels\\output_models.pl"
    - "C:\\Projects\\Claude\\cleanmodels\\make_checks.pl"
    - "C:\\Projects\\Claude\\cleanmodels\\fix_pivots.pl"
  verdict: "B: dobre do sanity-checku, nie jako core"
```

### 6.2 `nwn_mdl_webviewer`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\nwn_mdl_webviewer`

```yaml
nwn_mdl_webviewer:
  role:
    - "browserowy viewport dla binary i ASCII MDL"
    - "Three.js scene graph, animation, emitters, textures"
    - "debug UI dla przyszlego viewportu meshy2aurora"
  key_files:
    - "C:\\Projects\\Claude\\nwn_mdl_webviewer\\js\\parser.js"
    - "C:\\Projects\\Claude\\nwn_mdl_webviewer\\js\\animation.js"
    - "C:\\Projects\\Claude\\nwn_mdl_webviewer\\js\\scene_build.js"
    - "C:\\Projects\\Claude\\nwn_mdl_webviewer\\js\\textures.js"
    - "C:\\Projects\\Claude\\nwn_mdl_webviewer\\js\\mtr.js"
    - "C:\\Projects\\Claude\\nwn_mdl_webviewer\\js\\emitter.js"
    - "C:\\Projects\\Claude\\nwn_mdl_webviewer\\docs\\FORMAT.md"
    - "C:\\Projects\\Claude\\nwn_mdl_webviewer\\docs\\DECOMPILE.md"
  verdict: "B: bardzo dobry do UX/viewport debug, nie proof gry"
```

### 6.3 `borealis_nwn_model_viewer`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\borealis_nwn_model_viewer`

```yaml
borealis_nwn_model_viewer:
  role:
    - "Qt6/OpenGL viewer dla NWN MDL"
    - "animation controls, model tree, skinned shader"
  key_files:
    - "C:\\Projects\\Claude\\borealis_nwn_model_viewer\\src\\ui\\ModelViewport.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_model_viewer\\src\\ui\\AnimationControls.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_model_viewer\\src\\renderer\\Renderer.cpp"
    - "C:\\Projects\\Claude\\borealis_nwn_model_viewer\\src\\renderer\\shaders\\skinned.vert"
    - "C:\\Projects\\Claude\\borealis_nwn_model_viewer\\src\\animation\\AnimationPlayer.cpp"
  verdict: "B: dobry viewport/render reference"
```

### 6.4 `NWNFileFormats`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\NWNFileFormats`

```yaml
nwn_file_formats:
  role:
    - "lekka C++ biblioteka formatow krytycznych"
    - "GFF, ERF, KEY, BIF, TLK, 2DA"
  key_files:
    - "C:\\Projects\\Claude\\NWNFileFormats\\FileFormats\\Erf.hpp"
    - "C:\\Projects\\Claude\\NWNFileFormats\\FileFormats\\Gff.hpp"
    - "C:\\Projects\\Claude\\NWNFileFormats\\FileFormats\\Key.hpp"
    - "C:\\Projects\\Claude\\NWNFileFormats\\FileFormats\\Bif.hpp"
    - "C:\\Projects\\Claude\\NWNFileFormats\\FileFormats\\2da.hpp"
    - "C:\\Projects\\Claude\\NWNFileFormats\\Tools\\Tool_ErfExtractor.cpp"
    - "C:\\Projects\\Claude\\NWNFileFormats\\Tools\\Tool_2daMerge.cpp"
  verdict: "B: dobry prosty cross-check dla formatow kontenerow i 2DA/GFF"
```

### 6.5 `nwnexplorer`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\nwnexplorer`

```yaml
nwnexplorer:
  role:
    - "historyczny explorer i nwnmdlcomp"
    - "model raw hierarchy/view"
    - "NwnMdl serialize/decompile"
  key_files:
    - "C:\\Projects\\Claude\\nwnexplorer\\nwnmdlcomp\\nwnmdlcomp.cpp"
    - "C:\\Projects\\Claude\\nwnexplorer\\_NwnLib\\NwnMdlSerialize.cpp"
    - "C:\\Projects\\Claude\\nwnexplorer\\_NwnLib\\NwnMdlR2A.cpp"
    - "C:\\Projects\\Claude\\nwnexplorer\\_NwnLib\\NwnMdlDecomp.cpp"
    - "C:\\Projects\\Claude\\nwnexplorer\\_NwnLib\\NwnMdlGeometry.cpp"
    - "C:\\Projects\\Claude\\nwnexplorer\\_NwnLib\\NwnMdlNodes.cpp"
    - "C:\\Projects\\Claude\\nwnexplorer\\nwnexplorer\\ModelView.cpp"
  caveat:
    - "lokalne nwnmdlcomp.exe historycznie nie bylo zwalidowane jako oracle na tej maszynie"
  verdict: "B: bardzo ciekawy reference, ale nie traktowac jako dzialajacego tool-oracle bez proofu"
```

### 6.6 `nwn-lib-d`

Status: POTWIERDZONE.  
Path: `C:\Projects\Claude\nwn-lib-d`

```yaml
nwn_lib_d:
  role:
    - "D library/tools dla NWN/NWN2 resource files"
    - "GFF/ERF/2DA/TLK + test fixtures"
  key_files:
    - "C:\\Projects\\Claude\\nwn-lib-d\\source\\nwn\\gff.d"
    - "C:\\Projects\\Claude\\nwn-lib-d\\source\\nwn\\fastgff.d"
    - "C:\\Projects\\Claude\\nwn-lib-d\\source\\nwn\\erf.d"
    - "C:\\Projects\\Claude\\nwn-lib-d\\source\\nwn\\tlk.d"
    - "C:\\Projects\\Claude\\nwn-lib-d\\source\\nwn\\dds.d"
    - "C:\\Projects\\Claude\\nwn-lib-d\\tools\\nwn-gff"
    - "C:\\Projects\\Claude\\nwn-lib-d\\tools\\nwn-erf"
    - "C:\\Projects\\Claude\\nwn-lib-d\\tools\\nwn-2da"
    - "C:\\Projects\\Claude\\nwn-lib-d\\unittest\\test.hak"
    - "C:\\Projects\\Claude\\nwn-lib-d\\unittest\\2da"
  caveat:
    - "maintenance mode"
    - "GPL 3"
  verdict: "B: dobry walidator/cross-check, mniej atrakcyjny jako baza core"
```

### 6.7 `xoreos-tools` i `xoreos`

Status: POTWIERDZONE.  
Paths:

- `C:\Projects\Claude\xoreos-tools`
- `C:\Projects\Claude\xoreos`

```yaml
xoreos_family:
  role:
    - "Aurora engine reverse-engineering ecosystem"
    - "format conversion/extraction"
    - "engine-side resource/render perspective"
  key_files_xoreos_tools:
    - "gff2xml/xml2gff"
    - "convert2da"
    - "unerf"
    - "unkeybif"
    - "erf"
  caveat:
    - "GPL 3"
    - "szerszy zakres niz NWN-only"
  verdict: "B: dobra druga/trzecia linia dla formatow, mniej bezposrednia niz xoreos-docs"
```

## 7. Audyt C/D: repozytoria waskie, historyczne lub danych

Status: POTWIERDZONE.

```yaml
narrow_repositories:
  nwn_lib:
    path: "C:\\Projects\\Claude\\nwn-lib"
    role: "Ruby library for GFF/ERF/KEY/BIF/2DA/TLK"
    priority: C
    use: "historyczny cross-check formatu, szczegolnie GFF/ERF"
  Alia:
    path: "C:\\Projects\\Claude\\Alia"
    role: "GFF Reader/Writer legacy"
    priority: C
    use: "edge-case comparison dla GFF, nisko po Radoub"
  Moneo:
    path: "C:\\Projects\\Claude\\Moneo"
    role: "mass-edit GFF legacy"
    priority: C
    use: "mental model batch edits; nie core"
  NWNT:
    path: "C:\\Projects\\Claude\\NWNT"
    role: "GFF <-> text diff format"
    priority: C
    use: "czytelne diffy GFF, przydatne przy testach"
  serde_gff:
    path: "C:\\Projects\\Claude\\serde-gff"
    role: "Rust BioWare GFF serde"
    priority: C
    use: "dodatkowy cross-check GFF, zwlaszcza ograniczenia field label"
  nwn_sqlite:
    path: "C:\\Projects\\Claude\\nwn_sqlite"
    role: "module data extraction to SQLite"
    priority: C
    use: "raportowanie/indeksowanie modulow pozniej"
  nwn_mcp:
    path: "C:\\Projects\\Claude\\nwn-mcp"
    role: "TypeScript MCP wrapping neverwinter.nim"
    priority: C
    use: "UX/automation ideas, nie model converter"
  mudl:
    path: "C:\\Projects\\Claude\\mudl"
    role: "archived model viewer PoC"
    priority: C
    use: "render/viewer inspiration, nizej niz rollnw viewer i Borealis"
  nwn_mdl_viewer_tauri:
    path: "C:\\Projects\\Claude\\nwn_mdl_viewer_tauri"
    role: "desktop wrapper for nwn_mdl_webviewer"
    priority: C
    use: "opakowanie, nie nowa wiedza o MDL"
  nwn_minimap:
    path: "C:\\Projects\\Claude\\nwn_minimap"
    role: "area minimap extraction"
    priority: D
    use: "poza core Meshy -> MDL"
```

Repozytoria danych:

```yaml
data_repositories:
  NWN1EE_2DAs:
    path: "C:\\Projects\\Claude\\NWN1EE_2DAs"
    files: 601
    role: "official-ish NWN EE 2DA snapshot"
    priority: D
    warning: "nie zastapi lokalnych plikow gry i proofu"
  nwn_2da:
    path: "C:\\Projects\\Claude\\nwn-2da"
    files: 577
    role: "NWN 1.69 2DA snapshot"
    priority: D
    warning: "stare 1.69; tylko porownanie nazw/kolumn"
  NWNEEGameData:
    path: "C:\\Projects\\Claude\\NWNEEGameData"
    files: 34252
    role: "NWN EE scripts + 2DA"
    priority: D
    warning: "README deklaruje brak licencji; tylko searchable reference"
```

## 8. Mapowanie na pipeline `meshy2aurora`

Status: POTWIERDZONE + rekomendacja Codexa.

```yaml
pipeline_reference_map:
  s1_meshy_input_glb_fbx:
    repository_support:
      - "brak bezposredniego NWN repo; to nasza transformacja"
    note: "osi, skala, decymacja, bake PBR, UV sa nasza warstwa, nie Aurora legacy"

  s2_geometry_budget_and_mesh_shape:
    repository_support:
      - "borealis_nwn_mdl: mesh structures"
      - "rollnw: model/render structures"
      - "nwn_mdl_webviewer: scene_build/parser"
      - "xoreos-docs: binary layout"
    risk:
      - "repo nie odpowiadaja automatycznie na limity retail creature"
      - "budzety trzeba potwierdzac na retail/dekompilacji"

  s3_skeleton_and_skinning:
    repository_support:
      - "borealis_nwn_mdl: AnimationPlayer, Mesh, Node"
      - "borealis_nwn_model_viewer: skinned shader"
      - "rollnw: render model animation"
      - "nwn_mdl_webviewer: animation.js"
    risk:
      - "Meshy auto-rig dziala glownie humanoidalnie; dla potworow trzeba Aurora skeleton reference"

  s4_animation_transfer:
    repository_support:
      - "borealis_nwn_mdl: SupermodelSystem.dox"
      - "rollnw: preview_model_animation.cpp"
      - "nwnexplorer: ModelAnimDlg/ModelView"
    risk:
      - "dodanie nowych animacji to osobna funkcja, nie automatyczna konsekwencja konwersji modelu"

  s5_mdl_binary_writer:
    repository_support:
      - "xoreos-docs: NWN1MDL.bt"
      - "borealis_nwn_mdl: BinaryWriter/BinaryParser"
      - "rollnw: MdlBinaryParser"
      - "cleanmodels: load_binary/output_models"
      - "Radoub: MdlBinaryReader"
    proof_needed:
      - "wlasny minimalny MDL z testu"
      - "NWN EE/toolset/game loads model"

  s6_mdx_policy:
    repository_support:
      - "rollnw: mdx keyword signal"
      - "xoreos-docs: MDX/spec references"
      - "nwn_mdl_webviewer: MDX awareness"
    status: "NADAL LUKA"
    proof_needed:
      - "ustalic czy nasz writer tworzy standalone MDL, MDL+MDX, czy embedded data"

  s7_texture_material:
    repository_support:
      - "borealis_nwn_mdl: Tga/Dds/Plt/Txi/Material"
      - "nwn_mdl_webviewer: textures/mtr/txi"
      - "rollnw: texture_decode/Plt"
      - "xoreos-tools: texture extraction"
    note: "Meshy PBR -> NWN diffuse/TGA to nadal nasza warstwa"

  s8_2da_appearance:
    repository_support:
      - "local game 2DA first"
      - "Radoub: TwoDAReader"
      - "borealis_nwn_resman: TwoDAParser"
      - "neverwinter.nim"
      - "NWN1EE_2DAs and NWNEEGameData as snapshots"
    proof_needed:
      - "appearance.2da row generated and loaded by our HAK/module"

  s9_hak_erf_packaging:
    repository_support:
      - "borealis_nwn_resman: ErfWriter"
      - "Radoub: ErfWriter"
      - "neverwinter.nim: nwn_erf"
      - "NWNFileFormats: Erf"
      - "rollnw: StaticErf/Erf"
    proof_needed:
      - "nasz HAK contains MDL/TGA/2DA and NWN EE resolves it"

  s10_viewport_and_validation:
    repository_support:
      - "nwn_mdl_webviewer"
      - "borealis_nwn_model_viewer"
      - "rollnw mudl/viewer"
    caveat:
      - "viewport jest debug gate, nie final proof"
```

## 9. Luki po audycie repozytoriow

Status: POTWIERDZONE jako luki projektowe.

```yaml
remaining_gaps:
  G1_meshy_transform_layer:
    status: "LUKA"
    description: "Zadne repo NWN nie rozwiazuje gotowo osi/skali/decymacji/PBR bake dla Meshy"
    action: "opisac nasz kontrakt GLB -> intermediate scene"

  G2_mdx_policy:
    status: "LUKA"
    description: "MDX ma znacznie mniej jawnego materialu niz MDL; trzeba ustalic writer policy"
    action: "mini-audyt MDX w xoreos-docs, rollnw, nwnexplorer i retail models"

  G3_runtime_acceptance:
    status: "LUKA"
    description: "Repozytoria nie zastepuja proofu w NWN EE"
    action: "zdefiniowac minimalny generated HAK + module proof"

  G4_license_boundary:
    status: "LUKA_PROCESOWA"
    description: "Czesc repo jest GPL albo bez jasnej licencji"
    action: "utrzymac reference-only; copy code tylko po decyzji"

  G5_technology_choice:
    status: "NADAL_DO_DECYZJI"
    description: "Repozytoria maja C++/C#/Nim/D/Ruby/Rust/JS; nie narzucaja naszego stacku"
    action: "osobny dokument decyzji technologicznej dla meshy2aurora"

  G6_fixture_policy:
    status: "CZESCIOWO_ROZSTRZYGNIECIE"
    description: "Repo moga dostarczyc inspiracji, ale proof fixtures powinny byc nasze/generowane"
    action: "spisac test fixture matrix dla M1"
```

## 10. Rekomendowane kolejne kroki

Status: REKOMENDACJA.

```yaml
next_steps:
  K1_mdl_binary_crosswalk:
    priority: P0
    output: "documentation/mdl-binary-crosswalk-codex.md"
    scope:
      - "xoreos-docs NWN1MDL.bt"
      - "borealis_nwn_mdl BinaryParser/BinaryWriter"
      - "rollnw MdlBinaryParser"
      - "nwnexplorer NwnMdl*"
      - "Radoub MdlBinaryReader"
    goal: "jedna tabela pol MDL, offsets, typy node, controller/keyframe"

  K2_mdx_policy_audit:
    priority: P0
    output: "documentation/mdx-polityka-codex.md"
    goal: "czy generujemy MDX, kiedy, jakie dane ida do MDL/MDX, jak testujemy"

  K3_hak_2da_writer_crosswalk:
    priority: P1
    output: "documentation/hak-2da-gff-crosswalk-codex.md"
    scope:
      - "borealis_nwn_resman"
      - "Radoub"
      - "neverwinter.nim"
      - "NWNFileFormats"
    goal: "minimalny writer spec dla 2DA + HAK"

  K4_viewport_validation_design:
    priority: P1
    output: "documentation/viewport-walidacja-design-codex.md"
    scope:
      - "nwn_mdl_webviewer"
      - "borealis_nwn_model_viewer"
      - "rollnw viewer"
    goal: "co pokazujemy w viewport przed HAK proof"

  K5_license_policy:
    priority: P1
    output: "sekcja w PROJECT_RULES.md albo osobny plik"
    goal: "jasne reguly: reference-only, no-copy, allowed external validators"
```

## 11. Konkluzja

Status: POTWIERDZONE + decyzja robocza.

Mamy dobry ekosystem referencyjny. Najsilniejszy zestaw dla `meshy2aurora`:

1. `C:\Projects\Claude\xoreos-docs` - formalny layout/spec.
2. `C:\Projects\Claude\borealis_nwn_mdl` - praktyczny MDL/animation/texture.
3. `C:\Projects\Claude\rollnw` - szeroki nowoczesny cross-check, renderer i resman.
4. `C:\Projects\Claude\Radoub` - praktyczny C# GFF/2DA/ERF/HAK/resman + testy.
5. `C:\Projects\Claude\neverwinter.nim` - NWN EE resman i narzedzia CLI.
6. `C:\Projects\Claude\borealis_nwn_resman` - C++ resource manager / HAK / GFF / 2DA.

Te repozytoria wystarczaja, zeby nie zgadywac, gdy dekompilacja Aurory nie daje odpowiedzi. Nie wystarczaja jednak, zeby pominac:

- wlasny parser/writer,
- TDD,
- wygenerowane test fixtures,
- proof w NWN EE.

To jest dokladnie model pracy: `Aurora First`, potem suplement repozytoriami, potem test/proof.
