# neverblender-audyt-2026-07-09-codex.md

Data: 2026-07-09  
Autor: Codex  
Status: AKTYWNY AUDYT REFERENCYJNY  
Zakres: NeverBlender jako narzedzie pomocnicze dla `meshy2aurora`

## 0. Decyzja wykonawcza

NeverBlender jest wartosciowym narzedziem do:

- recznej inspekcji modeli NWN/Aurora w Blenderze,
- importu/eksportu ASCII MDL,
- zrozumienia praktycznych konwencji MDL: `trimesh`, `danglymesh`, `skin`, `aabb`, `emitter`, `animmesh`, `dummy`, `light`, `walkmesh`,
- pracy artystycznej i diagnostycznej wokol armature, pseudo-bones, animacji, UV, smoothing groups, materialow i MTR.

Nie powinien byc:

- zaleznoscia runtime/build `meshy2aurora`,
- glownym exporterem finalnych plikow gry,
- oracle dla poprawnosci binary MDL/MDX/HAK,
- proofem gry,
- zrodlem kodu do skopiowania do projektu.

Powod: NeverBlender operuje na ASCII MDL, wymaga zgodnej wersji Blendera, a binary MDL obsluguje tylko posrednio przez zewnetrzny dekompilator. Aktywny cel `meshy2aurora` pozostaje:

```text
Meshy GLB/FBX
  -> nasz parser/normalizator/walidator
  -> nasz writer binary MDL + polityka MDX + 2DA + HAK
  -> NWN EE Toolset/gra proof
```

ASCII MDL moze istniec tylko jako debug dump, golden snapshot albo format do recznej inspekcji w Blenderze.

## 1. Zrodla audytu

```yaml
sources:
  internet:
    neverblender_40_vault:
      url: "https://neverwintervault.org/project/nwnee/other/tool/neverblender-40"
      status: POTWIERDZONE
      used_for:
        - "wersja 4.1.0"
        - "kompatybilnosc Blender 4.0 / 3.6"
        - "brak kompatybilnosci Blender 4.1"
        - "instalacja z zip bez rozpakowywania"
    neverblender_28_vault:
      url: "https://neverwintervault.org/project/nwn1/other/tool/neverblender-28"
      status: POTWIERDZONE
      used_for:
        - "wersja 2.8.051"
        - "kompatybilnosc Blender 2.83, 2.93, 3.0, 3.1, 3.2, 3.3"
        - "problem Blender 3.4 z importem materialow"
        - "minimap tool"
    nwn_wiki_neverblender:
      url: "https://nwn.wiki/spaces/NWN1/pages/38176439/NeverBlender"
      status: POTWIERDZONE
      used_for:
        - "NeverBlender laduje ASCII MDL"
        - "binary MDL wymaga zewnetrznego dekompilatora"
        - "CleanModels EE jako rekomendowany dekompilator EE"
        - "mesh validation moze usuwac dwustronne faces"
    beamdog_forum_neverblender:
      url: "https://forums.beamdog.com/discussion/72134/neverblender"
      status: POTWIERDZONE
      used_for:
        - "lista funkcji: geometry, walkmeshes, uv, textures, smoothing groups, animations, normals, tangents, vertex colors, MTR, emitter, armature/pseudo-bones, NLA"
    beamdog_forum_page_2:
      url: "https://forums.beamdog.com/discussion/72134/neverblender/p2"
      status: POTWIERDZONE
      used_for:
        - "praktyczny problem binary MDL"
        - "wzmianka o game client compiler"
    cleanmodels_ee_vault:
      url: "https://neverwintervault.org/project/nwnee/other/tool/clean-modelsee"
      status: POTWIERDZONE
      used_for:
        - "CleanModels EE dekompiluje latest NWN:EE"
        - "renderhint, materialname, normals, tangents"
        - "do 64 bones"
        - "command line i UI"
        - "ryzyka: rescale/static, crash/weird output, RENDER 1->0 w komentarzu"
    old_mdltools_github:
      url: "https://github.com/gyoerkaa/mdltools"
      status: POTWIERDZONE
      used_for:
        - "historyczny projekt NeverBlender/mdltools"
  local_machine:
    blender:
      path: "C:\\Program Files\\Blender Foundation\\Blender 5.1\\blender.exe"
      version: "Blender 5.1.2"
      status: POTWIERDZONE
      compatibility_with_neverblender: NIE_WIEM
    downloaded_audit_packages:
      temp_root: "C:\\Users\\enonw\\AppData\\Local\\Temp\\meshy2aurora-neverblender-audit"
      neverblender_40_zip: "C:\\Users\\enonw\\AppData\\Local\\Temp\\meshy2aurora-neverblender-audit\\neverblender_40-010.zip"
      neverblender_28_zip: "C:\\Users\\enonw\\AppData\\Local\\Temp\\meshy2aurora-neverblender-audit\\neverblender_28-051.zip"
      status: POTWIERDZONE
      repo_policy: "nie kopiowac paczek do repo; temp tylko do audytu"
```

## 2. Macierz wersji

```yaml
version_matrix:
  neverblender_4_0_package:
    vault_version: "4.1.0"
    package_file: "neverblender_40-010.zip"
    blender_supported:
      confirmed_by_vault:
        - "4.0.x"
      additionally_noted_by_author:
        - "3.6"
    blender_not_supported:
      - "4.1"
      - "4.1.1"
    local_blender_5_1_2:
      status: NIE_WIEM
      recommendation: "nie zakladac kompatybilnosci; do testu uzyc portable Blender 4.0.2"
  neverblender_2_8_package:
    vault_version: "2.8.051"
    package_file: "neverblender_28-051.zip"
    blender_supported:
      - "2.83"
      - "2.93"
      - "3.0"
      - "3.1"
      - "3.2"
      - "3.3"
    blender_not_supported:
      - "3.4 material import"
    recommendation: "dobry fallback dla starszych workflow i porownan"
  old_versions:
    blender_2_79:
      status: HISTORYCZNE
      use: "tylko gdy stary asset lub instrukcja wymaga 2.79"
    blender_2_69:
      status: HISTORYCZNE
      use: "nie rekomendowane dla nowego pipeline"
    blender_2_5x:
      status: HISTORYCZNE
      use: "nie rekomendowane dla nowego pipeline"
```

Wniosek: na tej maszynie mamy Blender 5.1.2, a glowne zrodla NeverBlender nie potwierdzaja tej wersji. Dla realnego testu trzeba przygotowac osobny Blender 4.0.2 portable albo Blender 3.3/2.93 z pasujacym NeverBlender.

## 3. Co NeverBlender realnie obsluguje

### 3.1 Geometria i typy node'ow

Status: POTWIERDZONE w forum, wiki i kodzie paczki.

```yaml
mdl_node_types_observed:
  mesh_types:
    - trimesh
    - danglymesh
    - skin
    - aabb
    - emitter
    - animmesh
  empty_types:
    - dummy
    - reference
    - patch
    - pwk
    - dwk
  other_node_types:
    - light
  walkmesh_types:
    - wok
    - pwk
    - dwk
  classification_values:
    - unknown
    - tile
    - character
    - door
    - effect
    - gui
    - item
    - other
```

Dla `meshy2aurora` najwazniejsze sa:

- `skin` dla creature i animowanych potworow,
- `trimesh` dla prostych placeable/item/debug,
- `dummy`/pseudo-bones dla skeleton-like hierarchii MDL,
- `aabb` i walkmesh raczej pozniej, przy tile/placeable/door,
- `emitter` i `light` jako przyszly zakres, nie MVP creature.

### 3.2 Import/export

Status: POTWIERDZONE w kodzie `nvb_ops_io.py` i dokumentacji.

NeverBlender:

- importuje Aurora MDL,
- eksportuje Aurora MDL,
- eksportuje ASCII MDL, nie natywny binary MDL,
- zapisuje takze zalezne walkmesh files w scenariuszach door/tile/placeable,
- potrafi eksportowac MTR zalezne od materialow,
- ma batch import/export dla wielu modeli,
- moze importowac supermodel jako osobna operacje.

Konsekwencja:

```yaml
neverblender_output_policy_for_meshy2aurora:
  may_use:
    - "debug ASCII MDL"
    - "manual inspection in Blender"
    - "artist round-trip when final input returns as GLB/FBX"
  must_not_use:
    - "final binary MDL"
    - "final HAK writer"
    - "runtime proof"
    - "primary validator"
```

### 3.3 ASCII MDL top-level struktura

Status: POTWIERDZONE w kodzie `nvb_mdl.py`.

NeverBlender parsuje i generuje top-level elementy:

```yaml
ascii_mdl_top_level:
  model:
    - newmodel
    - setsupermodel
    - classification
    - setanimationscale
    - beginmodelgeom
    - endmodelgeom
    - donemodel
  animation:
    - newanim
    - doneanim
  parser_rule:
    animations_before_geometry: "blad MalformedMdlFile"
```

Dla nas to jest bardzo dobra lista kontrolna dla debug ASCII dump. Nie jest to jednak spec binary MDL.

### 3.4 Materialy, tekstury, MTR

Status: POTWIERDZONE w dokumentacji i kodzie `nvb_node.py` / `nvb_mtr.py`.

NeverBlender obsluguje:

- `bitmap`,
- `texture0..texture14` w MTR,
- `materialname`,
- `renderhint`,
- diffuse/specular/ambient/selfillum kolory,
- MTR parameters typu `int` i `float`,
- import/export MTR,
- opcje lower-case material/texture names,
- ignorowanie wybranych kolorow przy imporcie.

Dla `meshy2aurora` wazne:

- Meshy PBR musi zostac uproszczony do formatu zgodnego z NWN/Aurora; NeverBlender nie rozwiazuje automatycznie bake PBR -> diffuse TGA.
- MTR jest mozliwy w NWN:EE, ale MVP creature powinien miec prosty, przewidywalny material diffuse.
- `renderhint` i `materialname` sa cechami EE, ktore trzeba walidowac w naszym writerze, jezeli je emitujemy.

### 3.5 UV, tverts, smoothing groups, normals, tangents

Status: POTWIERDZONE w forum/wiki/kodzie.

NeverBlender:

- obsluguje wiele UV maps,
- mapuje je do warstw `tverts`,
- potrafi generowac `faces` z indeksami vertexow i tverts,
- importuje/eksportuje smoothing groups,
- konwertuje smoothing groups do/z ostrych krawedzi Blendera,
- ma tryby eksportu smoothing:
  - `GROUP`,
  - `SPLIT`,
  - `NONE`,
- ma opcje binary smoothing groups,
- ma opcje distinct verts dla smoothing groups,
- potrafi eksportowac normals i tangents.

Ryzyko dla naszego parsera:

```yaml
geometry_risks:
  duplicate_two_sided_faces:
    status: POTWIERDZONE
    note: "Blender mesh validation moze usuwac dwustronne faces o tych samych vertexach"
  uv_degenerate_triangles:
    status: POTWIERDZONE
    note: "NeverBlender ma logike naprawy zero-area UV triangles"
  triangulation:
    status: POTWIERDZONE
    note: "export opiera sie na triangulacji mesh; my tez musimy miec deterministyczny triangulator"
  smoothing_group_policy:
    status: WYMAGA_DECYZJI
    recommendation: "dla MVP generowac jawne normals i uproszczona polityke smoothing, a smoothing groups traktowac jako walidowana ceche debug"
```

### 3.6 Skinmesh, wagi i bone influences

Status: POTWIERDZONE w kodzie `nvb_node.py`.

NeverBlender dla `skin`:

- laduje `weights`,
- generuje `weights <vertex_count>`,
- sortuje wagi wedlug wielkosci,
- odrzuca wagi mniejsze niz `0.001`,
- zostawia maksymalnie 4 influences na vertex,
- normalizuje wagi,
- moze stripowac koncowki nazw typu `.001` zalezne od opcji.

Dla `meshy2aurora` to jest bardzo wazna kotwica walidacji:

```yaml
skin_weight_policy_candidate:
  max_influences_per_vertex: 4
  discard_below: 0.001
  normalize_after_prune: true
  strip_trailing_numbers:
    default_for_meshy2aurora: false
    reason: "bone mapping musi byc deterministyczny; automatyczne stripowanie moze zepsuc mapowanie"
```

Status `max_influences_per_vertex: 4`: POTWIERDZONE dla NeverBlender; dla runtime Aurora/NWN trzeba potwierdzic w dekompilacji/engine proof.

### 3.7 Animacje

Status: POTWIERDZONE w forum/wiki/kodzie `nvb_anim.py`, `nvb_animnode.py`, `nvb_ops_anim.py`, `nvb_ops_amt.py`.

NeverBlender obsluguje:

- import/export animacji MDL,
- keyframes dla:
  - `position`,
  - `orientation`,
  - `scale`,
  - `color`,
  - `radius`,
  - `alpha`,
  - `selfillumcolor`,
- `sampleperiod`,
- `animverts`,
- `animtverts`,
- animowane vertices jako shape keys,
- animowane texture coords,
- eventy animacji,
- clone/scale/crop/pad/focus/new/delete/move animacji,
- NLA Editor workflow,
- akcje i NLA tracks/strips,
- transfer pseudo-bones <-> Blender armature.

Operatorzy znalezieni w paczce:

```yaml
operators:
  armature:
    - id: "nvb.amt_apply_pose"
      label: "Apply Current Pose"
    - id: "nvb.amt_amt2psb"
      label: "Generate Pseudo Bones"
    - id: "nvb.amt_psb2amt"
      label: "Generate Armature"
  animation:
    - id: "nvb.anim_clone"
      label: "Clone animation"
    - id: "nvb.anim_scale"
      label: "Scale animation"
    - id: "nvb.anim_crop"
      label: "Crop animation"
    - id: "nvb.anim_pad"
      label: "Pad animation"
    - id: "nvb.anim_focus"
      label: "Set start and end frame of the timeline to the animation"
    - id: "nvb.anim_new"
      label: "Create new animation"
    - id: "nvb.anim_delete"
      label: "Delete an animation"
    - id: "nvb.anim_moveback"
      label: "Move an animation to the end"
    - id: "nvb.anim_move"
      label: "Move an animation in the list, without affecting keyframes"
    - id: "nvb.anim_event_new"
      label: "Add a new event to an animation"
    - id: "nvb.anim_event_delete"
      label: "Deletes an event from an animation"
    - id: "nvb.anim_event_move"
      label: "Move an item in the event list"
    - id: "nvb.amt_event_new"
      label: "Create new animation event"
    - id: "nvb.amt_event_delete"
      label: "Delete an animation event"
  setup_helpers:
    - id: "nvb.util_genwok"
      label: "Load walkmesh materials"
    - id: "nvb.util_nodes_pwk"
      label: "Setup Placeable"
    - id: "nvb.util_nodes_dwk"
      label: "Setup Door"
    - id: "nvb.util_nodes_tile"
      label: "Setup Tile"
    - id: "nvb.util_tileslicer"
      label: "Tileslicer"
  io:
    - id: "scene.nvb_mdlexport"
      label: "Export Aurora MDL"
    - id: "scene.nvb_mdlimport"
      label: "Import Aurora MDL"
    - id: "scene.nvb_superimport"
      label: "Import Supermodel"
    - id: "scene.nvb_setexport"
      label: "Export Setfile"
```

Dla `meshy2aurora` najwazniejsza nauka: animacje w Aurorze to nie tylko "clip na szkielecie". Model MDL moze miec:

- hierarchy node animation,
- eventy,
- sampled geometry animation (`animverts`),
- sampled UV animation (`animtverts`),
- supermodel chain.

MVP creature powinien zaczac od transferu siatki/wag do zgodnej hierarchii/supermodelu, a nie od generowania nowych animacji z Meshy.

### 3.8 Armature vs pseudo-bones

Status: POTWIERDZONE w forum/wiki/kodzie `nvb_ops_amt.py`.

NeverBlender ma narzedzia:

- generowania Blender armature z MDL pseudo-bones,
- generowania MDL pseudo-bones z Blender armature,
- przenoszenia animacji miedzy tymi reprezentacjami,
- tworzenia NLA tracks/strips z animacji MDL.

Dla `meshy2aurora` to jest przydatne glownie jako:

- manualna diagnostyka,
- wizualne sprawdzenie hierarchii,
- pomoc artystyczna przy recznym retargetowaniu,
- referencja nazewnictwa i transformacji.

Nie powinno to zastapic naszego retargetera i walidatora, bo:

- potrzebujemy deterministycznego pipeline batch,
- proof ma byc w NWN EE,
- eksport finalny ma byc binary MDL/MDX/2DA/HAK z naszego writera.

## 4. CleanModels EE w relacji do NeverBlender

NeverBlender sam z siebie laduje ASCII MDL. Dla binary MDL potrzebuje zewnetrznego dekompilatora.

CleanModels EE:

```yaml
cleanmodels_ee:
  role:
    - "zewnetrzny decompiler dla binary MDL"
    - "pomoc przy imporcie binary MDL do NeverBlender"
  confirmed_features:
    - "decompile latest NWN:EE models"
    - "renderhint"
    - "materialname"
    - "normals"
    - "tangents"
    - "up to 64 bones"
    - "Windows/Linux/Mac command line and UI builds"
  risks:
    - "rescale/static behavior can surprise placeables"
    - "some options may crash or produce odd output according to comments"
    - "reported critical bug: some decompiled meshes get RENDER changed from 1 to 0"
```

Polityka dla projektu:

- CleanModels EE moze byc opcjonalnym narzedziem referencyjnym/debug.
- Wynik CleanModels EE nie jest oracle.
- Jesli uzyjemy CleanModels EE w runbooku, raport musi zapisac wersje, komende, pliki wejscia/wyjscia i ostrzezenia.
- Nie wolno budowac testu, ktory przechodzi tylko dlatego, ze CleanModels cos "wyczyscil".

## 5. Lokalna sytuacja na tej maszynie

```yaml
local_state_2026_07_09:
  project_docs:
    canonical_path: "C:\\Projects\\meshy2aurora\\documentation"
    status: POTWIERDZONE
  wrong_shell_path:
    path: "C:\\Users\\enonw\\Documents\\meshy2aurora"
    status: FORBIDDEN_NON_CANONICAL_SELF_CREATED_PATH
    note: "nie uzywac ani nie odtwarzac jako repo, workspace, staging, scratch, backup, worktree, migration source ani documentation target"
  blender:
    path: "C:\\Program Files\\Blender Foundation\\Blender 5.1\\blender.exe"
    version: "Blender 5.1.2"
    status: POTWIERDZONE
  neverblender_installed_in_blender:
    status: NIE_WIEM
    reason: "nie instalowano dodatku w Blenderze; audyt dotyczy paczek i kodu"
  neverblender_compat_with_blender_5_1:
    status: NIE_WIEM
    recommendation: "nie uzywac lokalnego Blender 5.1.2 jako zalozenia implementacyjnego"
```

## 6. Ryzyka i pulapki

### R1. Blender 5.1.2 nie jest potwierdzony

Status: POTWIERDZONE dla faktu lokalnej wersji, NIE WIEM dla kompatybilnosci.

Vault potwierdza NeverBlender 4.1.0 dla Blender 4.0.x i notuje dzialanie na 3.6, ale wyraznie mowi, ze Blender 4.1 nie dziala. Lokalny Blender to 5.1.2, wiec nie zakladamy dzialania.

Mitigacja:

```yaml
mitigation:
  preferred:
    install_portable_blender: "4.0.2"
    addon: "NeverBlender 4.1.0 / neverblender_40-010.zip"
  fallback:
    install_portable_blender: "3.3.x albo 2.93.x"
    addon: "NeverBlender 2.8.051"
```

### R2. ASCII MDL moze przypadkiem wrocic jako output

Status: RYZYKO PROJEKTOWE.

NeverBlender jest wygodny, wiec latwo o odjazd: wygenerowac ASCII MDL i potraktowac go jako final. To lamie decyzje D9.

Mitigacja:

- w kodzie `meshy2aurora` nazwac taki output `debug-ascii`, nigdy `export-mdl`,
- finalny writer ma emitowac binary MDL/MDX policy,
- testy powinny failowac, jesli finalny HAK pakuje debug ASCII jako model runtime.

### R3. Binary MDL przez decompiler nie jest neutralnym dowodem

Status: POTWIERDZONE.

NeverBlender dla binary MDL potrzebuje CleanModels EE albo innego dekompilatora. Dekompilator moze zmienic semantyke: znane ryzyka to `RENDER`, `rescale`, brak/niepelna obsluga EE w starych narzedziach.

Mitigacja:

- traktowac decompiled ASCII jako "inspection artifact",
- zapisac hash binary input i hash ASCII output,
- porownywac semantyke przez nasz readback i NWN EE proof, nie tylko przez Blender.

### R4. Mesh validation moze usuwac dwustronne faces

Status: POTWIERDZONE w nwn.wiki.

Blender mesh validation moze usunac przypadki, ktore dla NWN sa intencjonalne.

Mitigacja:

- nie wlaczac automatycznie walidacji Blendera jako jedynej naprawy,
- w naszym walidatorze rozroznic:
  - duplicate accidental,
  - intentional two-sided face,
  - transparent/material two-sided policy.

### R5. Nazwy kosci i strip trailing numbers

Status: POTWIERDZONE jako opcja NeverBlender; RYZYKO dla naszej mapy.

Blender lub NeverBlender moga miec obiekty `.001`, a opcja stripowania moze zmieniac nazwy.

Mitigacja:

- jawna mapa nazw kosci,
- zakaz niejawnego rename w core pipeline,
- warning, gdy nazwa po normalizacji koliduje z inna.

### R6. Wagi sa zaokraglane/przycinane

Status: POTWIERDZONE dla NeverBlender.

NeverBlender przy eksporcie skin weights sortuje, obcina male wagi, ogranicza do 4 influences i normalizuje.

Mitigacja:

- w naszym pipeline zrobic jawny etap `weights.pruneNormalize`,
- emitowac raport ile wag obcieto,
- failowac przy vertexach bez sumy wag po prune,
- porownywac bounding box po skinning sample.

### R7. Animacje sampled mesh/UV moga zostac pominiete

Status: POTWIERDZONE jako cecha NeverBlender.

Aurora MDL moze miec nie tylko animacje transformacji node'ow, ale tez `animverts` i `animtverts`.

Mitigacja:

- parser MDL musi rozpoznawac te sekcje nawet jesli MVP ich nie emituje,
- walidator powinien zwracac `UNSUPPORTED_ANIMVERTS` albo `UNSUPPORTED_ANIMTVERTS`, zamiast cicho ignorowac.

### R8. GPL/licencja dodatku

Status: POTWIERDZONE w paczce przez plik `COPYING` i naglowki.

Paczka ma licencje GPL. Dla `meshy2aurora` oznacza to:

- nie kopiowac kodu NeverBlender do repo,
- mozna czytac zachowanie i dokumentowac obserwacje,
- jezeli chcemy uruchamiac Blender+NeverBlender jako zewnetrzne narzedzie, trzeba to opisac jako opcjonalny adapter, nie zintegrowana biblioteke.

## 7. Rekomendowany workflow z NeverBlender

### W1. Reference inspection

Cel: czlowiek chce obejrzec retail/CEP model albo porownac hierarchie.

```text
binary MDL reference
  -> CleanModels EE / NWN Explorer ASCII export
  -> ASCII MDL
  -> NeverBlender import
  -> screenshoty/notatki
  -> decyzja w documentation
```

Status: DOZWOLONE.

Warunki:

- zrodlo retail/CEP tylko read-only,
- wynik nie jest fixture source dla testow automatycznych,
- dokumentowac sciezke i narzedzie.

### W2. Debug output z meshy2aurora

Cel: sprawdzic, czy nasza reprezentacja ma sens wizualnie.

```text
Meshy input
  -> meshy2aurora canonical model
  -> debug ASCII MDL
  -> NeverBlender import
  -> screenshot/manual inspection
```

Status: DOZWOLONE.

Warunki:

- plik nazwany `*.debug-ascii.mdl` albo lezy w `debug/`,
- nie pakowac tego jako finalny HAK,
- raport musi mowic, ze to nie jest proof gry.

### W3. Reczna edycja w Blenderze

Cel: artysta poprawia mesh/UV/rig.

```text
Meshy GLB/FBX
  -> Blender
  -> manual edit
  -> GLB/FBX export
  -> meshy2aurora import again
  -> validator
  -> our binary writer
```

Status: REKOMENDOWANE dla recznej edycji.

Warunki:

- Blender jest edytorem inputu, nie finalnym exporterem gry,
- po edycji wracamy do naszego pipeline przez GLB/FBX.

### W4. Armature/pseudo-bone exploration

Cel: zrozumiec lub skorygowac hierarchie skeleton/pseudo-bones.

```text
ASCII MDL reference
  -> NeverBlender
  -> Generate Armature / Generate Pseudo Bones
  -> porownanie nazw, osi, rest pose, animacji
  -> reczna decyzja mapowania
```

Status: DOZWOLONE jako badanie.

Warunki:

- mapowanie kosci zapisac w dokumentacji/YAML,
- implementacja mapowania w `meshy2aurora` ma byc wlasna.

## 8. Czego brakuje w `meshy2aurora`

```yaml
gaps:
  G1_neverblender_runtime_test:
    status: NIE_ZROBIONE
    need:
      - "portable Blender 4.0.2 albo 3.3/2.93"
      - "instalacja pasujacego NeverBlender"
      - "minimalny import jednego debug ASCII MDL"
    output:
      - "raport wersji"
      - "screenshot importu"
      - "komenda/uruchomienie headless jesli mozliwe"
  G2_debug_ascii_contract:
    status: BRAK_IMPLEMENTACJI
    need:
      - "nazwa i katalog debug ASCII"
      - "zakaz uzycia jako final runtime output"
      - "test guard"
  G3_external_tools_report:
    status: BRAK_IMPLEMENTACJI
    need:
      - "raport wersji Blender"
      - "raport wersji NeverBlender"
      - "raport wersji CleanModels"
      - "hash wejsc/wyjsc"
  G4_weight_policy:
    status: DO_DECYZJI
    candidate:
      max_influences_per_vertex: 4
      discard_below: 0.001
      normalize_after_prune: true
    required_proof: "dekompilacja/runtime NWN EE"
  G5_animverts_animtverts_policy:
    status: DO_DECYZJI
    recommendation: "parser rozpoznaje, MVP nie emituje; walidator nie ignoruje"
  G6_blender_roundtrip_policy:
    status: DO_ZAPISANIA_W_ARCHITEKTURZE
    recommendation: "Blender jako edytor GLB/FBX inputu, NeverBlender jako debug ASCII viewer"
```

## 9. Proponowane zmiany w planie implementacji

### 9.1 Dodac modul `external-tools`

Proponowany kontrakt:

```yaml
external_tools:
  blender:
    env: "M2A_BLENDER_EXE"
    optional: true
    expected_for_neverblender:
      - "Blender 4.0.x + NeverBlender 4.1.0"
      - "Blender 3.3/2.93 + NeverBlender 2.8.051"
  neverblender:
    env: "M2A_NEVERBLENDER_ZIP"
    optional: true
    allowed_use:
      - "manual/debug import"
      - "diagnostic report"
    forbidden_use:
      - "final export"
  cleanmodels:
    env: "M2A_CLEANMODELS_CLI"
    optional: true
    allowed_use:
      - "reference decompile for inspection"
    forbidden_use:
      - "oracle"
      - "automatic semantic fix without explicit report"
```

### 9.2 Dodac `debug-ascii` output

Proponowany kontrakt:

```yaml
debug_ascii:
  filename_pattern: "*.debug-ascii.mdl"
  output_dir: "artifacts/debug-ascii"
  allowed_consumers:
    - "human"
    - "NeverBlender"
    - "diff snapshot"
  forbidden_consumers:
    - "final HAK packer"
    - "runtime proof"
  required_header_comment:
    - "debug artifact"
    - "not final runtime output"
```

### 9.3 Dodac walidatory inspirowane NeverBlender

```yaml
validators:
  mesh:
    - code: "MESH_TOO_MANY_TRIANGLES"
      source: "project limit, not NeverBlender"
    - code: "MESH_HAS_NGONS"
      action: "triangulate deterministically"
    - code: "MESH_DUPLICATE_TWO_SIDED_FACES"
      action: "warn; do not auto-delete unless policy says so"
    - code: "UV_ZERO_AREA_TRIANGLE"
      action: "warn or repair with explicit report"
    - code: "UV_LAYER_COUNT_UNSUPPORTED"
      action: "fail or reduce by policy"
  skin:
    - code: "SKIN_WEIGHT_INFLUENCES_GT_4"
      action: "prune/normalize with report"
    - code: "SKIN_WEIGHT_SUM_ZERO"
      action: "fail"
    - code: "BONE_NAME_NORMALIZATION_COLLISION"
      action: "fail"
  animation:
    - code: "UNSUPPORTED_ANIMVERTS"
      action: "fail unless feature enabled"
    - code: "UNSUPPORTED_ANIMTVERTS"
      action: "fail unless feature enabled"
    - code: "ANIMATION_EVENT_UNKNOWN"
      action: "warn/fail by profile"
  materials:
    - code: "PBR_NOT_BAKED_TO_DIFFUSE"
      action: "fail before writer"
    - code: "MTR_EE_FEATURE_UNDECIDED"
      action: "warn/fail by profile"
```

### 9.4 Dodac runbook zgodnosci NeverBlender

Minimalny runbook:

```yaml
runbook_neverblender_compat:
  prerequisites:
    - "portable Blender 4.0.2"
    - "neverblender_40-010.zip"
    - "sample debug ASCII MDL generated by meshy2aurora"
  steps:
    - "install addon from zip"
    - "enable addon"
    - "import sample debug ASCII MDL"
    - "capture screenshot"
    - "record Blender version"
    - "record NeverBlender version from addon bl_info"
  pass_criteria:
    - "model appears in viewport"
    - "basic hierarchy visible"
    - "no import crash"
  fail_is_not_blocker_for_core:
    reason: "NeverBlender jest helperem, nie core dependency"
```

## 10. Wplyw na viewport `meshy2aurora`

NeverBlender nie powinien byc viewportem `meshy2aurora`. Nasz viewport ma pokazywac model po transformacjach, ktore wykona nasz pipeline:

- osie i skala po normalizacji,
- liczba trojkatow/wierzcholkow po decymacji,
- tekstura po bake do diffuse,
- UV po docelowej polityce,
- wagi po prune/normalize,
- skeleton/bind pose po retarget,
- animacje z supermodelu albo z naszej listy,
- bledy walidacji przed writerem.

NeverBlender moze byc dodatkowym "otworz w Blenderze" dla debug ASCII, ale nie zastepuje docelowego viewportu ani proofu gry.

## 11. Wplyw na animacje

Pytanie "czy nalozymy od razu automatycznie animacje?" po tym audycie ma odpowiedz:

```yaml
animation_strategy:
  mvp:
    approach: "retarget mesh/weights do referencyjnej hierarchii/supermodelu NWN"
    new_custom_animations: "nie jako pierwszy krok"
    proof: "NWN EE Toolset/gra"
  neverblender_role:
    - "manualne obejrzenie animacji"
    - "porownanie pseudo-bones/armature"
    - "diagnostyka eventow i NLA"
  not_neverblender_role:
    - "automatyczny silnik retargetu produkcyjnego"
    - "finalny eksport runtime"
```

Dodawanie nowych animacji pozniej powinno miec osobny kontrakt:

- nazwa animacji zgodna z oczekiwaniami Aurora/NWN,
- zakres klatek,
- eventy,
- root/supermodel policy,
- czy animacja jest local w modelu, czy przez supermodel,
- czy zawiera `animverts`/`animtverts`,
- czy writer binary obsluguje dany typ kluczy.

## 12. Pytania otwarte

```yaml
open_questions:
  Q1:
    question: "Czy NeverBlender 4.1.0 w ogole uruchamia sie w lokalnym Blender 5.1.2?"
    status: NIE_WIEM
    priority: LOW_FOR_CORE_HIGH_FOR_MANUAL_WORKFLOW
    suggested_test: "portable copy of addon in Blender 5.1.2; no repo dependency"
  Q2:
    question: "Czy chcemy utrzymywac portable Blender 4.0.2 jako oficjalne narzedzie pomocnicze projektu?"
    status: DO_DECYZJI
    priority: MEDIUM
  Q3:
    question: "Czy debug ASCII ma byc generowany od M1, czy dopiero po binary writer readback?"
    status: DO_DECYZJI
    priority: MEDIUM
    recommendation: "dodac od M1 jako snapshot, ale z guardem przed final HAK"
  Q4:
    question: "Czy walidator wag ma przyjac polityke NeverBlender 4 influences / 0.001 jako tymczasowy default?"
    status: DO_POTWIERDZENIA_AURORA_FIRST
    priority: HIGH
  Q5:
    question: "Czy MVP ma jawnie failowac na animverts/animtverts?"
    status: DO_DECYZJI
    priority: MEDIUM
    recommendation: "tak, rozpoznac i failowac z czytelnym kodem"
```

## 13. Statusy faktow

```yaml
confirmed:
  - fact: "NeverBlender 4.1.0 jest opisany na Vault jako wersja dla Blender 4.0.x; nie dziala z Blender 4.1."
    evidence: "Vault NeverBlender 4.0"
  - fact: "NeverBlender 2.8.051 jest opisany dla Blender 2.83/2.93/3.0-3.3; 3.4 ma problem z material import."
    evidence: "Vault NeverBlender 2.8"
  - fact: "NeverBlender laduje ASCII MDL; binary MDL wymaga zewnetrznego dekompilatora."
    evidence: "nwn.wiki NeverBlender"
  - fact: "NeverBlender eksportuje Aurora MDL jako ASCII text."
    evidence: "nvb_ops_io.py / nvb_mdl.py"
  - fact: "NeverBlender obsluguje geometry, walkmeshes, uv, textures, smoothing groups, animations, normals, tangents, vertex colors, MTR, emitter, armature/pseudo-bones, NLA."
    evidence: "Beamdog forum + source package"
  - fact: "CleanModels EE dekompiluje latest NWN:EE, renderhint, materialname, normals, tangents, up to 64 bones."
    evidence: "Vault CleanModels EE"
  - fact: "Lokalny Blender to 5.1.2."
    evidence: "C:\\Program Files\\Blender Foundation\\Blender 5.1\\blender.exe --version"
hypotheses:
  - fact: "NeverBlender moze byc uzyteczny jako debug viewer dla outputu meshy2aurora."
    reason: "pasuje do ASCII MDL inspection, ale nie testowano jeszcze naszego pliku"
  - fact: "Polityka 4 influences / 0.001 jest dobrym defaultem dla MVP."
    reason: "NeverBlender tak robi, ale runtime Aurora/NWN wymaga potwierdzenia"
unknown:
  - fact: "Kompatybilnosc NeverBlender z Blender 5.1.2."
  - fact: "Czy headless Blender+NeverBlender bedzie stabilny dla automatycznego screenshot/debug report."
  - fact: "Czy wszystkie typy animacji, ktore spotkamy w creature reference, da sie poprawnie obejrzec w obecnej wersji NeverBlender."
```

## 14. Konkluzja

NeverBlender powinien zostac w projekcie jako narzedzie pomocnicze klasy "artist/debug/reference". Jest bardzo przydatny, bo pokazuje praktyczna interpretacje ASCII MDL, skin weights, smoothing groups, UV, animacji, armature i pseudo-bones. Ale dokladnie dlatego trzeba postawic mu granice: nie jest finalnym exporterem, nie jest proofem, nie jest core dependency.

Najzdrowszy uklad:

```text
core meshy2aurora:
  Meshy input -> validation -> our binary MDL/MDX/2DA/HAK -> NWN EE proof

optional debug:
  our canonical model -> debug ASCII MDL -> NeverBlender -> human inspection

optional art:
  Meshy GLB/FBX -> Blender manual edit -> GLB/FBX -> meshy2aurora again
```

To trzyma projekt przy pierwotnym celu i jednoczesnie daje praktyczny most do Blendera tam, gdzie czlowiek naprawia albo diagnozuje model.
