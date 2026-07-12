# M1B - suplement kontraktu deep binary MDL readera

Data: 2026-07-11 | Autor: Codex | Status: AKTYWNY SUPLEMENT M1B

## 1. Cel i autorytet

Ten suplement zamyka zakres implementacyjny M1B bez rozszerzania go na writer. Kolejnosc autorytetu pozostaje: dekompilacja Aurory, canonical local binary read-only, aktywne kontrakty projektu, a nastepnie zewnetrzne parsery jako cross-check.

Zewnetrzne implementacje nie sa dependency ani fixture source. Skan mirroru moze wyznaczyc hipoteze i kandydatow, ale canonical P-REF musi zostac ponowiony in-place przez wlasny locator M1C.

## 2. Pointer bases i runtime fields

```text
file header        = 0x0c
core absolute      = 0x0c + core pointer
raw/MDX absolute   = 0x0c + raw_data_offset + raw pointer
raw null           = 0xffffffff
```

Node fields `geometry_ptr @ 0x40` i `parent_ptr @ 0x44` sa runtime/ignore values, nie serialized core pointerami. Parent relationship wynika z children array. Pierwszy realny smoke `c_kocrachn` wykazal runtime `parent_ptr = 7733349`; syntetyczne zalozenie M1A zostalo skorygowane.

`declared node count` jest bezpiecznym budzetem, nie wymogiem rownosci z liczba node osiagalnych z model root. Dla canonical R1 dokumentacja oczekuje `66` deklarowanych i `38` osiagalnych z base root; animation roots sa osobnymi drzewami.

## 3. Potwierdzone rozmiary i istotne offsety

| Struktura | Rozmiar | Istotne pola |
|---|---:|---|
| FileHeader | `0x0c` | id `0x00`, core size `0x04`, raw size `0x08` |
| GeometryHeader | `0x70` | name `0x08`, root `0x48`, node budget `0x4c`, type `0x6c` |
| ModelHeader | `0xe8` | classification `0x72`, fog `0x73`, animations `0x78`, bounds `0x88/0x94`, radius `0xa0`, animationScale `0xa4`, supermodel `0xa8` |
| AnimationHeader | `0xc4` | length `0x70`, transition `0x74`, animroot `0x78`, events `0xb8` |
| AnimationEvent | `0x24` | time `0x00`, name `0x04` |
| NodeHeader | `0x70` | inherit `0x18`, number `0x1c`, name `0x20`, children `0x48`, controller keys `0x54`, controller data `0x60`, flags `0x6c` |
| ControllerKey | `0x0c` | type `i32/u32`; rows/time/data `u16`; byte `+0x0a`: low nibble columns, high nibble flags; byte `+0x0b`: padding/opaque |
| Face | `0x20` | plane, surface, adjacency, vertex indices |
| MeshNode | `0x270` | faces `0x78`, textures `0xe8..0x1a8`, raw vertices `0x22c`, counts `0x230/0x232`, UV `0x234`, normals `0x244` |

Node flags sa addytywne:

```text
header 0x001, light 0x002, emitter 0x004, camera 0x008,
reference 0x010, mesh 0x020, skin 0x040, animmesh 0x080,
dangly 0x100, aabb 0x200
```

M1B semantycznie wspiera dummy/header, trimesh i skin common prefix. Pozostale rodziny musza miec stabilny `M2A-MDL-UNSUPPORTED-NODE-FAMILY` z zachowanym inventory, bez cichej utraty.

## 4. REQUIRED

- model metadata, base root i wszystkie animation roots;
- controller keys/data z checked indices; common types position `8`, orientation `20`, scale `36`, self-illumination `100` i alpha `128`;
- animation length, transition, animroot i events;
- faces, vertices, UV0, normals i texture resrefs dla trimesh;
- walidacja zakresow pozostalych raw mesh pointerow bez deklarowania pelnej semantyki UV1-3/colors/tangents;
- skin raw weights `f32[4]`, bone refs `u16[4]`, common bind arrays i jawny wariant boundary mappingu; szerokosc `17/64` nie jest capacity limitem `map count` ani bone refs;
- synthetic fixture dla kazdej wspieranej sekcji oraz obu skin variants;
- stabilne diagnostics dla nieznanych flag/controller types;
- czysty-bytes P-REF packet: identity, SHA-256, reader/schema, capabilities i invariants;
- env-gated local smoke tylko w testach, bez sciezek prywatnych i bez payloadow w Git.

### 4.1. Minimalna macierz testow M1B

Happy fixtures REQUIRED:

- metadata modelu: classification, fog, child model count, bounds, radius, animation scale, supermodel oraz jawne core/raw ranges;
- common controllers: position `8`, orientation `20`, scale `36`, self-illumination `100` i alpha `128`, z co najmniej dwoma rows oraz sprawdzonymi times/values;
- trimesh z face w core oraz vertices, UV0 i normals w raw/MDX;
- osobne skiny `legacy17` i `extended64`, z takim samym `map count`, aby classifier nie mogl zalezec od tej wartosci;
- co najmniej dwie animacje, kazda z osobnym declared/reachable node tree, length, transition, animroot, events i controllerem na animation root;
- fixture laczona oraz deterministyczna serializacja deep reportu przez native Rust i publiczny adapter WASM.

Negative matrix REQUIRED:

- core OOB osobno dla animations pointer array, animation header/events, faces, controller keys/data oraz skin node-to-bone/q/t/constants arrays;
- raw OOB osobno dla vertices, UV0, normals, weights i bone refs; pozostale raw pointers musza byc `0xffffffff` albo wskazywac do raw range;
- `used > allocated` dla kazdego obslugiwanego `ArrayHeader`;
- `declared < reachable` jako invalid oraz `declared > reachable` jako dozwolony budzet, osobno dla base root i animation roots;
- graniczne `u16` rows/time/data, high-bit values bez signed reinterpretation, niepoprawny packed columns/flags layout, time/data index OOB i nieznany controller type zachowany w inventory;
- addytywne mesh/skin flags, kazda znana deferred family oraz prawdziwie nieznany node bit;
- skin header boundary `0x2d4/0x330`, classifier niezalezny od `map count` oraz brak pasujacego wariantu; nie wolno tworzyc testu capacity `16/17` lub `63/64` z samej szerokosci inline mappingu;
- cycle pod base root i pod animation root;
- truncation-no-panic dla kazdej deep fixture, nie tylko dla minimalnego R0.

### 4.2. Stabilna taksonomia M1B

Fatal errors:

- `M2A-MDL-HEADER-INVALID`;
- `M2A-MDL-POINTER-OOB`;
- `M2A-MDL-NODE-CYCLE`;
- `M2A-LIMIT-EXCEEDED`;
- `M2A-MDL-CONTROLLER-LAYOUT-INVALID`;
- `M2A-MDL-CONTROLLER-INDEX-OOB`;
- `M2A-MDL-SKIN-VARIANT-AMBIGUOUS`;
- `M2A-MDL-BONE-REF-OOB`.

Non-fatal structured diagnostics, z zachowanym inventory:

- `M2A-MDL-UNSUPPORTED-NODE-FAMILY`;
- `M2A-MDL-NODE-FLAGS-UNKNOWN`;
- `M2A-MDL-CONTROLLER-TYPE-UNKNOWN`.

Nowe M1B paths nie uzywaja ogolnego `M2A-MDL-UNSUPPORTED`, gdy mozna nazwac konkretna klase problemu.

## 5. Skin 17/64

Wspolny prefix skina zaczyna sie po MeshNode:

```text
0x270 weights ArrayHeader
0x27c raw weights pointer
0x280 raw bone refs pointer
0x284 node-to-bone map core pointer
0x288 map count
0x28c qBoneRefInv ArrayHeader
0x298 tBoneRefInv ArrayHeader
0x2a4 bone constants ArrayHeader
0x2b0 inline mapping
```

Profile:

```text
legacy17   size 0x2d4, i16[17] + spare i16
extended64 size 0x330, i16[64]
```

Classifier nie uzywa `map count`. Dla canonical evidence sprawdza, czy pierwszy nalezacy do skina core payload (`node-to-bone map`) zaczyna sie dokladnie przy `node + 0x2d4` albo `node + 0x330`, oraz waliduje wszystkie arrays/raw ranges. Dokladnie jeden z tych warunkow equality moze pasowac, poniewaz istnieje jeden pointer. Brak dopasowania daje `M2A-MDL-SKIN-VARIANT-AMBIGUOUS`. Dawne sformulowanie "oba lub zero" bylo nieprecyzyjne: przypadek "oba" nie jest konstruowalny dla jednego pointera porownywanego equality z dwoma roznymi offsetami i nie jest osobnym wymaganym testem.

`17` i `64` opisuja szerokosc inline header mappingu, nie capacity `map count`, bind arrays ani bone refs. Canonical bug evidence raportuje `c_kocrachn` jako extended64 z count `38` oraz `c_vampire_f` jako legacy17 z count `28`; parser fix i rozszerzony P-REF pozostaja `PENDING`. Dla `c_kocrachn` `0xffff` zaobserwowano w lanes o zerowej wadze i reader ma je zachowac bez aktywnego bone lookup. Szczegoly finalnej macierzy sa w `documentation/m1b-canonical-corpus-suplement-codex.md`.

## 6. DEFERRED i OPEN

DEFERRED: semantyka light/emitter/camera/reference/animmesh/dangly/AABB, UV1-3, colors, tangent rendering, opaque routines/tokens, writer i runtime acceptance.

OPEN:

- M4A Aurora First ustalilo orientation on-disk jako `XYZW` oraz high nibble `0x10` jako wybor sciezki Bezier; M1B reader nadal wymaga implementacyjnej korekty z signed `i16`/pelnego byte columns na `u16` i packed low/high nibble przed rozpoczeciem writera M4A;
- model bytes `0x70..0x75` poza classification/fog;
- emisja i pelny readback danych Bezier oraz pozostale nieznane controller layouts;
- skin fallback, gdy payload nie zaczyna sie bezposrednio po headerze;
- czy `0xffff` jest legalnym pustym bone ref przy zerowej wadze; do canonical Aurora/M1C evidence nie wolno wymyslac akceptacji ani odrzucenia tego sentinela;
- znaczenie opaque mesh tokens.

## 7. Stop conditions

Semantic decode danego payloadu zatrzymuje sie z diagnostyka, gdy zakres wychodzi poza core/raw, wystepuje overflow, `count > allocated`, cycle/reused offset o sprzecznym typie, nieznany node bit, niepoprawny controller index/layout, niejednoznaczny skin variant albo bone ref przekracza potwierdzona domene node-to-bone. Sama szerokosc headera `17/64` nie jest taka domena.

## 8. Canonical corpus identity i aktualna dostepnosc

Ponizsze identity sa zwiazane z canonical own-locator/own-reader P-REF dla R1/R3; R2 pozostaje opcjonalnym adapterem. Committable identity nie zawiera host paths:

| ID | Source class | Container identity | Resref | Type | SHA-256 observed | Status |
|---|---|---|---|---:|---|---|
| R1 | `named_hak` | `cep3_core1` | `c_kocrachn` | 2002 | `f16426310f826ae2ab15034ac979c65f812ee8bda0d13ee459bf2b293d7db270` | `CANONICAL_PACKET_PASS`; extended64/sentinel invariants PASS |
| R2 | `base_nwn` | `models_01.bif` | `c_horror` | 2002 | `2faf553a0665da200b232bd52d03c0e1d79b88959cabdbe840f35f16e5878c8e` | `OPTIONAL_NOT_RUN`; KEY/BIF adapter optional |
| R3a | `named_hak` | `cep3_core1` | `c_phod_horror_b` | 2002 | `62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a` | `CANONICAL_PACKET_PASS`; animation/event invariants PASS |
| R3b | `named_hak` | `cep3_core1` | `c_phod_horror_p` | 2002 | `09e43ee9493d2fe2bbf9cbeb44f24dcb999e5f38e651bdc79eefdd5e1f19722f` | `CANONICAL_PACKET_PASS`; animation/event invariants PASS |

Hash mismatch podczas canonical run nie jest automatycznie bledem parsera: najpierw trzeba nazwac wersje/pochodzenie lokalnego kontenera. Zadnego z tych payloadow nie wolno commitowac ani utrwalac jako fixture.

## 9. Kolejnosc M1B/M1C

Direct-file env nie zamyka corpus DoD: canonical R1/R3 sa subranges HAK, a kopie `aurora-web` sa zabronione jako test source. Wykonawcza kolejnosc:

1. M1B zaimplementowal deep reader, synthetic fixtures i pure-bytes P-REF contract;
2. synthetic/native/WASM checkpoint przeszedl przez `VERIFYING`, nigdy bezposrednio do `DONE`;
3. M1C dostarczyl zweryfikowany own HAK/ERF read-only locator i ma status `DONE`;
4. M1B ma status `VERIFYING`: canonical R1/R3/R4/R5/R6, skin boundary/sentinel, capability matrix i role invariants przechodza; finalny clean re-review pozostaje wymagany przed `DONE`;
5. M1B moze byc `DONE` dopiero po canonical packetach i evidence albo po jawnym, nazwanym statusie pozostalego problemu zgodnym z jego DoD; samo container inventory nie wystarcza;
6. R2 KEY/BIF adapter pozostaje opcjonalny, jesli R1/R3 daja wymagane rodziny.
