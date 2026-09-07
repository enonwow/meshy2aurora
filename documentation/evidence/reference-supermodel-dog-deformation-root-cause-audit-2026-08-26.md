# Audyt deformacji psów dziedziczących po supermodelu — 2026-08-26

## Werdykt

Fatalny wygląd borzoja nie wynika z tego, że `c_wolf` nie został wybrany albo
że NWN nie odtwarza animacji. Dziedziczenie klipów działa: model chodzi, atakuje
i zmienia pozę. Błąd leży między siatką źródłową a tymi poprawnymi
kontrolerami — automatyczny fitter umieszcza jointy zbyt ogólnie, generator
rozlewa wagi skóry prawie po całym połączonym modelu, a walidator przepuszcza
skrajnie złą deformację jako `PASS`.

To są defekty ogólnej trasy supermodeli, nie wyjątek specyficzny dla wilka.
Każdy proporcjonalnie odmienny model używający
`RetargetedTargetBind` może trafić na ten sam problem.

## Dokładna badana linia

- MOD: `m2aborzmod10.mod`, SHA-256
  `0a97d8fd3429fc359b7d56bd36b3b7a3927f557af3e8f073c06233f24e234456`;
- HAK: `m2aborzhak10.hak`, SHA-256
  `8a168fdc4cdb4b3439f08940d53c3e774cf23c13979c58064aafc379abfd3186`;
- MDL: `m2aborzcre10.mdl`, SHA-256
  `764298489ca8f6e60e4eb8f29bd8249cdd11d2d31226fd5b2eac2a68e164c4fd`;
- źródło GLB: SHA-256
  `3efd673f4ec953de568b2a30f6f14131a62828398f3133bb89855193b5fffd93`;
- supermodel: `c_wolf`, 42 wymagane klipy, bez lokalnych animacji w modelu;
- raport: `proof-output/borzoi-c-wolf-demo-v10-20260826-new-source/product-report.json`.

Materiał właściciela porównujący V10 z detalicznym wilkiem jest opisany w
`documentation/evidence/borzoi-v10-vs-retail-wolf-runtime-video-comparison-2026-08-26.md`.
Na nagraniu borzoj wykonuje animacje, ale okolice miednicy, uda i nasady ogona
składają się w ostre kliny oraz płaty. Detaliczny wilk zachowuje w analogicznym
ruchu ciągłą sylwetkę. Ogon borzoja zmienia pozycję, lecz w dużej części porusza
się jak sztywny element; wilk zgina łańcuch od nasady do końcówki.

Materiały wideo są dowodem obserwacyjnym właściciela, ale nie mają dołączonego
zweryfikowanego odczytu hashy z działającej sesji. Dlatego osie dowodu pozostają
rozdzielone: `modelVisibility=visible` obserwacyjnie oraz
`proofCompleteness=missing` formalnie. Nie zmienia to diagnozy offline — raport
samego generatora zawiera już skrajne deformacje.

## Potwierdzone przyczyny

### P0 — jointy są dopasowywane do obwiedni, a nie do anatomii

`derive_generic_reference_rig_from_surface_v2` w
`crates/m2a-core/src/reference_supermodel_generic.rs:1345` bierze pozycje
jointów referencji i mapuje każdą oś liniowo z obwiedni szkieletu do obwiedni
całej siatki. Implementacja `fit_point_between_bounds_v2` znajduje się przy
linii 2460. To zachowuje jedynie względną pozycję w pudełku AABB.

Nie są wykrywane anatomiczne środki miednicy, klatki piersiowej, barków, kolan,
nasady ogona ani końca ogona. Dla długonogiego, smukłego borzoja z obszerną
sierścią obwiednia nie jest odpowiednikiem anatomii krępego wilka. Dodatkowy
fitter poprawia tylko cztery terminalne łańcuchy kontaktu łap. Pozostałe pivoty
nadal są wynikiem globalnego przeskalowania pudełka.

V10 ma `fittedGroundContactChainCount=4`, ale ten zielony wskaźnik nie dowodzi
poprawnego ułożenia miednicy, tułowia ani ogona.

### P0 — 512 iteracji rozmywa wagi niemal do maksymalnych czterech jointów

Wagi początkowe w `hierarchy_chain_segment_weights_v3`
(`reference_supermodel_generic.rs:2105`) zależą głównie od odległości punktu od
najbliższego odcinka szkieletu. Następnie
`topology_smooth_reference_weights_v3` (`:1670`) wykonuje **512 pełnych
iteracji** dyfuzji: w każdej iteracji wierzchołek zachowuje 60% swojego wiersza,
dostaje 40% średniej sąsiadów, po czym lista jest obcinana do czterech kości.
Na końcu dochodzi jeszcze 12 iteracji adaptacyjnych.

Dla V10 suma dodatnich przypisań wynosi 642 677 przy 162 359 wierzchołkach,
czyli średnio **3,958 wpływu na wierzchołek** przy maksimum równym 4. To niemal
pełne nasycenie całej siatki. Zamiast lokalnych stref kości powstaje szerokie
mieszanie sąsiednich segmentów. Obrót jednej kości ciągnie więc sierść i ciało,
które anatomicznie powinny należeć do innego segmentu.

Metoda odległościowa nie rozróżnia też warstw leżących blisko w przestrzeni,
ale należących do innych części ciała: lewej i prawej nogi, sierści brzucha,
uda, ogona oraz powierzchni zachodzących na siebie.

### P0 — „pokrycie jointu” może być naprawione bez poprawnej semantyki

Po rozmyciu wag brakujące jointy są obsługiwane przez
`ensure_required_joint_clusters_v3` (`reference_supermodel_generic.rs:2215`).
Funkcja najpierw wzmacnia wierzchołki bliskie odcinkowi jointu, a jeśli joint
nadal nie ma klastra, rezerwuje dla niego cały najbliższy odłączony komponent
lub grupę współrzędnych i nadaje wagę 1.0.

V10 przed tą naprawą nie miał widocznych klastrów dla
`Wolf_Lbackmidleg` i `Wolf_Rbackmidleg`; po naprawie raport pokazuje pełne
23/23. To dowodzi obecności niepustych zbiorów, nie tego, że joint dostał
właściwy fragment anatomiczny.

Rozkład V10 pokazuje alarmujące proporcje:

- `Wolf_ribcage`: tylko 90 dominujących wierzchołków przy 65 625 dodatnich;
- `Wolf_pelvis`: 726 dominujących przy 33 395 dodatnich;
- `Wolf_tail`: 259 dominujących;
- `Wolf_tailend`: 6 191 dominujących.

W parze ogonowej ok. 96% dominujących wierzchołków należy do końcówki. Oba
jointy formalnie „działają”, ale masa powierzchni jest przypisana bardzo
nierówno. To tłumaczy ogon poruszający się prawie jak sztywny płat oraz złamanie
przy nasadzie.

### P0 — walidator wystawia fałszywy `PASS`

Raport V10 ma jednocześnie:

- `status=PASS`;
- 188 863 290 próbek krawędzi;
- 1 678 244 krawędzie poza twardym zakresem;
- dozwolone 1 888 632, bo limit jest liczony globalnie jako 1% wszystkich
  próbek;
- maksymalne rozciągnięcie 37,34× i minimalny stosunek ok. 0,01×;
- `triangleSampleCount=0`;
- 896 naruszeń skoku anchorów na początku klipów;
- 42/42 klipy i 907/907 komórek joint×clip uznanych za zaliczone.

Globalny procent rozcieńcza lokalną katastrofę ogromną liczbą poprawnych
krawędzi gęstej siatki. Dla przykładu `cdead` ma 172 140 twardych naruszeń i
rozciągnięcie 37,34×, `cwalk` ma 69 222 naruszenia i 25,95×, a `ca1slashl`
40 948 naruszeń i 26,59×. Żaden z tych lokalnych wyników nie blokuje eksportu,
jeśli łączny licznik mieści się pod globalnym 1%.

Warunek `pass` w `reference_supermodel_motion.rs:4719` nie uwzględnia
`clip_start_anchor_jump_violation_count`, mimo że licznik jest wyliczany i
raportowany. Nie uwzględnia też ekstremalnego `maxEdgeRatio`/`minEdgeRatio` dla
pojedynczego klipu lub komponentu.

### P0 — kontrola trójkątów była faktycznie wyłączona przez próg gęstości

Próbka trójkąta jest liczona tylko wtedy, gdy pole w bind pose przekracza
`diagonal² * 1e-5` (`reference_supermodel_motion.rs:4902`). V10 jest gęsty:
299 783 trójkąty. Wszystkie znalazły się poniżej progu, więc walidator zbadał
zero trójkątów i automatycznie dostał zero zapadnięć oraz zero ekspansji.

To nie jest pozytywny wynik jakości. To pusta dziedzina pomiaru, która powinna
blokować produkt. Obecny kod uznaje ją za zaliczoną.

Podobny problem dotyczy miękkiej kontroli krawędzi: V10 ma
`edgeSoftSampleCount=0`, ponieważ żaden odcinek nie przekroczył 1% przekątnej
modelu. Dla gęstych siatek działa więc tylko rozcieńczony globalny limit twardy.

### P1 — kontrola ogona bada ruch, ale nie zginanie łańcucha

Anchor gate wybiera powierzchnię na podstawie najsilniejszego wpływu danego
jointu i porównuje amplitudę oraz kierunek trajektorii klastra z kontrolerem.
Domyślne minimum amplitudy i zgodności trajektorii wynosi tylko 0,25
(`reference_supermodel_motion.rs:1478`).

Test nie sprawdza:

- kolejności przestrzennej nasada → końcówka;
- względnego kąta i krzywizny między `Wolf_tail` i `Wolf_tailend`;
- tego, czy oba klastry obejmują sensowną część powierzchni ogona;
- czy koniec ogona porusza się względem nasady, a nie tylko razem z całym
  modelem.

Dlatego sztywny ogon może uzyskać `PASS`, jeżeli oba wybrane probe'y przesuną
się w podobnym kierunku jak kontrolery.

### P1 — pięć faz na klip może ominąć najgorszy moment

`animation_sample_times_v2` (`reference_supermodel_motion.rs:4783`) bada tylko
0%, 25%, 50%, 75% i 100% długości każdego klipu. To potwierdza pokrycie nazw
klipów, lecz nie gwarantuje uchwycenia ekstremum między próbkami. Już przy tych
pięciu fazach V10 ma fatalne liczby; gęstsze próbkowanie może ujawnić więcej.

### P2 — materiał obniża jakość, ale nie jest główną przyczyną klinów

Trasa V10 celowo emituje minimalny materiał: diffuse-only, bez tangentów i map
normalnych, `twoSided=true`, a dla segmentów ustawia `cast_shadow=false`
(`reference_supermodel_motion.rs:2860`). To spłaszcza wygląd sierści i może
uwidaczniać charakter geometrii źródła. Nie wyjaśnia jednak klinów zmieniających
kształt razem z pozą, ogromnych współczynników deformacji ani sztywnego ogona.
To wtórny problem renderingu, a nie przyczyna P0.

## Dlaczego `c_wolf` jest obecny, a efekt nadal zły

V10 przechodzi przez
`bind_static_mesh_for_inherited_supermodel_motion_minimal_twosided_mtr_retargeted_v4`
wywołane w `crates/m2a-wasm/src/lib.rs:770`. Tryb
`RetargetedTargetBind` zachowuje automatycznie wyliczone pivoty celu i nadaje im
dokładne nazwy oraz hierarchię supermodelu. To jest prawidłowa koncepcja dla
modelu o innych proporcjach, ale jej jakość zależy całkowicie od poprawnych
pivotów i wag.

Łańcuch awarii jest więc następujący:

1. gęsta, wielokomponentowa siatka borzoja trafia do globalnego fittera AABB;
2. tylko terminale łap dostają korektę względem powierzchni;
3. odległościowe wagi są rozmywane przez 512+12 iteracji;
4. naprawa pokrycia zapewnia każdy joint, ale nie poprawną anatomię;
5. poprawne kontrolery `c_wolf` poruszają błędnie przypisanymi obszarami;
6. powstają kliny, płaty i sztywny ogon;
7. globalny, zależny od gęstości walidator uznaje wynik za `PASS` i pozwala go
   spakować jako demo.

## Hipotezy odrzucone

- **Brak supermodelu:** odrzucone. MDL wskazuje `c_wolf`, ma zero lokalnych
  klipów, a model wykonuje odziedziczone animacje.
- **Brak jointów:** odrzucone jako przyczyna techniczna. Hierarchia 30
  carrierów jest obecna; problemem są pivoty i przypisanie powierzchni.
- **Całkowicie nieruchomy ogon:** odrzucone. Ogon zmienia kąt, lecz deformuje
  się niewłaściwie i zbyt sztywno.
- **Wyłącznie błąd tekstury/UV:** odrzucone. Najgorsze „linie” i kliny zmieniają
  się z animacją i korelują z błędami geometrii raportowanymi przez oracle.
- **NWN nie potrafi animować takiego szkieletu:** odrzucone przez detaliczny
  wilk w drugim nagraniu.
- **Stała lewitacja:** niepotwierdzona. Oba modele w animacji ataku mają fazy
  oderwania od podłoża; osobny, zsynchronizowany test byłby potrzebny do oceny
  kontaktu w idle/chodzie.

## Luki w testach

Obecne testy przechodzą: 9/9 w `reference_supermodel_generic`, 15/15 w
`reference_supermodel_motion` oraz dwa celowane testy anchorów. Nie chronią one
przed tym przypadkiem:

- test skoku początkowego tylko sprawdza, że licznik rośnie; nie wymaga, aby
  końcowy status był `BLOCKED`;
- brak regresji wymagającej `triangleSampleCount > 0` dla gęstej siatki;
- brak testu blokującego ekstremum per klip/per komponent mimo globalnego
  procentu poniżej limitu;
- brak testu krzywizny i względnego ruchu wielojointowego ogona;
- brak testu, że 512 iteracji nie nasyca prawie każdego wierzchołka czterema
  kośćmi;
- brak korpusu porównawczego z kilkoma detalicznymi modelami dziedziczącymi po
  tym samym supermodelu.

## Priorytet napraw

1. **Najpierw naprawić walidator.** Obecny V10 musi zostać odrzucony offline
   przez nowy gate. Bez tego każda kolejna iteracja może ponownie zostać
   błędnie oznaczona jako gotowa.
2. **Zastąpić AABB fitter dopasowaniem anatomicznym.** Wszystkie istotne pivoty
   — nie tylko łapy — muszą wynikać z regionów/landmarków powierzchni albo z
   autorskich override'ów: miednica, klatka, szyja, głowa, stawy nóg, nasada i
   końcówka ogona.
3. **Zastąpić 512-iteracyjną dyfuzję wag ograniczonym, geodezyjnym
   skinningiem.** Potrzebne są bariery lewa/prawa strona, bariera ogon–udo i
   kontrola udziału sąsiednich kości. Wygładzanie ma usuwać lokalne skoki, a nie
   rozprowadzać kości po całym komponencie.
4. **Dodać semantyczne kryteria łańcuchów.** Szczególnie ogon: porządek,
   pokrycie nasady i końcówki, względny ruch oraz krzywizna. Sama niezerowa
   amplituda jest niewystarczająca.
5. **Uniezależnić kontrolę geometrii od gęstości.** Musi istnieć niepusta,
   reprezentatywna próbka trójkątów dla każdego segmentu i klipu, limit lokalny
   per komponent/klip oraz blokada ekstremalnych outlierów. Skoki startowe
   muszą być częścią końcowego `pass`.
6. **Dopiero po poprawnym skinningu ocenić materiał.** W osobnym A/B porównać
   diffuse-only z trasą zachowującą właściwe dane materiałowe. Nie maskować
   geometrii zmianą cieni.

## Kryteria zakończenia naprawy

- Aktualny, zamrożony V10 daje `BLOCKED` w nowym walidatorze z jednoznacznym
  wskazaniem lokalnej deformacji — bez zmiany jego bajtów.
- Żaden produkt nie może mieć `PASS`, gdy `triangleSampleCount=0`, gdy któryś
  wymagany klip nie ma reprezentatywnej próbki albo gdy istnieje niezaakceptowany
  ekstremalny outlier per klip/per komponent.
- `clipStartAnchorJumpViolationCount > 0` blokuje produkt.
- Fitter raportuje anatomiczne landmarki i błąd dopasowania dla miednicy,
  klatki, głowy, czterech nóg oraz całego łańcucha ogona.
- Gate wag raportuje lokalność, entropię i rozdział lewa/prawa; wynik o
  nasyceniu takim jak V10 nie może przejść bez świadomego override'u.
- Ogon przechodzi dopiero, gdy końcówka porusza się względem nasady, zachowuje
  kolejność łańcucha i nie tworzy zagięcia/rozciągnięcia poza referencyjną
  obwiednię jakości.
- Testy obejmują co najmniej gęstą wielokomponentową siatkę, model o odmiennych
  proporcjach oraz kilka detalicznych modeli dziedziczących po wybranym
  supermodelu. Progi powinny być skalibrowane na tym korpusie, a nie na jednym
  borzoju.
- Nowy kandydat może zostać wygenerowany dopiero po wdrożeniu powyższych bramek
  i po spełnieniu obowiązującego model iteration gate.

## Wynik audytu

Audyt zakończony. Przyczyna jest znana i reprodukowalna w danych offline:
**poprawne animacje supermodelu są nakładane na źle dopasowany oraz nadmiernie
rozmyty skin, po czym nieadekwatny walidator zatwierdza wynik**. Kolejny model
bez naprawy fittera, wag i bramek jakości nie jest uzasadnioną iteracją.
