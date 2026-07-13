# M6 - GFF V3.2 i generated module proof contract

Data: 2026-07-13 | Autor: Codex | Status: M6A_GFF_CORE_DONE_M6B_PRESET_NEXT

## 1. Zakres i granice etapu

M6 buduje wlasny generated-only modul NWN EE i dowodzi go w Aurora Toolset oraz
w grze. Etap jest podzielony na kolejne slice'y:

1. M6A: binary GFF V3.2 reader/writer/own-readback;
2. M6B: typed UTC/IFO/ARE/GIT/GIC oraz ich cross-resource invariants;
3. M6C: deterministic MOD V1.0 assembler/readback i plikowy proof packet;
4. M6D: rzeczywisty Toolset/game/animation proof.

M6A jest gotowy do implementacji. M6B nie zaczyna sie od sparse dwupolowego UTC
ani GIT zawierajacego tylko resref. Exact typed baseline musi zostac zamrozony
pole po polu. Live acceptance nie jest zastepowana przez own-readback.

## 2. Evidence i clean-room boundary

Primary read-only evidence:

- `C:\Projects\New Folder\export\decompiled_all.c`;
- official `Bioware_Aurora_GFF_Format.pdf`, strony 3 i 5-14;
- official `Bioware_Aurora_CommonGFFStructs.pdf`;
- official `Bioware_Aurora_Creature_Format.pdf`, w szczegolnosci strony 3 i 8;
- official `Bioware_Aurora_IFO_Format.pdf`, strony 1-4;
- official `Bioware_Aurora_AreaFile_Format.pdf`, strony 1 i 7-9;
- official `Bioware_Aurora_ERF_Format.pdf`, strony 1-4;
- lokalne UTC/IFO/ARE/GIT/MOD czytane in-place i identyfikowane tylko hashami.

Secondary read-only implementations nie sa oracle ani code source. W
szczegolnosci nie kopiujemy Radoub: lokalny writer ma bledny struct ID `0` dla
`Mod_HakList`, bledny typ `INT` dla `Tile_AnimLoop*` oraz bledny typ string dla
`Mod_ID`.

Do repo nie trafia retail/CEP payload, realny modul ani extracted GFF. CI i
publiczne testy korzystaja wylacznie z minimalnych generated fixtures.

## 3. Exact GFF V3.2 layout

Wszystkie liczby sa little-endian. Nie ma alignment padding. Header ma exact 56
bajtow:

| Offset | Pole | Reprezentacja |
|---:|---|---|
| `0x00` | FileType | 4 bytes, np. `UTC ` |
| `0x04` | FileVersion | exact `V3.2` |
| `0x08` | StructOffset | u32 |
| `0x0c` | StructCount | u32 |
| `0x10` | FieldOffset | u32 |
| `0x14` | FieldCount | u32 |
| `0x18` | LabelOffset | u32 |
| `0x1c` | LabelCount | u32 |
| `0x20` | FieldDataOffset | u32 |
| `0x24` | FieldDataCount | u32 bytes |
| `0x28` | FieldIndicesOffset | u32 |
| `0x2c` | FieldIndicesCount | u32 bytes |
| `0x30` | ListIndicesOffset | u32 |
| `0x34` | ListIndicesCount | u32 bytes |

Exact contiguous chain i EOF:

```text
Header[56]
StructArray[12 * StructCount]
FieldArray[12 * FieldCount]
LabelArray[16 * LabelCount]
FieldData[FieldDataCount]
FieldIndices[FieldIndicesCount]
ListIndices[ListIndicesCount]
EOF
```

Aurora reader wymaga tego lancucha i exact EOF w `decompiled_all.c:964219-964356`.
Aurora writer wylicza te sekcje w `decompiled_all.c:969377-969426` oraz finalny
rozmiar w `decompiled_all.c:969644-969655`.

### 3.1 Struct entry

Kazdy struct ma exact 12 bajtow:

```text
u32 struct_id
u32 data_or_offset
u32 field_count
```

- index `0` zawsze istnieje i ma `struct_id=0xffffffff`;
- `field_count=0`: canonical writer emituje `data_or_offset=0`;
- `field_count=1`: `data_or_offset` jest indeksem Field Array;
- `field_count>1`: `data_or_offset` jest byte offsetem do `field_count` indeksow
  u32 w Field Indices.

### 3.2 Field entry i type IDs

Kazde field ma exact 12 bajtow: `u32 type`, `u32 label_index`,
`u32 data_or_offset`.

| ID | Typ | Storage |
|---:|---|---|
| 0 | BYTE | inline low u8 |
| 1 | CHAR | inline low i8 |
| 2 | WORD | inline low u16 |
| 3 | SHORT | inline low i16 |
| 4 | DWORD | inline u32 |
| 5 | INT | inline i32 |
| 6 | DWORD64 | FieldData, 8 bytes |
| 7 | INT64 | FieldData, 8 bytes |
| 8 | FLOAT | inline IEEE-754 f32 bits |
| 9 | DOUBLE | FieldData, IEEE-754 f64 bits |
| 10 | CExoString | FieldData |
| 11 | CResRef | FieldData |
| 12 | CExoLocString | FieldData |
| 13 | VOID | FieldData |
| 14 | Struct | Struct Array index |
| 15 | List | List Indices byte offset |

Direct Aurora writer anchors dla typow znajduja sie w
`decompiled_all.c:968691-969144`; Struct/List w `968557-968686`.

### 3.3 Labels i complex payloads

Label ma exact 16 bajtow, ASCII, NUL padded, bez terminatora dla dlugosci 16.
Writer nie truncuje. Pusty label, embedded NUL, non-ASCII i `>16` sa fatal.
Labels sa globalnie deduplikowane first-use order; labels w jednym struct sa
unikalne.

FieldData nie ma paddingu:

- CExoString: `u32 byte_length + bytes`, bez NUL;
- CResRef: `u8 byte_length + bytes`, `0..=16`, bez NUL; empty jest legalne
  dla opcjonalnych skryptow/Conversation; generated nonempty values musza byc
  lowercase `[a-z0-9_]+`, writer nie normalizuje ukradkiem i odrzuca uppercase;
- CExoLocString: `u32 total_size` (nie liczy tego DWORD), `u32 string_ref`,
  `u32 substring_count`, potem kazdy substring jako `i32 string_id + i32
  byte_length + bytes`; `string_id=2*language_id+gender`;
- VOID: `u32 byte_length + bytes`;
- DWORD64/INT64/DOUBLE: exact 8 bajtow;
- list record: `u32 count + count*u32 struct_index`.

Generated M6 content uzywa printable ASCII. Core reader zachowuje string bytes i
nie deklaruje uniwersalnego UTF-8, bo format nie zamraza jednego kodowania.

Errata: GFF PDF na stronie 14 omylkowo nazywa storage List polem Field Indices;
header, strony 8-10 i Aurora `decompiled_all.c:967747-967775` potwierdzaja List
Indices. Diagram Field Array na stronie 7 rowniez blednie nazywa Field 0
top-level structem; top-level jest Struct 0.

## 4. M6A public core contract

Core udostepnia wersjonowane typy i nie przyjmuje dowolnego JSON w samym
writerze:

```rust
pub enum GffFileTypeV1 { Utc, Ifo, Are, Git, Gic }

pub struct GffDocumentV1 {
    pub schema_version: u32, // exact 1
    pub file_type: GffFileTypeV1,
    pub root: GffStructV1,   // exact struct_id 0xffffffff
}

pub struct GffStructV1 {
    pub struct_id: u32,
    pub fields: Vec<GffFieldV1>,
}

pub struct GffFieldV1 {
    pub label: String,
    pub value: GffValueV1,
}

pub struct GffLocStringV1 {
    pub string_ref: u32, // 0xffffffff = no TLK entry
    pub substrings: Vec<GffLocSubstringV1>,
}

pub struct GffLocSubstringV1 {
    pub string_id: i32,  // raw 2*language_id+gender identity
    pub bytes: Vec<u8>,
}

pub enum GffValueV1 {
    Byte(u8), Char(i8), Word(u16), Short(i16), Dword(u32), Int(i32),
    Dword64(u64), Int64(i64), Float(f32), Double(f64),
    String(Vec<u8>), ResRef(String), LocString(GffLocStringV1), Void(Vec<u8>),
    Struct(GffStructV1), List(Vec<GffStructV1>),
}

pub fn write_gff_v32(
    document: &GffDocumentV1,
    options: &GffWriterOptionsV1,
) -> Result<GffArtifactV1, GffErrorV1>;

pub fn read_gff_v32(
    bytes: &[u8],
    limits: &GffLimitsV1,
) -> Result<GffDocumentV1, GffErrorV1>;
```

`Float` i `Double` odrzucaja non-finite generated values. Substrings LocString
zachowuja input order, generated `string_id` musi byc `>=0` i jest unikalne w
jednym LocString, empty bytes sa legalne, a semantic equality zachowuje raw
`string_id`, bytes i order. Reader odrzuca ujemny `string_id` jako
`M6-GFF-VALUE-INVALID`, poniewaz nie moze on reprezentowac
`2*language_id+gender`.

Tree IR nie dopuszcza aliasow ani cykli. Reader przed rekonstrukcja wykonuje
indeksowy canonical-tree preflight: root nie ma parenta, kazdy non-root struct
ma exact jednego ownera przez Struct albo List, wszystkie structy sa osiagalne
z root, reuse/DAG i cycle sa fatal `M6-GFF-LAYOUT-INVALID`. Dopiero po tej
bramce reader buduje tree, wiec nie klonuje aliasowanego grafu i nie moze rosnac
wykladniczo. Rekonstrukcja jest dodatkowo depth-bounded i nie panikuje.

Canonical preflight obejmuje rowniez fizyczne ownership/coverage przed
alokacja tree:

- kazdy Field Array index ma exact jednego ownera w exact jednym Struct; kazdy
  Field entry jest osiagalny; reuse i unreachable field sa layout fatal;
- Label Array values sa globalnie unikalne, kazdy label entry ma co najmniej
  jedna referencje; wspoldzielenie jednego label przez wiele fields jest legalne;
- out-of-line FieldData payload ranges sa disjoint, nie aliasuja sie i w global
  field encounter order pokrywaja exact caly FieldData bez gaps/trailing;
- FieldIndices records w struct index order oraz ListIndices records w global
  field encounter order pokrywaja swoje sekcje exact bez unused bytes.

Naruszenie daje `M6-GFF-LAYOUT-INVALID` przed payload copy lub tree build.

Canonical writer phases:

1. przydziel Struct indices pre-order: root=0, potem fields input order, Struct
   child przed dalszym siblingiem, List elements w input order;
2. przejdz Struct Array w rosnacym struct index i emituj jego Fields w input
   order; to jest jedyny global field encounter order;
3. w tym samym encounter order przydzielaj labels first-use oraz appenduj
   complex FieldData;
4. po zaplanowaniu Fields emituj FieldIndices tylko dla structow z
   `field_count>1`, w struct index order;
5. emituj ListIndices jako kompletne records w global field encounter order;
6. zapisz exact contiguous section chain i EOF.

Report zawiera co najmniej FileType, counts/offsets, byteLength, outputSha256,
maxDepth oraz semantic readback status. Writer przed success uruchamia exact
layout verifier i public own reader, po czym porownuje cale typed tree.

## 5. Limity i taksonomia

Hard/default limity M6A (wszystkie inclusive, equality PASS):

```yaml
maxGffBytes: 67108864
maxStructs: 65536
maxFields: 262144
maxLabels: 65536
maxFieldsPerStruct: 65536
maxListElements: 65536
maxDepth: 64
maxStringBytes: 1024
maxLocStringBytes: 1048576
maxVoidBytes: 16777216
maxDiagnostics: 2048
```

Limity sa strict, nonzero, nie sa clampowane i sa sprawdzane przed alokacja
zalezna od inputu. `maxStringBytes` dotyczy kazdego CExoString i kazdego
LocString substring; `maxLocStringBytes` dotyczy calego encoded LocString;
`maxListElements` jest per-list; root ma depth 0. Writer i reader uzywaja tych
samych nazw, a typed generated schema moze miec dodatkowy, mniejszy limit.
Wszystkie mnozenia/sumy/offsety sa checked i musza miescic sie w u32 oraz
platform usize.

Stable taxonomy:

- `M6-GFF-OPTIONS-INVALID`;
- `M6-GFF-LIMIT-EXCEEDED`;
- `M6-GFF-HEADER-INVALID`;
- `M6-GFF-FILE-TYPE-UNSUPPORTED`;
- `M6-GFF-VERSION-UNSUPPORTED`;
- `M6-GFF-LAYOUT-INVALID`;
- `M6-GFF-INDEX-OOB`;
- `M6-GFF-TYPE-UNSUPPORTED`;
- `M6-GFF-LABEL-INVALID`;
- `M6-GFF-DUPLICATE-LABEL`;
- `M6-GFF-VALUE-INVALID`;
- `M6-GFF-DEPTH-LIMIT-EXCEEDED`;
- `M6-GFF-ALLOCATION-FAILED`;
- `M6-GFF-READBACK-FAILED`;
- `M6-GFF-SEMANTIC-DIFF`.

Stable precedence dla publicznego readera:

1. invalid limits;
2. input `maxGffBytes`;
3. header length;
4. version, potem FileType;
5. exact section layout/count arithmetic/EOF;
6. struct/field/label hard counts i type IDs;
7. raw index/range/payload bounds;
8. duplicate/invalid labels i value encoding;
9. canonical ownership: root/parent/reuse/reachability/cycle;
10. depth/per-struct/per-list/string/LocString/VOID semantic limits;
11. allocation failure;
12. readback/semantic diff (writer only).

Writer precedence: options/limits, schema/root, total struct/depth plan, labels
i typed values w canonical encounter order, count/list/string limits, checked
layout, allocation, exact verifier, public readback, semantic comparison.

Error JSON ma `schemaVersion,code,severity,path,message`, severity `FATAL`.
Reader offset moze byc zapisany w `path` jako stabilny logical path; publiczne
typed adaptery nie przepisuja core classification.

## 6. M6A TDD gates

- exact empty-root UTC i one-field fixtures z frozen bytes/report JSON;
- kazdy inline type i kazdy complex type;
- struct 0/1/>1 fields oraz FieldIndices transition;
- empty/nonempty list, nested structs, list order i duplicate global labels;
- exact header/section offsets/counts i EOF;
- own reader byte/semantic readback;
- every truncated prefix, mutated offset/count/index/type/length/root/version/
  FileType, duplicate label i trailing byte;
- physical canonical-preflight fixtures: reused Field index, unreachable Field,
  unused/duplicate Label entry, reused/unreachable/cyclic Struct, FieldData
  alias/overlap/gap oraz bytes niepokryte wewnatrz section, unused bytes/records
  w FieldIndices i ListIndices; kazdy daje exact `M6-GFF-LAYOUT-INVALID` przed
  payload copy/tree build;
- LocString negative/duplicate `string_id`, truncated substring i declared
  total-size mismatch; negative/duplicate ID daje `M6-GFF-VALUE-INVALID` po raw
  range validation, przed tree build;
- exact hard limit boundaries, checked arithmetic seams i forced allocation
  failure seams;
- maxDepth boundary i no-panic arbitrary deterministic byte corpus;
- HAK/M5 frozen bytes i gates pozostaja bez zmian.

M6A nie uzywa retail bytes jako golden fixture. Env-gated packets moga jedynie
potwierdzic, ze reader rozpoznaje inventory i typy bez kopiowania payloadu.

## 7. Typed resource contract - zamkniete relacje

Potwierdzone minimum powiazan, niezaleznie od finalnego baseline field list:

```yaml
UTC:
  file_type: "UTC "
  root_struct_id: 0xffffffff
  Appearance_Type: WORD
  TemplateResRef: CResRef
IFO:
  file_type: "IFO "
  Mod_HakList: "List -> StructID 8 -> Mod_Hak CExoString"
  Mod_Area_list: "List -> StructID 6 -> Area_Name CResRef"
  Mod_Entry_Area: CResRef
  Mod_ID: "VOID, generated as 16 zero bytes"
ARE:
  file_type: "ARE "
  Tile_List_element_struct_id: 1
  Tile_AnimLoop1_2_3: BYTE
GIT:
  file_type: "GIT "
  AreaProperties_struct_id: 100
  Creature_List_label: "Creature List"
  Creature_List_element_struct_id: 4
  placement: "X/Y/ZPosition + X/YOrientation, all FLOAT"
GIC:
  file_type: "GIC "
  invariant: "same named lists and exact 1:1 element counts/order as GIT"
```

Cross-resource invariants:

- `UTC.TemplateResRef == UTC resource resref == GIT instance.TemplateResRef`;
- UTC i embedded GIT instance maja ten sam `Appearance_Type`, rowny physical
  appended appearance.2da index;
- `IFO.Mod_Entry_Area == IFO.Mod_Area_list[0].Area_Name == ARE/GIT/GIC resref`;
- GIT zawiera pelny Creature Instance Struct, nie tylko reference do UTC;
- GIC ma odpowiadajacy 1:1 comment element dla kazdej GIT instance list entry;
- pierwszy proof ma exact jeden HAK w `Mod_HakList`.

`Tile_AnimLoop*` jest swiadomym Aurora First override: official Area PDF podaje
`INT`, lecz dekompilacja czyta BYTE w `decompiled_all.c:255412-255419` i zapisuje
type-0 BYTE w `255950-255952`; syntetyczny BYTE readback/runtime regression jest
obowiazkowy. Analogicznie official IFO PDF uzywa pisowni `Mod_Area_List` i
`AreaName`, podczas gdy Aurora zapisuje exact `Mod_Area_list` i `Area_Name` w
`decompiled_all.c:264456-264463`; generated schema stosuje wartosci Aurory.

Sparse UTC z tylko dwoma polami oraz reference-only GIT sa `OPEN_RUNTIME`, nie
sa implementacyjnym baseline. Exact ordered schemas UTC67, IFO55, ARE43/tile10,
GIT root/AreaProperties/creature70 i GIC sa zamrozone w
`documentation/m6-typed-resource-manifests-codex.md`. Schema jest `READY`, ale
generated gameplay preset pozostaje `NOT_READY`, dopoki nie ma zamrozonego
Feat/Class/Equip zestawu oraz legalnego synthetic Tileset/Tile_ID.

## 8. MOD V1.0 contract

MOD uzywa ERF V1.0 layoutu 160/24/8, ale domenowy writer nie jest podmiana
sygnatury HAK. `ErfFileType::Mod` i neutralny internal ERF emitter sa dozwolone,
jezeli frozen HAK bytes pozostaja identyczne.

Pierwszy deterministic MOD:

- signature `MOD `, version `V1.0`;
- `LanguageCount=0`, `LocalizedStringSize=0`;
- jawne `buildYearSince1900` i `buildDaySinceJan1`, bez zegara systemowego;
- `DescriptionStrRef=0xffffffff`, reserved zero;
- domain order: `module/2014`, area ARE/2012, area GIC/2046, area GIT/2023,
  creature UTC/2027;
- HAK pozostaje osobnym plikiem i jest wskazany z IFO;
- own `ErfArchive` readback, payload SHA i cross-resource semantic readback.

Kazdy obszar wymaga tego samego resref dla ARE, GIT i GIC. GIC nie jest
opcjonalny dla Toolset-compatible module structure. UTC trafia do MOD, aby nie
zmieniac zamrozonego trzyzasobowego HAK M5.

## 9. Open przed M6B/M6C/M6D

- exact generated defaults/kardynalnosci dla Feat/Class/Equip w non-caster
  UTC/GIT preset; ordered schemas sa juz locked w typed manifest supplement;
- synthetic, legal `Tileset + Tile_ID` proof fixture bez kopiowania retail;
- exact GIC comment element fields;
- ewentualne Toolset requirements dla palette/faction/support resources;
- live akceptacja pustej ERF description, build fields i domain resource order;
- Toolset/game/visual/deformation/animation acceptance.

Brak tych odpowiedzi nie blokuje M6A GFF core. Blokuje uznanie typed module
assemblera za contract-locked i blokuje `M6 DONE`.
