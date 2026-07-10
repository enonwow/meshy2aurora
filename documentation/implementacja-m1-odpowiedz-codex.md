# implementacja-m1-odpowiedz-codex.md

Status 2026-07-10: HISTORYCZNE / SUPERSEDED przez D7-D12. Fakty o formatach i lokalnych sciezkach moga byc uzyte jako reference-only. Rekomendacje CLI/subprocess/importow z `aurora-web`, Node/TypeScript i proofu przez `aurora-web` sa uniewaznione. Aktywny M1A to Rust 1.96.1 + WASM adapter opisany w `prompt-dla-claude-prototyp-parsera.md`.
Data: 2026-07-08 | Odpowiada na: implementacja-m1-pytania-cloud.md

## Q1: Postac zrodlowego c_kocrachn.mdl
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\.codex-tmp\module-blob-mirror\modules\current\__aurora\sources\hak\cep3_core1\c_kocrachn.mdl`, `C:\Projects\aurora-web\docs\AUDYT_AREA_RENDER.md`)

`c_kocrachn.mdl` w mirrorze jest binary MDL, nie ASCII. Pierwsze bajty pliku to `00 00 00 00 10 29 01 00 5C 54 01 00 ...`, a w pierwszym bloku danych sa osadzone stringi `c_kocrachn` i `c_Horror`, bez tekstowych linii `newmodel`/`setsupermodel`.

Nie znalazlem ASCII zrodla dla `c_kocrachn` w `C:\Projects\New Folder` ani w mirrorze `C:\Projects\aurora-web\.codex-tmp\module-blob-mirror\modules\current\__aurora\sources`. Dla M1 rekomenduje golden test semantyczny: `binary c_kocrachn.mdl -> v13 GLB` kontra `nasz ASCII m2a_koc01.mdl -> v13 GLB`, porownane po polach v13, nie po liniach tekstu.

```yaml
q1_decision:
  source_model: c_kocrachn
  source_path: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\hak\\cep3_core1\\c_kocrachn.mdl"
  source_format: binary_mdl
  source_size_bytes: 163192
  golden_test_shape: semantic_glb_comparison
  baseline_path:
    input: "binary c_kocrachn.mdl"
    converter: "aurora-mdl-to-glb/v13-binary-model-header-supermodel-animationscale"
    output: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_kocrachn.glb"
  emitted_path:
    input: "ASCII m2a_koc01.mdl emitted by meshy2aurora"
    converter: "same v13 converter"
  compare_fields:
    - sourceModel/modelName
    - supermodel
    - animationScale
    - textureReferences
    - sourceHierarchy skin/trimesh/dummy node names
    - skin node names
    - vertex/triangle counts with tolerances
```

## Q2: Uruchamianie konwertera v13 z meshy2aurora
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\backend\package.json`, `C:\Projects\aurora-web\backend\scripts\aurora-mdl-to-glb.ts`, `C:\Projects\aurora-web\backend\src\modules\runtime-settings\adapters\outbound\derived\cli-derived-asset-generator.service.ts`)

Konwerter ma oficjalna sciezke CLI w backendzie `aurora-web`. Najprostsza komenda do M1:

```powershell
cd C:\Projects\aurora-web\backend
npm.cmd run assets:convert:mdl -- --input "C:\Projects\meshy2aurora\.codex-tmp\m1\m2a_koc01.mdl" --output "C:\Projects\meshy2aurora\.codex-tmp\m1\m2a_koc01.glb"
```

Na tej maszynie lokalnie: `node v24.15.0`, `npm 11.12.0`. Backend `aurora-web` ma `ts-node@10.9.2`, `typescript@5.9.3`, `jest@30.4.2`; `package.json` ma skrypt `"assets:convert:mdl": "ts-node --transpile-only scripts/aurora-mdl-to-glb.ts"`.

Import TS z `C:\Projects\aurora-web\backend\src\modules\runtime-settings\adapters\outbound\derived\aurora-mdl-ascii-to-glb.converter.ts` jest technicznie mozliwy, bo funkcje sa eksportowane, ale dla `meshy2aurora` rekomenduje CLI/subprocess, nie zaleznosc `file:` w `package.json`. Powod: `aurora-web\backend` nie jest wydzielonym pakietem bibliotecznym, a import przez ts-node wymaga cudzego `tsconfig`, cudzych zaleznosci i aktualnej struktury folderow aplikacji.

```yaml
converter_v13:
  recommended_invocation: cli_subprocess
  cwd: "C:\\Projects\\aurora-web\\backend"
  package_manager: npm
  script: "npm.cmd run assets:convert:mdl -- --input <model.mdl> --output <model.glb>"
  cli_file: "C:\\Projects\\aurora-web\\backend\\scripts\\aurora-mdl-to-glb.ts"
  cli_usage: "aurora-mdl-to-glb --input <model.mdl> --output <model.glb>"
  exported_functions:
    file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
    names:
      - inspectAuroraMdlSource
      - convertAuroraAsciiMdlToGlb
      - convertAuroraMdlToGlb
      - convertAuroraMdlFileToGlb
  local_versions:
    node: "v24.15.0"
    npm: "11.12.0"
    ts-node: "10.9.2"
    typescript: "5.9.3"
    jest: "30.4.2"
  do_not_do_for_m1:
    - "package.json dependency file:../aurora-web/backend"
    - "production import of aurora-web private TS source"
```

## Q3: Semantyka appearance.2da z haka
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\backend\src\modules\catalog\adapters\outbound\blob\module-snapshot-template-catalog.repository.ts`)

W `aurora-web` nie ma bezpiecznego "row merge" dla partial `appearance.2da` z HAK-a. W runtime direct path kod czyta pierwszy znaleziony HAK `appearance.2da` i dopiero jezeli nie ma zadnego HAK source, spada do vanilla. To oznacza: partial HAK zawierajacy tylko wiersz `9000` moze odciac vanilla lookup.

W snapshot index path moze istniec wiele zrodel `appearance.2da`, ale lookup wpisuje wiersz tylko jezeli `rowId` jeszcze nie istnieje. To nadal nie jest jawny merge patch; to "pierwszy wiersz wygrywa" po priorytecie zrodel.

Decyzja dla M3: pakowac pelny `appearance.2da` (retail/base + wiersz custom), nie sam wiersz.

```yaml
appearance_2da_policy:
  m3_decision: pack_full_appearance_2da
  partial_hak_row_only: forbidden
  aurora_web_runtime_direct:
    behavior: "first HAK appearance.2da source wins"
    vanilla_fallback: "only when no HAK appearance.2da source exists"
    source_function: readRuntimeCreatureAppearanceTwoDaSourceFromHakPaths
  aurora_web_snapshot_index:
    multiple_sources: true
    row_resolution: "first rowId wins"
    current_source_priority:
      - vanilla
      - hak
      - other_aurora_source
      - other
  implementation_note: "Do not rely on additive row merge for HAK appearance.2da."
```

## Q4: Rozwiazywanie supermodelu cross-source
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\backend\src\modules\catalog\application\catalog.application.service.ts`, `C:\Projects\aurora-web\frontend\src\modules\layout\adapters\three\workspaceObjectMeshLayer.ts`, `C:\Projects\aurora-web\.codex-tmp\module-blob-mirror\modules\current\__aurora\derived\models\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\c_horror.glb`)

Pipeline potrafi podlaczyc supermodel przez asset manifest, ale `c_horror.glb` musi byc obecny jako deliverable asset w derived/module cache. Backend dekoruje kandydatow modelem direct + `supermodelGlbUrl`/`supermodelModelUrl` oraz chainem supermodeli. Frontend potem laduje `supermodelModelUrl` i `supermodelChainModelUrls` przez `ensurePlaceableModel`, a animacje buduje przez `buildAuroraSupermodelAnimationClips`.

Aktualny mirror juz ma cross-source przyklad: `c_kocrachn.mdl` pochodzi z HAK `cep3_core1`, a `c_horror.mdl` z vanilla. Derived istnieje dla obu:

```yaml
supermodel_cross_source:
  direct_model:
    resref: c_kocrachn
    source: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\hak\\cep3_core1\\c_kocrachn.mdl"
    derived_glb: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_kocrachn.glb"
    derived_glb_bytes: 155892
  supermodel:
    resref_exact_from_reference: c_Horror
    normalized_lookup_resref: c_horror
    source: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\vanilla\\models\\c_horror.mdl"
    source_bytes: 393692
    derived_glb: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_horror.glb"
    derived_glb_bytes: 1970548
  hak_requirement:
    add_c_horror_to_hak: false_if_vanilla_sources_are_enabled_and_synced
    required_for_animation_inheritance: "c_horror.glb must exist in derived/module cache"
```

Dla `m2a_koc01.mdl` uzyj `setsupermodel m2a_koc01 c_Horror`. Nie dodawaj `c_horror` do HAK-a, jezeli proof idzie przez obecny `aurora-web` stack z `includeVanilla=true`. Dodaj albo dosyncuj `c_horror` dopiero w wariancie standalone bez vanilla.

## Q5: Runbook proofu dla nowego resref
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\docker-compose.yml`, `C:\Projects\aurora-web\backend\config\runtime-settings.json`, `C:\Projects\aurora-web\backend\scripts\capture-creatures-mode-cdp.mjs`, `C:\Projects\aurora-web\backend\src\modules\runtime-settings\adapters\inbound\http\runtime-settings.controller.ts`)

Obecnie dziala Docker stack. `docker compose ps` pokazuje zdrowy backend na `127.0.0.1:3000` i frontend na `127.0.0.1:5173`. W Dockerze nie trzeba `AURORA_HOST_LOCAL_RUNTIME=1`, bo `docker-compose.yml` ma `AURORA_DERIVED_ASSET_CONVERTER_COMMAND=node` i args do `/app/scripts/aurora-mdl-to-glb.ts`. Host-local runtime jest alternatywa dla backendu odpalonego bez Dockera.

Wazne: `capture-creatures-mode-cdp.mjs` przyjmuje `--resref`, nie `templateId`. Skrypt znajduje wiersz w `[aria-label='creatures mode list'] button.scene-object-row`. Dla pelnego proofu `m2a_koc01` musi byc rzeczywistym creature template w katalogu (UTC z resref/tag widocznym jako `m2a_koc01` plus appearance row). `--modelOverrideGlbPath` istnieje, ale to tylko diagnostyka modelu na istniejacym wpisie, nie proof calego HAK/2DA/UTC pipeline.

Blok do wklejenia po przygotowaniu HAK-a, np. `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m1.hak`:

```powershell
cd C:\Projects\aurora-web
docker compose up -d seaweedfs mysql redis backend frontend
docker compose ps

$api = "http://127.0.0.1:3000/api/v1"
$login = Invoke-RestMethod -Method Post -Uri "$api/auth/login" -ContentType "application/json" -Body (@{
  username = "admin.user"
  password = "admin123"
} | ConvertTo-Json)
$token = $login.data.session.accessToken
$headers = @{ Authorization = "Bearer $token" }

$settings = (Invoke-RestMethod -Headers $headers -Uri "$api/settings/runtime").data
$settings.paths.hakDirectoryPath = "/host/nwn-docs/hak"
$settings.sources.creature.includeHaks = $true
$settings.sources.creature.includeVanilla = $true
$settings.sources.creature.hakProfileId = "lc_creatures"
$settings.sources.creature.hakPriorityList = @(
  "m2a_m1.hak",
  "lc_2da.hak",
  "lc_appearance.hak",
  "lc_heads.hak",
  "lc_character.hak",
  "lc_hd_animals.hak",
  "lc_ccc.hak",
  "bdhd_pheno_other.hak",
  "bdhd_pheno_chest.hak",
  "bdhd_items.hak",
  "cep3_core0.hak",
  "cep3_core1.hak",
  "cep3_core2.hak",
  "cep3_facelift.hak",
  "cep3_heads.hak",
  "cep3_phenos.hak",
  "cep3_reforge.hak",
  "cep3_armor.hak",
  "cep3_portraits.hak",
  "vale-c_and_m.hak",
  "witcher.hak",
  "witcher2.hak"
)

Invoke-RestMethod -Method Put -Headers $headers -Uri "$api/settings/runtime" -ContentType "application/json" -Body ($settings | ConvertTo-Json -Depth 20)
Invoke-RestMethod -Method Post -Headers $headers -Uri "$api/settings/runtime/module-cache/rebuild?forceFull=true" | ConvertTo-Json -Depth 10

Invoke-RestMethod -Headers $headers -Uri "$api/catalog/creatures?query=m2a_koc01&limit=10" | ConvertTo-Json -Depth 10

node backend/scripts/capture-creatures-mode-cdp.mjs `
  --baseUrl http://127.0.0.1:5173 `
  --resref m2a_koc01 `
  --animationFallback first `
  --animationFrameTimes 0,0.5,1.0 `
  --recordVideo true `
  --videoSeconds 4 `
  --outDir "C:\Projects\meshy2aurora\.codex-tmp\proofs\m2a_koc01-cdp"
```

```yaml
cdp_proof_requirements:
  frontend_base_url: "http://127.0.0.1:5173"
  backend_api_url: "http://127.0.0.1:3000/api/v1"
  capture_script: "C:\\Projects\\aurora-web\\backend\\scripts\\capture-creatures-mode-cdp.mjs"
  capture_selector: "[aria-label='creatures mode list'] button.scene-object-row"
  capture_argument_for_target: "--resref m2a_koc01"
  requires_catalog_template: true
  synthetic_template_cli_supported: false
  diagnostic_only:
    modelOverrideGlbPath: "can override loaded model bytes, but does not prove UTC/2DA/HAK indexing"
  proof_artifacts:
    - "C:\\Projects\\meshy2aurora\\.codex-tmp\\proofs\\m2a_koc01-cdp\\summary.json"
    - "C:\\Projects\\meshy2aurora\\.codex-tmp\\proofs\\m2a_koc01-cdp\\*.png"
    - "C:\\Projects\\meshy2aurora\\.codex-tmp\\proofs\\m2a_koc01-cdp\\*.mp4"
```

## Q6: Konwencje repo meshy2aurora
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\backend\package.json`, `C:\Projects\aurora-web\backend\tsconfig.json`, `C:\Projects\aurora-web\backend\jest.config.js`, `C:\Projects\aurora-web\frontend\package.json`, `C:\Projects\meshy2aurora`)

`C:\Projects\meshy2aurora` nie ma jeszcze `package.json`; zawiera tylko `.git` i `documentation`. Dla M1 rekomenduje spojnosc z backendem `aurora-web`, bo to bedzie pipeline/CLI/testy Node, nie frontend UI:

```yaml
meshy2aurora_repo_conventions:
  package_manager: npm
  node:
    observed_local: "v24.15.0"
    recommended_engines: ">=22 <25"
    note: "aurora-web uses @types/node 22.x, local machine runs Node 24.x"
  typescript:
    version_family: "5.9.x"
    module: commonjs
    target: ES2022
    strict: true
    skipLibCheck: true
  tests:
    framework: jest
    transform: ts-jest
    reason: "backend aurora-web uses Jest for Node/backend tests"
  frontend_exception:
    framework: vitest
    when: "only if meshy2aurora later grows a Vite/React frontend"
  cross_repo_dependency:
    file_dependency_to_aurora_web_backend: false
    recommended_access_to_converter: "subprocess CLI with configurable AURORA_WEB_BACKEND path"
```

Nie kopiowac zaleznosci przez `file:../aurora-web/backend` do `package.json`. Jezeli M1 potrzebuje v13 jako oracle, dodaj w testach zmienna/konfiguracje typu `AURORA_WEB_BACKEND=C:\Projects\aurora-web\backend` i odpalaj CLI. Dopiero pozniej warto wydzielic wspolna biblioteke, jesli kod konwertera ma stac sie zaleznoscia produkcyjna.

## Q7: Wartosci do skopiowania z referencji
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\.codex-tmp\module-blob-mirror\modules\current\__aurora\derived\models\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\c_kocrachn.glb`, `C:\Projects\aurora-web\.codex-tmp\module-blob-mirror\modules\current\__aurora\sources\hak\cep3_core1\c_kocrachn.mdl`; zakres: wartosci semantyczne binary/v13)

Poniewaz zrodlo `c_kocrachn.mdl` jest binary, nie istnieja potwierdzone tekstowe linie ASCII do skopiowania 1:1. Mozna potwierdzic wartosci semantyczne z binary/v13 GLB:

```yaml
reference_c_kocrachn:
  model: c_kocrachn
  source_format: binary_mdl
  reconstructed_ascii_header_for_m1:
    setsupermodel: "setsupermodel c_kocrachn c_Horror"
    setanimationscale: "setanimationscale 0.72"
    classification: null
  confirmed_v13_values:
    supermodel: c_Horror
    animationScale_float: 0.7200000286102295
    animationScale_for_ascii_emit: 0.72
    sourceHierarchyNodeCount: 38
    sourceHierarchyNodeTypes:
      - dummy
      - skin
      - trimesh
    textureReferences:
      - c_kocrachn
    material:
      name: "Aurora material c_kocrachn"
      slot0:
        role: diffuse
        reference: c_kocrachn
    skinNodes:
      - name: Lshinmesh01
        nodeType: skin
        texture_slot0_reference: c_kocrachn
        vertices: 59
        triangles: 70
      - name: Rshinmesh01
        nodeType: skin
        texture_slot0_reference: c_kocrachn
        vertices: 59
        triangles: 70
      - name: bodymesh01
        nodeType: skin
        texture_slot0_reference: c_kocrachn
        vertices: 185
        triangles: 316
    totals:
      meshCount: 24
      totalVertices: 1311
      totalTriangles: 1130
```

`classification Character` wystepuje w ASCII fixture'ach konwertera, ale nie zostalo potwierdzone dla binary `c_kocrachn.mdl`: v13 GLB nie eksponuje pola `classification`, a prosty odczyt stringow z binary nie znalazl `Character`. Dla golden M1 nie asercjonowac `classification` jako "z referencji 1:1". Mozna emitowac `classification Character` jako konwencje ASCII creature model, ale oznaczac to w testach jako wymog emitera, nie jako fakt odczytany z `c_kocrachn`.

Podobnie `bitmap` kontra `texture0`: z binary/v13 potwierdzona jest semantyka slotu 0 (`c_kocrachn`), nie oryginalna nazwa tokena tekstowego. Golden powinien sprawdzac `textureReferences: ["c_kocrachn"]` i material slot 0, nie dokladny tekst `bitmap` vs `texture0`.
