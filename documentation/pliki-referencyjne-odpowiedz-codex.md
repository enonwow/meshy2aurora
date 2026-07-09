# pliki-referencyjne-odpowiedz-codex.md

Status 2026-07-09: REFERENCE-ONLY. Sciezki i proofy z `C:\Projects\aurora-web` sa materialem porownawczym, nie dowodem dzialania `meshy2aurora`. Aktywna baza proofu to NWN EE + wygenerowany HAK/modul projektu.
Data: 2026-07-08 | Odpowiada na: pliki-referencyjne-pytania-cloud.md

## Q1: Model referencyjny
Status: POTWIERDZONE (zrodlo: lokalny mirror i proofy runtime w `C:\Projects\aurora-web`)

Model animowany referencyjny:

```yaml
animated_reference:
  resref: "c_kocrachn"
  source_mdl: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\hak\\cep3_core1\\c_kocrachn.mdl"
  supermodel_mdl: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\vanilla\\models\\c_horror.mdl"
  derived_glb: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_kocrachn.glb"
  derived_glb_bytes: 155892
  proof_summary: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-source-skin-modelspace-v194\\summary.json"
  proof_state: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-source-skin-modelspace-v194\\c_kocrachn-cpause1-state.json"
  proof_result:
    requested_count: 3
    ok_count: 3
    failed_count: 0
    selected_animation: "cpause1"
```

Statyczny model referencyjny do GLB/loader path:

```yaml
static_reference:
  resref: "ashto_011"
  source_mdl: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\hak\\bdhd_items\\ashto_011.mdl"
  derived_glb: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\ashto_011.glb"
  derived_glb_bytes: 28964
  runtime_visual_proof: "NIE_WIEM - nie znaleziono osobnego proof summary dla tego konkretnego resref w tej rundzie"
```

## Q2: Narzedzia do reuzycia
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web`)

```yaml
reusable_tools:
  aurora_mdl_to_glb:
    path: "C:\\Projects\\aurora-web\\backend\\scripts\\aurora-mdl-to-glb.ts"
    role: "CLI wrapper for MDL -> GLB conversion"
    usage: "aurora-mdl-to-glb --input <model.mdl> --output <model.glb>"
  in_process_converter:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
    role: "ASCII/binary Aurora MDL parsing and GLB metadata generation"
  derived_asset_generator:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\cli-derived-asset-generator.service.ts"
    role: "generates derived GLB/GLTF during module/blob sync"
  gltf_runtime_loader:
    path: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    role: "fetches/parses GLB/GLTF and merges Aurora extras into Three scene"
  creature_capture:
    path: "C:\\Projects\\aurora-web\\backend\\scripts\\capture-creatures-mode-cdp.mjs"
    role: "Chrome CDP proof capture for Creatures Mode"
  areas_capture:
    path: "C:\\Projects\\aurora-web\\backend\\scripts\\capture-areas-runtime-cdp.mjs"
    role: "Chrome CDP proof capture for Areas runtime"
external_tools:
  nwn_lib_d:
    status: "POTWIERDZONE jako opisane zrodlo pomocnicze w docs aurora-web, ale nie jako lokalny runtime dependency meshy2aurora"
    reference: "C:\\Projects\\aurora-web\\docs\\AUDYT_WATKU_019f189f_AREAS_DEKOMPILACJA_2026-07-02.md"
  NwnMdlComp:
    status: "NIE WIEM - nie potwierdzilem lokalnej sciezki binarki w tej rundzie"
```

## Q3: Weryfikacja wyniku
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\backend\scripts\capture-creatures-mode-cdp.mjs`, `C:\Projects\aurora-web\backend\scripts\capture-areas-runtime-cdp.mjs`, `C:\Projects\aurora-web\backend\src\common\config\env.schema.ts`)

Najmocniejsza weryfikacja dla creature to capture CDP z Creatures Mode, ktory zapisuje `summary.json`, `state.json`, PNG i opcjonalnie MP4. Dla area/placeable istnieje capture areas runtime.

```yaml
verification:
  local_runtime_env:
    AURORA_HOST_LOCAL_RUNTIME:
      file: "C:\\Projects\\aurora-web\\backend\\src\\common\\config\\env.schema.ts"
      default: false
      line: 47
    AURORA_LOCAL_BLOB_MIRROR:
      file: "C:\\Projects\\aurora-web\\backend\\src\\common\\config\\env.schema.ts"
      default: "../.codex-tmp/module-blob-mirror"
      line: 48
  creature_proof:
    script: "C:\\Projects\\aurora-web\\backend\\scripts\\capture-creatures-mode-cdp.mjs"
    example_route: "http://127.0.0.1:5173/app?mode=creatures_mode"
    expected_artifacts:
      - "summary.json"
      - "<resref>-<animation>-state.json"
      - "<resref>-<animation>-canvas.png"
      - "<resref>-<animation>-animation.mp4"
  area_proof:
    script: "C:\\Projects\\aurora-web\\backend\\scripts\\capture-areas-runtime-cdp.mjs"
    package_script: "backend/package.json -> areas:runtime-capture"
  pass_conditions_mvp:
    - "canvasNonblank or proof ok=true"
    - "model URL loaded"
    - "no placeholder/fallback if real model expected"
    - "for creature: requested or accepted fallback animation visible in state"
    - "summary failedCount == 0 for the selected MVP case"
```
