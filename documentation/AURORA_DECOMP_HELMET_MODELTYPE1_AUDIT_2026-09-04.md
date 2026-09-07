# Audyt dekompilacji Aurory — hełmy ModelType=1

Data: 2026-09-04

Zakres: wyłącznie analiza offline dekompilacji i zasobów retailowych pod kątem
problemów zgłoszonych dla helm_221. Nie uruchamiano ani nie sterowano Aurora
Toolset lub NWN.

## Źródła i tożsamość

- dekompilacja: C:\Projects\New Folder\export\decompiled_all.c
  - rozmiar: 30,366,020 B
  - SHA-256: 36bb8b1031afe2abf23f0e18180a5ad649401d9ea5e078e5170a31a96167c572
- indeks retail: C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\data\nwn_base.key
  - SHA-256: 09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935
- donor HAK: C:\Users\enonw\Documents\Neverwinter Nights\hak\bdhd_items.hak
  - SHA-256: e36eeeb7a0ffd37ade510538995859c3c65023fd2ef4f472e78b62dc791316f0
- efektywne appearance.2da: artifacts/diagnostics/effective-appearance-audit-20260904/appearance.2da
  - SHA-256: ca0b80b74e068d8ebbd94df6005b5971e50eca5c8662fca10a40688ea2c033a2
- inspekcja donora bdhd_items:helm_035:
  artifacts/diagnostics/retail-helmet-035-audit-20260904/helm_035-inspection.json
  - SHA-256: 661b9f8f66bde740c531ce1d7a94dec1cb0fae7a2fbeeec3e1ef51aed72397f5

## Wynik

### 1. Profil hełmu ma jedną część i sześć kolorów

FUN_00604d70 w liniach 299891–299913 rozpoznaje cztery wartości ModelType.
Dla wartości 1 zwraca jedną część modelu oraz sześć wartości kolorów. Odczyt
UTI w FUN_00604db8, linie 300254–300273, pobiera ModelPart1, a następnie
dokładnie:

1. Leather1Color
2. Leather2Color
3. Cloth1Color
4. Cloth2Color
5. Metal1Color
6. Metal2Color

Wniosek: brak osobnych obiektów geometrycznych nazwanych metal1, metal2 itd.
nie jest błędem. Kolorowanie jest własnością pikseli PLT, a nie podziałem
modelu na sześć meshów.

### 2. Renderer mapuje wartości UTI na warstwy PLT

Ścieżki wyposażenia w liniach 73641–73646 i 196618–196623 tworzą sześć
selektorów palety z wartości UTI. Potwierdzona semantyka warstw PLT:

| Pole UTI | Warstwa PLT |
| --- | ---: |
| Metal1Color | 2 |
| Metal2Color | 3 |
| Cloth1Color | 4 |
| Cloth2Color | 5 |
| Leather1Color | 6 |
| Leather2Color | 7 |

Wysokie bajty 0x200, 0x400, 0x500 w ścieżce renderera oraz 0x300, 0x400,
0x500 w kompozytorze ikony wybierają rodziny palet; nie są numerami warstw
zapisanymi w samym pikselu PLT.

FUN_0064a9c4, linie 338753–338806, pobiera zasób typu 6. To jest PLT. Dla
porównania FUN_0064b12c, linie 339208–339261, pobiera zasób typu 3, czyli TGA.

Wniosek dla R2: helm_221.tga może zostać pokazany przez część podglądów, ale
nie realizuje kontraktu kolorowalnego hełmu. Sześć pól UTI nie ma czego
przemapować, dlatego wybór Metal 1 nie zmienia wyglądu, a widok obiektu na
ziemi może wpaść w materiał zastępczy/srebrny.

### 3. Ikona hełmu ModelType=1 również musi być PLT

FUN_00503c74, linie 169023–169038, składa nazwę ikony wariantu w rodzinie
i<itemclass>_<NNN>, czyli dla wariantu 221 ihelm_221.

FUN_0064b39c jest kompozytorem ikony. Gałąź ModelType=2 w liniach
339401–339440 używa zwykłego obrazu. Gałąź obejmująca jednoczęściowy
kolorowalny przedmiot w liniach 339585–339685:

- sprawdza zasób typu 6 (339651);
- przekazuje sześć bajtów kolorów do FUN_0064ab58 (339684);
- FUN_0064ab58 ładuje PLT przez FUN_0064a9c4 (338867) i buduje palety z
  sześciu wartości (338882–338887).

Retailowy ihelm_035 potwierdza format: PLT V1, 64×64, zasób typu 6; tło jest
kodowane jako para colorIndex=255, layerIndex=0.

Wniosek dla R2: ihelm_221.tga był zarówno złego typu zasobu, jak i zawierał
nieprzezroczyste czarne tło. To bezpośrednia przyczyna czarnego pola w
ekwipunku.

### 4. Skala nie jest automatycznie normalizowana per wariant

Ścieżki w 73648–73727 oraz 196625–196704 odczytują z appearance.2da wyłącznie
rasowo-płciowe mnożniki HELMET_SCALE_F lub HELMET_SCALE_M. Dla wiersza 6
(Dynamic Human) efektywna tabela daje:

- kobieta: 0.85
- mężczyzna: 1.05

Nie znaleziono kroku normalizującego osobno helm_001, helm_035, helm_221 itd.
Wszystkie warianty muszą więc być zapisane w zgodnej, retailowej przestrzeni
modelu.

Donor bdhd_items:helm_035 ma efektywne granice:

- min [-0.1262488, -0.1597881, -0.1117951]
- max [0.1262488, 0.1663945, 0.2245254]

R2 zajmował tylko około 76.8% szerokości i 66.5% głębokości donora, ponieważ
algorytm brał minimum z trzech ograniczeń, w tym prowizorycznej granicy
kaptura. To źródło za małego hełmu; nie błąd appearance.2da.

### 5. Responsywność zaznaczania

Dekompilacja nie ujawniła flagi blokującej przesuwanie wariantu 221. R2
zawierał 300,000 trójkątów w 15 render-meshach, podczas gdy retailowy donor ma
978 wierzchołków i jeden render-mesh. Zatem najbardziej uzasadnioną przyczyną
niepraktycznego zaznaczania/manipulacji jest koszt geometrii w starym
Toolsecie, a nie właściwość UTI.

To jest wniosek diagnostyczny, nie obserwacja wykonania Toolsetu. R3 redukuje
koszt do 100,000 trójkątów w 5 strumieniach, nadal zachowując UV, normalne,
materiał, pełną powierzchnię i deterministyczny podział poniżej limitu 65,535
indeksów na strumień.

## Zastosowana poprawka R3

- nowy deterministyczny czytnik/pisarz PLT V1;
- ikony TGA pozostają dla ModelType=0/2, ale ModelType=1/3 wymagają PLT;
- pakiet Item zapisuje i waliduje właściwy typ zasobu ikony;
- helm_221.plt: warstwa 2 Metal 1 dla ciepłego brązu, warstwa 4 Cloth 1 dla
  kaptura i neutralnych wykończeń;
- ihelm_221.plt: 64×64, przezroczysta ramka/tło, te same dwie warstwy;
- skala jednostajna 0.18652023, 96% szerokości donora i 5 mm odstępu od
  górnej granicy;
- granice R3:
  - min [-0.12119886, -0.13216108, -0.13546540]
  - max [0.12119886, 0.13876748, 0.21952541]
- geometria: dokładnie 100,000 trójkątów, 5 strumieni;
- niezależne skalowanie osi: nie zastosowano.

## Granica dowodu

Odczyt zwrotny GLB, MDL, PLT, UTI, HAK i MOD przeszedł offline. Zgodnie z
decyzją właściciela finalny wynik wizualny w Toolsecie/NWN należy do
właściciela. Audyt nie twierdzi, że R3 został już wizualnie potwierdzony.

## Aneks 2026-09-04 — korekta po wyniku właściciela dla 224

Wynik 224 ujawnił brakujący warunek pakietu ModelType=1. Same poprawne indeksy
warstw w kilku niezależnych PLT nie wystarczają. Działające R3 ma wszystkie
render-meshe związane z jednym `helm_221.plt`, a retailowy donor analogicznie
wiąże `texture0=helm_035` z `helm_035.plt`. 224 jako pierwszy odszedł od tego
układu i wskazywał `h224cloth` oraz `h224metal`; właściciel zobaczył model, ale
nie uzyskał działającego kolorowania części.

Skorygowany kontrakt produktu dla hełmu ModelType=1:

1. wszystkie render-meshe wskazują jeden texture0 równy resrefowi modelu;
2. HAK zawiera dokładnie model-nazwany PLT typu 6;
3. wiele materiałów źródłowych jest najpierw pakowanych do jednego atlasu UV;
4. semantyka materiału jest kodowana per piksel atlasu (`Metal 1=2`,
   `Cloth 1=4`), a nie przez kilka nazw tekstur.

`renderHint` nie wyjaśnia błędu: działające R3 i 224 mają wartość 0. Nie ma też
podstaw do dodawania TXI `pltreplacement` do bezpośredniego modelowego PLT:
retailowy pakiet `bdhd_items:helm_035` zawiera dla tego resrefu MDL i PLT, ale
nie TXI. Implementacja 225 używa więc pojedynczego modelowego PLT bez TXI oraz
dodaje bramkę regresyjną, która odrzuca konstrukcję podobną do 224.
