# Texture Artifact Cleanup dla Creature — 2026-07-28

Status: `IMPLEMENTED / OFFLINE VERIFIED`

## Nazwa opcji

Widoczna nazwa w Studio:

`Repair texture artifacts`

Nazwa kontraktu technicznego:

`textureArtifactCleanup`

Nie używamy nazwy `Fix pixels`, ponieważ nie rozróżnia ona uszkodzonych danych,
szumu tekstury, celowych detali materiału i przezroczystości.

## Objaw i diagnoza

Właściciel potwierdził widoczność exact Creature P300K w NWN i wskazał jasne
punkty na ciemnej powierzchni modelu. Audyt exact tekstury wykluczył brak
tekstury, z-fighting, błędne normalne, biały specular i masowe uszkodzenie
kanału alfa.

Źródłowy base color Meshy zawiera lokalne jasne i ciemne odchylenia. Po
pomniejszeniu tekstury na modelu część z nich wygląda w rendererze NWN jak
izolowane białe punkty.

Konserwatywny profil wykrył w exact base color Stoneback:

- `4 186 116` sprawdzonych pikseli wewnętrznych;
- `1 050` izolowanych odchyleń koloru kwalifikujących się do naprawy;
- `0` izolowanych dziur alfa;
- input pixel SHA-256
  `fd05864b65c21dc98cc52131d0c6bc012ad546646a04940ef4135aaf9566f163`;
- output pixel SHA-256
  `af28f6a5298ad460ffe10fff3d4cdbc11829ceeee80a59f9fb86043743e177cc`.

## Zachowanie

Opcja jest domyślnie wyłączona. Wyłączenie zachowuje dotychczasowy obraz i
wynikowy TGA byte-for-byte.

Po włączeniu filtr działa wyłącznie na base color przed zapisem TGA:

1. naprawia pojedynczy jasny albo ciemny piksel, gdy jego ośmiu sąsiadów tworzy
   spójne, nieprzezroczyste otoczenie;
2. naprawia pojedynczą dziurę alfa tylko wtedy, gdy co najmniej siedmiu z ośmiu
   sąsiadów jest nieprzezroczystych;
3. używa mediany sąsiadów i jednego deterministycznego przebiegu;
4. nie rozmywa krawędzi, większych refleksów, klastrów detalu ani większych
   obszarów przezroczystych;
5. raportuje liczbę napraw i SHA-256 pikseli przed oraz po operacji.

## Przejście przez aplikację

Aktywna ścieżka:

`Studio checkbox -> Worker request -> WASM options JSON -> Rust core -> cleanup baseColor -> TGA -> HAK`

Obsługiwane są proceduralne lane Creature:

- produkt `PRODUCT_20K`;
- eksperyment segmentowany `P100K`;
- eksperyment segmentowany `P300K`.

Pełne rozszerzenie tej samej opcji na statyczny M0, jawny 42-clip H1,
Placeable i Tile pozostaje osobnym zadaniem. UI pokazuje opcję wyłącznie przy
celu Creature.

## Dowody offline

- testy syntetyczne naprawiają izolowane jasne, ciemne i przezroczyste piksele;
- testy negatywne zachowują krawędzie, klastry i większe regiony alfa;
- test wyłączonej opcji potwierdza byte-exact brak zmiany;
- env-gated test exact Stoneback potwierdza `1 050` napraw;
- test granicy WASM potwierdza strict options JSON i obecność
  `textureArtifactCleanup` w raporcie;
- test React potwierdza domyślnie wyłączony checkbox;
- test workflow P300K potwierdza przekazanie `textureArtifactCleanup=true` do
  Workera.

Nie utworzono nowego MOD/HAK ani nowej iteracji modelu. Weryfikacja wizualna
wyniku z włączonym filtrem pozostaje decyzją właściciela po osobnym
zmaterializowaniu zatwierdzonego kandydata.

## Exact test aplikacyjnego pipeline — Stoneback P300K

Test wykonano na canonical source:

`sample-3d/tlc-stoneback-brute-h1-p300k-v1/source.glb`

SHA-256 źródła:

`0eeb135c1077f2207b1f362222829a2493c003c64276db98633cf2af5c8385b3`

Ścieżka testu:

`Studio Worker -> WASM -> Rust core -> MDL/TGA/HAK/MOD writers -> canonical result projector`

Wynik:

- `textureArtifactCleanup=true`;
- `1 050` naprawionych izolowanych odchyleń koloru;
- `0` naprawionych dziur alfa;
- `290 318` trójkątów i `213 331` wierzchołków w wynikowym MDL;
- `42/42` animacje obecne w readback;
- TGA SHA-256
  `170d6ac6b1321fe8e5b12bab7e7fa3d804b8ed3d113ea53c05087ffdde339455`;
- MDL SHA-256
  `499a0e4dc4d98317ed0b4bbe79686354c7bd7e445876d6ca758ca000039ef91e`;
- HAK SHA-256
  `f4bbab67d3766e1d2be522212ad07353827032bee05c5dbf557e34d6421723a7`;
- MOD SHA-256
  `f35e0c796dc569af87dbdf253aa627ab4c0a73853a131bbf95b42766fb8d0907`;
- Appearance.2da SHA-256
  `572ee5364b52e2d49a52bd86789ebf8d08a0184cd449895a858d0c5b870bcd26`;
- hash osobnego artefaktu TGA jest równy hash TGA zapisanej w manifeście HAK;
- canonical result projector zaakceptował kompletny zestaw artefaktów.

Pierwszy przebieg wykrył brak kontraktowy: Worker pakował naprawioną teksturę
do HAK, ale nie wystawiał jej jako osobnego artefaktu Studio. Naprawiono Worker
dla proceduralnych lane Creature 20K/P100K/P300K, dodając `texture-tga`, oraz
zaostrzono projektor wyniku tak, aby wymagał i uzgadniał jego rozmiar i SHA-256
z `summary.outputs.texture`.

Test był in-memory. Nie zapisano nowego trwałego MOD/HAK, nie instalowano
artefaktów w katalogach NWN i nie wykonywano agentowego proof w Toolset/NWN.

## Amendment 2026-07-28 — robust local-neighbor median V2

Owner proof wykazał, że pierwsza implementacja pozostawiła liczne jasne kropki.
Kolory naprawcze nie były losowe: pochodziły z mediany ośmiu lokalnych
sąsiadów. Błąd znajdował się w klasyfikatorze — surowy rozstęp luminancji
wszystkich sąsiadów odrzucał naprawę, gdy obok badanego piksela znajdował się
choć jeden dodatkowy jasny albo ciemny punkt. Pary i krótkie szeregi artefaktów
wzajemnie blokowały naprawę.

Aktywny algorytm to teraz `ROBUST_LOCAL_NEIGHBOR_MEDIAN_V2`:

1. sprawdza lokalne nieprzezroczyste sąsiedztwo `3 x 3`;
2. klasyfikuje środek względem drugiego najciemniejszego i drugiego
   najjaśniejszego sąsiada, ignorując jeden dodatkowy skrajny punkt;
3. wpisuje medianę każdego kanału RGB wyłącznie z lokalnych sąsiadów;
4. wykonuje dwa deterministyczne przebiegi, dzięki czemu naprawia pary i
   krótkie szeregi;
5. nadal chroni normalne granice materiałów i większe regiony alfa;
6. raportuje nazwę algorytmu oraz liczbę przebiegów.

Exact Stoneback daje `3 682` naprawy koloru, `0` napraw alfa, pixel SHA-256
`029b4191f10820fe4d7e1c6772529a7a2dbab4289144b02b425c223a510f111e`
oraz TGA SHA-256
`6623657cd8f7c6b875eb7cefe0fd8a18a4cbb306dd0e8abfd1c647cdb5c73d1a`.

Pełny test produkcyjnej ścieżki Studio/Worker/WASM/core i pakiet owner proof są
opisane w
`evidence/tlc-stoneback-brute-p300k-cleanup-v3-ready-for-owner-proof-2026-07-28.md`.

## Amendment 2026-07-29 — edge-aware Hampel median V3

Owner proof Cleanup V3 nadal wykazał jasne punkty. Na tym etapie roboczo
przypisano je wysokoczęstotliwościowemu detalowi base color i minifikacji
gęstego atlasu UV. Ta historyczna hipoteza została później obalona przez
kontrolowany A/B geometrii V5; obowiązująca korekta znajduje się w amendmencie
`Stoneback root cause corrected` poniżej.

Aktywny algorytm checkboxa to teraz `EDGE_AWARE_HAMPEL_MEDIAN_V3`:

- lokalne okno `5 x 5`;
- mediana luminancji i medianowe odchylenie bezwzględne;
- minimalna różnica luminancji `24`;
- próg adaptacyjny `4 x MAD`;
- maksymalnie dwóch sąsiadów podobnych do badanego środka;
- co najmniej połowa nieprzezroczystych sąsiadów zgodna z medianą;
- naprawa tylko jasnych impulsów;
- RGB zastępowane medianą lokalnych sąsiadów;
- dwa deterministyczne przebiegi;
- bez globalnego rozmycia i bez losowego koloru.

Exact Stoneback daje `4 588` napraw jasnych impulsów i `0` napraw alfa.
Output pixel SHA-256 to
`62ebd7a04eddcdb13ae43133c2aa78513cb426ce2ed056c59db11aa74be50937`,
a TGA SHA-256 to
`ac6aceb3f2809c5ffe1170d2525df672fb077855cf8bc1d78f24241596f05fbc`.

Pełny pakiet V4 i instalacja do owner proof są opisane w
`evidence/tlc-stoneback-brute-p300k-cleanup-v4-ready-for-owner-proof-2026-07-29.md`.

## Amendment 2026-07-29 — Stoneback root cause corrected

Powyższa diagnoza V3 o `base color` jako źródle jasnych punktów została
obalona przez kontrolowany test geometrii V5 i pozytywny owner proof.

W porównaniu V4/V5 zachowano:

- tę samą teksturę TGA o SHA-256
  `ac6aceb3f2809c5ffe1170d2525df672fb077855cf8bc1d78f24241596f05fbc`;
- ten sam materiał i binding tekstury;
- te same animacje oraz 42 stany NWN;
- tę samą geometrię źródłową Meshy.

Jedyną istotną zmianą była reguła degeneratów. Stary absolutny próg epsilon
usunął `5 957` prawidłowych mikrotrójkątów na wejściu i jeszcze jeden w
późniejszej granicy. V5 odrzuca wyłącznie dane niefinitywne, trójkąty o
zerowym polu lub dokładnie współliniowe i zapisał wszystkie `296 276`
trójkątów. Właściciel potwierdził V5 jako `visible/verified` w Aurora Toolset
i NWN oraz `geometryArtifactResult=fixed`.

Wniosek obowiązujący dla tego modelu:

- jasne punkty/ubytki Stonebacka były artefaktem geometrii, nie pikseli;
- checkbox texture cleanup pozostaje opcjonalnym, domyślnie wyłączonym
  narzędziem dla rzeczywistych lokalnych wad texeli;
- statystyka naprawionych pikseli nie jest dowodem, że tekstura była przyczyną
  artefaktu renderowanego modelu;
- nie należy włączać filtra tekstury jako automatycznej naprawy tego problemu.

Dokładny wynik A/B i hashe V5 są zapisane w
`evidence/tlc-stoneback-brute-p300k-geometry-ab-v5-ready-for-owner-proof-2026-07-29.md`.

## Aktualizacja limitu produktu — 2026-07-29

Nazwy `PRODUCT_20K`, P100K i P300K wyżej opisują profile użyte w historycznych
testach. Bieżący domyślny profil to `PRODUCT_300K`, a opcjonalny cleanup
tekstury działa tak samo dla całego wspólnego budżetu do 300 000 trójkątów.
