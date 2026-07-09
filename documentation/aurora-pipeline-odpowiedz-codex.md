# aurora-pipeline-odpowiedz-codex.md

Status 2026-07-09: REFERENCE-ONLY. Ten dokument opisuje pipeline `aurora-web`. Nie jest aktywnym planem implementacji `meshy2aurora` i nie uprawnia do uzycia `aurora-web` jako dependency, CLI, oracle, validator ani proof base.
Data: 2026-07-08 | Odpowiada na: aurora-pipeline-pytania-cloud.md

## Q1: Format konsumowany przez aurora-web
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\frontend\src\modules\layout\adapters\three\workspaceThreeRuntime.ts`, `C:\Projects\aurora-web\frontend\src\modules\placeables\adapters\three\placeableThreeAssetLoader.ts`, `C:\Projects\aurora-web\backend\src\modules\runtime-settings\adapters\outbound\derived\cli-derived-asset-generator.service.ts`)

Renderer aurora-web konsumuje gotowe modele `glb`/`gltf` przez `GLTFLoader`. Backend moze generowac pochodne `glb`/`gltf` z plikow `mdl` podczas synchronizacji assetow do blob storage / local mirror. To nie jest konwersja MDL w locie w przegladarce.

```yaml
renderer_model_loader:
  runtime_file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\layout\\adapters\\three\\workspaceThreeRuntime.ts"
  loader: "GLTFLoader"
  loader_lines:
    import: 5
    runtime_field: 67
    instance: 529
  load_file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
  entry_function: "ensurePlaceableModel"
  fetch_then_parse_lines: "7738-7819"
backend_derived_assets:
  env_file: "C:\\Projects\\aurora-web\\backend\\src\\common\\config\\env.schema.ts"
  output_extension_default: "glb"
  output_extension_line: 97
  converter_service: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\cli-derived-asset-generator.service.ts"
  in_process_converter_lines: "212-281"
  cli_converter: "C:\\Projects\\aurora-web\\backend\\scripts\\aurora-mdl-to-glb.ts"
  cli_usage: "aurora-mdl-to-glb --input <model.mdl> --output <model.glb>"
```

## Q2: Punkt wpiecia meshy2aurora
Status: HIPOTEZA (uzasadnienie: punkt wpiecia nie istnieje jeszcze jako kod `meshy2aurora`; potwierdzone sa tylko istniejace etapy aurora-web)

Rekomendowany punkt wpiecia MVP: `Meshy export -> meshy2aurora normalizacja/retarget -> gotowy GLB + manifest -> blob storage/local mirror -> renderer aurora-web`. Najmniej ryzykowna sciezka to generowac `glb` zgodny z aktualnym `GLTFLoader` i z metadanymi potrzebnymi do animacji/retargetu. Sciezka `Meshy -> MDL -> istniejacy MDL->GLB` jest slabsza na start, bo wymaga pelniejszego emitera Aurora MDL.

```yaml
proposed_mvp_pipeline:
  input:
    source: "Meshy export"
    preferred_format: "glb"
  meshy2aurora:
    outputs:
      - "model.glb"
      - "manifest.json"
      - "optional texture files only if not embedded"
    required_work:
      - "scale_axis_normalization"
      - "aurora_resref_name"
      - "creature_supermodel_retarget_metadata"
      - "runtime_proof_case"
  aurora_web_reuse:
    model_loader: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    blob_storage: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\application\\runtime-settings.application.service.ts"
    existing_mdl_converter: "C:\\Projects\\aurora-web\\backend\\scripts\\aurora-mdl-to-glb.ts"
  not_confirmed_yet:
    - "automatic registration API for arbitrary external Meshy GLB in aurora-web catalog"
    - "final object key convention for non-module Meshy assets"
```

## Q3: Konwencje nazewnicze assetow
Status: POTWIERDZONE (zrodlo: lokalny mirror `C:\Projects\aurora-web\.codex-tmp\module-blob-mirror\modules\current\__aurora`, `C:\Projects\aurora-web\backend\src\modules\runtime-settings\domain\derived-asset-versions.ts`)

W aktualnym mirrorze zrodla MDL i pochodne GLB sa rozdzielone. Nazwa modelu jest resrefem pliku bez rozszerzenia. Derived GLB sa pod katalogiem wersji konwertera.

```yaml
module_blob_mirror:
  root: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current"
  source_models:
    root: "__aurora\\sources"
    examples:
      - path: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\hak\\cep3_core1\\c_kocrachn.mdl"
        resref: "c_kocrachn"
        bytes: 163192
      - path: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\hak\\lc_addanimations\\a_fa.mdl"
        resref: "a_fa"
  derived_models:
    root: "__aurora\\derived\\models"
    converter_version: "aurora-mdl-to-glb/v13-binary-model-header-supermodel-animationscale"
    converter_version_file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\domain\\derived-asset-versions.ts"
    versioned_folder: "__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale"
    examples:
      - path: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_kocrachn.glb"
        resref: "c_kocrachn"
        bytes: 155892
      - path: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\ashto_011.glb"
        resref: "ashto_011"
        bytes: 28964
```

## Q4: Uklad wspolrzednych i skala
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\frontend\src\modules\layout\adapters\three\workspaceAreaRenderCoordinates.ts`, `C:\Projects\aurora-web\frontend\src\modules\placeables\adapters\three\placeableThreeAssetLoader.ts`, `C:\Projects\aurora-web\frontend\src\modules\layout\adapters\three\workspaceObjectMeshLayer.ts`, `C:\Projects\aurora-web\frontend\src\modules\layout\adapters\three\workspaceTerrainMesh.ts`)

Three.js runtime jest `Y-up`. Aurora source hierarchy jest mapowana do Three jako `x/y/z -> x/z/y`. Dla area rendering obiekty i tile maja skale `0.1` dla jednostek modelu Aurory do tile-space.

```yaml
coordinate_system:
  three_up_axis: "Y"
  source_hierarchy_position_mapping:
    source: "Aurora x,y,z"
    three: "x,z,y"
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    lines: "5323-5332"
  area_coordinate_mapping:
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\layout\\adapters\\three\\workspaceAreaRenderCoordinates.ts"
    x: "areaWidth - sourceX"
    z: "sourceY"
    yaw: "-sourceYaw"
  area_layer_mirror:
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\layout\\adapters\\three\\workspaceObjectMeshLayer.ts"
    lines: "15220-15245"
    mirror: "coordinate-x"
  source_unit_scale:
    placeable_area_model: 0.1
    tile_model: 0.1
    files:
      - "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts:18"
      - "C:\\Projects\\aurora-web\\frontend\\src\\modules\\layout\\adapters\\three\\workspaceTerrainMesh.ts:41"
```
