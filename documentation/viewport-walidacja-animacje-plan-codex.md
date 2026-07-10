# viewport-walidacja-animacje-plan-codex.md

Data: 2026-07-09 | Autor: Codex | Status: AUDYT + PLAN FUNKCJONALNY

## 0. Cel dokumentu

Odpowiada na pytania:

- czy po wczytaniu modelu z Meshy moze byc widoczny w viewportcie,
- czy viewport powinien pokazywac model juz po przetworzeniu do postaci "jak w Aurorze",
- jak walidowac liczbe trojkatow / mesh parts / wag / tekstur,
- czy animacje nakladamy automatycznie,
- jak dodawac animacje,
- co z edycja.

Aktywny kierunek projektu pozostaje:

```text
Meshy GLB (FBX deferred after MVP)
  -> meshy2aurora
  -> binary MDL + MDX policy + 2DA + HAK
  -> NWN EE Toolset/gra
```

## 1. Zrodla internetowe

Status: POTWIERDZONE dla faktow z dokumentacji oficjalnej.

```yaml
internet_sources:
  meshy_image_to_3d:
    url: "https://docs.meshy.ai/en/api/image-to-3d"
    facts:
      - "Image to 3D przyjmuje obraz i zwraca zadanie generowania modelu."
      - "Parametr should_remesh kontroluje remesh."
      - "target_polycount istnieje i ma zakres zalezny od tieru."
      - "enable_pbr moze generowac PBR maps, a hd_texture moze dawac base color 4096x4096."
  meshy_remesh:
    url: "https://docs.meshy.ai/en/api/remesh"
    facts:
      - "Remesh przyjmuje modele .glb/.gltf/.obj/.fbx/.stl."
      - "target_formats obejmuje glb/fbx/obj/usdz/blend/stl/3mf."
      - "topology moze byc quad albo triangle."
      - "target_polycount domyslnie 30000, zakres 100-300000 zalezy od tieru."
  meshy_rigging:
    url: "https://docs.meshy.ai/en/api/rigging"
    facts:
      - "Rigging API jest dla humanoidow."
      - "Non-humanoid assets nie sa obecnie odpowiednim celem auto-riggingu."
      - "Dla input_task_id modele powyzej 300000 faces nie sa wspierane do riggingu."
      - "model_url wymaga teksturowanego GLB, przod postaci ma byc +Z."
  meshy_animation:
    url: "https://docs.meshy.ai/en/api/animation"
    facts:
      - "Animation API naklada akcje na wczesniej zriggowana postac przez rig_task_id i action_id."
      - "post_process moze zmienic FPS na 24/25/30/60."
      - "wyniki obejmuja animation_glb_url i animation_fbx_url."
  gltf_spec:
    url: "https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html"
    facts:
      - "glTF reprezentuje scene: nodes, transforms, hierarchy, meshes, materials, animations."
      - "mesh sklada sie z primitives; primitive odpowiada draw call i ma attributes, indices, material, mode."
      - "skinning: skins + joints + inverseBindMatrices + JOINTS_0/WEIGHTS_0."
      - "animations targetuja transformacje nodow i morph target weights; spec nie narzuca runtime playback."
  threejs_skinnedmesh:
    url: "https://threejs.org/docs/pages/SkinnedMesh.html"
    facts:
      - "SkinnedMesh laczy geometry z Skeleton."
      - "do skinningu potrzebne sa skin indices i weights."
      - "loadery takie jak GLTFLoader/FBXLoader typowo importuja takie modele."
```

## 2. Slownik: jak nazywac "shapy"

Najbezpieczniejsze nazwy w tym projekcie:

```yaml
vocabulary:
  gltf:
    node: "element sceny z transformem; moze wskazywac mesh, kamere, skina itp."
    mesh: "zbior primitives renderowanych jako jedna jednostka logiczna"
    primitive: "czesc mesha zwykle odpowiadajaca draw call; ma atrybuty, indeksy i material"
    triangle: "trojkat po triangulacji; najwazniejsza jednostka budzetu geometrii"
    vertex: "wierzcholek; w glTF atrybut POSITION/NORMAL/TEXCOORD/JOINTS/WEIGHTS"
    skin: "zbior joints + inverse bind matrices"
    joint: "node uzyty jako kosc skina"
    animation: "zestaw channels/samplers targetujacy transformy nodow lub morph targets"
    morph_target: "to, co w Blenderze bywa nazywane shape key; nie mylic z mesh primitive"
  aurora_mdl:
    node: "wezly modelu Aurora: dummy/trimesh/skin/danglymesh/itd."
    trimesh: "statyczna geometria"
    skin_node: "geometria skinowana/wazona do kosci"
    face: "twarz/trojkat w geometrii MDL"
    vert: "wierzcholek"
    tvert: "wspolrzedne UV"
    controller: "dane animacji/transformacji"
```

Odpowiedz na pytanie: "shapy" w kontekscie limitow to zwykle **mesh/primitives/triangles**, a nie shape keys. Dla Aurory interesuja nas szczegolnie: `triangles`, `vertices`, liczba `mesh nodes/skin nodes`, liczba kosci i wagi na vertex.

## 3. Viewport: czy mozemy odtworzyc model z Meshy?

Tak. Viewport powinien istniec bardzo wczesnie, ale musi pokazywac dwa tryby:

```yaml
viewport_modes:
  source_preview:
    input: "oryginalny Meshy GLB; FBX deferred after MVP"
    purpose: "zobaczyc, co przyszlo z Meshy bez udawania zgodnosci z Aurora"
    shows:
      - "raw mesh"
      - "raw materials/textures"
      - "raw skeleton/animations, jesli sa"

  aurora_preview:
    input: "model po pipeline meshy2aurora, z tych samych danych canonical, ktore ida do binary MDL writer"
    purpose: "pokazac jak najblizej finalnego modelu w Aurorze"
    shows:
      - "osie/rotacja po normalizacji"
      - "skala po dopasowaniu do bbox/policy"
      - "budzet geometrii po remesh/decymacji"
      - "tekstura po bake/resize do formatu docelowego"
      - "skeleton/skin po mapowaniu"
      - "animacje po mapowaniu, jezeli gate przeszedl"

  readback_preview:
    input: "wygenerowany binary MDL/MDX przeczytany z powrotem naszym parserem"
    purpose: "najuczciwszy preview finalnego outputu przed HAK/proofem"
    priority: "M2/M3; nie musi byc w pierwszym ekranie MVP"
```

Rekomendacja: po wczytaniu modelu od razu pokazujemy viewport, ale domyslnie w trybie `aurora_preview`, nie `source_preview`. Raw Meshy ma byc przelacznik/zakladka porownawcza.

## 4. Czy model po wczytaniu ma byc "sprawdzony"?

Tak. Po imporcie robimy automatyczny audit i pokazujemy model w stanie:

```yaml
import_pipeline:
  on_load:
    - "parse GLB only in MVP"
    - "collect stats"
    - "normalize axes preview"
    - "detect skeleton/skin/animations"
    - "detect textures/materials"
    - "run validation gates"
    - "show model in viewport with status badges"
  viewport_status:
    - "OK"
    - "WARN"
    - "BLOCKED"
  export_allowed_when:
    - "P0 gates pass"
    - "binary MDL/MDX policy known"
    - "2DA/HAK metadata valid"
```

## 5. Walidacja geometrii

Nie mamy jeszcze twardo potwierdzonego limitu Aurory dla kazdego typu modelu. Dlatego nie wpisujemy "engine max" z internetu. Wpisujemy budzet MVP i robimy go konfigurowalnym.

```yaml
geometry_budget_mvp:
  source: "projektowy gate do pierwszych proofow, nie twardy limit engine"
  creature:
    target_triangles: 1500
    warn_triangles: 5000
    block_triangles: 10000
  placeable:
    target_triangles: 1500
    warn_triangles: 8000
    block_triangles: 15000
  item:
    target_triangles: 300
    warn_triangles: 1000
    block_triangles: 3000
  texture:
    target_size: "512 or 1024"
    warn_size: "2048"
    block_size: "4096 until proven safe"
  skin:
    max_weights_per_vertex: 4
    require_normalized_weights: true
  resref:
    max_chars: 16
    charset: "lowercase ascii + digits + underscore"
```

Dlaczego tak: Meshy potrafi generowac bardzo geste modele i Remesh API oficjalnie pozwala targetowac polycount. Dla starego engine'u chcemy najpierw celowac konserwatywnie, a dopiero po proofach podnosic limity.

## 6. Co dokladnie walidujemy

```yaml
validation_gates:
  P0_block_export:
    - id: "mesh.triangles.block"
      check: "triangles <= type.block_triangles"
    - id: "mesh.vertices.present"
      check: "POSITION exists and count > 0"
    - id: "mesh.uv.present"
      check: "TEXCOORD_0 exists for textured output"
    - id: "texture.basecolor.present"
      check: "base color or baked diffuse exists"
    - id: "resref.valid"
      check: "resref <= 16 and valid chars"
    - id: "binary_mdl.policy.known"
      check: "MDX embedded/separate decision known before final HAK"

  P1_warn_allow_preview:
    - id: "mesh.triangles.warn"
      check: "triangles > target or warn threshold"
    - id: "mesh.primitives.many"
      check: "too many primitives/material splits"
    - id: "texture.size.warn"
      check: "texture > target"
    - id: "skeleton.missing"
      check: "no usable rig; can still use reference/supermodel path"
    - id: "animations.unmapped"
      check: "animations exist but no NWN mapping yet"

  P2_info:
    - "bbox dimensions"
    - "axis forward/up"
    - "materials count"
    - "mesh nodes / primitives count"
    - "skin joints count"
    - "animation clips and durations"
```

## 7. Czy nakladamy animacje automatycznie?

Tak, ale tylko warunkowo.

```yaml
animation_auto_policy:
  first_viewport_frame:
    default: "bind pose / neutral pose"
    reason: "najlatwiej ocenic skale, pivot, osie i rig"

  if_model_has_valid_skeleton_and_clips:
    action: "pokaz liste klipow i auto-preview pierwszego bezpiecznego idle/walk"
    allowed: "preview only until mapping to NWN names passes"

  if_nonhumanoid_creature_without_good_rig:
    action: "nie uzywac Meshy auto-rig jako zrodla prawdy"
    route: "reference skeleton / supermodel / wlasny transfer wag"

  if_humanoid_and_meshy_rigging_succeeded:
    action: "mozna previewowac basic animations i pozniej mapowac je do NWN names"
    caveat: "Meshy Rigging API oficjalnie celuje w humanoidy"
```

Czyli: viewport moze od razu odtwarzac animacje, ale eksport do Aurory moze uzyc animacji dopiero po przejsciu mapowania i walidacji.

## 8. Co jezeli chcemy dodac animacje?

Dodawanie animacji traktujemy jako osobny asset/workflow:

```yaml
add_animation_workflow:
  inputs:
    - "GLB animation clip z Meshy Animation API"
    - "GLB z Blender/inna aplikacja"
    - "FBX tylko po osobnej decyzji i implementacji importera po MVP"
    - "referencyjny clip z Aurory/NWN, read-only, jezeli legalnie dostepny lokalnie"
  steps:
    - "import animation source"
    - "sprawdz czy skeleton/joint names pasuja"
    - "jesli nie pasuja: retarget map"
    - "mapuj clip name -> NWN/Aurora name, np. pause1/walk/run/attack/death"
    - "ustal fps/sample rate; dla Meshy API dostepne 24/25/30/60 jako post_process"
    - "dodaj loop/one-shot/event metadata"
    - "preview in viewport"
    - "write animation controllers/newanim-equivalent into binary MDL"
    - "run animation gates"
```

Wazna zasada: nie "nakladamy animacji" przez samo wrzucenie GLB z animacja. Musimy miec zgodny skeleton, nazwy nodow/kosci, keyframes i mapowanie do tego, co Aurora/NWN rozumie.

## 9. Co z edycja?

Nie budujemy pelnego Blendera. Budujemy edytor konwersji i walidacji.

```yaml
editor_scope_mvp:
  metadata:
    - "resref"
    - "asset type: creature/placeable/item"
    - "appearance.2da row id"
    - "texture resref"
    - "animation mapping"

  transform:
    - "rotate/flip forward axis"
    - "scale to reference bbox"
    - "pivot/root offset"
    - "ground alignment"

  geometry:
    - "remesh target polycount"
    - "local decimation fallback"
    - "material merge/bake settings"
    - "segment preview: primitives/mesh nodes"

  texture:
    - "read-only Material/Texture Inspector: material -> primitive -> image -> UV"
    - "select base color/baseColor texture and replace the source image"
    - "tint color, hue, saturation, brightness and contrast"
    - "opacity and explicit alpha/cutout policy"
    - "UV flip, offset, scale and rotation"
    - "resize 512/1024"
    - "bake PBR -> diffuse TGA"
    - "reset recipe to untouched source GLB"

  rig_animation:
    - "choose reference skeleton path or Meshy rig path"
    - "map bones/joints"
    - "preview weights heatmap later"
    - "map animation clip names"
```

Poza zakresem MVP:

```yaml
not_mvp:
  - "manual pixel painting like a raster editor"
  - "reczne malowanie wag jak w Blenderze"
  - "pelne modelowanie siatki"
  - "timeline animation editor"
  - "zaawansowane rig authoring"
```

Jesli trzeba edytowac siatke/rigi recznie, robimy round-trip przez Blender i ponowny import do `meshy2aurora`.

### 9.1 Kontrakt edycji materialow i tekstur

Status: PLAN ZATWIERDZONY 2026-07-10.

Edycja tekstury nie zmienia oryginalnego GLB. Edytor zapisuje deklaratywna recipe w `m2a.project.json`; compiler ponownie buduje Aurora Preview oraz wynikowy TGA. Dzieki temu reset, porownanie i ponowny eksport sa deterministyczne.

```yaml
texture_editing_contract:
  source_preservation:
    source_glb: "read-only input; never overwritten"
    source_pbr_preview: "shows original GLB material state"
  editable_recipe:
    stored_in: "m2a.project.json"
    fields:
      - "selected base color texture or replacement image"
      - "tint RGBA"
      - "hue, saturation, brightness, contrast"
      - "opacity/alpha cutout policy"
      - "UV flip, offset, scale, rotation"
      - "target size, output texture resref and bake policy"
  derived_output:
    aurora_preview: "renders the baked/converted texture recipe"
    export: "writes generated TGA and optional TXI; never reuses a stale derived image"
  pbr_policy:
    rule: "normal, metallic-roughness, occlusion and emissive maps must be baked or explicitly reported as unsupported/discarded"
  validation:
    blocker:
      - "missing base color/diffuse output"
      - "invalid texture resref"
    warning:
      - "unsupported PBR map without an explicit bake/discard decision"
      - "texture dimensions over active policy"
```

## 10. Proponowana architektura viewportu

```yaml
viewport_architecture:
  ui:
    tabs:
      - "Source"
      - "Aurora Preview"
      - "Materials"
      - "Validation"
      - "Animations"
      - "Export"
  engine:
    recommended: "Three.js for preview only"
    rule: "preview engine nie jest walidatorem finalnym"
  data_flow:
    - "Meshy GLB parse in MVP"
    - "canonical model in memory"
    - "validation report"
    - "Aurora preview scene"
    - "binary MDL/MDX write"
    - "readback parse"
    - "HAK pack"
  important_rule:
    - "Viewport ma renderowac canonical converted model albo readback output, nie upiekszony raw Meshy model"
```

## 11. Format raportu walidacji

```json
{
  "asset": {
    "resref": "m2a_koc01",
    "type": "creature",
    "source": "sample-3d/m2a_koc01/source.glb"
  },
  "geometry": {
    "nodes": 12,
    "meshes": 1,
    "primitives": 3,
    "vertices": 1840,
    "triangles": 1460,
    "materials": 1
  },
  "textures": {
    "baseColor": "present",
    "target": "1024x1024 TGA",
    "warnings": []
  },
  "skeleton": {
    "hasSkin": true,
    "joints": 28,
    "maxWeightsPerVertex": 4,
    "weightsNormalized": true
  },
  "animations": {
    "clips": ["idle", "walk"],
    "mapped": ["pause1", "walk"],
    "unmapped": []
  },
  "gates": {
    "status": "WARN",
    "errors": [],
    "warnings": ["triangles above target 1500 by 60"]
  }
}
```

## 12. Plan implementacji

```yaml
phases:
  V0_import_stats:
    goal: "web import GLB + raport JSON"
    includes:
      - "triangle/vertex/material/primitive count"
      - "texture detection and material -> primitive -> image -> UV mapping"
      - "skin/animation detection"

  V1_viewport_source:
    goal: "pokaz raw Meshy GLB w viewportcie"
    caveat: "nie udawac zgodnosci z Aurora"

  V2_aurora_preview:
    goal: "pokaz model po normalizacji osi/skali/budzetu"
    includes:
      - "status badges"
      - "validation panel"
      - "source vs aurora preview toggle"
      - "read-only Material/Texture Inspector"

  V3_texture_editing:
    goal: "niedestrukcyjna korekta materialu i tekstury przed eksportem"
    includes:
      - "replace image, tint and basic color correction"
      - "alpha/cutout and UV transform policy"
      - "resize and bake to target TGA"
      - "recipe stored in m2a.project.json; reset to source"
    gates:
      - "derived Aurora Preview matches current recipe"
      - "texture validation passes"

  V4_animation_preview:
    goal: "lista klipow + bind pose + preview mapped animation"
    gates:
      - "valid skeleton"
      - "mapped clip name"
      - "weights pass"

  V5_export_readback:
    goal: "binary MDL/MDX writer + readback preview"
    reason: "najblizsze prawdzie 'tak bedzie w Aurorze'"

  V6_advanced_editor:
    goal: "zaawansowany niedestrukcyjny edytor konwersji"
    stores:
      - "m2a.project.json"
      - "validation-report.json"
      - "export-manifest.json"
    includes:
      - "history/revert of conversion recipes"
      - "advanced material and TXI/MTR profiles only after Aurora proof"
```

## 13. Decyzje do zamkniecia

```yaml
decisions:
  D_viewport:
    recommendation: "TAK, viewport jest czescia pierwszego wydania produktu, ale implementacyjnie zaczyna sie w S1 po M6 native proof"
    default_mode: "Aurora Preview"

  D_validation_budget:
    recommendation: "przyjac budzety MVP z tego dokumentu jako gates projektowe"
    note: "nie sa twardym limitem engine, dopoki NWN EE proofy tego nie potwierdza"

  D_animation_auto:
    recommendation: "auto-preview tak, auto-export tylko po mapping gates"

  D_animation_product_scope:
    recommendation: "wszystkie animacje wymagane przez wybrany profil Aurora/NWN musza miec potwierdzona sciezke: supermodel albo emisja wlasnego klipu"
    rule: "nie zamykac produktu samym bind pose albo jednym idle, gdy profil wymaga wiecej"

  D_editing:
    recommendation: "edytor konwersji tak; pelny modeler/rig editor poza MVP"

  D_first_focus:
    recommendation: "najpierw M1A-M6: parser/report + canonical IR + binary writer/readback + native proof; potem S1 source/Aurora viewport konsumujacy ten sam pipeline"
```

## 14. Najwazniejszy wniosek

Viewport ma sens tylko wtedy, gdy nie jest ladna atrapa. Powinien od poczatku odpowiadac na pytanie:

```text
czy ten model, po naszych transformacjach i walidacji, ma szanse stac sie dobrym binary MDL/HAK dla Aurory?
```

Dlatego minimalny produkt narzedzia to nie sam viewer. Minimalny produkt to:

```text
viewer + validation report + conversion settings + export gates
```

Animacje sa drugim krokiem: preview mozna dac szybko, ale zapis animacji do binary MDL wymaga mapowania skeletonu, nazw klipow i kontrolerow Aurory.
