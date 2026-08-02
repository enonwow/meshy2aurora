# Implementacja ustaleń ponownego audytu Item

Status: `IMPLEMENTED_AND_OFFLINE_VERIFIED`

Data: 2026-07-30

Branch/worktree:

- branch: `items`;
- worktree: `C:\Projects\meshy2aurora\.worktrees\items`.

## Granica zmiany

Zmiana naprawia luki wykryte w ponownym audycie pipeline’u Item. Nie tworzy
nowej iteracji modelu, nie regeneruje ani nie zastępuje zamrożonego kandydata
`item-composed-r01-20260730` i nie uruchamia Aurora Toolset ani NWN.

Zachowano publiczną ścieżkę materializacji V1 używaną przez `r01`. Skorygowane
zachowanie ikon i skalowania ma osobny kontrakt
`build_meshy_item_part_with_options_v2`.

## Zaimplementowane ustalenia

### CAPART jest planem 19 natywnych partów

ModelType `3` nie jest jednym modelem i nie jest też zestawem 19 Meshy GLB.
Każde pole `ArmorPart_*` ma trwałe mapowanie:

```text
pole UTI
  -> dokładny wiersz CAPART
  -> oczekiwane MDLNAME + NODENAME
  -> właściwa tabela parts_*
  -> zakres selektora BYTE
```

Pipeline wymaga wszystkich tabel:

```text
CAPART
PARTS_FOOT, PARTS_SHIN, PARTS_LEGS, PARTS_PELVIS, PARTS_CHEST,
PARTS_BELT, PARTS_NECK, PARTS_FOREARM, PARTS_BICEP,
PARTS_SHOULDER, PARTS_HAND, PARTS_ROBE
```

Resolver sprawdza każdą z 19 relacji, fizyczną liczbę wierszy właściwej tabeli
i odrzuca wariant poza zakresem. `CAPART.2da` musi mieć dokładnie 19 fizycznych
wierszy i kolumny `NAME/MDLNAME/NODENAME`; każda tabela `PARTS_*` musi mieć
dokładną nazwę pliku i swój oczekiwany schemat. Dla robe resolver czyta
`COSTMODIFIER`, `ACBONUS` i 19 dokładnych kolumn `HIDE*`, akceptuje wyłącznie
`0/1` i utrwala listę maskowanych `MDLNAME`.

Resolver V2 wymaga także kontekstu modelu postaci. Dla każdego niemaskowanego
partu próbuje natywnej postaci `%s%c_%s%03d`, następnie `%s_%s%03d`, i wymaga
rzeczywistych, niepustych bajtów MDL typu `2002`. Raport utrwala wybrany resref,
rozmiar, SHA-256, hash kontekstu i hash dokładnej tabeli źródłowej. Tabele nie
są globalnie przypięte do jednego retailowego SHA, aby poprawny modded
`parts_*.2da` był możliwy; ich tożsamość jest jednak jawnie związana przez
nazwę, bajty, SHA-256 i dokładny schemat.

### Ikony wynikają z geometrii

V2 nie skaluje prostokąta tekstury do rozmiaru ikony. Software rasterizer:

- rozwiązuje world matrix każdego węzła i segmentu;
- projektuje faktyczne trójkąty w stałym widoku izometrycznym;
- rasteryzując używa depth buffer i interpolowanych UV;
- próbkuje osadzoną teksturę;
- pozostawia alfa `0` poza sylwetką modelu.

Worker wykonuje dwa przebiegi. Pierwszy zbiera granice projekcji wszystkich
partów. W drugim każda warstwa jest rasteryzowana z jednymi wspólnymi granicami,
a Worker sprawdza, że hashe MDL i tekstury pozostały identyczne. Raport zawiera granice,
liczbę nieprzezroczystych pikseli i tryb `GEOMETRY_RASTER_TGA_V2`.

Review dekoduje dokładne artefakty TGA z odpowiednią orientacją i kolejnością
BGRA/RGBA. Pokazuje każdy layer oraz ich authoring composite. To podgląd
wyemitowanych bajtów; nie jest deklaracją natywnej visual parity Aurory.

### Preview i writer mają jeden transform

Euler w UI używa dokładnej konwencji Three.js `XYZ`; test porównuje
wieloosiowy przypadek bezpośrednio z `THREE.Euler`/`Quaternion`.

Skalowanie V2 zachowuje hierarchię:

- pozycje geometrii są skalowane lokalnie;
- translacje potomków są skalowane względem rodzica;
- tylko translacja korzenia używa modelowego pivotu;
- rotacje węzłów pozostają niezmienione;
- translation i rotation autora pozostają rigid kontrolerem korzenia.

Test wieloosiowy i test obróconej hierarchii potwierdzają tę samą semantykę,
którą stosuje viewport. Preview stosuje także dokładne source-space
odpowiedniki M0 bottom-center groundingu: środek X/Z i minimum Y przed
sprzężeniem bazy Y-up ↔ Z-up. Pełna macierz autora to
`T · P · R · S · P^-1`, a test regresyjny obejmuje grounding, pivot,
wieloosiowy obrót i skalę.

### Seam jest bramą builda

Viewport nadal klasyfikuje AABB sąsiednich partów, ale jest to jawnie tylko
pomocniczy podgląd i nigdy nie blokuje Build. Jedyną autorytatywną bramą jest
Worker/core:

- ponownie przygotowuje oba modele z dokładnych bajtów GLB, `sourceNode` i
  transformów użytych przez writer;
- rozwiązuje world matrix każdego segmentu i mierzy minimalną odległość między
  faktycznymi trójkątami przez BVH;
- wykrywa ścisłe przecięcia, zawieranie brył i wspólne wnętrze zamkniętych
  siatek;
- odrzuca każdy `GAP` przekraczający tolerancję i każdy `OVERLAP`;
- utrwala SHA-256 obu źródeł, obu transformów/sourceNode, liczby trójkątów,
  algorytm `TRIANGLE_SURFACE_BVH_CONTAINMENT_V1` i hash samego pomiaru.

UI nie przesyła już wyniku AABB jako rzekomego dowodu. Test UI celowo podaje
`PREVIEW GAP` i potwierdza, że dopiero Worker rozstrzyga Build.

### Preview jest związany z treścią źródła

Cache viewportu zawiera stabilną tożsamość obiektu `File`. Podmiana pliku na
inny obiekt o tej samej nazwie, rozmiarze i `lastModified` wymusza ponowne
parsowanie. `sourceNode` jest wyszukiwany globalnie w domyślnej scenie, musi
wystąpić dokładnie raz i musi być rootem sceny. Odpowiada to walidacji core,
więc UI nie może pokazać innego wyboru niż ten użyty przez build.

### Cloak jest związany z istniejącymi zasobami

Produkt używa resolvera V4. Z wybranego wiersza `CloakModel` oraz prefixu
wyprowadzonego z `appearance.2da` rozwiązuje dokładne:

```text
2002:<prefix>_cloak_<MODEL:03>.mdl
6:cloak_<TEXTURE:03>.plt
6:icloak_<gender>_<ICON:03>.plt
```

Każdy wpis musi być zadeklarowany w przypiętym manifeście `nwn_base.key` i
mieć rzeczywisty, niepusty payload o zgodnym SHA-256. Core sprawdza format
ASCII/binary MDL albo PLT V1, resref i locator BIF. Brak któregokolwiek zasobu
jest błędem `ITEM-CLOAK-RESOURCE-MISSING`; wynik raportuje
`PINNED_MANIFEST_PAYLOAD_VALIDATED`.

### Wielomateriałowy GLB nie traci tekstur po cichu

Obecny kontrakt jednego partu emituje dokładnie jedną teksturę. Jeżeli wybrana
geometria używa więcej niż jednego źródłowego materiału, pipeline kończy się
fail-closed kodem `ITEM-PART-MULTI-MATERIAL-UNSUPPORTED` przed wyborem
`first()` lub zapisem artefaktu. Drugi invariant po konwersji odrzuca więcej
niż jedno material binding. Test syntetyczny obejmuje dwa prymitywy i dwa
użyte materiały.

### CAPART i Cloak mają właściwy fixture wyposażenia

Zwykły fixture Item V1 nadal umieszcza dokładnie jeden UTI na ziemi przed
wejściem. CAPART i Cloak korzystają z osobnego
`build_item_equipped_proof_module_v2`:

- MOD zawiera dokładnie jedno stworzenie w GIT i odpowiadający mu UTC;
- lista ground-itemów jest pusta;
- UTI CAPART jest w `Equip_ItemList` o native struct id `2`;
- UTI Cloak jest w `Equip_ItemList` o native struct id `8192`;
- GIT i UTC przechodzą semantic readback tego samego slotu i resrefu UTI;
- GIT, UTC i raport utrwalają ten sam jawny kontekst stworzenia:
  `Appearance_Type`, `Race`, `Gender` i `Phenotype`;
- Worker rozwiązuje dokładny wiersz `appearance.2da`, wymaga `MODELTYPE=P`,
  zgodności `RACIALTYPE` z `Race` oraz wyprowadza prefix z
  `RACE/Gender/Phenotype`; nie zakłada na sztywno `pmh0`;
- raport utrwala fixture profile i resref stworzenia, ale nie deklaruje
  widoczności modelu.

Studio alokuje osobny, collision-safe resref UTC typu `2027` i pokazuje jawne
pola `Appearance`, `Race`, `Gender` i `Phenotype`.

### Testy retail nie przechodzą bez wykonania

Testy zależne od extractu/instalacji są jawnie `ignored` bez środowiska zamiast
kończyć się cichym `return`. Uruchomiono je jawnie przeciwko:

`C:\Projects\New Folder\item-retail-extract`

Wynik `4 PASS` obejmuje:

- dokładny hash i macierz aktywnych rekordów `baseitems.2da`;
- semantic round-trip siedmiu retailowych UTI;
- `IPRP_SPELLS`;
- Cloak V2;
- wszystkie 19 pól CAPART przez właściwe retailowe `parts_*`;
- rzeczywiste payloady MDL/PLT Cloak i CAPART odczytane z KEY/BIF.

## Weryfikacja offline

- `cargo test -p m2a-core --test item`: `25 PASS`;
- dokładne retail tests z trzema zmiennymi środowiska i `--ignored`:
  `4 PASS`;
- `cargo test -p m2a-wasm item`: `1 PASS`;
- `npm test`: `228 PASS`, `1 skipped`;
- `npm run typecheck`: PASS;
- browser Worker/WASM Item integration: `2 PASS`;
- `cargo clippy -p m2a-core -p m2a-wasm --lib -- -D warnings`: PASS;
- produkcyjny build WASM/Vite: PASS;
- statyczny mock Source/Review sprawdzony wizualnie w przeglądarce; konsola
  bez błędów.

Pełny niezawężony runner ma niezależne od Item niepowodzenia, ponieważ ten
worktree nie zawiera Git-ignorowanego payloadu
`sample-3d/h2-clockwork-sentinel-1500/source.glb`. Nie kopiowano ani nie
podstawiano tego modelu z innej lokalizacji. Testy i buildy Item są zielone.

## Wynik ponownego audytu

Nie pozostała znana luka P0/P1 w zakresie tego celu. Granica końcowa pozostaje
celowo poza agentem: nowy CAPART/Cloak MOD/HAK musi zostać zmaterializowany,
zainstalowany byte-identycznie i oceniony wizualnie przez właściciela. Ten cel
nie tworzył takiego kandydata, ponieważ model iteration gate nie dopuścił nowej
iteracji.

Zamrożony `item-composed-r01-20260730` nie został dotknięty. Jego kanoniczne i
natywnie zainstalowane MOD/HAK zachowują wcześniejsze SHA-256.

## Amendament po audycie kontraktu retail — 2026-07-30

Ten rozdział zastępuje wcześniejsze opisy Cloak V3, stałego kontekstu `pmh0`
oraz walidacji `BYTE_HASH_VERIFIED` w tym dokumencie.

### Rzeczywisty kontrakt Cloak i CAPART

Dokładny odczyt `nwn_base.key` o SHA-256
`09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935`
potwierdził dla wiersza CloakModel `1` trzy odrębne zasoby:

```text
2002:<prefix>_cloak_001.mdl
6:cloak_001.plt
6:icloak_<gender>_001.plt
```

`<prefix>` nie jest stałą. Worker wyprowadza go z dokładnego wiersza
`appearance.2da` jako `p + Gender + RACE + Phenotype`, sprawdza
`MODELTYPE=P` oraz zgodność `RACIALTYPE` z polem `Race` fixture'u. Ten sam
prefix i SHA-256 tabeli Appearance są zapisane w tożsamości MOD i readbacku
wyposażonego stworzenia.

Retailowy `pmh0_cloak_001.mdl` jest poprawnym ASCII MDL wyeksportowanym przez
NWmax, a nie binarnym MDL writerowym. Własny parser sprawdza:

- nagłówek i envelope modelu;
- zgodność `newmodel`/`beginmodelgeom`/`endmodelgeom` z resrefem;
- granice `node`/`endnode`;
- deklarowane `verts` i `faces`, skończone współrzędne oraz indeksy w zakresie;
- liczbę trójkątów renderowanej geometrii.

Ten sam publiczny inspektor nadal obsługuje binarne MDL przez istniejący pełny
readback writerowy. PLT jest walidowany strukturalnie: nagłówek `PLT V1`,
rozmiary, długość payloadu, pole zarezerwowane i indeks palety. Liczba palet
jest deklarowana przez plik w zakresie `1..10`; dokładna tekstura
`cloak_001.plt` deklaruje `10`, natomiast dokładna ikona
`icloak_m_001.plt` deklaruje `6`.

Każdy widoczny part CAPART wymaga dwóch wpisów pod tym samym resrefem:
MDL typu `2002` i PLT typu `6`. Dokładny retailowy
`pmh0_footr001.mdl/.plt` przechodzi ten sam inspektor. Part zamaskowany przez
robe nie udaje brakującego zasobu — ma jawny status `SKIPPED_BY_ROBE_MASK`.

### Proweniencja i budżet

Studio wymaga jednego manifestu JSON, który dla każdego dostarczonego
MDL/PLT wiąże:

- przypięty `nwn_base.key` i jego SHA-256;
- dokładny klucz `resourceType:resref`;
- nazwę pliku i SHA-256 jego bajtów;
- locator `bif:<index>:resource:<index>`;
- SHA-256 samego manifestu.

Worker odrzuca brakujące, dodatkowe, zduplikowane i niezgodne wpisy. Core
samodzielnie sprawdza hash payloadu i jego format; końcowy status brzmi
`PINNED_MANIFEST_PAYLOAD_VALIDATED`, a nie `BYTE_HASH_VERIFIED`.
Test zależny od lokalnej instalacji czyta KEY/BIF tylko do odczytu i potwierdza
zgodność tego kontraktu z rzeczywistymi zasobami Cloak oraz CAPART.

Budżet `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000` obejmuje teraz sumę
trójkątów własnych partów Meshy i wszystkich widocznych, rozwiązanych modeli
referencyjnych CAPART/Cloak. PLT nie wnosi geometrii.

### Pozostałe poprawki audytu

- zajęta przestrzeń nazw obejmuje typy numeryczne, HAK i MOD; wadliwy wpis jest
  fail-closed już w allocatorze UI, a każda kolizja wyjściowa jest ponownie
  sprawdzana w Workerze;
- dla trzyczęściowego modelu Worker mierzy wszystkie trzy pary; sąsiednie
  muszą się stykać w tolerancji, a niesąsiednie nie mogą się nakładać;
- wybór `sourceNode` filtruje i remapuje IR przed walidacją materiałów, więc
  nieużywany root domyślnej sceny nie powoduje fałszywego błędu wielu
  materiałów;
- fixture CAPART/Cloak jest wyposażony i związany z dokładnym wierszem
  Appearance zamiast polegać na hardkodowanym założeniu `pmh0`.

### Aktualna weryfikacja offline

- `cargo test -p m2a-core --test item`: `25 PASS`;
- dokładne testy retail z `--ignored`: `4 PASS`, w tym odczyt realnych
  MDL/PLT Cloak i CAPART z KEY/BIF;
- skoncentrowane testy UI Item: `17 PASS`;
- Item Worker/WASM integration: `2 PASS`, łącznie z negatywnymi przypadkami
  fałszywego manifestu, zajętego UTI, kolizji pary niesąsiedniej i niezgodnego
  `RACIALTYPE`;
- TypeScript typecheck i produkcyjny build WASM/Vite: PASS.

Pełny test biblioteki WASM ma `32 PASS` i dwa niezależne błędy testów H2
spowodowane brakiem lokalnego, Git-ignorowanego
`sample-3d/h2-clockwork-sentinel-1500/source.glb`. Nie kopiowano ani nie
podstawiano tego modelu. Pełny `cargo test -p m2a-core` potwierdził między
innymi `83 PASS / 2 ignored` dla biblioteki i również zatrzymał się dopiero na
niezależnym `h2_r42_visibility_candidate`, wymagającym tego samego pliku.
Nie uruchamiano Toolsetu ani NWN i nie tworzono nowej iteracji kandydata.
