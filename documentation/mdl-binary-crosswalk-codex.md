# Binary MDL crosswalk

Data: 2026-07-10 | Status: AKTYWNY KONTRAKT WIEDZY DLA M1/M4

## 1. Werdykt

Minimalny kierunek writera jest ustalony. Nie oznacza to jeszcze zgodnosci runtime: ta wymaga wlasnego readbacku oraz proofu w NWN EE.

```text
resource type 2002 = 12-byte file header + core block + volatile/MDX block
core pointer       = 12 + offset
raw pointer        = 12 + raw_data_offset + offset
```

## 2. Kolejnosc autorytetu

1. `C:\Projects\New Folder\export\decompiled_all.c` - Aurora First: rodziny runtime `CResMDL`, `MdlNode*` i typ zasobu 2002.
2. Lokalny binarny zasob `c_kocrachn` w `cep3_core1.hak` - fakty o granicach blokow i headerze.
3. `xoreos-docs/templates/NWN1MDL.bt`, `rollnw/MdlBinaryParser.*`, `borealis_nwn_mdl` - read-only cross-check layoutu.
4. Wlasny reader i syntetyczne fixture - test kontraktu, nie zewnetrzny oracle.
5. Toolset/gra - finalna akceptacja.

`borealis_nwn_mdl` jest GPL-3 (`COPYING`) i pozostaje reference-only. `rollnw` jest MIT, ale takze nie staje sie dependency bez osobnej decyzji. Nie kopiujemy implementacji writera.

## 3. Potwierdzony wspolny rdzen layoutu

| Struktura | Rozmiar | Istotne pola |
|---|---:|---|
| File header | `0x0c` | `zero`, `raw_data_offset`, `raw_data_size` |
| Array header | `0x0c` | `offset`, `count`, `allocated` |
| Geometry header | `0x70` | routines/padding, `name[64]`, root offset, node count, arrays, geometry type |
| Model header | `0xe8` | geometry, classification, fog, animations, bounds, radius, animation scale, `supermodel[64]` |
| Animation header | `0xc4` | geometry, length, transition, `animroot[64]`, events |
| Animation event | `0x24` | time + `name[32]` |
| Controller key | `0x0c` | type, rows, time/data offsets, columns |
| Node header | `0x70` | node name, parent/children, controller arrays, content flags |
| Face | `0x20` | plane, surface, adjacency, three vertex indices |
| Mesh header | `0x270` w legacy cross-checku | faces/material/texture fields oraz raw offsets |

Kotwice cross-checku:

- `C:\Projects\Claude\rollnw\lib\nw\model\MdlBinaryParser.hpp:20-125`;
- `C:\Projects\Claude\rollnw\lib\nw\model\MdlBinaryParser.cpp:80-220`;
- `C:\Projects\Claude\borealis_nwn_mdl\src\internal\MdlStructures.hpp:27-136`;
- `C:\Projects\Claude\borealis_nwn_mdl\src\BinaryWriter.cpp:919-984`.

## 4. Core kontra volatile

Do `core` naleza co najmniej:

- model/animation/node headers;
- tablice wskaznikow dzieci i animacji;
- controller keys i controller float data;
- faces;
- inverse bind pose arrays;
- eventy animacji.

Do `volatile/MDX` dla profilu A naleza tablice bezposrednio uzywane przez mesh:

- vertices;
- UV sets;
- normals;
- vertex indices;
- EE tangents i handedness, jezeli sa emitowane;
- colors, jezeli sa emitowane;
- skin weights i bone references dla skin mesh.

Kolejnosc blokow musi byc deterministyczna. Nie przyjmujemy kodu z zewnetrznego writera jako oracle; jego kolejnosc jest jedynie kandydatem do potwierdzenia przez wlasny readback.

## 5. Profil writera A

Pierwszy writer ma obslugiwac najmniejszy jawny podzbior:

```yaml
profile_A_required:
  - "file/model/geometry headers"
  - "dummy hierarchy"
  - "trimesh"
  - "skin mesh only after M1B resolves its exact header variant"
  - "faces, vertices, one UV set, normals, indices"
  - "diffuse texture resref"
  - "position and orientation controllers"
  - "model animations, animroot, transition time and events"
  - "appended volatile/MDX block"
profile_A_deferred:
  - "light, emitter, reference, animmesh, danglymesh, aabb/trigger"
  - "multiple UV sets beyond observed need"
  - "MTR/TXI extensions"
  - "round-trip preservation of unknown runtime pointers"
```

Wszystkie runtime routine pointers oraz pola niepotwierdzone maja byc zero/omitted tylko wtedy, gdy syntetyczny readback i NWN EE proof to zaakceptuja. Nie deklarujemy tego z gory jako faktu engine'u.

## 6. Jawny konflikt: skin header

Lokalne referencje nie sa zgodne:

- `xoreos-docs/templates/NWN1MDL.bt:510` i `rollnw/MdlBinaryParser.hpp:244-259` maja 17 wpisow mapowania kosci i rozmiar `0x2d4`;
- aktualny `borealis_nwn_mdl/MdlStructures.hpp:232-247` deklaruje 64 wpisy i opisuje starszy wariant 17 jako niepelny.

To jest nazwany problem `GB-001-SKIN`, a nie powod do zgadywania. Zamkniecie:

1. M1B raportuje dla lokalnego corpus offset nastepnego obiektu po kazdym skin node i oba kandydackie rozmiary;
2. fixture boundary test odrzuca odczyt poza core;
3. writer wybiera wariant per jawny profil formatu, nie globalna magiczna stala;
4. oba warianty, jesli realnie wystepuja, maja oddzielne testy.

## 7. TDD i warunek zamkniecia GB-001

```yaml
writer_gates:
  - "two identical inputs produce byte-identical output"
  - "every addition/multiplication and pointer range is checked"
  - "reader consumes exact core and volatile ranges"
  - "semantic readback matches the input IR"
  - "unsupported node/controller emits a stable diagnostic, never silent loss"
  - "generated type-2002 payload is readable after HAK round-trip"
  - "Toolset and game load the generated asset"
```

GB-001 ma status `DIRECTION_DEFINED_EVIDENCE_OPEN`: mozna projektowac IR i testy, ale M4 nie jest `DONE`, dopoki finalny proof nie przejdzie.
