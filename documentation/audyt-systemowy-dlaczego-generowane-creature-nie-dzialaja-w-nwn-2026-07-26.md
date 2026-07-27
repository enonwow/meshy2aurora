# Audyt systemowy: dlaczego generowane creature nie działają w NWN

Data: `2026-07-26`

Status: `AUDIT UPDATED WITH CURRENT EE ENGINE TRACE / EXACT ROOT CAUSE OPEN / NOT READY FOR OWNER PROOF`

## 1. Werdykt

To nie jest już uczciwe przedstawiać jako potwierdzoną tezę, że `r43` znikał,
bo miał jeden rigid mesh, siedem stanów type 0, `supermodel NULL` albo brak
SkinMesh. Read-only analiza aktualnego klienta EE z tej instalacji pokazuje,
że dokładny `r43` przechodzi jawne warunki utworzenia instancji TriMesh i
końcowy predykat renderowalności.

Kolejne nieudane testy nadal ujawniają **trzy potwierdzone klasy problemów
procesu**:

1. historyczne modele creature nie spełniały poprawnej, kompletnej rodziny
   kontraktu animowanego modelu NWN;
2. moduły testowe mieszały wadliwy model z niezweryfikowanym, generowanym
   kontenerem creature, przez co kontrola Hook Horror również nie była czystą
   kontrolą natywną;
3. różne i niezgodne payloady wielokrotnie otrzymywały te same nazwy
   `m2a_codex_aproof.hak`, `m2a_codex_aproof.mod` oraz `m2a_m6p01`, więc
   bieżący eksport nie ma świeżej, bezkolizyjnej tożsamości kandydata.

Równocześnie testy offline potwierdzały głównie zgodność:

```text
nasz writer -> nasz parser -> nasz preview
```

Nie potwierdzały zgodności:

```text
nasz writer -> parser i renderer NWN
```

Dlatego build, readback, WASM, Worker i web preview mogły być zielone, a model
dalej nie był rysowany przez NWN.

Najkrótsza uczciwa odpowiedź brzmi:

> Wiemy, dlaczego proces seryjnie produkował fałszywie „gotowe” wyniki:
> używał skorumpowanego H1 jako wzorca poprawności, nie izolował modelu od
> kontenera creature, ponownie używał nazw różnych artefaktów i opierał
> akceptację na własnym readbacku. Nie znamy jednak jeszcze jednego
> artifact-level scalaru odpowiedzialnego za całkowity brak obrazu. Aktualny
> kod klienta nie odrzuca struktury TriMesh `r43` w sprawdzonych bramkach, a
> owner proof nie zawierał czystego kwadrantu
> `native profile + native appearance/model` ani runtime capture/logu.

## 2. Granice audytu

Audyt wykonano offline. Nie uruchamiano, nie przejmowano i nie sterowano Aurora
Toolset ani NWN. Podstawą są:

- exact owner results dla `r40`, `r41`, `r42` i zmodyfikowanego `r43`;
- readback MOD/HAK/MDL/2DA i kod produkcyjny;
- lokalne natywne oraz CEP-owe rodziny whole-model creature;
- rzeczywista, read-only zawartość zainstalowanego
  `m2a_codex_aproof.hak`;
- bieżąca ścieżka proceduralna Core/WASM/Worker/Studio.

Owner pozostaje właścicielem końcowego testu Toolset/NWN.

## 3. Co rzeczywiście pokazała historia kandydatów

| Kandydat | Toolset | NWN | Znaczenie |
|---|---|---|---|
| H1 v20 | draw, ale ciężko zdeformowany blob | draw skorumpowany | tylko świadek wejścia na draw path, nie wzorzec poprawnego creature |
| `r40` | `visible/verified`, 20 rozdzielonych sztywnych części | `not_visible/verified` | Toolset nie jest dowodem akceptacji runtime NWN |
| `r41` | bez rozstrzygającego nowego sukcesu | `not_visible/verified` | samo dodanie pełnej topologii stanów type 5 nie naprawiło sztywnej rodziny modelu |
| `r42` | `visible/verified` | `not_visible`, owner: żaden oczekiwany model nie był widoczny | jednocześnie zawiódł custom model i generowana „stock” kontrola |
| zmodyfikowany `r43` | scena i trzy creature obecne w Toolset | Meshy i Hook Horror `not_visible`; custom appearance na natywnym Driderze nie działa | równocześnie aktywne były defekty modelu/custom appearance oraz fixture |
| bieżąca ścieżka proceduralna | zweryfikowana tylko offline | `not_tested` | nie wolno jej przedstawiać jako naprawionej w NWN |

`r43` nie zawierał pełnej kontroli `native container + native appearance`.
Zawierał:

- generowany GIT/UTC + custom appearance;
- generowany GIT/UTC + natywny appearance Hook Horror;
- natywny, kompletny profil Dridera + custom appearance.

Brakowało czwartego kwadrantu:

```text
natywny GIT/UTC + natywny appearance/model
```

To tłumaczy, dlaczego w jednym module „nic nie działało”: dwie osie były
wadliwe albo niezamknięte jednocześnie. Hook Horror nie izolował modelu, a
Drider po ręcznej zmianie `Appearance_Type=15100` nie izolował kontenera.

## 4. Potwierdzone przyczyny

### SYS-NWN-01 — skorumpowany H1 został uznany za wzorzec poprawności

Pewność: **potwierdzone**.

H1 v20 narysował w NWN wielki, silnie zdeformowany blob. Ten wynik dowodził
wyłącznie, że określony payload dochodził do draw path. Nie dowodził poprawnej
geometrii, transformacji, animacji, materiału ani skina.

Mimo tego `r43` świadomie odtworzył rodzinę H1:

- jeden duży rigid mesh;
- siedem lokalnych stanów type 0;
- `supermodel NULL`;
- root/state topology H1.

W praktyce pipeline optymalizował się do reprodukcji znanej korupcji.

### SYS-NWN-02 — historyczne modele nie należały do referencyjnej kompletnej rodziny creature

Pewność braku kompletności animacyjnej: **potwierdzone**.  
Pewność, że był to powód całkowitego no-draw: **niepotwierdzone i
sprzeczne z jawnymi bramkami aktualnego klienta EE**.

Lokalne poprawne whole-model creature pokazują dwie użyteczne rodziny:

1. model samowystarczalny, np. `c_Direwolf`, `c_Horror` i CEP R3
   `c_phod_horror_b`: pełne 42 lokalne stany type 5;
2. model dziedziczący zgodny szkielet, np. `c_kocrachn`: zero lokalnych
   stanów, lecz jawny kompatybilny supermodel `c_Horror`.

H2 ma własny, niekompatybilny szkielet, więc nie może bezpiecznie przejąć
animacji `c_Horror`. Historyczne H1/r40/r41/r42/r43 nie miały takiej
kompletności. Jest to realny brak jakości i zachowania animowanego creature.
Nie jest to jednak udowodniona bramka samego draw: aktualne
`MdlNodeTriMesh::InternalCreateInstance`, `CreateInstanceTreeR`,
`PartTriMesh::IsPartRenderable` i `PartTriMesh::Draw` nie sprawdzają liczby
animacji, SkinMesh ani supermodelu, a H1 v20 dochodził do draw path jako
siedmiostanowy rigid model.

### SYS-NWN-03 — Toolset visibility została potraktowana zbyt optymistycznie

Pewność: **potwierdzone**.

`r40` i `r42` były widoczne w Toolset, a niewidoczne w NWN. Toolset może
zaakceptować i wyświetlić model, którego runtime NWN nie narysuje. Widoczność
Toolsetu jest osobnym gate'em, nie substytutem testu NWN.

### SYS-NWN-04 — kontrola Hook Horror nie była kontrolą natywną end-to-end

Pewność: **potwierdzone**.

Hook Horror używał natywnego appearance/modelu, ale był osadzony w generowanym
minimalnym GIT/UTC. Wygenerowane instancje historycznie miały między innymi:

- `Race=0`;
- `FactionID=2`;
- `Interruptable=0`;
- `WalkRate=0`;
- `PerceptionRange=0`;
- minimalne HP;
- `Class=12`;
- brak domyślnych skryptów AI.

Sam brak któregoś z tych pól nie jest udowodnionym uniwersalnym powodem
no-draw. Łącznie jednak ten payload nie był niezależną, retailową kontrolą.
Jego porażka dowodzi, że testowy tor kontenera nie był czysty; nie dowodzi
automatycznie, że bazowy `c_horror.mdl` jest wadliwy.

### SYS-NWN-05 — test `r43` nie domknął macierzy A/B

Pewność: **potwierdzone**.

Natywny blueprint `x2_drider003` został ręcznie przełączony na customowy
`Appearance_Type=15100`. To był poprawny test custom appearance przy mocnym
kontenerze, ale nie był to natywny control. Bez
`native container + native appearance` moduł nie mógł odpowiedzieć na pytanie,
czy cały runtime scene/container jest zdrowy.

### SYS-NWN-06 — różne payloady używają jednej nazwy HAK/MOD/modelu

Pewność: **potwierdzone w kodzie i na dysku**.

Bieżący Core nadal deklaruje:

```text
model    m2a_m6p01
texture  m2a_m6t01
HAK      m2a_codex_aproof.hak
MOD      m2a_codex_aproof.mod
```

Worker przypisuje te same nazwy wszystkim niestatycznym lane'om, w tym nowej
ścieżce proceduralnej. Tych samych nazw używały wcześniej H1 v20, NeverBlender
reference oraz kontrola `c_squirrel`.

Read-only kontrola natywnego katalogu NWN w dniu audytu wykazała:

```text
C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_codex_aproof.hak
length: 8 277 166
sha256: 0506eacab54c785d3f1b1dd93f346272c677b8f7b36e802cefcbc003d0da008f
```

Wnętrze tego HAK-a zawiera:

- `appearance.2da`;
- `c_squirrel.mdl`;
- `c_squirrel.dds`;
- brak `m2a_m6p01`;
- brak `m2a_m6t01`.

Hash i zasoby dokładnie odpowiadają starej kontroli `c_squirrel`. Jednocześnie
`modules\m2a_codex_aproof.mod` jest nieobecny.

Skutki:

- obowiązkowa instalacja no-clobber nowego HAK-a pod obecną nazwą musi się
  zatrzymać na kolizji;
- ręczne zestawienie nowego MOD-a ze starą nazwą HAK-a może załadować stary
  `c_squirrel` payload zamiast nowego modelu;
- nazwa pliku nie identyfikuje już immutable lineage;
- obecny proceduralny output nie może otrzymać statusu
  `ready_for_owner_proof`.

To nie wyjaśnia historycznej porażki osobno nazwanych `r42/r43`, ale jest
potwierdzonym powodem, dla którego **bieżąca naprawiona ścieżka nadal nie jest
bezpiecznie testowalnym kandydatem**.

### SYS-NWN-07 — własny readback był mylony z walidacją silnika

Pewność: **potwierdzone**.

Writer oraz parser dzielą założenia i mogą być zgodne w tym samym błędzie.
CPU skin oracle i Three.js preview dowodzą, że nasza interpretacja danych jest
wewnętrznie spójna. Nie dowodzą, że NWN tak samo interpretuje:

- extended64 SkinMesh maps;
- inverse bind payload;
- offsets i streamy MDX;
- kontrolery type 5;
- eventy i node topology;
- wartości domyślne niewypełnionych pól.

Interfejs Studio sam poprawnie pokazuje `Runtime acceptance: OPEN_M6`, ale
wcześniejsze komunikaty i decyzje implementacyjne wyprzedzały ten gate.

### SYS-NWN-08 — exact owner-modified `r43` jest już immutable

Pewność: **potwierdzone**.

Obecny `m2a_h2r43.mod` ma `39 090` bajtów i otrzymał read-only SHA-256:

```text
a444722545f81da256d6733b10c71c830609939eb31df496a3db2e9e7e3f7f77
```

HAK `m2a_h2r43.hak` ma SHA-256:

```text
d4381dbdc9aee5f0dab2b8159a414586a913b7ed9bf68fe48515a951cdebbdb1
```

Zainstalowany HAK jest bitowo identyczny z canonical source. Exact zapisany
MOD nadal wskazuje tylko `m2a_h2r43` w `Mod_HakList`, a jego wyekstrahowany GIT
ma SHA-256:

```text
40275d3d358f11bbe97b7947b7311e9b2e932f68e1244f44872bf4154f1ec71d
```

Usunięto więc wcześniejszy blocker hasha. Bramka nowej iteracji nadal pozostaje
zamknięta z innego powodu: nie ma jeszcze zdiagnozowanej minimalnej delty,
ponieważ aktualny klient dopuszcza `r43` do draw, a scena nie domknęła czystej
macierzy A/B.

## 5. Otwarte, konkretne luki bieżącej implementacji

Poniższe punkty nie są już ogólnym „strzelaniem”. Są dokładnymi kontraktami,
których bieżący output nadal nie zamyka albo nie ma dla nich dowodu NWN.

### SYS-NWN-09 — custom `appearance.2da` nadal jest sparse

Pewność luki: **potwierdzone**. Wpływ na no-draw jako pojedynczy scalar:
**nieudowodniony**.

Produkcja wypełnia tylko dziewięć kolumn:

```text
LABEL, MOVERATE, MODELTYPE, RACE, PORTRAIT, ENVMAP,
BLOODCOLR, WEAPONSCALE, SIZECATEGORY
```

Pozostałe kolumny są puste. Poprawne whole-model rows zwykle określają także
między innymi movement distances, personal space, height, hit distance,
preferred attack distance, target height, parry behavior, racial type,
legs/arms, perception, footsteps, sound, head tracking i targetability.

Nie można nazwać sparse row jedyną przyczyną, bo Hook Horror na stock row 102
również zawiódł z generowanym GIT. Nie wolno też uznać obecnego custom row za
zamknięty kontrakt, dopóki nie zostanie porównany i zmaterializowany jako pełny
whole-model profile.

### SYS-NWN-10 — `ActiveMonsterBaseline` jest syntetyczny

Pewność luki: **potwierdzone**.

Nowy profil jest znacznie mocniejszy od legacy fixture: ustawia sensowne race,
faction, walk rate, perception, HP, klasę i domyślne AI scripts oraz jest
odczytywany niezależnie z GIT i UTC. Nadal jest profilem clean-room złożonym
przez projekt, a nie exact natywnym UTC/GIT. Ma dowody offline, ale jeszcze nie
ma własnego runtime-positive control.

### SYS-NWN-11 — engine semantics nowego SkinMesh pozostają nieudowodnione

Pewność luki: **potwierdzone**.

Nowy proceduralny output ma:

- controllerless model root;
- bezpośredni SkinMesh pod rootem;
- ważony szkielet pod rootem;
- 42 odrębne stany type 5;
- 23 callbacki gameplay;
- znormalizowaną skalę;
- poprawny bind reconstruction i pięć niezerowych deformacji offline.

To usuwa znane błędy historycznych rodzin. Nie stanowi jeszcze dowodu, że
konkretne binarne mapy SkinMesh/MDX i kontrolery są akceptowane przez NWN.

### SYS-NWN-12 — parser ma węższe pokrycie niż rzeczywisty silnik

Pewność: **potwierdzone historycznym audytem korpusu**.

Lokalny reader obsługuje wybrany podzbiór rodzin natywnych. To jest właściwe
dla fail-closed implementacji, ale oznacza, że „reader PASS” nie reprezentuje
pełnego zbioru semantyk, które rozumie NWN. Braku coverage nie wolno zamieniać
w pozytywną certyfikację emitera.

### SYS-NWN-13 — UI/artefakty nie wymuszają unikalnej tożsamości

Pewność: **potwierdzone**.

Nazwy artefaktów są hard-coded według szerokiej klasy lane, a nie wyliczane z
immutable candidate identity. Brakuje produkcyjnego gate'a, który przed
downloadem/instalacją dowodzi, że:

- MOD wskazuje dokładnie zamierzony HAK;
- HAK zawiera dokładnie zamierzony MDL/TGA/2DA;
- filename/resref nie był użyty przez inny payload;
- source i destination mają zgodne, zapisane hashe.

### SYS-NWN-14 — jedna scena próbowała diagnozować zbyt wiele osi

Pewność: **potwierdzone**.

`r42/r43` jednocześnie zmieniały lub porównywały model, topology, animation
family, appearance, generated container i Toolset-repacked MOD. Taki test
wykrywa „nie działa”, ale nie identyfikuje jednego minimalnego delta.

### SYS-NWN-15 — dowody procesu były traktowane jak dowody obrazu

Pewność: **potwierdzone**.

Build PASS, HAK attachment, MOD readback, poprawne miejsce fixture, Toolset
preview, web screenshot lub hash nie dowodzą widoczności w NWN. Są niezbędnymi
warunkami lineage, ale nie zastępują owner-bound runtime verdict.

## 6. Hipotezy wykluczone jako samodzielna przyczyna

Zebrane wyniki nie wspierają następujących pojedynczych wyjaśnień:

- „uruchomiono niewłaściwy moduł/Area” dla zmodyfikowanego `r43`;
- brak exact HAK/MDL/TGA w `r42/r43`;
- sam triangle count;
- sam `meshType`;
- same vertex colors;
- SkinMesh jako uniwersalny wymóg każdego draw;
- `supermodel NULL` jako jedyny problem;
- sam animation header type 0 albo type 5;
- sama liczba siedmiu stanów jako kompletne wyjaśnienie total no-draw;
- same bounds/radius;
- sama alpha tekstury;
- same ostrzeżenia `Invalid Class`;
- sam `MaxHitPoints`, `SkillList`, `WalkRate` albo `PerceptionRange`;
- sam sparse custom appearance row;
- sama nieobecność kolizji.

Wykluczenie pojedynczego scalaru nie oznacza, że częściowe kontrakty są dobre.
Historyczny problem był kombinacją niepełnego modelu, nieczystego fixture i
błędnej metody akceptacji.

## 7. Co dokładnie robi aktualny klient EE z `r43`

Źródło: read-only statyczna analiza zainstalowanego klienta:

```text
C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\bin\win32\nwmain.exe
build pliku: 2026-01-22
PE: x64
PDB identity: C:\Jenkins\workspace\Build Windows\vs2017\game\nwmain\Release\nwmain.pdb
```

### 7.1 Parser i roundtrip

Exact `r42`, H1 v20 i `r43`:

- dekompilują się niezależnym `nwnmdlcomp`;
- ponownie kompilują się z kodem wyjścia `0`;
- zachowują rozmiar core MDL w roundtripie;
- mają legalne indeksy, brak indeksów poza zakresem i brak degeneratów;
- mają zgodny winding oraz face normals;
- mają skończone bounds, niezerową geometrię i legalne strumienie MDX.

To wyklucza ogólne stwierdzenie „parser nie potrafi odczytać pliku”.

### 7.2 Utworzenie instancji

`MdlNodeTriMesh::InternalCreateInstance` pod
`0x140091B80`:

1. alokuje `PartTriMesh`;
2. kopiuje node surface/material field;
3. ustawia `PartTriMesh + 0x9e` na wynik
   `MdlNodeTriMesh.render != 0`;
4. zwraca instancję.

`r43` ma `render=1`, więc jego part otrzymuje flagę enabled.

`CreateInstanceTreeR` pod `0x14008DAB0`:

- wywołuje wirtualne `InternalCreateInstance` dla każdego istniejącego node;
- rekurencyjnie instancjonuje wszystkie dzieci;
- nie filtruje drzewa według SkinMesh, liczby animacji, typu stanów ani
  supermodelu.

### 7.3 Końcowy predykat renderowalności

`PartTriMesh::IsPartRenderable` pod `0x1400A2EB0` zwraca true, gdy:

```text
part.enabled != 0
node != null
node.render != 0
node.vertexCount != 0 OR node.alternateVertexData != null
```

Exact `r43` spełnia te warunki:

```text
part.enabled  = 1 po InternalCreateInstance
node.render   = 1
vertexCount   = 2678
faces         = 1543
maxIndex      = 2677
```

`PartTriMesh::Draw` pod `0x1400A24C0` powtarza te same cztery warunki, po czym
dochodzi do jednego z konkretnych rendererów materiału. Nie zawiera dodatkowej
bramki „42 animacje”, „SkinMesh” ani „supermodel != NULL”.

### 7.4 Ładowanie listy creature

`CNWSArea::LoadCreatures` pod `0x140360570`:

- czyta `Creature List`;
- wymaga struct ID `4`;
- iteruje każdy element niezależnie;
- błędny element usuwa i przechodzi do następnego, zamiast przerywać całą
  listę.

Exact GIT ma trzy elementy, każdy ze struct ID `4`. Wygenerowane dwa elementy
mają 70 pól; zapisany przez Toolset Drider ma także 70 pól.

`CNWSCreature::LoadCreature` zwraca failure wyłącznie, gdy
`CNWSCreatureStats::ReadStatsFromGff` zwróci kod błędu. Jawne błędy tej funkcji
obejmują między innymi:

- `0xF912`: `Race` poza zakresem;
- `0xF913`: zakazana wartość perception range `12`;
- `0xF914`: istniejący, ale pusty `ClassList`;
- `0xF915`: nieprawidłową lub niespójną klasę;
- `0xF916`: brak wymaganej klasy w ścieżce bez poprawnej listy.

Exact wartości sceny:

```text
generated H2: Race=0, PerceptionRange=0, Class=12 (Animal), ClassLevel=12
generated Hook: Race=0, PerceptionRange=0, Class=12 (Animal), ClassLevel=12
Toolset Drider: Race=7, PerceptionRange=11, Class=11/9
```

Wszystkie klasy są legalnymi `CLASS_TYPE_*`, listy nie są puste, a wartości
Race i PerceptionRange nie trafiają w powyższe błędy.

To nie znaczy, że historyczne `PerceptionRange=0` jest poprawnym authoringiem.
Oficjalny `Creature_Format.pdf` firmy BioWare wymaga zakresu 9–13.
`PerceptionRange=0` jest więc potwierdzonym brakiem generated profilu, ale nie
jest potwierdzoną przyczyną no-draw w tym exact kliencie: jego jawna bramka
odrzuca wartość 12, nie 0. Produkcyjny `ActiveMonsterBaseline` używa już
wartości 11.

### 7.5 Twardy wniosek z engine trace

Z exact bajtów i aktualnego kodu klienta wynika:

> Jeżeli `m2a_h2p43.mdl` zostanie rozwiązany i podłączony do model instance,
> jawne bramki TriMesh nie odrzucą go za brak SkinMesh, brak 42 stanów,
> `supermodel NULL`, liczbę wierzchołków, indeksy, winding, bounds ani
> `render`.

Z exact GIT i loadera wynika również:

> Błąd pierwszego wygenerowanego creature nie zatrzymałby automatycznie
> stockowego Hook Horrora ani trzeciego Dridera.

Dlatego obecny dowód nie pozwala nazwać błędu MDL ani minimalnego GIT
potwierdzonym powodem, że na ekranie nie było żadnego modelu. Pozostaje
niezwiązana granica runtime:

- rzeczywiste rozwiązanie resource/appearance i utworzenie client model
  instance;
- wysłanie/obecność creature object po stronie klienta;
- widok/camera oraz candidate-bound kadr;
- brak zachowanego logu/capture dokładnego runu `r43`.

To nie jest lista swobodnych hipotez. To jedyne granice znajdujące się przed
udowodnionym, przechodzącym predykatem draw. Rozstrzyga je czysty A/B lub
runtime trace, nie kolejna zmiana geometrii.

## 8. Dlaczego placeable widać, a creature nie

Placeable i creature nie przechodzą przez ten sam runtime contract.

Placeable może narysować statyczny rigid MDL bez:

- creature animation state machine;
- szkieletu i SkinMesh;
- map inverse-bind/MDX dla skina;
- creature `appearance.2da`;
- GIT/UTC profilu aktora;
- wymaganych nazw stanów i event callbacks.

Widoczny placeable dowodzi, że podstawowe zasoby HAK, geometria, materiał i
część writerów potrafią działać. Nie dowodzi poprawności creature pipeline.
Brak PWK wyjaśnia brak kolizji placeable, ale nie niewidoczność creature.

W przypadku exact `r43` nie wolno już jednak używać tej ogólnej różnicy jako
gotowej diagnozy. Końcowy predykat aktualnego klienta dla jego rigid TriMesh
jest spełniony. Różnica placeable/creature zawęża więc problem do wcześniejszej
ścieżki appearance/object/resource/scene, dopóki runtime trace nie pokaże
inaczej.

Pełny, źródłowy rozdział kontraktu znajduje się w
[Kontrakcie runtime creature MDL w NWN](nwn-creature-mdl-runtime-contract-2026-07-26.md).

## 9. Co obecna implementacja już naprawiła

Offline wdrożono:

- usunięcie produkcyjnego fallbacku do corrupted `H1_SKINNED`;
- proceduralne 42 state type 5;
- 23 callbacki;
- controllerless Aurora root i direct-root SkinMesh;
- ważony szkielet oraz normalizację skali;
- kontrolę bind pose, deformacji i grounded bounds;
- jeden `ActiveMonsterBaseline` creature bez H1/Hook Horror w nowym module;
- routing Core/WASM/Worker/Studio;
- niezależny binary readback i web readback.

To są realne poprawki braków znalezionych w audycie. Nie rozwiązują jeszcze:

- kolizyjnej tożsamości `m2a_codex_aproof`;
- sparse custom appearance;
- braku natywnej, pełnej kontroli kontenera;
- braku owner proof exact nowych bajtów w NWN.

## 10. Definition of Done dla następnego kandydata

Nie wystarczy kolejny zielony build. Następny kandydat może otrzymać
`ready_for_owner_proof` dopiero, gdy:

1. exact 39 090-byte `m2a_h2r43.mod` pozostaje przypisany do owner failure
   hashem
   `a444722545f81da256d6733b10c71c830609939eb31df496a3db2e9e7e3f7f77`;
2. istnieje jedna świeża, kolizyjnie wolna tożsamość MOD/HAK/model/texture/2DA,
   której wcześniej nie używał żaden inny payload;
3. materializacja używa aktualnej proceduralnej SkinMesh/42-state ścieżki, a
   nie H1/r43;
4. custom whole-model appearance ma jawnie zdefiniowany pełny profil;
5. moduł zawiera czystą macierz kontroli albo minimalną jedną fixture z osobno
   już runtime-potwierdzonym kontenerem;
6. MOD wskazuje exact HAK, a HAK zawiera exact MDL/TGA/2DA;
7. source i native destination są hash-identyczne po instalacji no-clobber;
8. owner otrzymuje najpierw exact nazwę `.mod`, potem nazwę modułu w Toolset,
   Area, object, HAK, appearance row i placement;
9. ta sama immutable lineage jest sprawdzana kolejno w Toolset i NWN;
10. tylko owner result może zamknąć `modelVisibility` i animacje.

## 11. Stan końcowy audytu

Pełna implementacja nie jest dziś gotowa do weryfikacji w NWN.

Stan osi:

```text
historyczne r40/r41/r42/r43:
  modelVisibility (NWN) = failed/not_visible według exact owner results

bieżący proceduralny output:
  modelVisibility (Toolset) = not_tested
  modelVisibility (NWN)     = not_tested
  proofCompleteness         = missing
  ready_for_owner_proof     = false
```

Najbliższy twardy blocker nie jest pytaniem „jaki controller dodać” ani
„czy dodać 42 animacje”. Hash zmodyfikowanego `r43` jest już odzyskany, a jego
TriMesh przechodzi jawne bramki aktualnego klienta. Przed nową iteracją trzeba
zamknąć czysty control boundary:

1. owner-test istniejącego stock-only `m2a_van02.mod`, który izoluje
   generowany kontener bez custom modelu; albo
2. candidate-bound runtime capture/log/trace pokazujący, na której wcześniejszej
   granicy exact `r43` nie tworzy model instance.

Bez tego zastąpienie `r43` nowym SkinMesh/42-state profilem byłoby ponownie
implementacją na podstawie korelacji, a nie poznanej przyczyny.

## 13. Aktualizacja wykonawcza: świeży produkcyjny r44

Po odrzuceniu przez właściciela starego, runtime-niepotwierdzonego
`m2a_van02` jako drogi do rozwiązania wdrożono brakującą caller-owned
tożsamość produkcyjnego proceduralnego pakietu. Historyczne, kolizyjne
`m2a_m6p01`/`m2a_codex_aproof` nie są już wymagane do zamrożenia świeżej
lineage.

Zmaterializowano i zainstalowano exact:

```text
m2a_h2r44.mod
m2a_h2r44.hak
model m2a_h2p44
Appearance row 15219
```

Kandydat używa jednego `ActiveMonsterBaseline`, direct-root SkinMesh,
ważonego szkieletu, 42 stanów type 5 i 23 callbacków. Pełny workspace test
przechodzi. Status i hashe znajdują się w
[handoffie r44](evidence/h2-r44-procedural-full-native-ready-for-owner-proof-2026-07-26.md).

Ta aktualizacja nie zmienia historycznego wniosku o braku exact runtime scalar
r43 i nie deklaruje sukcesu NWN. Ustanawia nową granicę wykonawczą:
`r44 ready_for_owner_proof`; widoczność zamyka wynik właściciela exact
zainstalowanej lineage.

## 12. Powiązane dowody

- `documentation/audyt-niewidocznego-creature-w-nwn-2026-07-25.md`
- `documentation/evidence/h2-r43-owner-added-native-control-result-2026-07-26.json`
- `documentation/evidence/h2-r43-deep-model-visibility-audit-2026-07-24.md`
- `documentation/evidence/h2-r42-owner-toolset-nwn-visual-result-2026-07-24.json`
- `documentation/evidence/m0-r40-owner-visual-result-2026-07-24.json`
- `documentation/evidence/m0-r41-owner-nwn-visual-result-2026-07-24.json`
- `documentation/evidence/lc-c-squirrel-reference-control-v1-2026-07-18.md`
- `crates/m2a-core/src/model_pipeline.rs`
- `crates/m2a-core/src/proof_module.rs`
- `apps/studio-web/src/worker/m2a.worker.ts`

## 14. Aktualizacja po właścicielskim wyniku r44: poznana wada ABI SkinMesh

Ta sekcja zastępuje wcześniejszy bieżący status audytu. Właściciel przetestował
dokładny `m2a_h2r44.mod` i podał świeży wynik:

```text
NWN modelVisibility = not_visible
proofCompleteness   = failed
```

Wynik jest związany z exact MOD/HAK/model:

```text
m2a_h2r44.mod  SHA-256 721f4653cc7c546fe6c2ed26df36e7643909a267c4c3f0bed061077ee578a315
m2a_h2r44.hak  SHA-256 c2a089f0fd6ee50ce186a6495f992b0fbb0a8f348dcca7406ddd670849f21ac6
m2a_h2p44.mdl  SHA-256 41138bf8b5ce94e04607ef119752f45d6800a107a6ee6f282888317908e6eb80
```

Po tym wyniku zbadano dokładny binarny MDL i odczytano natywny korpus
SkinMesh z zainstalowanego `cep3_core1.hak`. Rozkład `1437` natywnych węzłów
SkinMesh był następujący:

```text
0 base controllerów:    0
2 base controllery:   332
3 base controllery:   962
więcej niż 3:         143
```

Minimalny natywny profil dwóch controllerów to:

```text
type 8  position
type 20 orientation
```

Dokładny SkinMesh r44 miał pustą tablicę controller keys, pustą tablicę
controller data i `0` zdekodowanych base controllerów. Ta sama wada występowała
w historycznej generowanej rodzinie skinned r33-r39. Jednocześnie generowany
rigid TriMesh może być rysowany bez controllerów, dlatego działający placeable
nie potwierdza poprawności ABI skinned creature.

Potwierdzona wada emitera:

```text
M2A-NWN-SKIN-BASE-CONTROLLERS-MISSING
```

Wykluczono jako pojedynczą przyczynę r44:

- przekroczenie limitu kości — r44 ma 22 aktywne kości, a bieżąca ścieżka
  NWN:EE obsługuje do 64 na SkinMesh;
- błędną oś lub położenie pod podłogą — exact emitowany MDL jest Z-up,
  uziemiony i ma zakres wysokości około `0.0..1.7`;
- odwrócony winding;
- brak podstawowego profilu creature w GIT/UTC;
- niespójność Appearance, HAK, model resref albo placementu.

## 15. Minimalna implementacja r45

Emitter binarnego MDL został zmieniony tak, aby każdy bazowy węzeł SkinMesh
otrzymywał:

```text
position    type 8   [0,0,0]
orientation type 20  [0,0,0,1]
```

Semantic readback i testy regresyjne odrzucają teraz generowany SkinMesh bez
tego minimalnego profilu. Rigid TriMesh/placeable nie został zmieniony.

Zamrożony kandydat:

```text
MOD filename:  m2a_h2r45.mod
Module Name:   Meshy2Aurora procedural humanoid proof
Area:          Meshy2Aurora M0 binary vertical-slice area
HAK:           m2a_h2r45
Appearance:    15219
Model:         m2a_h2p45
```

Porównanie semantyczne exact r44/r45 zachowało geometrię, indeksy, normalne,
UV, mapy i wagi SkinMesh, referencje kości, bindy, animacje, GFF, Appearance i
placement. Jedyna zamierzona funkcjonalna różnica SkinMesh to `0 -> 2` bazowe
controllery.

MOD i HAK zostały zainstalowane create-new/no-clobber i zweryfikowane
źródło-destination:

```text
m2a_h2r45.mod  SHA-256 c6ce6e814fcd56b0ce8e86738b4cba4b01174065689f5a546d6206f9adc4955e
m2a_h2r45.hak  SHA-256 de8adc62b0dedadf0485f7a6072e9e5ce235255fdc20c7e0beef3182362e7937
m2a_h2p45.mdl  SHA-256 ba10e8b3221e5748a7c8072e3364815ed5e932adb03d45234bb4106c98a8e1dc
```

Stan końcowy po stronie agenta:

```text
r45 ready_for_owner_proof = true
Toolset modelVisibility   = not_tested
NWN modelVisibility       = not_tested
```

Nie jest to deklaracja sukcesu wizualnego. Tylko wynik właściciela dla exact
`m2a_h2r45.mod` może zamknąć etap widoczności albo — przy `not_visible` —
otworzyć bramkę kolejnej diagnozy.

Dowody:

- `documentation/evidence/h2-r44-owner-nwn-visual-result-2026-07-26.json`
- `documentation/evidence/h2-r45-skin-bind-controllers-ready-for-owner-proof-2026-07-26.md`
- `proof-output/h2-r45-skin-bind-controllers-20260726/ready-for-owner-proof.json`

## 16. Korekta po wyniku owner proof r45

Właściciel przetestował exact `m2a_h2r45.mod` i podał
`NWN modelVisibility=not_visible`, `proofCompleteness=failed`.

To falsyfikuje twierdzenie z sekcji 14, że brak dwóch bazowych controllerów
SkinMesh był poznaną przyczyną całkowitej niewidoczności. Był to potwierdzony
defekt zgodności, ale jego minimalna naprawa w r45 nie przywróciła widoczności.
Nie wolno go dalej nazywać `confirmedRootCause`.

Aktualny pełny audyt exact r45, lista wykluczonych przyczyn i potwierdzonych
defektów znajdują się w:

[Audyt exact r45 po wyniku „dalej pusto”](audyt-r45-dalej-pusto-w-nwn-2026-07-26.md).

Najwęższa udowodniona granica po r45:

```text
owner-bound NWN absence
-> custom appearance/model resolution lub client creature scene attachment
-> jeżeli PartSkin powstanie, exact r45 przechodzi jawne bramki draw
```

`r46` nie został utworzony. Potwierdzone niezależne defekty pozostające do
naprawy to sparse direct-S appearance oraz nienatywna numeracja
`model root=part 24` zamiast `part 0`. Statyczny audyt nie udowodnił, który z
nich pojedynczo powoduje totalny no-draw, więc żaden nie jest jeszcze nazwany
poznaną root cause.

## 17. Zamknięcie widoczności przez owner proof r46

Właściciel przetestował exact `m2a_h2r46.mod` 2026-07-27 i podał
`ładnie zadziałało <3`, wraz ze świeżymi obrazami Toolsetu i NWN.

Wynik:

```text
Toolset modelVisibility   = visible
Toolset proofCompleteness = missing

NWN modelVisibility       = visible
NWN proofCompleteness     = verified
```

Toolsetowy packet pozostaje formalnie niepełny tylko dlatego, że obraz nie
pokazuje zaznaczenia/readbacku exact obiektu. Tytuł okna identyfikuje jednak
`m2a_h2r46.mod`, Area jest właściwe, a model jest widoczny. Rozstrzygający
wynik NWN pokazuje proceduralnego humanoida bezpośrednio przed graczem w
czytelnej scenie.

r46 zachował source GLB, teksturę, geometrię, skin, 42 stany type 5,
23 callbacki, bazowe controllery SkinMesh, profil creature,
`Phenotype=INT 0`, Area oraz placement. Zmienił równocześnie:

1. model i animation roots na natywne root-first `part 0`, z pełnym
   przemapowaniem hierarchii i skin maps;
2. sparse direct-S appearance na pełny klon 35 komórek działającego donor
   row `102`, ze zmianą tylko `LABEL/RACE`;
3. resrefy na świeżą linię r46.

Udowodnione jest, że połączona naprawa dwóch pozostałych defektów r45
przywróciła widoczność w NWN. Ponieważ obie zmiany znalazły się w tym samym
dopuszczonym kandydacie, nie ma podstaw do nazywania tylko jednej z nich
samodzielną root cause. Brakujący phenotype pozostaje wykluczony.

Pełny wynik i zachowane obrazy:

- `documentation/evidence/h2-r46-owner-toolset-nwn-visual-result-2026-07-27.json`
- `documentation/evidence/h2-r46-owner-toolset-nwn-visual-result-2026-07-27.md`
- `documentation/evidence/h2-r46-owner-nwn-visible-2026-07-27.png`
- `documentation/evidence/h2-r46-owner-toolset-visible-2026-07-27.png`

Etap niewidoczności generowanego creature w NWN jest zamknięty sukcesem.
