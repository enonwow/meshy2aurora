# Plan naprawczy nakładania szkieletu supermodelu — 2026-08-26

## 1. Cel

Celem jest jedna, ogólna funkcja, która nakłada kompletną strukturę dowolnie
wybranego supermodelu na należącą do użytkownika siatkę Creature i daje
anatomicznie poprawny skinning.

Pipeline ma:

- zachować dokładne nazwy, hierarchię i role carrierów wybranego supermodelu;
- dopasować pozycje jointów do anatomii modelu docelowego, a nie tylko do jego
  bounding boxu;
- przypisać lokalne, ciągłe i semantycznie poprawne wagi powierzchni;
- animować całe łańcuchy, w tym nasadę i końcówkę ogona;
- blokować eksport, jeżeli automatyczne dopasowanie jest niepewne albo
  deformacja jest uszkodzona;
- używać tego samego zapieczętowanego rigu w preview, eksporcie MDL i produkcie
  MOD/HAK.

Nie jest celem kopiowanie retailowej geometrii lub wag. Supermodel dostarcza
kontrakt nazw, hierarchii i animacji. Pozycje jointów oraz skinning należą do
modelu docelowego.

## 2. Nienegocjowalne zasady architektury

1. **Brak wyjątków po nazwie.** W kodzie produkcyjnym nie może powstać warunek
   typu `if supermodel == "c_wolf"`, `if dog` ani lista ras. Zachowanie wynika z
   hierarchii, ról strukturalnych i danych wejściowych.
2. **Exact topology pozostaje niezmienne.** Nie wolno zmieniać nazw, parentów,
   numerów ani liczby carrierów wymaganych przez selected supermodel.
3. **Target bind jest anatomiczny.** W trybie retargetowanym jointy mają pivoty
   modelu docelowego; retail bind służy jako odniesienie proporcji i kierunków,
   nie jako bezpośrednia poza smukłego lub inaczej zbudowanego Creature.
4. **Brak cichego fallbacku.** Niska pewność wykrycia anatomii, brak regionu
   albo konflikt lewa/prawa kończy się `BLOCKED` i konkretną diagnostyką.
5. **Brak naprawiania metryki kosztem modelu.** Funkcja nie może przypisać
   przypadkowej wyspy powierzchni do brakującego jointu tylko po to, aby raport
   wykazał 23/23.
6. **Jeden artefakt rigu.** Preview i produkt konsumują dokładnie ten sam
   `contentSha256` fitted/authored riga oraz wag.
7. **Deterministyczność.** Ten sam GLB, exact-chain supermodelu, source-forward,
   profil fittera i authoring muszą dać byte-identyczny raport i wynik.
8. **Skala 300 000 trójkątów.** Algorytm musi respektować wspólny budżet
   produktu oraz niezależną segmentację strumieni binary MDL, bez usuwania
   geometrii.

## 3. Stan początkowy i oracle regresji

Zamrożony V10 jest negatywnym oracle:

- source GLB SHA-256:
  `3efd673f4ec953de568b2a30f6f14131a62828398f3133bb89855193b5fffd93`;
- MDL SHA-256:
  `764298489ca8f6e60e4eb8f29bd8249cdd11d2d31226fd5b2eac2a68e164c4fd`;
- 162 359 wierzchołków, 299 783 trójkąty, 326 komponentów;
- średnio 3,958 dodatniego wpływu na wierzchołek przy limicie 4;
- `triangleSampleCount=0`, 896 skoków anchorów oraz lokalne rozciągnięcia do
  37,34× pomimo `status=PASS`.

Pierwsze kryterium implementacji: nowa kontrola musi odrzucić dokładnie ten
niezmieniony wynik. Nie tworzymy najpierw V11 i nie dobieramy algorytmu pod
nowy obraz.

Audyt źródłowy:
`documentation/evidence/reference-supermodel-dog-deformation-root-cause-audit-2026-08-26.md`.

## 4. Docelowa architektura

### 4.1 `ReferenceSupermodelStructuralProfileV1`

Profil jest budowany z istniejącego exact motion contract i readbacku wybranego
supermodelu. Nie zależy od resrefu.

Ma opisywać:

- root i motion root;
- centralny łańcuch tułowia;
- rozgałęzienia kończyn;
- terminale kontaktu z podłożem;
- łańcuchy appendage, np. ogon, skrzydło lub macka;
- attachment/helper nodes, które nie deformują powierzchni;
- oczekiwany porządek jointów i kierunek każdego segmentu w bind pose;
- symetryczne pary gałęzi wykryte z topologii i pozycji referencji;
- wymagane oraz opcjonalne regiony powierzchni.

Profil nie narzuca „czterech łap”. Liczba gałęzi i terminali wynika z
kontraktu. Dzięki temu ta sama infrastruktura obsłuży quadruped, biped, flyer,
serpent i inne struktury.

### 4.2 `TargetSurfaceAnatomyV1`

Przed dopasowaniem jointów powstaje jawna analiza powierzchni:

- orientacja ze zweryfikowanego `sourceForward` oraz osi up;
- główny komponent ciała i komponenty pomocnicze;
- grupy duplikatów/seamów;
- płaszczyzna symetrii i confidence;
- punkty kontaktu z podłożem;
- medialna oś tułowia;
- region przedni/tylny oraz lewy/prawy;
- kandydaci głowy, klatki, miednicy i nasady appendage;
- centerline każdego wykrytego appendage;
- przypisanie luźnych komponentów sierści/ozdób do powierzchni bazowej;
- confidence i lista niejednoznaczności dla każdego landmarku.

Wysokopoligonowe i wielokomponentowe źródło nie może być traktowane jak jeden
jednorodny graf. Luźne fragmenty sierści powinny dziedziczyć pole wag z
najbliższego, zgodnego semantycznie fragmentu ciała, a nie wpływać na położenie
stawów.

### 4.3 `ReferenceSupermodelJointFitV1`

AABB może pozostać wyłącznie początkowym seedem. Końcowe jointy powstają przez
ograniczone dopasowanie łańcuchów:

1. root, klatka i miednica są osadzane na medialnej osi głównego korpusu;
2. każda kończyna jest dopasowywana od punktu przyłączenia do wykrytego
   terminala kontaktu;
3. stawy pośrednie zachowują kolejność, dodatnią długość segmentów i
   ograniczoną zmianę proporcji wobec struktury referencyjnej;
4. pary symetryczne zachowują stronę i zbliżoną wysokość, bez wymuszania
   identycznych proporcji siatki;
5. appendage jest dopasowywany po centerline od nasady do końca;
6. wszystkie pivoty otrzymują provenance: `auto`, `authored` lub `locked`;
7. każdy pivot raportuje błąd do landmarku, confidence oraz aktywne constraints.

Jeżeli wymagany łańcuch nie może zostać jednoznacznie dopasowany, wynik ma być
`BLOCKED_JOINT_FIT_AMBIGUOUS`, a nie automatycznie przeniesiony do środka AABB.

### 4.4 `ReferenceSupermodelSkinningV1`

Obecną globalną dyfuzję 512+12 należy zastąpić lokalnym generatorem wag:

- początkowe pole wag wynika z odległości **geodezyjnej po powierzchni** do
  dopasowanych segmentów, a nie wyłącznie z odległości euklidesowej;
- kandydaci są ograniczeni do kości z bieżącego łańcucha, rodzica i dzieci;
- lewa i prawa gałąź są odseparowane barierą symetrii;
- ogon i tylne nogi są rozdzielone granicą nasady/regionu;
- wygładzanie ma mały, profilowany promień grafu i kończy się po osiągnięciu
  lokalnej ciągłości, nie po stałych setkach iteracji;
- maksymalnie cztery wpływy są zachowane, lecz cztery wpływy nie są celem;
- duplikaty należące do prawdziwego seamu otrzymują identyczne wagi;
- odłączone elementy sierści dostają wagi przez projekcję na bazową
  powierzchnię, z zachowaniem semantycznego regionu;
- brak wymaganego klastra blokuje wynik; nie wolno rezerwować przypadkowej
  wyspy powierzchni z wagą 1.0.

Raport musi zawierać co najmniej:

- liczbę wpływów i entropię per wierzchołek/region;
- dominujący oraz dodatni vertex count per joint;
- geodezyczny zasięg klastra;
- cross-side leakage;
- tail-to-hindleg oraz limb-to-torso leakage;
- component binding provenance;
- liczbę wierzchołków wymagających manualnego override'u.

### 4.5 Authoring jako jawny fallback

Istniejący `ReferenceSupermodelRigAuthoringDocumentV1` i edytor jointów należy
rozszerzyć, a nie zastąpić specjalnym workflow dla psa.

Nowa wersja dokumentu powinna obsługiwać:

- `landmarkOverrides` — korekta wykrytych punktów anatomii;
- istniejące `jointOverrides` — precyzyjna lokalna macierz jointa;
- `componentBindings` — przypięcie fragmentu sierści/ozdoby do regionu;
- `regionWeightConstraints` — dozwolone/zabronione kości dla zaznaczonego
  regionu;
- istniejące `weightOverrides` — ostateczna korekta pojedynczych wierzchołków;
- locki oraz pełne związanie dokumentu z hashami źródła, chainu, kontraktu,
  algorytmu i bazowej analizy anatomii.

Zmiana authoringu musi unieważniać preview oraz build. Product V3/V4 nie może
sam przepieczętować zmodyfikowanego dokumentu.

### 4.6 `SkeletonApplicationQualityV1`

Jedna blokująca bramka łączy jakość bindu, wag i animowanej powierzchni.

#### Bind i anatomia

- pełna exact carrier topology;
- brak odwróconych lub zerowych segmentów;
- zachowany porządek każdego łańcucha;
- poprawne strony gałęzi symetrycznych;
- wymagane pivoty w dozwolonych regionach powierzchni;
- ground terminals przy odpowiadających im kontaktach;
- appendage base i tip w prawidłowej kolejności.

#### Skinning

- każdy wymagany joint ma lokalny, semantyczny klaster;
- klastry nie mogą być uznane za poprawne wyłącznie dlatego, że są niepuste;
- brak przekroczenia progów leakage i entropii profilu;
- brak prawie globalnego nasycenia czterema wpływami jak w V10;
- każdy odłączony komponent ma jawny binding provenance.

#### Deformacja

- niepusta, reprezentatywna próbka trójkątów dla każdego render segmentu oraz
  wymaganego klipu;
- wyniki per klip i per komponent, bez rozcieńczania jedną globalną liczbą;
- absolutna blokada ekstremalnych outlierów długości i pola;
- skoki anchorów na początku klipu są blokujące;
- adaptacyjne próbkowanie czasu obejmuje klucze kontrolerów oraz lokalne
  ekstrema, nie tylko pięć stałych faz;
- tail/appendage gate sprawdza względny ruch kolejnych jointów, krzywiznę i
  kolejność, nie tylko wspólny ruch całego klastra;
- paw/contact gate pozostaje niezależny od jakości pozostałych regionów.

Progi jakości muszą zostać skalibrowane na korpusie poprawnych retailowych
modeli różnych struktur. Nie wolno dobierać ich tak, aby przepuścić jeden
borzoj.

## 5. Plan wdrożenia

### Etap 0 — negatywne oracle i blokada false `PASS`

1. Dodać fixture raportu/surface V10 bez zmiany zamrożonych artefaktów.
2. Dodać test, że `triangleSampleCount=0` blokuje produkt.
3. Dodać skok startowy i ekstremum per klip/per komponent do warunku `pass`.
4. Dodać test, że V10 nie mieści się w jakości referencyjnego korpusu.
5. Zachować rozdzielenie `modelVisibility` i `proofCompleteness`.

Warunek wyjścia: obecny V10 ma deterministyczne `BLOCKED`, a poprawne
retailowe fixture nadal przechodzą.

### Etap 1 — kontrakty struktury i anatomii

1. Dodać `ReferenceSupermodelStructuralProfileV1`.
2. Wyprowadzać role oraz łańcuchy z exact motion contract/readbacku.
3. Dodać `TargetSurfaceAnatomyV1` i deterministyczną analizę komponentów.
4. Wystawić confidence i stabilne kody błędów zamiast fallbacków.
5. Dodać fixtures: quadruped z ogonem, biped, rozgałęziony flyer i długi
   serpent/appendage.

Warunek wyjścia: strukturalnie różne supermodele są opisane jednym kontraktem,
bez branchy po nazwie.

### Etap 2 — anatomiczny joint fitter

1. Wydzielić fitter z `reference_supermodel_generic.rs` do osobnego modułu,
   aby nie powiększać istniejącego pliku i nie mieszać analizy, wag oraz
   eksportu.
2. Zaimplementować trunk/medial-axis fit.
3. Zaimplementować constrained limb-chain fit.
4. Zaimplementować appendage centerline fit.
5. Dodać symmetry, joint-order, length i ground-contact constraints.
6. Zachować obecny exact carrier topology invariant.
7. Wystawić pełny raport przed generowaniem wag.

Warunek wyjścia: pivoty V10 są przypisane do właściwych regionów albo fitter
blokuje wynik i wskazuje brakujący landmark. AABB nie może być końcowym
autorytatywnym wynikiem.

### Etap 3 — lokalny generator wag

1. Usunąć 512-iteracyjną trasę z produkcyjnego path; pozostawić ją tylko jako
   jawnie nazwaną legacy/baseline, jeżeli jest potrzebna do testu.
2. Zbudować graf powierzchni z barierami semantycznymi.
3. Wygenerować lokalne geodezyjne wagi dla trunk, limbs i appendages.
4. Przenieść wagi na komponenty sierści przez projekcję do powierzchni
   bazowej.
5. Dodać kontrolowane wygładzanie i normalizację top-4.
6. Usunąć island-reservation jako mechanizm osiągania required coverage.
7. Dodać raport leakage, entropy, dominance i component provenance.

Warunek wyjścia: V10 nie wykazuje nasycenia 3,958/4, a każdy wymagany joint ma
lokalny klaster zgodny z anatomią. Brak klastra daje `BLOCKED`, nie sztuczne
23/23.

### Etap 4 — authoring V2 i Studio

1. Rozszerzyć schema authoringu o landmark, component i region constraints.
2. Zachować zgodność odczytu V1; migracja nie może wymyślać nowych korekt.
3. W Studio pokazać:
   - detected landmarks i confidence;
   - bind/animated skeleton;
   - heatmapę wag wybranego jointu;
   - leakage i komponenty powierzchni;
   - regiony niejednoznaczne/blokujące;
   - podgląd before/after dla override'u.
4. Wszystkie ciężkie operacje wykonywać w Workerze.
5. Preview ma używać tego samego sealed artifact co produkt.

Warunek wyjścia: użytkownik może naprawić niejednoznaczny pivot lub region bez
ręcznego przepisywania tysięcy wag i bez naruszenia carrier topology.

### Etap 5 — motion/deformation oracle V2

1. Zastąpić zależne od gęstości pomijanie trójkątów reprezentatywnym
   samplingiem per segment/komponent.
2. Dodać lokalne budżety jakości i absolutne outlier gates.
3. Próbkować czasy kluczy i adaptacyjnie doszukiwać ekstremów.
4. Włączyć clip-start jumps do końcowego werdyktu.
5. Dodać chain curvature dla ogona/appendage.
6. Porównać metryki z retailowym envelope właściwego structural profile.
7. Raportować dokładny klip, czas, komponent, trójkąt i dominujące wpływy.

Warunek wyjścia: model ze złożeniem takim jak V10 nie może uzyskać `PASS`, a
diagnostyka prowadzi do konkretnego jointu/regionu.

### Etap 6 — Core/WASM/Worker/API parity

1. Dodać jedną publiczną operację przygotowania fitted rig artifact.
2. Dodać jedną operację sealed authoring → preview/product.
3. Zwracać structural profile, anatomy, joint-fit, skinning i quality report.
4. Związać wynik z SHA-256 wszystkich wejść i wersją algorytmu.
5. Dodać parity tests Core/WASM/Worker: raporty i payloady byte-identical.
6. Usunąć produktowe wejścia korzystające bezpośrednio z legacy AABB+512.

Warunek wyjścia: nie istnieje osobna „demo route”, która może ominąć
produkcyjne dopasowanie lub walidację.

### Etap 7 — korpus regresyjny

Korpus musi zawierać:

- zamrożony, błędny V10 jako negative oracle;
- poprawny model retail dziedziczący po `c_wolf`;
- co najmniej dwa różne, prawidłowo przygotowane quadrupedy;
- co najmniej trzy strukturalnie odmienne rodziny supermodeli;
- gęstą siatkę blisko 300 000 trójkątów;
- wielokomponentową sierść/ozdoby;
- przypadki negatywne: brak ogona, błędny forward, połączone lewe/prawe nogi,
  odłączony appendage, niejednoznaczne kontakty i niepoprawny authoring.

Testy obejmują deterministyczność, topology, landmarki, pivoty, wagi,
deformację wszystkich wymaganych klipów, binary MDL readback i brak regresji
geometrii/UV/materiału.

Warunek wyjścia: wszystkie pozytywne fixture przechodzą, wszystkie negatywne
są blokowane z oczekiwanym kodem, a w produkcyjnym kodzie nie ma whitelisty
resrefów.

### Etap 8 — nowy kandydat i proof właściciela

Dopiero po zamknięciu etapów 0–7 można wygenerować jeden nowy fitted/authored
kandydat na już dopuszczonym źródle. Należy:

1. zamrozić GLB, supermodel chain, authoring, MDL, HAK i MOD hashami;
2. przygotować demo z widocznym idle, chodem, biegiem, atakiem, śmiercią oraz
   ruchem całego ogona;
3. zainstalować MOD/HAK w nieistniejących celach i zweryfikować hash
   byte-for-byte;
4. zatrzymać się na `ready_for_owner_proof`;
5. przekazać właścicielowi dokładny MOD, nazwę modułu, Area, Appearance i
   oczekiwane sceny testowe.

Agent nie uruchamia Toolsetu ani NWN. Końcowy werdykt wizualny należy do
właściciela zgodnie z `AGENTS.md`.

## 6. Kolejność zmian w kodzie

Zalecany podział, aby nie tworzyć kolejnego monolitu:

- `reference_supermodel_structure.rs` — structural profile;
- `reference_supermodel_surface_anatomy.rs` — analiza powierzchni i landmarki;
- `reference_supermodel_joint_fit.rs` — constrained joint fitting;
- `reference_supermodel_skinning.rs` — geodezyjne wagi i audit;
- `reference_supermodel_motion.rs` — wyłącznie motion/deformation oracle;
- `reference_supermodel_authoring.rs` — schema V2 i sealed overrides;
- `reference_supermodel_generic.rs` — cienka orkiestracja powyższych etapów;
- `m2a-wasm/src/lib.rs` — stabilna granica API bez alternatywnych demo path;
- Studio/Worker — wizualizacja i edycja tego samego artefaktu.

Każdy etap dostaje osobne typy raportów oraz testy. Nie należy rozszerzać jednej
funkcji o kolejne warunki naprawcze.

## 7. Kryteria zakończenia całej naprawy

Naprawa nakładania szkieletu jest zakończona dopiero, gdy wszystkie kryteria są
spełnione:

### Kontrakt

- dowolny selected supermodel przechodzi jedną ogólną trasą;
- exact nazwy, hierarchy, part numbers i clip lookup są zachowane;
- nie ma `c_wolf`/dog-specific branchy w produkcyjnym fitterze lub skinningu;
- preview i eksport mają ten sam fitted/authored rig SHA-256.

### Joint fit

- każdy wymagany pivot ma landmark, confidence i provenance;
- root/trunk, wszystkie limbs i appendages zachowują kolejność oraz dodatnie
  długości;
- strony symetryczne nie zamieniają się;
- AABB jest co najwyżej seedem, nigdy jedynym dowodem poprawności;
- niejednoznaczność blokuje wynik przed skinningiem/eksportem.

### Wagi

- brak globalnej 512-iteracyjnej dyfuzji w product path;
- klastry są lokalne i zgodne z regionami;
- leakage, entropy i component binding mieszczą się w zamrożonym profilu;
- brak required cluster daje `BLOCKED`;
- ogon ma rozdzielone pokrycie nasady i końcówki, zamiast dominacji niemal
  całej powierzchni przez jeden terminal;
- liczba trójkątów, indeksy, UV i materiały wejścia są zachowane.

### Deformacja

- każdy wymagany klip i render component ma niepustą próbkę;
- `triangleSampleCount=0` zawsze blokuje oceniany segment/klip;
- clip-start jump jest blokujący;
- lokalne ekstremum nie może zostać rozcieńczone globalnym procentem;
- ogon przechodzi relative-chain/curvature gate;
- żaden pozytywny fixture nie ma widocznych klinów, płatów ani pęknięć w
  offline sampled deformation.

### Regresje i produkt

- zamrożony V10 jest poprawnie odrzucany przez nowe testy;
- pozytywny korpus kilku struktur przechodzi Core, WASM, Worker i writer
  readback;
- dwa identyczne runy dają identyczne hashe;
- Build nie może ominąć sealed authoringu ani quality gate;
- nowy dokładny kandydat uzyskuje `ready_for_owner_proof` dopiero po wszystkich
  zielonych bramkach offline i instalacji zweryfikowanego MOD/HAK;
- właściciel potwierdza w NWN brak klinów, poprawny kontakt kończyn i zginanie
  ogona od nasady do końca.

## 8. Definicja „gotowe”

„Szkielet został nałożony” nie oznacza już tylko, że model ma wszystkie nazwy
jointów i reaguje na animacje. Oznacza łącznie:

1. exact carrier topology wybranego supermodelu;
2. anatomiczne pivoty modelu docelowego;
3. lokalny i semantyczny skinning;
4. poprawną deformację wszystkich wymaganych łańcuchów;
5. blokujący, niezależny od gęstości raport jakości;
6. identyczny artefakt w preview i eksporcie;
7. pozytywny finalny proof właściciela w NWN.

Dopiero spełnienie wszystkich siedmiu warunków zamyka temat nakładania
szkieletu.
