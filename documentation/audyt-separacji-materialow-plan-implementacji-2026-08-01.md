# Audyt separacji materiałów, plan implementacji i warunki ukończenia

Data: 2026-08-01
Status: `MS0-MS8_IMPLEMENTED / MS1_OWNER_PROOF_FAILED / MS2_GATE_PENDING_LANE_CLASSIFICATION`
Zakres: wspólna funkcja `Material Separation` dla Creature, Placeable, Tile i przyszłych Itemów

## 0. Stan realizacji celu — 2026-08-01

Zaimplementowano etapy MS0–MS8:

- neutralną inspekcję connected components i stabilne identyfikatory geometrii;
- dokument, walidację, kanoniczny hash i resolver Material Separation V1;
- projekcję per triangle z zachowaniem UV0, tangentów, wag, hierarchii i liczby trójkątów;
- wspólne capabilities dla Creature, Placeable, Tile i przyszłego ModelPart;
- limity 256 materiałów, 4096 sections oraz wspólny budżet 300 000 trójkątów;
- authoring tekstur `SOURCE`/`OVERRIDE`, konwersję PNG/JPEG do TGA i deduplikację po exact SHA-256;
- wieloteksturowe MDL/HAK/manifesty oraz integrację Creature, Placeable i Tile;
- wspólny edytor React/Worker z wyborem component, primitive i node, overlay,
  isolate, assign/unassign, rename, Apply/Cancel/Reset i Undo/Redo.

Potwierdzone bramki offline:

- realny Worker/WASM E2E funkcji materiałowych: 4/4 testy zaliczone;
- testy webowe Material Separation/App/identity: 30/30 zaliczonych;
- pełny zestaw web unit/integration: 256/256 zaliczone w 45 plikach;
- pełny zestaw Core: PASS;
- pełny zestaw WASM: 38/38 PASS;
- produkcyjny build Studio: PASS;
- manifest pakietu: 7 zaliczonych;
- test graniczny: dokładnie 300 000 trójkątów zaakceptowane, 300 001 zablokowane;
- kanoniczny statek Meshy, SHA-256
  `61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278`:
  152 574 trójkąty, 21 936 connected components, 1 primitive;
- odtwarzalny offline preview A/B:
  [`evidence/material-separation-offline-validation-2026-08-01.md`](evidence/material-separation-offline-validation-2026-08-01.md).

Aktualne bindingi WASM zostały zregenerowane, a offline preview A/B jest gotowe.
Niezależne regresje Creature opisane w poprzednim stanie są zamknięte. Skin
golden zaktualizowano dopiero po dodaniu i przejściu byte-exact testu no-op
pustej receptury dla Skin; pełny Core oraz pełny WASM są zielone.

Zamrożono osobny, pierwszy fixture Material Separation, który nie jest kolejną
iteracją istniejącego statku. Dokładne `m2a_ms1_mod.mod` i `m2a_ms1_hak.hak`
zostały zainstalowane bez overwrite i zweryfikowane byte-identical. Handoff:
[`evidence/material-separation-placeable-v1-ready-for-owner-proof-2026-08-01.md`](evidence/material-separation-placeable-v1-ready-for-owner-proof-2026-08-01.md).
Zgodnie z decyzją właściciela agent nie uruchamia Aurora Toolset ani NWN;
pozostaje owner-only visual verdict.

Amendment 2026-08-02: właściciel zgłosił dla exact MS1 „nic w tym module nie
ma”. Offline MOD zawiera Area oraz jeden Placeable w GIT, lecz proof
materializer błędnie użył unit-test `placeables.2da` i Appearance `3` zamiast
pełnego produkcyjnego baseline'u z appendem `16500`. MS1 jest zamrożony, MS2
nie został utworzony. Przed minimalną korektą właściciel musi wskazać, czy
wynik pochodzi z Toolsetu, czy z NWN, aby przypisać właściwą oś
`modelVisibility=not_visible`. Szczegóły:
[`evidence/material-separation-ms1-owner-empty-module-result-2026-08-02.json`](evidence/material-separation-ms1-owner-empty-module-result-2026-08-02.json).

## 1. Werdykt

Separację materiałów można zaimplementować bez ponownego generowania modelu w
Meshy i bez opłacania osobnych elementów. Aurora MDL oraz obecny writer projektu
już obsługują wiele segmentów z różnymi teksturami. Brakuje warstwy authoringu,
która przypisze wybrane fragmenty jednego source mesha do logicznych materiałów.

Funkcja nie powinna należeć do Placeable. Jej rdzeń ma być neutralny względem
typu modelu i działać przed target-specific packaging:

```text
GLB
  -> wspólna inspekcja geometrii
  -> ModelMaterialSeparationDocumentV1
  -> wspólna mapa source triangle -> authored material slot
  -> target projection (Creature / Placeable / Tile / Item part)
  -> wspólny AuroraModelIr + binary MDL writer
  -> target-specific 2DA/GFF/HAK/MOD
```

Rekomendowane V1 łączy podejście 3ds Max i Blendera:

- model danych jak w 3ds Max: stabilny Material ID i przypisanie geometrii do ID;
- sposób selekcji jak w Blenderze: node, primitive i `By Loose Parts`, czyli
  connected components;
- rozdzielenie fizycznego mesha jest wewnętrznym skutkiem eksportu, nie
  destrukcyjną operacją użytkownika;
- tryb pojedynczych face'ów i paint assignment jest V2, ponieważ wymaga nowego
  selektora regionów, region grow i skalowalnego zapisu masek trójkątów.

Porównanie zewnętrznych programów i źródła są zapisane w
[`audyt-material-separation-programy-graficzne-2026-08-01.md`](audyt-material-separation-programy-graficzne-2026-08-01.md).

## 2. Co dokładnie oznacza Material Separation

Material Separation nie tworzy nowej geometrii wizualnej i nie generuje nowego
obiektu w Meshy. Funkcja:

1. tworzy logiczne materiały, np. `Wood`, `Rope`, `Sail`, `Metal`;
2. przypisuje istniejące komponenty albo trójkąty do tych materiałów;
3. przy eksporcie grupuje trójkąty w osobne material slots/mesh sections;
4. pozwala każdemu slotowi odziedziczyć source texture albo otrzymać własny
   texture override;
5. zapisuje wynik w istniejącym MDL jako osobne mesh nodes/segments wskazujące
   różne resrefy tekstur.

Nie należy mylić tej funkcji z:

- automatycznym rozpoznawaniem semantycznym materiałów przez AI;
- nowym UV unwrapem;
- generowaniem nowych tekstur;
- geometry cleanupem albo redukcją trójkątów;
- fizycznym dzieleniem źródłowego GLB na osobne pliki.

## 3. Audyt obecnego kodu

### 3.1. Elementy już wspólne

| Warstwa | Stan obecny | Znaczenie dla separacji |
|---|---|---|
| GLB ingest | `IrPrimitive` zachowuje indeksy, pozycje, normalne, tangenty, UV0, joints i weights | można przypisać materiał bez utraty atrybutów |
| Model IR | `AuroraModelIrV1` jest jawnie neutralny względem rodzaju modelu | Creature, Placeable i Tile mogą korzystać z tego samego wyniku |
| Segmenty | `AuroraModelSegmentV1.material_slot` już istnieje | format wyjściowy rozróżnia materiały |
| Writer | `MdlMaterialTextureBindingV1` mapuje `material_slot -> texture resref` | wiele tekstur nie wymaga nowego formatu MDL |
| Partition writer | duże segmenty są dzielone pod granicę 65 535 indeksów bez usuwania geometrii | separacja nie może przywrócić starego limitu 21 845 jako product budgetu |
| Budżet render-modeli | wspólne maksimum wynosi 300 000 trójkątów, warning 150 000 | liczba trójkątów nie może się zmienić przez samo przypisanie materiału |

### 3.2. Istniejące elementy częściowo użyteczne

`placeable_authoring.rs` ma deterministyczny algorytm connected components.
Komponent jest wyznaczany przez łączność trójkątów po wspólnych indeksach
wierzchołków. Inspekcja zwraca node, primitive, component index, bounds, liczbę
trójkątów i wierzchołków.

Ten algorytm powinien zostać wyciągnięty do neutralnego modułu, np.
`model_components.rs`. Kopiowanie go do Creature, Tile i Item spowodowałoby
rozjazd selekcji między UI i wynikowym MDL.

Placeable authoring potrafi obecnie przepisać komponenty, duplikować je,
transformować i tworzyć dodatkowe material buckets dla flagi `cast_shadow`.
Nie ma jednak user-authored Material ID.

### 3.3. Główna luka w Profile A

`profile_a.rs` buduje material bindings z `IrPrimitive.material_id`, a następnie
wyznacza `material_slot` raz dla całego prymitywu. Bucket Creature ma klucz:

```text
(rig segment id, primitive material slot)
```

Dlatego jeden prymityw z jednym materiałem nie może obecnie zostać rozdzielony
na `Wood`, `Rope` i `Sail`, nawet jeśli ma wiele connected components.

Docelowy klucz musi być wyznaczany dla każdego trójkąta:

```text
(rig segment id, authored material slot)
```

To nie oznacza utraty skinningu. Przy emitowaniu każdego bucketu obecny kod już
kopiuje pozycje, normalne, tangenty, UV i weights. Wierzchołek leżący na granicy
dwóch materiałów może zostać zduplikowany, ale jego wszystkie atrybuty muszą
pozostać identyczne.

### 3.4. Luka w teksturach

Obecny `PlaceableTextureAuthoringDocumentV1` działa per istniejący source
material slot i jest Placeable-only. Waliduje source material id i source image
hash. Nowe dwa authored materials mogą jednak odziedziczyć ten sam source
material, a następnie otrzymać dwa różne override'y.

Potrzebny jest neutralny resolver tekstur identyfikujący materiał przez
`authoredMaterialId`, nie tylko `sourceMaterialId`:

```text
authored material
  -> source fallback material/image
  -> SOURCE albo OVERRIDE
  -> output material slot
  -> output texture resref
```

Dotychczasowy kontrakt Placeable należy zachować jako adapter zgodności. Nie
wolno po cichu zmienić znaczenia jego schema version 1.

### 3.5. Luka w aplikacji i Workerze

- `INSPECT_SOURCE` zna targety `CREATURE | PLACEABLE | TILE`, ale nie `ITEM`;
- `RESOLVE_PLACEABLE_TEXTURES` i odpowiadający mu UI są Placeable-only;
- edytor Placeable ma outliner, multi-select, isolate, viewport i undo/redo,
  lecz nie ma wspólnego trybu `Material Separation`;
- liczba slotów jest dziś niespójna: domyślny Profile A Creature ma guardrail
  jednego materiału, a Placeable dopuszcza do 256. To jest product guardrail,
  nie udowodniony limit Aurory.

UI nie może na stałe zakodować limitu Placeable i udawać, że jest on poprawny
dla Creature. Worker powinien zwracać capabilities wyliczone dla wybranego
targetu i konkretnego modelu, w tym maksymalną liczbę slotów oraz przewidywaną
liczbę wynikowych sections.

### 3.6. Itemy

Item nie jest jeszcze targetem Workera. Audyt Itemów wykazał, że
`baseitems.2da.ModelType` wybiera receptę jednoczęściową, trzyczęściową
Bottom/Middle/Top albo armor parts. Separacja materiałów powinna działać na
każdym model part przed złożeniem i packagingiem Itemu. Nie powinna znać pól
UTI ani zasad nazewnictwa ItemClass.

## 4. Docelowa architektura

### 4.1. Wspólne moduły Core

Rekomendowany podział:

```text
model_components.rs
  inspect_model_components_v1()
  stable source geometry identities

model_material_separation.rs
  schema + validation + canonical hash
  resolve_triangle_material_map_v1()
  reports and diagnostics

model_texture_authoring.rs
  source/override per authored material
  image validation, decode, TGA, deduplication

profile_a.rs / target projectors
  consume resolved material slot per source triangle

model_ir.rs + binary writer
  unchanged geometry contract; consume resulting slots
```

### 4.2. Stabilna tożsamość geometrii

Atomicznym selektorem V1 jest connected component związany z dokładnym source
GLB:

```text
SourceComponentKeyV1
  sceneId
  nodeId
  primitiveId
  componentIndex
```

Dokument jest związany przez `sourceSha256`, dlatego `componentIndex` nie musi
przetrwać zmiany źródłowego GLB. Przy innym hashu resolver zwraca błąd stale
recipe, zamiast próbować zgadywać nowe przypisania.

Kliknięcie node albo primitive w UI jest wygodnym sposobem zaznaczenia wielu
atomicznych komponentów. Dokument wynikowy zapisuje rozwiniętą, jednoznaczną
listę komponentów, bez reguł precedence i bez nakładających się selectorów.

### 4.3. Proponowany kontrakt V1

Przykład informacyjny:

```json
{
  "schemaVersion": 1,
  "sourceSha256": "<exact-glb-sha256>",
  "materials": [
    {
      "authoredMaterialId": "material:wood",
      "displayName": "Wood",
      "previewColor": "#7a4f2a",
      "sourceFallbackMaterialId": 0,
      "sourceFallbackImageSha256": "<sha256>"
    },
    {
      "authoredMaterialId": "material:sail",
      "displayName": "Sail",
      "previewColor": "#c8bea8",
      "sourceFallbackMaterialId": 0,
      "sourceFallbackImageSha256": "<sha256>"
    }
  ],
  "assignments": [
    {
      "component": {
        "sceneId": 0,
        "nodeId": 0,
        "primitiveId": 0,
        "componentIndex": 17
      },
      "authoredMaterialId": "material:sail"
    }
  ]
}
```

Brak jawnego assignmentu oznacza odziedziczenie source material przez
systemowy authored material. Dzięki temu pusty dokument jest operacją identity.

`authoredMaterialId` jest stabilnym kluczem recipe. Numer `materialSlot` jest
deterministycznym wynikiem resolvera, sortowanym według kanonicznych kluczy; UI
nie może zapisywać przypadkowej bieżącej kolejności panelu jako tożsamości.

### 4.4. Wynik resolvera

`ResolvedModelMaterialsV1` powinien zawierać:

- canonical recipe SHA-256;
- listę `authoredMaterialId -> materialSlot`;
- source fallback dla każdego slotu;
- mapę `(source node, primitive, triangle ordinal) -> materialSlot`;
- liczbę source/output triangles;
- liczbę source/output vertices oraz duplicated boundary vertices;
- liczbę wynikowych sections/segments;
- liczbę unassigned i stale components;
- przewidywaną liczbę tekstur i byte total;
- nazwane warnings oraz blocking errors.

Mapa per triangle jest strukturą wewnętrzną Core/WASM. Nie należy wysyłać
setek tysięcy numerów jako zwykłego JSON do Reacta. UI otrzymuje komponenty,
metryki i kolor overlay; ciężka projekcja pozostaje w Workerze.

## 5. Kolejność w pipeline

### 5.1. Wspólna część

```text
source GLB bytes
  -> strict GLB ingest
  -> source component inspection
  -> validate ModelMaterialSeparationDocumentV1
  -> resolve authored material table + triangle material map
```

### 5.2. Placeable

Placeable authoring nadal operuje na source node/component. Podczas append/copy
komponentu pyta wspólną mapę o authored material slot. Kopia dziedziczy materiał
źródłowego komponentu. Transformacja, hide, lock i delete nie zmieniają ID.

Material separation wpływa wyłącznie na projekcję renderową. PWK, collision
polygon i surface ID nie mogą się zmienić. `cast_shadow` pozostaje osobną osią;
wewnętrzny bucket może mieć klucz:

```text
(authored material slot, castShadow)
```

### 5.3. Creature

Profile A wybiera authored material slot dla każdego source triangle przed
umieszczeniem go w rig segment bucket. Przy kopiowaniu wierzchołków musi
zachować:

- node/skin ownership;
- joints i znormalizowane weights;
- normalne, tangenty i handedness;
- UV0;
- animacje, kontrolery i eventy;
- liczbę oraz kolejność trójkątów w sensie geometrycznym.

Samo przypisanie materiału nie może ponownie uruchamiać retargetingu ani
stabilizacji detached accessories.

### 5.4. Tile

Render MDL korzysta z tych samych authored slots. WOK, surface type, AABB,
tile footprint i SET pozostają niezależne. Material ID nie jest WOK surface ID.

### 5.5. Item

Po dodaniu targetu Item wspólny resolver działa osobno dla każdego partu.
Item assembler otrzymuje już gotowe MDL i nie interpretuje Material Separation.
Bottom/Middle/Top oraz armor-part placement pozostają kontraktem Itemu.

## 6. UX V1

Tryb w Studio: `Material Separation`.

Minimalny panel:

- lista materiałów: kolor, nazwa, liczba komponentów/trójkątów, texture mode;
- `New Material`, `Rename`, `Delete Unused`;
- `Assign Selection`, `Unassign to Source`;
- `Select by Material`;
- `Select Unassigned`;
- selekcja node, primitive i `By Loose Parts`;
- overlay `Material ID Colors` w viewport;
- isolate/hide selected;
- licznik wynikowych slots, sections, textures i rozmiaru HAK;
- `Apply`, `Cancel`, `Reset` oraz undo/redo;
- widoczny komunikat, że V1 nie zmienia UV i nie rozpoznaje materiałów
  automatycznie.

Operacje `Apply` i `Cancel` są wymagane. Samo kliknięcie komponentu nie może
mutować aktywnego build recipe poza historią undo/redo.

### 6.1. Gdy connected components nie wystarczą

Jeśli drewno, lina i żagiel są zespawane w jeden connected component, V1 nie
może rozdzielić ich poprawnie. UI pokazuje nazwany stan:

`MATERIAL-SEPARATION-COMPONENT-GRANULARITY-INSUFFICIENT`

Użytkownik może przypisać cały komponent albo pozostawić source material. UI
nie może udawać, że automatycznie rozpoznało granicę.

V2 dodaje:

- face selection;
- box/lasso selection;
- region grow po łączności i kącie normalnych;
- `Select Similar` jako sugestię użytkownika;
- kompaktową maskę/range encoding triangle ordinals;
- paint assignment Material ID.

## 7. Plan implementacji

### Etap MS0 — kontrakty i fixture'y

1. Dodać strict-schema `ModelMaterialSeparationDocumentV1` i canonical hash.
2. Ustalić target capabilities zamiast hardcodowania limitu w UI.
3. Dodać fixture jednego prymitywu, jednego source materialu i co najmniej
   czterech disconnected components.
4. Dodać skinned fixture z komponentami przypisywanymi do dwóch materiałów.
5. Zapisać identity/no-op golden dla modelu bez authoringu.

Wyjście: parser, walidator, fixture'y i testy negatywne; brak UI.

### Etap MS1 — neutralna inspekcja komponentów

1. Wyciągnąć connected-components z `placeable_authoring.rs` do wspólnego Core.
2. Dodać `inspect_model_components_v1` dla default scene i node instances.
3. Użyć tego samego wyniku w Placeable authoring i Material Separation.
4. Dodać granice czasu/pamięci i błędy dla topology/index OOB.

Wyjście: jeden algorytm i jedna tożsamość komponentu dla wszystkich targetów.

### Etap MS2 — resolver materiałów

1. Walidować source hash, unikalne Material IDs, component keys i coverage.
2. Rozwiązywać systemowe source materials i authored materials do stabilnych
   slots.
3. Tworzyć mapę material slot per source triangle.
4. Raportować duplicated boundary vertices oraz section explosion.
5. Zapewnić identity path dla braku recipe.

Wyjście: neutralny `ResolvedModelMaterialsV1`.

### Etap MS3 — integracja Profile A i Model IR

1. Zastąpić primitive-only lookup opcjonalnym lookupem per triangle.
2. Bucketować Creature po `(rigSegment, authoredMaterialSlot)`.
3. Zachować wszystkie vertex attributes i weights przy duplikacji granicznej.
4. Zachować automatyczny writer partition per 65 535 indeksów.
5. Rozszerzyć raport/readback o authored slot identity bez zmiany znaczenia
   istniejącej schema V1; jeżeli potrzebna jest zmiana kontraktu, wprowadzić
   jawną V2 albo osobny versioned report zamiast cichej mutacji.

Wyjście: wielomateriałowy binary MDL z jednego source prymitywu.

### Etap MS4 — wspólny texture authoring

1. Wyciągnąć neutralny resolver z `placeable_texture.rs`.
2. Związać texture recipe z `authoredMaterialId` i source fallback hash.
3. Obsłużyć `SOURCE` i `OVERRIDE` per authored material.
4. Deduplikować wyłącznie exact wynikowe bajty TGA.
5. Zachować adapter `PlaceableTextureAuthoringDocumentV1`.
6. Włączyć separation hash oraz override hashes do artifact identity.

Wyjście: różne tekstury dla fragmentów pierwotnie używających jednego materiału.

### Etap MS5 — WASM i Worker

1. Dodać neutralne requesty `INSPECT_MODEL_COMPONENTS` i
   `RESOLVE_MODEL_MATERIALS`.
2. Zwracać target capabilities i kompaktowy inspection/report JSON.
3. Przekazywać recipe do wszystkich aktualnych build requestów.
4. Nie wysyłać dużych triangle maps do głównego wątku.
5. Zachować transfer ownership dużych ArrayBufferów.

Wyjście: jedna ścieżka API dla Creature, Placeable i Tile.

### Etap MS6 — wspólny edytor Studio

1. Dodać `features/material-separation/`, niezależnie od
   `features/placeable-authoring/`.
2. Współdzielić selection/outliner/viewport adaptery, ale nie kopiować stanu.
3. Dodać Material ID overlay, panel materiałów, Assign/Unassign i Select by
   Material.
4. Dodać Apply/Cancel/Reset i bounded undo/redo.
5. Renderować source/edited preview na tej samej geometrii i UV.

Wyjście: ten sam ekran dostępny dla Creature, Placeable i włączonego Tile.

### Etap MS7 — target integrations

1. Placeable: render split, dziedziczenie przez copy, PWK invariant.
2. Creature: skin/animation invariants i co najmniej dwa slots.
3. Tile: render split przy byte-identical WOK/SET dla samej zmiany materiału.
4. Przygotować adapter `ModelPartInput` dla przyszłego Itemu bez dodawania
   pozornego, niedziałającego targetu `ITEM` do UI.

Wyjście: funkcja jest wspólna w kodzie i działa na wszystkich aktualnych
render-model targets.

### Etap MS8 — regresja, wydajność i wydanie

1. Testy Core, writer/readback, WASM, Worker, React i browser integration.
2. Test na modelu blisko 300 000 trójkątów.
3. Test na realnym lokalnym GLB Meshy z `sample-3d` bez nowego wywołania API.
4. A/B source vs separated w offline preview.
5. Dopiero po spełnieniu offline gates przygotować exact artifact do
   właścicielskiego proofu Toolset/NWN, zgodnie z aktywnym model iteration gate.

Wyjście: release candidate gotowy do human-owned final proof.

## 8. Warunki ukończenia

Funkcja jest ukończona dopiero wtedy, gdy wszystkie poniższe warunki są
spełnione. Zielony panel UI bez binary readbacku nie wystarcza.

### 8.1. Kontrakt i determinizm

- [x] Source GLB pozostaje byte-identical i read-only.
- [x] Dokument ma strict schema, exact `sourceSha256` i canonical recipe hash.
- [x] Ten sam GLB, recipe, target i tekstury dają byte-identical artefakty.
- [x] Inny source hash, brakujący komponent, duplikat ID i nieznany materiał są
      odrzucane nazwanym błędem.
- [x] Brak recipe daje ten sam wynik co legacy pipeline; golden/no-op nie ma
      nieuzasadnionego driftu MDL, TGA ani raportu.
- [x] Separacja nie wywołuje Meshy API i nie zużywa tokenów/kredytów.

### 8.2. Geometria i writer

- [x] Jednoprymitywowy, jednomateriałowy fixture tworzy co najmniej dwa
      authored material slots.
- [x] Source i output mają tę samą liczbę oraz geometryczny zbiór trójkątów.
- [x] Pozycje, normalne, tangenty, UV0 i face surface metadata są zachowane.
- [x] Duplikowane są wyłącznie wierzchołki wymagane przez granice bucketów lub
      writer partition; liczba i powód są raportowane.
- [x] Dokładnie 300 000 trójkątów jest przyjmowane, a więcej niż 300 000
      blokowane wspólnym product budgetem.
- [x] Segment ponad granicą jednego triangle-list stream jest deterministycznie
      dzielony bez usuwania trójkątów i bez zmiany materiału.
- [x] Binary readback potwierdza texture resref właściwy dla każdego wynikowego
      mesh section.

### 8.3. Tekstury

- [x] Każdy authored material może niezależnie używać `SOURCE` albo `OVERRIDE`.
- [x] Dwa authored materials wywodzące się z jednego source materialu mogą
      otrzymać dwa różne TGA/resrefy.
- [x] Identyczne wynikowe TGA są deduplikowane po exact SHA-256.
- [x] UV nie jest automatycznie zmieniane; UI i raport mówią o tej granicy.
- [x] Unsupported alpha jest blokowane; pomijane source PBR maps są jawnie
      raportowane przez istniejący texture contract, nigdy cicho spłaszczane.
- [x] Separation recipe i texture override hashes uczestniczą w artifact
      identity i manifest provenance.

### 8.4. Placeable

- [x] Komponent skopiowany w edytorze dziedziczy authored material.
- [x] Transformacja, rename, hide i lock nie zmieniają przypisania.
- [x] Delete usuwa tylko wynikową geometrię danego elementu.
- [x] Zmiana wyłącznie materiału pozostawia PWK/collision geometry i surface ID
      bez zmian; wymagany jest exact hash albo semantyczny readback zależnie od
      aktualnego deterministic packaging contractu.
- [x] `cast_shadow` i Material ID są niezależnymi osiami bucketowania.

### 8.5. Creature

- [x] Creature z jednym source materialem jest zapisany z co najmniej dwoma
      poprawnymi texture resrefami.
- [x] Bone IDs, influence count i znormalizowane weights są równe przed i po
      separacji dla odpowiadających sobie wierzchołków.
- [x] Liczba klipów, controller keys, eventy i duration nie zmieniają się przez
      samo przypisanie materiałów.
- [x] Rigid i Skin segments przechodzą osobne testy.
- [x] Material separation nie zmienia retargetingu, root motion ani accessory
      stabilization.

### 8.6. Tile i przyszły Item

- [x] Tile render MDL używa authored slots, a WOK surface IDs, footprint i SET
      readback pozostają bez zmian.
- [x] Wspólny Core nie importuje modułów Placeable, Creature, Tile ani Item.
- [x] Publiczny kontrakt przyjmuje target-neutral model/part input.
- [x] Dodanie Itemu nie wymaga kopiowania resolvera ani UI; pełne ukończenie
      Item integration nastąpi dopiero wraz z rzeczywistym Item targetem i
      testami jedno-, trzy- oraz armor-part, nie przez sam placeholder enum.

### 8.7. UI i Worker

- [x] Ten sam `Material Separation` editor działa dla Creature i Placeable;
      Tile korzysta z niego po włączeniu targetu.
- [x] Node/primitive selection może zostać rozwinięta do connected components.
- [x] Assign, Unassign, Select by Material, Unassigned, overlay, isolate,
      Apply, Cancel, Reset oraz undo/redo mają testy.
- [x] UI pokazuje wynikową liczbę slots, sections, tekstur i warnings.
- [x] Ciężka analiza 300k nie blokuje głównego wątku; odbywa się w Workerze.
- [x] Worker zwraca capabilities targetu; UI nie hardcoduje limitu 256.
- [x] Stale odpowiedź Workera nie może nadpisać nowszego source/recipe state.

### 8.8. Testy i proof

- [x] Zielone są Rust unit/integration, writer/readback, WASM, Worker, React i
      browser integration tests.
- [x] Jest synthetic fixture oraz co najmniej jeden realny GLB z `sample-3d`.
- [x] Offline preview pokazuje source i separated variant z exact hashami.
- [x] Nie utworzono kolejnej iteracji istniejącego modelu tylko po to, aby
      ominąć model iteration gate.
- [ ] Finalny Toolset/NWN verdict wykonuje właściciel na jednym zamrożonym,
      hash-verified lineage; agent nie deklaruje wizualnego sukcesu na podstawie
      samego preview webowego. Exact lineage jest zainstalowany i ma status
      `ready_for_owner_proof`; checkbox pozostaje otwarty wyłącznie do wyniku
      właściciela.

## 9. Ryzyka i zabezpieczenia

| Ryzyko | Zabezpieczenie |
|---|---|
| cały statek jest jednym welded componentem | jawny błąd granularity; Face Mode w V2 |
| zbyt wiele materiałów mnoży draw calls i rig buckets | capabilities + section-count gate + raport przed Apply |
| nowe tekstury nie pasują do source UV | source/edited preview, brak obietnicy re-UV |
| rozdzielenie niszczy skin weights | atrybut-preserving vertex duplication i testy per vertex |
| Placeable material zmienia PWK | oddzielna render projection i invariant PWK |
| UI oraz Core inaczej liczą komponenty | jeden algorytm Core; UI konsumuje inspection |
| stare Placeable recipe przestaje działać | jawny adapter V1, brak cichej zmiany schema |
| duża mapa trójkątów przeciąża React | mapa pozostaje w Worker/Core, UI dostaje agregaty |
| przypadkowe koszty Meshy | brak połączeń sieciowych w funkcji separation |

## 10. Decyzje wymagane przed kodowaniem

Nie ma blockerów koncepcyjnych. Etap MS0 musi jednak utrwalić dwie decyzje
produktowe:

1. docelowy material/section budget dla każdego targetu. Istniejące `1` dla
   Creature i `256` dla Placeable są guardrailami historycznych profili, a nie
   wspólnym faktem Aurory;
2. czy V1 kończy się na connected components. Rekomendacja: tak. Dodanie Face
   Mode do tego samego pierwszego slice'u istotnie zwiększa koszt, pamięć UI i
   ryzyko błędnego przypisania na modelach 300k.

## 11. Rekomendowana kolejność produktu

Najmniejszy wartościowy vertical slice to:

```text
Placeable + Creature
  + connected-components selection
  + Material ID list
  + SOURCE/OVERRIDE texture per ID
  + binary readback
  + zachowanie PWK/skin/animation invariants
```

To rozwiązuje bieżący problem statku: jeden model Meshy można podzielić na
logiczne drewno, liny, płótno i metal bez generowania każdego elementu osobno.
Jeżeli source topology nie rozdziela tych części jako komponentów, następnym
etapem jest Face Mode V2, a nie agresywne czyszczenie lub automatyczne usuwanie
geometrii.
