# aurora-mdl-format-codex.md
Data: 2026-07-08  
Status: POTWIERDZONE czesciowo; NIE WIEM tam, gdzie brak lokalnej kotwicy runtime
Status 2026-07-09: AKTYWNA REFERENCJA FORMATU, z korekta D9. Sekcje ASCII sa przydatne do zrozumienia semantyki i debug dumpow, ale docelowy output `meshy2aurora` to native binary MDL/MDX policy + 2DA + HAK.

## Zakres

Dokument opisuje kontrakt MDL potrzebny do emisji pierwszego `direct creature` dla `meshy2aurora`. Zasada zrodel: najpierw dekompilacja `C:\Projects\New Folder\export\decompiled_all.c`, potem istniejacy kod `aurora-web`, dopiero potem Internet jako uzupelnienie.

## Zrodla

```yaml
primary_sources:
  decompiled_aurora:
    path: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
    anchors:
      top_level_keywords: "881733-881758, 881821-881828"
      node_keywords: "615410-615420, 886230-886340"
      skin_keywords: "615387-615394, 885933-885957"
      animation_keywords: "615434-615442, 886456-886494"
      appearance_gff: "194114, 194535, 195146, 195211"
  aurora_web_converter:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
    anchors:
      ascii_binary_route: "544-555"
      renderable_ascii_nodes: "62"
      parsed_model_contract: "72-115"
      ascii_animation_parser: "1668-1760"
      ascii_anim_node_keys: "1770-1818"
      ascii_skin_weights: "2280-2284, 2422-2448"
      binary_skin_limit: "1410-1445"
      glb_aurora_extras: "3130-3241"
internet_supplement:
  - "https://nwn.fandom.com/wiki/.mdl"
  - "https://github.com/xoreos/xoreos-docs/blob/master/templates/NWN1MDL.bt"
```

## Top-level ASCII MDL

Status: POTWIERDZONE dla keywordow parsera; HIPOTEZA dla wymogu konkretnej wartosci `classification`.

Dekompilacja potwierdza parser top-level dla: `setsupermodel`, `setanimationscale`, `classification`, `newmodel`, `newanim`, `beginmodelgeom`, `donemodel`, `ignorefog`. Rejestracja handlerow jest widoczna dla `newanim`, `newmodel`, `classification`, `setsupermodel`, `setanimationscale`.

```yaml
minimal_ascii_mdl_top_level:
  required_for_model_identity:
    - line: "newmodel <model_resref>"
      status: POTWIERDZONE
      decompiled_all_c: "881751-881758, 881822"
    - line: "beginmodelgeom <model_resref>"
      status: POTWIERDZONE
      decompiled_all_c: "881733-881758"
    - line: "donemodel <model_resref>"
      status: POTWIERDZONE
      decompiled_all_c: "top-level parser keyword set"
  recommended_for_creature:
    - line: "setsupermodel <model_resref> NULL"
      status: POTWIERDZONE keyword; HIPOTEZA wartosc NULL jako brak supermodelu
      decompiled_all_c: "614802-614805, 881733-881740, 881827"
    - line: "setanimationscale <model_resref> 1.0"
      status: POTWIERDZONE keyword
      decompiled_all_c: "614804-614805, 881739-881740, 881828"
    - line: "classification CHARACTER"
      status: HIPOTEZA wartosci; POTWIERDZONE keyword
      decompiled_all_c: "614806-614807, 881745-881746, 881825"
  optional:
    - "ignorefog"
```

`aurora-web` zachowuje w GLB metadata `sourceModel`, `supermodel`, `animationScale`, `sourceAnimations`, `sourceHierarchy` i referencje tekstur. `classification` nie jest obecnie istotnym polem runtime w potwierdzonym kodzie `aurora-web`.

## Typy nodow

Status: POTWIERDZONE dla nazw widocznych w dekompilacji i/lub obslugiwanych przez konwerter.

```yaml
ascii_node_types:
  dummy:
    status: POTWIERDZONE w praktyce przez hierarchie z binary/sourceHierarchy; nazwa stringu w dekompilacji nie zostala odzyskana jako czytelny s_dummy
    role: "kosc/root/transform node"
    emit_for_creature: true
  trimesh:
    status: POTWIERDZONE
    decompiled_all_c: "615413, 886230-886340"
    aurora_web_renderable: true
  skin:
    status: POTWIERDZONE w aurora-web; dekompilacja potwierdza pola skin/weights, sam string typu jest w nieczytelnych DAT_00cc8247/DAT_00cc824c
    aurora_web_renderable: true
    emit_for_creature: true
  animmesh:
    status: POTWIERDZONE
    decompiled_all_c: "615414"
    aurora_web_renderable: true
  danglymesh:
    status: POTWIERDZONE
    decompiled_all_c: "615415"
    aurora_web_renderable: true
  emitter:
    status: POTWIERDZONE
    decompiled_all_c: "615411"
    emit_for_creature_mvp: false
  light:
    status: POTWIERDZONE
    decompiled_all_c: "615412"
    emit_for_creature_mvp: false
  camera:
    status: POTWIERDZONE
    decompiled_all_c: "615410"
    emit_for_creature_mvp: false
  reference:
    status: POTWIERDZONE
    decompiled_all_c: "615416"
    emit_for_creature_mvp: false
  aabb:
    status: NIE WIEM lokalnie; typ znany z ekosystemu MDL, ale nie zamkniety kotwica do zdekompilowanego stringu w tej rundzie
    emit_for_creature_mvp: false
```

## Geometria i material

Status: POTWIERDZONE.

Dekompilacja i parser `aurora-web` potwierdzaja podstawowy zestaw:

```yaml
mesh_fields:
  transform:
    - "parent <parent_node_name>"
    - "position <x> <y> <z>"
    - "orientation <axis_x> <axis_y> <axis_z> <angle_rad>"
    - "scale <number>"
  geometry:
    - "verts <count>"
    - "tverts <count>"
    - "tverts1 <count>"
    - "tverts2 <count>"
    - "tverts3 <count>"
    - "faces <count>"
  material_texture:
    - "bitmap <resref>"
    - "texture0 <resref>"
    - "diffuse <r> <g> <b>"
    - "ambient <r> <g> <b>"
    - "specular <r> <g> <b>"
    - "shininess <number>"
    - "transparencyhint <integer>"
```

Konwerter `aurora-web` mapuje tekstury do slotow `diffuse`, `normal`, `specular`, `roughness`, `height`, `emissive` i zachowuje `materialName`, `renderHint`, `selfIllumColor`.

## Skin i bind pose

Status: POTWIERDZONE dla parsera `weights`; POTWIERDZONE dla binary limitu 4 wplywow; HIPOTEZA dla limitu engine'u ASCII, wiec dla emisji przyjmujemy konserwatywnie max 4.

Dekompilacja potwierdza pola `weights`, `qbone_ref_inv`, `tbone_ref_inv`, `boneconstantindices`. `aurora-web` czyta `weights` w ASCII jako pary `bone weight` per vertex, bez twardego limitu po stronie ASCII parsera. Binary path czyta zawsze 4 wplywy na vertex.

```yaml
ascii_skin_contract:
  node_type: "skin"
  parent: "<model_root_or_mesh_parent>"
  required:
    verts: "<same count as weights rows>"
    faces: "<triangles>"
    weights:
      syntax: "weights <vertex_count>, then exactly <vertex_count> rows"
      row_syntax: "<bone_1> <weight_1> <bone_2> <weight_2> ..."
      aurora_web_parser: "readSkinWeightRows, lines 2422-2448"
      recommended_max_influences_per_vertex: 4
      weight_sum: 1.0
  bind_pose:
    required_nodes: "all bone names referenced by weights must exist in source hierarchy"
    inverse_bind_pose_fields:
      - "qbone_ref_inv"
      - "tbone_ref_inv"
      - "boneconstantindices"
```

Przy emisji z Meshy:

```yaml
emission_rule:
  humanoid:
    source: "Meshy rig + animations"
    output: "skin nodes with normalized weights, <=4 influences per vertex"
  monster_or_quadruped:
    source: "Meshy mesh only + Aurora reference skeleton"
    output: "skin nodes weighted/retargeted against Aurora skeleton reference"
```

## Animacje w MDL

Status: POTWIERDZONE dla skladni bloku; NIE WIEM dla pelnej listy semantycznych eventow wymaganych przez retail runtime.

Dekompilacja potwierdza `newanim`, `event`, `length`, `transtime`, `animroot`. `aurora-web` czyta `positionkey`, `orientationkey`, `scalekey`, `alphakey`, `selfillumcolorkey` i zachowuje metadata animacji.

```yaml
ascii_animation_block:
  start: "newanim <animation_name> <target_model>"
  fields:
    - "length <seconds>"
    - "transtime <seconds>"
    - "animroot <root_node_name>"
    - "event <time_seconds> <event_name>"
  node_block:
    start: "node <node_type> <node_name>"
    keys:
      - "positionkey <count>"
      - "orientationkey <count>"
      - "scalekey <count>"
      - "alphakey <count>"
      - "selfillumcolorkey <count>"
    end: "endnode"
  end: "doneanim <animation_name>"
```

Dla `direct creature` z wlasnym szkieletem animacje musza targetowac te same nazwy nodow/kosci, ktore sa w geometrii. Dla pierwszego MVP bez wlasnego supermodelu najbezpieczniej emitowac minimum `cpause1`, `cwalk`, `crun`, `ca1slashl`, `cdamagel`, `cdead` i dopiero rozszerzac.

## ASCII vs binary

Status: POTWIERDZONE w `aurora-web`.

```yaml
format_detection:
  binary:
    predicate: "bytes.length >= 12 and first uint32 little-endian == 0"
    aurora_web: "aurora-mdl-ascii-to-glb.converter.ts:544-555"
  ascii:
    predicate: "fallback if not binary"
    encoding: "utf-8 text"
binary_mdl:
  header:
    raw_data_offset: "uint32 at byte 4"
    raw_data_size: "uint32 at byte 8"
  skin_weights:
    influences_per_vertex: 4
ascii_mdl:
  skin_weights:
    parser_limit: "no hard max in aurora-web ASCII parser"
    emitter_rule: "emit <=4 influences for engine/binary parity"
```

## Kontrakt emisji MVP

Status: HIPOTEZA wdrozeniowa oparta na potwierdzonych parserach.

```yaml
meshy2aurora_mvp_direct_creature_mdl:
  resref:
    max_chars_for_hak_erf_v10: 16
    recommended_prefix: "m2a_"
  top_level:
    - "newmodel <resref>"
    - "setsupermodel <resref> NULL"
    - "setanimationscale <resref> 1.0"
    - "classification CHARACTER"
    - "beginmodelgeom <resref>"
    - "node dummy <resref>"
    - "node dummy <bone/root nodes...>"
    - "node skin <mesh nodes...>"
    - "donemodel <resref>"
  animations:
    required_minimum:
      - "cpause1"
      - "cwalk"
      - "crun"
      - "ca1slashl"
      - "cdamagel"
      - "cdead"
    every_animation:
      - "length"
      - "transtime"
      - "animroot <root_node_name>"
      - "node blocks for animated bones"
  validation_tests:
    - "parse generated binary MDL/MDX with meshy2aurora readback parser"
    - "assert canonical/readback hierarchy contains all weighted bones"
    - "assert each skin weights row count equals verts count"
    - "assert max influences per vertex <= 4"
    - "assert exported animation manifest/readback includes required clips"
```
