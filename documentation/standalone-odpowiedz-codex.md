# standalone-odpowiedz-codex.md

Status 2026-07-09: CZESCIOWO SUPERSEDED przez D9. Fragmenty o szukaniu referencyjnego ASCII MDL i emisji ASCII jako M1 sa historyczne. Aktualny output `meshy2aurora` to natywny binary MDL + MDX policy + 2DA + HAK; ASCII moze byc tylko debug dump/snapshot.

Status 2026-07-10: ten plik jest REFERENCJA, nie aktywnym planem wykonawczym. Aktualny M1A emituje deterministyczny JSON inspection report, nie ASCII MDL. Dla pierwszego profilu `direct creature` obowiazuje jeden zasob MDL typu 2002 z appended MDX block; osobny zasob MDX typu 2003 nie wystepuje w potwierdzonym `cep3_core1.hak` i nie jest oczekiwany. Aktywny gate znajduje sie w `audyt-gotowosci-startowej-2026-07-10-codex.md` oraz `engine-mdl-odpowiedz-codex.md`.

Data: 2026-07-08  
Adresat: `standalone-pytania-cloud.md`  
Zakres: standalone M1 po decyzji D7

## Granica projektu

Status: POTWIERDZONE.

`C:\Projects\aurora-web` jest osobnym projektem. Dla `meshy2aurora` wolno go czytać wyłącznie jako materiał porównawczy. Nie wolno używać `aurora-web` jako dependency, CLI, oracle, walidatora, importu, fixture source ani elementu runtime/testów. Implementacja `meshy2aurora` ma być samodzielna: własny parser MDL, własny emiter ASCII, własny writer 2DA i własny writer ERF/HAK.

Źródła dla tej odpowiedzi:

```yaml
local_sources:
  documentation_rules:
    - "C:\\Projects\\meshy2aurora\\documentation\\reguly-dokumentacji-cloud.md"
    - "C:\\Projects\\meshy2aurora\\documentation\\standalone-pytania-cloud.md"
    - "C:\\Projects\\meshy2aurora\\documentation\\decyzje-i-zadania-cloud.md"
  binary_mdl_layout:
    - "C:\\Projects\\Claude\\xoreos-docs\\templates\\NWN1MDL.bt"
    - "C:\\Projects\\Claude\\xoreos-docs\\specs\\torlack\\binmdl.html"
  local_nwn:
    install_root: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights"
    user_root: "C:\\Users\\enonw\\Documents\\Neverwinter Nights"
    cep_core1_hak: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\cep3_core1.hak"
internet_sources_supplementary:
  nwnmdlcomp_vault: "https://neverwintervault.org/project/nwn1/other/tool/nwnmdlcomp-model-compilerdecompiler"
  nwnmdlcomp_source: "https://github.com/niv/nwn-tools/blob/master/nwnmdlcomp/nwnmdlcomp.cpp"
  xoreos_docs: "https://github.com/xoreos/xoreos-docs"
  nwn_lib_d: "https://github.com/CromFr/nwn-lib-d"
```

## Q1: Droga do referencyjnego ASCII MDL bez aurora-web

Status: POTWIERDZONE dla faktów lokalnych.  
Status: NIE WIEM dla gotowej, działającej lokalnie ścieżki binary MDL -> ASCII bez naprawy narzędzia.  
Decyzja implementacyjna: własny parser binary MDL jest potrzebny dla M1; zewnętrzny dekompilator może być później dodatkowym cross-checkiem, nie fundamentem.

### Ustalenia

`c_kocrachn` jest dostępny poza `aurora-web` w CEP HAK:

```yaml
c_kocrachn_reference:
  container: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\cep3_core1.hak"
  container_signature: "HAK "
  container_version: "V1.0"
  entry_count: 6402
  resource:
    resref: "c_kocrachn"
    key_index: 724
    resource_id: 724
    resource_type: 2002
    resource_type_name: "MDL"
    file_offset: 179725952
    file_size: 163192
  mdl_header:
    bin_mdl_id: 0
    binary: true
    p_start_mdx: 76048
    size_mdx: 87132
```

Lokalny NwnMdlComp istnieje:

```yaml
nwnmdlcomp:
  primary_path: "C:\\Projects\\nwn\\VFX\\source-assets\\loose-graphics-and-references\\nwn-vfx-research\\downloads\\tools\\nwn_model_compiler\\NWN Model Compiler\\nwnmdlcomp.exe"
  primary_size_bytes: 1015808
  old_version_path: "C:\\Projects\\nwn\\VFX\\source-assets\\loose-graphics-and-references\\nwn-vfx-research\\downloads\\tools\\nwn_model_compiler\\NWN Model Compiler\\OldVersion\\nwnmdlcomp.exe"
  old_version_size_bytes: 180224
  help_command:
    - "nwnmdlcomp.exe"
  documented_usage:
    decompile: "nwnmdlcomp.exe -d infile [outfile]"
    compile: "nwnmdlcomp.exe -c infile [outfile]"
  observed_decompile_result:
    command: "nwnmdlcomp.exe -d c_kocrachn.mdl c_kocrachn.mdl.ascii"
    output_contains:
      - "Unable to locate or open Neverwinter Night"
    ascii_output_created: false
  current_m1_status: "BLOCKED_AS_ORACLE"
```

Wniosek: NwnMdlComp potwierdza, że istnieje historyczna ścieżka binary MDL -> ASCII, ale na tej maszynie nie jest dziś działającym oracle dla M1. Błąd wygląda jak problem wykrywania instalacji NWN/Steam, nie jak błąd formatu `c_kocrachn`.

`nwn-lib-d`:

```yaml
nwn_lib_d:
  internet_repo: "https://github.com/CromFr/nwn-lib-d"
  listed_tools:
    - "nwn-gff"
    - "nwn-tlk"
    - "nwn-2da"
    - "nwn-erf"
    - "nwn-trn"
    - "nwn-srv"
  mdl_binary_to_ascii_tool_listed: false
  local_built_binary_found: false
  usable_for_mdl_m1: false
local_ruby_nwn_lib_sources:
  paths:
    - "C:\\Projects\\ai-documentations\\nwn-repo-audit\\repo-audit\\dunahan__nwn-lib"
    - "C:\\Projects\\ai-documentations\\nwn-repo-audit\\repo-audit\\niv__nwn-lib"
  local_bins_seen:
    - "bin\\nwn-gff"
    - "bin\\nwn-erf"
    - "bin\\nwn-dsl"
  mdl_binary_to_ascii_tool_seen: false
```

Wniosek: `nwn-lib-d` nie rozwiązuje binary MDL -> ASCII. Lokalnie są kopie źródeł/starego Ruby `nwn-lib`, ale bez gotowego walidatora MDL i bez roli dependency dla `meshy2aurora`.

## Q2: Pełny layout binarnego MDL dla własnego parsera

Status: POTWIERDZONE dla layoutu rdzeniowego z `NWN1MDL.bt` i lokalnego nagłówka `c_kocrachn`.  
Status: HIPOTEZA dla pól oznaczonych `unknown*`, których nie interpretujemy w M1.  
Zakres M1: czytać model, geometrię, hierarchię node, controllery, skin weights i animacje w zakresie potrzebnym do emisji deterministycznego ASCII. Golden snapshot dopiero po zewnętrznym/manualnym potwierdzeniu. Nie pisać jeszcze binary MDL.

Offsety w YAML są relatywne do początku danej struktury, chyba że pole ma `absolute_offset`.

```yaml
binary_mdl:
  endian: "little"
  file_header:
    size: 12
    fields:
      - { name: "bin_mdl_id", offset: 0x00, type: "uint32", note: "0 means binary MDL in observed c_kocrachn" }
      - { name: "p_start_mdx", offset: 0x04, type: "uint32", note: "offset from after file header to MDX block" }
      - { name: "size_mdx", offset: 0x08, type: "uint32", note: "MDX block size" }
  pointer_rules:
    model_data_base: 12
    raw_mdx_base: "12 + file_header.p_start_mdx"
    mdl_pointer: "absolute_file_offset = 12 + pointer_value"
    mdx_pointer: "absolute_file_offset = 12 + p_start_mdx + pointer_value"
    null_mdx_pointer: -1
  primitive_structs:
    vertex:
      size: 12
      fields:
        - { name: "x", offset: 0x00, type: "float32" }
        - { name: "y", offset: 0x04, type: "float32" }
        - { name: "z", offset: 0x08, type: "float32" }
    texcoord:
      size: 8
      fields:
        - { name: "u", offset: 0x00, type: "float32" }
        - { name: "v", offset: 0x04, type: "float32" }
    color:
      size: 12
      fields:
        - { name: "r", offset: 0x00, type: "float32" }
        - { name: "g", offset: 0x04, type: "float32" }
        - { name: "b", offset: 0x08, type: "float32" }
    rgba:
      size: 4
      fields:
        - { name: "r", offset: 0x00, type: "uint8" }
        - { name: "g", offset: 0x01, type: "uint8" }
        - { name: "b", offset: 0x02, type: "uint8" }
        - { name: "a", offset: 0x03, type: "uint8" }
    quaternion:
      size: 16
      fields:
        - { name: "w", offset: 0x00, type: "float32" }
        - { name: "x", offset: 0x04, type: "float32" }
        - { name: "y", offset: 0x08, type: "float32" }
        - { name: "z", offset: 0x0C, type: "float32" }
    array_definition:
      size: 12
      fields:
        - { name: "p_array_start", offset: 0x00, type: "uint32", pointer_kind: "mdl_pointer" }
        - { name: "nr_used_entries", offset: 0x04, type: "uint32" }
        - { name: "nr_alloc_entries", offset: 0x08, type: "uint32" }
  array_of_pointers:
    layout: "array_definition + external uint32 pointer list"
    pointer_list_entry_type: "uint32 mdl_pointer"
```

```yaml
headers:
  geometry:
    size: 0x70
    fields:
      - { name: "p_func1", offset: 0x00, type: "uint32" }
      - { name: "p_func2", offset: 0x04, type: "uint32" }
      - { name: "name", offset: 0x08, type: "char[64]" }
      - { name: "p_node_header", offset: 0x48, type: "uint32", pointer_kind: "mdl_pointer" }
      - { name: "count_nodes", offset: 0x4C, type: "uint32" }
      - { name: "unknown1", offset: 0x50, type: "array_definition" }
      - { name: "unknown2", offset: 0x5C, type: "array_definition" }
      - { name: "ref_count", offset: 0x68, type: "uint32" }
      - { name: "type", offset: 0x6C, type: "uint8" }
      - { name: "padding", offset: 0x6D, type: "uint8[3]" }
  model:
    size: 0xE8
    fields:
      - { name: "geometry", offset: 0x00, type: "header_geometry" }
      - { name: "unknown0", offset: 0x70, type: "uint8" }
      - { name: "unknown1", offset: 0x71, type: "uint8" }
      - { name: "flags", offset: 0x72, type: "uint8" }
      - { name: "fog", offset: 0x73, type: "uint8" }
      - { name: "count_child_model", offset: 0x74, type: "uint32" }
      - { name: "animations", offset: 0x78, type: "array_of_pointers" }
      - { name: "p_supermodel", offset: 0x84, type: "uint32", pointer_kind: "mdl_pointer_or_zero" }
      - { name: "bound_min", offset: 0x88, type: "vertex" }
      - { name: "bound_max", offset: 0x94, type: "vertex" }
      - { name: "model_radius", offset: 0xA0, type: "float32" }
      - { name: "scale", offset: 0xA4, type: "float32" }
      - { name: "supermodel_name", offset: 0xA8, type: "char[64]" }
  node:
    size: 0x70
    fields:
      - { name: "p_func1", offset: 0x00, type: "uint32" }
      - { name: "p_func2", offset: 0x04, type: "uint32" }
      - { name: "p_func3", offset: 0x08, type: "uint32" }
      - { name: "p_func4", offset: 0x0C, type: "uint32" }
      - { name: "p_func5", offset: 0x10, type: "uint32" }
      - { name: "p_func6", offset: 0x14, type: "uint32" }
      - { name: "color_inherit", offset: 0x18, type: "uint32" }
      - { name: "node_number", offset: 0x1C, type: "uint32" }
      - { name: "node_name", offset: 0x20, type: "char[32]" }
      - { name: "p_geometry", offset: 0x40, type: "uint32", pointer_kind: "mdl_pointer" }
      - { name: "p_parent_node", offset: 0x44, type: "uint32", pointer_kind: "mdl_pointer_or_zero" }
      - { name: "children", offset: 0x48, type: "array_definition", entry_type: "uint32 mdl_pointer" }
      - { name: "controller_keys", offset: 0x54, type: "array_definition", entry_type: "controller[used]" }
      - { name: "controller_data", offset: 0x60, type: "array_definition", entry_type: "float32[]" }
      - { name: "content", offset: 0x6C, type: "uint32 bitmask" }
    content_flags:
      has_header: 0x001
      has_light: 0x002
      has_emitter: 0x004
      has_camera: 0x008
      has_reference: 0x010
      has_mesh: 0x020
      has_skin: 0x040
      has_anim: 0x080
      has_dangly: 0x100
      has_aabb: 0x200
  controller:
    size: 12
    fields:
      - { name: "type", offset: 0x00, type: "uint32" }
      - { name: "value_count", offset: 0x04, type: "uint16" }
      - { name: "timekey_start", offset: 0x06, type: "uint16" }
      - { name: "data_start", offset: 0x08, type: "uint16" }
      - { name: "column_count", offset: 0x0A, type: "uint8" }
      - { name: "padding", offset: 0x0B, type: "uint8" }
    common_types:
      position: 8
      orientation: 20
      scale: 36
      mesh_self_illum_color: 100
      mesh_alpha: 128
    value_lookup:
      time_keys: "controller_data[timekey_start ...]"
      data_values: "controller_data[data_start ...]"
      values_per_key: "column_count"
```

```yaml
mesh:
  face:
    size: 0x20
    fields:
      - { name: "normal", offset: 0x00, type: "vertex" }
      - { name: "distance", offset: 0x0C, type: "float32" }
      - { name: "surface_id", offset: 0x10, type: "int32" }
      - { name: "adj_face_ids", offset: 0x14, type: "uint16[3]" }
      - { name: "vertex_id", offset: 0x1A, type: "uint16[3]" }
  header_mesh:
    size: 0x200
    fields:
      - { name: "p_func1", offset: 0x000, type: "uint32" }
      - { name: "p_func2", offset: 0x004, type: "uint32" }
      - { name: "faces", offset: 0x008, type: "array_definition", entry_type: "face" }
      - { name: "bound_min", offset: 0x014, type: "vertex" }
      - { name: "bound_max", offset: 0x020, type: "vertex" }
      - { name: "radius", offset: 0x02C, type: "float32" }
      - { name: "average", offset: 0x030, type: "vertex" }
      - { name: "diffuse", offset: 0x03C, type: "color" }
      - { name: "ambient", offset: 0x048, type: "color" }
      - { name: "specular", offset: 0x054, type: "color" }
      - { name: "shininess", offset: 0x060, type: "float32" }
      - { name: "shadow", offset: 0x064, type: "uint32" }
      - { name: "beaming", offset: 0x068, type: "uint32" }
      - { name: "render", offset: 0x06C, type: "uint32" }
      - { name: "transparency", offset: 0x070, type: "uint32" }
      - { name: "unknown1", offset: 0x074, type: "uint32" }
      - { name: "texture0", offset: 0x078, type: "char[64]" }
      - { name: "texture1", offset: 0x0B8, type: "char[64]" }
      - { name: "texture2", offset: 0x0F8, type: "char[64]" }
      - { name: "texture3", offset: 0x138, type: "char[64]" }
      - { name: "tile_fade", offset: 0x178, type: "uint32" }
      - { name: "vertex_indices", offset: 0x17C, type: "array_definition" }
      - { name: "face_leftover", offset: 0x188, type: "array_definition" }
      - { name: "vertex_indices_count", offset: 0x194, type: "array_definition", entry_type: "uint32" }
      - { name: "vertex_indices_offset", offset: 0x1A0, type: "array_definition", entry_type: "int32 mdx_pointer" }
      - { name: "p_mdx_unknown1", offset: 0x1AC, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "unknown2", offset: 0x1B0, type: "uint32" }
      - { name: "type", offset: 0x1B4, type: "uint32 enum mesh_type" }
      - { name: "p_start_mdx", offset: 0x1B8, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_vertex", offset: 0x1BC, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "count_vertexes", offset: 0x1C0, type: "uint16" }
      - { name: "count_textures", offset: 0x1C2, type: "uint16" }
      - { name: "p_mdx_texture0", offset: 0x1C4, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_texture1", offset: 0x1C8, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_texture2", offset: 0x1CC, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_texture3", offset: 0x1D0, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_vertex_normals", offset: 0x1D4, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_vertex_colors", offset: 0x1D8, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_tex_anim0", offset: 0x1DC, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_tex_anim1", offset: 0x1E0, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_tex_anim2", offset: 0x1E4, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_tex_anim3", offset: 0x1E8, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_tex_anim4", offset: 0x1EC, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_mdx_tex_anim5", offset: 0x1F0, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "light_mapped", offset: 0x1F4, type: "uint8" }
      - { name: "rotate_texture", offset: 0x1F5, type: "uint8" }
      - { name: "padding", offset: 0x1F6, type: "uint16" }
      - { name: "vertex_normal_sum", offset: 0x1F8, type: "float32" }
      - { name: "unknown3", offset: 0x1FC, type: "uint32" }
  mdx_arrays:
    vertices: { pointer: "p_mdx_vertex", count: "count_vertexes", element: "vertex" }
    texture0: { pointer: "p_mdx_texture0", count: "count_vertexes", element: "texcoord" }
    texture1: { pointer: "p_mdx_texture1", count: "count_vertexes", element: "texcoord" }
    texture2: { pointer: "p_mdx_texture2", count: "count_vertexes", element: "texcoord" }
    texture3: { pointer: "p_mdx_texture3", count: "count_vertexes", element: "texcoord" }
    normals: { pointer: "p_mdx_vertex_normals", count: "count_vertexes", element: "vertex" }
    vertex_colors: { pointer: "p_mdx_vertex_colors", count: "count_vertexes", element: "rgba" }
```

```yaml
skin:
  header_skin:
    size: 0x64
    fields:
      - { name: "weights", offset: 0x00, type: "array_definition" }
      - { name: "p_weight_vertex", offset: 0x0C, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_bone_ref_index", offset: 0x10, type: "int32", pointer_kind: "mdx_pointer" }
      - { name: "p_bone_mapping", offset: 0x14, type: "int32", pointer_kind: "mdl_pointer" }
      - { name: "count_bone_mapping", offset: 0x18, type: "int32" }
      - { name: "bone_quats", offset: 0x1C, type: "array_definition", entry_type: "quaternion" }
      - { name: "bone_vertex", offset: 0x28, type: "array_definition", entry_type: "vertex" }
      - { name: "bone_constants", offset: 0x34, type: "array_definition", entry_type: "int16[2]" }
      - { name: "bone_parts", offset: 0x40, type: "int16[17]" }
      - { name: "spare", offset: 0x62, type: "int16" }
  mdx_skin_arrays:
    weights_per_vertex:
      pointer: "p_weight_vertex"
      count: "mesh.count_vertexes"
      element:
        - { name: "weight0", type: "float32" }
        - { name: "weight1", type: "float32" }
        - { name: "weight2", type: "float32" }
        - { name: "weight3", type: "float32" }
    bone_indices_per_vertex:
      pointer: "p_bone_ref_index"
      count: "mesh.count_vertexes"
      element:
        - { name: "index0", type: "int16" }
        - { name: "index1", type: "int16" }
        - { name: "index2", type: "int16" }
        - { name: "index3", type: "int16" }
```

```yaml
animation:
  node_anim_header:
    size: 0x38
    fields:
      - { name: "sample_period", offset: 0x00, type: "float32" }
      - { name: "animation_vertices", offset: 0x04, type: "array_definition" }
      - { name: "animation_texcoords", offset: 0x10, type: "array_definition" }
      - { name: "animation_normals", offset: 0x1C, type: "array_definition" }
      - { name: "p_animation_vertex", offset: 0x28, type: "uint32", pointer_kind: "mdl_pointer" }
      - { name: "p_animation_texcoord", offset: 0x2C, type: "uint32", pointer_kind: "mdl_pointer" }
      - { name: "count_set_vertex", offset: 0x30, type: "uint32" }
      - { name: "count_set_vertex_texcoord", offset: 0x34, type: "uint32" }
  event:
    size: 0x24
    fields:
      - { name: "start", offset: 0x00, type: "float32" }
      - { name: "name", offset: 0x04, type: "char[32]" }
  animation_header:
    size: 0xC4
    fields:
      - { name: "geometry", offset: 0x00, type: "header_geometry" }
      - { name: "length", offset: 0x70, type: "float32" }
      - { name: "transition", offset: 0x74, type: "float32" }
      - { name: "name_root_node", offset: 0x78, type: "char[64]" }
      - { name: "events", offset: 0xB8, type: "array_definition", entry_type: "event" }
  read_order:
    - "read file_header"
    - "read header_model at absolute offset 12"
    - "read root node at header_model.geometry.p_node_header"
    - "walk node children recursively"
    - "for each node, read optional node payloads according to content flags"
    - "read animations from header_model.animations pointer array"
    - "for each animation, read animation root node at animation.header.geometry.p_node_header"
```

Minimalny zakres parsera dla M1:

```yaml
m1_parser_scope:
  required:
    - "validate file_header.bin_mdl_id == 0"
    - "bounds-check every mdl_pointer and mdx_pointer"
    - "read model header and supermodel_name"
    - "read node tree with node_name, parent, children, content flags"
    - "read controller keys/data for position/orientation/scale and mesh alpha"
    - "read mesh faces, vertices, normals, texture0"
    - "read skin weights and bone indices where has_skin is set"
    - "read animation headers, events and animation node controllers"
    - "emit deterministic versioned JSON inspection report; optional human-readable debug snapshot only"
  defer:
    - "binary MDL writer"
    - "full emitter/light/dangly/aabb semantics beyond safe parse/skip"
    - "semantic meaning of unknown* fields"
```

## Q3: Runbook weryfikacji w NWN EE bez aurora-web

Status: POTWIERDZONE dla lokalnych ścieżek.  
Status: HIPOTEZA dla automatyzacji `nwmain`/`nwtoolset`, bo nie potwierdziłem headless CLI do wizualnego proofu.  
Wniosek: proof M1 powinien być manualno-wizualny w Toolset/grze plus automatyczne testy plikowe we własnym kodzie. Testowy moduł, HAK i minimalne zasoby proofu mają być tworzone przez `meshy2aurora`; gotowe moduły/HAKi/asset-packi mogą być tylko zewnętrzną referencją read-only, nie bazą testu.

```yaml
nwn_ee_environment:
  install_root: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights"
  user_root: "C:\\Users\\enonw\\Documents\\Neverwinter Nights"
  executables:
    nwmain: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwmain.exe"
    nwtoolset: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwtoolset.exe"
    nwserver: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwserver.exe"
    nwhak: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwhak.exe"
    nwnexplorer: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwnexplorer.exe"
  content_dirs:
    hak: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak"
    modules: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\modules"
    override: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\override"
```

Minimalny proof HAK:

```yaml
m1_hak_install:
  generated_hak: "C:\\Projects\\meshy2aurora\\dist\\m2a_m1.hak"
  install_to: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m1.hak"
  source_policy: "generated_by_meshy2aurora_only"
  forbidden_as_test_base:
    - "prebuilt CEP HAK copied as test artifact"
    - "prebuilt retail module"
    - "prebuilt community module"
    - "aurora-web output"
  expected_resources:
    - { resref: "m2a_koc01", type: "MDL", resource_type: 2002, payload: "binary MDL plus appended MDX block" }
    - { resref: "appearance", type: "2DA", resource_type: 2017 }
    - { resref: "m2a_koc01", type: "TGA_or_DDS", resource_type: "3_or_2033" }
```

Manualny runbook Toolset:

```yaml
toolset_runbook:
  - step: "Skopiuj HAK"
    action: "Copy C:\\Projects\\meshy2aurora\\dist\\m2a_m1.hak -> C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m1.hak"
    note: "m2a_m1.hak musi być wygenerowany przez meshy2aurora, nie skopiowany z gotowego content packa"
  - step: "Uruchom Toolset"
    action: "\"C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwtoolset.exe\""
  - step: "Utwórz moduł testowy"
    action: "File -> New Module; area minimalna/domyślna"
  - step: "Podłącz HAK"
    action: "Module Properties -> Custom Content -> Hak Paks -> add m2a_m1"
  - step: "Utwórz creature blueprint"
    action: "Custom palette -> New Creature; resref/tag m2a_koc01; Appearance ustawione na nowy row z appearance.2da"
  - step: "Połóż creature w area"
    action: "Placeable/Creature palette -> m2a_koc01 -> place in area"
  - step: "Zapisz moduł"
    action: "Save as C:\\Users\\enonw\\Documents\\Neverwinter Nights\\modules\\m2a_m1_test.mod"
```

Proof artefacts:

```yaml
proof_gates:
  file_gate:
    status: "automatable"
    checks:
      - "HAK signature == HAK "
      - "HAK version == V1.0"
      - "resource table contains MDL/MDX/2DA/texture entries"
      - "appearance.2da has row for m2a_koc01"
  toolset_gate:
    status: "manual_visual"
    evidence:
      - "screenshot: Toolset module properties with m2a_m1.hak attached"
      - "screenshot: creature blueprint m2a_koc01 with selected appearance"
      - "screenshot: area viewport with creature visible, no missing-model marker"
  game_gate:
    status: "manual_visual"
    evidence:
      - "screenshot: nwmain running m2a_m1_test.mod with creature visible"
      - "short video or screenshot series: idle -> walk/run -> attack -> death if animations exist"
  fail_conditions:
    - "Toolset crash"
    - "game crash"
    - "model invisible"
    - "missing texture checkerboard or blank material"
    - "T-pose/no animation where reference has animation"
    - "wrong scale/orientation causing unusable creature"
```

Automatyzacja:

```yaml
automation_status:
  confirmed:
    - "file-level HAK/2DA/MDL parser tests can be automated in meshy2aurora"
  not_confirmed:
    - "nwtoolset.exe headless module creation"
    - "nwmain.exe headless screenshot capture for local module"
    - "nwhak.exe reliable CLI validation; observed -h opened GUI instead of printing useful help"
  possible_but_not_visual:
    - "nwserver.exe may be useful for module-load smoke testing, but it cannot replace visual model/animation proof"
```

## Q4: Niezależny walidator plików

Status: POTWIERDZONE dla dostępności lokalnych narzędzi i ich ścieżek.  
Status: HIPOTEZA/NIE WIEM dla komend CLI tam, gdzie narzędzie zachowuje się jak GUI albo nie jest zbudowane lokalnie.

```yaml
validators:
  nwnmdlcomp:
    status: "POTWIERDZONE_LOKALNIE_BUT_BLOCKED"
    path: "C:\\Projects\\nwn\\VFX\\source-assets\\loose-graphics-and-references\\nwn-vfx-research\\downloads\\tools\\nwn_model_compiler\\NWN Model Compiler\\nwnmdlcomp.exe"
    commands:
      decompile: "\"...\\nwnmdlcomp.exe\" -d input.mdl output.mdl.ascii"
      compile: "\"...\\nwnmdlcomp.exe\" -c input.mdl output.mdl"
    validates:
      - "binary MDL can be decoded to ASCII"
      - "ASCII MDL can be compiled to binary"
    current_blocker: "Unable to locate or open Neverwinter Night"
    m1_role: "optional future cross-check after path/Steam detection is fixed"
  nwhak:
    status: "POTWIERDZONE_GUI"
    path: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwhak.exe"
    cli_command: "NIE WIEM"
    observed_cli_behavior: "\"nwhak.exe -h\" opened/stayed as GUI, no confirmed help output"
    validates:
      - "manual open/list/extract HAK resources"
      - "manual check that generated HAK is readable by official NWN tool"
    m1_role: "manual independent HAK inspection"
  nwnexplorer:
    status: "POTWIERDZONE_GUI"
    path: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwnexplorer.exe"
    cli_command: "NIE WIEM"
    validates:
      - "manual browse of NWN/HAK resources"
      - "manual export/inspect references if GUI supports the target resource"
    m1_role: "manual independent resource browser"
  gffeditor:
    status: "POTWIERDZONE_GUI"
    path: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\util\\win32\\GFFEditor.exe"
    cli_command: "NIE WIEM"
    validates:
      - "manual UTC/GFF inspection if M1 emits a test creature blueprint"
    m1_role: "optional manual validator for generated UTC"
  nwn_lib_d:
    status: "NIE_WIEM_LOCAL_BUILD; POTWIERDZONE_NO_MDL_TOOL_IN_LIST"
    repo: "https://github.com/CromFr/nwn-lib-d"
    local_built_commands_found: false
    listed_tools:
      - "nwn-gff"
      - "nwn-tlk"
      - "nwn-2da"
      - "nwn-erf"
      - "nwn-trn"
      - "nwn-srv"
    validates_if_built:
      - "2DA"
      - "ERF/HAK"
      - "GFF"
    validates_mdl: false
    exact_commands: "NIE WIEM until built and --help verified"
    m1_role: "not a dependency; possible optional independent 2DA/ERF check later"
  local_ruby_nwn_lib:
    status: "POTWIERDZONE_SOURCE_ONLY"
    paths:
      - "C:\\Projects\\ai-documentations\\nwn-repo-audit\\repo-audit\\dunahan__nwn-lib"
      - "C:\\Projects\\ai-documentations\\nwn-repo-audit\\repo-audit\\niv__nwn-lib"
    bins:
      - "bin\\nwn-gff"
      - "bin\\nwn-erf"
      - "bin\\nwn-dsl"
    validates_mdl: false
    m1_role: "reference only; do not import or vendor"
  xoreos_010_template:
    status: "POTWIERDZONE_SPEC_REFERENCE"
    path: "C:\\Projects\\Claude\\xoreos-docs\\templates\\NWN1MDL.bt"
    validates:
      - "independent structure layout for binary MDL"
    executable_validator: false
    m1_role: "parser layout reference"
  nwn_ee_toolset_game:
    status: "POTWIERDZONE_FINAL_RUNTIME_VALIDATOR"
    paths:
      nwtoolset: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwtoolset.exe"
      nwmain: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwmain.exe"
    validates:
      - "real engine/toolset loads HAK"
      - "appearance.2da row resolves"
      - "model renders"
      - "animation behaves in engine"
    m1_role: "required proof gate"
  aurora_web:
    status: "EXCLUDED_BY_D7"
    path: "C:\\Projects\\aurora-web"
    allowed_use: "read-only comparison of implementation ideas"
    forbidden_use:
      - "dependency"
      - "subprocess CLI"
      - "import"
      - "oracle"
      - "validator"
      - "fixture source"
```

## Q5: Licencja/pochodzenie referencji w fixtures

Status: NIE WIEM dla ostatecznej odpowiedzi prawnej.  
Status: POTWIERDZONE dla bezpiecznej polityki inżynieryjnej: nie commitować kopii retail/CEP assetów do repo.

Rekomendacja: testy proof/integration `meshy2aurora` mają generować własny moduł, własny HAK i własne minimalne zasoby testowe. Lokalne assety retail/CEP można wskazywać przez konfigurację ścieżek wyłącznie jako read-only materiał referencyjny do badań parsera albo porównań struktury, nie jako gotową bazę testowego modułu/HAK. Repo może przechowywać własne syntetyczne fixtures, metadane, manifesty, hashe i snapshoty struktury, ale nie payloady retail/CEP ani zdekompilowane pochodne tych payloadów.

```yaml
fixture_policy:
  default: "GENERATE_OWN_TEST_MODULE_AND_HAK"
  ready_made_content_policy:
    use_prebuilt_modules_as_test_base: false
    use_prebuilt_haks_as_test_base: false
    use_aurora_web_outputs_as_test_base: false
    allow_external_assets_as_readonly_reference: true
  commit_allowed:
    - "synthetic mini MDL/2DA/HAK created by meshy2aurora tests"
    - "small generated fixtures with no BioWare/Beamdog/CEP payload"
    - "generated test module metadata created by meshy2aurora"
    - "generated test HAK created by meshy2aurora"
    - "YAML/JSON manifests of expected parsed fields"
    - "hashes, sizes, resource ids, offsets, resrefs"
    - "test code that reads user-provided local paths as read-only reference"
  commit_forbidden_without_explicit_license_clearance:
    - "extracted c_kocrachn.mdl"
    - "extracted c_kocrachn.mdx"
    - "decompiled c_kocrachn ASCII MDL"
    - "copied CEP HAKs"
    - "extracted c_horror.mdl"
    - "extracted c_horror.mdx"
    - "decompiled c_horror ASCII MDL"
    - "retail textures/sounds/models copied from NWN install"
  local_references:
    cep_c_kocrachn:
      container: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\cep3_core1.hak"
      resref: "c_kocrachn"
      resource_type: 2002
    retail_root:
      install_root: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights"
      data_dir: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\data"
      key_files_seen:
        - "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\data\\nwn_base.key"
        - "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\data\\nwn_retail.key"
      c_horror_exact_container: "NIE WIEM until KEY/BIF lookup or nwnexplorer confirms"
  env_config:
    M2A_NWN_ROOT: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights"
    M2A_NWN_USER_ROOT: "C:\\Users\\enonw\\Documents\\Neverwinter Nights"
    M2A_CEP_CORE1_HAK: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\cep3_core1.hak"
    M2A_REF_C_KOCRACHN_RESREF: "c_kocrachn"
    M2A_REF_C_HORROR_RESREF: "c_horror"
  test_behavior:
    unit_tests:
      use: "synthetic fixtures only"
      require_game_assets: false
    integration_tests:
      use: "generated module + generated HAK + generated minimal assets"
      missing_paths: "skip with explicit message"
      never_write_extracted_assets_to_repo: true
    reference_readonly_tests:
      use: "local external NWN/CEP paths only for parser/reference inspection"
      never_package_external_payloads: true
      never_use_as_test_module_or_hak_base: true
```

Minimalny pattern testów:

```yaml
tdd_fixture_plan:
  parser_unit:
    source: "synthetic binary MDL fixture generated in test"
    assertion: "parser decodes header/pointers/nodes deterministically"
  hak_unit:
    source: "synthetic HAK V1.0 fixture generated in test"
    assertion: "writer output can be read back by our reader and has correct key/resource tables"
  generated_proof_integration:
    source: "meshy2aurora-generated module + meshy2aurora-generated HAK"
    assertion: "Toolset/game can load our generated proof content"
    skip_when_missing: false
  reference_readonly_cep:
    source: "M2A_CEP_CORE1_HAK + M2A_REF_C_KOCRACHN_RESREF"
    assertion: "parser locates c_kocrachn MDL and reads binary header/tree without packaging it into our proof HAK"
    skip_when_missing: true
  reference_readonly_retail:
    source: "M2A_NWN_ROOT + M2A_REF_C_HORROR_RESREF"
    assertion: "parser locates retail reference after KEY/BIF lookup is implemented, without using it as proof content"
    skip_when_missing: true
```

## Blokery po tej rundzie

Status: POTWIERDZONE.

```yaml
blockers:
  - id: "B1"
    topic: "NwnMdlComp"
    status: "blocked"
    detail: "local executable exists, but decompile fails with 'Unable to locate or open Neverwinter Night'"
    impact: "cannot use as M1 oracle today"
  - id: "B2"
    topic: "c_horror exact retail location"
    status: "open"
    detail: "NWN EE data/key files exist, but exact BIF/resource lookup for c_horror not confirmed in this answer"
    impact: "retail integration fixture should wait for KEY/BIF reader or manual nwnexplorer confirmation"
  - id: "B3"
    topic: "headless visual proof"
    status: "open"
    detail: "no confirmed nwmain/nwtoolset CLI path for screenshot proof"
    impact: "M1 proof requires manual Toolset/game screenshots or separate automation discovery"
```
