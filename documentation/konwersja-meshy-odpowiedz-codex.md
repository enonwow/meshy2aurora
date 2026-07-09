# konwersja-meshy-odpowiedz-codex.md

Status 2026-07-09: CZESCIOWO REFERENCE-ONLY. Pomiary z `aurora-web`/v13 sa heurystykami i materialem porownawczym, nie oracle `meshy2aurora`. Limity i transformacje trzeba utrwalac testami standalone.
Data: 2026-07-08 | Odpowiada na: konwersja-meshy-pytania-cloud.md

## Q1: Os forward modelu Aurora
Status: NIE WIEM (dokladny front bind-pose creature nie jest jeszcze potwierdzony)

Nie potwierdzam jeszcze finalnej odpowiedzi `+Y`, `-Y` albo `+X` jako faktu engine'u. Potwierdzone jest tylko to, ze:

- konwerter v13 mapuje pozycje MDL/Aurora do glTF jako `[x, z, y]`, czyli lokalnie traktuje eksport jako zmiane basis `Aurora -> glTF`;
- frontend przy odtwarzaniu `sourceHierarchy` robi analogiczna zamiane pozycji `sourceNode.position[0], sourceNode.position[2], sourceNode.position[1]`;
- orientacja instancji GIT jest liczona z `XOrientation/YOrientation` przez `atan2(-YOrientation, XOrientation)`, a area renderer potem odwraca yaw przez `resolveAuroraAreaRenderYaw(sourceYaw) = -sourceYaw`;
- mesh placement w aurora-web obraca obiekty wokol osi Y w Three, co odpowiada obrotowi wokol osi pionowej po konwersji basis.

Wniosek dla importu Meshy: nie wolno jeszcze na stale zakodowac rotacji "glTF +Z forward -> Aurora X/Y" bez malego testu wizualnego. Dla s3 dodac gate: model-test ze strzalka/kostka osi w GLB i porownanie po eksporcie z referencja creature w aurora-web albo Toolset.

```yaml
confirmed_transform_anchors:
  aurora_mdl_to_gltf_position:
    mapping: "[x, y, z] -> [x, z, y]"
    file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
    lines: "3476-3478"
  source_hierarchy_to_three_position:
    mapping: "[x, y, z] -> [x, z, y]"
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    lines: "5324-5333"
  source_hierarchy_to_three_quaternion:
    mapping: "[x, y, z, w] -> [-x, -z, -y, w]"
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    lines: "5349-5360"
  git_orientation_to_bearing:
    formula: "degrees(atan2(-YOrientation, XOrientation))"
    file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\areas\\adapters\\outbound\\blob\\aurora-gff-area-parser.ts"
    lines: "1781-1792"
  area_render_yaw:
    formula: "renderYaw = -sourceYaw"
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\layout\\adapters\\three\\workspaceAreaRenderCoordinates.ts"
    lines: "23-24"
blocked_decision:
  exact_creature_bind_forward_axis: "NIE WIEM"
  required_fixture: "axis-arrow Meshy GLB + reference creature screenshot/runtime proof"
```

## Q2: Jednostki Aurory
Status: NIE WIEM (1 MDL unit = 1 meter niepotwierdzone)

Nie potwierdzam tezy `1 MDL unit = 1 meter` jako faktu. Lokalna kotwica w aurora-web mowi cos innego dla Areas: source coordinates GIT sa normalizowane do tile-space, a aktualne mapowanie renderera operuje na szerokosci/wysokosci area w kaflach (`tileCol + 0.5`, `tileRow + 0.5`). To jest poprawne dla web-renderingu area, ale nie rozstrzyga absolutnej jednostki creature MDL w metrach.

Wniosek dla Meshy: skalowanie robic przez bbox referencji, nie przez globalne zalozenie metrow. Dla s2: `scale = target_reference_bbox_height / meshy_bbox_height_after_axis_conversion`.

```yaml
confirmed_area_tile_space:
  source_doc:
    file: "C:\\Projects\\aurora-web\\docs\\AUDYT_THE_LAST_CITY_AREA_2026-07-02.md"
    line: 146
    fact: "source coordinates sa juz w tile units"
  renderer_tile_centers:
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\layout\\adapters\\three\\workspaceAreaRenderCoordinates.ts"
    lines: "28-42"
    formulas:
      tile_center_x: "tileCol + 0.5"
      tile_center_z: "tileRow + 0.5"
implementation_rule:
  do_not_assume: "1 MDL unit = 1 meter"
  scale_strategy: "bbox_to_reference_model"
  first_reference_candidates:
    - "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\hak\\cep3_core1\\c_kocrachn.mdl"
    - "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\vanilla\\models\\c_horror.mdl"
```

## Q3: Budzet geometrii creature
Status: POTWIERDZONE (pomiary v13; twardy limit engine/derived v13 opisany jako NIE WIEM w odpowiedzi)

Zmierzylem realne `*.glb` wygenerowane przez v13 w mirrorze aurora-web. Zakres dla sprawdzonych creature: `424-1343` trojkaty w modelach referencyjnych bez animacji supermodelu i ok. `740-2201` vertexow. `c_kocrachn` ma `1130` trojkatow i `1311` vertexow. Wiekszy potwor w probce: `c_driderchf` ma `1343` trojkaty i `2201` vertexow; `c_bathorror` ma `1014` trojkatow i `2002` vertexow.

Nie znalazlem potwierdzonego twardego limitu v13/engine. Rekomendacja dla `m2a_*`: target decymacji `1000-1500` trojkatow dla pierwszego potwora, ostrzezenie powyzej `5000`, blokada TDD powyzej `10000` dopoki nie mamy runtime/oracle proof.

```yaml
measured_v13_creature_glbs:
  source_root: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale"
  models:
    - name: "c_kocrachn"
      file: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_kocrachn.glb"
      bytes: 155892
      meshes: 24
      primitives: 24
      vertices: 1311
      triangles: 1130
      skin_weight_nodes: 3
      max_influences_per_vertex: 2
      source_hierarchy_nodes: 38
      supermodel: "c_Horror"
    - name: "c_drider"
      vertices: 2022
      triangles: 1238
      meshes: 38
      source_hierarchy_nodes: 46
      supermodel: "c_driderchf"
    - name: "c_bugbearb"
      vertices: 1254
      triangles: 740
      meshes: 17
      source_hierarchy_nodes: 26
      supermodel: "c_bugbeara"
    - name: "c_goblinb"
      vertices: 742
      triangles: 424
      meshes: 20
      source_hierarchy_nodes: 29
      supermodel: "c_goblina"
    - name: "c_driderchf"
      vertices: 2201
      triangles: 1343
      meshes: 44
      source_hierarchy_nodes: 52
    - name: "c_bathorror"
      vertices: 2002
      triangles: 1014
      meshes: 9
      source_hierarchy_nodes: 20
recommended_m2a_geometry_budget:
  first_target_triangles: [1000, 1500]
  first_target_vertices_max: 2500
  warn_above_triangles: 5000
  reject_without_runtime_proof_above_triangles: 10000
  meshy_raw_30k_plus: "reject/remesh before MDL emission"
```

## Q4: Smoothing groups i normalne
Status: POTWIERDZONE (ASCII MDL -> GLB v13; dokladny engine normal solver opisany jako NIE WIEM w odpowiedzi)

ASCII MDL `faces` w obecnym parserze v13 traktuje 4. pole w wierszu face jako `smoothingGroup`, a pola 5-7 jako indeksy `tverts`. v13 czyta source normals, jezeli `mesh.normals.length === mesh.vertices.length`. Gdy source normals sa obecne, smoothing groups nie sa uzywane do wyliczenia normalnych. Gdy normals brak, v13 buduje normalne z smoothing groups: `0` oznacza normalna per-face, a niezerowe grupy lacza face'y przez bitowy overlap `(leftGroup & rightGroup) !== 0`.

Wniosek dla siatki Meshy:

- emitowac source normals, jezeli sa wiarygodne po decymacji i bake;
- rownolegle emitowac smoothing group jako zabezpieczenie: `0` dla twardych krawedzi i wspolna grupa/bit dla gladkich klastrow;
- nie dawac jednej globalnej smoothing group na caly model, bo potwor po decymacji bedzie mial rozmyte pazury, zeby i ostre krawedzie;
- w TDD sprawdzac, ze brak normals nadal daje deterministyczne normals z grup.

```yaml
ascii_face_layout_v13:
  vertex_indices: "row[0..2]"
  smoothing_group: "row[3]"
  tvert_indices: "row[4..6]"
  source:
    file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
    lines: "2508-2519"
normal_selection_v13:
  uses_source_normals_when: "mesh.normals.length > 0 && mesh.normals.length === mesh.vertices.length"
  source_lines:
    - "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts:2670-2673"
    - "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts:2798-2846"
smoothing_group_solver_v13:
  no_group_zero_behavior: "face:<faceIndex>, per-face normal"
  nonzero_merge_rule: "(leftGroup & rightGroup) !== 0"
  source: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts:2904-2992"
emit_policy_for_meshy:
  normals: "emit after axis conversion and decimation"
  smoothing_group:
    hard_edges: 0
    smooth_clusters: "bitmask per angle/material/UV island"
  angle_gate_degrees_initial: 45
```

## Q5: Tekstury - limity
Status: POTWIERDZONE (lokalne zasoby i loadery aurora-web; twardy limit retail engine opisany jako NIE WIEM w odpowiedzi)

W lokalnym mirrorze zasobow najwiekszy znaleziony TGA ma `1024x1024`, 32 bpp, typ `2` (uncompressed true-color), `4194348` bajtow: `t_karandas.tga`. Najwiekszy TGA 24 bpp ma tez `1024x1024`: `t_malagr.tga`. DDS w cache HAK dochodzi do `2048x2048` DXT1. To nie jest dowod twardego limitu engine'u, tylko potwierdzony zakres zasobow, ktore mamy na dysku.

TGA nie musi byc zawsze bez RLE jako format wejscia aurora-web: w probce jest `imageType=10` RLE true-color, a frontendowy item decoder obsluguje typy `2`, `3`, `10`, `11`. Dla generatora `m2a_*` rekomenduje jednak emitowac proste TGA `imageType=2`, `24 bpp` bez alpha albo `32 bpp` z alpha, bo to najlatwiejsza sciezka do testowania i najmniej ryzykowna w Toolset/engine.

```yaml
measured_texture_inventory:
  scanned_roots:
    - "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources"
    - "C:\\Projects\\aurora-web\\.codex\\creature-proof\\module-blob-mirror\\modules\\current\\__aurora\\sources"
  totals:
    tga: 117
    dds: 234
    plt: 211
  tga_types:
    type2_bpp24: 24
    type2_bpp32: 84
    type10_bpp32: 1
    type3_bpp8: 8
  max_tga:
    file: "C:\\Projects\\aurora-web\\.codex\\creature-proof\\module-blob-mirror\\modules\\current\\__aurora\\sources\\vanilla\\textures\\t_karandas.tga"
    width: 1024
    height: 1024
    bpp: 32
    image_type: 2
    bytes: 4194348
  max_tga_24bpp:
    file: "C:\\Projects\\aurora-web\\.codex\\creature-proof\\module-blob-mirror\\modules\\current\\__aurora\\sources\\vanilla\\textures\\t_malagr.tga"
    width: 1024
    height: 1024
    bpp: 24
    image_type: 2
    bytes: 3145772
  max_dds:
    file: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\hak\\bdhd_items\\a_towershield.dds"
    width: 2048
    height: 2048
    four_cc: "DXT1"
    bytes: 2796344
aurora_web_texture_loader_anchors:
  three_loaders:
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\layout\\adapters\\three\\workspaceThreeRuntime.ts"
    lines: "4-6, 518-520"
    loaders: ["TextureLoader", "DDSLoader", "TGALoader"]
  placeable_texture_flip:
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    line: 968
    value: "texture.flipY = false"
  backend_tga_summary:
    file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\domain\\aurora-tga-texture-summary.ts"
    lines: "31-46, 112-120"
  frontend_tga_decoder:
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\items\\application\\itemRawTextureDecoder.ts"
    lines: "116-120, 137-177"
recommended_m2a_texture_policy:
  default_size: "512x512"
  upper_size_without_extra_proof: "1024x1024"
  tga_image_type: 2
  tga_bpp: [24, 32]
  alpha: "32 bpp only when material needs cutout/transparency"
  avoid_initially: ["RLE as output", "color-mapped TGA", "PLT generation"]
```

## Q6: Orientacja UV
Status: POTWIERDZONE

v13 przy czytaniu tverts z MDL do GLB robi `1 - v` zarowno dla binarnego MDL, jak i ASCII MDL. To znaczy, ze MDL `tverts` i glTF `TEXCOORD_0` maja przeciwna orientacje V w aktualnym pipeline.

Wniosek dla eksportu Meshy glTF -> ASCII MDL: przy przepisywaniu `TEXCOORD_0` do `tverts` trzeba zrobic flip V:

```text
mdl_tvert_u = gltf_u
mdl_tvert_v = 1 - gltf_v
```

```yaml
uv_mapping:
  gltf_to_mdl:
    u: "u"
    v: "1 - v"
  mdl_to_gltf_confirmed_in_v13:
    binary_mdl_tverts:
      file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
      lines: "1320-1328"
      code: "tverts.push([u, 1 - v])"
    ascii_mdl_tverts:
      file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
      lines: "2245-2250"
      code: "mesh.tverts = [u, 1 - v]"
tdd_gate:
  fixture: "quad with four colored UV corners"
  assertion: "after MDL->GLB roundtrip colors are not vertically mirrored"
```

## Q7: Segmentacja skin nodes
Status: HIPOTEZA (legalnosc pojedynczego skin node w retail engine; zachowanie aurora-web/v13 potwierdzone w odpowiedzi)

`c_kocrachn` w v13 ma `3` skin weight nodes, `303` source weight vertices, `maxInfluencesPerVertex=2`, i liste kosci ograniczona do: `Lcalf2`, `Lfoot`, `Rcalf2`, `Rfoot`, `pelvis`, `torso`. v13 buduje `influencingBoneNames` per skin mesh na podstawie realnych `vertexWeights`; frontend potem czyta `skinWeights` z kazdego node'a i aplikuje source skinning per mesh. To oznacza, ze aurora-web nie wymaga "globalnie wielu skin nodes"; obsluguje kazdy mesh, ktory ma metadane `skinWeights`.

Nie mam jeszcze potwierdzenia z dekompilacji retail, ze jeden gigantyczny `skin` node z pelna lista kosci jest legalny i bezpieczny. Kod binarnego parsera v13 ma bezpieczny odczyt do `128` numerow bone node, ale to jest limit czytania parsera, nie potwierdzony limit engine'u.

Wniosek dla s6: w pierwszej implementacji zachowac segmentacje referencji. Dla potwora/czworonoga transferowac wagi per referencyjny skin node i ograniczac kandydatow do `influencingBoneNames` danego node'a. Dopiero osobny test moze sprawdzic wariant "single skin node".

```yaml
c_kocrachn_skin_measurement:
  file: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_kocrachn.glb"
  skin_weight_nodes: 3
  source_weight_vertex_count: 303
  max_influences_per_vertex: 2
  influencing_bone_names:
    - "Lcalf2"
    - "Lfoot"
    - "Rcalf2"
    - "Rfoot"
    - "pelvis"
    - "torso"
v13_skin_metadata_anchors:
  build_skin_weights:
    file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
    lines: "2578-2615"
    facts:
      - "skinWeights tylko dla nodeType === skin"
      - "influencingBoneNames zbierane z realnych vertexWeights"
      - "maxInfluencesPerVertex liczony z par bone/weight"
  binary_bone_number_read_guard:
    file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
    lines: "1503-1520"
    parser_safe_count_max: 128
frontend_source_skinning_anchors:
  read_per_node_skin_weights:
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    lines: "7137-7192"
  runtime_summary_influencing_bones:
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    lines: "7448-7455"
implementation_policy_s6:
  first_pass: "preserve reference skin node segmentation"
  weight_transfer_scope: "only bones from influencingBoneNames for the current reference skin node"
  max_influences_per_vertex_initial: 4
  normalize_weights: true
  single_skin_node_variant: "separate experiment, not default"
```

## Minimalny zestaw fixture dla s1-s7
Status: HIPOTEZA robocza na podstawie powyzszych gate'ow

```yaml
required_fixtures:
  meshy_input:
    required: true
    type: "real GLB from Meshy, even free/test"
    blocks: ["s1", "s2", "s3", "s4", "s5", "s6", "s7"]
  axis_orientation_probe:
    required: true
    contents:
      - "arrow or colored mesh marking glTF +Z forward"
      - "colored XYZ axes"
      - "asymmetric top/front marker"
    proves: ["Q1 forward mapping", "Q6 UV not mirrored if textured"]
  uv_probe:
    required: true
    contents: "single quad with labeled/color-coded UV corners"
    proves: ["Q6 flip V"]
  reference_creature_pair:
    required: true
    paths:
      - "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\hak\\cep3_core1\\c_kocrachn.mdl"
      - "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\sources\\vanilla\\models\\c_horror.mdl"
    proves: ["bbox scale", "skin segmentation", "animation/supermodel compatibility"]
```
