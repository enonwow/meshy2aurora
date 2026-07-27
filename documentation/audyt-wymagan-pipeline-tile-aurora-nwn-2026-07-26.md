# Audyt wymagań pipeline tile dla Aurora Toolset i NWN

Data: 2026-07-26  
Status: `IMPLEMENTATION_COMPLETE_OFFLINE / READY_FOR_OWNER_PROOF`  
Zakres: `Meshy/GLB -> tile MDL + WOK + SET + HAK + test MOD -> owner proof`

## 1. Werdykt

Meshy2Aurora ma już większość wspólnego fundamentu potrzebnego do tile:

- import GLB i neutralny `AuroraModelIrV1`;
- wspólny binary MDL writer/readback dla geometrii rigid;
- tekstury TGA/DDS;
- ogólny writer/readback HAK i MOD;
- writer/readback GFF oraz istniejący szkielet Area z `Tileset` i
  `Tile_List`.

Nie ma jednak jeszcze pipeline tile. Cztery blokery P0 to:

1. profil emisji `TileStaticV1` w tym samym writerze MDL, którego używają
   creature i placeable;
2. semantyczny reader/writer węzła binary MDL `AABB` (`flags=0x221`) wraz
   z drzewem AABB;
3. osobny serializer/readback tekstowego WOK typu `2016`;
4. reader/writer SET typu `2013` oraz resolver
   `ARE.Tile_ID -> SET.TILEN.Model -> MDL/WOK`.

Najważniejsza korekta względem wcześniejszej dokumentacji:

> WOK tile'a nie jest zasobem wskazanym przez wartość pola
> `SET.[TILEN].WalkMesh`. Lokalny corpus retail pokazuje, że WOK typu `2016`
> ma ten sam resref co `Model`. Dla przykładu `tms01` tile `12` deklaruje
> `Model=tms01_c01_01` i `WalkMesh=msb01`, a faktyczne zasoby to
> `tms01_c01_01.mdl` oraz `tms01_c01_01.wok`. `msb01.wok` nie istnieje.

Pierwszy pionowy przekrój powinien dostarczyć jeden płaski, statyczny,
przechodni tile `10 x 10`, powtarzany w Area `2 x 2`. Wizualny mesh z Meshy
może mieć `20 000` trójkątów, ale WOK i AABB muszą używać osobnej, uproszczonej
geometrii nawigacyjnej.

## 2. Klasy twierdzeń

Każde ustalenie w dokumencie należy do jednej z klas:

- **[DECOMP FACT]** — fakt z lokalnej dekompilacji Aurora Toolset;
- **[RETAIL FACT]** — fakt z lokalnego, odczytanego in-place zasobu NWN;
- **[PROJECT FACT]** — stan kodu w kanonicznym repo Meshy2Aurora;
- **[REFERENCE FACT]** — obserwacja z `C:\Projects\aurora-web` albo lokalnego
  repozytorium pomocniczego; nie jest zależnością ani oracle produktu;
- **[IMPLEMENTATION DECISION]** — decyzja projektowa wynikająca z faktów;
- **[OPEN]** — hipoteza lub kontrakt, który nadal wymaga testu.

Audyt nie uruchamiał Aurora Toolset ani NWN i nie wykonuje wizualnego proofu.
Końcową weryfikację w obu aplikacjach wykonuje właściciel.

## 3. Źródła prawdy

### 3.1. Dekompilacja Aurora

Źródło:

`C:\Projects\New Folder\export\decompiled_all.c`

Najważniejsze kotwice:

| Kontrakt | Funkcja / lokalizacja |
|---|---|
| lookup SET typu `2013` | `decompiled_all.c:255530-255536` |
| parser całego SET | `FUN_005e3df4`, początek `decompiled_all.c:281755` |
| parser jednego `[TILEN]` | `FUN_005dfa50`, początek `decompiled_all.c:279314` |
| pobranie modelu z `TTileSetTile` | `FUN_005ddc4c`, początek `decompiled_all.c:278142` |
| budowa instancji tile | `FUN_00712100`, początek `decompiled_all.c:452870` |
| wspólny loader modelu | `FUN_00712380 -> FUN_00a546cc`, `decompiled_all.c:452973` |
| odczyt `ARE.Tile_List` | `decompiled_all.c:255345-255420` |
| zapis `ARE.Tile_List` | `decompiled_all.c:255934-255952` |
| pozycja środka komórki co 10 jednostek | `decompiled_all.c:255369-255373` |
| wysokość tile | `Tile_Height * tileset[0x60]`, `decompiled_all.c:255396-255399` |

Stringi i RTTI:

`C:\Projects\New Folder\export\strings.tsv`

- `CResHelper<CResSET,2013>`: wiersze `927` i `966`;
- `MdlNodeAABB`: rodzina wspólnej fabryki node'ów MDL;
- klucze `Model`, `WalkMesh`, `PathNode`, `VisibilityNode`,
  `DoorVisibilityNode`, orientacje, światła i `AnimLoop1..3`.

### 3.2. Lokalny retail NWN

Odczyt wykonano wyłącznie in-place przez KEY/BIF. Żaden payload retail nie
został skopiowany do repo.

```text
KEY:
C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\data\nwn_base.key

SHA-256:
09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935

KEY entries:
113 489

resource counts:
MDL 2002 = 32 832
SET 2013 = 33
WOK 2016 = 13 118
```

Reprezentatywny, wcześniej użyty przez projekt tile:

| Zasób | Locator | SHA-256 |
|---|---|---|
| `tms01.set` / `2013` | KEY `17885`, `data\aurora_tms.bif` | `9d764583f2e1a0e8a0473dba6c2e665226ff306bdcd1d710962fa1b8bdb64a8f` |
| `tms01_c01_01.mdl` / `2002` | KEY `17910` | `b8736f8f4b06124a8b0619cbfb0265607e16f62b098320cf8518d9aebc096c3e` |
| `tms01_c01_01.wok` / `2016` | KEY `17911` | `88fd0497b89a82975449c6e6db48f0f19397fd6c3823c361a2f0ee8e2784b1f5` |
| `surfacemat.2da` / `2017` | KEY `26707`, `data\base_2da.bif` | `dde79c68899fb95b89baa334b30db2b6e29c8a5d30c43910ce28b8bfd1af845f` |

`tms01` tile `12` jest dobrym wzorcem pierwszego profilu:

- `Model=tms01_c01_01`;
- `WalkMesh=msb01`;
- cztery narożniki `Grass`, wysokość `0`;
- `PathNode=A`;
- `Orientation=0`;
- binary MDL ma `classification=2`;
- WOK ma zakres XY `[-5, -5]..[5, 5]`, `9` rekordów vertices,
  `8` faces i powierzchnię `3` (`Grass`, `Walk=1`);
- MDL ma jeden node AABB `0x221`, `8` faces oraz drzewo
  `15` rekordów: `7` wewnętrznych i `8` liści;
- liście AABB pokrywają dokładnie indeksy faces `0..7`.

### 3.3. Corpus retail SET -> MDL/WOK

Read-only skan wszystkich `33` bazowych SET-ów dał:

| Miara | Wynik |
|---|---:|
| deklarowane i znalezione sekcje tile | `12 342` |
| unikalne wartości `Model` | `12 184` |
| unikalne tokeny pola `WalkMesh` | `6` |
| brakujące pary MDL/WOK | `2` znane wpisy źródłowe |
| istniejący WOK pod resrefem `Model` | `12 182 / 12 184` |
| istniejący WOK pod wartością pola `WalkMesh` | `0` |
| zbadane istniejące WOK w innym niż ASCII formacie | `0` |
| binary MDL z nodem AABB | `8 607` |
| ASCII MDL z deklaracją `node aabb` | `3 564` |
| modele tile bez AABB | `11` specjalnych/outlierowych modeli |

Wniosek nie brzmi „każdy możliwy tile musi mieć AABB”. Corpus zawiera
jedenaście wyjątków. Pierwszy obsługiwany przez Meshy2Aurora profil będzie
jednak jawnie wymagał AABB, ponieważ wybrany przechodni wzorzec
`tms01_c01_01` go ma, a `12 171 / 12 182` istniejących modeli referencyjnych
również go zawiera.

### 3.4. Stan Meshy2Aurora

Źródła:

- `crates/m2a-core/src/model_ir.rs`;
- `crates/m2a-core/src/mdl/parse_binary_mdl.rs`;
- `crates/m2a-core/src/mdl/writer_types.rs`;
- `crates/m2a-core/src/mdl/write_binary_mdl.rs`;
- `crates/m2a-core/src/placeable_collision.rs`;
- `crates/m2a-core/src/hak.rs`;
- `crates/m2a-core/src/proof_module.rs`;
- `crates/m2a-wasm/src/lib.rs`;
- `apps/studio-web/src/worker/types.ts`;
- `apps/studio-web/src/worker/m2a.worker.ts`.

### 3.5. Źródła pomocnicze

`C:\Projects\aurora-web` jest reference-only. Jego:

- `aurora-set-tileset-parser.ts` potwierdza praktyczny podział SET na defaults,
  terrain, crossers, rules, tiles i groups;
- `aurora-wok-walkmesh.ts` parsuje tekstowy WOK, vertices, faces i rekordy
  AABB, lecz jawnie zachowuje nierozstrzygnięty tail face jako surowe tokeny;
- parser SET nie zachowuje dziś pola `WalkMesh`, `Doors` ani `Sounds`, więc nie
  może być skopiowany jako kompletny kontrakt Meshy2Aurora.

Lokalne `nwn-tools` jest tylko niezależnym potwierdzeniem struktury binary
AABB. `NwnMdlNodes.h` oraz `NwnMdlSerialize.cpp` wskazują:

- pointer do drzewa na końcu mesh headera, offset noda `+0x270`;
- rekord drzewa długości `0x28`;
- bounds min/max, pointer lewy/prawy, leaf face oraz plane;
- `leafFace=-1` oznacza node wewnętrzny.

Układ ten został niezależnie potwierdzony bezpośrednim odczytem bajtów
retailowego `tms01_c01_01.mdl`; nie opieramy implementacji wyłącznie na
zewnętrznym kodzie.

## 4. Rzeczywisty pipeline rozwiązywania tile

```text
MOD / module.ifo
    |
    +-- ordered HAK list
    |
    v
ARE
    +-- Tileset = <set_resref>
    +-- Width, Height
    +-- Tile_List[y * Width + x]
           +-- Tile_ID
           +-- Tile_Orientation
           +-- Tile_Height
           +-- światła i AnimLoop1..3
                    |
                    v
SET type 2013
    +-- [TILE<Tile_ID>]
           +-- Model = <model_resref>
           +-- terrain/corners/heights/crossers
           +-- PathNode/Visibility/doors/lights/loops
                    |
                    +----------------------+
                    |                      |
                    v                      v
          MDL type 2002           WOK type 2016
          resref = Model          resref = Model
          render + AABB node      ASCII walkmesh + AABB tree
                    |
                    v
             textures/materials
```

**[DECOMP FACT]** SET jest parserem domenowego manifestu tilesetu. Po pobraniu
`Model` Aurora tworzy model przez ten sam wspólny loader MDL, którego używają
inne kategorie obiektów.

**[RETAIL FACT]** WOK jest wiązany po resrefie modelu, nie po tokenie
`SET.WalkMesh`.

**[IMPLEMENTATION DECISION]** W `m2a-core` nie powstanie osobny
`TileMdlParser`. Powstaną adapter tile, AABB w istniejącym parserze/writerze,
serializer WOK i parser/writer SET.

## 5. Kontrakt ARE

### 5.1. Pola instancji tile

**[DECOMP FACT]** Każdy element `ARE.Tile_List` ma dokładnie potwierdzony
zestaw:

- `Tile_ID` — Int;
- `Tile_Orientation` — Int;
- `Tile_Height` — Int;
- `Tile_MainLight1` — Byte;
- `Tile_MainLight2` — Byte;
- `Tile_SrcLight1` — Byte;
- `Tile_SrcLight2` — Byte;
- `Tile_AnimLoop1` — Byte;
- `Tile_AnimLoop2` — Byte;
- `Tile_AnimLoop3` — Byte.

`Tile_group` nie jest polem ARE. Grupy należą do SET.

### 5.2. Kardynalność i współrzędne

**[DECOMP FACT]**

- liczba rekordów musi odpowiadać `Width * Height`;
- indeks ma postać `y * Width + x`;
- środki komórek są rozmieszczone co `10` jednostek:
  `x * 10 + 5`, `y * 10 + 5`;
- `Tile_Height` jest mnożony przez wartość przejścia wysokości z tilesetu.

**[IMPLEMENTATION DECISION]** Pierwszy fixture użyje Area `2 x 2`, czterech
rekordów jednego customowego `Tile_ID=0`, wysokości `0` i czterech orientacji
testowych dopiero po przejściu wariantu `orientation=0`.

## 6. Kontrakt SET

### 6.1. Warstwy danych

**[DECOMP FACT + RETAIL FACT]** SET jest tekstowym manifestem sekcyjnym:

| Warstwa | Potwierdzone sekcje/pola |
|---|---|
| tożsamość | `[GENERAL]`: `Name`, `Type=SET`, `Version`, `DisplayName`, `UnlocalizedName`, `Interior`, `HasHeightTransition`, `Transition`, `EnvMap`, `Border`, `Default`, `Floor` |
| grass | `[GRASS]` i parametry grass |
| terrain | `[TERRAIN TYPES]`, `[TERRAINn]` |
| crossers | `[CROSSER TYPES]`, `[CROSSERn]` |
| reguły | `[PRIMARY RULES]`, `[PRIMARY RULEn]`, `[SECONDARY RULES]`, `[SECONDARY RULEn]` |
| tiles | `[TILES]`, `[TILEn]` |
| grupy | `[GROUPS]`, `[GROUPn]` |

Parser `FUN_005e3df4` liczy elementy i konstruuje tile, grupy, typy terenu,
crossers oraz reguły. Nie wolno implementować SET jako mapy zawierającej tylko
`Tile_ID -> Model`.

### 6.2. Pola `[TILEn]`

**[DECOMP FACT]** `FUN_005dfa50` czyta:

- `Model`;
- `WalkMesh`;
- `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`;
- cztery odpowiadające pola `*Height`;
- `Top`, `Right`, `Bottom`, `Left`;
- `PathNode`;
- `VisibilityNode` i `VisibilityOrientation`;
- `DoorVisibilityNode` i `DoorVisibilityOrientation`;
- `Orientation`;
- `ImageMap2D`;
- `MainLight1`, `MainLight2`;
- `SourceLight1`, `SourceLight2`;
- `AnimLoop1`, `AnimLoop2`, `AnimLoop3`;
- podstawowe liczniki/metadane powiązane z dalszymi rekordami.

Bezpośrednie pętle parsera całego SET po wywołaniu `FUN_005dfa50` dokładają
rekordy drzwi i dźwięków należące do danego tile'a. Dlatego `Doors` i `Sounds`
są częścią `TileDescriptor`, chociaż nie należy przypisywać całego ich parse
samemu `FUN_005dfa50`.

W całym lokalnym corpusie każde z `12 342` pól tile zawiera bazowy zestaw od
`Model` do `ImageMap2D`. `VisibilityNode` występuje w `4 099`, a
`DoorVisibilityNode` w `336` wpisach.

### 6.3. Minimalny SET a pełny tileset

**[IMPLEMENTATION DECISION]** V1 ma serializer jawnego podzbioru:

- prawidłowe `[GENERAL]`;
- jeden terrain type;
- zero crossers;
- minimalna, jawna reguła lub zero reguł, zależnie od testu parsera Toolset;
- jeden `[TILE0]`;
- zero grup w wariancie bez painter authoring;
- komplet bazowych pól `[TILE0]`, nawet jeśli wartości są puste/zerowe.

**[OPEN]** Sam SET z jednym tile i bez grup musi zostać sprawdzony przez
właściciela w Toolsecie. Zielony parser/readback nie dowodzi, że edytor
zaakceptuje go jako pełny palette tileset. Jeżeli celem stanie się malowanie
terenu i grup, trzeba wdrożyć cały kontrakt terrain/crossers/rules/groups.

## 7. Kontrakt render MDL

### 7.1. Wspólny pipeline

**[DECOMP FACT]** Tile kończy w `FUN_00a546cc`, czyli wspólnym loaderze MDL.

**[PROJECT FACT]** `AuroraModelIrV1` jest neutralny względem category. To jest
właściwe wejście do tile, tak samo jak do creature/placeable.

### 7.2. Profil tile

**[RETAIL FACT]** Reprezentatywny binary tile ma:

- `classification=2`;
- root dummy `0x01`;
- render mesh `0x21`;
- AABB mesh `0x221`;
- `fog=1`;
- własne bounds;
- tekstury powiązane przez wspólny mesh/material contract.

**[PROJECT FACT]** Obecny `MdlFormatProfileV1` nie ma profilu tile. Writer
ustawia model classification na stałą używaną przez aktualne profile i emituje
tylko dummy/mesh/skin. Placeable profile nie jest wystarczający dla tile.

**[IMPLEMENTATION DECISION]** Dodać jawny profil:

```rust
MdlFormatProfileV1::TileStaticV1
```

Profil musi co najmniej:

- emitować `classification=2`;
- używać caller-owned model bounds;
- emitować statyczne render meshes ze wspólnego `AuroraModelIrV1`;
- emitować jeden oddzielny AABB mesh z geometrii nawigacyjnej;
- nie dodawać creature rig, skin ani local animation state trees;
- zachować wspólny limit jednego triangle-list streamu:
  `65 535` indeksów, czyli `21 845` trójkątów;
- pozwalać na wiele render mesh nodes, jeśli model przekracza limit jednego
  streamu.

Model wizualny `20 000` trójkątów mieści się w jednym mesh node. Limit nie
dotyczy całego modelu, lecz pojedynczego strumienia indeksów.

### 7.3. Binary AABB

**[RETAIL FACT + REFERENCE FACT]** Minimalny layout:

```text
AABB mesh node:
  common node header
  common mesh header
  +0x270: u32 pointer do korzenia drzewa

AABB entry: 0x28 bajtów
  +0x00 vec3 boundsMin
  +0x0c vec3 boundsMax
  +0x18 u32 leftPointer
  +0x1c u32 rightPointer
  +0x20 i32 leafFace
  +0x24 u32 plane
```

Kontrakt drzewa:

- `leafFace=-1` oznacza rekord wewnętrzny;
- rekord wewnętrzny ma dwa prawidłowe child pointery;
- rekord liścia ma indeks istniejącego face;
- bounds potomka mieszczą się w bounds rodzica;
- nie ma cyklu, aliasingu niedozwolonego przez profil ani pointera poza core;
- wszystkie faces wymagane przez profil są pokryte dokładnie raz;
- root bounds obejmuje całą geometrię AABB.

**[PROJECT FACT]** Parser zna flagę `0x200` jako `aabb`, ale
`SUPPORTED_NODE_FLAGS` obejmuje dziś tylko `HEADER | MESH | SKIN`.
`NodeReport` nie ma semantycznego pola drzewa AABB, a writer nie emituje
pointera `+0x270` ani rekordów.

## 8. Kontrakt WOK

### 8.1. Format

**[RETAIL FACT]** Wszystkie `12 182` istniejące WOK powiązane z wpisami SET w
bazowym corpusie są tekstowe. Reprezentatywny plik zaczyna się od rodziny
formatu:

```text
#MAXWALKMESH ASCII
beginwalkmeshgeom <model_resref>
node aabb <node_name>
...
endnode
endwalkmeshgeom <model_resref>
```

Wewnątrz znajdują się:

- `parent`;
- `position`;
- `orientation`;
- `wirecolor`;
- `verts <count>` i wiersze XYZ;
- `faces <count>` i osiem liczb na face;
- `aabb` oraz kolejne siedmioelementowe rekordy drzewa.

Osiem pól face odpowiada obecnemu wspólnemu kontraktowi:

```text
vertex0 vertex1 vertex2 smoothing adjacency0 adjacency1 adjacency2 surfaceId
```

WOK nie jest binary MDL i nie może używać obecnego binary writera.

### 8.2. Powierzchnie

**[RETAIL FACT]** `surfacemat.2da` definiuje m.in.:

| ID | Label | Walk |
|---:|---|---:|
| `1` | Dirt | `1` |
| `2` | Obscuring | `0` |
| `3` | Grass | `1` |
| `4` | Stone | `1` |
| `5` | Wood | `1` |
| `7` | Nonwalk | `0` |

Placeable PWK używa powierzchni `7`, ponieważ opisuje przeszkodę. Tile WOK musi
zawierać co najmniej jedną powierzchnię z `Walk=1`. Nie wolno ponownie użyć
placeable default `7`.

### 8.3. WOK i AABB w MDL

**[RETAIL FACT]** Dla `tms01_c01_01`:

- WOK ma `9` vertices, `8` faces, powierzchnię `3` i `15` rekordów AABB;
- AABB node w MDL ma `12` vertices, `8` faces, powierzchnię `3` i również
  `15` rekordów drzewa;
- oba warianty pokrywają ten sam zakres `[-5, -5]..[5, 5]`;
- liście obu drzew odwołują się do faces `0..7`.

**[IMPLEMENTATION DECISION]** Źródłem WOK i noda MDL AABB ma być jeden
`TileNavigationIr`, ale oba serializery zachowują własną kopertę i dozwoloną
topologię wierzchołków. Nie należy:

- serializować WOK binary writerem;
- używać wysokopoligonowego render mesha jako WOK;
- wyprowadzać resrefu WOK z pola `SET.WalkMesh`;
- używać placeable surface `7` jako walkable floor.

### 8.4. Seamy

**[IMPLEMENTATION DECISION]** V1 wymaga:

- tile-local bounds XY dokładnie `-5..5`;
- graniczne vertices leżące na `x=-5`, `x=5`, `y=-5`, `y=5`;
- identyczne wysokości po obu stronach seam dla powtarzanego tile'a;
- adjacency wewnątrz tile poprawne;
- brak degenerate faces i NaN/Inf;
- co najmniej jeden spójny komponent `Walk=1`.

**[OPEN]** Dokładna semantyka globalnego stitchingu wielu różnych tile'i,
przejść wysokościowych i crossers pozostaje zakresem pełnego tilesetu, nie
pierwszego statycznego tile'a.

## 9. Pakowanie HAK i MOD

### 9.1. Minimalny HAK

```yaml
resources:
  - resref: "<tileset_resref>"
    type: 2013
    extension: "set"
    required: true
  - resref: "<model_resref>"
    type: 2002
    extension: "mdl"
    required: true
  - resref: "<model_resref>"
    type: 2016
    extension: "wok"
    required: true
  - resref: "<diffuse_resref>"
    type: 3
    extension: "tga"
    required: true
  - resref: "<image_map_resref>"
    type: 3
    extension: "tga"
    required: false_for_runtime_required_for_complete_authoring
```

Opcjonalne później:

- DDS `2033`;
- TXI `2022`;
- MTR `2072`;
- dodatkowe tekstury/lightmapy;
- zasoby drzwi, dźwięków i animowanych tile'i.

**[PROJECT FACT]** Ogólny `HakResourceInputV1` przyjmuje dowolny type `u16`,
więc writer HAK nie wymaga osobnej implementacji dla SET/WOK. Potrzebuje
natomiast tile-specific manifestu i readbacku wiązań.

### 9.2. Minimalny MOD

MOD musi zawierać:

- `module.ifo` z dokładnie uporządkowaną listą HAK;
- ARE z customowym `Tileset`;
- `Width=2`, `Height=2`;
- dokładnie cztery rekordy `Tile_List`;
- GIT/GIC zgodne z istniejącym module builderem;
- entry point nad przechodnią powierzchnią WOK.

**[PROJECT FACT]** `proof_module.rs` potrafi już pisać pola ARE i pakować MOD,
ale używa stockowego `tms01`. To jest wspólne rusztowanie Area, nie custom tile
pipeline.

## 10. Co jest wspólne, a co tile-specific

| Warstwa | Stan | Decyzja |
|---|---|---|
| GLB ingest | gotowe | współdzielić |
| normalizacja basis/transform | gotowe dla model IR | współdzielić, dodać tile bounds gate |
| `AuroraModelIrV1` | gotowe | współdzielić |
| render mesh MDL | gotowy fundament | współdzielić |
| parser binary MDL | gotowy dla dummy/mesh/skin | rozszerzyć o AABB, nie forkować |
| writer binary MDL | gotowy dla istniejących profili | dodać `TileStaticV1` |
| materiały i TGA/DDS | gotowy fundament | współdzielić |
| `face_surface_ids` | jest w IR | wykorzystać w navigation IR |
| placeable PWK | gotowy ASCII subset | wyodrębnić wspólną gramatykę, nie używać koperty PWK jako WOK |
| WOK | brak | nowy serializer/readback |
| SET | brak | nowy parser/writer/resolver |
| ARE/GFF | częściowo gotowe | rozszerzyć o custom tileset fixture |
| HAK/MOD | gotowy ogólny writer | dodać tile manifest i verifier |
| WASM | brak tile export | dodać boundary |
| Studio Web | brak lane tile | dodać workflow i WOK preview |

## 11. Docelowe granice modułów

```text
model_ir.rs
  AuroraModelIrV1                 # wspólny render model

mdl/
  parse_binary_mdl.rs             # + semantic AABB
  write_binary_mdl.rs             # + TileStaticV1
  semantic_readback.rs            # + AABB tree diff

walkmesh/
  ascii_common.rs                 # wspólne tokeny PWK/WOK
  tile_navigation_ir.rs
  tile_wok_reader.rs
  tile_wok_writer.rs
  aabb_tree.rs                    # deterministyczny builder i walidator

tile/
  set_types.rs
  set_reader.rs
  set_writer.rs
  tile_resolver.rs
  tile_profile.rs
  tile_package.rs
  tile_package_readback.rs

wasm/
  build_meshy_static_tile_package_v1

studio-web/
  BUILD_TILE_PACKAGE
  TileReview
  render/WOK overlay
```

Proponowane neutralne typy:

```rust
struct TileNavigationIrV1 {
    model_resref: String,
    node_name: String,
    vertices: Vec<[f32; 3]>,
    faces: Vec<WalkmeshFaceV1>,
    aabb_tree: AabbTreeV1,
}

struct TileDescriptorV1 {
    tile_id: u32,
    model_resref: String,
    walkmesh_class_token: String,
    corner_terrain: [String; 4],
    corner_heights: [i32; 4],
    edge_crossers: [Option<String>; 4],
    path_node: String,
    orientation_quarter_turns: u8,
    image_map_2d: Option<String>,
    lights: TileLightFlagsV1,
    anim_loops: [bool; 3],
}
```

`walkmesh_class_token` przechowuje `SET.WalkMesh`, ale nie jest używany jako
resource resref.

## 12. Pierwszy pionowy przekrój

### 12.1. Zakres

Pierwszy kandydat:

- jeden zewnętrzny, płaski tile;
- footprint `10 x 10`;
- render model z Meshy, maksymalnie `20 000` trójkątów w jednym mesh node;
- jedna diffuse TGA;
- własny binary MDL `classification=2`;
- osobny AABB node;
- osobny ASCII WOK;
- walkable surface `3` lub `4`, jawnie wybrany w profilu;
- jeden custom SET;
- jeden HAK;
- jeden test MOD z Area `2 x 2`;
- brak drzwi, dźwięków, crossers, height transitions i animowanych tile'i.

### 12.2. Niezakres V1

- terrain painter z pełną automatyczną zamianą sąsiadów;
- wielokomórkowe groups;
- crossers, drogi, rzeki i mosty;
- height transitions;
- drzwi tile i door visibility;
- animowane elementy tile;
- światła emiterowe;
- water/transparent walkmesh;
- drugie piętro i `tilefade`;
- pełne minimapy;
- automatyczne generowanie całej rodziny wariantów narożników.

### 12.3. Dlaczego Area `2 x 2`

`1 x 1` potwierdzi tylko lokalne ładowanie. `2 x 2` pozwala równocześnie
sprawdzić:

- cztery instancje tego samego modelu;
- granice `10` jednostek;
- seam render;
- seam WOK;
- cardinality `Width * Height`;
- później cztery orientacje bez zmiany zasobów.

## 13. Checklista implementacji

Checkbox `[x]` oznacza wyłącznie etap rzeczywiście zamknięty. Audyt nie odhacza
funkcji, których jeszcze nie ma.

### T0 — kontrakt źródeł

- [x] potwierdzić SET jako resource type `2013`;
- [x] potwierdzić WOK jako resource type `2016`;
- [x] potwierdzić wspólny loader MDL typu `2002`;
- [x] zmapować pola `[TILEn]` z dekompilacji;
- [x] zmapować pola `ARE.Tile_List` z odczytu i zapisu;
- [x] potwierdzić siatkę `10 x 10`;
- [x] przeskanować lokalny corpus SET/MDL/WOK;
- [x] potwierdzić ASCII WOK;
- [x] potwierdzić wiązanie WOK po resrefie `Model`;
- [x] potwierdzić binary AABB layout na retail bytes;
- [x] wybrać `tms01` tile `12` jako wzorzec pierwszego profilu;

### T1 — testy i typy wspólne

- [x] dodać testy negatywne przed kodem produkcyjnym;
- [x] wprowadzić wspólny `WalkmeshFaceV1`;
- [x] wprowadzić `AabbTreeV1` i `AabbEntryV1`;
- [x] wprowadzić `TileNavigationIrV1`;
- [x] wprowadzić `TileDescriptorV1` i `TilesetIrV1`;
- [x] rozdzielić render geometry od navigation geometry;
- [x] zachować kompatybilność publicznego placeable API;
- [x] dodać stabilne kody błędów `TILE-*`;

### T2 — binary MDL AABB reader

- [x] rozszerzyć `NodeReport` o semantyczny AABB;
- [x] obsłużyć node flags `0x221`;
- [x] odczytać pointer `node + 0x270`;
- [x] odczytać rekordy `0x28`;
- [x] walidować core bounds i overflow;
- [x] odrzucać cykle;
- [x] odrzucać child pointer poza core;
- [x] odrzucać leaf face poza zakresem faces;
- [x] odrzucać niepełne lub wielokrotne pokrycie faces;
- [x] sprawdzać zawieranie bounds child -> parent;
- [x] dodać mutation tests każdego pointera i licznika;
- [x] dodać env-gated read-only test retail KEY/BIF bez kopiowania payloadu;

### T3 — binary MDL AABB writer i profil tile

- [x] dodać `MdlFormatProfileV1::TileStaticV1`;
- [x] emitować `classification=2`;
- [x] używać caller-owned bounds;
- [x] emitować render meshes z `AuroraModelIrV1`;
- [x] emitować AABB mesh `0x221`;
- [x] emitować deterministyczne drzewo AABB;
- [x] zapewnić jednoznaczne pokrycie wszystkich AABB faces;
- [x] wykonać wspólny binary MDL semantic readback;
- [x] wykonać determinism test byte-for-byte;
- [x] zachować limit `21 845` trójkątów per mesh stream;
- [x] dodać test modelu wizualnego dokładnie `20 000` trójkątów;

### T4 — ASCII WOK

- [x] wydzielić wspólną gramatykę ASCII walkmesh z placeable PWK;
- [x] dodać kopertę `MAXWALKMESH / beginwalkmeshgeom / endwalkmeshgeom`;
- [x] emitować `node aabb`;
- [x] emitować vertices i ośmiopolowe faces;
- [x] zachować surface ID per face;
- [x] emitować adjacency;
- [x] emitować siedmiopolowe rekordy drzewa AABB;
- [x] odczytać własny output i porównać semantykę;
- [x] walidować `Walk=1` przez jawny profil powierzchni;
- [x] odrzucać surface `7` dla jedynej podłogi V1;
- [x] odrzucać binary payload;
- [x] odrzucać złe count, NaN/Inf, degenerate faces i OOB indices;
- [x] testować bounds `-5..5`;
- [x] testować seam dwóch identycznych tile'i;
- [x] potwierdzić, że resref WOK jest równy resrefowi `Model`;

### T5 — SET reader/writer/resolver

- [x] dodać parser sekcji i kluczy bez utraty nieznanych pól;
- [x] parsować `[GENERAL]`;
- [x] parsować terrain/crossers;
- [x] parsować primary/secondary rules;
- [x] parsować pełny `[TILEn]`;
- [x] parsować groups i `TileN=-1` holes;
- [x] walidować wszystkie `Count`;
- [x] walidować ciągłość `TILE0..TILE(count-1)`;
- [x] walidować referencje group -> tile;
- [x] zachować `WalkMesh` jako token domenowy;
- [x] zabronić resolverowi używania `WalkMesh` jako WOK resrefu;
- [x] pisać deterministyczny minimalny SET V1;
- [x] wykonać parser -> writer -> parser semantic roundtrip;
- [x] dodać mutation tests braków sekcji, count i tile ID;

### T6 — package builder i offline verifier

- [x] dodać `StaticTileBuildRequestV1`;
- [x] generować MDL, WOK, SET i TGA;
- [x] pakować zasoby `2002/2016/2013/3` do HAK;
- [x] wygenerować MOD z customowym `Tileset`;
- [x] wygenerować `2 x 2` i cztery rekordy `Tile_List`;
- [x] ustawić entry nad face `Walk=1`;
- [x] dołączyć dokładnie jeden ordered HAK;
- [x] wykonać HAK/MOD readback po `(resref, type)`;
- [x] zweryfikować SHA-256 każdego payloadu;
- [x] zweryfikować ARE -> SET -> MDL/WOK chain;
- [x] zweryfikować WOK seam offline;
- [x] wygenerować raport bez retail payloadów;

### T7 — WASM i Studio Web

- [x] dodać `buildMeshyStaticTilePackageV1`;
- [x] dodać worker request `BUILD_TILE_PACKAGE`;
- [x] przekazywać GLB i opcje tile wyłącznie jako bytes/JSON;
- [x] zwracać HAK, MOD, raport i readback jako artefakty Workera;
- [x] dodać wybór `Tile` w Studio;
- [x] dodać kontrolę terrain/surface/interior;
- [x] dodać podgląd footprintu `10 x 10`;
- [x] dodać overlay render mesh / WOK / AABB;
- [x] pokazać model i WOK triangle counts osobno;
- [x] pokazać SET/MDL/WOK resref bindings;
- [x] umożliwić pobranie HAK i MOD;
- [x] dodać test Worker/WASM bez backendu i filesystemu;

### T8 — owner proof

- [x] zamrozić jeden exact candidate i hashe;
- [x] przygotować handoff zaczynający się od dokładnej nazwy `.mod`;
- [x] podać nazwę modułu widoczną w Toolsecie;
- [x] podać dokładną nazwę Area;
- [x] podać HAK, SET, Tile_ID, MDL, WOK i placement;
- [ ] właściciel potwierdza brak `bad tile group`;
- [ ] właściciel potwierdza widoczność w Aurora Toolset;
- [ ] właściciel potwierdza widoczność w NWN;
- [ ] właściciel potwierdza spawn na powierzchni `Walk=1`;
- [ ] właściciel przechodzi przez seam co najmniej dwóch tile'i;
- [ ] właściciel potwierdza brak niewidzialnej bariery na środku podłogi;
- [ ] wynik zostaje zapisany osobno dla render, walkability i seams;

### Zamrożony kandydat offline

Kandydat został zmaterializowany dokładnie raz po zielonym readbacku do
`artifacts/tile-static-v1`. Na bezpośrednie polecenie właściciela nie
instalowano go do natywnych katalogów NWN i nie uruchamiano ani nie
kontrolowano Aurora Toolset/NWN.

```yaml
candidate:
  status: "ready_for_owner_proof"
  module_file: "m2a_tile_static_v1.mod"
  module_sha256: "aa5f4d5a3e99406332eef53e4e6bfa29ced954a7ecdb73c66d80bec0ed3841a7"
  module_display_name: "Meshy2Aurora Tile Static V1"
  area_name: "M2A Tile Static 2x2"
  area_resref: "m2atilearea"
  entry_position: [5, 5, 0]
  hak_file: "m2a_tile_static_v1.hak"
  hak_sha256: "50606968f5e981b1d816d45c3471f3bc1a0871e621262ade4969f48e40768ee8"
  ordered_hak_resref: "m2atilestv1"
  set: "m2atilesetv1.set"
  tile_id: 0
  mdl: "m2atilemdl1.mdl"
  wok: "m2atilemdl1.wok"
  walkmesh_token: "msb01"
  surface_id: 3
  area_size: [2, 2]
  model_visibility: "not_tested"
  proof_completeness: "missing"
```

Offline wykonano: syntetyczne i mutacyjne testy AABB/WOK/SET, read-only
KEY/BIF na dokładnym retailowym `tms01_c01_01.mdl`, wspólne regresje
creature/placeable/tile, natywne testy WASM, rzeczywisty browser Worker,
pełny zestaw testów Studio oraz produkcyjny build.

## 14. Testy wymagane przed owner proof

### 14.1. Parser/writer safety

- każdy count/pointer ma checked arithmetic;
- nie ma alokacji przed preflightem count/size;
- cycle i overlap w AABB są błędami;
- unknown SET fields są zachowane albo jawnie raportowane;
- nieobsługiwany tile feature jest błędem profilu, nie cichym pominięciem.

### 14.2. Semantyka

- MDL classification jest `2`;
- MDL zawiera co najmniej root, render mesh i AABB mesh;
- AABB leaf faces są pełnym zbiorem collision faces;
- WOK jest ASCII i ma ten sam resref co MDL;
- WOK zawiera `Walk=1`;
- WOK ma dokładny footprint `10 x 10`;
- SET `Tile0.Model` wiąże oba zasoby;
- ARE `Tile_ID=0` rozwiązuje się do `Tile0`;
- cztery wpisy ARE odpowiadają Area `2 x 2`.

### 14.3. Determinizm

To samo wejście i opcje muszą dać identyczne:

- MDL bytes i SHA-256;
- WOK bytes i SHA-256;
- SET bytes i SHA-256;
- HAK bytes i SHA-256;
- MOD bytes i SHA-256;
- raport/readback.

## 15. Definition of Done

### Offline done

- wszystkie checkboxy T1-T7 są zamknięte;
- własne readbacki przechodzą;
- testy negatywne przechodzą;
- kandydat jest zamrożony i gotowy do przekazania właścicielowi.

Status: `ready_for_owner_proof`.

### Toolset done

Właściciel potwierdzi dla exact hash lineage:

- brak błędu SET/group;
- Area otwiera się;
- tile renderuje się;
- orientacja `0` jest poprawna;
- selection/bounds nie są zerowe;
- nie ma brakujących tekstur.

### NWN done

Właściciel potwierdzi dla tego samego lineage:

- tile renderuje się;
- gracz stoi na prawidłowej wysokości;
- podłoga jest przechodnia;
- seam jest przechodni;
- powierzchnie nieprzechodnie blokują wyłącznie tam, gdzie zaprojektowano.

Wymagane osie wyniku:

```yaml
toolset:
  modelVisibility: visible | not_visible | not_tested
  proofCompleteness: verified | failed | missing
nwn:
  modelVisibility: visible | not_visible | not_tested
  proofCompleteness: verified | failed | missing
navigation:
  spawnWalkable: verified | failed | missing
  seamWalkable: verified | failed | missing
  unintendedBlocking: absent | present | not_tested
```

## 16. Ryzyka

| Ryzyko | Skutek | Gate |
|---|---|---|
| skopiowanie profilu placeable | zła classification, brak AABB | `TileStaticV1` i semantic readback |
| WOK pod resrefem `WalkMesh` | zasób nie zostanie znaleziony | resolver wymaga `WOK.resref == Model` |
| użycie PWK surface `7` | cała podłoga nieprzechodnia | profil wymaga `Walk=1` |
| użycie render mesh jako WOK | koszt pathfindingu, złe kolizje | osobny navigation IR i limity |
| brak AABB node w MDL | niezgodny profil tile | profile validation error |
| zły footprint/origin | luki i nakładanie | bounds/seam tests `-5..5` |
| tylko `1 x 1` proof | brak testu seam | finalny fixture `2 x 2` |
| minimalny SET odrzucony przez Toolset | lane failure | owner verdict, bez cichej nowej iteracji |
| brak `ImageMap2D` | słaby picker/minimap UX | osobny authoring completeness gate |
| high-poly mesh > limit streamu | writer overflow | `21 845` triangles per stream |

## 17. Decyzje trwałe

```yaml
tile_pipeline_audit:
  status: "READY_FOR_OWNER_PROOF"
  shared_binary_mdl_pipeline: true
  separate_tile_mdl_parser: false
  resource_types:
    mdl: 2002
    set: 2013
    wok: 2016
    tga: 3
  tile_size_xy: 10
  model_local_xy_bounds_v1: [-5, 5]
  wok_format: "ASCII MAXWALKMESH"
  wok_resref_source: "SET.TileN.Model"
  set_walkmesh_is_wok_resref: false
  first_profile:
    id: "TileStaticV1"
    visual_triangle_target: 20000
    navigation_mesh: "separate low-poly geometry"
    requires_binary_mdl_aabb: true
    requires_walkable_surface: true
    area_size: [2, 2]
    height: 0
  current_p0_blockers: []
  proof_owner: "human owner"
  agent_completion_boundary: "ready_for_owner_proof"
```

## 18. Najbliższa bezpieczna kolejność prac

1. Test-first binary AABB reader na syntetycznym fixture oraz env-gated retail
   witness.
2. Wspólny deterministyczny `AabbTreeV1`.
3. `TileStaticV1` z prostym AABB node.
4. Generalizacja wspólnej gramatyki walkmesh i ASCII WOK.
5. Minimalny SET parser/writer oraz resolver.
6. HAK/MOD `2 x 2` z płaskim tile.
7. WASM/Studio lane.
8. Zamrożenie exact candidate i handoff do owner proof.

Nie należy rozpoczynać od pełnych groups, terrain transitions ani tile doors.
Najpierw trzeba udowodnić jeden widoczny i przechodni kafel przez ten sam
wspólny pipeline modelu.
