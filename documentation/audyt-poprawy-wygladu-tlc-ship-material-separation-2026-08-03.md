# Audyt poprawy wyglądu statku TLC po separacji materiałów

Data: 2026-08-03

## Wniosek

Separacja materiałów działa technicznie, ale obecny wynik nie spełnia jakości
artystycznej. Największym problemem nie jest liczba trójkątów ani rozdzielczość
tekstury. Przyczyną jest połączenie:

1. zbyt ogólnej klasy `Wood`, obejmującej niemal cały statek;
2. projekcji UV wykonywanej osobno dla ponad 21 tysięcy drobnych komponentów;
3. braku semantycznej ciągłości między deskami, belkami, pokładem i masztami;
4. zbyt czystej tekstury diffuse, która w NWN nie odtwarza struktury wcześniej
   zakodowanej w źródłowym atlasie;
5. ograniczeń samej geometrii Meshy: bardzo dużej fragmentacji, nieregularnych
   powierzchni i słabo zdefiniowanych desek.

Samo podmienienie tekstury `Wood` na kolejną teksturę drewna nie doprowadzi
modelu do poziomu referencji. Potrzebne są semantyczne grupy powierzchni,
projekcja UV oparta na powierzchniach statku, a nie surowych komponentach, oraz
diffuse przygotowany specjalnie pod sposób renderowania NWN.

## Zakres i dokładny kandydat

- MOD: `m2a_tlcsm2_mod.mod`
- nazwa modułu: `The Last City - Component UV Ship Demo`
- Area: `The Last City Component UV Shipyard`
- HAK: `m2a_tlcsm2_hak.hak`
- MOD SHA-256: `30f0bfc99ea9a32e9fc83c241e7c843ca1bc85ccf01bb5134e44c170bd519bc5`
- HAK SHA-256: `b7577438c53039e78518f6b7d574cbb06a38216415641bf370a3945ba0a17493`
- MDL SHA-256: `c5f2621aedc912b3479ae1eade5fc2ced84cef6debce5a26302255fa59afebb3`
- źródłowy GLB SHA-256:
  `61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278`

Właściciel potwierdził, że dokładny kandydat istnieje i jest widoczny w Aurora
Toolset, ale odrzucił jego wygląd. Test tej samej linii w NWN pozostaje
nieukończony.

## Fakty z artefaktów

| Metryka | Wynik |
|---|---:|
| trójkąty źródłowe / wyjściowe | 152 574 / 152 574 |
| czyszczenie geometrii | wyłączone |
| źródłowe komponenty | 21 936 |
| komponenty `Wood` | 21 060 |
| trójkąty `Wood` | 141 183 (92,5%) |
| średnia liczba trójkątów na komponent `Wood` | 6,70 |
| wierzchołki przed projekcją UV | 179 319 |
| wierzchołki po projekcji UV | 436 847 |
| dodatkowe wierzchołki UV | 257 528 (+143,6%) |
| wzmocnienie liczby wierzchołków | 2,44x |

Rozkład materiałów:

| Materiał | Komponenty | Trójkąty |
|---|---:|---:|
| Wood | 21 060 | 141 183 |
| Sail | 577 | 6 464 |
| Cloth | 380 | 3 633 |
| Rope | 397 | 1 178 |
| Metal | 57 | 116 |

Maska materiałów pokazuje, że kadłub, poszycie, żebra, belki, pokład, maszty i
większość nadbudowy są jednym materiałem `Wood`. Separacja na pięć ogólnych
materiałów jest poprawna jako dowód techniczny, ale zbyt mało precyzyjna dla
wiarygodnego statku.

## Diagnoza

### P0 — projekcja UV po surowych komponentach tworzy patchwork

Aktualny algorytm wybiera dwie najdłuższe osie AABB każdego komponentu,
normalizuje współrzędne osobno w jego lokalnym zakresie i dodaje fazę zależną
od indeksu komponentu. Przy 21 060 komponentach drewna oznacza to tysiące
niezależnych kierunków, skal i przesunięć słojów.

Reguła `Wood` wykorzystuje tylko pas `V=0.58..0.72`, `U repeats=0.35` oraz
deterministyczną fazę U. Sąsiednie elementy statku nie mają wspólnej ciągłości
UV. Lokalna projekcja skaluje również maleńki fragment tak, jakby był pełną
deską. To jest główna przyczyna plamistego, losowego i płaskiego wyglądu.

### P0 — `Wood` nie jest jednym materiałem wizualnym

Na statku co najmniej cztery klasy drewna wymagają innego kierunku, skali i
charakteru tekstury:

- poszycie kadłuba — długie deski zgodne z osią statku;
- żebra i belki konstrukcyjne — ciemniejsze, masywniejsze drewno wzdłuż belki;
- pokład i pomosty — projekcja z góry, wyraźne równoległe deski;
- maszty, reje i słupy — projekcja cylindryczna lub osiowa.

Jedna tekstura oraz jedna reguła UV nie mogą poprawnie obsłużyć wszystkich tych
powierzchni, nawet gdy sama tekstura jest wysokiej jakości.

### P0 — źródłowa geometria ma niski sufit jakości teksturowania

Model zawiera dużo trójkątów, lecz nie przekładają się one na uporządkowane
deski i belki. Średnio komponent drewna ma tylko 6,70 trójkąta. Referencyjna
łódź ma około 17 233 polygonów, ale lepiej zdefiniowane poszycie, krawędzie,
łączniki i kierunki konstrukcji. To dowodzi, że problemem jest organizacja
powierzchni i czytelność formy, a nie brak polygonów.

Tekstura może poprawić odbiór, lecz nie naprawi nieregularnej sylwetki,
zlepionych elementów, przypadkowych otworów ani powierzchni, które geometrycznie
nie przypominają desek.

### P1 — nowa tekstura usunęła informacje pomagające czytać formę

Źródłowy atlas Meshy zawierał wypalone cienie, zabrudzenia i kontrast krawędzi.
Był brudny i chropowaty, ale wzmacniał konstrukcję. Czystsze drewno V13 ma
lepszy kolor, jednak na rozdrobnionych UV staje się jednolitą brązową masą.

W ścieżce NWN najważniejszy jest diffuse. Normal, roughness i metallic z PBR
nie mogą być traktowane jako mechanizm, który uratuje efekt. Potrzebny jest
neutralny bake AO/cavity i kontrolowane przyciemnienie szczelin w diffuse, bez
wypalania kierunkowego światła.

### P1 — koszt projekcji UV jest za duży

Projekcja podnosi liczbę wierzchołków z 179 319 do 436 847. Nie usuwa
trójkątów, ale tworzy bardzo dużo szwów i dyskontynuacji. Zwiększa rozmiar MDL,
utrudnia spójne cieniowanie i jest sygnałem, że jednostką projekcji jest zbyt
mały element.

### P2 — brakuje diagnostyki UV i jakości powierzchni w aplikacji

Przed eksportem aplikacja nie pokazuje wystarczająco jasno:

- kierunku i gęstości texeli dla każdej grupy;
- liczby patchy UV i wzrostu liczby wierzchołków;
- nieciągłości między sąsiednimi powierzchniami;
- różnicy między materiałem a semantyczną grupą powierzchni;
- czy model osiągnął już sufit możliwy do poprawienia samą teksturą.

## Podejścia, których nie należy powtarzać

- Zwiększenie rozdzielczości jednej tekstury `Wood`: nie naprawi kierunku ani
  skali UV.
- Dodanie polygonów: model już ma ich dużo; referencja osiąga lepszy efekt przy
  znacznie mniejszej liczbie.
- Wypiekanie nowego drewna do starego atlasu: wcześniejsza próba dała płaski,
  rozmazany wynik, ponieważ źródłowe wyspy UV nie opisują sensownie materiałów.
- Kolejne strojenie `uRepeats`, `vMin` i `vMax`: może zmienić wielkość słojów,
  ale nie naprawi 21 tysięcy niezależnych projekcji.
- Agresywne czyszczenie geometrii: wcześniejsze testy usuwały potrzebne
  fragmenty; nie rozwiązuje problemu organizacji UV.

## Docelowe rozwiązanie

### 1. Semantic Surface Groups V1

Oddzielić pojęcie materiału od grupy powierzchni. Minimalny zestaw dla statku:

1. `wood_hull_planks`
2. `wood_structure`
3. `wood_deck`
4. `wood_masts_spars`
5. `sail`
6. `cloth`
7. `rope`
8. `metal`

Nie oznacza to generowania ośmiu modeli ani kosztu Meshy za każdy element.
Podział odbywa się wewnątrz jednego źródłowego modelu. Grupy mogą współdzielić
atlas tekstur, ale zachowują niezależne zasady UV i wyglądu.

### 2. Surface Patch UV V2

Zastąpić projekcję per-komponent projekcją per-spójny patch semantyczny:

- kadłub: oś U zgodna z długością statku, V zgodna z wysokością/krzywizną;
- pokład: projekcja planarna z góry;
- belki: oś U zgodna z długą osią całej belki, nie pojedynczego fragmentu;
- maszty/reje: projekcja cylindryczna albo osiowa;
- wspólna, fizyczna gęstość texeli dla wszystkich patchy;
- faza UV na poziomie patcha lub brak fazy, nigdy faza z indeksu surowego
  komponentu;
- duplikacja wierzchołków tylko na rzeczywistych szwach materiału/UV.

W trudnym modelu Meshy patch może łączyć pobliskie fragmenty według orientacji,
odległości i przypisanej grupy, nawet gdy topologicznie nie współdzielą
wierzchołków.

### 3. NWN Diffuse Bake V1

Przygotować atlas lub zestaw tekstur specjalnie dla renderera docelowego:

- 2–3 warianty drewna o spójnej palecie The Last City;
- ciemniejsze belki i żebra, nieco jaśniejszy pokład, postarzałe poszycie;
- neutralne AO/cavity oraz delikatne szczeliny między deskami;
- bez silnego światła namalowanego z jednego kierunku;
- wyższy kontrast materiałowy: jasny żagiel, ciepła lina, chłodny ciemny metal;
- kontrola czytelności przy odległości kamery typowej dla NWN, nie tylko w
  zbliżeniu.

### 4. Texture-only Suitability Gate

Dodać preflight określający, czy model nadaje się do automatycznego
reteksturyzowania. Powinien raportować:

- komponenty i medianę trójkątów na komponent;
- procent powierzchni w największej klasie materiałowej;
- odsetek bardzo małych i cienkich fragmentów;
- niespójne normalne i potencjalne otwory;
- przewidywany wzrost liczby wierzchołków po UV;
- wynik `suitable`, `risky` albo `geometry-limited` z uzasadnieniem.

Ten statek powinien otrzymać co najmniej `risky`, prawdopodobnie
`geometry-limited` dla jakości porównywalnej z ręcznie przygotowaną referencją.

### 5. Diagnostyka w aplikacji

Dodać widoki:

- Material ID i Semantic Group ID;
- UV checker z kierunkiem U/V;
- heatmapę gęstości texeli;
- liczbę patchy i przewidywane wzmocnienie wierzchołków;
- porównanie Source UV / Projected UV / NWN diffuse preview;
- per-grupa: tryb projekcji, skala, obrót i przesunięcie.

## Plan implementacji

1. Rozszerzyć model danych separacji o semantyczne grupy powierzchni i
   niezależne ustawienia UV, bez zmiany wspólnego budżetu 300 000 trójkątów.
2. Dodać klasyfikację czterech grup drewna oraz możliwość korekty zaznaczenia
   ścian w UI.
3. Zaimplementować generowanie patchy przestrzennych i orientacyjnych zamiast
   projekcji wszystkich 21 060 komponentów osobno.
4. Zaimplementować projekcję kadłuba, pokładu, belek i masztów z jednolitą
   gęstością texeli.
5. Ograniczyć duplikację wierzchołków do rzeczywistych szwów.
6. Dodać atlas diffuse dla NWN z neutralnym AO/cavity i wariantami drewna.
7. Dodać preflight przydatności modelu oraz widoki diagnostyczne.
8. Wykonać offline comparison board w tych samych rzutach i oświetleniu.
9. Zamrozić dopiero jednego nowego kandydata, z pełną identyfikacją hashy i
   niezmienioną liczbą trójkątów/PWK.
10. Przekazać go właścicielowi do dowodu w Toolset i NWN.

## Kryteria ukończenia

### Funkcjonalne

- Jeden model może mieć niezależne grupy `hull`, `structure`, `deck` i
  `masts_spars`, nawet jeśli wszystkie używają rodziny materiału `Wood`.
- Każda grupa ma niezależny tryb, skalę i orientację UV.
- Materiały `sail`, `cloth`, `rope` i `metal` zachowują poprawne przypisania.
- Pipeline nie wymaga dodatkowych generacji Meshy ani ręcznego rozdzielania
  źródłowego modelu na osobne pliki.

### Geometryczne i deterministyczne

- 152 574 trójkąty wejściowe daje dokładnie 152 574 trójkąty wyjściowe.
- PWK pozostaje bajtowo identyczny, jeśli zmiana dotyczy wyłącznie render UV i
  materiałów.
- Ten sam input i recipe daje te same hashe outputu.
- Wzrost liczby wierzchołków po UV nie przekracza 1,6x względem wejścia po
  separacji, chyba że raport jawnie uzasadni większy koszt rzeczywistymi
  szwami.
- Sąsiednie deski kadłuba mają zgodny kierunek i porównywalną gęstość texeli.
- Brak przesunięcia UV zależnego od przypadkowego indeksu komponentu.

### Wizualne offline

- Kadłub, belki, pokład i maszty są rozpoznawalne jako różne elementy
  konstrukcyjne bez polegania na kolorowej masce materiałów.
- Z odległości odpowiadającej kamerze NWN widoczne są kierunek desek, szczeliny
  i hierarchia konstrukcji.
- Wynik nie jest bardziej płaski niż poprzedni wariant zachowujący źródłowy
  atlas, a jednocześnie nie odzyskuje jego kredowej/chropowatej plamistości.
- Comparison board zawiera bok, przód, tył i górę dla źródła, wyniku oraz
  referencji.

### Dowód właściciela

- Właściciel potwierdza widoczność dokładnego kandydata w Toolset i NWN.
- Właściciel nadaje osobny werdykt `visualAcceptance=accepted` dla obu widoków.
- Werdykt jest przypisany do dokładnych hashy MOD, HAK, MDL i tekstur.

## Blokada procesu wymagająca decyzji właściciela

Obecna bramka iteracji modelu dopuszcza nową iterację wyłącznie po wyniku
`modelVisibility=not_visible`. Tutaj model jest widoczny, ale estetycznie
odrzucony. To oznacza, że zgodnie z aktualną regułą nie wolno jeszcze zamrozić
nowego kandydata materiałowego. Najpierw należy przetestować dokładnie tę samą
linię w NWN.

Jednocześnie audyt rekomenduje zmianę polityki dla iteracji artystycznych:

- zachować `modelVisibility` wyłącznie jako werdykt techniczny;
- utrwalić osobny `visualAcceptance=accepted|rejected|not_tested`;
- dopuścić minimalną iterację materiałów/UV po świeżym wyniku
  `visible + visualAcceptance=rejected`, bez wymagania awarii widoczności.

Bez tego rozdzielenia poprawnie działający, lecz brzydki model formalnie blokuje
każdą próbę poprawy jakości.

## Źródła lokalne audytu

- `proof-output/tlc-ship-material-separation-v2-component-uv-20260802/generated/placeable-report.json`
- `documentation/evidence/tlc-ship-component-uv-materials-ready-for-owner-proof-2026-08-02.md`
- `documentation/evidence/tlc-ship-component-uv-owner-toolset-visible-result-2026-08-03.json`
- `documentation/evidence/tlc-ship-wood-external-reference-comparison-2026-08-02.md`
- `artifacts/material-separation/tlc-ship-under-construction-v2/offline-preview-v13-component-uv-large-grain/retextured-side-xy.png`
- `artifacts/material-separation/ship-wood-comparison/external-half-built-viking-vs-tlc-v13-component-uv.png`
