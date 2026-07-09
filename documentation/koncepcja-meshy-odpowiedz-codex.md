# koncepcja-meshy-odpowiedz-codex.md

Status 2026-07-09: HISTORYCZNE / REFERENCE-ONLY. Odpowiedz dotyczy starego wariantu GLB/aurora-web. Nie jest planem implementacji standalone po D7-D8.
Data: 2026-07-08 | Odpowiada na: koncepcja-meshy-pytania-cloud.md

## Q1: Wykonalnosc strategii B wzgledem GLB v13
Status: POTWIERDZONE (zrodla: `C:\Projects\aurora-web\backend\src\modules\runtime-settings\adapters\outbound\derived\aurora-mdl-ascii-to-glb.converter.ts`, `C:\Projects\aurora-web\backend\src\modules\runtime-settings\domain\derived-asset-versions.ts`, `C:\Projects\aurora-web\frontend\src\modules\placeables\adapters\three\placeableThreeAssetLoader.ts`, `C:\Projects\aurora-web\.codex-tmp\module-blob-mirror\modules\current\__aurora\derived\models\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\c_kocrachn.glb`)

Tak, GLB emitowany przez `meshy2aurora` moze byc ladowany przez ten sam loader, jezeli jest zapisany jako zwykly GLB oraz niesie kontrakt `extras.aurora` zgodny z derived v13. Sam `GLTFLoader.parse` zaladuje tez zwykly GLB, ale dla zgodnosci z Aurora runtime to nie wystarcza: `resolvePlaceableGltfScene` scala `extras.aurora` do `userData`, syntetyzuje/odtwarza hierarchie z `sourceHierarchy`, buduje klipy z `sourceAnimations`, a `buildAuroraSupermodelAnimationClips` retargetuje animacje po nazwach nodow.

Dla normalnej sciezki katalogowej plik powinien trafic do aktualnego segmentu derived:

```text
C:\Projects\aurora-web\.codex-tmp\module-blob-mirror\modules\current\__aurora\derived\models\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\<resref>.glb
```

Wersja aktualna:

```yaml
derived_model_version:
  converter_version: "aurora-mdl-to-glb/v13-binary-model-header-supermodel-animationscale"
  derived_segment: "aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale"
```

Minimalny kontrakt dla creature direct model z animacjami supermodelu:

```yaml
gltf_contract_v13_for_meshy2aurora:
  asset:
    version: "2.0"
    generator: "meshy2aurora <version>"
  top_level_extras:
    aurora:
      sourceModel: "<new_resref>"
      converterVersion: "aurora-mdl-to-glb/v13-binary-model-header-supermodel-animationscale"
      supermodel: "c_Horror"
      animationScale: 0.7200000286102295
      sourceHierarchyNodeCount: 38
      sourceHierarchyRootNodeNames:
        - "c_kocrachn"
      sourceHierarchyNodeTypes:
        - "dummy"
        - "skin"
        - "trimesh"
      sourceHierarchy:
        - name: "c_kocrachn"
          nodeType: "dummy"
          position: [0, 0, 0]
          scale: 1
        - name: "rootdummy"
          nodeType: "dummy"
          parentName: "c_kocrachn"
          position: [0, -0.37894999980926514, 0.9040600061416626]
          orientation: [0, 0, 0, 1]
          orientationFormat: "quaternion"
          scale: 1
      textureReferences:
        - "<diffuse_texture_resref>"
      sourceAnimationNames: []      # opcjonalne; c_kocrachn ma 0 lokalnych animacji
      sourceAnimations: []          # opcjonalne; animacje przychodza z supermodelu c_Horror
  scene_0_extras:
    aurora: "*same_as_top_level_extras.aurora"
  node_or_mesh_extras_for_each_render_mesh:
    aurora:
      sourceMesh: "<mesh_or_node_name>"
      sourceNodeType: "skin"        # albo "trimesh" dla segmentow sztywnych
      sourceParentNode: "<aurora_parent_node>"
      skinWeights:                  # wymagane tylko dla sourceNodeType == skin
        format: "aurora-mdl-original-vertex-weights"
        sourceWeightVertexCount: "<vertex_count>"
        gltfVertexSourceIndices: [0]
        influencingBoneNames:
          - "pelvis"
          - "torso"
        maxInfluencesPerVertex: 4
        requiresWeightNormalization: false
        originalVertexWeights:
          - - bone: "pelvis"
              weight: 1
        inverseBindPoses:
          - bone: "pelvis"
            qboneRefInv: [0, 0, 0, 1]
            tboneRefInv: [0, 0, 0]
  primitive_extras:
    aurora:
      sourceMesh: "<mesh_or_node_name>"
      sourceNodeType: "skin"
      skinWeights: "*same_schema_as_node_or_mesh_extras"
      textureReference: "<diffuse_texture_resref>"
```

Fakty z realnego `c_kocrachn.glb`:

```yaml
c_kocrachn_v13_glb:
  file: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_kocrachn.glb"
  bytes: 155892
  sourceModel: "c_kocrachn"
  supermodel: "c_Horror"
  animationScale: 0.7200000286102295
  sourceAnimationCount: 0
  sourceHierarchyNodeCount: 38
  sourceHierarchyRootNodeNames: ["c_kocrachn"]
  sourceHierarchyNodeTypes: ["dummy", "skin", "trimesh"]
  standardGltfSkinsCount: 0
  nodeCount: 24
  meshCount: 24
```

Wniosek dla strategii B: generowany GLB nie musi miec animacji Meshy. Musi miec hierarchie/nazwy nodow Aurory zgodne z referencja oraz `supermodel` i `animationScale`, zeby runtime mogl retargetowac klipy supermodelu.

## Q2: Struktura skinningu w derived GLB
Status: POTWIERDZONE (zrodla: `C:\Projects\aurora-web\backend\src\modules\runtime-settings\adapters\outbound\derived\aurora-mdl-ascii-to-glb.converter.ts`, `C:\Projects\aurora-web\frontend\src\modules\placeables\adapters\three\placeableThreeAssetLoader.ts`, `C:\Projects\aurora-web\.codex-tmp\module-blob-mirror\modules\current\__aurora\derived\models\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\c_kocrachn.glb`)

Konwerter v13 nie zapisuje Aurora skinningu jako standardowe glTF `skins[]`, `JOINTS_0`, `WEIGHTS_0` i `inverseBindMatrices`. W realnym `c_kocrachn.glb` liczba standardowych `skins` wynosi `0`, a prymitywy maja atrybuty typu `POSITION`, `NORMAL`, `TEXCOORD_0`. Wagi sa zapisane w `extras.aurora.skinWeights`.

Runtime creature nie polega tu na `THREE.SkinnedMesh`. Uzywa zwyklych `THREE.Mesh` i wykonuje CPU skinning: przy starcie zapisuje kopie pozycji bind pose z `geometry.position`, potem dla kazdej klatki przelicza pozycje wierzcholkow przez macierze kosci/nodow i metadane `originalVertexWeights`.

Struktura realnych skin nodes w `c_kocrachn.glb`:

```yaml
c_kocrachn_skinning:
  standardGltfSkinsCount: 0
  cpu_skinning_metadata_location:
    - "nodes[].extras.aurora.skinWeights"
    - "meshes[].extras.aurora.skinWeights"
    - "meshes[].primitives[].extras.aurora.skinWeights"
  skin_nodes:
    - name: "Lshinmesh01"
      sourceNodeType: "skin"
      vertexCount: 59
      sourceWeightVertexCount: 59
      gltfVertexSourceIndicesLength: 59
      originalVertexWeightsLength: 59
      maxInfluencesPerVertex: 2
      influencingBoneNames: ["Lcalf2", "Lfoot"]
      inverseBindPoseCount: 27
    - name: "Rshinmesh01"
      sourceNodeType: "skin"
      vertexCount: 59
      sourceWeightVertexCount: 59
      gltfVertexSourceIndicesLength: 59
      originalVertexWeightsLength: 59
      maxInfluencesPerVertex: 2
      influencingBoneNames: ["Rcalf2", "Rfoot"]
      inverseBindPoseCount: 27
    - name: "bodymesh01"
      sourceNodeType: "skin"
      vertexCount: 185
      sourceWeightVertexCount: 185
      gltfVertexSourceIndicesLength: 185
      originalVertexWeightsLength: 185
      maxInfluencesPerVertex: 2
      influencingBoneNames: ["pelvis", "torso"]
      inverseBindPoseCount: 27
```

Konsekwencja dla strategii B:

```yaml
implementation_choice:
  transfer_weights_needed_for_single_meshy_body: true
  reason: "v13 runtime deforms CPU-side by extras.aurora.skinWeights; plain Meshy GLB mesh without Aurora skinWeights will animate only as rigid object or not follow bones."
  accepted_mvp_paths:
    - id: "cpu_skinning_v13"
      description: "jeden lub kilka meshow Meshy z sourceNodeType=skin i originalVertexWeights przeniesionymi z referencji"
      preferred_for: "organiczny creature"
    - id: "rigid_segments"
      description: "pocieta siatka per Aurora node, kazdy segment jako trimesh pod odpowiednim parentem"
      preferred_for: "twarde/czesciowe modele, slabsza jakosc deformacji"
  rejected_as_primary_mvp:
    - id: "standard_gltf_skin_only"
      reason: "nie jest kontraktem derived v13 uzywanym przez obecny CPU skinning Aurory"
```

Wniosek: transfer wag jest potrzebny dla rekomendowanej strategii B, jezeli mesh z Meshy ma zachowywac sie jak organiczny creature. Sama segmentacja per-node jest mozliwa jako uproszczony fallback, ale nie da takiej deformacji jak referencyjny skin.

## Q3: Rejestracja nowego resref
Status: POTWIERDZONE (zrodla: `C:\Projects\aurora-web\backend\src\modules\catalog\adapters\inbound\http\creatures.controller.ts`, `C:\Projects\aurora-web\backend\src\modules\catalog\adapters\outbound\blob\module-snapshot-template-catalog.repository.ts`, `C:\Projects\aurora-web\backend\src\modules\catalog\application\catalog.application.service.ts`, `C:\Projects\aurora-web\backend\src\modules\catalog\application\catalog-processing.helpers.ts`, `C:\Projects\aurora-web\frontend\src\modules\layout\components\WorkspaceLayout.tsx`, `C:\Projects\aurora-web\backend\scripts\capture-creatures-mode-cdp.mjs`)

Nie ma potwierdzonej sciezki, w ktorej samo wrzucenie `zcp_x.glb` do derived mirrora automatycznie dodaje nowy wpis do listy Creatures Mode. Lista backendowa `GET /catalog/creatures` pochodzi z katalogu template creature, a repozytorium klasyfikuje creature po plikach `.utc`. Render payload idzie przez `POST /catalog/creatures/process`.

Samo UI ma lokalne custom creature:

```yaml
frontend_custom_creature_templates:
  storage_key: "aurora_web.creatures_mode_custom_templates.v1"
  object_id_prefix: "cre_wizard_"
  required_source:
    type: "aurora_git"
    sourceKind: "creature"
  limitation: "lokalny wpis dziedziczy templateId z bazy; nie rejestruje samodzielnie nowego modelu GLB w backend asset manifest"
```

Backend moze syntetyzowac creature z metadanych area-instance, ale model direct nadal wynika z profilu appearance:

```yaml
backend_creature_model_resolution:
  list_endpoint: "GET /catalog/creatures"
  process_endpoint: "POST /catalog/creatures/process"
  process_payload_keys:
    - "templateId"
    - "resref"
    - "creatureMetadata"
    - "creatureAppearanceOverride"
  direct_model_candidate_source:
    metadata_key: "appearanceRaceModel"
    source: "appearance.2da.RACE"
    direct_when: "appearanceModelType != P"
  part_model_mode:
    direct_when: "appearanceModelType == P"
    source: "part model prefix + body part ids"
```

Najmniej inwazyjne sciezki:

```yaml
least_invasive_paths:
  loader_smoke_test:
    method: "uzyc istniejacego template creature i CDP --modelOverrideGlbPath"
    command_shape: "node backend/scripts/capture-creatures-mode-cdp.mjs --resrefs c_kocrachn --animationName cpause1 --modelOverrideGlbPath <path-to-generated.glb>"
    caveat: "obecny override przechwytuje kazde zgloszenie .glb, wiec moze podmienic takze supermodel; dobry smoke test loadera, nie docelowy proof animacji bez filtra URL/resref"
  robust_mvp_proof:
    method: "wstawic GLB jako derived model pod resrefem, ktory backend juz wybiera jako direct model, albo dodac minimalny filtr override w CDP runnerze"
    preferred_reference_resref: "c_kocrachn"
    required_followup: "runner powinien podmieniac tylko direct model URL, zostawiajac c_Horror/supermodel chain bez podmiany"
  persistent_catalog_entry:
    method: "dodac source-backed .utc oraz appearance/direct-model dane do source layer albo dodac jawny backend hook dla generated creature metadata"
    current_status: "wymaga implementacji lub przygotowania zasobu source; samo GLB nie wystarcza"
```

Rekomendacja dla MVP: najpierw nie rejestrowac nowego creature jako pelnego katalogowego wpisu. Zrobic `zcp_x.glb` zgodny z v13, uruchomic proof na `c_kocrachn` z selektywna podmiana direct modelu, a dopiero po proofie dodac mala sciezke backendowa dla wygenerowanych creature:

```yaml
proposed_generated_creature_registration:
  generated_manifest:
    path: "C:\\Projects\\meshy2aurora\\samples\\<resref>\\manifest.json"
    fields:
      resref: "zcp_x"
      baseTemplateId: "cre_c_kocrachn"
      directModelResRef: "zcp_x"
      appearanceModelType: "S"
      appearanceRaceModel: "zcp_x"
      supermodel: "c_Horror"
      animationScale: 0.7200000286102295
  aurora_web_minimal_hook:
    accept_process_payload_metadata:
      appearanceModelType: "S"
      appearanceRaceModel: "zcp_x"
    ensure_asset_manifest_sees:
      - "__aurora/derived/models/aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale/zcp_x.glb"
```

## Q4: Bind pose referencji
Status: POTWIERDZONE (zrodla: `C:\Projects\aurora-web\.codex-tmp\module-blob-mirror\modules\current\__aurora\derived\models\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\c_kocrachn.glb`, `C:\Projects\aurora-web\frontend\src\modules\placeables\adapters\three\placeableThreeAssetLoader.ts`)

Tak. Bind/rest pose referencji jest dostepna bezposrednio w GLB jako domyslne pozycje geometrii i domyslne transformy nodow. Nie trzeba odtwarzac animacji. Dla `c_kocrachn.glb`:

```yaml
bind_pose_sources:
  default_geometry:
    source: "accessors POSITION w GLB"
    usage: "pozycje wierzcholkow przed odpaleniem AnimationMixer"
  hierarchy:
    source: "extras.aurora.sourceHierarchy"
    node_count: 38
    root: "c_kocrachn"
    node_types: ["dummy", "skin", "trimesh"]
  cpu_skinning_bind_positions:
    runtime_source: "geometry.attributes.position"
    stored_key: "auroraSourceSkinningBindPositions"
    behavior: "runtime kopiuje pozycje bind pose przed przeliczaniem CPU skinningu"
  inverse_bind_poses:
    source: "extras.aurora.skinWeights.inverseBindPoses"
    present_on_skin_nodes: true
```

Przykladowe nody bind pose z `sourceHierarchy`:

```yaml
c_kocrachn_bind_pose_sample:
  - name: "c_kocrachn"
    type: "dummy"
    parent: null
    position: [0, 0, 0]
    scale: 1
  - name: "rootdummy"
    type: "dummy"
    parent: "c_kocrachn"
    position: [0, -0.37894999980926514, 0.9040600061416626]
    orientation: [0, 0, 0, 1]
    scale: 1
  - name: "pelvis"
    type: "trimesh"
    parent: "rootdummy"
    position: [0, 0.07205890119075775, 0.03405189886689186]
    orientation: [0, 0, 0, 1]
    scale: 1
  - name: "torso"
    type: "trimesh"
    parent: "pelvis"
    position: [0, -0.05049100145697594, -0.039669398218393326]
    orientation: [0, 0, 0, 1]
    scale: 1
  - name: "bodymesh01"
    type: "skin"
    parent: "c_kocrachn"
    position: [0, -0.0032346199732273817, 0.9565079808235168]
    orientation: [0, 0, 0, 1]
    scale: 1
```

Jak wyciagnac bind pose dla obrazu referencyjnego do Meshy:

```yaml
reference_render_steps:
  - load_glb: "C:\\Projects\\aurora-web\\.codex-tmp\\module-blob-mirror\\modules\\current\\__aurora\\derived\\models\\aurora-mdl-to-glb_v13-binary-model-header-supermodel-animationscale\\c_kocrachn.glb"
  - use_loader: "THREE.GLTFLoader"
  - do_not_start: "AnimationMixer"
  - render_default_scene: true
  - camera_frame: "bbox calego modelu"
  - output: "PNG bind pose jako obraz referencyjny do promptu Meshy"
```

Jak wyciagnac baze do transferu wag:

```yaml
weight_transfer_reference_data:
  geometry:
    read: "mesh.primitives[].attributes.POSITION"
    note: "to sa pozycje bind pose"
  node_transforms:
    read: "extras.aurora.sourceHierarchy"
  weights:
    read: "nodes/meshes/primitives extras.aurora.skinWeights.originalVertexWeights"
  vertex_mapping:
    read: "extras.aurora.skinWeights.gltfVertexSourceIndices"
  inverse_bind:
    read: "extras.aurora.skinWeights.inverseBindPoses"
  transfer_target:
    write: "generated GLB extras.aurora.skinWeights.originalVertexWeights aligned to generated vertex order"
    maxInfluencesPerVertex: 4
```

Wniosek: `c_kocrachn.glb` jest wystarczajaca referencja zarowno dla obrazu bind pose, jak i dla transferu wag. Dla MVP trzeba tylko wygenerowac widok referencyjny bez animacji i utrzymac proporcje Meshy mozliwie blisko tego bind pose.
