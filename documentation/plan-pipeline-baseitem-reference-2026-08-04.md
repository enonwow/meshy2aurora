# Pipeline BaseItem + model referencyjny: plan implementacji i kryteria ukończenia

Data: 2026-08-04
Status: `M2_VERTICAL_SLICE_IMPLEMENTED / READY_FOR_OWNER_PROOF`
Branch/worktree: `items` / `C:\Projects\meshy2aurora\.worktrees\items`

## 1. Cel

Użytkownik ma móc wskazać spójny kontekst zasobów NWN, wybrać istniejący
`BaseItem` i bazowy wygląd, a następnie wygenerować w Meshy nowe party, które:

- używają prawidłowej recepty Aurory;
- są dopasowane do natywnej ramy przestrzennej wybranego wyglądu;
- zachowują wspólny origin i właściwy attachment;
- składają się zarówno ze sobą, jak i z kompatybilnymi partami natywnymi;
- są pakowane jako nowe zasoby bez kopiowania referencyjnych payloadów.

Osią produktu nie jest `weapon / shield`. Autorytatywny przepływ pozostaje:

```text
resource context
  -> effective baseitems.2da
  -> BaseItem row
  -> ModelType + ItemClass + attachment route
  -> reference appearance
  -> reference MDL slot frames
  -> Meshy generation + reference fit
  -> UTI/MDL/TGA/HAK/MOD
```

## 2. Decyzje zakresowe

1. Wersja pierwsza tworzy nowe wyglądy dla **istniejących BaseItemów**. Nowy
   rekord `baseitems.2da` pozostaje osobnym przypadkiem authoringu.
2. Vanilla NWN jest bazowym providerem zasobów. HAK-i są opcjonalną,
   uporządkowaną warstwą wejściową; opcjonalny MOD może dostarczyć UTI lub
   zasoby referencyjne.
3. Pipeline nigdy nie miesza ręcznie tabeli z jednego kontekstu z modelami z
   innego. Wszystkie resrefy są rozwiązywane przez jeden zamrożony kontekst.
4. Referencyjne HAK/MOD/BIF są read-only. Do produktu trafiają locatory,
   resrefy, hashe i wyprowadzone invarianty liczbowe, nie cudze payloady.
5. Brak kompletnej referencji lub profilu attachmentu kończy się
   `MANUAL_REQUIRED` albo `UNSUPPORTED`; nie wolno wracać do cursor fitu.
6. ModelType 2 nie oznacza automatycznie broni. Render/attachment route musi
   wynikać z danych Aurory, a nie z nazwy lub wyglądu obiektu.

## 3. Docelowy model wejść

### 3.1 `ResourceContextV1`

Kontekst opisuje jeden deterministyczny widok zasobów:

```yaml
resourceContext:
  vanilla:
    installationRoot: <local read-only path>
    keys: [nwn_base.key, ...]
  orderedHaks:
    - path: <optional first HAK>
    - path: <optional second HAK>
  module: <optional MOD>
```

Publiczny sidecar/report nie musi ujawniać lokalnych ścieżek. Musi zawierać
role, kolejność, nazwy logiczne, rozmiary i SHA-256 każdego źródła oraz wersję
polityki rozwiązywania zasobów.

### 3.2 `EffectiveBaseItemsSelectionV1`

Resolver wybiera dokładny zwycięski zasób `baseitems.2da` w tym samym
kontekście i zapisuje:

- SHA-256 tabeli;
- provider i logiczny locator;
- drukowany numer wiersza `BaseItem`;
- snapshot wymaganych pól, między innymi `Label`, `ModelType`, `ItemClass`,
  `GenderSpecific`, `EquipableSlots`, `DefaultModel`, `DefaultIcon`;
- capability i dokładny schemat slotów.

### 3.3 `ReferenceAppearanceSelectionV1`

Bazowy wygląd można wskazać trzema sposobami:

1. `REFERENCE_UTI` — istniejący UTI z tego samego kontekstu;
2. `EXPLICIT_VARIANTS` — ręcznie wybrane wartości partów zgodne z BaseItemem;
3. `APPROVED_PROFILE` — wcześniej zatwierdzony profil o dokładnej tożsamości
   źródeł.

Dla ModelType 2 wybór UTI lub wariantów rozwiązuje dokładnie Bottom, Middle i
Top. Dla ModelType 0/1 rozwiązuje jeden model. CAPART, robe i cloak wymagają
dodatkowego kontekstu postaci/szkieletu i nie używają profilu dłoni.

### 3.4 `ItemAttachmentProfileV1`

Profil przestrzenny jest deterministycznym wynikiem naszego readera i zawiera:

- identity: hash kontekstu, `baseitems.2da`, BaseItem, ItemClass, ModelType,
  reference appearance i wszystkich referencyjnych MDL;
- `renderRoute` i attachment class;
- wspólny model-local origin, osie i handedness;
- per slot: node/controller ownership, translation, orientation, bounds,
  connector zones i dozwolone obszary rozszerzenia;
- wspólną strefę attachment/grip, jeżeli dana trasa ją ma;
- tolerancje conformance i SHA-256 całego profilu;
- provenance `AURORA_DECOMPILATION`, `RETAIL_P_REF` albo
  `OWNER_APPROVED_CUSTOM_REFERENCE`.

Profil nie zawiera geometrii ani tekstur modelu referencyjnego.

## 4. Plan implementacji

### P0 — audyt kolejności zasobów Aurory

1. Uzupełnić audyt dekompilacji o dokładną kolejność rozwiązywania BIF/HAK/MOD
   dla 2DA, UTI, MDL i TGA.
2. Potwierdzić zachowanie przy dwóch HAK-ach zawierających ten sam resref.
3. Potwierdzić, czy `baseitems.2da` jest wybierany jako jeden zwycięski zasób,
   oraz zapisać zachowanie braków i kolizji.
4. Dopiero z potwierdzonej kolejności utworzyć wersjonowaną politykę resolvera.

Ta faza jest obowiązkowym Aurora First gate. Nie implementujemy kolejności na
podstawie pamięci lub przypuszczenia.

### P1 — domena i lineage

1. Dodać ścisłe schematy `ResourceContextV1`,
   `EffectiveResourceResolutionV1`, `EffectiveBaseItemsSelectionV1`,
   `ReferenceAppearanceSelectionV1` i `ItemAttachmentProfileV1`.
2. Związać je z `ItemGenerationSessionV1` i raportem builda.
3. Każda zmiana źródła, kolejności HAK-ów, tabeli, UTI, wariantu albo MDL
   unieważnia profil, fit i wcześniejszą akceptację.
4. Dodać redakcję lokalnych ścieżek z eksportowalnych raportów bez utraty
   hashy i logicznego provenance.

### P2 — własny read-only resolver zasobów

1. Rozszerzyć własny KEY/BIF/ERF/HAK reader o jeden publiczny resolver
   działający według polityki z P0.
2. Zwracać payload do analizy oraz pełne `winningProvider` i listę zasłoniętych
   kolizji.
3. Rozwiązywać przez ten sam kontekst `baseitems.2da`, UTI, MDL i ikony.
4. Nie modyfikować instalacji, źródłowych HAK-ów ani MOD-ów.

### P3 — wybór BaseItemu i bazowego wyglądu

1. Studio pokazuje katalog wierszy z efektywnego `baseitems.2da`.
2. Po wyborze BaseItemu pokazuje wykryty ModelType, ItemClass, sloty,
   equipability i wymaganą trasę referencyjną.
3. Użytkownik wybiera UTI, warianty albo approved profile.
4. Resolver blokuje referencję z innym BaseItemem/ModelType albo niekompletnym
   zestawem zasobów.
5. UI pokazuje pochodzenie każdego zwycięskiego zasobu: Vanilla, konkretny HAK
   lub MOD.

### P4 — ekstrakcja profilu referencyjnego

1. Odczytać każdy referencyjny binary MDL własnym readerem in-place.
2. Wyprowadzić wspólny origin, pełną ramę, per-slot controller transform,
   world-in-model bounds, connector overlap i attachment zone.
3. Dla pierwszego profilu `BaseItem=1 / WSwLs / ModelType 2` użyć istniejącego
   P-REF `23/63/23` jako golden contract.
4. Zapisać deterministyczny profil liczbowy i jego hash.
5. Odrzucać referencje z nieobsługiwaną rodziną node/controller zamiast
   upraszczać je bez ostrzeżenia.

### P5 — `REFERENCE_SLOT_FRAME_FIT_V1`

1. Zastąpić dla profili referencyjnych `target_lengths + cursor` fitterem
   pracującym osobno w ramie każdego slotu.
2. Znormalizować orientację Meshy, ale zachować wspólny origin profilu.
3. Dopasować geometrię partu do target bounds/connector zone odpowiedniego
   slotu i wypalić transform do child node kontrolera MDL.
4. Pozwolić na jawne rozszerzenia tylko w kierunkach dozwolonych przez profil,
   np. wydłużenie ostrza na zewnątrz od złącza bez przesuwania chwytu.
5. Walidować cały model oraz kombinacje custom/native, a nie tylko trzy
   customowe party razem.
6. Usunąć cichy fallback do starego fittera dla equipowalnych profili.

### P6 — generowanie Meshy sterowane profilem

1. Każdy draft partu otrzymuje rolę slotu, docelowe proporcje, connector zone,
   kierunek osi oraz reference silhouette/envelope.
2. Review przed płatnym taskiem pokazuje cały wybrany profil i przewidywany
   koszt wszystkich brakujących partów.
3. Wynik Meshy nadal przechodzi niezależny intake, triangle budget, UV,
   materiały, tekstury i provenance.
4. Nie zakładamy, że Meshy zachowa pivot lub skalę; autorytatywny jest fit i
   readback po konwersji.

### P7 — Studio i podgląd

1. Dodać kroki `Resource source`, `Base item`, `Reference appearance` i
   `Reference frame` przed generowaniem.
2. Preview pokazuje jednocześnie:
   - origin całego itemu;
   - attachment/grip zone;
   - bounds i connector zone każdego slotu;
   - custom geometry i opcjonalny obrys referencyjny.
3. Dodać tryb mixed-part preview: wymiana jednego customowego slotu na
   referencyjny bez zmiany pozostałych.
4. Build pozostaje zablokowany przy stale profile, nierozwiązanym resrefie,
   wyjściu poza niedozwolony envelope lub nieprzejściu connector gate.

### P8 — pakowanie i zależności

1. Alokować nowe, wolne warianty w namespace wybranego ItemClassu; nigdy nie
   nadpisywać modelu referencyjnego.
2. UTI zapisuje istniejący `BaseItem` i nowe wartości partów.
3. Wygenerowany HAK zawiera tylko własne MDL, tekstury, ikony i wymagane własne
   tabele. Nie kopiuje zasobów referencyjnego HAK-a lub Vanilli.
4. Jeżeli runtime wymaga wejściowego custom HAK-a, raport deklaruje jego
   uporządkowaną zależność nazwą i SHA-256.
5. Niezmieniony retailowy `baseitems.2da` nie jest bez potrzeby pakowany do
   outputu. Customowa tabela jest traktowana zgodnie z jawną polityką
   dependency/ownership, bez cichego vendoringu.

### P9 — proof profiles

1. Zachować prawdziwy ground Item dla Item Properties.
2. Dodać osobny equipped-render witness z tym samym exact UTI i tym samym HAK;
   creature jest wyłącznie nośnikiem wyposażenia, nigdy substytutem itemu.
3. Raportować osobno:
   - Item Properties assembly;
   - mixed native/custom assembly;
   - ground presentation;
   - NWN equipped attachment/grip.
4. Granicą agenta pozostaje `ready_for_owner_proof`; wizualny werdykt wydaje
   właściciel.

### P10 — kolejne rodziny

Po zamknięciu vertical slice `WSwLs` rozszerzyć ten sam model źródeł i lineage,
ale nie ten sam profil przestrzenny:

1. ModelType 0 — single model z attachmentem wynikającym z BaseItemu;
2. ModelType 1 — single model + native color channels/gender rules;
3. ModelType 2 bez hand attachmentu — własny render route, np. rodziny
   niebędące bronią;
4. ModelType 3/CAPART — profile race/gender/phenotype/body-node;
5. robe — skeleton/body masks i właściwe części ukrywane;
6. cloak — osobny `CloakModel` i profil szkieletu/animacji.

## 5. Kryteria ukończenia

### K0 — audyt i resolver

- kolejność zasobów jest udokumentowanym faktem z dekompilacji Aurory;
- test Vanilla-only rozwiązuje dokładny retailowy `baseitems.2da`;
- test Vanilla + ordered HAK stack wybiera deterministycznego providera;
- zmiana kolejności HAK-ów zmienia identity kontekstu i wynik tam, gdzie
  istnieje kolizja;
- raport wskazuje zwycięski i zasłonięte zasoby;
- źródłowe instalacje/HAK/MOD pozostają byte-identical i read-only.

### K1 — wybór danych bazowych

- użytkownik nie może wybrać BaseItemu spoza efektywnej tabeli;
- wybór zapisuje hash tabeli i pełny snapshot wiersza;
- reference UTI musi pochodzić z tego samego zamrożonego kontekstu;
- manualne warianty muszą spełnić dokładny schemat ModelType;
- brak modelu, niezgodny BaseItem albo niekompletny zestaw kończy się
  jednoznacznym błędem bez fallbacku.

### K2 — profil referencyjny

- profil jest deterministyczny i ma własny SHA-256;
- każda zmiana tabeli, UTI lub MDL zmienia profile identity;
- dla P-REF `WSwLs 23/63/23` odtworzone zostają dokładne controller positions i
  skorygowane triangle-surface ranges Y: Bottom `[-0.2025382, 0.0841520]`,
  Middle `[0.042125102, 0.1570407]`, Top `[0.1283190, 0.9313860]`;
- profil zawiera wspólny origin oraz per-slot frame, nie wyłącznie długości;
- żaden retail/custom payload nie jest zapisany w profilu ani skopiowany do
  repo/outputu.

### K3 — fit i MDL writer

- stary V7 układ `[0,.22] / [.212,.292] / [.284,1.184]` jest odrzucany przez
  profil WSwLs;
- każdy wygenerowany MDL po own readback ma controller i bounds zgodne z jego
  referencyjnym slotem w ustalonej tolerancji;
- Bottom, Middle i Top zachowują identyczny wspólny origin;
- custom/custom oraz trzy kombinacje z jednym wymienionym partem native/custom
  przechodzą slot-frame i connector conformance;
- dozwolone wydłużenie ostrza nie przesuwa Bottom, Middle, grip zone ani
  wewnętrznego złącza Top;
- brak profilu nie uruchamia cursor fitu.

### K4 — generowanie i Studio

- użytkownik przechodzi kolejno przez źródło, BaseItem, referencję i profil;
- UI pokazuje providera oraz hash efektywnego `baseitems.2da`;
- każdy task Meshy jest związany z exact slotem i profile hash;
- zmiana kontekstu lub referencji unieważnia oczekujące taski/fit w sposób
  kontrolowany i nie tworzy automatycznie kolejnego płatnego taska;
- preview pokazuje wspólny origin, grip/attachment i slot envelopes;
- klawiatura, błędy, retry/recovery oraz zapis sesji mają pokrycie testowe.

### K5 — build i lineage

- UTI używa wybranego istniejącego BaseItemu i nowych selektorów modeli;
- wszystkie resrefy mieszczą się w limicie, są collision-safe i nie nadpisują
  referencji;
- output zawiera wyłącznie własne wygenerowane payloady;
- zależności na wejściowe HAK-i są jawne, uporządkowane i zahashowane;
- raport wiąże context, 2DA, BaseItem, reference, profile, Meshy task/GLB, fit,
  MDL/TGA, UTI, HAK i MOD jednym immutable lineage;
- triangle budget, binary stream limits, texture/icon i semantic readback
  pozostają zielone.

### K6 — właścicielski proof pierwszego vertical slice

Ten sam exact kandydat `WSwLs` musi spełnić wszystkie punkty:

- prawidłowy złożony item w Item Properties;
- prawidłowa ikona;
- brak rozłączenia w trzech próbach mixed native/custom;
- prawidłowe położenie ground itemu;
- prawidłowy chwyt przez postać w NWN w porównaniu z natywnym długim mieczem;
- `modelVisibility=visible`, `proofCompleteness=verified` i
  `visualAcceptance=accepted` dla każdej wymaganej powierzchni.

Sukces builda, samego preview albo samego Item Properties nie zamyka K6.

### K7 — bezpieczeństwo pozostałych tras

- ModelType 2 potion/non-hand route nie dostaje profilu WSwLs tylko dlatego,
  że ma trzy party;
- ModelType 0/1 nie używa trzech slotów;
- CAPART/robe/cloak nie używają weapon grip origin;
- niezaimplementowana trasa jest jawnie zablokowana jako `UNSUPPORTED`, nie
  prezentowana jako gotowa;
- każda później dodana trasa otrzymuje własny corpus, profile, readback i
  właścicielski proof.

## 6. Kamienie milowe

### M1 — fundament źródeł

P0-P3 oraz K0-K1. Użytkownik może bezpiecznie wybrać kontekst, BaseItem i
referencję, ale nie generuje jeszcze nowego modelu.

### M2 — pierwszy pełny przypadek `WSwLs`

P4-P9 oraz K2-K6. To jest minimalny release, który dowodzi całej ścieżki na
istniejącym BaseItem 1.

### M3 — rozszerzenie rodzin

P10 oraz K7 plus osobne kryteria wizualne każdej trasy. Dopiero ten etap można
nazywać ogólnym pipeline'em istniejących BaseItemów.

## 7. Definition of Done

Implementacja nie jest ukończona, dopóki:

1. nie ma deterministycznego resource context i efektywnego `baseitems.2da`;
2. bazowy wygląd nie prowadzi do zahashowanego profilu referencyjnego;
3. fit nie zachowuje natywnego originu oraz per-slot frames;
4. output nie ma pełnego lineage i collision-safe namespace;
5. exact `WSwLs` nie przejdzie właścicielskiego proofu Item Properties,
   mixed-part i equipped NWN;
6. pozostałe attachment routes nie są obsługiwane osobnymi profilami albo
   jawnie blokowane.

## 8. Stan bieżący

Vertical slice `BaseItem=1 / WSwLs / ModelType 2` jest zaimplementowany i
zamrożony jako V8. Pipeline ma read-only KEY/BIF resolver, fail-closed
`ResourceContextV1`, wybór efektywnego BaseItemu, profil attachmentu wyprowadzany
z dokładnych referencyjnych MDL, `ITEM_REFERENCE_SLOT_FRAME_FIT_V1`, conformance
binary MDL, osobny presentation fit ikon oraz komplet UTI/MDL/TGA/HAK/MOD.

Exact V8 ma status `ready_for_owner_proof`. K6 pozostaje otwarte do chwili
werdyktu właściciela w Item Properties i NWN. Wielokontenerowa kolejność
kolizyjnych custom HAK-ów pozostaje fail-closed do potwierdzenia jako fakt z
Aurory; nie jest zastępowana zgadywaną kolejnością. P10 i K7 pozostają osobnymi
przypadkami implementacyjnymi dla kolejnych render/attachment routes.
