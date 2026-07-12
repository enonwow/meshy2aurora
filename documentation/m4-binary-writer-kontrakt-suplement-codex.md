# M4 binary MDL/MDX writer - suplement kontraktu

Data: 2026-07-12
Status: `LOCKED_RIGID_SLICE`; `SKIN_CONTRACT_OPEN`

## 1. Cel i pierwszenstwo

Ten suplement zamyka kontrakt pierwszego implementowanego wycinka M4 i
rozstrzyga sprzecznosci pomiedzy starszym
`mdl-binary-crosswalk-codex.md` a aktywnym planem etapow. Obowiazuje zasada
Aurora First i kolejnosc dowodow z `PROJECT_RULES.md`.

Klasyfikacja dowodow:

- fakt z dekompilacji: typ zasobu `0x7d2` (`2002`) jest przekazywany do resource
  managera w `C:\Projects\New Folder\export\decompiled_all.c:169799-169805`;
- fakt z dekompilacji: dispatch rodzin node i maski flag sa widoczne w
  `decompiled_all.c:872664-872758` oraz `:872824-872927`;
- fakt z eksportu RTTI/strings, nie z nazwanego kodu: `CResMDL`,
  `CResHelper<CResMDL,2002>` i rodziny `MdlNode*` wystepuja w
  `C:\Projects\New Folder\export\strings.tsv:594-598` i `:5617-5648`;
- fakt z lokalnego binary/P-REF: file/core/raw pointer bases, rozmiary common
  headers, `extended64` boundary `node + 0x330` oraz pusty lane
  `weight=0/ref=0xffff` sa potwierdzone przez M1B;
- wniosek implementacyjny do sprawdzenia own readbackiem: deterministyczna
  kolejnosc blokow, wartosci pol niepotwierdzonych i projekcja IR do MDL;
- runtime/visual acceptance pozostaje `OPEN_M6`.

Dekompilacja nie zawiera nazwanego writera ani kompletnego serialized layout
oracle. Nie kopiujemy kodu z zewnetrznych writerow ani payloadow referencyjnych.

## 2. Granice etapow

```yaml
M4:
  owns:
    - "structural model header and zero animation array"
    - "bind-pose node hierarchy and position/orientation controllers"
    - "dummy and trimesh nodes"
    - "deterministic core/raw planning and appended MDX"
    - "own-reader semantic projection and exact EOF"
  emits_own_animation_clips: false
M4A:
  owns:
    - "animation headers, animation roots, tracks, transition and events"
    - "self-contained loader/movement/gameplay animation inventory"
M6:
  owns:
    - "HAK/module/appearance integration"
    - "NWN EE Toolset/game runtime and visual acceptance"
```

M4 moze osiagnac `DONE_STRUCTURAL` bez M6. Globalny `GB-001` pozostaje otwarty
do runtime proof. Starsze wymagania animacji i Toolset/game w sekcji 5/7
`mdl-binary-crosswalk-codex.md` nie sa DoD M4; odpowiednio naleza do M4A i M6.

## 3. Publiczne wejscie M4

Writer przyjmuje wylacznie zwalidowany `AuroraCreatureIrV1` oraz jawne
`MdlWriterOptionsV1`. Nie wraca do GLB ani `AuroraAssetIr`.

```yaml
MdlWriterOptionsV1:
  schemaVersion: 1
  formatProfile: M4_DIRECT_CREATURE_EXTENDED64_V1
  modelResourceResref: generated_resref_1_to_16
  diffuseTextureResrefByMaterialSlot:
    - { materialSlot: u32, resref: generated_resref_1_to_16 }
```

Generated resref ma charset `[a-z0-9_]`, dlugosc `1..16` i jest jednoczesnie
nazwa modelu w model header. `modelResourceResref` i output texture resref nie
sa wyprowadzane z source label. M5
dostarczy finalne zasoby tekstur; syntetyczne testy M4 podaja jawne resrefy.
Brak bindingu dla uzytego slotu jest fatalny. Legacy17 nie jest wybierany
automatycznie; request innego profilu daje stabilne `M4-UNSUPPORTED-PROFILE`.
Material slots w options musza byc unikalne; duplicate i unused binding sa
fatalnym `M4-MATERIAL-BINDING-INVALID`, aby artifact identity nie zalezala od
ignorowanych danych.

## 4. Jawne ograniczenia pierwszego writera

- rig ma dokladnie jeden root;
- IR node id jest arbitralny; rig nodes dostaja dense `partNumber`
  `0..nodes.len()` w kolejnosci IR, a syntetyczne mesh nodes nastepne numery
  `nodes.len()..nodes.len()+segments.len()` w kolejnosci segmentow;
- artefakt zachowuje obie mapy: `IR node id -> partNumber` oraz
  `segmentId -> mesh partNumber`; ID w kazdej domenie musza byc unikalne;
- node name musi byc niepustym ASCII C-stringiem do 31 bajtow; bez truncation;
- bind-local matrix musi byc rigid affine: translacja plus proper orthonormal
  rotation, bez shear, reflection i non-uniform scale;
- kazdy segment ma najwyzej `u16::MAX` vertices, wszystkie indices mieszcza sie
  w `u16` i tworza trojkaty;
- segment mesh name jest deterministyczny: `m2a_seg_<segmentId>`;
- `tangents: Some(...)` jest zachowane w raporcie jako jawna dewiacja
  `M4-TANGENTS-NOT-EMITTED`; native NWN1 common mesh path nie ma jeszcze
  potwierdzonego tangent field. Nie wolno gubic tego bez raportu;
- RIGID slice nie emituje skin headers, weights ani bone refs.

Niereprezentowalne wejscie zwraca stabilny fatal error; writer nigdy nie
truncuje, nie dzieli mesh automatycznie i nie zmienia kolejnosci IR.

## 5. Layout i checked arithmetic

Payload jest jednym zasobem typu 2002:

```text
file = 12-byte header || core || raw/MDX
core absolute = 12 + core relative
raw absolute  = 12 + raw_data_offset + raw relative
file_len      = 12 + raw_data_offset + raw_data_size
```

Planner najpierw liczy caly layout, bez zapisu bajtow. Kazde `add`, `mul`,
alignment i konwersja `usize/u64/u32/u16/i16/i32` jest checked. Dopiero po
udanym planie sa alokowane dokladne bufory core/raw.

Deterministyczna kolejnosc core dla rigid slice:

1. model header `0xe8`;
2. rig node headers w kolejnosci `creature.nodes`;
3. mesh node headers `0x270` w kolejnosci `creature.segments`;
4. children pointer arrays w kolejnosci binary node;
5. controller key arrays, potem controller data arrays w kolejnosci rig node;
6. face arrays w kolejnosci segmentu;
7. index-count arrays (`u32[1]`) w kolejnosci segmentu;
8. raw-index-offset arrays (`i32[1]`) w kolejnosci segmentu.

Deterministyczna kolejnosc raw dla kazdego segmentu:

1. positions `f32[3]`;
2. UV0 `f32[2]`;
3. normals `f32[3]`;
4. triangle indices `u16` w dokladnej kolejnosci `segment.indices`.

Wszystkie bloki zaczynaja sie na 4-byte alignment. Puste core arrays maja
`pointer/used/allocated = 0/0/0`. Niepotwierdzone raw pointers maja `-1`.
Node hierarchy wynika wylacznie z children arrays; runtime `geometry_ptr` i
`parent_ptr` nie sa serialized parent pointers.

Children array kazdego rig noda zawiera najpierw jego rig children w globalnym
binary-node order, a potem przypiete mesh nodes w kolejnosci segmentow. Mesh
node nie ma controllers; brak controller arrays jest jawnym identity-local
wzgledem `segment.parentNodeId`.

Model ma pusty animation pointer array, `supermodel="null"` i
`animationScale=1.0` jako jawne wartosci strukturalnego profilu produktu, nie
fakt o wymaganiu engine.

### 5.1 Zamrozony bind controller encoding

Canonical R1/R4 P-REF potwierdza pojedyncze bind rows:

```text
controller data = [0.0, px, py, pz, 0.0, qx, qy, qz, qw]
position key    = { type=8,  rows=1, timeIndex=0, dataIndex=1, columns=3 }
orientation key = { type=20, rows=1, timeIndex=4, dataIndex=5, columns=4 }
```

Quaternion jest emitowany jako `x,y,z,w`; canonical identity w R1/R4 ma
`[0,0,0,1]`. Matrix jest column-major, translacja pochodzi z `[12,13,14]`, a
proper orthonormal 3x3 jest konwertowane standardowym branch-on-trace.
Quaternion jest normalizowany i canonicalizowany znakiem: `w > 0`; dla
`w == 0` pierwszy niezerowy z `x,y,z` ma byc dodatni. Readback odtwarza macierz
z tolerancja absolutna `1e-5`. Jest to binary/P-REF plus jawny wniosek
implementacyjny; runtime pozostaje M6.

### 5.2 Zamrozone pola rigid mesh

| pole | wartosc/formula M4 | klasyfikacja i gate |
|---|---|---|
| file id/core/raw sizes | `0`, exact planned sizes | M1B binary fact; exact readback/EOF |
| model geometry type `0x6c` | `2` | canonical R1 observation; raw assertion |
| model root/count | offset wskazanego single-root rig node / total rig+mesh nodes | format fact; semantic assertion |
| model bounds/radius | world bind-pose geometry AABB; max norm from origin | implementation formula; report + readback |
| model classification/fog/child count | `4/1/0` | product profile values; readback assertion |
| model routines/unknown bytes | zero | explicit deviation; OPEN_M6 |
| node routines/geometry/parent | `0` | runtime fields; explicit deviation OPEN_M6 |
| node inherit/flags | `0`; dummy `0x01`, trimesh `0x21` | canonical P-REF observation |
| mesh routines `0x70..0x77` | zero | runtime fields; explicit deviation OPEN_M6 |
| mesh faces | one core face per IR triangle | format fact; semantic assertion |
| mesh bounds/radius/average | local AABB, max norm, arithmetic vertex mean | implementation formula; reader fields added/asserted |
| diffuse/ambient/specular | `[1,1,1]`, `[1,1,1]`, `[0,0,0]` | product defaults; OPEN_M6 |
| shininess/shadow/beaming/render | `1.0/1/0/1` | product defaults; OPEN_M6 |
| transparency/render hint/tile fade | `0/0/0` | product defaults; OPEN_M6 |
| face plane | normalized cross; distance `-dot(normal,p0)` | format/canonical observation; readback assertion |
| face surface/adjacency | `0`; `[-1,-1,-1]` | explicit product default/deviation; OPEN_M6 |
| index path | one count array entry=`indices.len`, one raw-offset entry, raw `u16` indices | canonical R1 layout; reader must decode and compare with faces |
| vertex-indices/leftover arrays | empty | canonical R1 observation for direct mesh path |
| mesh type byte `0x224` | `3` | canonical R1 observation; raw assertion |
| `startMdx` `0x228` | `0` | canonical R1 observation; raw assertion |
| positions/UV0/normals | planned raw offsets; unused raw pointers `-1` | M1B format fact; semantic assertion |
| texture count | `1` | product profile and canonical observation |
| lightmap/rotate/unknown tail | `0/0/0` | product default; OPEN_M6 |
| face-normal-sum `0x268` | `0.0` | explicit deviation; OPEN_M6 |

Pola oznaczone `OPEN_M6` nie sa przedstawiane jako engine-proven. Musza byc
jawnie zapisane w artifact deviations, aby own readback nie udawal runtime
acceptance.

## 6. Projekcja semantyczna i own readback

`M4SemanticProjectionV1` porownuje tylko dane reprezentowalne w tym profilu:

- model name, zero animacji, exact core/raw ranges i exact EOF;
- dense node mapping, nazwy, hierarchy i bind position/orientation;
- jeden syntetyczny mesh node per segment oraz jego parent;
- texture resref, positions, normals, UV0 oraz triangles z core faces i raw
  index path; obie reprezentacje musza byc identyczne;
- jawne dewiacje, w tym tangents;
- brak unexpected diagnostics i unsupported node families.

Pola `sourceSha256`, `profileId`, proof statuses i source material labels nie
maja reprezentacji w MDL i nie sa udawane jako round-trip fields. Artefakt
zachowuje layout mapping, aby readback mapowal dense partNumber z powrotem na
IR node id/segment id.

Writer po emisji zawsze uruchamia `inspect_binary_mdl` na wlasnym payloadzie i
konczy sie bledem, jezeli semantic diff nie jest pusty. Parser zostanie
zaostrzony do exact EOF; trailing bytes sa invalid.

## 7. Skin gate

Pierwszy jawny profil skin to `M4_DIRECT_CREATURE_EXTENDED64_V1`, zgodny z
target witness R1 `c_kocrachn`. Header boundary jest `node + 0x330`, a puste
lanes maja `weight=0/ref=0xffff`.

To nie wystarcza jeszcze do implementacji skin emission. Przed skin code trzeba
zamknac i przetestowac:

- forward node-partNumber -> bone-slot map i odwrotny inline mapping;
- aktywna domene slotow niezalezna od samego `map_count`;
- inverse-bind quaternion order/translation oraz bone constants;
- semantic readback `bone ref -> partNumber -> IR node id`;
- runtime index path wspolny z face triangles.

Rigid slice moze byc wdrozony i zacommitowany wczesniej. M4 nie jest finalnie
DONE, dopoki extended64 skin slice i jego readback nie przejda.

## 8. Stabilne kody M4 rigid slice

```text
M4-INVALID-SCHEMA
M4-UNSUPPORTED-PROFILE
M4-INVALID-NAME
M4-MATERIAL-BINDING-MISSING
M4-MATERIAL-BINDING-INVALID
M4-HIERARCHY-INVALID
M4-BIND-TRANSFORM-UNSUPPORTED
M4-MESH-LIMIT
M4-MESH-INVALID
M4-LAYOUT-OVERFLOW
M4-READBACK-FAILED
M4-SEMANTIC-DIFF
M4-TANGENTS-NOT-EMITTED        # recorded deviation, not fatal
```

## 9. Rigid slice DoD

- test zostaje napisany przed implementacja;
- minimal rigid i multi-node/multi-segment rigid przechodza;
- identyczne input/options daja byte-identical payload i report;
- exact EOF i wszystkie pointer ranges przechodza own reader;
- hierarchy/controllers/faces/positions/UV0/normals/resrefs maja pusty semantic
  diff;
- negative matrix ma stabilne kody dla nazw, hierarchy, transformow, limitow,
  material binding i malformed geometry;
- brak mutacji input IR;
- workspace fmt/clippy/test, WASM regression i Docker quality gate przechodza;
- niezalezny review nie ma otwartych P1/P2 dla rigid slice.
