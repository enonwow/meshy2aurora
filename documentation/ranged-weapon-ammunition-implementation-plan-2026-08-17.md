# Plan implementacji: amunicja i pociski dla broni dystansowych

**Data:** 2026-08-17

**Status:** etapy inżynieryjne 0–10 zaimplementowane i zweryfikowane offline; dokładny kandydat demo MOD/HAK został zamrożony, zainstalowany i zweryfikowany bajt w bajt 2026-08-18; wizualny test właściciela w NWN pozostaje niewykonany

**Zakres wersji V1:** natywne kanały amunicji NWN: Arrow, Bolt i Bullet; własny przedmiot amunicji, własny model lecącego pocisku, istniejąca animacja strzału postaci oraz kompletne pakowanie do HAK/MOD.

## 0. Stan implementacji 2026-08-17

Zaimplementowana ścieżka V1 wykonuje jedną atomową transakcję:

`BaseItem broni -> BaseItem amunicji -> AmmunitionType -> DamageRangedProjectile -> sześciowierszowy blok ammunitiontypes.2da -> model pocisku -> clip postaci`.

Gotowe są:

- kontrakty i walidacja kanałów `Arrow`, `Bolt` i `Bullet`;
- odczyt i zapis `AmmunitionType` oraz raport semantyczny trasy runtime;
- pełny blok sześciu wierszy `ammunitiontypes.2da`, wraz z kolizjami i readbackiem;
- kontrolowana aktualizacja istniejącego, audytowanego wiersza `DamageTypes.2DA` dla wariantu pocisku;
- osobny profil konwersji GLB lecącego pocisku z jawną osią źródłową, ręczną rotacją do Aurora `+Y`, skalą, binarnym MDL i TGA;
- osobny UTI stosu amunicji, model/tekstura/ikona przedmiotu oraz walidacja natywnego nazewnictwa zasobów;
- interfejs „Amunicja i pocisk”, worker/WASM, HAK i fixture MOD zawierający dokładną broń i stos amunicji;
- raporty odczytu kontrolnego, które nie oznaczają testów offline jako sukcesu runtime.

Pierwszy dokładny kandydat demo został zmaterializowany przez rzeczywisty Worker/WASM jako `m2aimod35.mod` + `m2aihak35.hak`. Moduł zawiera dwa pickupy: broń i stos amunicji, a także nieruchomy cel treningowy `m2arngtarget`. Odczyt kontrolny MOD potwierdza `groundItemCount=2`, `creatureCount=1`, `WalkRate=0`, trzynaście pustych resrefów skryptów AI i pozycje wszystkich obiektów. Ten wynik jest dowodem offline struktury pakietu, nie dowodem widoczności ani działania strzału w NWN.

W V1 dedykowany wariant Hextech korzysta z istniejącego typu obrażeń `Divine` (`DamageTypes.2DA` row `6`, właściwość UTI subtype `8`) i przypisuje mu własny `DamageRangedProjectile` o identyfikatorze co najmniej `6`. Implementacja nie dopisuje nowego bazowego typu obrażeń do całego łańcucha tabel silnika. Taki rozszerzony tryb pozostaje osobnym zakresem po dowodzie runtime obecnej trasy.

Interfejs domyślnie pozostawia funkcję wyłączoną. Włączenie jej dla broni dystansowej wymaga obu aktualnych tabel 2DA i GLB pocisku; niespójna para kanału, BaseItemu i `AmmunitionType`, częściowy blok 2DA albo model skierowany inaczej niż Aurora `+Y` blokują build.

## 1. Cel produktu

Studio ma pozwalać zdefiniować dla dowolnej obsługiwanej broni dystansowej:

1. natywny kanał amunicji: `Arrow`, `Bolt` albo `Bullet`;
2. osobny przedmiot amunicji zapisany jako UTI, wraz ze stosem, nazwą, opisem, ikoną i modelem;
3. własny model lecącego pocisku oraz jego tekstury;
4. istniejący, audytowany typ obrażeń powiązany z własnym wariantem pocisku i efektu trafienia;
5. dźwięk wystrzału i trafienia;
6. istniejącą animację postaci `bowshot` albo `xbowshot`;
7. deterministyczny pakiet HAK/MOD z pełnym odczytem kontrolnym wszystkich wygenerowanych zasobów.

Wersja V1 nie tworzy czwartego natywnego slotu amunicji. `ammunitiontypes.2da` rozszerza warianty pocisku i obrażeń używane przez trzy kanały silnika, ale nie dodaje nowego bazowego kanału ekwipunku.

Concept art służy wyłącznie jako materiał źródłowy i porównawczy podczas przygotowania modelu. Nie może być automatycznie dołączany do interfejsu, HAK ani MOD.

## 2. Stan obecny i zidentyfikowane braki

| Obszar | Stan | Wymagana zmiana |
|---|---|---|
| Samodzielny typ bazowy broni | istnieje | zachować tworzenie bez klonowania tożsamości donora |
| Ogólne generowanie UTI, 2DA, binarnego MDL, HAK i MOD | istnieje | rozszerzyć o kontrakt amunicji i pocisku |
| Kanoniczny model Hextech Shell | istnieje | użyć przez manifest i hash, bez kopiowania do drugiego katalogu źródłowego |
| `AmmunitionType` w modelu domenowym `baseitems.2da` | zaimplementowany | parser, walidacja, zapis i semantic readback są częścią transakcji |
| Dedykowany kontrakt amunicji | zaimplementowany | profile broni, amunicji, pocisku i proof identity są wersjonowane |
| Writer `ammunitiontypes.2da` | zaimplementowany | zapisuje pełny blok sześciu wierszy i blokuje luki, częściowe bloki oraz kolizje |
| Powiązanie wariantu obrażeń | zaimplementowane dla audytowanego Divine | writer patchuje tylko oczekiwaną etykietę i wartość `0` albo identyczną; inne dane są kolizją |
| Trasa animacji dla własnego BaseItem | zaimplementowana offline | `WeaponWield=5` wybiera `bowshot`, `WeaponWield=6` wybiera `xbowshot`; runtime wymaga proof właściciela |
| Własny recoil/pompowanie modelu broni | brak | pozostawić poza V1; zaplanować osobno jako animację samego modelu broni |

Receptura samodzielnego Hextech Shotgun została poprawiona z niespójnej trasy Bolt na atomową trasę Bullet: `RangedWeapon=27` oraz `AmmunitionType=3`. Nie wystarczy zmienić tylko jednego pola i walidator odrzuca taką próbę.

## 3. Kontrakty danych

### 3.1. `RangedWeaponProfileV1`

Profil broni powinien zawierać co najmniej:

- `ammoChannel`: `ARROW | BOLT | BULLET`;
- `ammoBaseItem`: odpowiednio `20 | 25 | 27`;
- `ammunitionType`: odpowiednio `1 | 2 | 3`;
- `wielderClip`: `BOWSHOT | XBOWSHOT`;
- `projectileDefinitionId`;
- `damageTypeRow` oraz przypisany identyfikator `DamageRangedProjectile`;
- identyfikatory dźwięku strzału i, jeżeli wymagany, trafienia.

Kanał, bazowy typ przedmiotu amunicji i `AmmunitionType` tworzą jeden atomowy wybór. API i UI nie mogą pozwolić zapisać niespójnej kombinacji.

### 3.2. `AmmunitionDefinitionV1`

Definicja amunicji powinna zawierać:

- nazwę, opis, tag i resref UTI;
- wielkość generowanego stosu;
- natywny kanał amunicji;
- identyfikator definicji pocisku;
- ikonę oraz opcjonalny model przedmiotu leżącego w świecie;
- listę właściwości UTI;
- sposób zadawania obrażeń i efekt trafienia;
- politykę kompatybilności z bronią.

W V1 różne warianty amunicji korzystające z tego samego natywnego kanału pozostają technicznie wymienne. Wymuszenie, aby konkretna broń akceptowała wyłącznie jeden własny wariant stosu, wymaga osobnego zakresu skryptowego lub NWNX.

### 3.3. `ProjectileDefinitionV1`

Definicja lecącego pocisku powinna zawierać:

- ścieżkę do kanonicznego GLB przez `sample-3d/<asset-id>/manifest.yaml`;
- oczekiwany SHA-256 źródła;
- resref modelu, tekstur i dźwięków;
- identyfikator `DamageRangedProjectile`;
- skalę, oś lotu, punkt początku i transformację korekcyjną;
- politykę materiału, w tym base color, metallic/roughness, normal i emissive;
- tryb obrażeń i efekt trafienia.

Identyfikatory i resrefy są przydzielane deterministycznie, sprawdzane pod kątem wartości zarezerwowanych oraz blokowane przy kolizji. Wartości zostały potwierdzone w lokalnych zasobach aktualnej wersji EE podczas etapu 0.

## 4. Etapy implementacji

### Etap 0 — kontrakt Aurora First

Etap wykonano na lokalnych, aktualnych zasobach EE. Bazowe tabele są rozdzielone między `nwn_base.key` i aktualne override'y `nwn_retail.key`. Potwierdzony kształt `ammunitiontypes.2da` to:

`label Model ShotSound ImpactSound AmmunitionType DamageRangedProjectile`.

Potwierdzone bloki wariantów obejmują identyfikatory obrażeń `0..5`, po sześć wierszy każdy; następny kompletny blok zaczyna się w wierszu `36`. W bloku kanał Bullet używa offsetu `2`, czyli wiersza `38`. `DamageTypes.2DA` zawiera kolumnę `DamageRangedProjectile`, a audytowany wiersz Divine `6` ma wartość źródłową `0`.

Zebrany kontrakt obejmuje:

- odczytu `RangedWeapon` i `AmmunitionType` z `baseitems.2da`;
- klucza i sposobu wyboru wiersza `ammunitiontypes.2da`;
- wyboru modelu, orientacji, początku, skali i dźwięków lecącego pocisku;
- wyboru `bowshot`/`xbowshot` oraz momentu wypuszczenia pocisku;
- dokładnej postaci odpowiednich detalicznych wierszy 2DA w lokalnej instalacji EE.

Każde ustalenie ma zostać oznaczone jako fakt, dowód pośredni albo hipoteza. Hipoteza wpływająca na format generowanych zasobów musi otrzymać fixture i test przed rozpoczęciem właściwego writera.

### Etap 1 — schemat domenowy i testy kontraktowe

Najpierw powstają testy, następnie implementacja:

- serializacja i deserializacja trzech nowych kontraktów;
- poprawne mapowanie wszystkich trzech natywnych kanałów;
- odrzucenie niespójnych zestawów `RangedWeapon`/`AmmunitionType`;
- odrzucenie brakującego manifestu, niezgodnego hasha, kolizji resrefów i wartości zarezerwowanych;
- stabilność serializowanego formatu i jawne wersjonowanie schematu.

### Etap 2 — obsługa kanału w `baseitems.2da`

- dodać `AmmunitionType` do parsera, modelu, writera i semantic readback;
- zmienić recepturę Hextech Shotgun na `RangedWeapon=27` i `AmmunitionType=3`;
- udostępnić ten wybór jako ogólny profil, a nie wyjątek zaszyty dla jednej broni;
- rozszerzyć raport trasy runtime o wynikający z pól semantycznych kanał oraz clip postaci;
- nie przywracać klonowania tożsamości bazowego przedmiotu detalicznego.

### Etap 3 — generowanie przedmiotu amunicji UTI

- utworzyć UTI stosu na natywnym BaseItem `20`, `25` albo `27`;
- zapisać nazwę, opis, tag, resref, rozmiar stosu i `PropertiesList`;
- wygenerować lub dołączyć ikonę oraz model przedmiotu ekwipunkowego/światowego;
- sprawdzić odczytem semantycznym, że amunicja trafia do właściwego natywnego kanału;
- objąć testem trzy kanały oraz negatywny przypadek broni i stosu z różnych kanałów.

### Etap 4 — profil konwersji lecącego pocisku

Należy dodać osobny profil wyjściowy dla pocisku, niezależny od profilu modelu przedmiotu:

- pobranie GLB wyłącznie z kanonicznego `sample-3d`;
- sanityzacja degeneratów bez zmiany zamierzonej geometrii;
- jawna korekta osi lotu, skali i punktu początku;
- generowanie binarnego MDL i wymaganych tekstur;
- obsługa base color, metallic/roughness, normal i emissive zgodnie z możliwościami materiału Aurora;
- binarny readback modelu, hierarchii, geometrii, UV i przypiętych tekstur;
- fixture potwierdzający, że przód pocisku wskazuje kierunek lotu, a nie bok lub tył.

Obowiązuje wspólny limit produktu `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000`. Nie wolno wprowadzać osobnego, niższego limitu blokującego tylko dla pocisków. Niezależna granica pojedynczego strumienia binarnego MDL pozostaje bramą writera i w razie przekroczenia wymaga deterministycznego podziału geometrii. Kanoniczny Hextech Shell ma 1536 trójkątów i mieści się z dużym zapasem w obu granicach.

### Etap 5 — writery 2DA pocisku i obrażeń

- zaimplementowano append pełnego bloku dla `ammunitiontypes.2da`;
- wiersze są przydzielane deterministycznie i istniejąca inna zawartość pod tym samym identyfikatorem jest kolizją;
- zapis obejmuje model, oba dźwięki, `AmmunitionType` i `DamageRangedProjectile`;
- semantic readback obejmuje każdy z sześciu wierszy, a nie tylko wybrany kanał;
- patch `DamageTypes.2DA` wymaga dokładnej etykiety `Divine` oraz źródłowej wartości `0` albo identycznego żądanego identyfikatora;
- dowolny nowy bazowy typ obrażeń wymagający rozszerzenia kolejnych tabel nie jest częścią V1.

### Etap 6 — animacja strzału

W V1:

- postać używa istniejącego clipu `bowshot` albo `xbowshot`;
- lecący pocisk jest prowadzony przez mechanizm silnika, a nie przez osobny ręcznie animowany klip MDL;
- raport kompilacji wskazuje wybrany clip i wymagany standard humanoida/supermodelu;
- fixture potwierdza prawidłowy wybór clipu z profilu broni.

Własny recoil, ruch zamka, pompowanie dolnego modułu lub obrót rdzenia broni jest osobnym zakresem V2. Nie może być przedstawiany jako gotowy w ramach V1.

### Etap 7 — interfejs Studio

Sekcja „Amunicja i pocisk” powinna pojawiać się tylko dla broni dystansowej i umożliwiać:

- wybór natywnego kanału;
- utworzenie albo wybranie definicji amunicji;
- wskazanie kanonicznego źródła GLB pocisku;
- wybór istniejących lub własnych obrażeń;
- wybór dźwięków oraz clipu postaci;
- ustawienie skali i jawnej korekty osi modelu;
- walidację przed wysłaniem zadania do workera.

Ekran podsumowania ma pokazać pełne mapowanie:

`BaseItem broni -> BaseItem stosu -> AmmunitionType -> DamageRangedProjectile -> model/dźwięki -> clip postaci`.

Concept art nie jest elementem wynikowego ekranu ani pakietu.

### Etap 8 — worker, pakowanie i fixture modułu

Worker powinien wygenerować w jednej deterministycznej transakcji:

- zmieniony lub nowy wiersz broni w `baseitems.2da`;
- UTI broni oraz UTI stosu amunicji;
- model i tekstury pocisku;
- wymagane tabele 2DA;
- pozostałe zasoby broni;
- HAK i testowy MOD.

Każdy zasób ma przejść odczyt kontrolny przed zapakowaniem, a gotowe HAK/MOD — kontrolę spisu i hashy po zapakowaniu. Kolizja istniejącego zasobu ma przerwać zadanie, nie powodować nadpisania ani cichego przydzielenia innej tożsamości.

Fixture modułu musi dostarczać postaci właściwą broń i znany stos amunicji oraz ustawić cel w czytelnej odległości i kierunku. Sam fixture nie stanowi dowodu wizualnego działania.

### Etap 9 — testy regresji i integracji

Minimalny zestaw:

- testy jednostkowe Rust kontraktów, walidatorów, alokatorów i writerów;
- golden tests wynikowych wierszy 2DA, UTI i binarnego MDL;
- testy negatywne kolizji, złego hasha, złej pary kanałów, zarezerwowanych ID i nieprawidłowej osi;
- testy Arrow, Bolt i Bullet;
- syntetyczny mały fixture do CI oraz warunkowy test dokładnego lokalnego Hextech Shell;
- testy WASM/workera i przepływu Studio;
- regresja istniejących tras łuku, kuszy, broni oraz zwykłych itemów;
- pełny typecheck, build, clippy, testy repozytorium, `git diff --check` i `assert-meshy-asset-layout.ps1`.

### Etap 10 — zamrożenie kandydata i handoff właścicielski

Po przejściu testów należy zamrozić jedną dokładną linię kandydata i zapisać hashe wszystkich źródeł i wyników. Handoff musi zaczynać się od:

1. dokładnej nazwy pliku testowego `.mod`;
2. nazwy modułu widocznej w Toolset;
3. dokładnej nazwy Area;
4. dalej: obiekt, broń, stos amunicji, HAK, BaseItem, wiersze 2DA i położenie testowe.

Agent kończy na stanie `ready_for_owner_proof`. Właściciel wykonuje i ocenia finalny test w Aurora Toolset/NWN zgodnie z aktualną polityką proof. Agent nie deklaruje wizualnego sukcesu na podstawie testów offline ani samego istnienia plików.

## 5. Kryteria zakończenia

### 5.1. Kryteria funkcjonalne

- Studio pozwala wybrać dokładnie jeden z kanałów Arrow, Bolt lub Bullet dla każdej obsługiwanej broni dystansowej.
- Niespójna kombinacja kanału, BaseItem stosu i `AmmunitionType` jest blokowana przed generowaniem.
- Pipeline generuje osobne UTI stosu amunicji z poprawną nazwą, resrefem, wielkością stosu i właściwościami.
- Pipeline generuje własny model lecącego pocisku z materiałem i teksturami, nie zastępuje go modelem amunicji detalicznej.
- Wystrzał korzysta z przypisanego istniejącego clipu `bowshot` albo `xbowshot`.
- Własny wariant pocisku jest wiązany wyłącznie z audytowanym istniejącym typem Divine; V1 nie deklaruje dowolnego nowego bazowego typu obrażeń.
- Dokumentacja nie twierdzi, że powstał czwarty natywny slot ani że bez NWNX wymuszono wyłączne użycie jednego wariantu stosu w ramach wspólnego kanału.
- Concept art nie znajduje się w wynikowym UI, HAK ani MOD.

### 5.2. Kryteria techniczne

- `AmmunitionType` jest parsowany, zachowywany, zapisywany i potwierdzany semantic readbackiem.
- Hextech Shotgun w trybie Bullet ma jednocześnie `RangedWeapon=27` i `AmmunitionType=3`.
- Samodzielny BaseItem nie dziedziczy ani nie klonuje tożsamości donora detalicznego.
- Wszystkie modyfikacje 2DA są deterministyczne, odporne na kolizje i sprawdzają wartości zarezerwowane.
- Binarne MDL przechodzi readback geometrii, UV, materiałów, hierarchii, tekstur i transformacji osi lotu.
- Nie wprowadzono limitu trójkątów sprzecznego ze wspólnym budżetem 300 000; granica pojedynczego strumienia jest obsługiwana przez writer.
- HAK i MOD mają deterministyczny spis zasobów, hashe i pozytywny odczyt kontrolny.
- Istniejące trasy łuku, kuszy i pozostałych itemów przechodzą regresję bez zmiany zachowania.

### 5.3. Kryteria jakości

- Wszystkie nowe zachowania mają test napisany przed lub razem z implementacją oraz co najmniej jeden test negatywny.
- Testy obejmują Arrow, Bolt i Bullet, a nie tylko Hextech Shell.
- Przechodzą: testy Rust, testy web/workera, typecheck, build, clippy, `git diff --check` i guard layoutu Meshy.
- Nie ma niejawnej drugiej biblioteki modeli źródłowych ani referencji produktu do `test-assets/meshy`.
- Dokumentacja kontraktu, ograniczeń i procedury handoff jest aktualna.

### 5.4. Kryteria właścicielskiego proof

Etap wizualny może zostać zamknięty dopiero po raporcie właściciela z dokładnego, zamrożonego kandydata, że w NWN:

- broń przyjmuje właściwy natywny stos amunicji;
- liczba amunicji maleje po strzale;
- własny model pocisku jest widoczny w locie;
- pocisk ma poprawną skalę, początek i orientację — leci przodem, bez obrotu bokiem lub tyłem;
- postać wykonuje wybrany clip strzału;
- odtwarzają się właściwe dźwięki i efekt trafienia;
- nie występuje placeholder, model detaliczny zamiast własnego ani niewłaściwy wariant pocisku.

Toolset może potwierdzić zasoby, itemy i konfigurację, lecz nie zastępuje testu pocisku w locie. Ten element wymaga NWN runtime.

### 5.5. Wynik weryfikacji implementacji

Na stanie po remediacji audytu z 2026-08-19:

- `cargo test -p m2a-core --test item`: **58/58 passed**;
- testy jednostkowe Studio: **274 passed, 1 skipped**;
- `npm run typecheck`: **passed**;
- `npm run build`, łącznie z release build WASM: **passed**;
- ukierunkowany zestaw prawdziwego workera przeglądarkowego i WASM, obejmujący transakcję Bullet, zachowanie niezależnej ścieżki legacy oraz negatywną bramkę zarezerwowanej tożsamości 113: **5/5 passed**;
- `cargo fmt --all -- --check`, `git diff --check`, guard kanonicznego worktree i `assert-meshy-asset-layout.ps1`: **passed**;
- `cargo clippy -p m2a-core -p m2a-wasm --all-targets`: **passed**; wyjście nadal zawiera wcześniejsze ostrzeżenia bibliotek, fixture'ów i przykładów, dlatego nie jest deklarowany czysty przebieg z `-D warnings`;
- `cargo test -p m2a-wasm`: **32 passed, 2 failed**, ponieważ lokalny, ignorowany payload `sample-3d/h2-clockwork-sentinel-1500/source.glb` nie jest dostępny; oba błędy należą do wcześniejszej trasy proceduralnego creature i nie dotyczą itemów, amunicji ani pocisku;
- pełne zbieranie browser integration ma również zastane blokery nieobecnych lokalnych fixture'ów `local-reference-assets/appearance.2da` i `sample-3d/h2-clockwork-sentinel-1500/source.glb`; ukierunkowany test nowej transakcji przechodzi.

Zamrożono dokładny kandydat z właściwą bronią, kanonicznym źródłem Hextech Shell i nieruchomym celem: MOD SHA-256 `3b0305aaf1997a72b1efdaa58fdf86af486e438cc9709e24355c2d9425560ef2`, HAK SHA-256 `dd4f269cfea4867490ca19c49b23f48c7632c4d0686f39db0ddc3d176c426d55`. Pipeline raportuje `OFFLINE_ITEM_PACKAGE_PASSED` oraz zachowuje uczciwe `modelVisibility=not_tested`, `proofCompleteness=missing`. Dokładną parę zainstalowano następnie w natywnych katalogach NWN z ochroną przed nadpisaniem i potwierdzono identyczność SHA-256 źródło→cel. Stan agentowy wynosi teraz `ready_for_owner_proof`; właścicielski test wizualny nadal nie został wykonany.

## 6. Definicja Done

Implementacja inżynieryjna opcji jest ukończona offline w zakresie etapów 0–9. Cała funkcja produktu, łącznie z dowodem runtime, jest ukończona dopiero wtedy, gdy:

1. etapy 0–9 są zaimplementowane i wszystkie kryteria techniczne oraz jakościowe przechodzą;
2. zamrożono dokładnie jednego kandydata z kompletem hashy i handoffem;
3. stan agenta wynosi `ready_for_owner_proof`;
4. właściciel potwierdził kryteria runtime dla tej samej linii MOD/HAK/model/2DA;
5. nie pozostała żadna hipoteza na krytycznej ścieżce implementacji;
6. ewentualna kolejna iteracja powstaje wyłącznie po świeżym, związanym z kandydatem wyniku `modelVisibility=not_visible`, zgodnie z bramą iteracji projektu.

## 7. Poza zakresem V1

- czwarty natywny kanał lub slot amunicji;
- wymuszenie konkretnego wariantu stosu przez NWNX lub skrypty;
- własne animacje supermodelu postaci;
- recoil, pompowanie, zamek i animowane mechanizmy samej broni;
- rozbudowana fizyka balistyczna;
- prezentowanie concept artu w aplikacji;
- wykonywanie finalnego live proof przez agenta.
