# M4A - suplement kontraktu self-contained animation writera

Data: 2026-07-13 | Autor: Codex | Status: M4A1_WRITER_VERIFIED_M4A2_NEXT

## 1. Cel, zakres i autorytet

Ten dokument zamyka kontrakt przed implementacja M4A. Kolejnosc autorytetu:

1. dekompilacja lokalnej Aurory;
2. canonical R3a/R3b odczytane in-place wlasnym locator/readerem;
3. zamrozone kontrakty M1B/M4 i clean-room rules;
4. decyzje produktowe wyraznie oznaczone jako structural choice;
5. NWN EE runtime proof w M6.

M4A jest self-contained. Nie kopiuje keyframes, szkieletu, eventow ani opaque
runtime bytes z retail/CEP. Canonical payload nie trafia do repo, fixture ani
generated test vector. `aurora-web` nie jest dependency ani oracle.

M4A sklada sie z dwoch wymaganych czesci:

- `M4A1`: wersjonowany `MdlAnimationSetV1`, binary writer, own readback,
  semantic diff, negative/mutation matrix i frozen zero-animation regression;
- `M4A2`: jawny mapper source GLB animation -> output rig/coordinate space ->
  `MdlAnimationSetV1`.

Samo M4A1 nie daje statusu `DONE`. M4A jest `DONE` dopiero po M4A2 i wszystkich
gate'ach strukturalnych. Toolset/game acceptance pozostaje `OPEN_M6`.

## 2. Aurora First anchors

| Fakt | Exact anchor w `decompiled_all.c` |
|---|---|
| serialized animation `0xc4 + events*0x24 + rootTree` | `871536-871545` |
| model animation ArrayDef `+0x78/+0x7c/+0x80` | `862902-862955`, `871551-871572` |
| own animation root i events `+0xb8` | `863021-863047` |
| length `+0x70`, transition `+0x74`, animroot `+0x78` | `864534-864541` |
| children, key ArrayDef, data ArrayDef | `863740-863779` |
| root pairing i recursive child-name matching | `889100-889146` |
| animation lookup po GeometryHeader name, supermodel/default fallback | `844151-844216` |
| key ma `0x0c`, rows/time/data sa `u16` | `843831-843852`, `882458-882566` |
| type 8/20/36 i columns 3/4/1 | `882852-882975` |
| low nibble columns, high nibble `0x10` Bezier | `882814-882826`, `844683-844739` |
| time selection oraz scalar/vector interpolation | `844653-844897` |
| position type 8 mnozone przez animationScale | `844795-844810` |
| orientation XYZW, shortest-path slerp bez jawnej normalizacji | `844904-844990`, `1020552-1020628` |
| transition jako blend-in/out seconds; zero immediate | `843901-843947`, `846138-846155`, `846199-846211` |
| play time, wrap/clamp, controller apply i event dispatch | `846113-846167` |
| event window forward/reverse/wrap | `844407-844458` |
| event parse/sort/copy, stride `0x24` | `886456-886582` |

## 3. Zamrozony binary layout

```text
AnimationHeader size = 0xc4

0x00..0x6f  GeometryHeader
0x08..0x47  clip name char[64]
0x48        animation root core offset u32
0x4c        declared node budget u32
0x50..0x67  opaque/empty geometry arrays in M4A1
0x68..0x6f  opaque runtime fields; zero in M4A1
0x70        length f32 seconds
0x74        transition_time f32 seconds
0x78..0xb7  animroot char[64]
0xb8        event pointer u32
0xbc        event used u32
0xc0        event allocated u32
```

```text
AnimationEvent size = 0x24
0x00 time f32 seconds
0x04 name char[32]
```

Model `+0x78/+0x7c/+0x80` jest ArrayDef `u32` core-offsetow AnimationHeader.
Puste array maja `0/0/0`. `used <= allocated`; M4A emituje `used=allocated`.

ControllerKey:

```text
size 0x0c
+0x00 type/runtime byte offset i32/u32
+0x04 row count u16
+0x06 time index u16
+0x08 data index u16
+0x0a packed u8: low nibble columns, high nibble interpolation flags
+0x0b zero padding in M4A1
```

Pierwszy slice emituje tylko:

| Path | Type | Low nibble columns | Packed byte |
|---|---:|---:|---:|
| translation | 8 | 3 | `0x03` |
| rotation | 20 | 4 | `0x04` |

Scale type 36, STEP i CUBICSPLINE/Bezier emission sa poza M4A1. High nibble
`0x10` jest rozpoznawany przez reader/report jako Bezier, ale writer M4A1 go
nie emituje.

## 4. Exact canonical R3 observations

R3a `c_phod_horror_b` i R3b `c_phod_horror_p` zostaly odczytane in-place z
`cep3_core1`; zadnego subrange nie zapisano. Oba maja:

- payload `846064`, core `788416 = 0xc07c0`, raw `57636 = 0xe124`;
- animation ArrayDef `pointer=0xe8, used=42, allocated=42`;
- headers od `0x190` do `0xb1aec`, strict increasing;
- `42` own clips, `41` events;
- per clip osobny pelny 27-node tree zgodny z base number/name/topology;
- per clip 21 nodow `0x21` oraz 6 nodow `0x01`;
- wszystkie 882 animation mesh placeholders maja vertex/texture count `0/0`
  i pusty faces ArrayDef; nie posiadaja animation-specific raw geometry;
- 966 controllers: type 20 = 923, type 8 = 43;
- controller sequences `(20)` = 880, `(8,20)` = 43;
- packed byte `+0x0a` tylko `3/4`, byte `+0x0b` zawsze `0`;
- kazdy track ma finite, strict increasing times, zaczyna w `0` i konczy w
  clip `length`;
- event names: `snd_footstep=33`, `hit=6`, `cast=1`,
  `snd_hitground=1`; wszystkie times sa finite, ordered i in-range.

Canonical physical order:

```text
0x000000..0x0000e8  ModelHeader
0x0000e8..0x000190  animation offset array
0x000190..0x0b5b74  animation blocks
0x0b5b74..0x0c07c0  base model tree/core data
raw begins at payload absolute 0x0c07cc
```

Per animation canonical order jest: header, non-empty event array, preorder
node headers/children, a key array i data array danego noda po jego potomkach.
M4A1 zachowuje ten deterministyczny plan. Zero-event clip ma event ArrayDef
`0/0/0` i root bezposrednio po headerze.

Wybrane obserwacje sluza tylko do porownania struktury, nigdy jako dane do
kopiowania:

| Clip | Length | Transition | Events |
|---|---:|---:|---:|
| `cpause1` | 2.666667 | 0.7 | 0 |
| `cwalk` | 1.333333 | 0.4 | 2 |
| `crun` | 1.0 | 0.3 | 2 |
| `ca1slashl` | 1.0 | 0.25 | 1 |
| `cdamagel` | 0.333333 | 0.15 | 2 |
| `cdead` | 0.333333 | 0.5 | 0 |

## 5. Dziewiec jawnych decyzji designu

### D01 - animroot

`animroot` jest wymaganym, jawnym polem owned/user-provided animation setu.
Writer nie wyprowadza go z canonical i nie udaje poznanej semantyki. M4A1
fixture uzywa nazwy output rig root. M4A2 ma przekazac output rig root po
mapowaniu. Bezposredni runtime consumer pola pozostaje
`M4A-DECOMP-ANIMROOT-CONSUMER / OPEN_M6`.

R3 pokazuje, ze animroot nie musi byc nazwa serialized animation root node:
`phod_horror_blak` i `phod_horror_purg` sa stale per model, podczas gdy root
nodes nazywaja sie jak modele. Nie wolno zamrazac tych nazw jako produktu.

### D02 - opaque GeometryHeader `+0x68/+0x6c`

M4A1 zapisuje zero, tak jak zamrozona polityka M4 dla runtime/opaque fields.
Canonical nonzero tokens nie sa offsetami do kopiowania i nie moga trafic do
outputu. Akceptacja zer pozostaje `M4A-RUNTIME-OPAQUE-ZERO / OPEN_M6`.

### D03 - duplicate times

Controller times musza byc strict increasing. Equal lub descending times sa
fatal. Nie powstaje dzielenie przez zero w `(t-prev)/(next-prev)`. Duplicate
event times sa dozwolone; stable sort zachowuje ich input order.

### D04 - animationScale dla own clips

Decomp potwierdza, ze type-8 position jest mnozone przez model
`animationScale` takze w normalnej sciezce evaluatora. M4A1 zachowuje model
`animationScale=1.0`. M4A2 zapisuje translation juz w output/Aurora units i nie
skaluje jej drugi raz. Non-1 compatibility profile jest poza M4A.

### D05 - zero events

Zero events jest legalnym structural loader-smoke: `0/0/0`. Nie zamyka to
event gameplay proof. Dla non-empty eventow writer wymaga finite time w
`0..length`, niepustej poprawnej nazwy oraz stable time sort.

### D06 - skin/mesh nodes w animation tree

Structural choice M4A1: emitowany jest kompletny mirror output rig nodes,
wszyscy jako dummy/header `0x01`, z tymi samymi part numbers, names i parent
topology co base rig. Output mesh segment nodes, w tym skin mesh nodes, sa
pomijane. Controllery sa tylko na rig nodes. Akceptacja rig-only tree pozostaje
`M4A-RUNTIME-ANIM-TREE-PROFILE / OPEN_M6`.

### D07 - charset i length

Bez truncation i bez embedded NUL:

- clip name: non-empty ASCII C string, max 63 bytes;
- animroot: non-empty ASCII C string, max 63 bytes;
- node name: non-empty ASCII C string, max 31 bytes;
- event name: non-empty ASCII C string, max 31 bytes.

Clip names musza byc unikalne po ASCII case-fold.

Target identity jest hierarchiczna. Root jest parowany jawnie; kazde dziecko
musi miec dokladnie jedno name match wsrod bezposrednich dzieci sparowanego
rodzica. Duplicate sibling names sa fatal. Global duplicate names sa dozwolone.
Ta walidacja jest wykonywana tylko dla niepustego animation setu. Jawny empty
set wywoluje zamrozona sciezke M4 bez dodatkowej walidacji nazw/hierarchii.

### D08 - packed key i quaternion

Rows/time/data sa `u16`, nie signed `i16`. Reader rozdziela packed byte na
`columns = raw & 0x0f` oraz `interpolation_flags = raw & 0xf0`. M4A1 emituje
flags zero. Quaternion jest raw `XYZW`, finite, unit w tolerancji `1e-5` i
canonicalized znakiem tak jak M4. Aurora slerp nie normalizuje jawnie danych.

### D09 - mierzalny loader-smoke, frozen zero-animation i clean room

Owned `cpause1` musi zawierac przynajmniej jeden track z minimum dwoma rows
oraz roznica co najmniej `0.01` output unit w translacji albo `1.0` stopnia w
orientacji, liczona w radianach jako `2*acos(abs(dot(q0,q1)))` po unit
normalization i porownana z `pi/180`. Jest
to produktowy gate przeciw pozornie animowanemu rest pose, nie fakt engine'u i
nie zastepuje widocznego M6 proof.

Istniejace `write_binary_mdl` bez animation setu oraz jawny empty set musza
dac byte-identyczny M4 payload:

```text
payload_length = 1188
core_length    = 1072
raw_length     = 104
sha256         = e100130d1dfbd18657413cdb7a701396d466cee081683591fc9836bf0c11b4b2
```

Nie wolno utrwalac R3 bytes, keyframes, event tables, skeletonu ani runtime
tokens. Testy animacji sa synthetic/owned.

## 6. M4A1 public data contract

Wersjonowany typ logiczny:

```yaml
MdlAnimationSetV1:
  schema_version: 1
  clips:
    - name: "cpause1"
      animation_root: "owned ASCII name"
      length_seconds: 1.0
      transition_seconds: 0.25
      events:
        - { time_seconds: 0.5, name: "owned_event" }
      tracks:
        - target_node_id: 7
          path: "TRANSLATION | ROTATION"
          interpolation: "LINEAR"
          times_seconds: [0.0, 1.0]
          values: "Vec3[] or XYZW[]"
```

Animation data nie nalezy do `MdlWriterOptionsV1`. Writer otrzymuje osobny
`MdlAnimationSetV1`, aby zachowac immutable M4 API i frozen empty path.
Raport dodaje clip/event/track counts, layout offsets, packed flags i semantic
diff. Own reader musi expose packed interpolation flags oraz ArrayDef metadata.

M4A1 loader-smoke wymaga co najmniej jednego owned clipu `cpause1`, co najmniej
jednego LINEAR translation lub rotation tracku z dwoma rows i zmiana zgodna z
progiem D09.
Nie oznacza to runtime akceptacji nazwy/routingu.

## 7. M4A2 mapper contract

M4A2 jest wymagany przed `M4A DONE`:

- bierze `IrAnimation` z source GLB i jawne mapowanie source node -> output rig
  node;
- nie mapuje po przypadkowej globalnej nazwie;
- przelicza translation/rotation do tego samego output basis/local space co
  bind rig;
- nie dodaje model alignment do kazdego lokalnego keyframe;
- zachowuje seconds i duration;
- akceptuje tylko `LINEAR`, `translation`, `rotation`;
- odrzuca `STEP`, `CUBICSPLINE`, weights oraz scale w pierwszym profilu;
- normalizuje i sign-canonicalizuje XYZW;
- wynik jest w pelni owned/user-provided oraz przechodzi provenance gates.

Wykonalna granica API jest zamrozona jako nowa, wersjonowana orchestration
route, przy zachowaniu dotychczasowego M3 bez zmian semantycznych:

```rust
pub struct ProfileAAnimatedOutcomeV1 {
    pub base: ProfileAConversionOutcomeV1,
    pub animations: Option<MdlAnimationSetV1>,
}

pub fn convert_profile_a_with_animations_v1(
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    profile_options: &ProfileAOptionsV1,
    mapping: &ProfileAAnimationMappingV1,
) -> Result<ProfileAAnimatedOutcomeV1, ProfileAAnimationFatalError>;
```

Implementacja wyodrebnia stage-private `convert_profile_a_impl` z polityka
source inventory. Istniejace publiczne `convert_profile_a` zawsze przekazuje
`RejectPresent` i zachowuje `M3A-SOURCE-RIG-DEFERRED` oraz
`M3A-SOURCE-ANIMATION-DEFERRED` bit-for-bit. Nowa route moze przekazac
`AllowMappedForM4A2` dopiero po tym, jak M4A2 zwaliduje pelny source rig,
animations, provenance i jawne mapowanie kazdego uzytego source node do output
rig node. Nie czysci, nie klonuje ani nie falsyfikuje `source.ir`/reportu.
Base geometry/weight transfer nadal pochodzi z istniejacego Profile A rig;
source skin jest w M4A2 tylko kontrolowanym zrodlem joint identity/animation
channels, nie niejawna zamiana kontraktu wag M3.

Jesli base conversion jest blocked albo jakikolwiek channel nie ma jawnego
target mapping, outcome nie zawiera ani creature, ani animation setu. Ta route
produkuje oba artefakty w jednym spojnym wyniku i zamyka problem, w ktorym
animowany GLB nie mogl dostarczyc `AuroraCreatureIrV1` przez domyslny gate M3.

## 8. Stable fatal taxonomy

- `M4A-ANIMATION-SET-SCHEMA-INVALID`;
- `M4A-ANIMATION-NAME-INVALID`;
- `M4A-ANIMROOT-INVALID`;
- `M4A-EVENT-NAME-INVALID`;
- `M4A-EVENT-TIME-INVALID`;
- `M4A-CLIP-LENGTH-INVALID`;
- `M4A-TRANSITION-INVALID`;
- `M4A-TRACK-TARGET-MISSING`;
- `M4A-TRACK-TARGET-AMBIGUOUS`;
- `M4A-TRACK-DUPLICATE`;
- `M4A-TRACK-PATH-UNSUPPORTED`;
- `M4A-INTERPOLATION-UNSUPPORTED`;
- `M4A-TRACK-TIME-NOT-STRICT`;
- `M4A-TRACK-TIME-OOB`;
- `M4A-TRACK-VALUE-NONFINITE`;
- `M4A-TRACK-ARITY-INVALID`;
- `M4A-QUATERNION-INVALID`;
- `M4A-CONTROLLER-U16-OVERFLOW`;
- `M4A-LAYOUT-OVERFLOW`;
- `M4A-READBACK-FAILED`;
- `M4A-SEMANTIC-DIFF`;
- `M4A-MAPPER-TARGET-MISSING`;
- `M4A-MAPPER-BASIS-INVALID`.

Reader zachowuje istniejace `M2A-MDL-CONTROLLER-LAYOUT-INVALID` i
`M2A-MDL-CONTROLLER-INDEX-OOB`; nie interpretuje high-bit `u16` jako liczby
ujemnej.

## 9. Required tests i mutation matrix

Happy:

- zero/one/multiple clips round-trip;
- clip name, length, transition, animroot, eventy i tracks exact readback;
- translation type8 i rotation type20 z co najmniej dwoma rows;
- root-only track i deep descendant track z zachowanym ancestor path;
- global duplicate name w roznych branches przechodzi;
- zero events i equal-time stable events przechodza;
- multiple controllers per node maja deterministic `(8,20)` i per-track
  times-then-values data order;
- unit XYZW sign canonicalization i shortest-path pair (`dot < 0`);
- core pointer/ArrayDef claims i exact EOF;
- native/public WASM byte-identical output/report;
- empty set zachowuje frozen M4 SHA.

Negative/mutation:

- header/event/tree/key/data pointer OOB osobno;
- `used > allocated` dla model animations, events, node keys i data;
- `u16` boundary `65535` oraz plan wymagajacy `65536`;
- packed columns `0`, zly low nibble dla type8/20, unsupported high nibble;
- equal/descending/nonfinite/out-of-range controller times;
- zero/negative/nonfinite length; negative/nonfinite transition;
- value count/arity mismatch i nonfinite values;
- zero-length/non-unit quaternion poza tolerancja;
- duplicate clip names po ASCII case-fold;
- invalid/too-long/non-ASCII/NUL clip, animroot, node i event names;
- target node missing, duplicate sibling match i cycle;
- duplicate `(target_node_id,path)` daje `M4A-TRACK-DUPLICATE`;
- STEP/CUBICSPLINE/scale/weights rejected stabilnym kodem;
- event time nonfinite/out-of-range; stable equal-time order nie moze sie zmienic;
- event time nonfinite/out-of-range daje `M4A-EVENT-TIME-INVALID`;
- mutation model animation pointer, animation root, declared budget, event
  stride, key index, data index, packed byte i node name daje nazwany blad;
- truncation-no-panic dla kazdej granicy animation blocku;
- input `AuroraCreatureIrV1` i `MdlAnimationSetV1` pozostaja niemutowane.

## 10. Definition of Done M4A

M4A moze przejsc `VERIFYING`, gdy:

- reader uzywa `u16` i packed columns/flags;
- M4A1 types/writer/readback/semantic diff sa zaimplementowane;
- full synthetic happy/negative/mutation matrix przechodzi;
- frozen zero-animation M4 hash jest bez zmian;
- M4A2 mapuje owned GLB LINEAR translation/rotation do output rig;
- native/WASM, fmt, clippy, workspace tests, wasm32 i Docker quality przechodza;
- independent review nie ma P1/P2;
- evidence nie zawiera canonical payloadu ani prywatnej sciezki.

M4A jest `DONE_STRUCTURAL` dopiero po wszystkich powyzszych punktach. Nie jest
`DONE_RUNTIME`; to nalezy do M6.

## 11. OPEN_M6

- `M4A-DECOMP-ANIMROOT-CONSUMER`;
- `M4A-DECOMP-EVENT-NAME-SEMANTICS`;
- `M4A-RUNTIME-STATE-ROUTING` i loop/one-shot behavior;
- `M4A-RUNTIME-OPAQUE-ZERO` dla GeometryHeader/runtime fields;
- `M4A-RUNTIME-ANIM-TREE-PROFILE` dla rig-only dummy tree bez skin mesh nodes;
- widoczna cpause1 motion oraz pozniejsze movement/gameplay clips w NWN EE.

Te problemy nie blokuja strukturalnej implementacji M4A, ale blokuja kazde
twierdzenie o runtime acceptance.

## 12. Implementation checkpoint M4A1.1

Status: `READER_FIXED_M4A1_WRITER_NEXT`.

Reader correction jest zaimplementowana i zweryfikowana: rows/time/data sa
`u16`, packed columns/flags sa rozdzielone, unknown high bits sa fatal, zero
columns jest fatal, a znany deferred Bezier zachowuje inventory i sprawdza
time range oraz minimalny data-index bound bez falszywego linear decode.

Gates: core `151`, workspace `153` (`151+2`), MDL `40`, WASM `14`, canonical
R3/P-REF `1/1` z exact `966` controllers (`type8=43`, `type20=923`,
`packed3=43`, `packed4=923`, flags zero, decoded true), Docker no-cache
`124.4s`, tag `m2a-quality:m4a-reader`, digest
`sha256:2c8e9c44c349cea030ed919ce705d8c51c5563da0735f08b9d650779989d57bb`,
size `1195609328`, final review `P1=0/P2=0`.

W chwili checkpointu M4A1.1 nastepnym slice'em bylo M4A1.2
`MdlAnimationSetV1` writer/readback; M4A2 pozostawalo wymagane przed M4A DONE.

## 13. Implementation checkpoint M4A1.2

Status: `M4A1_WRITER_VERIFIED_M4A2_NEXT`.

M4A1 types/writer/readback/semantic diff oraz kompletna happy/negative/mutation
matrix sa zaimplementowane i zweryfikowane. Frozen empty-set SHA pozostaje bez
zmian. Publiczny WASM zwraca byte-identyczny payload i report wzgledem core.

Finalne gate'y checkpointu: core `167`, workspace `169` (`167+2`), MDL parser
`40`, MDL writer `31`, WASM Node `15`, canonical R3/P-REF `1/1`, fmt/clippy/
wasm32/diff-check PASS, Docker no-cache digest
`sha256:3f5c035faf90fbe831b9fe7c11dd7bbd24cd3fe4d71ec3a54f37a7332e1d12a6`,
size `1224136248`, dwa finalne rereview `P1=0/P2=0`.

M4A pozostaje `IN_PROGRESS`. Nastepny wymagany slice to M4A2 mapper; runtime
acceptance pieciu nazwanych ograniczen nadal pozostaje `OPEN_M6`.
