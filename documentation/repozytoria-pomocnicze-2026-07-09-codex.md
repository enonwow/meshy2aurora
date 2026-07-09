# repozytoria-pomocnicze-2026-07-09-codex.md

Data: 2026-07-09  
Autor: Codex  
Status: AKTYWNY AUDYT DRUGIEJ LINII  
Zakres: repozytoria, ktore moga pomoc po wyczerpaniu dekompilacji Aurory

## 0. Zasada uzycia

To nie jest zmiana zasady Aurora First.

Kolejnosc szukania odpowiedzi dla implementacji `meshy2aurora`:

```yaml
source_order:
  1_aurora_first:
    path: "C:\\Projects\\New Folder"
    role: "glowne zrodlo wiedzy z dekompilacji Aurory"
    status: OBOWIAZKOWE_PIERWSZE
  2_local_game_assets:
    examples:
      - "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights"
      - "C:\\Users\\enonw\\Documents\\Neverwinter Nights"
    role: "read-only retail/CEP/NWN EE resources, proof przez NWN EE"
    status: PO_DEKOMPILACJI
  3_project_docs:
    path: "C:\\Projects\\meshy2aurora\\documentation"
    role: "lokalne decyzje, odpowiedzi, audyty, kontrakty"
    status: PO_DEKOMPILACJI
  4_external_repositories:
    role: "druga linia: referencje formatu, porownania, opcjonalne narzedzia debug"
    status: PO_WYCZERPANIU_AURORA_FIRST
  5_internet_forums_wiki:
    role: "uzupelnienie kontekstu, nie dowod implementacyjny"
    status: OSTATNIE
```

Repozytoria z tego dokumentu wolno wykorzystywac jako:

- reference-only,
- material porownawczy,
- zrodlo nazw/struktur do sprawdzenia w dekompilacji i runtime proof,
- opcjonalne narzedzia debug uruchamiane poza core pipeline,
- inspiracje do testow, ale nie jako oracle.

Nie wolno z nich robic automatycznie:

- zaleznosci produkcyjnej `meshy2aurora`,
- zrodla fixture/proof payload,
- substytutu naszego parsera/writera,
- dowodu dzialania w NWN EE,
- miejsca kopiowania kodu bez osobnej decyzji licencyjnej i architektonicznej.

## 1. Najkrotsza rekomendacja

```yaml
top_picks:
  mdl_binary_layout:
    primary:
      - "C:\\Projects\\Claude\\xoreos-docs\\templates\\NWN1MDL.bt"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl"
      - "C:\\Projects\\Claude\\cleanmodels"
    optional_crosscheck:
      - "https://github.com/jd28/rollnw"
      - "C:\\Projects\\Claude\\nwnexplorer\\nwnmdlcomp"
  mdl_runtime_viewport_reference:
    primary:
      - "C:\\Projects\\Claude\\nwn_mdl_webviewer"
      - "C:\\Projects\\Claude\\borealis_nwn_model_viewer"
    caveat: "nie sa proofem gry; tylko viewport/reference"
  erf_hak_writer_reference:
    primary:
      - "https://github.com/niv/neverwinter.nim"
      - "https://github.com/CromFr/nwn-lib-d"
      - "C:\\Projects\\Claude\\NWNFileFormats"
    optional:
      - "https://github.com/xoreos/xoreos-tools"
  gff_2da_tlk_reference:
    primary:
      - "https://github.com/niv/neverwinter.nim"
      - "https://github.com/CromFr/nwn-lib-d"
      - "C:\\Projects\\Claude\\NWNFileFormats"
      - "C:\\Projects\\Claude\\Radoub"
  texture_material_reference:
    primary:
      - "C:\\Projects\\Claude\\nwn_mdl_webviewer"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl"
    secondary:
      - "NeverBlender package audit"
```

## 2. Lokalne repozytoria juz na dysku

```yaml
local_repositories:
  xoreos_docs:
    path: "C:\\Projects\\Claude\\xoreos-docs"
    remote: "https://github.com/xoreos/xoreos-docs.git"
    last_commit_seen: "4e1c197 DOCS: Move NDS files to their own subdirectory"
    status: POTWIERDZONE_LOKALNIE
    priority: A
  borealis_nwn_mdl:
    path: "C:\\Projects\\Claude\\borealis_nwn_mdl"
    remote: "https://github.com/varenx/borealis_nwn_mdl.git"
    last_commit_seen: "bdc889c updated inherited skin data and bone counts"
    status: POTWIERDZONE_LOKALNIE
    priority: A
  borealis_nwn_model_viewer:
    path: "C:\\Projects\\Claude\\borealis_nwn_model_viewer"
    remote: "https://github.com/varenx/borealis_nwn_model_viewer.git"
    last_commit_seen: "16c8739 updated mdl library with supermodel chain fix"
    status: POTWIERDZONE_LOKALNIE
    priority: B
  cleanmodels:
    path: "C:\\Projects\\Claude\\cleanmodels"
    remote: "https://github.com/dunahan/cleanmodels.git"
    upstream_related: "https://github.com/plenarius/cleanmodels"
    last_commit_seen: "990e9a0 Add the rescale option"
    status: POTWIERDZONE_LOKALNIE
    priority: A
  nwn_mdl_webviewer:
    path: "C:\\Projects\\Claude\\nwn_mdl_webviewer"
    remote: "https://github.com/dunahan/nwn_mdl_webviewer.git"
    last_commit_seen: "dfb219c Merge pull request #178 from dunahan/v186-update-dropdown-menues"
    status: POTWIERDZONE_LOKALNIE
    priority: A_MINUS
  nwnexplorer:
    path: "C:\\Projects\\Claude\\nwnexplorer"
    remote: "https://github.com/dunahan/nwnexplorer.git"
    last_commit_seen: "56da6dc Bump version to 1.8.3"
    status: POTWIERDZONE_LOKALNIE
    priority: B
  nwn_file_formats:
    path: "C:\\Projects\\Claude\\NWNFileFormats"
    remote: "https://github.com/dunahan/NWNFileFormats.git"
    last_commit_seen: "a13236d Add a tool for diffing creatures."
    status: POTWIERDZONE_LOKALNIE
    priority: B_PLUS
  radoub:
    path: "C:\\Projects\\Claude\\Radoub"
    remote: "https://github.com/LordOfMyatar/Radoub.git"
    last_commit_seen: "d31cf86 Fix: GFF 64-bit field types"
    status: POTWIERDZONE_LOKALNIE
    priority: B_MINUS
  nwn_minimap:
    path: "C:\\Projects\\Claude\\nwn_minimap"
    remote: "https://github.com/dunahan/nwn_minimap.git"
    last_commit_seen: "e0bfb53 update from github"
    status: POTWIERDZONE_LOKALNIE
    priority: C
```

## 3. Repozytoria internetowe warte sledzenia

```yaml
internet_repositories:
  rollnw:
    url: "https://github.com/jd28/rollnw"
    local_copy: NIE
    priority: A_MINUS
    license: MIT
    status: POTWIERDZONE_INTERNET
  neverwinter_nim:
    url: "https://github.com/niv/neverwinter.nim"
    local_copy: NIE
    priority: A
    license: MIT
    status: POTWIERDZONE_INTERNET
  nwn_lib_d:
    url: "https://github.com/CromFr/nwn-lib-d"
    local_copy: NIE
    priority: A_MINUS
    license: GPL_3
    status: POTWIERDZONE_INTERNET
  xoreos_tools:
    url: "https://github.com/xoreos/xoreos-tools"
    local_copy: NIE
    priority: B
    license: GPL_3
    status: POTWIERDZONE_INTERNET
  xoreos_engine:
    url: "https://github.com/xoreos/xoreos"
    local_copy: NIE
    priority: B
    license: GPL_3
    status: POTWIERDZONE_INTERNET
  nwn_py:
    url: "https://github.com/niv/nwn.py"
    local_copy: NIE
    priority: C_PLUS
    license: MIT
    status: POTWIERDZONE_INTERNET
  pynwn_archive:
    url: "https://github.com/jd28-archive/pynwn"
    local_copy: NIE
    priority: C
    license: MIT
    status: POTWIERDZONE_INTERNET_ARCHIVED
  nwn_2da_static:
    url: "https://github.com/calgacus/NWN1EE_2DAs"
    local_copy: NIE
    priority: C
    license: NIE_WIEM
    status: POTWIERDZONE_INTERNET
```

## 4. Szczegolowa ocena

### 4.1 `xoreos-docs`

Status: POTWIERDZONE lokalnie i internetowo.  
Lokalnie: `C:\Projects\Claude\xoreos-docs`  
Internet: `https://github.com/xoreos/xoreos-docs`

Najwazniejsze elementy:

```yaml
xoreos_docs:
  key_files:
    - "C:\\Projects\\Claude\\xoreos-docs\\templates\\NWN1MDL.bt"
    - "C:\\Projects\\Claude\\xoreos-docs\\specs\\torlack\\binmdl.html"
    - "C:\\Projects\\Claude\\xoreos-docs\\specs\\bioware"
  useful_for:
    - "binary MDL layout"
    - "NWN1MDL 010 Editor template"
    - "ERF/resource specs"
    - "formal field order checks"
  use_policy: "reference-only; dobra kotwica po dekompilacji Aurory"
  risk:
    - "template/spec moze byc niepelny wzgledem NWN EE"
    - "nie jest runtime proof"
```

Ocena: A. To jest jedno z najlepszych zrodel drugiej linii dla layoutu binary MDL.

### 4.2 `borealis_nwn_mdl`

Status: POTWIERDZONE lokalnie i internetowo.  
Lokalnie: `C:\Projects\Claude\borealis_nwn_mdl`  
Internet: `https://github.com/varenx/borealis_nwn_mdl`

Repo opisuje sie jako biblioteka C++23 do NWN MDL i deklaruje:

- binary i ASCII MDL,
- texture loading: TGA, DDS, PLT,
- material parsing,
- animation playback,
- decompile back to ASCII.

```yaml
borealis_nwn_mdl:
  useful_for:
    - "porownanie binary MDL parsera"
    - "MDL + MDX load signature"
    - "skin data / inherited skin / bone counts"
    - "texture/material handling"
    - "animation playback concepts"
    - "debug ASCII decompile comparison"
  local_evidence:
    readme: "C:\\Projects\\Claude\\borealis_nwn_mdl\\README.md"
    header_hint: "include\\borealis\\nwn\\mdl"
    source_hint: "src"
  use_policy:
    - "czytac jako reference-only"
    - "nie kopiowac kodu do core"
    - "ewentualnie uruchomic jako external comparison po osobnej decyzji"
  risk:
    - "GPL-3.0"
    - "malo commitow na GitHub"
    - "C++23 / GLM / CMake, nie pasuje bezposrednio do ewentualnego TS/Node core"
    - "nie jest NWN EE proof"
```

Ocena: A dla wiedzy o MDL. Dla implementacji jako dependency: NIE bez osobnej decyzji.

### 4.3 `borealis_nwn_model_viewer`

Status: POTWIERDZONE lokalnie i internetowo.  
Lokalnie: `C:\Projects\Claude\borealis_nwn_model_viewer`  
Internet: `https://github.com/varenx/borealis_nwn_model_viewer`

Repo opisuje 3D viewer dla NWN MDL:

- binary i ASCII MDL parsing,
- decompile to ASCII,
- animation playback,
- skinned mesh rendering with bone transforms,
- particle system,
- TGA/DDS/PLT textures,
- model tree / hierarchy inspection,
- MDL source viewer.

```yaml
borealis_nwn_model_viewer:
  useful_for:
    - "porownanie jak viewer sklada hierarchy"
    - "animation playback i interpolation"
    - "skinned mesh render"
    - "texture/material render"
    - "manualny viewer reference"
  use_policy:
    - "pomocniczy viewport/reference"
    - "nie proof gry"
  risk:
    - "GPL-3.0"
    - "Qt6/OpenGL stack"
    - "viewer moze miec swoje uproszczenia wzgledem silnika"
```

Ocena: B. Przydatne jako porownanie dla naszego przyszlego viewportu.

### 4.4 `cleanmodels`

Status: POTWIERDZONE lokalnie i internetowo.  
Lokalnie: `C:\Projects\Claude\cleanmodels`  
Lokalny remote: `https://github.com/dunahan/cleanmodels.git`  
Powiazany upstream/release: `https://github.com/plenarius/cleanmodels`

Najwazniejsze lokalne pliki:

```yaml
cleanmodels:
  key_files:
    - "C:\\Projects\\Claude\\cleanmodels\\cleanmodels.pl"
    - "C:\\Projects\\Claude\\cleanmodels\\load_binary.pl"
    - "C:\\Projects\\Claude\\cleanmodels\\output_models.pl"
    - "C:\\Projects\\Claude\\cleanmodels\\make_checks.pl"
  useful_for:
    - "binary MDL -> internal model facts"
    - "binary MDL -> ASCII MDL decompile"
    - "skin weights, bone refs, normals, tangents"
    - "materialname, renderhint"
    - "smoothing groups / tverts / repairs"
    - "external debug decompiler"
  use_policy:
    - "external debug only"
    - "reference-only for parser behavior"
    - "nie oracle"
  risk:
    - "moze naprawiac/zmieniac semantyke"
    - "rescale/repair options moga maskowac bledy"
    - "nie wolno przepuszczac testu tylko dlatego, ze CleanModels naprawil model"
```

Ocena: A jako decompiler/reference; NIE jako dowod finalnego wyniku.

### 4.5 `nwn_mdl_webviewer`

Status: POTWIERDZONE lokalnie i internetowo.  
Lokalnie: `C:\Projects\Claude\nwn_mdl_webviewer`  
Internet: `https://github.com/dunahan/nwn_mdl_webviewer`

Repo opisuje browser-based viewer dla NWN1:EE binary i ASCII MDL. Deklarowane funkcje z README:

- ASCII MDL parser dla `trimesh`, `skin`, `danglymesh`, `animmesh`, `dummy`, `emitter`, `aabb`, `light`, `reference`,
- binary MDL przez WebAssembly CleanModelsEE,
- smoothing-group-aware normals,
- CPU Linear Blend Skinning w NWN Z-up,
- `NormalAndSpecMapped` / `NormalTangents`,
- `animmesh` UV animation,
- danglymesh simulation,
- supermodel chain,
- TGA/DDS/PLT/MTR/TXI/WOK/PWK/DWK/SET.

```yaml
nwn_mdl_webviewer:
  useful_for:
    - "viewport behavior reference"
    - "CPU skinning model-space vs world-space"
    - "animmesh/animtverts"
    - "danglymesh"
    - "MTR/TXI/material texture loading"
    - "PLT layer handling"
    - "supermodel chain viewer behavior"
    - "future meshy2aurora viewport feature list"
  use_policy:
    - "reference-only"
    - "moze inspirowac walidatory i viewport acceptance gates"
    - "nie traktowac jako engine proof"
  risk:
    - "viewer ma wlasny renderer, nie Aurora/NWN runtime"
    - "binary MDL path zalezy od embedded CleanModelsEE"
    - "lokalny README ma znaki spoza ASCII/mojibake; nie kopiowac doslownie do naszych docs"
```

Ocena: A- dla viewportu i walidatorow, B dla binary MDL jako cross-check.

### 4.6 `rollnw`

Status: POTWIERDZONE internetowo, brak lokalnej kopii.  
Internet: `https://github.com/jd28/rollnw`

Repo deklaruje:

- C++ i Python,
- implementacje wielu formatow NWN,
- model parser,
- resource manager dla ERF/KEY/NWSync/Zip,
- GFF/JSON object loading,
- MIT license,
- releases.

```yaml
rollnw:
  useful_for:
    - "file format comparison"
    - "resource manager design"
    - "model parser comparison"
    - "possible external validator later"
    - "Python bindings possibility"
  use_policy:
    - "internet reference first"
    - "nie klonowac, dopoki nie bedzie konkretnej luki"
    - "mozliwe narzedzie porownawcze po decyzji"
  risk:
    - "README mowi WIP"
    - "cele repo nie sa 'Aurora Engine way', tylko praktyczne tooling/engine"
    - "C++ stack"
```

Ocena: A- jako nowoczesna referencja ogolna, szczegolnie dla formatow i resource managera.

### 4.7 `neverwinter.nim`

Status: POTWIERDZONE internetowo, brak lokalnej kopii.  
Internet: `https://github.com/niv/neverwinter.nim`

Repo deklaruje biblioteke i CLI dla NWN:EE:

- `nwn_resman_stats`,
- `nwn_resman_grep`,
- `nwn_resman_extract`,
- `nwn_resman_cat`,
- `nwn_resman_diff`,
- `nwn_key_pack`,
- `nwn_key_unpack`,
- `nwn_key_shadows`,
- `nwn_key_transparent`,
- `nwn_gff`,
- `nwn_erf`,
- `nwn_tlk`,
- `nwn_twoda`,
- `nwn_compressedbuf`,
- `nwn_script_comp`,
- NWSync tools.

```yaml
neverwinter_nim:
  useful_for:
    - "ERF/HAK pack/unpack external validator"
    - "GFF JSON roundtrip"
    - "2DA transform/check"
    - "TLK transform/check"
    - "resman shadowing/priority diagnostics"
    - "NWSync later"
  use_policy:
    - "najlepszy kandydat na external CLI validator dla HAK/2DA/GFF"
    - "nie core dependency bez decyzji"
  risk:
    - "Nim stack"
    - "trzeba lokalnie pobrac albo uzyc release"
    - "nie rozwiazuje binary MDL writera"
```

Ocena: A dla ERF/GFF/2DA/TLK/resman. Dla MDL: C.

### 4.8 `nwn-lib-d`

Status: POTWIERDZONE internetowo, brak lokalnej kopii.  
Internet: `https://github.com/CromFr/nwn-lib-d`

Repo deklaruje tools i biblioteke dla NWN/NWN2:

- `nwn-gff`,
- `nwn-tlk`,
- `nwn-2da`,
- `nwn-erf`,
- reproducible ERF files,
- supported files: GFF, TLK, 2DA, TRN/TRX, limited DDS/MDB.

```yaml
nwn_lib_d:
  useful_for:
    - "reproducible HAK/ERF writer reference"
    - "2DA check/merge"
    - "GFF/TLK diffs"
    - "external validator"
  use_policy:
    - "external CLI/tool reference"
    - "nie binary MDL solution"
  risk:
    - "GPL-3.0"
    - "D language stack"
    - "wczesniejszy audyt: nie daje nam binary MDL decompiler path"
```

Ocena: A- dla HAK/2DA/GFF, C dla MDL.

### 4.9 `NWNFileFormats`

Status: POTWIERDZONE lokalnie i internetowo.  
Lokalnie: `C:\Projects\Claude\NWNFileFormats`  
Internet: `https://github.com/dunahan/NWNFileFormats` i historycznie `https://github.com/Liareth/NWNFileFormats`

Lokalny README mowi o:

- GFF,
- ERF,
- KEY,
- BIF,
- TLK,
- tools:
  - `2da_merge`,
  - `diff_creature`,
  - `generate_placeable_blueprints`,
  - `key_bif_extractor`,
  - `erf_extractor`.

```yaml
nwn_file_formats:
  useful_for:
    - "ERF/HAK reader/writer reference"
    - "KEY/BIF extraction"
    - "GFF model for UTC/UTI/module proof"
    - "2DA merge behavior"
    - "resource type mapping"
  use_policy:
    - "reference-only"
    - "mozliwe porownanie dla writerow"
  risk:
    - "C++17"
    - "nie jest MDL parser"
    - "lokalny fork moze miec zmiany wzgledem upstream"
```

Ocena: B+ dla HAK/2DA/GFF/KEY/BIF.

### 4.10 `nwnexplorer`

Status: POTWIERDZONE lokalnie i internetowo.  
Lokalnie: `C:\Projects\Claude\nwnexplorer`  
Internet: `https://github.com/dunahan/nwnexplorer`

Wazny podkatalog:

```yaml
nwnexplorer:
  key_paths:
    - "C:\\Projects\\Claude\\nwnexplorer\\nwnmdlcomp"
    - "C:\\Projects\\Claude\\nwnexplorer\\nwnmdlcomp\\nwnmdlcomp.cpp"
  useful_for:
    - "legacy NWN Model Compiler behavior"
    - "binary/ascii MDL compile/decompile legacy reference"
    - "NWN resource loader behavior"
  known_local_issue:
    - "lokalny nwnmdlcomp.exe wczesniej zwrocil: Unable to locate or open Neverwinter Night"
  use_policy:
    - "czytac jako historyczne zrodlo"
    - "nie oracle, dopoki lokalnie nie dekompiluje poprawnie"
```

Ocena: B jako historyczna referencja, C jako praktyczne narzedzie na tej maszynie dopoki nie przejdzie testu.

### 4.11 `Radoub`

Status: POTWIERDZONE lokalnie.  
Lokalnie: `C:\Projects\Claude\Radoub`  
Internet: `https://github.com/LordOfMyatar/Radoub`

Repo jest duzym toolsetem modderskim. Dla `meshy2aurora` nie jest glowne dla MDL, ale ma wartosc dla:

- GFF roundtrip,
- 2DA/TLK data service,
- HAK/BIF resource indexing,
- UTC/UTI/UTM tools,
- creature/inventory editor logic,
- duze lessons learned o custom content i cache.

```yaml
radoub:
  useful_for:
    - "GFF field safety"
    - "2DA/TLK game data lookup"
    - "HAK/BIF indexing and cache strategy"
    - "UTC creature blueprint editing semantics"
    - "future module/blueprint UI"
  use_policy:
    - "reference-only"
    - "nie MDL source of truth"
  risk:
    - "duzy projekt UI/toolset"
    - "latwo odpasc od M1 parser/writer"
```

Ocena: B- teraz, moze A dla pozniejszego edytora/blueprintow.

### 4.12 `xoreos-tools` i `xoreos`

Status: POTWIERDZONE internetowo, brak lokalnej kopii.  
Internet:

- `https://github.com/xoreos/xoreos-tools`
- `https://github.com/xoreos/xoreos`

`xoreos-tools` to narzedzia do reverse engineering Aurora-engine games. `xoreos` to open-source implementation Aurora engine derivatives.

```yaml
xoreos_family:
  useful_for:
    - "resource formats"
    - "ERF/TLK/GFF style tools"
    - "Aurora-family engine reference"
    - "cross-game caveats"
  use_policy:
    - "czytac selektywnie"
    - "u nas wazniejsze xoreos-docs niz caly engine"
  risk:
    - "GPL-3.0"
    - "bardzo szeroki zakres"
    - "nie zawsze NWN-specific"
```

Ocena: B. Najpierw `xoreos-docs`, potem dopiero kod `xoreos`/`xoreos-tools`.

### 4.13 `nwn.py` i `pynwn`

Status: POTWIERDZONE internetowo.

```yaml
python_family:
  nwn_py:
    url: "https://github.com/niv/nwn.py"
    status: "ALPHA according to README"
    use: "lekka Python reference dla formatow NWN:EE"
    priority: C_PLUS
  pynwn:
    url: "https://github.com/jd28-archive/pynwn"
    status: "ARCHIVED; moved to rollnw"
    use: "historyczne tylko jesli rollnw nie wystarczy"
    priority: C
```

Ocena: niska dla M1, chyba ze wybierzemy Python tooling.

### 4.14 Statyczne repo 2DA

Status: POTWIERDZONE internetowo.

Przyklady:

- `https://github.com/calgacus/NWN1EE_2DAs`
- `https://github.com/kucik/nwn-2da`

```yaml
static_2da_repos:
  useful_for:
    - "szybki podglad kolumn i nazw"
    - "diff historyczny"
  use_policy:
    - "tylko pomocniczo"
    - "lokalne retail/EE 2DA ma wyzszy priorytet"
  risk:
    - "wersja moze nie odpowiadac lokalnej instalacji"
    - "nie proof"
```

Ocena: C. Uzyc tylko gdy lokalne 2DA nie jest pod reka albo do porownania wersji.

## 5. Mapa problem -> repo

```yaml
problem_to_repo:
  binary_mdl_header_offsets:
    first:
      - "C:\\Projects\\Claude\\xoreos-docs\\templates\\NWN1MDL.bt"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl"
    second:
      - "C:\\Projects\\Claude\\cleanmodels\\load_binary.pl"
      - "https://github.com/jd28/rollnw"
      - "C:\\Projects\\Claude\\nwnexplorer\\nwnmdlcomp"
  skin_weights_bones:
    first:
      - "C:\\Projects\\Claude\\borealis_nwn_mdl"
      - "C:\\Projects\\Claude\\cleanmodels\\load_binary.pl"
      - "C:\\Projects\\Claude\\nwn_mdl_webviewer\\js\\animation.js"
    second:
      - "NeverBlender source package"
      - "https://github.com/jd28/rollnw"
  normals_tangents_smoothing:
    first:
      - "C:\\Projects\\Claude\\cleanmodels"
      - "C:\\Projects\\Claude\\nwn_mdl_webviewer"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl"
    second:
      - "NeverBlender source package"
  animation_system:
    first:
      - "C:\\Projects\\Claude\\borealis_nwn_mdl"
      - "C:\\Projects\\Claude\\borealis_nwn_model_viewer"
      - "C:\\Projects\\Claude\\nwn_mdl_webviewer"
    second:
      - "NeverBlender source package"
      - "https://github.com/jd28/rollnw"
  animmesh_animtverts:
    first:
      - "C:\\Projects\\Claude\\nwn_mdl_webviewer"
      - "NeverBlender source package"
    second:
      - "C:\\Projects\\Claude\\borealis_nwn_mdl"
  emitters_lights:
    first:
      - "C:\\Projects\\Claude\\nwn_mdl_webviewer"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl"
      - "NeverBlender source package"
  texture_tga_dds_plt:
    first:
      - "C:\\Projects\\Claude\\nwn_mdl_webviewer"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl"
      - "C:\\Projects\\Claude\\Radoub"
    second:
      - "https://github.com/jd28/rollnw"
  mtr_txi:
    first:
      - "C:\\Projects\\Claude\\nwn_mdl_webviewer"
      - "NeverBlender source package"
    second:
      - "C:\\Projects\\Claude\\borealis_nwn_mdl"
  erf_hak_writer:
    first:
      - "https://github.com/niv/neverwinter.nim"
      - "https://github.com/CromFr/nwn-lib-d"
      - "C:\\Projects\\Claude\\NWNFileFormats"
    second:
      - "https://github.com/xoreos/xoreos-tools"
  key_bif_resource_lookup:
    first:
      - "https://github.com/niv/neverwinter.nim"
      - "C:\\Projects\\Claude\\NWNFileFormats"
      - "C:\\Projects\\Claude\\Radoub"
    second:
      - "https://github.com/jd28/rollnw"
  appearance_2da:
    first:
      - "local retail/EE 2DA"
      - "https://github.com/niv/neverwinter.nim"
      - "https://github.com/CromFr/nwn-lib-d"
      - "C:\\Projects\\Claude\\NWNFileFormats"
    second:
      - "C:\\Projects\\Claude\\Radoub"
      - "static 2DA repos"
  utc_creature_blueprint:
    first:
      - "https://github.com/niv/neverwinter.nim"
      - "https://github.com/CromFr/nwn-lib-d"
      - "C:\\Projects\\Claude\\Radoub"
      - "C:\\Projects\\Claude\\NWNFileFormats"
```

## 6. Polityka licencji i kopiowania

```yaml
license_policy:
  safe_to_read_as_reference:
    - "all listed repos"
  copy_code_into_meshy2aurora:
    default: false
    requires:
      - "explicit decision"
      - "license review"
      - "architecture note"
      - "tests proving behavior independently"
  gpl_repos:
    examples:
      - "borealis_nwn_mdl"
      - "borealis_nwn_model_viewer"
      - "cleanmodels"
      - "nwn-lib-d"
      - "xoreos-tools"
      - "xoreos"
    policy:
      - "nie kopiowac kodu do core"
      - "mozna opisac zachowanie"
      - "mozna uruchomic jako zewnetrzne narzedzie debug po decyzji"
  mit_or_permissive_repos:
    examples:
      - "rollnw"
      - "neverwinter.nim"
      - "nwn_mdl_webviewer"
      - "nwn.py"
      - "pynwn"
      - "NWNFileFormats local README declares free copying"
    policy:
      - "nadal nie dodawac jako dependency bez decyzji"
      - "moga byc mocniejszym kandydatem na adapter/test helper"
```

## 7. Proponowany workflow badawczy

Kiedy dekompilacja Aurory nie daje odpowiedzi:

```yaml
research_workflow:
  step_1_define_gap:
    output: "jedno pytanie techniczne, np. MDL skin weights layout"
  step_2_check_local_docs:
    path: "C:\\Projects\\meshy2aurora\\documentation"
  step_3_check_exact_repo_bucket:
    examples:
      mdl: "xoreos-docs + borealis_nwn_mdl + cleanmodels"
      hak: "neverwinter.nim + nwn-lib-d + NWNFileFormats"
      viewport: "nwn_mdl_webviewer + borealis viewer"
  step_4_record_status:
    allowed_status:
      - POTWIERDZONE
      - HIPOTEZA
      - NIE_WIEM
  step_5_write_test_gate:
    rule: "jesli repo daje hipoteze implementacyjna, przed kodem piszemy test/gate"
  step_6_verify_with_nwn_ee:
    rule: "repozytorium nie zastepuje Toolset/game proof"
```

## 8. Co warto zrobic nastepnie

```yaml
next_actions:
  N1_clone_or_fetch_rollnw:
    status: DO_DECYZJI
    reason: "duzy nowoczesny reference stack z model parserem i resource managerem"
    recommendation: "nie teraz, dopiero gdy trafimy na konkretna luke"
  N2_get_neverwinter_nim_release:
    status: REKOMENDOWANE
    reason: "najpraktyczniejszy external validator dla ERF/GFF/2DA/TLK/resman"
    output: "external-tools report z wersja i komendami"
  N3_catalog_borealis_parser_files:
    status: REKOMENDOWANE
    reason: "lokalne repo wydaje sie najtrafniejsze dla binary/ascii MDL + textures + animations"
    output: "osobny mini-audyt plikow parsera Borealis"
  N4_catalog_cleanmodels_binary_layout:
    status: REKOMENDOWANE
    reason: "CleanModels ma konkretny load_binary.pl i output_models.pl"
    output: "mapa pol binary MDL porownana z xoreos NWN1MDL.bt"
  N5_external_validator_contract:
    status: REKOMENDOWANE
    reason: "zanim zaczniemy M1, trzeba ustalic ktore narzedzia moga byc WARN vs FAIL"
    output: "documentation/external-tools-policy-codex.md albo sekcja w architekturze"
```

## 9. Konkluzja

Najbardziej obiecujacy zestaw po Aurora First:

1. `xoreos-docs` dla formalnego layoutu binary MDL i specyfikacji.
2. `borealis_nwn_mdl` dla praktycznego parsera MDL, tekstur, materialow i animacji.
3. `cleanmodels` dla dekompilacji i sanity-checku binary MDL -> ASCII.
4. `nwn_mdl_webviewer` dla viewportu, skinningu, MTR/TXI/PLT i animacji w praktyce.
5. `neverwinter.nim` / `nwn-lib-d` / `NWNFileFormats` dla ERF/HAK/GFF/2DA/TLK.

Ten zestaw nie zmienia architektury. Core `meshy2aurora` nadal ma miec wlasny parser, wlasny writer i wlasny proof w NWN EE. Repozytoria z tej listy sa mapa ratunkowa, gdy dekompilacja Aurory i lokalne zasoby nie zamykaja pytania.

## 10. Dodatkowy GitHub search 2026-07-09

Status: POTWIERDZONE przez GitHub/web search, GitHub API i lokalne klony w `C:\Projects\Claude`.

Cel tej rundy: sprawdzic, czy poza juz znanymi repozytoriami istnieja podobne projekty, ktore moga pomoc przy `meshy2aurora` po wyczerpaniu dekompilacji Aurory.

```yaml
github_search_pass:
  date: "2026-07-09"
  method:
    - "GitHub/web search dla hasel NWN MDL parser/model viewer/ERF/HAK/GFF/2DA"
    - "GitHub API search dla hasel Neverwinter Nights MDL parser, NWN model viewer, NWN ERF HAK GFF 2DA"
    - "lokalny przeglad klonow w C:\\Projects\\Claude"
  most_relevant_confirmed:
    - "C:\\Projects\\Claude\\borealis_nwn_mdl"
    - "C:\\Projects\\Claude\\borealis_nwn_model_viewer"
    - "C:\\Projects\\Claude\\Radoub"
    - "C:\\Projects\\Claude\\nwn_mdl_webviewer"
    - "C:\\Projects\\Claude\\NWNFileFormats"
    - "C:\\Projects\\Claude\\Alia"
    - "C:\\Projects\\Claude\\Moneo"
    - "C:\\Projects\\Claude\\nwn_sqlite"
    - "C:\\Projects\\Claude\\nwn-mcp"
  new_internet_only_hits:
    - "https://github.com/varenx/borealis_nwn_resman"
    - "https://github.com/niv/nwn-lib"
    - "https://github.com/WilliamDraco/NWNT"
    - "https://github.com/Mingun/serde-gff"
    - "https://github.com/kucik/nwn-2da"
    - "https://github.com/dunahan/nwn_mdl_viewer_tauri"
    - "https://github.com/jd28-archive/mudl"
```

### 10.1 Nowe lub podniesione repozytoria

```yaml
new_or_reclassified_repositories:
  radoub:
    path: "C:\\Projects\\Claude\\Radoub"
    url: "https://github.com/LordOfMyatar/Radoub"
    priority_after_search: B_PLUS
    reason:
      - "lokalnie zawiera Radoub.Formats z GFF/2DA/TLK/KEY/BIF/ERF i readerami/writerami blueprintow"
      - "ma ErfReader/ErfWriter, GffReader/GffWriter, TwoDAReader oraz testy corrupted/round-trip"
      - "ma ModuleHakResolver i GameResourceResolver, czyli praktyczne resman/HAK lookup"
      - "ma MdlAsciiReader i MdlBinaryReader, wiec jest dodatkowym porownaniem dla MDL"
    key_local_paths:
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Erf\\ErfReader.cs"
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Erf\\ErfWriter.cs"
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Gff\\GffReader.cs"
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Gff\\GffWriter.cs"
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\TwoDA\\TwoDAReader.cs"
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Resolver\\ModuleHakResolver.cs"
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Mdl"
    caveat:
      - "GPL/copy risk: reference-only"
      - "to nie jest proof silnika NWN EE"

  borealis_nwn_mdl:
    path: "C:\\Projects\\Claude\\borealis_nwn_mdl"
    url: "https://github.com/varenx/borealis_nwn_mdl"
    priority_after_search: A
    reason:
      - "najcelniejsze lokalne repo dla MDL binary/ascii, animacji, skinning, texture/material"
      - "ma BinaryParser.cpp, BinaryWriter.cpp, AsciiParser.cpp, AsciiWriter.cpp"
      - "ma docs/BinaryMdlFormat.dox i docs/SupermodelSystem.dox"
    key_local_paths:
      - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\BinaryParser.cpp"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\BinaryWriter.cpp"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\AnimationPlayer.cpp"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl\\docs\\BinaryMdlFormat.dox"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl\\docs\\SupermodelSystem.dox"
    caveat:
      - "GPL/copy risk: reference-only"
      - "C++23, nie pasuje bezposrednio jako dependency do ewentualnego TS/Node core"

  borealis_nwn_resman:
    path: null
    url: "https://github.com/varenx/borealis_nwn_resman"
    priority_after_search: B
    reason:
      - "repo opisane jako resource manager library for NWN used by Borealis Toolset"
      - "moze pomoc przy kolejnej wersji resman/resource lookup dla HAK/BIF/override"
    caveat:
      - "nie ma lokalnego klonu"
      - "najpierw porownac z dekompilacja Aurory i naszym writerem HAK"

  nwn_lib_ruby:
    path: null
    url: "https://github.com/niv/nwn-lib"
    priority_after_search: B_MINUS
    reason:
      - "stara Ruby biblioteka do common NWN/NWN2 resource files"
      - "moze byc historycznym cross-checkiem dla GFF/ERF/2DA"
    caveat:
      - "starsza technologia"
      - "nizej niz neverwinter.nim, nwn-lib-d, Radoub i NWNFileFormats"

  nwnt:
    path: null
    url: "https://github.com/WilliamDraco/NWNT"
    priority_after_search: C_PLUS
    reason:
      - "GFF <-> NWNT text conversion"
      - "bazuje na nwn_gff z neverwinter.nim"
      - "moze pomoc w czytelnych diffach GFF, ale nie w core model pipeline"
    caveat:
      - "nie rozwiazuje MDL/MDX/HAK"

  serde_gff:
    path: null
    url: "https://github.com/Mingun/serde-gff"
    priority_after_search: C_PLUS
    reason:
      - "Rust implementation of BioWare GFF"
      - "moze byc dodatkowym cross-checkiem GFF field layout"
    caveat:
      - "GFF only"
      - "nie jest NWN-specific end-to-end proof"

  alia:
    path: "C:\\Projects\\Claude\\Alia"
    url: "https://github.com/dunahan/Alia"
    priority_after_search: C_PLUS
    reason:
      - "lokalnie opisuje sie jako GFF Reader/Writer for Neverwinter Nights"
      - "historycznie przydatne dla GFF round-trip i edge cases"
    caveat:
      - "stare narzedzie"
      - "nizej niz Radoub dla aktualnej implementacji"

  moneo:
    path: "C:\\Projects\\Claude\\Moneo"
    url: "https://github.com/dunahan/Moneo"
    priority_after_search: C
    reason:
      - "mass-edit GFF files of Neverwinter Nights"
      - "moze pomoc z mentalnym modelem batch edits na GFF"
    caveat:
      - "nie dotyczy MDL/MDX"
      - "raczej narzedzie historyczne"

  nwn_mcp:
    path: "C:\\Projects\\Claude\\nwn-mcp"
    url: "https://github.com/Txpple/nwn-mcp"
    priority_after_search: C_PLUS
    reason:
      - "TypeScript MCP server opakowujacy neverwinter.nim dla modulow .mod"
      - "ma przyklady tools/resman/gff-path/testow integracyjnych"
      - "moze inspirowac UX/automation pozniej, nie core converter"
    caveat:
      - "bazuje na neverwinter.nim"
      - "dotyczy modulow i edycji, nie Meshy -> MDL"

  nwn_sqlite:
    path: "C:\\Projects\\Claude\\nwn_sqlite"
    url: "https://github.com/dunahan/nwn_sqlite"
    priority_after_search: C
    reason:
      - "czyta informacje z modulow/GFF/2DA/TLK do SQLite"
      - "moze pomoc przy raportowaniu i indeksowaniu zasobow"
    caveat:
      - "nie pisze naszego HAK/MDL"

  nwn_mdl_viewer_tauri:
    path: null
    url: "https://github.com/dunahan/nwn_mdl_viewer_tauri"
    priority_after_search: C
    reason:
      - "desktop wrapper dla nwn_mdl_webviewer"
    caveat:
      - "nie wnosi nowego parsera ponad nwn_mdl_webviewer"

  mudl:
    path: null
    url: "https://github.com/jd28-archive/mudl"
    priority_after_search: C
    reason:
      - "archived proof-of-concept bgfx model viewer for NWN models"
    caveat:
      - "archived"
      - "opisuje sie jako proof-of-concept/not for public consumption"
```

### 10.2 Repozytoria podobne, ale niskiego priorytetu

```yaml
low_priority_or_out_of_scope:
  nwnxee_unified:
    url: "https://github.com/nwnxee/unified"
    reason: "runtime/server extender; moze byc ciekawe dla serwera, ale nie dla offline Meshy -> MDL/HAK"
  nwnsc:
    url: "https://github.com/nwneetools/nwnsc"
    reason: "NWScript compiler; przydatne dopiero przy proof module scripts"
  anvil:
    url: "https://github.com/nwn-dotnet/Anvil"
    reason: "server scripting API; nie dotyczy model conversion"
  eos_toolset:
    url: "https://github.com/Cjreek/Eos-Toolset"
    reason: "custom data/toolset workflow; mozliwe pozniej, ale nie P0"
  pykotor:
    url: "https://github.com/NickHugi/PyKotor"
    reason: "Aurora/Odyssey adjacent, ale KOTOR; last resort only"
  static_2da_repos:
    examples:
      - "https://github.com/kucik/nwn-2da"
      - "https://github.com/calgacus/NWN1EE_2DAs"
      - "https://github.com/Finaldeath/NWNEEGameData"
    reason: "snapshoty 2DA; nizszy priorytet niz lokalne retail/EE 2DA"
  tlk_2da_editors:
    examples:
      - "https://github.com/calgacus/TlkEdit-EE"
      - "https://github.com/DaedalusGame/NWNOver"
      - "https://github.com/Cavcode/aribethTool"
      - "https://github.com/Morderon/nwn_2da_tlkify"
    reason: "edytory danych, nie parser/writer MDL/HAK dla core pipeline"
```

### 10.3 Zaktualizowana mapa: problem -> gdzie patrzec

```yaml
problem_to_repo_after_extra_search:
  mdl_binary_writer:
    first_after_aurora:
      - "C:\\Projects\\Claude\\xoreos-docs\\templates\\NWN1MDL.bt"
      - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\BinaryWriter.cpp"
      - "C:\\Projects\\Claude\\cleanmodels"
    second:
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Mdl"

  mdl_animation_skinning_viewport:
    first_after_aurora:
      - "C:\\Projects\\Claude\\borealis_nwn_mdl\\src\\AnimationPlayer.cpp"
      - "C:\\Projects\\Claude\\borealis_nwn_model_viewer"
      - "C:\\Projects\\Claude\\nwn_mdl_webviewer"
    second:
      - "https://github.com/jd28-archive/mudl"

  hak_erf_writer:
    first_after_aurora:
      - "https://github.com/niv/neverwinter.nim"
      - "https://github.com/CromFr/nwn-lib-d"
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Erf\\ErfWriter.cs"
      - "C:\\Projects\\Claude\\NWNFileFormats"
    second:
      - "https://github.com/varenx/borealis_nwn_resman"

  gff_writer_and_diff:
    first_after_aurora:
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Gff"
      - "https://github.com/niv/neverwinter.nim"
      - "https://github.com/CromFr/nwn-lib-d"
    second:
      - "C:\\Projects\\Claude\\Alia"
      - "https://github.com/WilliamDraco/NWNT"
      - "https://github.com/Mingun/serde-gff"

  two_da_writer_and_static_data:
    first_after_aurora:
      - "local retail/EE 2DA files"
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\TwoDA"
      - "https://github.com/niv/neverwinter.nim"
      - "https://github.com/CromFr/nwn-lib-d"
    static_snapshot_only:
      - "https://github.com/kucik/nwn-2da"
      - "https://github.com/calgacus/NWN1EE_2DAs"
      - "https://github.com/Finaldeath/NWNEEGameData"

  module_resource_indexing_later:
    first_after_aurora:
      - "C:\\Projects\\Claude\\Radoub\\Radoub.Formats\\Radoub.Formats\\Resolver"
      - "C:\\Projects\\Claude\\nwn_sqlite"
      - "C:\\Projects\\Claude\\nwn-mcp"
```

### 10.4 Konkluzja po dodatkowym szukaniu

Wynik nie zmienia architektury, ale zmienia liste repo, ktore warto miec pod reka:

1. `C:\Projects\Claude\borealis_nwn_mdl` zostaje najlepszym drugim zrodlem dla MDL, animacji i skinningu.
2. `C:\Projects\Claude\Radoub` awansuje jako bardzo praktyczne lokalne repo dla GFF/2DA/ERF/HAK/resman i dodatkowo ma parsery MDL.
3. `https://github.com/varenx/borealis_nwn_resman` warto sklonowac dopiero, gdy wejdziemy w resource lookup/HAK/BIF resolver.
4. `https://github.com/niv/nwn-lib`, `https://github.com/WilliamDraco/NWNT`, `https://github.com/Mingun/serde-gff` sa dobrymi cross-checkami, ale nie core.
5. Statyczne repozytoria 2DA moga pomagac przy szybkim porownaniu nazw kolumn, ale canonical source dla implementacji pozostaja lokalne pliki gry/dekompilacja i proof w NWN EE.

Najwazniejsze: dodatkowe repozytoria pomagaja przewidziec bledy, ale nie uniewazniaja zasady `Aurora First` ani decyzji, ze `meshy2aurora` ma miec wlasny parser/writer.

## 11. Status lokalnych klonow po dociagnieciu 2026-07-09

Status: POTWIERDZONE LOKALNIE.  
Folder: `C:\Projects\Claude`

Uwaga: sekcja 3 byla snapshotem sprzed dociagniecia brakujacych repozytoriow. Aktualny stan lokalny jest ponizej.

Klonowanie wykonano jako shallow clone (`--depth 1`), bo repozytoria sa referencyjne/suplementacyjne, nie sa naszym core repo ani zaleznoscia produkcyjna.

```yaml
clone_policy:
  target_dir: "C:\\Projects\\Claude"
  clone_mode: "git clone --depth 1"
  use_as:
    - "reference-only"
    - "cross-check po Aurora First"
    - "material do testow i walidacji hipotez"
  not_use_as:
    - "automatyczna zaleznosc produkcyjna"
    - "oracle zamiast NWN EE proof"
    - "kod do kopiowania bez osobnej decyzji licencyjnej"
```

Nowo dociagniete repozytoria:

```yaml
newly_cloned_repositories:
  neverwinter_nim:
    path: "C:\\Projects\\Claude\\neverwinter.nim"
    remote: "https://github.com/niv/neverwinter.nim.git"
    branch: "master"
    commit: "db755db"
    shallow: true
  nwn_lib_d:
    path: "C:\\Projects\\Claude\\nwn-lib-d"
    remote: "https://github.com/CromFr/nwn-lib-d.git"
    branch: "master"
    commit: "7bbb614"
    shallow: true
  rollnw:
    path: "C:\\Projects\\Claude\\rollnw"
    remote: "https://github.com/jd28/rollnw.git"
    branch: "main"
    commit: "b3512c1"
    shallow: true
  xoreos_tools:
    path: "C:\\Projects\\Claude\\xoreos-tools"
    remote: "https://github.com/xoreos/xoreos-tools.git"
    branch: "master"
    commit: "b2ebf4f"
    shallow: true
  xoreos:
    path: "C:\\Projects\\Claude\\xoreos"
    remote: "https://github.com/xoreos/xoreos.git"
    branch: "master"
    commit: "89c99d2"
    shallow: true
  borealis_nwn_resman:
    path: "C:\\Projects\\Claude\\borealis_nwn_resman"
    remote: "https://github.com/varenx/borealis_nwn_resman.git"
    branch: "main"
    commit: "03f4d28"
    shallow: true
  nwn_lib:
    path: "C:\\Projects\\Claude\\nwn-lib"
    remote: "https://github.com/niv/nwn-lib.git"
    branch: "master"
    commit: "7041a0f"
    shallow: true
  nwnt:
    path: "C:\\Projects\\Claude\\NWNT"
    remote: "https://github.com/WilliamDraco/NWNT.git"
    branch: "main"
    commit: "692bd26"
    shallow: true
  serde_gff:
    path: "C:\\Projects\\Claude\\serde-gff"
    remote: "https://github.com/Mingun/serde-gff.git"
    branch: "master"
    commit: "2bfacbb"
    shallow: true
  nwn_2da:
    path: "C:\\Projects\\Claude\\nwn-2da"
    remote: "https://github.com/kucik/nwn-2da.git"
    branch: "master"
    commit: "4b071f6"
    shallow: true
  nwn1ee_2das:
    path: "C:\\Projects\\Claude\\NWN1EE_2DAs"
    remote: "https://github.com/calgacus/NWN1EE_2DAs.git"
    branch: "master"
    commit: "417bf2b"
    shallow: true
  nwnee_game_data:
    path: "C:\\Projects\\Claude\\NWNEEGameData"
    remote: "https://github.com/Finaldeath/NWNEEGameData.git"
    branch: "master"
    commit: "10f77144"
    shallow: true
  nwn_mdl_viewer_tauri:
    path: "C:\\Projects\\Claude\\nwn_mdl_viewer_tauri"
    remote: "https://github.com/dunahan/nwn_mdl_viewer_tauri.git"
    branch: "main"
    commit: "a1becb9"
    shallow: true
  mudl:
    path: "C:\\Projects\\Claude\\mudl"
    remote: "https://github.com/jd28-archive/mudl.git"
    branch: "main"
    commit: "5d6a9ab"
    shallow: true
```

Aktualny inwentarz `C:\Projects\Claude`:

```yaml
local_inventory_after_clone:
  repository_count: 27
  measured_total_bytes: 1217890023
  repositories:
    - "Alia"
    - "borealis_nwn_mdl"
    - "borealis_nwn_model_viewer"
    - "borealis_nwn_resman"
    - "cleanmodels"
    - "Moneo"
    - "mudl"
    - "neverwinter.nim"
    - "nwn_mdl_viewer_tauri"
    - "nwn_mdl_webviewer"
    - "nwn_minimap"
    - "nwn_sqlite"
    - "NWN1EE_2DAs"
    - "nwn-2da"
    - "NWNEEGameData"
    - "nwnexplorer"
    - "NWNFileFormats"
    - "nwn-lib"
    - "nwn-lib-d"
    - "nwn-mcp"
    - "NWNT"
    - "Radoub"
    - "rollnw"
    - "serde-gff"
    - "xoreos"
    - "xoreos-docs"
    - "xoreos-tools"
```
