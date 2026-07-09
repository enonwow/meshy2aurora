# meshy-input-odpowiedz-codex.md
Data: 2026-07-08 | Odpowiada na: meshy-input-pytania-cloud.md

## Q1: Format eksportu z meshy.ai
Status: POTWIERDZONE (zrodlo: `https://docs.meshy.ai/en`, `https://docs.meshy.ai/en/api/rigging`, `https://docs.meshy.ai/en/api/animation`)

Wedlug oficjalnych docs Meshy eksportuje m.in. `GLB`, `FBX`, `OBJ`, `STL`, `USDZ`, `3MF`. Dla web/Three.js najlepszy start to `GLB`. Meshy ma osobne kroki/API dla riggingu i animacji; animacja API dziala na zakonczonym zadaniu riggingu i moze zmieniac FPS na `24`, `25`, `30`, `60`.

```yaml
meshy_confirmed_formats:
  source: "https://docs.meshy.ai/en"
  exports:
    - "GLB"
    - "FBX"
    - "OBJ"
    - "STL"
    - "USDZ"
    - "3MF"
aurora_web_preferred_input:
  format: "GLB"
  reason: "aurora-web renderer consumes GLB/GLTF through GLTFLoader"
rigging_animation:
  rigging_api: "https://docs.meshy.ai/en/api/rigging"
  animation_api: "https://docs.meshy.ai/en/api/animation"
  animation_fps_options:
    - 24
    - 25
    - 30
    - 60
not_confirmed_locally:
  - "actual project Meshy export files"
  - "whether Mateusz wants Meshy auto-rig enabled for MVP"
```

## Q2: Typy modeli i priorytet
Status: HIPOTEZA (uzasadnienie: decyzja projektowa dla MVP, oparta na stanie aurora-web)

Rekomendowany priorytet MVP:

```yaml
mvp_priority:
  1:
    type: "placeable_static_or_item_static"
    reason: "lowest animation risk; validates GLB ingestion, scale, texture/material path"
  2:
    type: "creature_direct_model"
    reason: "tests Aurora skeleton/supermodel retarget without full humanoid part assembly"
    reference: "c_kocrachn"
  3:
    type: "humanoid_creature_part_model"
    reason: "highest risk: skeleton, equipment, part assembly, weapon animation"
    reference: "x2_mephdrow011"
deferred:
  - "full humanoid equipment parity"
  - "placeable light/shadow retail parity"
  - "VFX/progfx parity"
```

## Q3: Pliki testowe
Status: NIE WIEM

Nie znalazlem lokalnych eksportow Meshy w `C:\Projects\meshy2aurora`, `C:\Projects\aurora-web` ani `C:\Projects\New Folder`. Testowe eksporty musi dostarczyc Mateusz albo trzeba je wygenerowac przez Meshy.

```yaml
mesh_export_samples:
  local_samples_found: false
  required_from_mateusz:
    - "1 static GLB: prop/placeable"
    - "1 creature GLB without rig or with Meshy rig"
    - "1 animated/rigged humanoid GLB or FBX, if animation MVP is in scope"
  required_metadata_per_sample:
    - "source prompt or source image"
    - "Meshy export format"
    - "rigging enabled: true/false"
    - "animation clips exported: list"
    - "license/usage note"
```
