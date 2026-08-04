# Audyt naprawczy pipeline Item po wyniku V8

Data: 2026-08-04
Status: `IMPLEMENTACJA_OFFLINE_ZAKONCZONA / READY_FOR_OWNER_PROOF`
Zakres: `ModelType 2`, istniejący BaseItem `1 / WSwLs`, Item Properties oraz
item-only demo. Creature/equipped witness jest poza zakresem tej poprawki.

## 1. Wynik

V8 nie może być zaakceptowane. Właściciel potwierdził trzy oddzielne błędy:

1. model jest renderowany, ale pola `Model` dla Bottom/Middle/Top są puste;
2. `Middle` jest nieproporcjonalnie szeroki jak na jednoręczny longsword;
3. creature nazwany `Equipped Item model render witness` nie pokazuje
   poprawnie postaci ani wyposażonego miecza.

Wynik właściciela oraz hashe obrazów są zapisane w:

`documentation/evidence/tlc-guard-longsword-v8-owner-reference-profile-result-2026-08-04.json`.

Poprawka nie będzie modyfikować immutable packetu V8. Powstanie najpierw
implementacja i testy pipeline, a dopiero potem jeden nowy kandydat.

## 2. Fakty z Aurory i retail

### 2.1 Selektor ModelType 2

Fakt z dekompilacji Aurory:

- `C:\Projects\New Folder\export\decompiled_all.c`, okolice
  `FUN_004c03e8` (`131357..131473`), pobierają `MinRange` i `MaxRange` z
  wybranego rekordu BaseItem;
- Aurora przechodzi po bucketach modelu krokiem `10`, sprawdza zasoby ikon i
  dodaje odnalezione modele do combo;
- wartość UTI jest rozdzielana na model `ModelPart - ModelPart % 10` i kolor
  `ModelPart % 10`; widoczny numer modelu to bucket podzielony przez `10`;
- `C:\Projects\New Folder\export\strings.tsv` potwierdza nazwy kolumn
  `MinRange` i `MaxRange`.

Fakt z retail `baseitems.2da`:

- BaseItem `1`, `ItemClass=WSwLs`, `ModelType=2`;
- `MinRange=10`, `MaxRange=100`;
- oznacza to modele wyświetlane jako `1..10`, a nie `10..100`.

Fakt z V8:

- UTI zawiera `253/253/253`;
- bucket modelu wynosi `250`, a widoczny model wynosi `25`;
- HAK zawiera `wswls_{b,m,t}_251..254` oraz odpowiadające warstwy ikon;
- bezpośredni renderer znajduje te zasoby, dlatego miecz i ikona są widoczne;
- bucket `250` nie mieści się w retailowym `10..100`, dlatego Aurora nie ma
  pozycji combo, którą mogłaby zaznaczyć.

### 2.2 Obwiednia Middle

Fakt z reportu V8 `fit-report.json`:

| Oś | V8 Middle | Retail `wswls_m_063` | Stosunek rozmiaru |
|---|---:|---:|---:|
| X / depth | `0.1037794` | `0.0518068` | około `2.00x` |
| Z / width | `0.4552096` | `0.1947293` | około `2.34x` |

Fakt z kodu:

- `fit_meshy_item_parts_to_attachment_profile_v1` wylicza jeden
  `uniform_scale` wyłącznie z długości osi Y;
- następnie dopasowuje środki X/Z, ale nie ich rozmiary;
- `validate_item_fit_report_v4_against_profile_v1` sprawdza zakres Y i środki
  X/Z, lecz nie sprawdza poprzecznych spanów X/Z;
- `PASSED` V8 potwierdza więc położenie osiowe i seam overlap, ale nie
  proporcje jednoręcznej broni.

### 2.3 Creature witness

Fakt z kodu produktu:

- `requiresEquippedItemProof` wymaga osobnego proofu wyposażenia tylko dla
  `CAPART_COMPOSITE` i `CLOAK_MODEL`;
- zwykły `WSwLs` nie wymaga creature do itemowego vertical slice;
- creature V8 został dodany ręcznie w materializerze demo, poza tym
  kontraktem.

Fakt z wyniku właściciela:

- oczekiwany miecz nie jest widoczny na creature;
- geometria creature jest zwinięta w kulę;
- ten obiekt nie jest dowodem poprawnego chwytu i nie może pozostać w demo.

## 3. Braki w obecnym pipeline

### 3.1 Braki danych BaseItem

`ItemBaseItemV1` nie zawiera `minRange` ani `maxRange`, mimo że Aurora używa
ich do enumeracji modeli. Studio oraz allocator nie mogą więc sprawdzić, czy
wybrany model będzie widoczny w combo Toolsetu.

### 3.2 Błąd allocatora

Allocator ModelType 2:

- próbuje modele modulo `26`;
- sprawdza kolizje MDL/TGA;
- nie sprawdza bucketu względem `MinRange/MaxRange`;
- nie tworzy jawnego planu rozszerzenia zakresu wybranego BaseItem.

Sama nieobecność resrefu w retail nie wystarcza. Nowy model musi być także
enumerowalny przez dokładny efektywny `baseitems.2da`.

### 3.3 Brak bezpiecznej poprawki istniejącego wiersza 2DA

Wspólny writer 2DA potrafi czytać, klonować i dopisywać wiersze. Nie ma jeszcze
operacji zmiany jednej komórki istniejącego wiersza z readbackiem zachowania
pozostałych komórek. Jest ona potrzebna, gdy nie ma wolnego modelu w obecnym
zakresie i trzeba podnieść `MaxRange` istniejącego BaseItem bez tworzenia
nowego BaseItem.

### 3.4 Brak target-space transverse fit

`ItemPartTransformV1` ma tylko skalę uniform. Do zachowania referencyjnego
zakresu Y i jednoczesnego ograniczenia przekroju X/Z potrzeba osobnego,
automatycznie obliczanego skalowania płaszczyzny poprzecznej po obrocie do
ramy Aurory.

Skalowanie ma być wspólne dla X i Z, aby nie zmieniać proporcji przekroju
parta. Dla każdego slotu:

```text
transverseScale = min(
  1,
  referenceSpanX / generatedSpanX,
  referenceSpanZ / generatedSpanZ
)
targetScaleXYZ = [transverseScale, 1, transverseScale]
```

Oś Y zachowuje niezależny fit osiowy. Po skalowaniu translacja ponownie ustawia
środek X/Z i referencyjny zakres Y.

## 4. Docelowa poprawka pipeline

### 4.1 Kontrakt zakresu modelu

1. Parser zapisuje `minRange` i `maxRange` w `ItemBaseItemV1`.
2. Dla ModelType 2 wartości są interpretowane jako zakodowane buckety.
3. Allocator szuka atomowo jednego modelu dla wszystkich trzech partów i
   czterech kolorów.
4. Najpierw sprawdza collision-free luki w bieżącym zakresie.
5. Jeśli nie ma luki, szuka najmniejszego wolnego modelu powyżej aktualnego
   maksimum, nie przekraczając modelu `25` / wartości `254`.
6. Rozszerzenie tworzy plan zmiany wyłącznie `MaxRange` wybranego wiersza.
7. Wygenerowany `baseitems.2da` jest pakowany do HAK-a tylko wtedy, gdy zakres
   faktycznie trzeba rozszerzyć.
8. Readback wymaga, aby wszystkie inne wiersze i komórki zachowały znaczenie,
   a wybrany bucket był w efektywnym zakresie.

Nie tworzymy nowego BaseItem. Nadal używany jest istniejący BaseItem `1`.

### 4.2 Kontrakt poprzeczny partów

1. Profil referencyjny nadal określa wspólny origin, ramę Y/Z/X i sloty.
2. Fit osiowy ustala długość i położenie Y.
3. Nowy transverse fit ogranicza X/Z do obwiedni wybranego referencyjnego
   parta, bez automatycznego powiększania mniejszej geometrii.
4. Transformacja jest bake'owana w geometrii w target space; root i Trimesh
   zachowują kontrakt Aurory.
5. Normale używają inverse-transpose, tangenty są ponownie normalizowane i
   ortogonalizowane względem normali, a handedness tangenta jest zachowany.
6. Raport zapisuje skalę poprzeczną, wejściowe i wyjściowe spany oraz wynik
   gate'u dla każdego slotu.
7. Status `PASSED` wymaga jednocześnie: frame, zakres Y, center X/Z, envelope
   X/Z, seam overlap i brak niedozwolonego non-adjacent overlap.

### 4.3 Kontrakt demo

Następne demo zawiera:

- jeden prawdziwy Item `Last City Bronze Guard Longsword`;
- UTI z `BaseItem=1`, trzema ModelPart, kolorem `3` i `Identified=1`;
- HAK z kompletną macierzą zasobów model/kolor/ikona;
- opcjonalny, wygenerowany `baseitems.2da`, jeśli wymaga tego zakres;
- item-only Area do oceny Item Properties oraz ground Item.

Następne demo nie zawiera:

- creature;
- UTC;
- `CreatureList` witnessa;
- deklaracji, że pipeline rozwiązał chwyt broni przez postać.

Obsługa wyposażenia i chwytu jest osobnym przyszłym case'em.

## 5. Plan implementacji

### Etap 1 — TDD danych BaseItem

- dodać testy parsera `MinRange/MaxRange`, w tym retail-gated test dla
  `WSwLs = 10/100`;
- rozszerzyć Rust, WASM i typy Studio o oba pola;
- dodać walidację semantyki bucketów ModelType 2 i błędnych zakresów.

### Etap 2 — TDD bezpiecznej zmiany 2DA

- dodać writer zmiany wskazanej komórki istniejącego fizycznego wiersza;
- wymagać jawnego source SHA, numeru wiersza, nazwy kolumny i oczekiwanej
  starej wartości;
- zachować newline, kolejność kolumn, etykiety i wszystkie niezmieniane
  komórki;
- wykonać pełny readback i raport pojedynczej zmiany;
- dodać Item helper zmieniający wyłącznie `MaxRange` wybranego BaseItem.

### Etap 3 — allocator enumerowalnego modelu

- zastąpić modulo `26` planowaniem względem `minRange/maxRange`;
- sprawdzać atomowo 3 sloty x 4 kolory dla MDL i ikon;
- zwracać `effectiveMinRange`, `effectiveMaxRange`, wybrany bucket oraz
  informację, czy wymagany jest override 2DA;
- blokować build, gdy nie istnieje collision-free i kodowalny model;
- pokazać plan rozszerzenia zakresu w Studio przed generowaniem.

### Etap 4 — transverse fit i geometry bake

- najpierw dodać test reprodukujący błędne spany Middle V8;
- wprowadzić wersjonowany raport fit z `transverseScale` i gate'em X/Z;
- zastosować target-space `[s,1,s]` po orientacji, przed ostateczną translacją;
- poprawnie przeliczyć pozycje, normale i tangenty;
- rozszerzyć bounds, seam i composer conformance o nową transformację;
- zachować triangle count, UV, materiały i tekstury.

### Etap 5 — pakowanie HAK i Studio

- pakować wygenerowany `baseitems.2da` jako resref `baseitems`, typ `2017`,
  tylko przy rzeczywistym rozszerzeniu;
- uwzględnić jego hash i readback w manifeście artifactu;
- pokazać w UI: zakres źródłowy, wybrany model, zakres efektywny, status
  override oraz wyniki obwiedni per slot;
- blokować Download, jeśli model nie będzie enumerowalny albo envelope nie
  przechodzi.

### Etap 6 — item-only materializer

- usunąć tworzenie i pakowanie witnessa UTC z longsword demo;
- wygenerować Area z jednym prawdziwym Itemem;
- sprawdzić offline MOD/HAK/UTI/2DA oraz brak creature resources;
- utworzyć jeden nowy immutable candidate dopiero po zielonych testach.

### Etap 7 — handoff właścicielowi

- zahashować exact MOD i HAK;
- zainstalować exact artefakty zgodnie z regułą absent-target i potwierdzić
  byte-identical źródło/cel;
- przekazać najpierw nazwę `.mod`, następnie Toolset module name i Area;
- właściciel zamyka wizualny proof Item Properties.

Agent nie uruchamia ani nie steruje Toolsetem lub NWN.

## 6. Kryteria ukończenia

### A. BaseItem i selektory

- [ ] `ItemBaseItemV1` zwraca dla retail WSwLs `minRange=10` i
  `maxRange=100`.
- [ ] Każdy wybrany ModelType 2 ma bucket w efektywnym zakresie.
- [ ] Wszystkie 12 kombinacji slot x kolor są collision-free i istnieją w
  pakiecie.
- [ ] Jeśli wymagany jest override, HAK zawiera `baseitems.2da` typu `2017`.
- [ ] Readback override'u wykazuje dokładnie jedną dozwoloną zmianę:
  `BaseItem 1 / MaxRange`; pozostałe dane są semantycznie identyczne.
- [ ] W Item Properties pola Top/Middle/Bottom pokazują numer modelu, nie są
  puste, i po zmianie koloru nadal wskazują kompletne zasoby.

Ostatni punkt jest kryterium wizualnym właściciela w Aurorze.

### B. Geometria jednoręcznego WSwLs

- [ ] Każdy slot zachowuje referencyjny zakres Y i środek X/Z w tolerancji
  `1e-5`.
- [ ] Span X i Z żadnego slotu nie przekracza wybranej referencyjnej
  obwiedni o więcej niż tolerancję numeryczną.
- [ ] Dla obecnego Middle span X wynosi najwyżej `0.0518068`, a span Z
  najwyżej `0.1947293`.
- [ ] Skala poprzeczna jest skończona, dodatnia, nie większa niż `1` i
  identyczna dla X/Z.
- [ ] Triangle count wejścia i wyjścia jest identyczny; pipeline nie usuwa
  prawidłowych trójkątów.
- [ ] Normale i tangenty są skończone i znormalizowane, tangent pozostaje
  ortogonalny do normalnej, a UV i materiały są zachowane.
- [ ] Wszystkie adjacent seams przechodzą, a niedozwolony non-adjacent overlap
  nie występuje.
- [ ] Owner akceptuje szerokość jelca i złożenie partów w lewym viewportcie
  Item Properties.

### C. Item-only demo

- [ ] MOD zawiera jeden prawdziwy Item w `ItemList`.
- [ ] UTI ma `BaseItem=1`, trzy poprawne ModelPart, kolor `3` i
  `Identified=1`.
- [ ] MOD nie zawiera proof creature ani `CreatureList` witnessa.
- [ ] HAK nie zawiera UTC ani zasobów creature dodanych przez longsword demo.
- [ ] Ground Item i duży viewport Item Properties renderują ten sam poprawnie
  złożony miecz.
- [ ] Handoff nie deklaruje obsługi chwytu przez postać.

### D. Testy i zamrożenie

- [ ] Rust unit/integration, WASM, Studio unit i browser integration są
  zielone.
- [ ] `cargo fmt --check`, lint/typecheck oraz `git diff --check` przechodzą.
- [ ] Canonical workspace i Meshy asset layout gates przechodzą.
- [ ] Powstaje tylko jeden nowy candidate po V8, z pełnymi hashami źródeł,
  profilu, fitu, 2DA, MDL/TGA/UTI/HAK/MOD.
- [ ] Exact MOD/HAK są zainstalowane i po instalacji byte-identical.
- [ ] Status końcowy agenta to `ready_for_owner_proof`; `visualAcceptance`
  może zamknąć wyłącznie właściciel.

## 7. Granica tej poprawki

Ta implementacja naprawia generowanie i prezentację modularnego Itemu
ModelType 2. Nie naprawia creature, animacji ręki, skeleton attachment ani
wyposażenia postaci. Te elementy nie mogą być ponownie dodane do longsword demo
bez osobnego audytu, planu i kryteriów proofu.

## 8. Wynik implementacji 2026-08-04

Implementacja została zamknięta jednym kandydatem V9:
`tlc-guard-longsword-item-range-envelope-v9-20260804`.

- parser i Studio uwzględniają `MinRange/MaxRange`;
- allocator ModelType 2 wybiera numer w dozwolonej domenie i jawnie raportuje
  wymagany override;
- wersjonowany patcher 2DA zmienia dokładnie jedną oczekiwaną komórkę i
  fail-closed odrzuca mismatch wiersza lub starej wartości;
- HAK V9 zawiera `baseitems.2da` z jedyną zmianą `WSwLs MaxRange=250`;
- geometry bake obsługuje docelową skalę `[s,1,s]`, inverse-transpose normali i
  ponowną ortogonalizację tangentów;
- fitter ogranicza Bottom/Middle/Top do referencyjnych obwiedni X/Z i zachowuje
  referencyjne sloty Y;
- moduł ma profil `ITEM_ONLY_GROUND_ITEM_V1`, jeden Item i zero creatures;
- exact MOD/HAK są zainstalowane oraz byte-identical z zamrożonym źródłem.

Handoff właściciela:
`documentation/evidence/tlc-guard-longsword-item-range-envelope-v9-ready-for-owner-proof-2026-08-04.md`.

Kryteria offline są spełnione. Kryteria wymagające wyglądu w Item Properties
oraz Area pozostają otwarte do wyniku właściciela; agent nie uruchamiał Aurory
ani NWN.
