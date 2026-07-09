# aurora-models-animations-audit-2026-07-08.md

Status 2026-07-09: HISTORYCZNY AUDYT WEJSCIOWY. Zachowac jako zrodlo kontekstu, ale aktualne granice implementacji i proofu sa w `PROJECT_RULES.md`, `decyzje-i-zadania-cloud.md` i `architektura-meshy2aurora-codex.md`.
Data: 2026-07-08

## Status

Status: POTWIERDZONE dla lokalnych kotwic w `C:\Projects\New Folder` i `C:\Projects\aurora-web`; HIPOTEZA dla decyzji MVP `meshy2aurora`; NIE WIEM tam, gdzie nie ma lokalnych danych lub runtime proofu.

## Zasada audytu

Aurora First: dekompilacja i lokalny kod sa zrodlem glownym. Internet jest tylko uzupelnieniem. Nie zgadujemy brakujacych limitow, nazw kosci ani animacji.

```yaml
sources:
  decompilation: "C:\\Projects\\New Folder"
  aurora_web: "C:\\Projects\\aurora-web"
  meshy2aurora_docs: "C:\\Projects\\meshy2aurora\\documentation"
  internet_used:
    - "https://docs.meshy.ai/en"
    - "https://docs.meshy.ai/en/api/rigging"
    - "https://docs.meshy.ai/en/api/animation"
```

## Najwazniejsze ustalenia

1. `aurora-web` renderuje gotowe `glb`/`gltf` przez `GLTFLoader`; MDL jest konwertowany po stronie backend/sync/tooling, nie w przegladarce.
2. Istniejacy backend ma konwerter `aurora-mdl-to-glb` i wersjonowany folder derived assets.
3. Creature animacje wymagaja nazw zgodnych z Aurora/NWN supermodel chain; nie ma jednej globalnej listy kosci dla wszystkich creature.
4. Dla MVP `meshy2aurora` najbezpieczniej zaczac od static GLB, potem direct creature, potem humanoid part/equipment creature.
5. Twardych limitow `max_bones` i `animation_fps` dla aurora-web nie potwierdzilem. Potwierdzone sa 4 wplywy na wierzcholek w parserze binary skin metadata.

## Potwierdzone kotwice dekompilacji

```yaml
decompilation_anchors:
  mdl_animation_parser:
    file: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
    strings:
      newanim: 881821
      setsupermodel: 881827
      setanimationscale: 881828
      event: "886456-886462"
      transtime: "886485-886487"
      animroot: "886491-886494"
  creature_fields:
    file: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
    fields:
      Appearance_Type: "194114, 195146"
      BodyPart_LFoot: "194501, 195178"
      BodyPart_RHand: "194531, 195208"
      BodyPart_LHand: "194533, 195210"
  placeable_fields:
    file: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
    fields:
      VisualTransform: 172875
      AnimationState_read: 329773
      AnimationState_write: 329889
```

## Potwierdzone kotwice aurora-web

```yaml
aurora_web_anchors:
  renderer:
    gltf_loader: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\layout\\adapters\\three\\workspaceThreeRuntime.ts:5,67,529"
    model_loader: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts:7587-7819"
  converter:
    cli: "C:\\Projects\\aurora-web\\backend\\scripts\\aurora-mdl-to-glb.ts"
    core: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
    version: "aurora-mdl-to-glb/v13-binary-model-header-supermodel-animationscale"
    version_file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\domain\\derived-asset-versions.ts"
  blob_mirror:
    root: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current"
    source_root: "__aurora\\sources"
    derived_root: "__aurora\\derived\\models"
  coordinate_scale:
    source_to_three_position: "x,y,z -> x,z,y"
    source_file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts:5323-5332"
    area_unit_scale: 0.1
    area_unit_scale_file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts:18"
```

## Creature i animacje

```yaml
creature_runtime_proofs:
  c_kocrachn:
    status: "POTWIERDZONE"
    source_mdl: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\hak\\cep3_core1\\c_kocrachn.mdl"
    supermodel_mdl: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\vanilla\\models\\c_horror.mdl"
    derived_glb: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_kocrachn.glb"
    proof_summary: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-source-skin-modelspace-v194\\summary.json"
    ok_count: 3
    failed_count: 0
    selected_animation: "cpause1"
    available_clip_count: 42
  x2_mephdrow011:
    status: "POTWIERDZONE z blockerem"
    proof_summary: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-mephdrow-bowshot-xbow-map-v197\\summary.json"
    selected_animation: "bowshot"
    available_clip_count: 202
    selected_track_count: 47
    selected_target_count: 46
    blocker: "creature_xbowshot_runtime_ammunition_direction_not_forward"
```

## Modele i assety powiazane

```yaml
related_model_systems:
  creature:
    depends_on:
      - "UTC/GIT fields"
      - "appearance.2da"
      - "CAPart/bodypart fields for modeltype P"
      - "MDL"
      - "supermodel chain"
      - "equipment attachment targets"
      - "textures/PLT/DDS/TGA"
  placeable:
    depends_on:
      - "UTP/GIT"
      - "placeables.2da"
      - "MDL/GLB"
      - "AnimationState"
      - "LightState/lightcolor/placeablelight"
      - "walkmesh"
  item:
    depends_on:
      - "UTI"
      - "baseitems.2da"
      - "model parts"
      - "armor parts"
      - "PropertiesList"
      - "icons/textures"
  area_tile:
    depends_on:
      - "ARE/GIT"
      - "SET"
      - "tile MDL"
      - "tile textures"
      - "light/shadow/fog fields"
```

## MVP rekomendacja

```yaml
mvp_recommendation:
  first:
    type: "static GLB placeable/item"
    reason: "validates Meshy GLB ingestion with lowest animation risk"
  second:
    type: "direct creature retarget"
    reference: "c_kocrachn"
    required_animation: "cpause1 or cwalk"
  third:
    type: "humanoid creature"
    reference: "x2_mephdrow011"
    known_risk: "weapon/ammunition direction and equipment animation"
acceptance:
  - "runtime proof summary.json exists"
  - "PNG canvas is nonblank"
  - "model is not placeholder"
  - "for creature, selected animation is playing or fallback is explicit"
  - "status POTWIERDZONE/HIPOTEZA/NIE WIEM recorded in docs"
```

## Otwarte blokery

```yaml
open_blockers:
  - id: "mesh_samples_missing"
    status: "NIE_WIEM"
    detail: "No local Meshy exports found yet."
  - id: "global_creature_bone_list"
    status: "NIE_WIEM"
    detail: "No single global skeleton; must derive from target supermodel chain."
  - id: "max_bones_limit"
    status: "NIE_WIEM"
    detail: "No hard aurora-web max bones limit found in this audit slice."
  - id: "humanoid_weapon_animation_parity"
    status: "POTWIERDZONE blocker"
    detail: "x2_mephdrow011 bowshot still fails ammunition direction proof in v197."
```
