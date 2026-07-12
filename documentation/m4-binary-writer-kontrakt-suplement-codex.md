# M4 binary MDL/MDX writer - suplement kontraktu

Data: 2026-07-12
Status: `DONE_STRUCTURAL_M4`; rigid and extended64 skin locked and verified;
runtime/controller deformation remains `OPEN_M6`

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

Extended64 rozszerza te regule bez zmiany rigid bytes:

1. po rig node headers kazdy segment nadal jest planowany w kolejnosci IR;
2. RIGID rezerwuje `0x270`; SKIN rezerwuje nierozdzielne
   `0x330 header || forwardMap[mapCount]`, po czym nastepuje 4-byte alignment;
3. children/controllers/faces/index-count/index-offset zachowuja kolejnosc
   z rigid slice;
4. po index-offset arrays sa kolejno wszystkie skin q arrays w kolejnosci
   segmentow, potem wszystkie skin t arrays, potem wszystkie constants arrays;
5. header pointers wskazuja te kategorie, a forward pointer zawsze rowna sie
   dokladnie `skinNodeOffset + 0x330`.

Skin node ma `contentFlags=0x61`. Ponizsza tabela jest zamknietym kontraktem
pol w extended64 header; nie sa dozwolone alternatywne offsety ani nadmiarowa
`allocated` capacity przechodzaca tylko przez own reader:

| offset od skin node | pole | wartosc M4 |
|---:|---|---|
| `0x270` | weights metadata `ArrayHeader` | `0/0/0` |
| `0x27c` | raw weights offset | checked `i32` do `f32[vertexCount][4]` |
| `0x280` | raw bone refs offset | checked `i32` do `u16[vertexCount][4]` |
| `0x284` | forward node-to-bone pointer | core pointer do `i16[mapCount]` |
| `0x288` | map count | `reachable_base_binary_node_count` |
| `0x28c` | q inverse `ArrayHeader` | pointer; `used=allocated=mapCount` |
| `0x298` | t inverse `ArrayHeader` | pointer; `used=allocated=mapCount` |
| `0x2a4` | constants `ArrayHeader` | pointer; `used=allocated=mapCount` |
| `0x2b0` | inline reverse map | `i16[64]` |

Deterministyczna kolejnosc raw dla kazdego segmentu:

1. positions `f32[3]`;
2. UV0 `f32[2]`;
3. normals `f32[3]`;
4. triangle indices `u16` w dokladnej kolejnosci `segment.indices`.

Dla SKIN po indices dochodza `weights f32[4]` i `refs u16[4]`, w tej
kolejnosci, z 4-byte alignment przed kolejnym segmentem. RIGID layout pozostaje
bez dodatkowych blokow.

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

Aurora First audit lokalnego R1 `c_kocrachn` zamyka structural emission contract:

```yaml
extended64_skin_v1:
  header_size: 0x330
  node_identity_domain: deterministic_preorder_tree_ordinal
  map_count: reachable_base_binary_node_count
  forward_map: "i16[mapCount], indexed by tree ordinal; value=bone slot or -1"
  inline_reverse: "i16[64], indexed by bone slot; value=tree ordinal; unused=-1 product choice OPEN_M6"
  active_slots: "distinct influencing rig nodes sorted by tree ordinal, assigned dense 0..B-1"
  slot_limit: "B <= 64 product guardrail; boundary runtime proof OPEN_M6"
  vertex_weights: "raw f32[4] per vertex; preserve M3 lane order and values"
  vertex_refs: "raw u16[4]; active lane=bone slot, zero lane=0xffff"
  weights_metadata_header: "0/0/0"
  q_inverse: "mapCount raw f32[4], WXYZ, deterministic product sign"
  t_inverse: "mapCount raw f32[3], XYZ"
  constants: "mapCount i16[2], emitted [0,0]"
```

Tree ordinal jest osobna tozsamoscia od raw `node.number`/binary partNumber.
R1 rozroznia te domeny: `Lcalf2` ma tree ordinal `7`, lecz raw node number
`23`; skin zapisuje `forward[7]=0` i `inline[0]=7`.

Dla kazdego reachable base binary node tree ordinal `i` (rig i synthesized
mesh nodes):

```text
relative_i = inverse(bindWorld(node_i)) * bindWorld(skinMeshNode)
tBoneRefInv[i] = translation(relative_i)
qBoneRefInv[i] = rotation(relative_i) written as W,X,Y,Z
boneConstant[i] = [0,0]
```

Deterministyczny product sign dla q-inverse: wybierz reprezentacje, w ktorej
`W > 0`; gdy `W == 0`, pierwszy dokladnie niezerowy element z `X,Y,Z` ma byc
dodatni. To rozszerza regule znaku bind controllers na kolejnosc WXYZ i
usuwa byte-level dowolnosc `q/-q`. R1 zawiera tez rownowazne rotacyjnie
wartosci z ujemnym `W`, dlatego canonical-byte parity znaku nie jest tu
twierdzeniem; to jawna deterministyczna decyzja produktu, a zgodnosc runtime
pozostaje `OPEN_M6`.

### 7.1 RAW controller composition boundary

Aurora First decompilation potwierdza downstream uzycie surowych komponentow
quaternionu bez widocznej normalizacji w zakotwiczonej node composition:

- `decompiled_all.c:875324-875339` - konstruktor stanu noda kopiuje cztery
  komponenty quaternionu bez transformacji;
- `:875429-875464` - local bez parenta jest kopiowany, a world composition
  uzywa przechowywanego quaternionu;
- `:1020337-1020365` i `:1020438-1020485` - odpowiednio surowe quaternion
  multiply i quaternion-to-rotated-vector, bez dzielenia przez norme;
- helper normalizacji istnieje w `:1020523-1020546`, ale nie jest wywolywany
  w powyzszej zakotwiczonej downstream node composition.

Dlatego structural M4 definiuje `bindWorld` jako kompozycje dokladnie tych
`f32 qx,qy,qz,qw`, ktore zostaly wyemitowane w controller data, bez ponownej
normalizacji downstream. Own inspection-only test odbudowuje swiaty wylacznie
z odczytanych controllers i porownuje wszystkie q/t inverse rows. Gdy
adversarialna gleboka kompozycja RAW przestaje byc proper rigid w tolerancji
`1e-5`, writer odrzuca wejscie stabilnym
`M4-SKIN-INVERSE-BIND-UNSUPPORTED`, zamiast wyemitowac niespojny skin.

Czy serialized controller type `20` normalizuje quaternion przed zapisem do
pola noda pozostaje nierozstrzygniete i `OPEN_M6`. M4 nie
przedstawia tej kwestii ani finalnej deformacji jako engine-proven.

W obecnym profilu synthesized skin mesh jest identity-local wzgledem
`segment.parentNodeId`, dlatego `bindWorld(skinMeshNode)` jest numerycznie rowne
parent world. Kontrakt zachowuje jednak jawna domene skin node. Active slots
moga wskazywac tylko influencing rig nodes; pozostale binary ordinals maja
forward `-1`, lecz nadal posiadaja q/t/constants row.

R1 ma trzy extended64 skiny, kazdy z `map/q/t/constants count=38`:

| skin | forward nonnegative | reverse first slots | active refs |
|---|---|---|---|
| `Lshinmesh01` | `7->0, 8->1` | `0->7, 1->8` | `{0,1}` |
| `Rshinmesh01` | `11->0, 12->1` | `0->11, 1->12` | `{0,1}` |
| `bodymesh01` | `4->0, 13->1` | `0->4, 1->13` | `{0,1}` |

Canonical inverse-bind reconstruction daje maksymalny blad macierzy `0` dla
WXYZ i do `2` dla XYZW. Lokalny witness ma pierwszenstwo nad konfliktem
suplementow. Wszystkie `114` canonical constants sa `[0,0]`; ich szersza
funkcja runtime pozostaje `OPEN_M6`.

Stabilne wymagania semantic readback:

- exact `node + 0x330` forward-map boundary;
- exact forward/reverse bijection dla active slots;
- kazdy active raw ref rozwiazuje sie `slot -> tree ordinal -> IR node id`;
- kazdy zero lane ma `weight=0/ref=0xffff`, a `0xffff` przy wadze dodatniej
  jest fatalny;
- q/t reconstruct `relative_i` within absolute tolerance `1e-5`;
- q/t/constants counts rowne `mapCount`, weights/ref rows rowne vertex count;
- rigid frozen payload pozostaje byte-identical.

Runtime-only named deviations: unused inline entries `-1`, granica 64 slotow,
rola constants, WXYZ deformation i finalna skin deformation wymagaja M6
Toolset/game proof. Nie blokuja structural M4 emission/readback.

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

Skin-specific stable errors:

| code | canonical path | condition |
|---|---|---|
| `M4-SKIN-LANE-INVALID` | `creature.segments[i].weights[v]` | weight/ref lane count, active `Some+positive finite`, inactive `None+0`, sum or influenceCount mismatch |
| `M4-SKIN-BONE-MISSING` | `creature.segments[i].weights[v].boneNodeIds[lane]` | active bone id is absent from rig hierarchy |
| `M4-SKIN-SLOT-LIMIT` | `creature.segments[i].weights` | distinct active influencing bones exceed product guardrail `64` |
| `M4-SKIN-MAPPING-INVALID` | `creature.segments[i]` | forward/reverse mapping is non-bijective, non-dense or resolves outside tree-ordinal domain |
| `M4-SKIN-INVERSE-BIND-UNSUPPORTED` | `creature.segments[i].parentNodeId` | relative inverse-bind matrix is non-finite or not representable as rigid WXYZ+XYZ |
| `M4-SKIN-LAYOUT-INVALID` | `layout.skinNodes[i]` | boundary/count/pointer invariant differs from locked extended64 layout |

Reader failure nadal mapuje sie do `M4-READBACK-FAILED`, a jakakolwiek roznica
skin semantic projection do `M4-SEMANTIC-DIFF` z exact skin path. Writer nie
renormalizuje, nie sortuje ani nie naprawia M3 lanes.

Negatywna macierz TDD musi rozrozniac co najmniej: SKIN z liczba weight rows
inna niz vertex count; RIGID z niepustymi weights; `influenceCount` poza `1..4`
lub niespojny z lanes; aktywne `None`, `NaN/Inf`, waga ujemna albo zerowa;
nieaktywne `Some` lub niezerowa waga; suma wag poza tolerancja `1e-5`;
nieistniejacy bone ID; oraz `B=65`. Bledy lanes/sumy/influenceCount mapuja sie
do `M4-SKIN-LANE-INVALID`, brak bone do `M4-SKIN-BONE-MISSING`, a limit do
`M4-SKIN-SLOT-LIMIT`; RIGID z weights rowniez jest
`M4-SKIN-LANE-INVALID` na exact segment path.

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

## 10. Extended64 skin slice DoD

- TDD obejmuje 1, 2 i 4 active lanes oraz osobne profile z 1, 4 i 64
  distinct slots; 65 daje `M4-SKIN-SLOT-LIMIT`;
- fixture celowo rozroznia IR id, dense partNumber i pre-order tree ordinal;
- forward map zaczyna sie dokladnie `skinNode+0x330`, ma `mapCount` rowne
  wszystkim reachable base binary nodes i jest bijekcja z active inline slots;
- raw refs rozwiazuja sie `slot -> tree ordinal -> IR node id`; zero lanes sa
  dokladnie `0/0xffff`, bez active `0xffff`;
- nontrivial analytic bind hierarchy potwierdza
  `inverse(nodeWorld)*skinNodeWorld`, WXYZ sign i XYZ translation w tolerancji
  `1e-5`;
- q/t/constants counts rownaja sie mapCount, constants sa `[0,0]`, weights
  metadata `0/0/0`;
- mixed rigid+skin, multiple skin segments i arbitrary IR order zachowuja
  deterministic bytes/report oraz nie mutuja inputu;
- mutacje boundary/map/q/t/constants/weights/refs sa odrzucane stabilnym kodem;
- frozen rigid payload pozostaje `len=1188`, core/raw `1072/104`, SHA-256
  `e100130d1dfbd18657413cdb7a701396d466cee081683591fc9836bf0c11b4b2`;
- canonical R1 reader P-REF pozostaje PASS bez payloadu w repo;
- native workspace, WASM, no-cache Docker i niezalezny review przechodza;
- M4 moze byc `DONE_STRUCTURAL` po tych gate'ach, ale runtime deformation oraz
  visual acceptance pozostaja `OPEN_M6`.

## 11. Final M4 structural closure - 2026-07-12

Rigid i extended64 skin DoD sa zamkniete. Finalna implementacja zachowuje
zamrozony rigid payload, emituje deterministic tree-ordinal forward/reverse
maps, exact zero-lane `+0.0/0xffff`, q/t/constants oraz semantic bone readback.
Wszystkie signed conversions sa sprawdzane przed alokacja, a runtime-only
wartosci maja jawne artifact deviations.

Finalne gate'y:

- writer tests `16/16`, core `145`, workspace `147` (`145 + 2 WASM native`);
- Node/WASM `14/14`, canonical CEP `1/1` z szescioma packetami;
- Docker no-cache `m2a-quality:m4-skin-final` PASS;
- dwa finalne niezalezne rereview: `P1=0`, `P2=0`;
- runtime/controller deformation nadal nalezy do M6.
