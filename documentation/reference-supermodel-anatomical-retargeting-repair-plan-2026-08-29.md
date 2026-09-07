# Plan naprawy automatycznego nakładania szkieletu supermodelu — 2026-08-29

## 1. Cel

Naprawić ogólną trasę reference-supermodel tak, aby wybrany dowolny supermodel
nie tylko przekazywał nazwy carrierów, hierarchię i animacje, lecz również
otrzymywał na modelu docelowym anatomicznie poprawne pivoty jointów oraz lokalny,
semantyczny skinning.

Końcowy kontrakt ma rozdzielać i oddzielnie dowodzić:

1. dokładną zgodność struktury z wybranym supermodelem;
2. poprawne rozpoznanie anatomii powierzchni docelowej;
3. poprawne dopasowanie pivotów i łańcuchów jointów;
4. poprawne przypisanie powierzchni do jointów;
5. poprawną deformację we wszystkich wymaganych klipach;
6. gotowość produktu do właścicielskiego proofu w Aurora Toolset i NWN.

`READY` nie może już oznaczać wyłącznie kompletności danych. Musi oznaczać, że
każdy wymagany warunek danego etapu został zmierzony i zaliczony.

## 2. Stan wyjściowy i potwierdzony problem

### Fakty z kodu i artefaktów

- Exact carrier topology, nazwy, parenty i klipy wybranego supermodelu są
  odczytywane i zachowywane.
- Aktualny joint fitter rozpoczyna od przeskalowania AABB, a następnie używa
  lokalnych centrów powierzchni, osi medialnej, terminali kontaktu z podłożem i
  prostego dopasowania appendage chain.
- `jointFit=READY` zależy obecnie od minimalnego confidence `>= 0.05`.
  Confidence jest pochodną bliskości i liczebności punktów powierzchni, a nie
  błędu względem jawnych landmarków anatomicznych.
- `surfaceAnatomy=READY` wymaga obecnie dostatecznej symetrii, co najmniej kilku
  punktów osi medialnej i niepustych kandydatów kontaktu z podłożem.
- Pokrycie skinningu dowodzi obecności dodatnich wag jointu, ale nie dowodzi,
  że jego klaster należy do poprawnego regionu ciała.
- Zamrożony diagnostyczny borzoj raportuje jednocześnie:
  `surfaceAnatomy=READY`, `jointFit=READY`, 30 carrierów, 907/907 komórek
  joint×clip oraz `motionQuality=BLOCKED` z 2328 błędnymi próbkami komponentów.
- Studio prawidłowo blokuje eksport, gdy `motionQualityStatus != PASS`, lecz
  nie zapobiega wcześniejszemu fałszywemu oznaczeniu anatomii i joint-fit jako
  gotowych.

### Wniosek implementacyjny

To nie jest brak supermodelu ani brak klipów. Brakuje twardej, mierzalnej
warstwy rejestracji anatomicznej pomiędzy exact carrier topology a generowaniem
wag. Walidator ruchu wykrywa skutek zbyt późno.

## 3. Nienaruszalne invarianty

1. **Dowolny supermodel.** Kod produkcyjny nie zawiera warunków po nazwie
   supermodelu, rasie ani gatunku. `c_wolf` może występować w testach i proofach,
   ale nie jako reguła algorytmu.
2. **Exact structure.** Nazwy, kolejność, parenty, klasy carrierów i wymagane
   kontrolery pochodzą wyłącznie z wybranego exact chain.
3. **Target-owned anatomy.** Pozycje pivotów i wagi są wyliczane dla geometrii
   docelowej; nie są ślepą kopią proporcji modelu referencyjnego.
4. **Brak cichego fallbacku.** Niepewny region, joint albo component kończy się
   `NEEDS_AUTHORING`/`BLOCKED`, a nie AABB-only, przypadkowym komponentem lub
   sztucznym zwiększeniem coverage.
5. **Jedno źródło decyzji.** Preview, authoring, produkt i eksport używają tego
   samego zahashowanego rig artifact i jednej centralnej decyzji admission.
6. **Diagnostyka nie jest produktem.** Niepoprawny rig może zostać pokazany do
   diagnozy, ale interfejs musi jawnie oznaczyć go jako `DIAGNOSTIC_BLOCKED` i
   nie może przedstawiać go jako poprawnie nałożonego supermodelu.
7. **Brak pustego pomiaru.** Brak próbki, regionu, komponentu lub klipu jest
   wynikiem `BLOCKED`, nigdy `PASS`.
8. **Deterministyczność.** Identyczne wejścia, exact chain, opcje i dokument
   authoringu dają identyczny raport, rig i SHA-256 wyniku.

## 4. Etapy implementacji

### Etap 0 — zamrożenie oracle i kontraktu statusów

Najpierw powstają testy, które obecny kod musi oblać.

Zakres:

- zachować zamrożony raport złego borzoja jako negatywny oracle bez zmiany jego
  bajtów;
- dodać wersjonowany `ReferenceSupermodelAdmissionV3` z osobnymi stanami:
  `structure`, `surfaceAnatomy`, `jointFit`, `skinning`, `motionQuality`,
  `export`;
- zdefiniować stabilne kody blokad i zabronić agregowania `READY` z samych
  zielonych liczników coverage;
- dodać test regresyjny: znany zły fit nie może dojść dalej niż
  `BLOCKED_JOINT_FIT` albo `BLOCKED_SKINNING`, nawet jeśli ma 100% carrierów i
  klipów;
- dodać test, że raport nie może jednocześnie nazywać całej aplikacji
  `READY` i mieć blokujący etap podrzędny.

Główne miejsca zmian:

- `crates/m2a-core/src/reference_supermodel_structure.rs`;
- `crates/m2a-core/src/reference_supermodel_generic.rs`;
- `crates/m2a-core/src/reference_supermodel_motion.rs`;
- `crates/m2a-wasm/src/lib.rs`;
- `apps/studio-web/src/features/supermodels/types.ts`.

Kryterium wyjścia: wszystkie nowe negatywne testy są czerwone przed zmianą
algorytmu i jednoznacznie wskazują brakującą bramkę.

### Etap 1 — topology-aware analiza powierzchni

Zastąpić ocenę anatomii opartą głównie na globalnej obwiedni i licznikach
komponentów wersjonowanym `TargetSurfaceAnatomyV2`.

Zakres:

- utworzyć analizową, niemutującą geometrii renderowanej warstwę powierzchni:
  near-duplicate weld map, adjacency, component graph oraz uproszczony
  occupancy/body proxy;
- odróżnić główną bryłę ciała od cienkich kart sierści i odłączonych ozdób na
  podstawie topologii, grubości, pola, sąsiedztwa oraz spójności przestrzennej;
- karty i małe komponenty nie mogą przesuwać landmarków joint-fit;
- wyprowadzić jawne regiony: central body, branch roots, symmetric limbs,
  ground-contact terminals oraz appendage chains;
- każdy landmark raportuje pozycję, region, źródło, residual, confidence oraz
  aktywne constraints;
- niepełna albo wieloznaczna segmentacja kończy się `NEEDS_AUTHORING`.

Nie wolno zmieniać renderowanej liczby trójkątów ani UV. Proxy służy wyłącznie
do analizy.

Kryterium wyjścia: znana siatka borzoja nie traktuje setek kart sierści jako
równorzędnych powierzchni przesuwających pivoty; każdy komponent ma jawny role
i binding provenance.

### Etap 2 — constrained anatomical joint fitter V2

Zastąpić `minimumConfidence >= 0.05` pełnym kontraktem dopasowania.

Algorytm:

1. similarity transform exact reference rig służy tylko jako inicjalizacja;
2. central chain jest dopasowywany do osi głównej bryły z osobnymi punktami
   miednicy, klatki, szyi i głowy;
3. branch roots są wiązane z właściwymi regionami central body;
4. symetryczne kończyny są rozdzielane na lewą/prawą i przednią/tylną przed
   dopasowaniem łańcuchów;
5. terminale łap są dopasowywane do czterech odrębnych klastrów kontaktu;
6. kolejne jointy kończyn zachowują kolejność, dodatnie długości segmentów,
   stronę ciała i dozwolone kąty bind pose;
7. appendage root jest osadzany w odpowiednim regionie central body, a dalsze
   pivoty są dopasowywane do osobnej centerline appendage;
8. solver minimalizuje residual landmarków z regularizacją symetrii, długości
   kości, orientacji referencyjnej i położenia wewnątrz właściwego regionu;
9. wynik zawiera per-joint residual i per-constraint verdict.

Status `READY` jest możliwy wyłącznie, gdy wszystkie wymagane jointy i
constraints są zmierzone i zaliczone. Samo znalezienie wielu pobliskich
wierzchołków nie zwiększa już confidence do `READY`.

Kryterium wyjścia: na zdefiniowanym oracle pozycji jointów błąd centralnych i
kończynowych pivotów mieści się w zamrożonych tolerancjach, a zły układ znany z
diagnostyki zostaje odrzucony przed skinningiem.

### Etap 3 — semantyczny skinning V2

Wagi muszą wynikać z poprawnego fitted rig i regionów powierzchni.

Zakres:

- ziarna wag powstają wyłącznie we właściwym semantic region jointu;
- propagacja używa odległości geodezyjnej po analizowej powierzchni;
- twarde bariery zabraniają przecieku: lewa↔prawa strona,
  przód↔tył, tors↔ogon oraz między niesąsiednimi gałęziami;
- odłączone karty sierści dziedziczą wagi z najbliższej zgodnej semantycznie
  powierzchni głównej; nie biorą udziału w joint-fit;
- duplikaty pozycji należące do jednego szwu otrzymują identyczne wiersze wag;
- smoothing pozostaje lokalny i nie może zmienić primary carrier na
  niesąsiadujący w hierarchii;
- coverage jointu wymaga jednego poprawnego, połączonego klastra o odpowiednim
  regionie i mierzalnym polu, a nie tylko jednej dodatniej wagi;
- usunąć lub fail-closed zastąpić każdą naprawę promującą przypadkowy komponent
  wyłącznie dla uzyskania pełnego licznika.

Kryterium wyjścia: nie występuje cross-side/cross-branch leakage, każdy
skin-relevant joint ma lokalny semantic cluster, a wszystkie wiersze są
znormalizowane i mają najwyżej cztery wpływy.

### Etap 4 — walidacja bind pose przed animacją

Dodać osobną bramkę pomiędzy skinningiem i motion quality.

Sprawdzenia:

- exact nazwy, parenty i lokalne bazy carrierów;
- zgodność fitted world positions z raportem joint-fit;
- joint znajduje się w oczekiwanym regionie albo na dozwolonym terminalu;
- dodatnie długości i poprawna kolejność każdego łańcucha;
- zachowana strona jointów symetrycznych;
- cztery łapy są czterema różnymi terminalami;
- tail/appendage root i kolejne pivoty są przestrzennie uporządkowane;
- barycentrum dominującego klastra każdego jointu leży w jego semantic region;
- brak trójkątów łączących niesąsiednie carrier branches.

Kryterium wyjścia: zły borzoi skeleton widoczny w obecnej diagnostyce nie może
otrzymać `BIND_POSE_PASS`.

### Etap 5 — motion-quality jako końcowy oracle deformacji

Zachować obecne per-clip i per-component sprawdzenia i rozszerzyć je o
semantyczne metryki regionów.

Zakres:

- każdy wymagany klip ma niepustą próbkę każdego render componentu;
- każdy wymagany joint ma pełną joint×clip coverage;
- per-component edge i triangle budgets są blokujące;
- seam violation, clip-start anchor jump, paw-contact/paw-side violation i
  brak klastra są blokujące;
- appendage chain sprawdza względny ruch kolejnych jointów, zmianę krzywizny i
  ruch końcówki względem nasady;
- klastry powierzchni muszą podążać za właściwym carrierem, nie tylko poruszać
  się w zbliżonym kierunku;
- próbkowanie adaptacyjne dodaje fazy wokół lokalnych ekstremów, zamiast
  polegać wyłącznie na stałych pięciu punktach klipu.

Kryterium wyjścia: żadna lokalna katastrofa nie może zostać rozcieńczona przez
globalną liczbę poprawnych próbek.

### Etap 6 — jedna bramka preview/product/export

Usunąć rozbieżne wywołania sterowane luźnym booleanem
`enforce_joint_fit_authoring`.

Wprowadzić jawny tryb:

- `DIAGNOSTIC` — może zwrócić i pokazać zablokowany rig wraz z powodami;
- `PRODUCT` — zawsze wymaga pełnego `ReferenceSupermodelAdmissionV3=PASS`.

Preview zaakceptowane do produktu i build muszą używać dokładnie tych samych:

- source SHA-256;
- exact-chain SHA-256;
- analysis/fitter/weighting algorithm IDs;
- sealed authoring SHA-256;
- target rig SHA-256;
- motion report SHA-256.

Rekomputacja z innymi danymi albo inną wersją algorytmu unieważnia preview i
wymaga ponownej walidacji.

Kryterium wyjścia: nie istnieje publiczna trasa produktu, która może ominąć
anatomy, joint-fit, bind-pose, skinning albo motion gate.

### Etap 7 — Studio: porównanie i authoring jointów

Rozwinąć istniejący rig inspector oraz `RigJointEditor`.

Wymagania UI:

- jednoczesny overlay exact retail carrier rig i fitted target rig;
- niezależne przełączniki: mesh, bones, joints, skin clusters i failed regions;
- rozdzielone liczniki: carrier rig nodes, scene transform nodes, skin bones,
  helpers i geometry nodes;
- lista jointów z reference position, auto-fit position, residual, confidence,
  constraint verdict i provenance;
- kolory: zielony `PASS`, żółty `NEEDS_AUTHORING`, czerwony `BLOCKED`;
- symetryczna edycja jointów, edycja landmarków, reset pojedynczego jointu,
  undo/redo oraz walidacja na żywo;
- component binding i region-weight authoring dla powierzchni pomocniczych;
- zapis tylko wersjonowanego, zahashowanego dokumentu authoringu;
- przycisk produktu nieaktywny, dopóki pełny admission nie ma `PASS`.

Kryterium wyjścia: użytkownik widzi dokładnie, który pivot lub region blokuje
wynik i może poprawić go bez ręcznej edycji wygenerowanego MDL.

### Etap 8 — korpus testowy i regresje

#### Testy zawsze wykonywane w CI

- syntetyczny quadruped z czterema kończynami i ogonem;
- długonogi quadruped o innych proporcjach niż reference rig;
- gęsta siatka z odłączonymi kartami sierści i duplicate seams;
- biped, serpent/long-chain i flyer/branched-appendage;
- przypadki negatywne: złączone przednie łapy, odwrócona oś, brak kończyny,
  ogon sklejony z udem, asymetryczna geometria i brak jednoznacznego regionu;
- deterministyczny snapshot raportów oraz hashy.

#### Realny korpus read-only przed wydaniem

- exact `c_wolf` oraz jego znane retail children odczytywane in-place;
- co najmniej jeden realny supermodel o strukturze innej niż quadruped;
- kanoniczny Meshy borzoi z `sample-3d` jako wymagany target regresyjny;
- jawny release command, który kończy się błędem, jeśli wymagany lokalny corpus
  nie jest dostępny — brak zasobu nie może być raportowany jako zaliczony test;
- żadnego kopiowania retail payloadów do fixture ani wyników produktu.

Kryterium wyjścia: syntetyczny corpus przechodzi w zwykłym CI, a pełny lokalny
release corpus przechodzi bez pominiętych wymaganych przypadków.

### Etap 9 — refactor po zielonych testach

Po działającej implementacji:

- podzielić duże moduły na wyraźne warstwy: analysis, fitting, skinning,
  validation i admission;
- usunąć nieużywane starsze algorytmy i fallbacki dopiero po potwierdzeniu, że
  nie są publicznym kontraktem;
- zastąpić kolejne sufiksy `Vxx` wersjonowanymi typami raportu i changelogiem
  algorytmu;
- centralizować progi w jednym profilu tolerancji, nie w rozproszonych stałych;
- każdy wyjątek algorytmiczny musi wynikać z topologii/semantyki i mieć test
  negatywny; nazwa supermodelu nie może sterować zachowaniem.

Kryterium wyjścia: jeden publiczny vertical slice prowadzi od exact chain do
admission, a każda blokada ma jedno źródło i stabilny kod.

### Etap 10 — kandydat demonstracyjny i proof właściciela

Nowy MOD/HAK powstaje dopiero po przejściu wszystkich bramek offline. Nie
tworzymy kolejnych numerów demo podczas naprawy pojedynczych testów.

Po zamrożeniu jednego exact kandydata:

1. zapisać SHA-256 GLB, exact chain, authoringu, MDL, tekstur, 2DA, HAK i MOD;
2. przygotować raport z nazwą pliku MOD, nazwą modułu, Area, obiektem,
   Appearance row, HAK i placementem;
3. zainstalować dokładny MOD/HAK według obowiązującej polityki absent-target i
   byte-identical hash verification;
4. nadać `ready_for_owner_proof` bez uruchamiania przez agenta Toolsetu/NWN;
5. właściciel wykonuje końcowy proof Toolset/NWN i przekazuje wynik.

## 5. Mierzalne kryteria zakończenia

### 5.1 Struktura supermodelu

- 100% exact carrier names i parent links zgodnych z wybranym chain;
- 100% wymaganych controller channels i klipów;
- zero produkcyjnych warunków po `c_wolf`, nazwie rasy lub gatunku;
- brak skopiowanych retail payloadów;
- raport jednoznacznie rozdziela carrier nodes od scene/geometry/helper nodes.

### 5.2 Anatomia i joint-fit

Progi należy skalibrować w Etapie 1 na realnym read-only corpusie, zatwierdzić
w jednym profilu i zamrozić przed implementacją fittera. Początkowy kontrakt
akceptacyjny:

- 100% wymaganych jointów ma niepusty semantic region, landmark, residual i
  constraint verdict;
- zero jointów z provenance `aabb_seed`/`unknown` w wyniku `READY`;
- central-chain landmark residual `<= 2.5%` przekątnej authoritative body;
- limb/appendage landmark residual `<= 2.0%` przekątnej authoritative body;
- ground terminal wysokość `<= 1.0%` wysokości ciała od wykrytej płaszczyzny
  kontaktu i planar residual `<= 2.0%` przekątnej;
- różnica lustrzana par lewa/prawa po odbiciu `<= 1.5%` przekątnej;
- każda długość parent→child `> 0.5%` przekątnej albo ma jawny wyjątek
  strukturalny pochodzący z exact contract;
- cztery terminale kończyn są odrębnymi klastrami i zachowują stronę oraz
  porządek przód/tył;
- appendage chain ma poprawną kolejność od nasady do końcówki i nie jest
  dopasowany do regionu tylnej nogi;
- znany zły borzoi fit nie otrzymuje `JOINT_FIT_PASS`.

Jeżeli kalibracja realnego corpusu wymaga zmiany liczby, zmiana musi nastąpić
przed zakończeniem Etapu 1, mieć raport porównawczy i nie może być poluzowana
wyłącznie po to, aby przepuścić borzoja.

### 5.3 Skinning

- każdy renderowany wierzchołek ma 1–4 skończone, nieujemne wpływy;
- suma wag każdego wierzchołka wynosi `1.0 ± 1e-4`;
- zero odwołań do carrierów spoza allowed set;
- zero cross-side i cross-nonadjacent-branch leakage na regionach o jednoznacznej
  klasyfikacji;
- każdy skin-relevant joint ma co najmniej jeden połączony, dominujący klaster
  w poprawnym semantic region;
- wszystkie duplicate seam groups mają identyczne wiersze wag;
- 100% auxiliary components ma jawne projection/binding provenance;
- zero przypadkowych component reservations służących wyłącznie coverage.

### 5.4 Deformacja i animacje

- `requiredClipCount == sampledClipCount`;
- `jointClipPassCount == jointClipRequiredCount`;
- `perClipGeometryCoverage=true`;
- `perComponentGeometryCoverage=true`;
- `perClipDeformationCoverage=true`;
- `perComponentDeformationCoverage=true`;
- zero failed component samples;
- zero seam violations, missing anchor clusters, paw-side violations,
  paw-contact violations i clip-start anchor jumps;
- `appendagePairPassCount == appendagePairRequiredCount` i zero relative-motion
  violations;
- każda próbka mieści się w zamrożonych hard edge/triangle tolerances;
- dla `c_wolf`: wszystkie 42 wymagane klipy i pełna macierz 907/907;
- końcowy `motionQualityStatus=PASS` bez wyjątków diagnostycznych zmieniających
  admission.

### 5.5 Testy i jakość kodu

- wszystkie testy `m2a-core`, `m2a-wasm`, Studio i Worker przechodzą;
- wymagany realny local release corpus ma zero skipów;
- negatywny oracle nie może zostać odwrócony w `PASS` zmianą progu bez
  zatwierdzonego raportu kalibracji;
- dwa kolejne uruchomienia na tych samych danych mają identyczne raporty i
  SHA-256 rig/model output;
- brak publicznej ścieżki omijającej centralny admission;
- brak nowych specjalnych warunków po nazwie supermodelu;
- dokumentacja algorytmu, progów i kodów blokad odpowiada implementacji.

### 5.6 Studio

- retail rig i fitted target rig mogą być oglądane jednocześnie;
- liczby carrier/scene/skin/helper/geometry nie są ze sobą mieszane;
- każdy błędny joint lub region jest wskazany w modelu i tabeli;
- joint/landmark/component/weight authoring jest wersjonowany, hash-bound,
  walidowany i odwracalny;
- `Apply/Build/Export` pozostaje zablokowane dla każdego admission innego niż
  pełny `PASS`;
- użytkownik nie musi ręcznie modyfikować MDL poza aplikacją.

### 5.7 Proof końcowy

Agent może zakończyć implementację statusem `ready_for_owner_proof` dopiero po
spełnieniu wszystkich kryteriów offline oraz zamrożeniu i instalacji exact
MOD/HAK z weryfikacją hashy.

Pełne zakończenie problemu wymaga wyniku właściciela dla tego samego lineage:

- model widoczny w Toolsecie i NWN;
- poprawna sylwetka w idle, walk, run, attack i death;
- cztery kończyny pracują niezależnie bez sklejonych lub sztywnych łap;
- łapy nie lewitują w neutralnej pozie i nie zamieniają stron;
- ogon zgina się od nasady do końcówki i porusza względnie, nie jako jeden
  sztywny płat;
- brak klinów, rozrywanych płatów, przerywanych linii deformacji i widocznych
  pęknięć szwów;
- widoczny wynik odpowiada dokładnym hashom przygotowanego kandydata.

## 6. Poza zakresem tej naprawy

- generowanie nowego modelu w Meshy;
- poprawa tekstury, normal/specular maps i materiału niezwiązana z deformacją;
- tworzenie nowych animacji zamiast dziedziczenia z wybranego supermodelu;
- obrót broni w dłoni i animacje trzymania przedmiotów;
- agent-run finalny proof w Aurora Toolset/NWN bez nowej, bezpośredniej zgody
  właściciela dla exact kandydata.

## 7. Kolejność krytyczna

Etapy 0–4 są P0 i muszą zostać wykonane kolejno. Etap 5 nie może maskować
błędów Etapów 1–4. Etapy 6–8 zamykają spójność produktu i regresje. Refactor z
Etapu 9 następuje dopiero po zielonych testach. Demo i proof z Etapu 10 są
ostatnim krokiem, nie narzędziem do strojenia algorytmu metodą prób i błędów.
