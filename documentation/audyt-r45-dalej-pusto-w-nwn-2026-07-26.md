# Audyt exact r45 po wyniku „dalej pusto” w NWN

Data audytu: 2026-07-26

## 1. Wynik, którego nie wolno reinterpretować

Właściciel sprawdził exact kandydat:

```text
MOD:        m2a_h2r45.mod
Module:     Meshy2Aurora procedural humanoid proof
Area:       Meshy2Aurora M0 binary vertical-slice area
HAK:        m2a_h2r45
Appearance: 15219
Creature:   m2a_h2utc45
Model:      m2a_h2p45
```

Tożsamość immutable:

```text
m2a_h2r45.mod  c6ce6e814fcd56b0ce8e86738b4cba4b01174065689f5a546d6206f9adc4955e
m2a_h2r45.hak  de8adc62b0dedadf0485f7a6072e9e5ce235255fdc20c7e0beef3182362e7937
m2a_h2p45.mdl  ba10e8b3221e5748a7c8072e3364815ed5e932adb03d45234bb4106c98a8e1dc
```

Wynik:

```text
NWN modelVisibility = not_visible
proofCompleteness   = failed
```

Źródło związania wyniku:
`documentation/evidence/h2-r45-owner-nwn-visual-result-2026-07-26.json`.

## 2. Najważniejsza korekta poprzedniego audytu

Brak bazowych controllerów `position` i `orientation` w SkinMesh r44 był
rzeczywistym defektem zgodności z przebadanym natywnym korpusem. Nie był jednak
poznaną przyczyną całkowitej niewidoczności.

r45 dodał dokładnie te dwa controllery, zachowując pozostały payload r44, a
model nadal nie jest widoczny. Hipoteza:

```text
M2A-NWN-SKIN-BASE-CONTROLLERS-MISSING == jedyna root cause niewidoczności
```

jest więc sfalsyfikowana. Dokumenty r44/r45, które nazywają ją
`confirmedRootCause`, zawierają zbyt mocne twierdzenie. Poprawna klasyfikacja
to:

```text
confirmed conformance defect
insufficient visibility fix
not a proven root cause of total no-draw
```

## 3. Co jest potwierdzone w łańcuchu MOD -> creature

Statyczny odczyt exact zainstalowanych bajtów potwierdza:

- źródłowy i natywnie zainstalowany MOD mają ten sam SHA-256;
- źródłowy i natywnie zainstalowany HAK mają ten sam SHA-256;
- `module.ifo` wskazuje ordered HAK `m2a_h2r45`;
- HAK zawiera exact `appearance.2da`, `m2a_h2p45.mdl` i
  `m2a_h2t45.tga`;
- w `appearance.2da[15219]` istnieje `MODELTYPE=S` oraz
  `RACE=m2a_h2p45`;
- GIT zawiera jeden creature `m2a_h2utc45` na `[10,14.5,0]`;
- UTC ma `Appearance_Type=15219`, pełny profil
  `ActiveMonsterBaseline`, legalną klasę 11, rasę 7 i faction 1;
- player start `[10,10,0]` jest skierowany na creature;
- log klienta wiąże uruchomienie z `m2a_h2r45`;
- `currentgame\m2a_h2r45.mod` ma exact SHA-256 źródłowego MOD, co potwierdza,
  że runtime skopiował właściwy moduł;
- nie ma kolizji exact resrefów w `override` ani `development`;
- nie ma loose `appearance.2da` w drzewie użytkownika NWN, a sam MOD nie
  zawiera zasobu typu 2017 zasłaniającego HAK;
- ostatni wiersz `appearance.2da[15219]` jest fizycznie zakończony `CRLF`,
  identycznie jak działające generowane tabele placeable;
- TGA jest niepusty, 2048x2048 RGB i nie ma kanału alpha zerującego model.

To wyklucza pomylenie modułu, brak pliku w natywnym katalogu, błędny hash,
nieobecny wpis GIT/UTC, położenie pod mapą i przezroczystą teksturę.

## 4. Co dokładnie przechodzi w binary MDL

Exact `m2a_h2p45.mdl` ma:

```text
geometryType       2
classification     4
fog                1
supermodel         NULL
animationScale     1.0
base nodes         26
SkinMesh nodes     1
vertices           2678
faces              1543
active bones       22
animations         42
animationType      5 dla wszystkich 42
base skin controls position + orientation
```

Topologia, winding, normals, UV, indeksy, raw MDX pointers, wagi,
bone references, node-to-bone map i inverse binds są legalne. Wszystkie 42
stany zostały policzone offline w trzech punktach czasu; bind i sampled bounds
pozostają skończone i blisko początku modelu. Model nie jest zerowany ani
wyrzucany poza scenę przez zapisane transformacje.

## 5. Audyt dokładnego klienta NWN: finalna bramka SkinMesh

Przebadany klient:

```text
C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\bin\win32\nwmain.exe
version 89.8193.37-17
SHA-256 3b7f... (pełny hash w kontrakcie runtime)
```

`PartTriMesh::IsPartRenderable` pod `0x1400A2EB0`, wywoływany również przez
`PartSkin::Draw` pod `0x1400DEB40`, wymaga:

1. `Part.enabled != 0`;
2. niepustego wskaźnika model node;
3. `MdlNodeTriMesh.render != 0`;
4. niezerowej liczby wierzchołków albo alternatywnego zasobu runtime.

Binary loader ustawia `Part.enabled` z `mesh.render`. Dla r45:

```text
render       = 1
vertex count = 2678
```

Dlatego r45 spełnia jawny predykat wejścia do skinned draw. Pole runtime
`+0x278` nie jest wymagane przy niezerowej liczbie wierzchołków. Pole plikowe
`node+0x264` także nie jest bramką: loader je pomija i inicjalizuje odpowiadające
pole runtime samodzielnie.

Skin executor w tej wersji klienta obsługuje 64 wpisy palety. r45 używa 22.
Liczba 22 przekracza maksimum zaobserwowane w lokalnym natywnym korpusie (17),
ale nie przekracza implementowanego limitu klienta i nie jest podstawą do
orzeczenia no-draw.

Wniosek:

> Jeżeli resolver zwróci exact `m2a_h2p45`, powstanie jego instancja, a
> SkinMesh zostanie dołączony do renderowanej sceny, jawne bramki aktualnego
> klienta nie odrzucą r45 za pustą geometrię, `render`, brak materiału,
> liczbę kości ani brak controllerów.

## 6. Pełna lista potwierdzonych defektów r45

### P1. Sparse `appearance.2da[15219]`

Wiersz ma 35 kolumn, z czego tylko 7 ma wartość:

```text
LABEL, RACE, BLOODCOLR, MODELTYPE, WEAPONSCALE, MOVERATE, SIZECATEGORY
```

Pozostałe 28 komórek to `****`:

```text
STRING_REF, NAME, ENVMAP, WING_TAIL_SCALE, HELMET_SCALE_M,
HELMET_SCALE_F, WALKDIST, RUNDIST, PERSPACE, CREPERSPACE, HEIGHT,
HITDIST, PREFATCKDIST, TARGETHEIGHT, ABORTONPARRY, RACIALTYPE,
HASLEGS, HASARMS, PORTRAIT, PERCEPTIONDIST, FOOTSTEPTYPE,
SOUNDAPPTYPE, HEADTRACK, HEAD_ARC_H, HEAD_ARC_V, HEAD_NAME,
BODY_BAG, TARGETABLE
```

Dokładny parser klienta zapisuje `0` dla numerycznej komórki `****`, zamiast
zachować wartości domyślne konstruktora. Szczególnie r45 zeruje
`PERSPACE`, `CREPERSPACE`, `HEIGHT`, `HITDIST`, `PREFATCKDIST` i
`TARGETABLE`.

W lokalnym korpusie direct `MODELTYPE=S` r45 jest jedynym przebadanym wierszem
bez jednocześnie `HEIGHT`, `TARGETHEIGHT` i `TARGETABLE`. To jest potwierdzony
defekt kontraktu appearance. Nie ma jednak statycznego dowodu, że pojedynczo
wywołuje całkowity no-draw: pola są również używane przez collision,
targeting i camera/interaction.

### P2. Nienatywna numeracja i kolejność drzewa bazowego

r45 emituje:

```text
model root m2a_h2p45 = part 24
Hips                   = part 0
...
SkinMesh               = part 25
```

Wszystkie 53 przebadane samodzielne animowane modele creature z natywnego
korpusu mają:

```text
model root = part 0
```

`CreateInstanceTreeR` nie odrzuca wprost niezerowego numeru root i kopiuje go
do instancji. Skin gather używa drzewa, a nie samego numeru. Numeracja r45 jest
więc potwierdzonym odstępstwem od natywnego kontraktu oraz błędem
conformance/writera, ale sam statyczny kod nie wystarcza do nazwania jej
bezpośrednią bramką no-draw.

### P3. Oracle testowy zatwierdza własny układ writera

Semantic readback porównuje wygenerowany payload z oczekiwaniem zbudowanym
przez ten sam planner. Dlatego uznaje `root=24` za poprawne, mimo że cały
przebadany natywny korpus ma `root=0`.

To jest luka testowa: roundtrip i self-readback potwierdzają spójność własnego
formatu, ale nie zgodność z natywnym układem.

### P4. Walidator appearance sprawdza tylko dwie komórki resolvera

Materializer r45 wymaga tylko:

```text
MODELTYPE=S
RACE=m2a_h2p45
```

Nie wymaga pełnego profilu direct creature ani nawet
`HEIGHT/TARGETHEIGHT/TARGETABLE`. Zielony test HAK/2DA nie dowodzi więc
kompletnego runtime contract.

### P5. Poprzednia diagnoza została awansowana ponad dowód

Kontrakt r45 wymuszał, by dowód r44 zawierał kod
`M2A-NWN-SKIN-BASE-CONTROLLERS-MISSING` jako `confirmedRootCause`. Wynik r45
pokazuje, że był to tylko defekt korelacyjny. Admission gate i dokumentacja
muszą rozróżniać:

```text
confirmed defect
proven cause of exact visual failure
```

### P6. Brak candidate-bound śladu dynamicznego przed draw

Zachowane logi potwierdzają moduł, ale nie zapisują:

```text
appearance[15219] resolved
RACE m2a_h2p45 selected
MDL resource found
binary loader returned success
model instance attached to client creature
creature instance submitted to scene
```

To jest jedyny brakujący dowód pozwalający wskazać dokładnie jedną z granic
przed draw. Brak takiego śladu nie jest wynikiem `not_tested`; owner-bound
wynik wizualny r45 pozostaje `not_visible/failed`.

## 7. Hipotezy wykluczone jako przyczyna r45

Wykluczone przez exact bajty albo dokładny kod klienta:

- niewłaściwy MOD;
- inny/stary HAK;
- nadpisanie destination innym payloadem;
- ucięty ostatni wiersz 2DA albo brak terminatora linii;
- `appearance.2da` osłonięte loose plikiem lub zasobem z MOD;
- brak creature w GIT;
- błędny Appearance_Type w UTC;
- obiekt za graczem albo pod podłogą;
- pusty mesh;
- nielegalne indeksy lub winding;
- zerowe albo nieskończone bounds;
- brak UV/normals;
- przezroczysta TGA;
- `render=0`;
- wymaganie niepustego runtime `+0x278` dla binary mesh;
- pole plikowe `node+0x264`;
- brak SkinMesh;
- brak 42 stanów type 5;
- brak minimalnych base controllerów SkinMesh;
- przekroczenie implementowanego limitu 64 wpisów palety;
- obowiązkowy `supermodel != NULL`;
- obowiązkowy `rootdummy` jako warunek pierwszego draw;
- ogólna niezdolność projectowego writera do narysowania binary MDL
  (działający placeable to falsyfikuje).

## 8. Uczciwy werdykt przyczynowy

Na podstawie obecnego materiału można być pewnym następującego zdania:

> r45 jest niewidoczny, mimo że jego SkinMesh spełnia jawne końcowe bramki
> rysowania klienta. Awaria zachodzi przed tym predykatem: przy rozwiązaniu
> custom appearance/model, utworzeniu model instance albo dołączeniu instancji
> creature do renderowanej sceny.

Można też być pewnym, że r45 ma dwa niezależne, potwierdzone odstępstwa od
natywnego kontraktu:

1. sparse whole-model appearance zerujący parametry runtime;
2. niezerowy model root i nienatywna kolejność numerów part.

Nie ma obecnie dowodu pozwalającego uczciwie wskazać, które z nich pojedynczo
powoduje totalny no-draw. Nazwanie jednego z nich „na pewno root cause” byłoby
strzałem.

## 9. Decyzja iteracyjna

`r46` nie został utworzony. Nie wolno ponownie zamrozić i instalować payloadu
na podstawie kolejnej korelacji.

Minimalne prace implementacyjne, które wynikają z potwierdzonych defektów i
mogą być wykonane przed materializacją następnego kandydata:

1. zmienić planner binary MDL na natywny układ `model root=part 0`, a wszystkie
   pozostałe numery, animation nodes i skin maps przesunąć spójnie;
2. budować custom direct-S appearance przez sklonowanie kompletnego,
   działającego wiersza o zgodnym typie i zmianę tylko `LABEL/RACE`;
3. dodać niezależne native-corpus conformance tests dla obu kontraktów;
4. usunąć `confirmedRootCause` z dowodu r44 i zastąpić go
   `confirmedConformanceDefect`;
5. przed materializacją zapisać minimalny delta contract i wymagany wynik
   owner proof dla exact nowych bajtów.

Żaden z tych kroków nie jest deklaracją sukcesu w NWN. Zgodnie z decyzją
właściciela finalny wynik wizualny należy do owner proof.

## 10. Wynik implementacji potwierdzonych braków

Status offline 2026-07-26: oba potwierdzone odstępstwa zostały poprawione w
produkcyjnej ścieżce, bez tworzenia, pakowania ani instalowania `r46`.

### 10.1 Root-first part numbering

Planner `write_binary_mdl.rs` nie utożsamia już indeksu źródłowego IR z
`partNumber`. Buduje deterministyczny preorder hierarchii od jedynego
rzeczywistego korzenia, tworzy osobną mapę `sourceIndex -> partNumber` i
stosuje ją spójnie do:

- modelowego drzewa bazowego;
- parent/child pointers;
- węzłów lokalnych animacji;
- parentów mesh i AABB;
- SkinMesh forward/reverse maps;
- inverse-bind worlds oraz raportu semantic readback.

Korzeń modelu i korzeń każdego lokalnego stanu mają teraz `part 0`, nawet gdy
korzeń wejściowego IR nie znajduje się pod indeksem 0.

### 10.2 Pełny direct-S appearance

Produkcyjny M0/M6 append nie emituje już sparse dziewięciu pól. Wymaga pełnego
35-kolumnowego kontraktu runtime, odczytuje sprawdzony stockowy direct-S donor
`appearance.2da` physical row `102` (`MODELTYPE=S`, `RACE=c_horror`,
`TARGETABLE=1`), kopiuje wszystkie komórki i zmienia wyłącznie:

```text
LABEL
RACE
```

Dla małej tabeli testowej dozwolony jest pełny zgodny donor pod physical row
`0`; niepełny schemat albo donor niebędący sprawdzonym direct-S kończy się
fail-closed.

### 10.3 Niezależna weryfikacja

Przeszły:

```text
cargo test -p m2a-core --test mdl_writer
  43 passed

cargo test -p m2a-core --test model_pipeline
  27 passed

cargo test -p m2a-core --test two_da
  test pełnego clone request przeszedł

M2A_REQUIRE_RUNTIME_WITNESSES=1 cargo test -p m2a-core \
  --test runtime_witness_conformance \
  corrupt_h1_draw_and_three_in_place_native_families_lock_the_offline_oracle \
  -- --ignored
  1 passed

M2A_REQUIRE_RUNTIME_WITNESSES=1 cargo test -p m2a-core \
  --test model_pipeline \
  exact_last_city_direct_s_donor_is_cloned_full_width_with_only_label_and_race_changed \
  -- --ignored
  1 passed
```

Test korpusowy wiąże `root part 0` z trzema niezależnymi natywnymi rodzinami:
retail `c_Direwolf`, retail `c_horror` i CEP R3 `c_phod_horror_b`. Test 2DA
czyta exact 35-kolumnową tabelę Last City in-place i potwierdza, że w
dopisanym wierszu wszystkie komórki poza `LABEL/RACE` są identyczne z row 102.

Historyczne pole r44 `confirmedRootCause` zostało skorygowane do
`confirmedConformanceDefect`; późniejszy wynik r45 dowiódł, że brak
kontrolerów był realnym defektem emitera, ale nie wystarczał do wyjaśnienia
całkowitej niewidoczności.

### 10.4 Audyt hipotezy `Phenotype`

Hipoteza właściciela z 2026-07-27 została sprawdzona w exact kodzie generatora,
zamrożonym `m2a_h2r45.mod` oraz dekompilacji Aurory:

- `m2a_h2r45.mod` zawiera pole `Phenotype` osobno w GIT i UTC;
- generator zapisuje w obu miejscach `Phenotype=0` jako GFF `INT`;
- loader `FUN_00532174` czyta brakujące pole z defaultem `0`, a wartość `1`
  normalizuje do `0`;
- saver `FUN_0053521c` zapisuje `Phenotype` jako `INT`;
- resolver/model builder `FUN_00536e60` rozwiązuje whole-model direct-S przez
  `Appearance_Type -> MODELTYPE/RACE`. `PHENOTYPE.2DA/DefaultPhenoType`
  uczestniczy w składaniu phenotype/body parts, a nie dodaje drugiego model
  resrefu do ścieżki `MODELTYPE=S`;
- exact 35-kolumnowy `appearance.2da` r45 nie zawiera kolumny
  `DefaultPhenoType`; należy ona do osobnej tabeli `phenotype.2da`.

Werdykt: hipoteza brakującego fenotypu jest wykluczona jako przyczyna totalnego
no-draw exact r45. Jawny `Phenotype=INT 0` pozostaje poprawnym wymogiem
conformance. Podstawowy MOD readback został wzmocniony tak, aby odrzucał brak,
inny typ lub inną wartość; test negatywny najpierw wykazał lukę, a po poprawce
przeszły wszystkie trzy testy `binary_creature_profile_matrix_module`.

Granica wyniku pozostaje uczciwa: testy offline potwierdzają nową zgodność
strukturalną, lecz widoczność nowych exact bajtów w NWN pozostaje
`not_tested/missing`, dopóki nie powstanie dopuszczony następny kandydat i
właściciel nie wykona jego proofu.
