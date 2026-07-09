# aurora-web-architektura-codex.md

Status 2026-07-09: REFERENCE-ONLY. To jest mapa osobnego projektu `C:\Projects\aurora-web`, nie architektura `meshy2aurora`. Aktywna architektura standalone jest w `architektura-meshy2aurora-codex.md`.
Data: 2026-07-08  
Status: POTWIERDZONE dla aktualnych modulow i sciezek; HIPOTEZA dla przyszlego endpointu/importera `meshy2aurora`

## Zakres

Mapa `C:\Projects\aurora-web` istotna dla `meshy2aurora`: source layer, runtime HAK settings, derived pipeline, catalog creature, frontend runtime/proof i env.

## Zrodla

```yaml
primary_sources:
  runtime_settings:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\application\\runtime-settings.application.service.ts"
    anchors:
      hak_supplement_extract: "1790-1870"
      hak_settings_resolution: "2134-2219"
      creature_template_from_hak: "2797-2836"
      texture_from_hak: "3222-3277"
      tile_model_from_hak: "3161-3198"
      placeable_model_from_hak: "3418-3460"
  erf_reader:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\nwn\\nwn-erf-packed-module-extractor.service.ts"
  derived_converter:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
  derived_generator:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\cli-derived-asset-generator.service.ts"
  derived_version:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\domain\\derived-asset-versions.ts"
  creature_catalog:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\catalog\\adapters\\outbound\\blob\\module-snapshot-template-catalog.repository.ts"
  creature_processing:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\catalog\\application\\catalog-processing.helpers.ts"
  frontend_loader:
    path: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
  cdp_proof:
    path: "C:\\Projects\\aurora-web\\backend\\scripts\\capture-creatures-mode-cdp.mjs"
```

## Warstwy danych

Status: POTWIERDZONE.

```yaml
source_layers:
  module:
    example: "modules/current/<module resources>"
  aurora_sources_hak:
    pattern: "modules/current/__aurora/sources/hak/<hakName>/<asset>"
    examples:
      - "__aurora/sources/hak/lc_core/h_fire.dds"
      - "__aurora/sources/hak/lc_2da/appearance.2da"
      - "__aurora/sources/hak/creature_parts/pfh2.mdl"
  aurora_sources_vanilla:
    pattern: "modules/current/__aurora/sources/vanilla/<asset>"
  derived_models:
    pattern: "modules/current/__aurora/derived/models/<versionSegment>/<resref>.glb"
```

Domyslny lokalny mirror blobow:

```yaml
env:
  AURORA_LOCAL_BLOB_MIRROR:
    default: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror"
  AURORA_HOST_LOCAL_RUNTIME:
    default: false
  converter_output_extension:
    default: "glb"
```

## Runtime HAK settings

Status: POTWIERDZONE.

`resolveHakSettingsByAsset` buduje ustawienia HAK per asset kind. Dla `placeable`, `creature`, `tile` moze uzyc listy z `module.ifo` jako fallback. Kolejnosc zrodel:

```yaml
hak_resolution_modes:
  disabled:
    when: "source.includeHaks == false"
    result: "hakPaths=[]"
  inline:
    when: "source.hakPriorityList not empty"
    result: "hakPaths from settings.paths.hakDirectoryPath + hakPriorityList"
  module_ifo:
    when: "fallback module IFO hak list not empty"
    result: "hakPaths from module.ifo"
  profile:
    when: "source.hakProfileId points to settings.hakProfiles"
    result: "hakPaths from selected profile"
  none:
    when: "includeHaks true but no list/profile/fallback"
    result: "hakPaths=[]"
```

## HAK source extraction

Status: POTWIERDZONE.

`loadSourceSupplementFiles` rozpakowuje HAK-i przez `packedModuleExtractor.extractFromPackedModule` i zapisuje:

```yaml
hak_source_sync:
  textures:
    destination: "__aurora/sources/hak/<hakName>/<texture.fileName>"
    detected_by: "isTextureFileName"
  placeable_assets:
    destination: "__aurora/sources/hak/<hakName>/<asset.fileName>"
    detected_by: "isPlaceableHakAssetFileName and not texture"
  creature_templates:
    destination: "__aurora/sources/hak/<hakName>/<asset.fileName>"
    detected_by: "isCreatureTemplateAssetFileName"
  sounds:
    destination: "__aurora/sources/hak/<hakName>/<asset.fileName>"
    detected_by: "isSoundAssetFileName"
```

Dodatkowe dedykowane czytniki potrafia czytac konkretne zasoby z HAK bez pelnego rozpakowania:

```yaml
direct_hak_reads:
  appearance_2da:
    function: "readRuntimeCreatureAppearanceTwoDaSourceFromHakPaths"
    resource: "appearance.2da"
    type: 2017
  capart_2da:
    function: "readRuntimeCreaturePartTwoDaSourceFromHakPaths"
    resource: "capart.2da"
    type: 2017
  wingmodel_tailmodel_2da:
    function: "readRuntimeCreatureAccessoryModelTwoDaSourceFromHakPaths"
    resources: ["wingmodel.2da", "tailmodel.2da"]
    type: 2017
  utc:
    function: "readCreatureTemplateFilesFromHak"
    type: 2027
  textures:
    function: "readHakTextureAssets"
    types: ["DDS", "TGA", "TXI", "MTR", "BMP", "PLT"]
```

## Derived pipeline MDL -> GLB

Status: POTWIERDZONE.

Konwerter:

```yaml
converter:
  path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
  input:
    - "ASCII MDL"
    - "binary MDL"
  output:
    extension: "glb"
    extras:
      - "sourceModel"
      - "converterVersion"
      - "supermodel"
      - "animationScale"
      - "sourceAnimationNames"
      - "sourceAnimations"
      - "sourceHierarchy"
      - "textureReferences"
      - "skinWeights per primitive/node"
  current_version:
    name: "aurora-mdl-to-glb/v13-binary-model-header-supermodel-animationscale"
    derived_path_segment: "aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale"
```

Przyklad potwierdzony z `c_kocrachn`:

```yaml
c_kocrachn_derived_glb:
  path: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_kocrachn.glb"
  sourceModel: "c_kocrachn"
  supermodel: "c_Horror"
  animationScale: 0.7200000286102295
  sourceHierarchyNodeCount: 38
  sourceHierarchyNodeTypes: ["dummy", "skin", "trimesh"]
  standardGltfSkinsCount: 0
  skinWeightNodeCount: 3
```

Wazne: `aurora-web` nie uzywa standardowego `THREE.SkinnedMesh` dla source-skin runtime; frontend odczytuje `extras.aurora.skinWeights` i robi CPU skinning na `THREE.Mesh`.

## Creature catalog

Status: POTWIERDZONE.

Kluczowe zachowanie:

```yaml
creature_catalog:
  appearance_sources:
    - "runtime HAK appearance.2da"
    - "vanilla fallback"
    - "blob snapshot sources"
  direct_model_resolution:
    field: "appearance.2da.RACE"
    condition: "MODELTYPE != P"
    confidence: "direct-model"
  part_model_resolution:
    condition: "MODELTYPE == P"
    confidence: "assembly-prefix"
    related_tables:
      - "capart.2da"
      - "wingmodel.2da"
      - "tailmodel.2da"
```

Dla nowego `meshy2aurora` creature krytyczne jest, zeby `appearance.2da` mial row id wskazany w UTC/GFF jako `Appearance_Type`, a `RACE` odpowiadal `newmodel`/plikowi MDL.

## Frontend runtime

Status: POTWIERDZONE.

```yaml
frontend_runtime:
  loader:
    path: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
  source_hierarchy_transform:
    position_mapping: "Aurora x,y,z -> Three x,z,y"
    quaternion_mapping: "Aurora x,y,z,w -> Three -x,-z,-y,w"
  supermodel_animation:
    builder: "buildAuroraSupermodelAnimationClips"
    requires:
      - "matching node names"
      - "sourceHierarchy"
      - "supermodel GLB clips"
  source_skinning:
    function: "buildAuroraSourceSkinningRuntime"
    data: "extras.aurora.skinWeights"
```

## Proof tooling

Status: POTWIERDZONE.

```yaml
cdp_creature_proof:
  script: "C:\\Projects\\aurora-web\\backend\\scripts\\capture-creatures-mode-cdp.mjs"
  useful_options:
    modelOverrideGlbPath:
      status: POTWIERDZONE
      caveat: "globalny interceptor .glb; moze podmienic takze supermodel, wiec to smoke test, nie final proof"
  expected_artifacts:
    - "summary.json"
    - "*-state.json"
    - "*.png"
    - "*.mp4"
```

## Jak nowy HAK wchodzi end-to-end

Status: POTWIERDZONE dla krokow w `aurora-web`; HIPOTEZA dla brakujacego UI/CLI `meshy2aurora`.

```yaml
end_to_end_steps:
  - step: "Generate HAK"
    actor: "meshy2aurora"
    output: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\<name>.hak lub profilowy hakDirectoryPath"
  - step: "Configure runtime HAK source"
    actor: "aurora-web settings"
    required:
      - "settings.sources.creature.includeHaks=true"
      - "hakPriorityList/profile/module_ifo zawiera <name>.hak"
      - "settings.paths.hakDirectoryPath wskazuje katalog z HAK"
  - step: "Run source supplement sync"
    actor: "aurora-web runtime-settings"
    result:
      - "__aurora/sources/hak/<hakName>/appearance.2da"
      - "__aurora/sources/hak/<hakName>/<model>.mdl"
      - "__aurora/sources/hak/<hakName>/<texture>.*"
  - step: "Generate derived model"
    actor: "cli-derived-asset-generator + aurora-mdl converter"
    result: "__aurora/derived/models/<version>/<model>.glb"
  - step: "Catalog creature"
    actor: "catalog module"
    result: "appearance.2da row resolves direct model candidate"
  - step: "Proof"
    actor: "CDP creature capture"
    result: "state.json + PNG/MP4"
```

Odpowiedz na pytanie "czy wystarczy dodac katalog do mirrora": NIE dla normalnego przeplywu. Trzeba, zeby runtime settings znaly HAK albo z `hakPriorityList`, albo z profilu, albo z `module.ifo`. Samo wrzucenie plikow do mirrora moze wystarczyc tylko jako reczny fixture/debug, nie jako potwierdzony end-to-end ingest.

## Testy TDD dla integracji

Status: REKOMENDACJA.

```yaml
tests:
  hak_settings:
    - "includeHaks=false returns empty hakPaths"
    - "inline hakPriorityList resolves to hakDirectoryPath + file names"
    - "module_ifo fallback resolves creature hak paths"
  source_sync:
    - "generated HAK with appearance.2da uploads __aurora/sources/hak/<hak>/appearance.2da"
    - "generated HAK with m2a_koc01.mdl uploads model source"
    - "generated HAK with TGA/DDS/PLT uploads texture source"
  derived:
    - "ASCII MDL direct creature converts to GLB with extras.aurora.sourceModel"
    - "skin weights metadata exists for skin node"
  catalog:
    - "appearance MODELTYPE=S yields direct-model candidate"
  proof:
    - "CDP state has selectedAnimationName cpause1"
    - "CDP summary ok == requested"
```
