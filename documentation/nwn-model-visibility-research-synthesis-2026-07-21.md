# Synteza badań: niewidoczne i zdeformowane modele w NWN:EE

Status: `ACTIVE / ROOT CAUSE NIEZAMKNIĘTE`  
Data: `2026-07-21`  
Zakres: Meshy2Aurora, native binary MDL/MDX, direct creature oraz odróżnienie
braku modelu od błędnego proofu Area/kamery.

## 1. Cel i zasada interpretacji

Ten dokument porządkuje ustalenia z rozproszonych packetów M0/M6 oraz z
read-only badań NWN:EE. Odpowiada na dwa różne pytania:

1. dlaczego model może być niewidoczny lub wyglądać jak brakujący w NWN;
2. co obecne dowody mówią konkretnie o H1 i M0 Meshy2Aurora.

Nie jest to deklaracja naprawy. Zgodnie z Aurora First:

- `verified` oznacza dowód o dokładnie wskazanej tożsamości i warstwie;
- `failed` oznacza negatywny wynik dokładnie tej bramki, nie automatycznie
  przyczynę błędu;
- `missing` oznacza brak dowodu, a nie porażkę modelu;
- Toolset i klient NWN są odrębnymi bramkami;
- brak sylwetki na pojedynczym obrazie NWN nie dowodzi odrzucenia przez
  renderer, dopóki kamera i pozycja fixture nie są powiązane z capturem.

Nie skopiowano żadnego retailowego ani community payloadu, modelu, tekstury,
animacji ani kodu do produktu. Materiały z gry i Internetu pozostają wyłącznie
źródłami read-only.

## 2. Stan w skrócie

| Tor | Najnowszy fakt | Status | Czego nie dowodzi |
| --- | --- | --- | --- |
| Łańcuch zasobów M0 | `GIT -> Appearance_Type=848 -> appearance.2da -> m2a_m0p01 -> TGA` jest związany hashami dla v12 | `verified` offline/saved binding | poprawnego rysowania w NWN |
| M0 w Aurora | Właściwy `TScrollBox` dla `m2a_m0v12.mod` pokazuje szaro-zieloną sylwetkę docelowego M0 | `verified` Toolset | wyniku runtime NWN |
| M0 w aktualnym kadrze NWN | `nwmain` załadował `m2a_m0v12`, ale capture nie pokazuje M0; fixture nie leży na osi startu kamery | `failed` dla kadru, przyczyna renderera `missing` | że renderer odrzucił MDL |
| H1 v20 RIGID | Ten sam H1 został w NWN narysowany jako duża zdeformowana figura | `verified` obserwacja runtime | poprawności wspólnego layoutu mesh/MDX |
| H1 v21 `meshType=0` | Wariant został zmaterializowany offline | `verified` offline | że `0` naprawia runtime |
| Skin H1 v19 | Wariant skin nie uzyskał wiarygodnej poprawnej widoczności runtime | `failed/missing` zależnie od bramki | że winna jest wyłącznie lista klipów albo liczba kości |

Źródła bieżącego stanu: [binding M0 v12](evidence/m0-v12-toolset-viewport-binding-2026-07-20.md),
[diagnostyka kadru NWN M0 v12](evidence/m0-v12-runtime-visual-diagnostic-2026-07-20.md),
[bramka geometrii M0 v12](evidence/m0-v12-native-geometry-gate-diagnostic-2026-07-20.md),
[audyt H1](evidence/m6-nwn-runtime-model-visibility-audit-2026-07-18.md) oraz
[plan rozdzielenia hipotez](evidence/m6-nwn-runtime-model-resolution-plan-2026-07-18.md).

## 2a. Aktualny ledger r21-r27 — czytać przed historycznymi packetami v12

Tabela w §2 zachowuje historię v12/H1, ale **nie jest bieżącym werdyktem dla
r21-r27**. W szczególności nie wolno przenosić starego, nierozstrzygającego
kadru NWN v12 na widoczny w Toolsecie r21. Równocześnie późniejszego,
kandydat-bound wyniku r21/r26 nie wolno wymazywać przez dawne
`modelVisibility=not_tested`: obserwacja `not_visible` jest faktem, choć jej
`proofCompleteness` nie daje jeszcze przyczynowego werdyktu o MDL.

| Pytanie | Aktualny fakt o r21 | Status i najbliższa granica |
| --- | --- | --- |
| Tożsamość kandydata | MOD `m2a_m0r21.mod`, jeden HAK `m2a_m0r21`, MDL `m2a_m0p01` i TGA `m2a_m0t01` są związane hashami; fizyczny wiersz `appearance.2da` ma numer `848`. | `verified` offline. Zmiana hasha lub resrefu tworzy nowy kandydat, więc nie jest dopuszczalnym skrótem proofu. |
| Resolver direct creature | W GIT jest jawne `Appearance_Type=848`; wiersz ma `MODELTYPE=S`, `RACE=m2a_m0p01`. Aurora odczytuje appearance jako `ushort` i pobiera `MODELTYPE`; xoreos wybiera fizyczny wiersz `appearance`, a dla typu innego niż `P` ładuje dokładnie `RACE`. | `verified` offline oraz spójne z Toolsetem. Nie zmieniać `S` na `P` ani nie mylić `TemplateResRef` UTC z resrefem MDL. |
| Toolset | Ten sam single rigid M0 został pokazany jako mała sylwetka w zweryfikowanym `TScrollBox`. | `modelVisibility=visible`, `proofCompleteness=verified` **wyłącznie dla Toolsetu**. |
| NWN r21 | Jednorazowy hash-bound run ładuje `m2a_m0r21` i pokazuje brak M0, lecz GIT ma pozycję `[5.0642,14.9358,0]`, a nie kontraktowe `[10,14.5,0]`. | `modelVisibility=not_visible`, `proofCompleteness=missing` dla diagnozy MDL. Wynik dopuścił wyłącznie placement-only następcę, nie zmianę MDL. |
| NWN r26 / bieżący r27 | R26 (stały fixture r25) jest widoczny w Toolsecie i nie pokazuje M0 w świeżym NWN; identity controllers nie wystarczyły. R27 różni się od r26 tylko siedmioma bajtami `animation_type: 0 -> 5`, ale nie ma jeszcze live capture. | R26: `modelVisibility=not_visible`, runtime `proofCompleteness=missing`. R27: `modelVisibility=not_tested`, `proofCompleteness=missing`; następny legalny krok to proof r27, nie r28. |
| Base mesh i materiał | Root `0x01`, rigid mesh `0x21`, `render=1`, `meshType=3`, tekstura RGB bez alpha oraz bez aktywnego shadowingu r21 przeszły odczyt offline. | Negatywny wynik dla root/classification/alpha/tekstury jako obecnej przyczyny. Nie jest to emulator renderera NWN. |
| Lokalne animacje | W wdrożonym MDL r21/r26 siedem nagłówków ma `type=0`; bieżący writer i source/readback emitują oraz wymagają `type=5`, a r27 materializuje tylko tę delte. | To jedyny realistyczny ABI A/B oczekujący na proof r27. Runtime nie ustalił jeszcze, czy `0 -> 5` przywraca widoczność. |
| Produktowe bramki niezależne od r21 | Zakres skróconego `appearance.2da`, UTC readback oraz namespace wielu HAK-ów są osobnymi wymaganiami generatora. | Są ważne dla ogólnego pakietowania, lecz nie są dowodem błędu widocznego r21. |

### Granica źródeł

Aurora decomp `FUN_00532174 -> FUN_00512810` potwierdza odczyt
`Appearance_Type` oraz pobranie `MODELTYPE`; xoreos `Creature::loadModel()`
potwierdza modelowy rozdział `P` od direct `RACE`. NeverBlender opisuje
authoring MDL: root ma nazwę pliku, a `Character`/`Render` są właściwościami
modelu. Nie dokumentuje GIT/UTC, HAK, `appearance.2da` ani binarnego
`animation_type`; nie może zatem być użyty jako uzasadnienie dla zmiany
resolvera albo lifecycle r21.

Jedyny dowód, który może rozstrzygnąć otwarty ABI A/B, to osobno autoryzowany,
kontrolowany proof NWN dla dokładnego hash-bound łańcucha r27. Do jego czasu
kolejne analizy rozdzielają wady źródeł/writera od własności wdrożonego
artefaktu i od niepełnego proofu, zamiast dopisywać kolejną hipotezę renderera.

## 3. Co faktycznie może uczynić model niewidocznym

| Warstwa | Mechanizm | Charakterystyczny objaw | Najmniejsza poprawna kontrola |
| --- | --- | --- | --- |
| Wybór modelu | Zły `Appearance_Type`, przesunięte kolumny `appearance.2da`, zły `RACE`, niezgodny `MODELTYPE`, brak lub zła kolejność HAK | Gra ładuje inny model albo nie rozwiązuje docelowego resrefu | GIT + HAK + exact row + inventory HAK + log klienta |
| Identyfikatory MDL | Nazwa pliku/modelu, texture lub supermodelu przekracza 16 znaków; node jest dłuższy niż 32 znaki albo koliduje po skróceniu | Błąd ładowania, crash, dziwna hierarchia | Odczyt tekstu/binarnego MDL i test resrefów |
| Flaga geometryczna | `render 0` świadomie nie rysuje mesha; `alpha 0` czyni go przezroczystym | Cień lub selection może istnieć bez widocznej geometrii | Odczyt flag node/mesh, bez wnioskowania z samego cienia |
| Materiał i tekstura | TXI/MTR nadpisuje `texture0`, używa przezroczystości albo błędnego blendingu; alpha tekstury jest zerowa | MDL istnieje, lecz obiekt znika lub miesza się z tłem | Jeden nieprzezroczysty TGA, bez MTR/TXI jako A/B |
| Tile/static | `tilefade` może ukrywać mesh tile'a lub statycznego placeable przy opcji ukrywania drugiego piętra | Znika zależnie od kamery/opcji klienta | Sprawdzenie typu obiektu i `tilefade`, nie przenoszenie tej hipotezy na dynamic creature |
| Culling/geometria | Odwrócony winding/normalne, ujemna skala, skala ekstremalna, błędne bounds lub pozycja poza kadrem/pod terenem | Widoczny tylko z jednej strony, jako plama albo tylko po zmianie kamery | Widok z obu stron, kontrola transformu/bounds oraz fixture na znanej powierzchni |
| Binary mesh/MDX | Zły primitive type, offset, stride lub format strumienia POSITION/UV/NORMAL/indeksów | Toolset bywa tolerancyjny, NWN deformuje lub nic nie rysuje | Niezależne porównanie binarne z retailowym MDL i clean-room fixture |
| Skin/animacja | Złe inverse-bind, mapowanie kości, base pose, kolejność node'ów lub supermodel | RIGID jest widoczny, ale skin znika albo deformuje się | Drabina S1--S4: jedna kość/identity, potem jeden ruch, potem pełny rig |
| Proof/scena | Creature nie zostało zapisane, jest poza walkable surface albo kamera nie patrzy na fixture | Pusty viewport/kadr mimo poprawnego pakietu | Native GIT readback + zapisana Area + capture związany z pozycją i kamerą |

### Fakty ogólne potwierdzone przez dokumentację NWN

- `render 0` wyłącza rysowanie w świecie; `shadow` jest niezależny. Brak
  tekstury zazwyczaj daje pustą białą powierzchnię, a nie sam z siebie
  niewidoczność. [Model Table of Parameters](https://nwn.wiki/spaces/NWN1/pages/53671005/Model%2BTable%2Bof%2BParameters)
- `bitmap` jest aliasem `texture0`; materiał MTR ma pierwszeństwo przed
  teksturą z MDL. MTR umożliwia też jawne `transparency` i `twosided`.
  [MTR](https://nwn.wiki/spaces/NWN1/pages/12027232/MTR)
- TXI kontroluje blending i `alphamean`, zatem sama obecność prawidłowego TGA
  nie wystarcza do wykluczenia przezroczystości. [TXI](https://nwn.wiki/spaces/NWN1/pages/38174929/TXI)
- Dla direct creature `MODELTYPE=S` używa pojedynczego MDL wskazanego przez
  `RACE`; `P` jest modelem z części, a `F` i `L` mają inny kontrakt animacji
  i attachmentów. Przesunięte kolumny `appearance.2da` są znaną przyczyną
  brakujących części modeli. [appearance.2da](https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da)
- Modele mogą być ASCII lub binary MDL. Poprawne nazwy, root dummy, parentage,
  limity węzłów i ograniczenia skinu są warunkami bezpieczeństwa ładowania,
  ale własny readback nie jest dowodem runtime. [MDL](https://nwn.wiki/spaces/NWN1/pages/38175669/MDL)
- Dla aktualnego NWN:EE maksymalnie 64 kości przypadają na pojedynczy skinmesh,
  a pojedynczy wierzchołek może mieć najwyżej 4 wpływy. [Models](https://nwn.wiki/spaces/NWN1/pages/38175602/Models)

## 4. Kluczowe ustalenie: `meshType` w native binary MDL

### 4.1 Odczyt retailowego klienta

Wykonano read-only dekodowanie `nwn_base.key` i `models_01.bif` z lokalnej
instalacji NWN:EE. Dla trzech binary MDL typu zasobu `2002` odczytano pole
mesha pod tym samym offsetem, którego używa writer Meshy2Aurora:

| Retailowy model | Liczba mesh node'ów | Wartość pola `meshType` |
| --- | ---: | --- |
| `c_badger` | 30 | `3` dla wszystkich 30 |
| `c_bear` | 17 | `3` dla wszystkich 17 |
| `c_Wolf` | 24 | `3` dla wszystkich 24 |

Niezależny szablon formatu NWN1 opisuje wartość `3` jako `triangle_list`.
[xoreos: NWN1MDL.bt](https://raw.githubusercontent.com/xoreos/xoreos-docs/master/templates/NWN1MDL.bt)

To jest **fakt z retailowego binary MDL**, nie implementacyjna instrukcja
kopiowania payloadu. Ustanawia mocną pozytywną referencję dla interpretacji
pola w profilu direct creature.

### 4.2 Porównanie H1 v20 i v21

Pliki `m2a_m6p01.mdl` dla H1 RIGID v20 i wariantu v21
`draw-mode-zero` mają identyczną długość i różnią się dokładnie jednym bajtem:

```text
offset pliku 0x6ECC8: 03 -> 00
v20: meshType = 3
v21: meshType = 0
```

V20 został obserwowany w NWN jako zdeformowana figura. V21 ma tylko pakiet
offline, bez wiarygodnego capture runtime. Z tego wynika:

1. nie wolno opisywać `0` jako potwierdzonego `triangle-list` ani naprawy;
2. sama wartość `3` nie wyjaśnia deformacji v20, bo jest normalna w retailowym
   corpusie i jest też używana przez M0 v12;
3. A/B `3` kontra `0` ma sens wyłącznie po ustaleniu poprawnej Area, pozycji,
   kamery i kontroli dodatniej, z niezmienionym każdym innym bajtem.

Aktualny zapis writera, który opisuje `0` jako wariant triangle-list, należy
traktować jako **hipotezę wymagającą proofu**, nie jako ustalony kontrakt.

## 5. Ustalenia specyficzne dla Meshy2Aurora

### 5.1 Łańcuch `appearance -> MDL` nie jest obecnie głównym tropem M0

Dla M0 v12 zachowano i odczytano jednoznaczne wiązanie:

```text
GIT fixture
  -> Appearance_Type 848
  -> M2A_M0_MESHY_RIGID w appearance.2da
  -> MODELTYPE S, RACE m2a_m0p01
  -> binary MDL type 2002 i TGA type 3 w tym samym HAK
```

Wystarczy to do statusu `verified` dla bindingu zasobów, nie do statusu
runtime. W aktualnym capture NWN nie wystąpił komunikat o brakującym HAK,
`m2a_m0p01` ani teksturze, co osłabia — ale nie eliminuje globalnie — hipotezę
braku zasobu.

### 5.2 Toolset nie jest substytutem gry

M0 v12 przeszedł właściwy proof `TScrollBox` i jest widoczny w Aurora jako
docelowa sylwetka. Wcześniejsze obserwacje M0, w tym preview w `Creature
Properties`, nie są używane jako dowód canvasa po zapisie. Zasada dowodowa i
pełna historia są w [M0 viewport evidence discipline](evidence/m0-viewport-evidence-discipline-2026-07-20.md).

Wynik Toolsetu oznacza jedynie, że bieżący model/render path jest akceptowany
przez Aurora w związanej Area. Nie dowodzi, że ten sam MDL ma poprawny draw,
culling i deformację w `nwmain`.

### 5.3 Aktualny obraz NWN M0 nie rozstrzyga renderera

W runtime v12 model nie pojawił się w zapisanym obrazie, lecz fixture
`[5.0642, 14.9358, 0]` nie znajdowało się na bezpośredniej osi startu IFO
`[10, 10, 0]`/kierunku kamery. Jest to negatywny wynik **dla kadru**, a nie
dowód brakującego draw calla. Nie należy zmieniać MDL, HAK, skinu ani animacji
na podstawie tego obrazu.

Ponadto formalna bramka AUR-S07 pozostaje `missing`: centralny observer nie
udowodnił jeszcze dostępnej powierzchni entry point. Wcześniejsze
`area_geometry_invalid` w v12 było częściowo artefaktem braku natychmiastowego
hasha pliku zablokowanego przez Toolset, a nie potwierdzonym błędem Area.

### 5.4 H1: co zostało osłabione lub odrzucone

| Hipoteza | Stan | Uzasadnienie |
| --- | --- | --- |
| „Brakuje HAK-a albo wpisu appearance” | osłabiona, nie globalnie odrzucona | H1/M0 mają zgodne offline bindingi; M0 nie zgłasza braku zasobu w logu; niezależny referencyjny tortoise działał w środowisku właściciela |
| „Siedem aliasów animacji globalnie blokuje draw” | odrzucona jako globalny gate | H1 v20 RIGID ma ten sam minimalny profil klipów, a został narysowany przez NWN |
| „25 kości H1 przekracza limit EE” | odrzucona | H1 v19 ma 25 aktywnych slotów i maks. 4 wpływy na wierzchołek, czyli mieści się w limitach EE |
| „`meshType=3` sam powoduje deformację” | niepotwierdzona i słaba | `3` występuje w retailowych modelach, H1 v20 i M0 v12; różnice MDX/transformów pozostają nieporównane |
| „Skin/base pose/inverse bind deformuje H1” | otwarta | v20 nie ma skinu, dlatego trzeba najpierw zamknąć RIGID, potem budować skin po jednym czynniku |
| „Kamera/Area daje fałszywy negatywny proof” | potwierdzona jako klasa ryzyka | v11 nie umieścił creature poza tilem; v12 ma Toolset presence, ale runtimeowy kadr nie obejmuje fixture w sposób rozstrzygający |

## 6. Referencje i ich wartość dowodowa

| Źródło | Co wnosi | Granica użycia |
| --- | --- | --- |
| Lokalna dekompilacja Aurora | Semantyka Toolsetu i resolver `Appearance_Type` | Priorytet dla implementacji; nie zastępuje obserwacji klienta NWN |
| Retailowe BIF/KEY NWN:EE | Rzeczywiste binary MDL, w tym `meshType=3` | Read-only corpus; żadnego kopiowania payloadów |
| `c_tortoise` / ZND Tortoises | Minimalny pakiet direct creature (`appearance`, ASCII MDL, TGA), self-supermodel, skin i animacje | Referencja community, nie fixture produktu; owner observation nie jest capturem Codex |
| NWN Wiki / Neverwinter Vault / xoreos | Znaczenie parametrów, limity i format jako kontekst | Uzupełnia Aurora First; nie zastępuje lokalnego dowodu |
| Own reader/writer | Deterministyczny layout i własny readback | Nie może sam potwierdzać własnej zgodności runtime |

Najważniejsze zewnętrzne odnośniki:

- [MDL](https://nwn.wiki/spaces/NWN1/pages/38175669/MDL)
- [Models i limity EE](https://nwn.wiki/spaces/NWN1/pages/38175602/Models)
- [appearance.2da](https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da)
- [Model Table of Parameters](https://nwn.wiki/spaces/NWN1/pages/53671005/Model%2BTable%2Bof%2BParameters)
- [MTR](https://nwn.wiki/spaces/NWN1/pages/12027232/MTR)
- [TXI](https://nwn.wiki/spaces/NWN1/pages/38174929/TXI)
- [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf)
- [ZugothNDeadly's Tortoises](https://neverwintervault.org/project/nwn1/model/zugothndeadlys-tortoises)

## 7. Jednoznaczna macierz następnych testów

Żaden z poniższych kroków nie jest automatyczną zgodą na uruchomienie Toolsetu,
NWN, zmianę MRU/INI ani zapis do katalogu gry. Live work wymaga aktualnej
kanonicznej trasy Aurora i odpowiedniej zgody właściciela.

| Kolejność | Pojedyncza zmienna | Warunek interpretowalnego wyniku | Interpretacja |
| --- | --- | --- | --- |
| P0 | Związać runtimeowy capture z właściwą pozycją M0 i osią kamery oraz kontrolą dodatnią | Prawidłowy zapis Area/GIT, dokładna pozycja fixture, uwierzytelniony obraz NWN, znany model kontrolny | Dopiero wtedy „brak sylwetki” może dotyczyć renderera |
| P1 | `meshType=3` vs `meshType=0` | Dokładnie ten sam MDL poza jednym polem, identyczne HAK/MOD/Area/kamera, proof obu wariantów | Rozstrzyga wartość pola bez mieszania skinu i animacji |
| P2 | RIGID mesh/MDX | `P1` ma poprawny, rozpoznawalny RIGID baseline | Porównać tylko stride, offsets, indeksy, node transform i bounds z native corpus |
| S1 | Jeden skin, jedna kość, waga 1.0, identity inverse bind | Sylwetka identyczna jak RIGID | Rozdziela skin layout od geometrii bazowej |
| S2 | Base pose jednej kości | Obraz identyczny jak S1 | Testuje hierarchy/base pose |
| S3 | Jeden kontrolowany ruch kości w `cpause1` | Lokalna, przewidywalna deformacja | Testuje routing animacja -> palette -> skin |
| S4 | Pełny rig H1 | Poprawny idle i ruch | Dopiero tu diagnozować konkretne wagi/inverse bind H1 |

Nie wolno jednocześnie zmieniać `meshType`, offsets MDX, skin map, inverse-bind
i listy animacji. Taki przebieg nie wskazuje przyczyny niezależnie od wyniku.

## 8. Warunek zamknięcia problemu

Problem widoczności/deformacji może otrzymać status `resolved` dopiero, gdy
ten sam hash wygenerowanego modelu:

1. przejdzie own writer/readback oraz niezależny gate binarny;
2. będzie rozpoznawalny w Aurora Toolset na poprawnie zapisanej Area;
3. będzie rozpoznawalny w NWN w capture związanym z modułem, HAK, Area,
   fixture, pozycją i kamerą;
4. dla RIGID zachowa poprawną sylwetkę, a dla skinu przejdzie przynajmniej
   kontrolowany idle i jeden ruch kości;
5. otrzyma packet z hashami MDL/HAK/MOD, logiem klienta oraz obrazami Toolsetu
   i NWN.

Do tego czasu prawidłowe statusy to `verified`, `failed` albo `missing` dla
poszczególnych bramek — nie ogólne „model działa” ani „model jest niewidoczny”.

## 9. Powiązane dokumenty lokalne

- [Audyt widoczności H1](evidence/m6-nwn-runtime-model-visibility-audit-2026-07-18.md)
- [Plan rozwiązania H1](evidence/m6-nwn-runtime-model-resolution-plan-2026-07-18.md)
- [Profil animacji H1](evidence/m6-nwn-runtime-animation-profile-v14-2026-07-18.md)
- [Audyt referencyjnego HAK ZND Tortoises](evidence/znd-tortoises-hak-audit-2026-07-18.md)
- [Baseline NeverBlender/CleanModels](evidence/m6-incaxje-neverblender-runtime-baseline-2026-07-18.md)
- [M0 v12: binding Toolset viewport](evidence/m0-v12-toolset-viewport-binding-2026-07-20.md)
- [M0 v12: diagnoza runtimeowego kadru](evidence/m0-v12-runtime-visual-diagnostic-2026-07-20.md)
- [M0 v12: native geometry gate](evidence/m0-v12-native-geometry-gate-diagnostic-2026-07-20.md)
- [Dyscyplina dowodowa viewportu M0](evidence/m0-viewport-evidence-discipline-2026-07-20.md)

## 10. Iteracja I1 -- binarny mesh, odrzucenie renderu i `meshType` (2026-07-21)

Ten pass celowo nie zmienia writer'a ani nie uruchamia Toolsetu lub klienta
NWN. Jego celem jest oddzielenie warunku odrzucenia mesha od pola wyboru
prymitywu. Kazde z ponizszych zrodel zostalo sprawdzone w tej iteracji.

| Zrodlo | Fakt ustalony w I1 | Status dowodowy |
| --- | --- | --- |
| Aurora decompilation, `C:\\Projects\\New Folder\\export\\decompiled_all.c` | `FUN_00a5e508` odrzuca runtime mesh, gdy `render` pod `+0xdc` jest zerem albo jednoczesnie `vertexCount` pod `+0x230` i `startMdx` pod `+0x228` sa zerowe. `FUN_00a5e54c` przekazuje wartosc pod `+0x224` dalej do callu renderera. | Fakt z dekompilacji Aurory; nazwy funkcji sa robocze. |
| [xoreos -- binary NWN mesh reader](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp) | Reader czyta byte pod binarnym `+0x224` jako `triangleMode`; komentarz kodu rozroznia `3 = Triangle`, `4 = TriStrip`. Nastepnie czyta offsets/counts dla pozycji, UV, normalnych i indeksow. | Fakt z niezaleznej implementacji; nie jest dowodem zachowania zamknietego klienta Beamdog. |
| [xoreos -- NWN model loader](https://github.com/xoreos/xoreos/blob/master/src/engines/nwn/modelloader.cpp) | Model jest ladowany wedlug resref; komentarz kodu zaznacza, ze MOD/HAK moga nadpisywac model, a cache trzeba czyscic po unloadzie modulu. | Fakt z xoreos, klasa ryzyka dla testu HAK/runtime. |
| [NWN Wiki: Tutorials / Blender](https://nwn.wiki/spaces/NWN1/pages/14618048/Tutorials#Tutorials-Blender) oraz [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Tutorial wiki byl niedostepny dla automatycznego odczytu (blad serwera), wiec nie przypisano mu zadnej tezy. Manual potwierdza, ze Trimesh jest podstawowym meshem, eksport trianguluje faces, `Render` steruje rysowaniem, a `Tilefade` moze uczynic mesh niewidocznym tylko w kontekcie tiles. | Manual: fakt pomocniczy; wiki: sprawdzone, lecz bez ekstrakcji tresci. |

### Co to zmienia w naszej diagnozie

1. Nasz writer zapisuje `render = 1`, niezerowy `vertexCount` oraz pozycje,
   UV, normalne, faces i indeksy na offsetach odpowiadajacych kolejnosci
   czytanej przez xoreos. To przechodzi wykryty warunek odrzucenia z Aurory.
   Nie jest jeszcze dowodem, ze dane sa semantycznie poprawne dla NWN:EE.
2. `meshType=3` nie jest sam w sobie kandydatem na usuniecie. Zostal tez
   zaobserwowany w retailowych creature MDL (`c_badger`, `c_bear`, `c_wolf`),
   a xoreos nazywa go trybem trojkatow. Wariant `0` nie ma jeszcze tu takiego
   potwierdzenia. Hipoteza "zero = triangle list" zostaje wycofana.
3. Aktualny komentarz writer'a, ktory opisuje `0` jako "triangle-list
   variant", jest nieudokumentowany i powinien zostac usuniety albo
   zastapiony opisem neutralnym po osobnym, malym review. To jest korekta
   dokumentacji kodu, nie zmiana formatu.
4. Dla M0 brak sylwetki w NWN pozostaje nierozstrzygniety: fixture v12 nie
   byl na osi kamery. Przed kazda zmiana MDL trzeba najpierw zamknac ten
   falszywy negatyw testem pozycji i pozytywna kontrola.

### Kolejne najmniejsze eksperymenty (bez laczenia zmiennych)

| Priorytet | Zmiana do zrobienia w aplikacji / proofie | Kryterium |
| --- | --- | --- |
| I2-P0 | Raport fixture musi zawierac `resref`, hash MDL/HAK/MOD, pozycje GIT, pozycje startowa i kierunek kamery. Generator testowej Area ma ustawic fixture na potwierdzonym widocznym promieniu kamery oraz dodac native positive control. | Jednoznacznie odroznia off-camera, resolver/HAK i renderer. |
| I2-P1 | Dodac statyczna bramke `runtimeMeshEligibility`: `render != 0`, `vertexCount > 0 || startMdx != 0`, poprawne zakresy offsetow, liczniki faces/indices i referencje resref/appearance. | Wczesny, wyjasnialny blad przed pakowaniem HAK/MOD. Nie zastepuje proofu runtime. |
| I2-P2 | Dla rigid baseline stworzyc dwa identyczne fixture tylko z odwroconym windingiem faces, po uprzednim P0. | Rozdziela back-face culling od zlego stride/offsetu i od problemu kamery. |
| I2-P3 | Zachowac `meshType=3` dla direct-creature M0. Porownanie `3` z `0` wolno wykonac dopiero po P0, przy identycznym hashowanym HAK/MOD poza pojedynczym polem. | Nie wolno przypisywac wyniku zlym offsetom, skinowi lub pozycji. |

### Status po I1

- `runtime-mesh-eligibility`: **writer-passes / client-unproved**.
- `meshType=3`: **native-supported / not-yet-A-B-proved in NWN:EE**.
- `M0 missing in NWN`: **camera-and-placement-confounded**.
- `H1 deformation`: **open; nie wolno laczyc jej z M0 visibility**.

## 11. Iteracja I2 -- resolver creature i provenance runtime packetu (2026-07-21)

### Fakty z trzech wymaganych zrodel

| Zrodlo | Ustalony fakt | Wniosek dla aplikacji |
| --- | --- | --- |
| Aurora decompilation, `C:\\Projects\\New Folder\\export\\decompiled_all.c` | Ponownie potwierdzono, ze decyzja renderu dotyczy danych juz zaladowanego mesha (`render`, `vertexCount`, `startMdx`) i nie zawiera tozsamosci MOD/HAK/GIT ani polozenia kamery. | Poprawny MDL readback nie moze byc uznany za proof, ze to ten sam artefakt zostal pokazany przez NWN. |
| [xoreos creature resolver](https://github.com/xoreos/xoreos/blob/master/src/engines/nwn/creature.cpp) | `Appearance_Type` jest odczytywane z instancji GFF; wiersz `appearance.2da` rozroznia model skladany (`MODELTYPE=P`) od direct modelu ladowanego z `RACE`, a nastepnie nakladane sa pozycja i orientacja instancji. | Runtime contract musi wiazac razem GIT `Appearance_Type`, konkretny wiersz `appearance.2da`, `RACE`, resref MDL i polozenie instancji. Sam hash MDL nie wystarcza. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Root modelu musi miec nazwe pliku bez rozszerzenia; klasyfikacja `Character` obejmuje creature/placeable. | Wlasciwy direct-creature root jest wymaganiem eksportu, a nie opcjonalna kosmetyka. |

### Co wykryto w naszym kodzie i proofach

1. Profile A nie zostawia GLTF w zlej przestrzeni docelowej. Choc ingest
   przechowuje surowe dane GLTF (Y-up, preserved winding), konwersja blokuje
   polityki `GltfToAuroraXzy`, `FlipVOnce` i `ReverseOnce`; dla M0 stosuje
   `basis * mesh_world`, a ujemny wyznacznik odwraca indeksy trojkata.
   **Nie wolno dodawac drugiej konwersji osi, UV ani windingu.**
2. `normalize_direct_creature_runtime_root` wymusza pojedynczy root o nazwie
   resref modelu i ustawia tak samo roots generowanych clipow. To pokrywa
   wymaganie NeverBlender dla M0/H1 direct creature.
3. Generator izolowanego M0 ma testowany kontrakt: IFO `[10,10,0]`, kierunek
   `[0,1]`, GIT `[10,14.5,0]`, jeden creature i wiersz appearance. Jednak
   packet v12, ktory byl widoczny w Toolsecie i sfotografowany w NWN, ma inny
   MOD/HAK oraz zapisane GIT `[5.0642,14.9358,0]`. Nie jest wiec prostym
   runtime proofem aktualnego kontraktu generatora.
4. Istniejacy `ProofModuleReportV1` przechowuje hashe, resrefy i row
   appearance, ale nie serializuje wejscia IFO/GIT (pozycja, kierunek,
   TemplateResRef, ordered HAK list). Wlasny semantic readback podczas builda
   to sprawdza, lecz pozniejszy capture nie ma kompletnego, przenosnego
   kontraktu do porownania.

### Zmiany do zaimplementowania przed kolejnym runtime proofem

1. Dodac `RuntimeFixtureContractV1` do raportu i manifestu paczki. Ma zawierac
   lane (`M0 isolated` albo `canonical`), resref MOD/Area/creature/modelu,
   uporzadkowana liste HAK, `Appearance_Type`, docelowy wiersz i wartosc
   `RACE`, `TemplateResRef`, IFO entry/direction, GIT position/orientation
   oraz sha256 MOD/HAK/MDL/TGA/appearance. Dane maja pochodzic z ponownego
   odczytu wygenerowanych bajtow, nie z samych stalych wejsciowych.
2. Dodac niezalezny verifier tego kontraktu dla HAK+MOD oraz testy negatywne:
   zmiana jednego pola GIT, kolejnosci HAK, `RACE`, `Appearance_Type` albo
   hasha zasobu ma konczyc sie czytelnym bledem. Dopiero taki artefakt moze
   zostac przekazany do live proofu.
3. Studio ma pokazywac lane i kontrakt przy downloadzie oraz nie moze nazywac
   capture proofem aktualnego builda, gdy jego MOD/HAK/hash lub placement nie
   odpowiada temu kontraktowi. Rozroznienie isolated vs canonical trzeba
   pokazac jawnie; nie laczyc ich w jednym wyroku o widocznosci.

Nie implementowano tych zmian w I2: sa wymaganiami wynikajacymi z porownania
artefaktow, wymagaja najpierw testu kontraktowego zgodnie z TDD.

Weryfikacja istniejacego generatora: `cargo test -p m2a-core --test
binary_m0_vertical_slice_module` przeszlo 2026-07-21 (`2 passed`). Potwierdza
to aktualny kontrakt **wygenerowanego** M0, lecz nie wiaze go wstecznie z
osobno zapisanym capturem v12 o innych identyfikatorach i polozeniu.

### Status po I2

- `GLTF -> Aurora transform`: **implemented / nie jest obecnie kandydatem do
  poprawki**.
- `direct-creature root`: **implemented / statycznie sprawdzony**.
- `build-to-runtime provenance`: **niepelny; najwyzszy priorytet proofowy**.
- `M0 v12 runtime capture`: **nie jest porownywalny z aktualnym
  generator-contract bez jawnego verifiera**.

## 12. Iteracja I3 -- tekstura i przezroczystosc M0 (2026-07-21)

### Dowody

| Zrodlo | Ustalony fakt | Status |
| --- | --- | --- |
| Aurora decompilation, `C:\\Projects\\New Folder\\export\\decompiled_all.c` | Wykryty gate `FUN_00a5e508` odrzuca mesh z `render=0` lub bez vertex/MDX startu; nie jest to gate kanalu alpha tekstury. | Fakt z dekompilacji, ograniczony do tej znalezionej galezi. |
| [xoreos binary mesh reader](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp) | Reader czyta `render`, resrefy tekstur i `textureCount`, laduje tekstury, a potem tworzy vertex/index buffer. | Fakt z niezaleznej implementacji; nie zastapia testu Beamdog renderer. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Jeden MDL mesh ma jeden material/teksture; alpha materialu lub tekstury ma znaczenie dla wyniku eksportu. | Fakt pomocniczy. |
| Lokalny packet `proof-output/m0-meshy-golem-20260720-v10-grounded` | `m2a_m0t01.tga` ma `RGB8`, `24` bpp, descriptor `0`: nie zawiera kanalu alpha. Writer zapisuje tez kolor wierzcholka `[255,255,255,255]`, `render=1` i brak transparency hint. | Fakt z wygenerowanego artefaktu i writer'a; nie jest proofem runtime. |

### Wniosek

Przezroczystosc tekstury nie jest przyczyna braku sylwetki M0 w capture v12.
Nie dodawac globalnej mutacji "alpha=255" ani konwersji wszystkich TGA na
RGBA: zniszczylaby legalne materialy z alpha, a nie rozstrzygnelaby obecnego
problemu pozycji/provenance. Pozostaje jedynie wymaganie obserwowalnosci:
runtime contract z I2 ma serializowac pixel format TGA, obecny kanal alpha,
vertex-color alpha i MDL `render`/transparency dla kazdego mesha, aby ta
hipoteza byla automatycznie wykluczana lub wskazywana dla przyszlych modeli.

### Status po I3

- `M0 texture alpha`: **rejected for v10/v12 source artifact**.
- `global alpha normalization`: **nie implementowac**.
- `per-package material diagnostics`: **dopiac do RuntimeFixtureContractV1**.

## 13. Iteracja I4 -- topologia prymitywu M4 i rozjazd kontraktu `meshType` (2026-07-21)

### Pytanie

Czy aktualne `meshType=0` w profilu
`M4DirectCreatureExtended64V1` jest poprawnym sposobem narysowania faces
direct-creature, czy samowystarczalny writer wprowadza M4/H1 na zla sciezke
topologii?

### Fakty z trzech wymaganych zrodel

| Zrodlo | Fakt | Granica faktu |
| --- | --- | --- |
| Aurora decompilation, `C:\\Projects\\New Folder\\export\\decompiled_all.c` | `FUN_00a5e54c` przekazuje pole mesha `+0x224` do `FUN_00a78128`, a wrapper prowadzi do `FUN_00a75e3c` i dalej do draw path. Osobno `FUN_00a62954` buduje liste indeksow z trzech `u16` zapisanych w kazdym face, wpisuje `3` pod `+0x224` i przygotowuje bufor indeksow. | Dekompilacja potwierdza przeplyw wartosci do renderera i lokalne uzycie `3`; przez utracone typy/prototypy nie nazywa enumu publicznym API. |
| [xoreos -- binary NWN mesh reader](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp) oraz [xoreos NWN1 MDL template](https://github.com/xoreos/xoreos-docs/blob/master/templates/NWN1MDL.bt) | Reader nazywa byte `+0x224` `triangleMode` i komentuje `3 = Triangle`, `4 = TriStrip`. Template opisuje enum: `0=point_list`, `3=triangle_list`, `4=triangle_strip`. Przy ladowaniu binarnego mesha xoreos buduje geometrię z face records (`vertex_id[3]`); pola `vertex_indices_count`/`vertex_indices_offset` czyta jako osobny inventory, ale nie podmienia nimi faces. | To niezalezna implementacja/specyfikacja pomocnicza, nie bezposredni proof zachowania klienta Beamdog. |
| [Tutorials / NeverBlender](https://nwn.wiki/spaces/NWN1/pages/14618048/Tutorials) i [NeverBlender documentation](https://nwn.wiki/spaces/NWN1/pages/12027194/NeverBlender-Old) | Tutorial klasyfikuje NeverBlender jako sciezke tworzenia contentu NWN. Dokumentacja NeverBlender okresla Trimesh jako siatke zlozona wylacznie z trojkatow i dokonuje triangulacji przy eksporcie; `Render` decyduje o rysowaniu mesha w grze. | Fakt o semantyce eksportu, nie o binarnym enumie engine. |

### Stan naszego kodu i wykryty rozjazd

1. Aktywny writer wybiera teraz `0` dla
   `M4DirectCreatureExtended64V1`, a `3` tylko dla
   `M0StaticRigidNativeV1` (`write_binary_mdl.rs`, plan profilu). Test
   `minimal_rigid_writer_roundtrips_every_locked_semantic_and_exact_eof`
   utrwala obecnie `mesh_type == 0`; zatem own readback nie moze wykryc
   tej klasy bledu -- potwierdza tylko to, co writer sam zapisal.
2. Ten sam plik ma komentarz, ktory nazywa `0` "triangle-list variant".
   To jest sprzeczne jednoczesnie z xoreos/template i z aktywnym suplementem
   M4: `m4-binary-writer-kontrakt-suplement-codex.md`, tabela 5.2, wymaga
   `mesh type byte 0x224 = 3` jako obserwacji R1.
3. W pierwotnej iteracji I4 lokalny re-read R1 nie zostal wykonany, bo
   `M2A_REFERENCE_CEP_HAK` byl nieustawiony. §57 wykonal juz nowy, in-place
   re-read hashowanego `cep3_core1.hak` i objal R1 oraz piec dalszych native
   direct-creature MDL. Historyczny P-REF pozostaje kontekstem, ale nie jest
   juz jedynym dostepnym odczytem niezaleznego pliku referencyjnego.

### Wniosek implementacyjny

`meshType=0` nie moze dalej byc opisywany jako lista trojkatow. Dla profilu
direct-creature, ktory zapisuje trojkatne face records i `u16` indeksy, zbieg
trzech niezaleznych obserwacji wskazuje na `3` jako docelowy wybor topologii.
To jest **P0 dla M4/H1**, w odroznieniu od M0, ktory juz emituje `3`.

Nie jest to jeszcze runtime proof, ze sama korekta naprawi H1: M4 skin ma
niezalezne otwarte ryzyko map/weights/inverse bind, a niewidoczny M0 nadal ma
nierozstrzygniety kontrakt pozycji/kamery. Nie nalezy globalnie odrzucac
`meshType=0` w readerze -- format moze legalnie reprezentowac inne prymitywy.
Gate ma dotyczyc **emitowanego profilu direct-creature**, nie kazdego MDL
spotkanego w corpusie.

### Minimalny, test-first pakiet zmian

| Priorytet | Zmiana | Wymagany dowod |
| --- | --- | --- |
| I4-P0 | Najpierw zmienic/napisac testy writer'a: rigid M4 i extended64 skin M4 musza emitowac `mesh_type=3`; M0 pozostaje `3`. Test musi wykazac, ze `0` jest odrzucony jako wartosc *emitowanego* direct-creature profilu, przy zachowaniu tolerancyjnego parsera referencyjnego. | Test czerwony na obecnym kodzie, potem own readback oraz zaktualizowane deterministyczne hashe. |
| I4-P1 | Zmienic selektor `M4DirectCreatureExtended64V1 => 3`, usunac nieprawdziwe sformulowanie "zero/triangle-list" i zapisac w raporcie czytelna etykiete `triangle-list` obok surowej wartosci. | Semantyczny readback, aktualizacja frozen SHA oraz niezalezny verifier profilu. |
| I4-P2 | Rozszerzyc `runtimeMeshEligibility` z I1: dla writer-emitted direct creature wymagac `render=1`, nonzero vertices/faces, zgodnosci face-index/raw-index i `meshType=3`; reader nadal raportuje, nie blokuje, inne tryby. | Negatywne testy pojedynczej mutacji `0`, zlego countu i zlego face/index. |
| I4-P3 | Po zamknieciu RuntimeFixtureContractV1 i osi kamery wykonac tylko A/B `M4=0` kontra `M4=3`, przy identycznych MDL poza 4 bajtami, tym samym HAK/MOD/GIT i native positive control. | Capture NWN zwiazany z hashami i placementem; wynik ma osobny status od skin/deformacji. |

### Status po I4

- `M4 meshType=0`: **contradicted by current format evidence / runtime not yet proved**.
- `M4 meshType=3`: **implementation P0; test-first correction required**.
- `M0 meshType=3`: **retain; native-supported**.
- `global reader rejection of 0`: **do not implement**.

Kontrola 2026-07-21: aktualny test
`cargo test -p m2a-core --test mdl_writer
minimal_rigid_writer_roundtrips_every_locked_semantic_and_exact_eof` przeszedl
(`1 passed`). Jest to oczekiwany wynik regresji **obecnego** writer'a, nie
kontrdowod dla I4: test sam jawnie oczekuje `mesh_type == 0`, wiec pokazuje
wlasnie koniecznosc najpierw zmienic kontrakt testowy.

## 14. Iteracja I5 -- native culling envelope kontra wlasciwe rozmiary geometrii (2026-07-21)

### Fakty z trzech wymaganych zrodel

| Zrodlo | Fakt | Granica faktu |
| --- | --- | --- |
| Aurora decompilation, `C:\\Projects\\New Folder\\export\\decompiled_all.c` | `FUN_00a5d69c` rozpoznaje ASCII `radius` / `radiuskey` / `radiusbezierkey` i zapisuje controller pod `+0x58`. W znalezionej galezi nie ma porownania naglowkowego AABB/radius. | To fakt o parserze kontrolera ASCII, **nie** potwierdzenie odczytu binary model-header radius ani predykatu cullingu. Brak porownania w tej galezi nie dowodzi, ze inna czesc klienta nie culluje po bounds. |
| [xoreos -- binary NWN model reader](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp) i [model base](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model.cpp) | Loader czyta model-level i mesh-level min/max/radius. Przy finalizacji xoreos buduje jednak wlasny bound box z transformed bounds nodow (`createBound`), a nie z naglowkowego model AABB. | Zachowanie xoreos, nie orzeczenie o zamknietym rendererze NWN:EE. Pokazuje, ze sam poprawny header nie jest substytutem zgodnej geometrii/transformow. |
| [Tutorials / NeverBlender](https://nwn.wiki/spaces/NWN1/pages/14618048/Tutorials) i [NeverBlender Transform Helper](https://nwn.wiki/spaces/NWN1/pages/12027194/NeverBlender-Old) | NeverBlender wskazuje, ze MDL obsluguje tylko scale uniform, a transform helper stosuje scale i translacje do calego modelu oraz animacji. | Fakt eksportowy: bounds do kontroli musza pochodzic z koncowych pozycji po konwersji/transformach, nie z surowych bounds GLB. |

### Co jest potwierdzone lokalnie

1. Aktualny writer emituje staly model-level envelope
   `[-5,-5,-1]..[5,5,10] / radius=7`. Powtarzalny, hashowany binarny corpus
   `cep3_core1.hak` potwierdza go teraz dla szesciu direct-creature entries:
   `c_kocrachn`, `c_phod_horror_b`, `c_phod_horror_p`, `c_nulltail`,
   `c_vampire_f` i `c_eye` (nie dla ASCII `c_squirrel`). Kazdy ma rowniez
   `geometryType=2`, `classification=4`, `fog=1` i zero child-models; test
   own ERF/MDL readera przeszedl in-place. Sam staly header pozostaje
   kontraktem obecnego writera; nie ma danych uzasadniajacych jego zmiane dla
   M0.
2. Mesh-level AABB/radius nadal sa liczone z emitowanej geometrii. W
   `write_binary_mdl.rs` writer liczy tez faktyczny model-wide world AABB i
   radius po transformie parent nodes, lecz obecnie nadpisuje wynik stalym
   native envelope przed emisja naglowka.
3. M0 nie normalizuje dowolnego rozmiaru zrodlowego -- jedynie ustawia dolny
   srodek na `Z=0`. H1 skaluje wzgledem `target_bounds`, ale ten target
   pochodzi z koncowej geometrii Meshy. Zaden z obu lane'ow nie ma obecnie
   jawnej bramki, raportu ani komunikatu, gdy finalne punkty wychodza poza
   envelope direct-creature.

### Wniosek

Nie wolno przywracac source-derived AABB/radius do naglowka modelu ani
"naprawiac" problemu widocznosci przez globalne przeskalowanie. To co nalezy
zaimplementowac, to obserwowalnosc i preflight relacji:

```text
final emitted geometry after Aurora basis + node transforms
    -> actual world AABB (report only)
    -> contained by native direct-creature envelope? (gate)
    -> HAK/MOD runtime contract
```

Nie nalezy wymagac, aby `radius=7` byl prostym promieniem od zera zawierajacym
kazdy rog AABB: same native wartosci `[-5,-5,-1]..[5,5,10] / 7` nie uzasadniaja
takiej interpretacji. Semantyka radius pozostaje **open**; gate ma porownywac
tylko potwierdzony AABB envelope.

### Zmiany do zaimplementowania

| Priorytet | Zmiana | Kryterium |
| --- | --- | --- |
| I5-P0 | Dodac do `MdlWriterReportV1` i `RuntimeFixtureContractV1` `actualWorldBounds`, `nativeHeaderBounds`, `containedByNativeHeaderBounds` oraz liste segmentow/osi poza envelope. Te dane maja pochodzic z obliczenia, ktore writer juz wykonuje. | Own readback i raport rozrozniaja native header od realnej geometrii. |
| I5-P1 | Test-first preflight direct-creature: fixture w granicach przechodzi; pojedynczy punkt poza `x/y=-5..5` lub `z=-1..10` daje stabilny blad albo jawny blokujacy gate przed pakowaniem. Nie wolno automatycznie zmieniac skali ani AABB naglowka. | Brak cichego exportu modelu, ktorego geometryczna skala nie ma native precedensu. |
| I5-P2 | Studio pokazuje rzeczywiste wymiary i wynik envelope obok lane/RuntimeFixtureContract; download/proof ostrzega o `outside-native-envelope`. | Operator nie interpretuje stalego naglowka jako faktycznego rozmiaru modelu. |
| I5-P3 | Dopiero po P0 placement/provenance wykonac osobny native A/B fixed-envelope kontra source-derived-envelope dla jednej geometrycznie contained fixture. | Rozdziela culling header od topologii I4, skinu i kamery. |

### Status po I5

- fixed native model envelope: **retain / reference-supported**.
- automatic rescale or restoration of derived header bounds: **do not implement**.
- final geometry-to-envelope preflight: **missing / implementation P1**.
- exact use of header radius by NWN renderer: **open**.


## 15. Iteracja I6 -- drzewa stanow animacji i mesh placeholders (2026-07-21)

### Pytanie

Czy dla animowanego modelu wystarczy drzewo rig-only, czy w kazdym `newanim`
musza wystapic rowniez wierzcholki odpowiadajace meshom? Jezeli tak, czy maja
one byc pelnymi kopiami geometrii, zwyklymi dummy `0x01`, czy native mesh
placeholder `0x21` bez geometrii?

### Fakty z trzech wymaganych zrodel

| Zrodlo | Fakt | Granica faktu |
| --- | --- | --- |
| Aurora decompilation, `C:\\Projects\\New Folder\\export\\decompiled_all.c` | `FUN_00a5d758` odczytuje dla `newanim` `length`, `transtime`, `animroot` i uporzadkowane eventy, nastepnie przekazuje dalsza obsluge do parsera wezlow. `FUN_00a1999c` naklada na stan wynik kontrolera (skalar, vec3 albo quaternion) przez blend z biezacym polem wezla. | Jest to bezposredni dowod parsera ASCII i evaluacji kontrolerow, ale nie odzyskuje sam typow binarnych `0x01/0x21` ani zamknietego zachowania klienta EE. |
| [xoreos -- loader stanow NWN](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp) i [xoreos -- update animacji](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/animation.cpp) | Dla kazdego binary `newanim` xoreos tworzy osobny `State`, laduje jego drzewo od `nodeHeadPointer` i buduje `AnimNode` dla wszystkich wezlow. Przy update mapuje animowany wezel na wezel modelu przez jego `_nodeNumber`; pozycje/orientacje sa nakladane tylko gdy wezel ma odpowiednie keyframes. | To bardzo dobra niezalezna wskazowka modelu danych, lecz xoreos sam zaznacza nieukonczone aspekty animacji; nie jest oraclem Beamdog. |
| [Tutorial NeverBlender: Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations) (dostepny z podanej strony Tutorials) | Zalecany workflow importuje bazowy model bez materialow i animacji, generuje rig, retargetuje go na rig Aurora, a potem przenosi animacje na poruszajace sie czesci modelu. Autor ostrzega, ze sam eksport animacji z NeverBlender moze miec uszkodzone naglowki, i zaleca wklejenie ASCII bloku animacji do modelu-szablonu, ktory NWN juz akceptuje. | Praktyczne potwierdzenie, ze kontrakt bloku animacji i tozsamosc czesci sa istotne; nie definiuje binarnego rozmiaru placeholdera. |

### Referencyjny wzorzec i stan aktualnego writer'a

Zachowany read-only zapis P-REF R3 w
`documentation/evidence/M4A-evidence.md`, sekcja 4, jest bardziej precyzyjny
niz poprzednie zalozenie M4A "rig-only": kazde z 42 drzew zachowuje base
`number/name/topology`; na klip przypada 27 wezlow, z czego 21 ma flagi
`0x21`, a 6 `0x01`. Wszystkie 882 (`42 * 21`) animation mesh placeholders
maja `vertexCount=0`, `textureCount=0` i pusty faces `ArrayDef`; blok animacji
nie ma wlasnego raw geometry. To jest silny wzorzec **mesh placeholdera**, a
nie kopia mesha i nie czysty rig-only tree. Ponowne odczytanie tego zewnetrznego
P-REF nie bylo mozliwe w I6 (brak ustawionego locatora corpusu), dlatego
liczby sa historycznym zapisem repo, nie nowym capturem.

Aktualny roboczy writer zrobil juz wazny krok: `AnimationNodeKind::Mesh`
uzupelnia topology drzewa o meshe, zachowuje ich part number, nazwe i parent,
ale zapisuje je wszystkie jako generic node `0x01` bez trackow. Test
`model_pipeline` potwierdza to dla M0: dziecko `m2a_seg_1` ma `number=1`,
`content_flags=0x01` i `mesh=None`. Wobec R3 jest to bezpieczniejszy etap niz
pomijanie wezla, lecz nadal **nie jest native profilem mesh placeholdera**.

Najwazniejsze: nie wolno odpowiedziec na ten rozjazd przez kopiowanie
`0x21/0x61` wraz z vertexami, UV, skinem i MDX do kazdego `newanim`. R3
pokazuje odwrotnie: mesh marker istnieje, ale jego geometryczne count/pointer
payloady sa puste. Wlasciwym kierunkiem jest osobny, maly binarny profil
placeholdera; skiny wymagaja dodatkowego witnessa i nie wolno automatycznie
przenosic na nie `0x21` albo `0x61`.

### Wniosek implementacyjny

`rig-only` nalezy uznac za rozjazd z dotychczasowym native corpus, a obecne
generic mesh dummy `0x01` za **niezamknieta korekte strukturalna**. P0 I4
(`meshType=3` dla base direct-creature) pozostaje wyzsze dla samego draw path.
Nastepny niezalezny kandydat po I4 to jednak nie "wiekszy animation payload",
lecz profil:

```text
base mesh node 0x21 (pelna geometria tylko w base state)
    -> animation node o tym samym part/name/parent
    -> 0x21 mesh placeholder z zerowym vertex/texture/faces
    -> tracki tylko tam, gdzie ma je output rig
```

Nie jest jeszcze dozwolone twierdzenie, ze to samo wyjasnia H1 v19: widoczny
Toolsetowy M0/H1 oraz niewidoczny runtime rozdzielaja rowniez placement,
provenance, `meshType`, skin i kamere. Te warianty musza dostac osobny A/B.

### Zmiany do zaimplementowania (test-first)

| Priorytet | Zmiana | Kryterium |
| --- | --- | --- |
| I6-P0 | Dodac `AnimationMeshPlaceholderV1` tylko dla potwierdzonego rigid/trimesh profilu. Planer ma emitowac `0x21` plus wymagany mesh common-prefix, ale `vertexCount=0`, `textureCount=0`, pusty faces ArrayDef i zadnego payloadu MDX w bloku animacji. | Nowy test byte/readback rozroznia base mesh od animation placeholdera; wymagany jest komplet `part/name/parent/topology` i brak raw geometry. |
| I6-P1 | Zmienic obecny test generic `0x01` dla child mesh na test profilu `0x21` placeholder. Dodac negatywne mutacje: pomijany mesh node, zly part/name/parent, `0x01` zamiast placeholdera oraz nonzero vertex/face w animation block maja byc rozpoznane przez policy verifier. | Own reader pozostaje tolerancyjny dla corpusu, natomiast `runtimeAnimationTreeEligibility` blokuje tylko writer-emitted direct-creature profile. |
| I6-P2 | Nie rozszerzac tego automatycznie na skin. Dodac oddzielny fixture/reference audit z lokalna animacja i skin node; do tego czasu raport ma deklarowac `animationMeshPlaceholderProfile=rigid-only-proven` oraz `skin=unproven`. | Brak falszywego `0x21`/`0x61` dla skina bez witnessa. |
| I6-P3 | Po I2 provenance/placement i I4 `meshType=3` wykonac native A/B dla identycznego rigid M0: `0x01` generic dummy kontra `0x21` zero-geometry placeholder. Jeden aktywny `cpause1`, ta sama kamera, HAK/MOD/2DA i hashe; positive control musi zostac w tym samym obszarze. | Capture rozstrzyga wplyw profilu stanu, nie miesza go z cullingiem, skinem ani zasobami. |

### Status po I6

- complete rig-only animation tree: **contradicted by recorded native R3**.
- generic mesh dummy `0x01`: **skin-only / runtime open**.
- full mesh payload w kazdym `newanim`: **do not implement**.
- zero-geometry `0x21` placeholder dla rigid animation tree: **implemented and own-readback-verified / runtime open**.
- profil animation tree dla skin: **open; witness required**.

### Realizacja I6 dla M0/RIGID (2026-07-21)

Writer alokuje teraz dla kazdego rigid mesha w drzewie `newanim` pelny common
prefix mesha (`0x270`) i zapisuje `content_flags=0x21`, ale bez vertexow,
faces, tekstur, raw indices ani payloadu MDX. Zachowane sa `part`, `name`,
parent i topology drzewa. Wlasny semantic readback wymaga tych pustych pol,
zatem przypadkowe cofniecie do `0x01`, dodanie geometrii lub zly layout nie
moze przejsc bez zmiany testu.

Zakres jest celowo waski: rigid M0 i inne rigid segments dostaja placeholder
`0x21`; skin mesh pozostaje generic `0x01`. Nie przenosimy profilu placeholdera
na skin bez niezaleznego native witnessa.

Weryfikacja: `cargo test -p m2a-core --test mdl_writer` (31/31) oraz
`cargo test -p m2a-core --test model_pipeline` (14/14), w tym readback M0
z jednym `0x21` placeholderem na kazdy aktywny clip. Nie wykonano live Aurora
ani NWN, wiec wynik runtime nadal jest `missing`.


## 16. Iteracja I7 -- zamknieta diagnoza: bug w logice `eligibility` M0 (2026-07-21)

### Diagnoza

To nie jest aktualny blad emisji statycznego M0 ani powod, dla ktorego
dzisiejszy green M0 znika w runtime. To jest blad **bramki kontraktu**:
wyliczony werdykt `eligible` nie jest nigdzie wymagany.

Dokladna sciezka jest w
`crates/m2a-core/src/model_pipeline.rs`:

1. `m0_runtime_mesh_eligibility_v1` (linia 819) wylicza w linii 847 tylko
   narrow pre-gate:

   ```rust
   mesh.render != 0 && (mesh.vertex_count != 0 || mesh.start_mdx != 0)
   ```

   i zapisuje wynik do `M0RuntimeMeshEligibilityV1::eligible`.
2. `build_m0_binary_runtime_fixture_contract_v1` (linia 936) tylko umieszcza
   ten wynik w kontrakcie; nie zwraca bledu, gdy `eligible == false`.
3. `verify_m0_binary_runtime_fixture_contract_v1` (linia 1090) porownuje
   jedynie, czy zserializowany wynik jest rowny wynikowi ponownie obliczonemu.
   Nie wymaga, aby oba byly `true`.

W konsekwencji self-consistent MOD/HAK z niekwalifikujacym sie meshem moze
mieć kontrakt z `eligible=false`, ktory verifier uzna za zgodny. Pole jest
wiec obecnie telemetryczne, a nie ochronne -- mimo nazwy i przeznaczenia
runtime fixture contract.

### Fakty z trzech wymaganych zrodel

| Zrodlo | Fakt | Znaczenie dla buga |
| --- | --- | --- |
| Aurora decompilation, `C:\\Projects\\New Folder\\export\\decompiled_all.c` | `FUN_00a5e508` odrzuca wezel, gdy nie ma render flag, runtime mesh albo jednoczesnie `vertexCount==0` i `startMdx==0`. `FUN_00a5e54c` pozniej przekazuje `+0x224` do draw path. | Obecny predicate jest wierna **czescia** wykrytej wstepnej bramki, ale taki wynik musi byc w kontrakcie egzekwowany. Sama dekompilacja nie uzasadnia, ze to pelny test gotowosci do narysowania. |
| [xoreos -- NWN binary mesh reader](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp) | `readMesh` konczy ladowanie bez utworzenia MeshData, gdy `vertexCount==0`, `facesCount==0` albo `facesOffset==0`; dopiero potem czyta vertices i face indices. Rozpoznaje takze `triangleMode` 3/4. | Potwierdza, ze `vertexCount/startMdx` to za malo, by nazwac mesh rzeczywiscie rysowalnym; face topology i tryb prymitywu pozostaja istotne. To nie jest oracle klienta Beamdog. |
| [Tutorial NeverBlender: Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations) | Rekomendowany workflow wkleja wynikowy blok do modelu-szablonu juz akceptowanego przez NWN, poniewaz eksport NeverBlender moze miec problemy z naglowkami. | Praktycznie uzasadnia rygor po eksporcie: kontrakt nie moze tylko zarejestrowac zlego stanu jako faktu; musi go odrzucic. Nie definiuje binarnego gate'a. |

### Czy dotyczy dzisiejszego statycznego M0?

**Nie jako potwierdzona przyczyna obecnego M0.** Aktualny test
`static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice`
przeszedl w I7 (1/1). W tym artefakcie jest dokladnie jeden direct child mesh,
`render=1`, `vertexCount>0`, `startMdx=0` i `meshType=3`; jego aktualny
wynik `eligible=true` jest zatem prawidlowy nawet wedlug waskiego predicate.

**Tak jako luka przyszlego/tampered statycznego M0.** Gdyby przyszla zmiana
writera ustawila `render=0` albo wyzerowala vertex stream, builder moglby
zapisac `eligible=false`, a verifier nadal zaakceptowalby kontrakt z tym
samym `false`. Dodatkowo obecny predicate dalby false-positive dla
`render=1, vertexCount>0, faces=0` albo dla `meshType=0`: przypadki nie
wystepuja w obecnym M0, lecz nie sa wykluczone przez sam verifier. Skany sa
tez celowo plytkie (`root.children`), co jest zgodne z aktualnym M0
`root -> mesh`, ale nie jest ogolnym traversalem modelu.

### Minimalna poprawka (P0) i test

Minimalna poprawka nie zmienia osi, windingu, MDX ani writer'a. Nalezy dodac
jedna wspolna funkcje, np.
`require_m0_runtime_mesh_eligibility_v1(&eligibility)`, ktora zwraca stabilny

```text
M0-RUNTIME-CONTRACT-MESH-INELIGIBLE
path: contract.meshEligibility
```

gdy `eligible == false`. Trzeba wywolac ja:

1. zaraz po `m0_runtime_mesh_eligibility_v1(model)?` w builderze kontraktu;
2. po recompute w verifierze -- **przed** return `Ok(())`.

To zamyka blad self-consistent `false` bez zmiany poprawnego M0 i bez
rozszerzania publicznego formatu MDL.

Test-first dla P0:

1. dodatni: obecny statyczny M0 nadal buduje i weryfikuje kontrakt z
   `eligible=true`;
2. ujemny builder: fixture/model mutowany tylko w `render=0` (albo
   `vertexCount=0,startMdx=0`) ma zwrocic dokladnie
   `M0-RUNTIME-CONTRACT-MESH-INELIGIBLE`, a nie zapisac kontraktu;
3. ujemny verifier: self-consistent MOD/HAK i kontrakt z tym samym
   `eligible=false` ma zostac odrzucony tym samym kodem. To jest test,
   ktorego obecny kod nie przejdzie.

### Nastepna, ale osobna poprawka predicate (P1)

Dopiero po P0 nalezy zawęzic predicate **tylko dla writer-emitted M0
direct-creature**, nie dla ogolnego parsera, do:

```text
render == 1
&& meshType == 3
&& vertexCount > 0
&& faces niepuste
&& każdy face index < vertexCount
&& jeden raw-index stream = konkatenacja face indices
&& indexCount odpowiada temu streamowi
&& zdekodowany POSITION stream ma vertexCount pozycji
```

Negative tests P1 powinny osobno mutowac `meshType=0`, puste faces i rozjazd
raw-index/face przy zachowanym `render=1, vertexCount>0`; stary predicate
błędnie oznaczylby je jako `eligible`, nowy musi odrzucic. Nie mieszac P1 z
P0 ani nie stosowac go do referencyjnych modeli o innych legalnych primitive
modes.

### Status po I7

- primary eligibility bug (wynik `false` jest akceptowany): **confirmed**.
- current static M0 emitted artifact: **not affected / 1 targeted test passed**.
- P0: **enforce `eligible==true` in builder and verifier**.
- P1: **tighten M0-emitter predicate; do not globally tighten the reader**.
- Aurora/NWN execution and INI changes: **none**.

### Realizacja I7 (2026-07-21)

P0 i P1 zostaly zaimplementowane w
`crates/m2a-core/src/model_pipeline.rs`. Kontrakt jest teraz emitowany tylko,
gdy `eligible == true`; verifier ponownie liczy gate i odrzuca nawet
samospojny MOD/HAK/kontrakt z `eligible == false` kodem
`M0-RUNTIME-CONTRACT-MESH-INELIGIBLE` na sciezce
`contract.meshEligibility`.

Gate dotyczy wylacznie emitowanego M0 direct-creature. Wymaga `render == 1`,
`meshType == 3`, niepustych vertices i faces, zgodnosci wszystkich face
indices z `vertexCount`, jednego raw-index streamu rownego konkatenacji faces
oraz zgodnego licznika indeksow. Ogolny reader binary MDL pozostaje
tolerancyjny dla innych legalnych trybow referencyjnych.

Weryfikacja lokalna:

- `cargo test -p m2a-core --test model_pipeline static_meshy_m0` — 4/4;
- `cargo test -p m2a-core --test model_pipeline m0_runtime_mesh_eligibility` —
  odrzuca `render=0`, `meshType=0`, puste faces i rozjazd raw-index/faces;
- `npm test -- --run src/features/results/projectCanonicalResult.test.ts` —
  27/27;
- `npm run typecheck` w `apps/studio-web` — passed.

To jest wzmocnienie preflightu eksportu, **nie proof runtime**. Aktualny M0
pozostaje `verified` dla świeżego viewportu Toolsetu i `missing` dla
wiążącego capture NWN, dopóki nie powstanie profil akceptowany przez centralną
bramkę native geometry/AUR-S07. Nie wykonano live Aurora/NWN ani nie zmieniono
`nwtoolset.ini`, MRU ani konfiguracji gry.


## 18. Iteracja I8 -- ASCII, binary MDL i pola runtime: ktorych zer nie „naprawiac” (2026-07-21)

### Pytanie

Eksport binarnego MDL zawiera kilka pol wygladajacych jak wskazniki/routine oraz
wiele wartosci zero. Celem I8 jest rozdzielenie:

1. zera, ktore sa legalnym stanem wybranego profilu;
2. zera, ktore bezposrednio wylaczaja draw path;
3. pol nieudowodnionych, ktorych nie wolno „naprawiac” adresem, bajtami z
   cudzego modelu ani przypuszczeniem.

Nie zmieniono writera, Aurora Toolsetu, NWN ani konfiguracji gry.

### Fakty ze zrodel

| Zrodlo | Fakt | Granica wniosku |
| --- | --- | --- |
| Aurora decompilation, C:\Projects\New Folder\export\decompiled_all.c, FUN_00a5d758 | Ta funkcja rozpoznaje tekstowe dyrektywy sekcji animacji, m.in. animlength, event, length, transtime i animroot. | To jest dowod, ze ASCII MDL jest osobna sciezka parsera tekstowego; nie opisuje layoutu binarnego ani nie legitymizuje eksportu ASCII jako runtime artefaktu produktu. |
| Aurora decompilation, FUN_00a5e508 | Render gate odrzuca obiekt przy wylaczonym stanie, braku runtime mesha pod +0xdc, albo gdy jednoczesnie vertexCount @ +0x230 == 0 i startMdx @ +0x228 == 0. Potem FUN_00a5e54c przekazuje wartosc +0x224 do draw path. | To jest najblizszy lokalny fakt o tym, ktore pola sa semantyczne dla rysowania. Nie nadaje znaczenia nieopisanym slotom routine w pliku. |
| [xoreos -- reader NWN binary MDL](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp) | Reader pomija 8 bajtow „Function pointers” w naglowku modelu, 24 w naglowku node i 8 w bloku mesha. Nastepnie czyta pola semantyczne, w tym render, faces, triangleMode, vertex stream i tekstury. | To jest niezalezny od Aurory opis konsumpcji formatu, a nie oracle klienta Beamdog. „skip” dowodzi tylko, ze xoreos ich nie interpretuje; nie dowodzi, ze kazda wartosc jest legalna w NWN. |
| xoreos, readMesh | Przed utworzeniem danych renderera zwraca z mesha, gdy vertexCount==0, facesCount==0 albo facesOffset==0; komentarz opisuje mode 3 jako Triangle, 4 jako TriStrip. | Jest to silny dodatkowy gate dla naszego emitowanego M0, lecz nie formalny pelny gate retailowego renderera. |
| [NWN Wiki -- Models](https://nwn.wiki/spaces/NWN1/pages/38175602/Models) oraz [Importing Animations / NeverBlender](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations) | Wiki rozdziela niekompilowany ASCII od skompilowanego modelu i zaleca kompilator gry dla pelnej zgodnosci EE. Tutorial NeverBlender opisuje przypadki, w ktorych eksport ma problem z naglowkiem, a poprawny blok ASCII jest wklejany do szablonu juz akceptowanego przez NWN. | To jest praktyczny dowod, ze naglowek/struktura sa istotne; nie zmienia zasady projektu: runtime produkcyjny pozostaje native binary MDL, a ASCII tylko materialem diagnostycznym. |

### Klasyfikacja zer i sentinel values dla obecnego profilu

| Pole / wartosc | Klasyfikacja | Uzasadnienie i decyzja |
| --- | --- | --- |
| Model/node/mesh „function pointers” | **nieudowodnione runtime fields** | xoreos je omija. Obecny writer pozostawia je zerowe poprzez zainicjalizowany bufor; kontrakt M4 oznacza je jawnie OPEN_M6. Nie wpisywac adresow kodu, nie kopiowac wskaznikow z referencji i nie traktowac tych zer jako potwierdzonej przyczyny niewidocznosci. |
| startMdx = 0 przy vertexCount > 0 | **legalne w M0** | Aurora odrzuca tylko koniunkcje obu zer. Aktualny M0 ma pozycje w raw streamie i startMdx=0; jest to zamierzony native-style profil. |
| vertexCount = 0 oraz startMdx = 0 | **niesprawny dla draw gate** | Bezposredni warunek FUN_00a5e508. Nalezy odrzucac przed pakowaniem/runtime fixture. |
| render = 0 | **niesprawny dla draw gate** | Aurora wymaga runtime flag pod +0xdc; obecny M0 writer emituje render=1, a jego preflight wymaga tej wartosci. |
| facesCount = 0 lub facesOffset = 0 przy geometrii M0 | **odrzucic dla emitowanego M0** | xoreos nie tworzy w takim przypadku danych mesha; aktualny M0 gate takze wymaga niepustych faces oraz zgodnego streamu indeksow. To jest gate profilu produktu, nie globalny zakaz dla dowolnego referencyjnego MDL. |
| meshType = 0 | **nie jest bezpiecznym domyslem dla M0** | Aurora przekazuje to pole do draw path, xoreos dokumentuje 3/4 jako znane modes, a kontrakt M0 wymaga 3. Historyczna sciezka M4 z zerem jest osobnym OPEN_M6, nie precedensem dla statycznego M0. |
| raw offsets -1 / 0xffffffff dla nieuzytych strumieni | **sentinel, nie zero** | Nie normalizowac do zera: xoreos porownuje takie offsety z 0xffffffff przed proba odczytu. |

### Wniosek implementacyjny

Brak podstaw do implementacji „wypelnij routines/wskazniki”. W binarnym pliku
nie wolno serializowac adresow funkcji z procesu, pointerow z cudzego modelu
ani wartosci zależnych od wersji klienta. Taka zmiana bylaby nieprzenosna i
nie mialaby lokalnego dowodu, ze rozwiazuje niewidocznosc.

Zamiast tego priorytet diagnostyki pozostaje:

1. pakiet i resolver (appearance.2da, resref modelu/tekstury, HAK/MOD);
2. semantyczny gate mesha (render=1, meshType=3, vertices, faces, indeksy,
   pozycje);
3. bounds/orientacja/material i tekstura;
4. dopiero na koncu porownanie nieudowodnionych pol binary layoutu.

Dla raportu artefaktu nalezy dalej oznaczac routines jako
opaqueRuntimeFields: zero/unverified, a nie jako „runtime valid”. Nie jest
to zadanie do przypisania wartosci, dopoki kontrolowany proof runtime nie
wykaze konkretnej zaleznosci.

### Nastepny proof, gdy wlasciciel osobno autoryzuje zywe NWN

Minimalny eksperyment A/B musi uzyc tego samego wygenerowanego M0 oraz
niezmienionego pakietu resource/appearance:

1. zachowac hash binarnego MDL i pelny readback semantyczny;
2. utworzyc wyłącznie diagnostyczny ASCII snapshot o tej samej geometrii;
3. skompilowac go przez zalecana przez wiki sciezke loaded in game,
   nie przez losowy zewnetrzny compiler;
4. porownac tylko sklasyfikowane pola binarne, a potem sprawdzic Toolset i NWN;
5. uznac pole routine za istotne wyłącznie, gdy zmiana jest powtarzalna i
   koreluje z widocznym wynikiem przy identycznych zasobach.

Ten eksperyment pozostaje **missing**: w I8 nie uruchomiono Aurora/NWN i nie
zmieniono zadnego INI. Przed nim nadal obowiazuje centralna native geometry
bramka oraz osobna zgoda wlasciciela na live action.

### Status po I8

- ASCII i binary MDL: **rozroznione**; ASCII nie jest zamiennikiem runtime
  binary MDL produktu.
- Zera w polach routine: **opaque / OPEN_M6**, nie kandydat do „naprawy”.
- startMdx=0 z poprawnym vertex streamem: **dozwolone dla M0**.
- render=0 albo vertexCount=0 && startMdx=0: **potwierdzony non-renderable
  gate**.
- Puste faces/face pointer oraz zly meshType: **odrzucane dla M0 przez
  profil produktu**, z dodatkowym wsparciem xoreos.
- Proof w prawdziwym Toolset/NWN: **missing**, bez zywej akcji w tej iteracji.


## 19. Iteracja I9 -- resolver: GIT, appearance.2da, MDL, tekstura i HAK (2026-07-21)

### Wynik

Dla creature typu direct model jedynym meaningful łańcuchem nie jest „MDL
istnieje w HAK”, lecz:

    GIT Appearance_Type (fizyczny indeks wiersza)
      -> appearance.2da[ten indeks]
      -> MODELTYPE = S (profil statycznego M0)
      -> RACE = m2a_m0p01
      -> resource MDL m2a_m0p01 (type 2002)
      -> każdy resref tekstury wskazany przez mesh
      -> resource tekstury o tym resrefie (type 3)

Po załadowaniu xoreos naklada jeszcze pozycje i orientacje instancji. Zatem
poprawny resolver i poprawny draw gate są konieczne, ale żaden nie jest
samodzielnym dowodem, że fixture jest w kadrze klienta.

### Fakty ze zrodel

| Zrodlo | Fakt | Znaczenie |
| --- | --- | --- |
| Aurora decompilation, C:\Projects\New Folder\export\decompiled_all.c, FUN_00463fe8 | Lokalny loader obiektu odczytuje GFF Appearance_Type i przekazuje wartosc do pola appearance; odczytuje też TemplateResRef. Osobna funkcja FUN_00463d54 pobiera MODELTYPE z tabeli appearance, pokazuje blad „Invalid Appearance / ModelType for appearance not found”, a dla P uruchamia sciezke czesci. | Potwierdza, ze Appearance_Type i MODELTYPE sa funkcjonalnym kontraktem Toolsetu, a nie metadanymi raportu. Nazwy typow struktur sa nadal robocze po dekompilacji. |
| [xoreos creature resolver](https://github.com/xoreos/xoreos/blob/master/src/engines/nwn/creature.cpp) | Odczytuje Appearance_Type z GFF. Dla MODELTYPE=P buduje model z czesci; w przeciwnym razie laduje pojedynczy model spod RACE. Nastepnie aplikuje X/Y/Z i orientation instancji. | Jest to niezalezna implementacja resolvera; potwierdza caly kierunek zaleznosci, lecz nie jest zamknietym oracle NWN:EE. |
| [appearance.2da](https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da) | RACE jest resrefem modelu. MODELTYPE S oznacza jeden MDL z pojedyncza tekstura TGA/DDS; P to model skladany. LABEL jest pomoca Toolsetu, ale nie jest polem runtime gry. | Dla M0 powinno byc wymagane S oraz RACE wskazujacy dokladnie jego MDL; nie wolno wykonywac runtime gate tylko po LABEL. |
| [2DA Files](https://nwn.wiki/spaces/NWN1/pages/38174875/2da%2BFiles) | Pierwsza kolumna z numerem jest ignorowana: engine indeksuje fizyczne niepuste wiersze od zera. appearance.2da jest ladowane przez klienta, serwer i Toolset, z limitem indeksu 0..65535 wynikajacym z pola GFF WORD. | Kontrakt musi przechowywac physical row, nie tylko tekst z pierwszej kolumny ani LABEL. |
| [xoreos NWN model loader](https://github.com/xoreos/xoreos/blob/master/src/engines/nwn/modelloader.cpp) | Loader ma cache modeli; komentarz wskazuje, że module/HAK mogą nadpisywać model i cache powinien być czyszczony po unloadzie modułu. | Kolejnosc i środowisko zasobów są częścią proofu. Nie wolno uznać stalego runtime capture za proof nowego HAK po samej zmianie pliku. |

### Co aktualny M0 juz dowodzi

Generator M0 appenduje row z MODELTYPE=S i RACE=m2a_m0p01, pakuje do jednego
HAK model m2a_m0p01 (type 2002), teksture m2a_m0t01 (type 3) oraz
appearance.2da (type 2017), a MOD odczytywany przez binary-scene verifier
wskazuje jeden uporzadkowany HAK i jeden physical appearance row.

Istniejacy test static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice
sprawdza dodatnio:

- label, MODELTYPE=S i RACE=m2a_m0p01 w bindzie runtime;
- hash MOD, HAK, MDL, TGA i appearance.2da;
- polozenie oraz kierunek IFO/GIT;
- pojedynczy renderable mesh i jego resref tekstury m2a_m0t01;
- negatywna mutacje GIT, kontraktowego RACE, HAK hash i niekwalifikujacego
  render gate.

To jest mocny proof build/readback aktualnie emitowanego M0. Nie jest
dowodem, że dowolny samospójny, zewnętrznie zmieniony MOD/HAK wybierze ten sam
model co generator.

### Dokladna luka verifiera semantyki resolvera

Funkcje build_m0_binary_runtime_fixture_contract_v1 oraz
verify_m0_binary_runtime_fixture_contract_v1 odczytuja LABEL, MODELTYPE i
RACE z physical row i porownuja strukture M0RuntimeAppearanceBindingV1.
W generatorze wartosci sa poprawne, ale verifier nie wymaga niezaleznie:

- MODELTYPE == S;
- RACE == m2a_m0p01;
- każdy niepusty mesh texture resref == m2a_m0t01;
- każdy taki texture resref istnieje w HAK jako type 3.

W efekcie kontrakt i HAK jednoczesnie zmienione na MODELTYPE=P albo inny RACE
moglyby byc wzajemnie zgodne hashami i komorkami, lecz nie reprezentowalyby
M0 direct-creature. Analogicznie model moglby wskazywac inna teksture, gdy HAK
nadal zawiera nieuzywane m2a_m0t01: obecny mesh eligibility ponownie zapisze
nowy resref, ale nie łączy go z zasobem texture z kontraktu.

To jest **potwierdzona luka semantyczna verifiera**, nie blad aktualnego
emitowanego M0. Aktualny builder zapisuje prawidlowe wartosci, a pozytywny test
to potwierdza.

LABEL nalezy nadal serializowac jako provenance/Toolset metadata, ale nie
traktowac jego innej wartosci jako przyczyny runtime niewidocznosci, ponieważ
gra go nie uzywa.

### Minimalna implementacja po TDD (I9-P0)

Dodac wspolny direct-M0 resolver gate wywolywany przez builder oraz verifier:

1. odczytaj physical row wskazany przez GIT;
2. wymagaj MODELTYPE=S i RACE=m2a_m0p01;
3. odczytaj wszystkie non-empty texture resrefs z readbacku meshy;
4. wymagaj, aby zbior byl dokladnie oczekiwanym zbiorem profilu M0, a kazdy
   resource istnial w tym samym HAK jako TGA/DDS zgodnego type;
5. opcjonalnie do raportu dodaj actualModelHeaderName oraz actualRootName,
   lecz nie zastępuj nimi resrefu HAK.

Stabilne bledy powinny rozroznic:

- M0-RUNTIME-CONTRACT-APPEARANCE-MODELTYPE-INVALID;
- M0-RUNTIME-CONTRACT-APPEARANCE-RACE-MISMATCH;
- M0-RUNTIME-CONTRACT-MESH-TEXTURE-RESREF-MISMATCH;
- M0-RUNTIME-CONTRACT-MESH-TEXTURE-MISSING.

Testy negatywne musza mutowac jedną zaleznosc naraz i odtworzyc HAK oraz
kontrakt tak, by byly samospójne. Obecne testy mutuja tylko obiekt kontraktu,
wiec nie wykazuja jeszcze tej luki.

### Granica zewnetrznego srodowiska

Jeden HAK i unikalne resrefy M0 ograniczaja collision risk, ale nie dowodza
braku override/userpatch albo cache'u klienta. Wiki ostrzega o konfliktach
globalnego contentu, a xoreos jawnie notuje cache i overrides modulu/HAK.
Tego nie da sie uczciwie zamknac samym parserem MOD/HAK.

Przed live proofem packet ma zatem zawierac: hash oraz uporzadkowana liste
HAK z IFO, pelny inventory zasobow M0, timestamp/lane capture i wskazanie
czystego lub jawnie opisanego środowiska klienta. Live proof musi zaczynac
nowa sesje wedlug centralnego standardu; I9 nie uruchomil Aurora/NWN.

### Status po I9

- GIT physical appearance row -> MODELTYPE/RACE -> MDL: **potwierdzony
  kontrakt resolvera**.
- Aktualny M0 build/readback: **verified** dla poprawnej konfiguracji.
- Samospójny, ale semantycznie zly resolver packet: **nieodrzucany w pelni;
  I9-P0**.
- LABEL jako runtime gate: **nie implementowac**.
- Cache/override/userpatch proof runtime: **missing**, wymaga kontrolowanej
  sesji klienta.


## 20. Iteracja I10 -- bazowy mesh, stany animacji i granica skinu (2026-07-21)

### Najwazniejszy podzial

Dla prostego M0 typu MODELTYPE=S trzeba rozdzielic trzy warstwy:

1. bazowy stan modelu z meshem, który decyduje, czy istnieje geometria do
   narysowania;
2. stany animacji, które podmieniaja lub blenduja transformaty wezlow;
3. skinning, który dopiero wykorzystuje palette kości i wagi wierzcholkow.

Nie wolno zmieniac statycznego M0 w skinmesh ani dodawac ruchu tylko po to,
aby „wymusic widocznosc”. To mieszaloby oddzielne hipotezy i zniszczylo
najprostsza kontrolę pozytywną.

### Fakty ze zrodel

| Zrodlo | Fakt | Znaczenie |
| --- | --- | --- |
| Aurora decompilation, FUN_00a5d758 i FUN_00a1999c w C:\Projects\New Folder\export\decompiled_all.c | Parser animacji rozpoznaje length, transtime, animroot i event. Evaluator blenduje controller skalarny, vec3 oraz quaternion do biezacego stanu wezla. | Animacja ma zdolnosc zmieniac stan wezlów, ale nie stanowi dowodu, że brak jednego klipu usuwa bazowy mesh. |
| [xoreos binary NWN model reader](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp) | loadBinary tworzy bazowy State i laduje jego root mesh przed iteracja po animation offsets. readAnimBinary tworzy osobny State z name, animLength, transTime, animRoot, events oraz drzewem nodes. Reader controllerow odroznia position, orientation i alpha; alpha 0 w stanie poczatkowym wylacza render w xoreos. | To mocna niezalezna wskazowka kolejności danych. Zachowanie alpha jest opisem xoreos, nie bezposrednim dowodem Beamdog. |
| [appearance.2da](https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da) | S to single-MDL, prosty model creature używajacy nazw creature animations, w tym crun i cwalk. Footstep event wymaga wpisu w animacji, aby odtworzyc dzwiek. | Dla M0 nazwy creature clipów sa prawidlowym profilem zgodnosci; same eventy nie warunkują widocznosci geometrii. |
| [NeverBlender animation tutorial](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations) | Poprawny workflow zaczyna od bazowego modelu/armature, zachowuje zgodny origin i przenosi animacje na odpowiadajace czesci. Eksport moze miec problem z naglowkami, dlatego autor stosuje szablon zaakceptowany przez NWN. | Praktyczne potwierdzenie, że animation tree i bind pose muszą być zgodne z base model; nie jest regułą, że pełna geometria ma być kopiowana do kazdego stanu. |
| [Models](https://nwn.wiki/spaces/NWN1/pages/38175602/Models) | W aktualnym EE skinmesh ma limit 64 kości na node i maksymalnie 4 wpływy na wierzcholek; większą geometrię należy dzielić na mesh nodes. | Jest to preflight H1/skin, nie warunek rigid M0. |

### Stan aktualnego M0

M0 emituje jeden geometry root nazwany m2a_m0p01, niezaleznie renderowalny
mesh oraz siedem konservatywnych creature state names:

- cappear;
- cpause1;
- cwalk;
- crun;
- ca1slashl;
- cdamagel;
- cdead.

Dla statycznej kontroli wszystkie tracki są puste, kazdy stan ma ten sam
animroot i czas 1 s. To jest profil produktu oparty na lokalnym
direct-creature reference, a nie udowodnione przez engine minimalne minimum
nazw stanów.

Ponieważ M0 nie emituje controllerow alpha ani zmieniajacych position/
orientation, nie ma w tym artefakcie animacyjnej sciezki, która celowo
ukrywałaby bazowy mesh. Aktualny target testu potwierdza brak ruchu
cpause1 oraz renderable base mesh.

### Co wynika dla diagnostyki

| Przypadek | Interpretacja |
| --- | --- |
| Rigid M0 niewidoczny od pierwszej klatki | Najpierw badaj resolver, geometry gate, bounds, material, placement i cache. Nie przypisuj automatycznie winy brakowi motion/skin. |
| Model znika dopiero po zmianie state | Wtedy zbadaj dwa osobne artefakty: topology/name/animroot danego state oraz controllery alpha/position/orientation. Nie kopiuj base mesh payloadu do state bez witnessa. |
| RIGID widoczny, H1 skin zdeformowany | Przejdz drabine S1: jedna kość, weight 1.0, identity inverse bind; dopiero potem base pose i jeden ruch. To jest problem skinu, nie dowod na zly resolver M0. |
| Liczba kości >64 albo >4 wpływy | Odrzuc przed writerem lub podziel mesh. Nie obcinaj wagi skrycie po runtime failure. |

### Aktualne otwarte ryzyko tree animacji

Historyczny P-REF R3 opisany w I6 pokazuje mesh placeholders z flagą 0x21,
bez vertex/faces, w drzewach animacji. Aktualny writer zachowuje meshowe wezly
animacji, lecz emituje je jako generic 0x01. To jest strukturalne OPEN_M6:

- nie wpływa na potwierdzona base geometry M0 przed zmianą animacji;
- może mieć znaczenie przy przejsciu state albo dla animowanego H1;
- nie uzasadnia kopiowania vertices, UV, indeksow lub skin payloadu do
  każdego anim state.

Minimalny następny krok to test-first profile placeholdera: rigid animation
mesh marker 0x21 z pustymi geometry counts/pointers, zachowanym name/number/
parent. Dla skin marker potrzebny jest oddzielny witness i osobny proof.

### Status po I10

- Base mesh kontra animation states: **rozroznione**.
- Puste tracki statycznego M0 jako przyczyna braku widocznosci od pierwszej
  klatki: **niepotwierdzone i slabe**.
- Alpha controller jako mozliwy późniejszy cause: **monitorowac per state**.
- 64 bones / 4 influences: **twardy preflight skinu, nie M0 rigid**.
- Native animation mesh placeholder 0x21: **OPEN_M6**, wymaga osobnego
  structural testu i runtime A/B.


## 21. Synteza decyzji implementacyjnych po I1--I10 (2026-07-21)

Ten priorytet nie miesza dwoch celow: M0 jest kontrola rigid/direct-creature
do ustalenia widocznosci i proofu, a H1/M4 jest trudniejszym torem
skin/deformacji. Nie wolno wprowadzac poprawki H1 do M0 ani uznawac
pojedynczego capture za proof obu torow.

| Priorytet | Zmiana / decyzja | Dokladny zakres | Test przed implementacja | Czego wynik jeszcze nie dowodzi |
| --- | --- | --- | --- | --- |
| A0 | **Nie zmieniac binarnych function pointers/routines** | Nie serializowac adresow procesu ani nie kopiowac ich z referencji. W raporcie pozostawic zero/unverified. | Parser/readback toleruje zapisane zero, bez testu „adres jest poprawny”. | Akceptacji takich pol przez klienta; obecnie nie ma podstaw, by uznac je za root cause. |
| A1 | **Dopiac M0 semantic resolver gate** | crates/m2a-core/src/model_pipeline.rs: wymagaj MODELTYPE=S, RACE=m2a_m0p01 oraz wszystkich resrefów tekstur mesha w tym samym HAK. | Samospójny mutant HAK+MOD+kontrakt z P, zlym RACE albo obca/brakujaca tekstura musi zostac odrzucony. | Draw calla NWN; zamyka false-positive prover, nie renderer. |
| A2 | **Wiazacy proof runtime M0** | Najpierw centralna bramka native geometry, potem nowa sesja z jednym hashowanym MOD/HAK, określonym environment inventory, placementem i capturem. | Offline verifier A1, geometry gate i readback GIT/IFO musza przejsc przed live lane. | H1 skin/deformacji. Bez tego nie klasyfikowac pustego kadru jako odrzucenia MDL. |
| B1 | **Naprawic M4 direct-creature meshType** | crates/m2a-core/src/mdl/write_binary_mdl.rs: M4DirectCreatureExtended64V1 ma emitowac 3, nie 0; usunac komentarz nazywajacy 0 triangle-list. | Najpierw zmienic oczekiwania mdl_writer dla M4 rigid i skin na 3; parser referencyjny ma nadal raportowac, nie odrzucac 0. | Naprawy deformacji H1. 3 jest silnym formatowym kandydatem, lecz wymaga osobnego runtime A/B. |
| B2 | **Dodac report/gate native envelope** | Writer i runtime contract raportuja final world AABB oraz contained-by-native-envelope; blokują geometry outside known direct-creature envelope, bez auto-rescale. | Fixture contained przechodzi, jeden punkt poza granica daje stabilny blad. | Semantyki radius=7 albo całego cullingu klienta. |
| B3 | **Native animation mesh placeholder** | writer animacji: dla rigid mesh node w state zachowac name/number/parent i native-style 0x21 bez geometrii; skin dopiero po osobnym witnessie. | Readback state tree odroznia 0x21 empty mesh marker od generic 0x01 i nie emituje raw geometry. | Ze to poprawi pierwsza klatke M0; dotyczy przede wszystkim state transitions/H1. |
| C1 | **Skin ladder, nie „pełny H1 od razu”** | Jedna kość/identity, base pose, jeden ruch, pelny rig. Pozostawic twarde 64 bones i 4 influences. | Osobny fixture i proof na kazdym stopniu; brak cichego obcinania wag. | Zgodności wszystkich source rigów bez proofu. |
| C2 | **Material diagnostics per mesh** | Raportuj render, transparency, vertex alpha, texture resrefs, pixel format i ewentualne MTR/TXI. | Mutacja alpha/render/resource musi wskazywac konkretna przyczyne. | Ze brak tekstury sam z siebie oznacza invisibility; zwykle jest to osobna, widoczna degradacja. |

### Co usunac lub wycofac

1. W M4 writerze trzeba usunac stale twierdzenie, że wartosc meshType=0 jest
   „triangle-list variant”; lokalna dekompilacja, retail corpus i xoreos
   wspieraja 3 dla tej topologii.
2. Nie dopuszczac oznaczenia „runtime valid” dla artefaktu, ktory przeszedl
   tylko own writer/readback. Poprawny status to verified offline albo
   verified Toolset; NWN wymaga osobnego capture.
3. Nie traktowac LABEL w appearance.2da jako runtime resolver gate; jest to
   metadata Toolsetu, nie wybor MDL przez grę.
4. Nie wprowadzac globalnego alpha=255, globalnego rescale, globalnego
   odrzucania meshType=0 w readerze ani kopiowania base geometry do state
   animations. Kazda z tych zmian maskuje informacje i wykracza poza dowody.

### Definition of done dla „model widoczny i dziala w NWN”

Status mozna zamknac tylko, gdy jeden identyczny hash modelu:

1. przejdzie semantic writer/readback, A1 resolver gate i B2 geometry gate;
2. jest potwierdzony w Aurora na zapisanej Area;
3. jest widoczny w NWN w kontrolowanym capture powiazanym z MOD/HAK/GIT/IFO;
4. dla rigid zachowuje sylwetke przy co najmniej idle i transition state;
5. dla skinu przejdzie C1 krok po kroku, nie tylko „nie crashuje”.

### Stan prac

- Gotowa wiedza do bezpiecznej implementacji offline: **A1, B1, B2, B3**
  oraz testy kontraktowe C1.
- Gotowa wiedza do zakazania ryzykownych zmian: **A0 i sekcja wycofan**.
- Wiedza wymagajaca live proof przed uznaniem za runtime: **A2 i wszystkie
  wnioski o deformacji skinu**.
- Przeszkoda: proof live jest obecnie **missing**, nie „failed”; w tym
  badaniu nie uruchomiono Aurora/NWN i nie zmieniono konfiguracji.


## 22. Korekta stanu I9--I10 i iteracja I11 -- material/render pipeline (2026-07-21)

### Korekta: I9-P0 jest juz obecne w aktualnym worktree

Opis luki z sekcji 19 byl prawidlowy dla wczesniejszego worktree, ale nie
opisuje juz aktualnego stanu kodu. W
`crates/m2a-core/src/model_pipeline.rs` funkcja
`require_m0_direct_creature_resolver_binding_v1` jest wywolywana przez builder
kontraktu i przez verifier. Wymaga ona:

- `MODELTYPE == S` (w przeciwnym razie
  `M0-RUNTIME-CONTRACT-APPEARANCE-MODELTYPE-INVALID`);
- `RACE == m2a_m0p01` (`M0-RUNTIME-CONTRACT-APPEARANCE-RACE-MISMATCH`);
- dokladnie jednego resrefu tekstury mesha: `m2a_m0t01`
  (`M0-RUNTIME-CONTRACT-MESH-TEXTURE-RESREF-MISMATCH`).

Test `static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice`
odtwarza samospójne mutacje `MODELTYPE=P`, zlego `RACE` oraz obcego resrefu
tekstury i odrzuca je stabilnymi kodami. Uruchomione 2026-07-21 polecenie
`cargo test -p m2a-core --test model_pipeline static_meshy_m0 --quiet`
przeszlo: 4/4.

**Granica:** gate wiąże semantykę emitowanego M0 z jego wlasnym readbackiem;
nie jest dowodem kolejności override/userpatch ani cache'u klienta. Kontrola
tego ostatniego nadal wymaga osobnego proofu runtime.

### Korekta: rigid animation placeholder 0x21 jest juz emitowany

Sekcja 20 opisywala historyczny stan `0x01`. Aktualny writer w
`crates/m2a-core/src/mdl/write_binary_mdl.rs` rozpoznaje
`RigidMeshPlaceholder`, alokuje mu pelny naglowek mesha i wpisuje flagi `0x21`.
Geometria pozostaje pusta: zero vertices/faces/indexow i `startMdx == 0`.
Test M0 odczytuje te warunki dla wszystkich siedmiu state trees. Jest to
**verified offline, rigid-M0 only**; skin dummy nadal nie ma takiego witnessa,
a zachowanie przy przejsciu stanu w prawdziwym NWN pozostaje nieudowodnione.

### I11: co rzeczywiscie moze ukryc material

Material jest osobna warstwa od resolvera i geometrii. Lokalna dekompilacja
Aurory (`C:\Projects\New Folder\export\decompiled_all.c`) rozpoznaje na meshu
co najmniej `render`, `alpha`, `materialname`, `bitmap` i `texture0`; parser
MTR rozpoznaje takze `texture0..texture5`, `transparency` i `twosided`.
To dowodzi, ze te pola maja byc traktowane jako wejsciowe dane renderingu,
ale samo w sobie nie ustala pelnej semantyki NWN:EE.

Niezalezny reader xoreos odczytuje binarne `render`, transparency hint,
cztery sloty tekstur, triangle mode oraz dane vertex color. Dla mesha bez
vertices, faces lub face offset konczy odczyt geometrii; dopiero przy poprawnej
geometrii przepisuje `render` do stanu renderingu i laduje tekstury. Jest to
zgodne z diagnostycznym podzialem: najpierw geometry gate, potem material.
Xoreos jest implementacja referencyjna, nie oraclem zachowania Beamdog.

Aktualny self-contained M0 pakuje w HAK tylko trzy zasoby: MDL (2002), TGA
(3) i `appearance.2da` (2017). Nie emituje MTR ani TXI. Writer wymaga
jednoznacznego diffuse resrefu dla kazdego uzytego material slotu; M0 wiąże go
z `m2a_m0t01`. TGA writer zachowuje format wejściowy RGB8 albo RGBA8 i
readback porownuje piksele. Base mesh M0 ma `render == 1` i RGBA vertex colors
`[255,255,255,255]`.

W konsekwencji **dla kontrolowanej paczki M0 nie ma offline dowodu na
wewnetrzny, wygenerowany MTR/TXI albo alpha zero jako przyczyne niewidocznosci**.
Pozostaja jednak realne zewnetrzne zmienne: zasob o tym samym resrefie,
override/userpatch, cache, a dla przyszlych modeli takze alpha z tekstury,
controller alpha, `materialname`/MTR oraz ENVMAP/TXI. Dokumentacja
`appearance.2da` wyraznie zaznacza, ze TXI moze zmienic stosowanie ENVMAP dla
modelu z alpha; dokumentacja materialow wskazuje, ze MTR moze podawac
`texture0` i render hints.

### Decyzja implementacyjna po I11

Nie nalezy dodawac globalnego "alpha=255" ani automatycznie generowac MTR.
Najmniejsza uzasadniona nastepna zmiana, gdy wejscia MTR/TXI zostana obslugiwane,
to deterministyczny inventory kontraktu: resref, typ zasobu, `render`,
transparency hint, vertex-alpha range, format TGA/DDS oraz deklaracje MTR/TXI.
Mutacja kazdego z tych elementow ma dostac osobny test i osobny kod bledu.
Do tego czasu M0 powinien pozostac prostym MDL+TGA bez MTR/TXI.

### Uaktualniony status priorytetow

| Pozycja z sekcji 21 | Stan aktualnego worktree | Co nadal jest otwarte |
| --- | --- | --- |
| A1, resolver M0 | **zaimplementowane i zweryfikowane offline** przez 4 testy M0 | HAK/resource precedence i cache klienta |
| B3, placeholder rigid | **zaimplementowane i zweryfikowane structural readback** | runtime A/B state transition oraz skin placeholder |
| C2, material diagnostics | czesciowo: render/resref/pixel format sa w artefakcie; brak pelnego inventory MTR/TXI | implementacja tylko po wprowadzeniu takich wejsc |
| A2, runtime proof | **missing** | wymaga osobnej autoryzacji i kontrolowanej sesji Aurora -> NWN |

## 23. Pilny werdykt: Appearance_Type M0 `848`, nie `15109` (2026-07-21)

### Werdykt

**Dla aktualnego artefaktu M0 fizycznym, wlasciwym wierszem jest `848`. Nie
nalezy zmieniac samego `Appearance_Type` na `15109`.** Liczba `15109` jest
historycznym przykladem ze starszej paczki/profilu vertical-slice, a nie
globalnym wymogiem silnika ani aktualnego generatora.

Ta decyzja dotyczy dokladnie wpisu `M2A_M0_MESHY_RIGID` o rasie
`m2a_m0p01` w HAK generowanym obecnie przez Meshy2Aurora. Wiersz ma znaczenie
fizycznego indeksu w dolaczonym `appearance.2da`, wiec GIT i HAK musza
pozostac ze soba zgodne.

### Dowody offline

1. Aktualny generator (`crates/m2a-core/src/model_pipeline.rs`) zachowuje
   tylko fizyczny prefiks wierszy `0..847`, a nastepnie dopisuje wiersz M0.
   Raport appendu zwraca indeks `848`, ktory jest przekazywany bezposrednio do
   `build_binary_m0_vertical_slice_module_v1` jako `Appearance_Type` GIT.
   Kontrakt readbacku odrzuca paczke, gdy GIT nie jest rowny fizycznemu
   indeksowi dopisanego wiersza.
2. Test
   `static_meshy_m0_uses_the_toolset_visible_appearance_prefix` buduje
   wejscie z 15 105 wierszami (ogon zaczyna sie od `OS_RESERVED_848`) i
   dowodzi rezultatu: `appended_row_index == 848`, `appearance_row == 848`,
   dokladnie 849 fizycznych wierszy w wyjsciu oraz brak `OS_RESERVED_848`.
   Uruchomiony 2026-07-21 test przechodzi.
3. Dekompilacja Aurora w
   `C:\Projects\New Folder\export\decompiled_all.c` odczytuje pole GFF
   `Appearance_Type` i serializuje je jako `ushort`; komunikaty diagnostyczne
   obejmuja `Invalid Appearance` i `ModelType for appearance not found`.
   Nie znajduje sie tam stala nakazujaca indeks `15109`.
4. Niezalezny reader xoreos (`src/engines/nwn/creature.cpp`) odczytuje
   `Appearance_Type`, pobiera wiersz przez `appearance.getRow(_appearanceID)`
   i dla `MODELTYPE != P` laduje model z kolumny `RACE`. To potwierdza model
   indeksowania wiersza, lecz nie jest oraclem zachowania Aurora.

### Pochodzenie `15109` i decyzja

Wczesniejszy profil `m0-static-rigid-v5` oraz referencyjny, centralny test
vertical-slice uzywaly przykladowo `15109`. Walidator tego standardu przyjmuje
profilowo zadeklarowany, nieujemny indeks; nie ustanawia jednej stalej dla
wszystkich HAK-ow. Historyczna dokumentacja M0 v10 odnotowuje juz przejscie na
materializowany wiersz `848`.

`15109` bylby poprawny tylko wraz z innym `appearance.2da`, ktory ma fizycznie
w tym indeksie wlasciwy wiersz M0, oraz z odpowiadajacym mu GIT. Wstawienie
`15109` do obecnego GIT przy obecnym 849-wierszowym HAK-u tworzy niespojnosc;
nie jest minimalna poprawka i nie powinno zostac zaimplementowane.

### Zrodla

- Aktualna implementacja i kontrakt: `crates/m2a-core/src/model_pipeline.rs`
  (`AURORA_TOOLSET_VISIBLE_APPEARANCE_ROWS_V1`, budowa M0 i walidacja GIT).
- Aktualny test regresji:
  `crates/m2a-core/tests/model_pipeline.rs`;
  `cargo test -p m2a-core --test model_pipeline static_meshy_m0_uses_the_toolset_visible_appearance_prefix --quiet`.
- Aurora decomp: `C:\Projects\New Folder\export\decompiled_all.c`
  (odczyt/zapis `Appearance_Type`, diagnostyka appearance/modeltype).
- xoreos: `C:\Projects\Claude\xoreos\src\engines\nwn\creature.cpp`.
- Pochodzenie historyczne: `documentation/evidence/m0-appearance-visible-range-correction-2026-07-19.md`
  i `documentation/evidence/m0-v10-binary-runtime-preflight-2026-07-20.md`.
- Zewnetrzna semantyka kolumn `appearance.2da`:
  <https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da>.

## 24. Skin: inverse-bind i bind pose - granica obecnego dowodu (2026-07-21)

### Werdykt tego przebiegu

Aktualny writer skin ma **zweryfikowana spojnosc strukturalna**. Wzor q/t i
surowa kolejnosc WXYZ zostaly dodatkowo zgodne z kompilowanym binary witness,
ale nadal nie ma dowodu, ze pola inverse-bind daja poprawna deformacje w NWN.
Nie wolno oglaszac ani pelnego skinu H1, ani przejscia source inverse-bind do
MDL jako `verified`.

Nie stwierdzono tez jeszcze konkretnego buga typu "nalezy skopiowac inverse
bind matrices z GLB do MDL". M4 celowo przyjmuje gotowy
`AuroraCreatureIrV1`, a nie source-preserving GLB; M3 moze poprawnie
przebudowac geometrie, wagi i bind pose do docelowego rigu. Brakuje jednak
jawnego testu, ktory rozstrzyga te dwie mozliwosci i nie pozwala milczaco
zgubic semantyki zrodla.

### Co jest faktem

| Zrodlo | Fakt | Granica dowodu |
| --- | --- | --- |
| Aurora decomp | Parser rozpoznaje `weights`, `qbone_ref_inv`, `tbone_ref_inv` i `boneconstantindices`; zapisuje je do pol node'a pod offsetami `0x274`, `0x290`, `0x29c`, `0x2a8`. | Same nazwy i storage nie odslaniaja wzoru deformacji ani znaczenia constants. |
| xoreos | Binary NWN node ma flage skin `0x40`; implementacja po mesh wykonuje `// TODO: Skin` i omija `0x64` bajty. ASCII reader obsluguje `weights`. | xoreos nie jest parserem semantyki binarnego skinu i nie moze walidowac naszych `q/t/constants`. |
| Dokumentacja NWN / NeverBlender | Dla aktualnego NWN:EE podawany jest limit 64 bones na skinmesh node i maks. 4 bones na vertex; wielokrotne skinmeshe sa dozwolone. Tutorial NeverBlender ostrzega przed roznica orientacji, skali i pose zrodla oraz Aurora, a takze przed zawodnym eksportem animacji. | To praktyczna informacja i limit, nie specyfikacja binarnego layoutu. |
| Aktualny kod + binary witness | Ingest GLB zachowuje `IrSkin.inverse_bind_matrices`. M4 wylicza tablice MDL jako `inverse(boneWorld) * skinWorld`, emituje WXYZ i translation. Dla wszystkich 30 wierszy obu skinow `c_squirrel` odczytane raw q/t odpowiadaja temu wzorowi (q <= `2.23e-6`, t <= `4.14e-7`). | Potwierdza zapis wobec natywnego binarium, nie deformacje w Beamdog ani polityke source IBM. |

Wymuszone granice 64 aktywnych bone slots na skin segment i 4 influences na
vertex sa zgodne z powyzszym limitem; wymaganie wprowadza sie przed writerem,
nie jako ciche obciecie po failure runtime.

### Co sprawdzily testy, a czego nie

Przeszly dwa testy offline:

```text
cargo test -p m2a-core --test glb skin_animation_fixture_preserves_source_values_and_optional_ibm_semantics --quiet
cargo test -p m2a-core --test mdl_writer emitted_skin_inverse_bind_matches_worlds_rebuilt_only_from_inspection_controllers --quiet
```

Pierwszy dowodzi, ze ingest przechowuje source IBM (rowniez jego brak). Drugi
odtwarza expected q/t tylko z controllerow i hierarchii juz wyemitowanego
artefaktu. Jest to wartosciowy test layoutu, kolejnosci WXYZ i self-consistency,
ale **nie porownuje rezultatu do deformacji source GLB ani do renderera NWN**.
Nie jest zatem niezaleznym dowodem wzoru engine'u.

Writer sam raportuje otwarte odchylenia
`M4-SKIN-WXYZ-DEFORMATION-OPEN-M6`,
`M4-SKIN-CONSTANTS-MEANING-OPEN-M6` i
`M4-SKIN-VISUAL-DEFORMATION-OPEN-M6`; ten status jest zgodny z wynikiem
badania, nie nalezy go usuwac na podstawie green testu.

### Nowy binary witness dla mapowania slotow

Read-only skompilowany CEP `c_squirrel` (ten sam payload SHA-256
`fce071fcebc04d0ffce33b6ef48891af615e70dd402120554dcc963cca36f78b`,
opisany w sekcji 40) wzmacnia **strukture** aktualnego writera, lecz nie
rozstrzyga deformacji. Oba jego base skiny maja `mapCount=30`, czyli tyle,
ile nodow ma caly base tree. Tablice `qbone_ref_inv`, `tbone_ref_inv` i
`boneconstantindices` maja rowniez `used=30`. Forward map jest mapowaniem
`nodeOrdinal -> slot` z `-1` dla nodow niebedacych koscia; dla kazdego
aktywnego wpisu binary invariant jest dokladnie:

```text
forward[nodeOrdinal] = slot >= 0
inlineReverse[slot] = nodeOrdinal
```

W `sqrlfront` aktywnych jest 9 wpisow (np. `node 2 -> slot 0`, a
`inlineReverse[0] == 2`); w `sqrlback` jest ich 10 (np. `node 15 -> slot 0`,
`inlineReverse[0] == 15`). Wszystkie aktywne wpisy obu skinow zamykaja ten
round-trip. W szczegolnosci ich wlasne ordinale (`27` i `28`) maja w forward
map wartosc `-1`.

To odpowiada kontraktowi naszego `semantic_readback`: forward jest indeksowany
ordinalem noda i daje slot dla `bone_references`, a inline map jest odwrotny.
Nie trzeba zmieniac kierunku map, liczyc tablic tylko po aktywnych slotach ani
podstawiać indeksow GLB bezposrednio w miejsce ordynalow nodow. Przyszly
env-gated test P-REF ma wczytywac ten resource in-place i sprawdzac powyzszy
round-trip oraz rownosc trzech countow z `mapCount`; fixture CI pozostaje
wlasna i minimalna.

Wniosek ma granice: witness potwierdza formule q/t i WXYZ jako surowy binary
kontrakt, ale nie ujawnia skutku dla pozycji wierzcholka. Nie wolno wiec na
tej podstawie oznaczac `M4-SKIN-WXYZ-DEFORMATION-OPEN-M6` ani constants jako
zamknietych. Osobna negatywna analiza constants jest w sekcji 41.

### Minimalne nastepne bramki (bez zgadywania implementacji)

1. **Jawna polityka source IBM w M3.** Dla skin GLB z IBM konwersja musi
   deklarowac i testowac jedno z dwoch zachowan: (a) rebind do docelowego rigu
   z mierzalnym invariantem transformacji/pozycji, albo (b) stabilne
   odrzucenie jako nieobslugiwana semantyka. Niedopuszczalne jest tylko
   milczace pominiecie IBM bez raportu i testu.
2. **Drabina runtime S1--S3.** S1: jeden skinmesh, jedna kosc, weight 1.0,
   identity bind i obraz identyczny z rigid control. S2: nieidentity base pose
   bez ruchu. S3: pojedynczy kontrolowany ruch w `cpause1`. Kazdy stopien ma
   osobny, hashowany artefakt oraz proof Toolset -> NWN.
3. **Constants: zachowac zera, uszczelnic odczyt.** Nie klonowac
   `boneconstantindices` z `c_squirrel`: szeroki corpus ma tam same zera, a
   przejscie palety nie czyta tej tablicy. Zmienic P1 parser na lossless
   4-bajtowe slowa i dodac test mutacji calego slowa; nie nadawac mu jeszcze
   semantyki float/bool/index. WXYZ i wzor q/t sa juz potwierdzone wobec
   binarium, lecz pozostaja bez wizualnego proofu.

S1--S3 pozostaja `missing`: w tym przebiegu nie uruchamiano Aurory ani NWN.

### Zrodla

- Aurora decomp: `C:\Projects\New Folder\export\decompiled_all.c`, okolice
  deklaracji stringow `qbone_ref_inv` i funkcji przy polach `0x274..0x2a8`.
- xoreos (reference-only):
  `C:\Projects\Claude\xoreos\src\graphics\aurora\model_nwn.cpp`
  (`kNodeFlagHasSkin`, binary `TODO: Skin`, ASCII `readWeights`).
- Aktualny ingest i writer: `crates/m2a-core/src/glb/mod.rs`,
  `crates/m2a-core/src/mdl/write_binary_mdl.rs`,
  `crates/m2a-core/tests/glb.rs`, `crates/m2a-core/tests/mdl_writer.rs`.
- Kontrakt M3/M4 i konflikt headera: `documentation/mdl-binary-crosswalk-codex.md`.
- [Modele NWN:EE - limity skinmesh](https://nwn.wiki/spaces/NWN1/pages/38175602/Models)
  oraz [NeverBlender: import animacji](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations).

## 25. Direct-creature `meshType`: aktualny kod jest niespojny z referencja (2026-07-21)

### Diagnoza

Aktualny profil `M4DirectCreatureExtended64V1` nadal emituje `meshType = 0` w
`write_binary_mdl.rs`, a jego test `minimal_rigid_writer_roundtrips_every_locked_semantic_and_exact_eof`
utrwala `0`. Komentarz przy zapisie na offset `0x224` nazywa je
"zero/triangle-list variant".

To twierdzenie jest slabsze niz dostepne dowody i powinno zostac usuniete.

- Wlasny read-only corpus retail NWN:EE opisany w sekcji 4 ma `3` na kazdym
  mesh node modelow `c_badger`, `c_bear` i `c_Wolf`; M0 oraz obserwowany H1
  RIGID v20 takze maja `3`.
- Lokalny xoreos odczytuje bajt jako `triangleMode` i komentuje `3 - Triangle`,
  `4 - TriStrip`. To jest niezalezna implementacja formatu, nie runtime oracle.
- Dekompilacja Aurora potwierdza rozroznienie ASCII node `trimesh`, ale z
  dostepnej funkcji nie wynika enum `0/3`; nie nalezy jej nadinterpretowac.
- Materiały NeverBlender/NNW wiki nie daja odmiennego binarnego enumu. Sa
  przydatne dla hierarchy i animation workflow, nie usprawiedliwiaja `0` jako
  triangle-list.

### Wniosek implementacyjny

Jest uzasadniona, mala korekta **wyjsciowego profilu M4**, nie globalna zmiana
parsera: `M4DirectCreatureExtended64V1` powinien emitowac `3`, a komentarz
powinien mowic o native direct-creature triangle mode. Najpierw trzeba zmienic
oczekiwanie testu na `3`, potem jedna wartosc emitera i zaktualizowac celowy
hash golden artefaktu. Reader ma nadal raportowac `0` bez odrzucenia, bo
corpus nie stanowi jeszcze specyfikacji wszystkich legalnych typow mesha.

To jest **formatowa korekta profilu**, nie deklarowana naprawa deformacji H1:
obserwowany H1 v20 z `3` byl nadal zdeformowany. Poprawne zamkniecie wymaga
pozniejszego P1 A/B z jednym zmienionym polem i tym samym MOD/HAK/placementem,
po uprzednim zwiazaniu kadru runtime z fixture.

### Zrodla

- Aktualny emitter i komentarz: `crates/m2a-core/src/mdl/write_binary_mdl.rs`
  (`M4DirectCreatureExtended64V1 => 0`, zapis `base + 0x224`).
- Aktualny test utrwalajacy `0`: `crates/m2a-core/tests/mdl_writer.rs`,
  `minimal_rigid_writer_roundtrips_every_locked_semantic_and_exact_eof`.
- xoreos (reference-only):
  `C:\Projects\Claude\xoreos\src\graphics\aurora\model_nwn.cpp`,
  `triangleMode` (`3 - Triangle`, `4 - TriStrip`).
- Aurora decomp: `C:\Projects\New Folder\export\decompiled_all.c`, parser
  ASCII node `trimesh`; retail corpus i A/B H1: sekcje 4, 21 oraz evidence
  cytowane w tym dokumencie.

## 26. B2 follow-up: exact implementation boundary for native envelope (2026-07-21)

### Current-state audit

I5 pozostaje aktualne: fixed model header
`[-5,-5,-1]..[5,5,10] / radius=7` jest reference-supported i musi pozostac
nienaruszony. Audit bieżącego kodu pokazuje jednak, ze B2 nie zostal jeszcze
zaimplementowany:

1. `plan()` oblicza `model_min`, `model_max` i `model_radius` z koncowych
   pozycji po transformie parent node'a, ale zwracany `Plan` wpisuje stale
   `DIRECT_CREATURE_MODEL_BOUNDS_*` oraz `DIRECT_CREATURE_MODEL_RADIUS`.
   Obliczone wartosci nie trafiaja ani do naglowka, ani do raportu, ani do
   gate'a.
2. Mesh-level bounds/radius sa poprawnie emitowane z lokalnej geometrii
   segmentu. To nie dowodzi contained-by-model-envelope, poniewaz segmenty sa
   pozniej umieszczane przez hierarchy.
3. `MdlWriterReportV1` nie ma pola bounds; podobnie
   `M0RuntimeFixtureContractV1` zawiera resource binding, scene i
   mesh-eligibility, ale nie relacje finalnej geometrii do native envelope.
4. `MdlWriterReportV1` ma obecnie derive `Eq`, wiec dodanie surowych `f32`
   jest takze mala decyzja API/testowa, nie bezkosztowym polem raportu.

### R21 narrow check: native envelope nie odrzuca aktualnego M0

To jest dodatkowy **negatywny wynik dla konkretnego artefaktu r21**, a nie
zamkniecie B2 dla dowolnego przyszlego importu. Own-reader odczytal z
`m2a_m0p01.mdl`, SHA-256
`cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5`:

| Metryka | Model header | Rzeczywisty base mesh | Wynik |
| --- | --- | --- | --- |
| X | `[-5, 5]` | `[-0.54365003, 0.54365003]` | contained |
| Y | `[-5, 5]` | `[-0.31950998, 0.31950998]` | contained |
| Z | `[-1, 10]` | `[0, 1.892962]` | contained |
| Radius | `7` | `1.8960401` | ponizej native radius |

Base root ma identity position/orientation, a jedyny mesh child nie ma
wlasnych kontrolerow; dla tego M0 local mesh bounds sa zarazem bounds po
hierarchii. Nie ma wiec mechanizmu, przez ktory stale native bounds mialyby
odciac cala sylwetke tego artefaktu. Jest to zgodne z obserwacja malej
widocznej sylwetki w r21 Toolsecie.

**xoreos — ograniczony cross-check.** Loader NWN odczytuje model-level i
mesh-level bounds, ale `Model::createBound()` buduje wlasny box z aktualnego
drzewa nodow. To potwierdza rozdzielenie obu warstw w niezaleznej
implementacji; nie jest dowodem algorytmu cullingu Beamdog. **Aurora decomp**
rozpoznaje `radius` w parserze node'ow, lecz obecny dekompilat nie daje
wiarygodnego przypisania model-header radius do konkretnej decyzji cullingu.
Dlatego nie nalezy ani zmieniac stalego native headera, ani twierdzic, ze
`radius=7` sam dowodzi widocznosci runtime.

Minimalny test negatywny dla r21 jest read-only: odczytac header i base mesh,
uwzglednic tylko faktycznie emitowane transformacje hierarchii, a potem
sprawdzic containment na szesciu granicach oraz `meshRadius <= 7`. Przebieg
z tego wpisu przeszedl dla wszystkich osi. B2 nadal jest potrzebne jako
produkcyjny gate przed emisja dla nowych modeli, dokladnie wedlug TDD nizej.

### Wniosek implementacyjny B2

Implementacja ma dodac **obserwowalnosc i gate**, nie zmieniac headra:

```text
final emitted world positions
  -> actualWorldAabb
  -> per-axis containment in fixed native model header AABB
  -> report + M0 runtime-fixture contract
  -> reject outside-native-envelope before MDL bytes are written
```

`radius=7` pozostaje odczytywalna native wartoscia, ale nie wolno uzywac go
jako dodatkowego gate'a: dostepny corpus nie dowodzi, ze jest on geometrycznym
promieniem od zera ani jaka dokladnie ma role w cullingu NWN.

Raport musi rozdzielac `nativeHeaderBounds` od `actualWorldBounds`, zachowac
dokladny wynik per os (`minX`, `maxX`, `minY`, `maxY`, `minZ`, `maxZ`) i
`containedByNativeHeaderBounds`. Implementacja ma rowniez jawnie wybrac
serializowalna reprezentacje floatow zgodna z dotychczasowym `Eq` albo
kontrolowanie zmienic ten kontrakt; nie nalezy ukrywac tego przez zaokraglenie
wartosci w gate.

### TDD minimalnego gate'a

1. Fixture z punktem dokladnie na kazdej granicy (`x/y = -5,5`, `z = -1,10`)
   przechodzi i raportuje containment.
2. Mutanty z jednym punktem poza kazda z szesciu granic maja stabilny blad,
   np. `M4-NATIVE-ENVELOPE-OUTSIDE`, z segmentem i osia w diagnostyce.
3. Test potwierdza, ze native header pozostaje bajtowo
   `[-5,-5,-1]..[5,5,10] / 7` dla fixture contained.
4. M0 package contract serializuje oba zestawy bounds i ten sam wynik gate'a;
   readback HAK/MOD nie moze usunac tej informacji.

To jest `missing / offline implementation`; nie jest dowodem, ze NWN odrzuci
artefakt poza envelope. Culling i finalna widocznosc pozostaja osobnym
runtime proofem po kontrolowanym placement/camera packet.

### Zrodla

- Aktualny writer: `crates/m2a-core/src/mdl/write_binary_mdl.rs`
  (obliczenie world AABB/radius, stale `DIRECT_CREATURE_MODEL_*`, zapis headera).
- Aktualne typy raportu/kontraktu: `crates/m2a-core/src/mdl/writer_types.rs`
  i `crates/m2a-core/src/model_pipeline.rs`.
- Aurora decomp i xoreos/NeverBlender: sekcja 14/I5 tego dokumentu; wyniki
  zostaly ponownie sprawdzone w tym przebiegu bez live action.

## 27. Resolver zasobow: zamkniety pakiet M0 kontra nieznane otoczenie runtime (2026-07-21)

### Werdykt

**Lancuch zasobow wewnatrz jednego, wygenerowanego pakietu M0 jest
zweryfikowany offline. Kolejnosc wobec override, userpatch, innych HAK-ow i
cache klienta nie jest zweryfikowana i nie moze byc symulowana jako lokalny
"resolver NWN".**

Brak widocznosci runtime po przejsciu kontraktu M0 moze nadal wynikac z
zewnetrznej kolizji `appearance`, MDL albo TGA. Nie jest to dowod, ze aktualny
HAK ma zly wewnetrzny binding.

### Fakty

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp | Odczyt i zapis `module.ifo` operuja na `Mod_HakList` oraz wpisach `Mod_Hak`; lista jest osobnym kontraktem od legacy root field. | Jest to fakt o formacie/Toolsetowym GFF, nie dowod globalnego priorytetu loadera klienta. |
| Aktualny M0 | Binarna scena wymaga dokladnie jednego ordered HAK. Readback IFO wymaga listowego `Mod_Hak` jako GFF String; kontrakt wiaze hash MOD, HAK, `appearance`, MDL i TGA, `Appearance_Type`, `MODELTYPE=S`, `RACE=m2a_m0p01` oraz texture resref. | HAK writer wykrywa duplicate `(resref,type)` tylko w swoim archiwum. |
| xoreos | Creature loader wybiera wiersz `appearance` przez `Appearance_Type`, a dla modeltype innego niz `P` laduje `RACE`. Model loader zawiera jawny komentarz, ze modul/HAK moga nadpisac model i cache trzeba czyscic po unloadzie modulu. | To niezalezna implementacja; nie ustala kolejnosci override/HAK ani cache Beamdog. |
| NeverBlender / wiki | NeverBlender jest zrodlem workflow authoringu MDL, nie dokumentacja priorytetu HAK/override. `appearance.2da` dokumentuje `RACE` jako model resref i `S` jako pojedynczy MDL z tekstura. | Nie wolno wyprowadzac order-of-resolution z exportera. |

### Co jest juz chronione testami

Przeszly:

```text
cargo test -p m2a-core --test model_pipeline static_meshy_m0 --quiet
cargo test -p m2a-core --test binary_m0_vertical_slice_module binary_m0_vertical_slice_can_bind_a_fresh_module_area_and_hak_identity --quiet
```

Pierwszy zestaw obejmuje mutacje samospojnej paczki z blednym `MODELTYPE`,
`RACE`, texture resrefem i nieeligible meshem; drugi sprawdza pojedynczy
stringowy wpis `Mod_HakList`. To jest silny proof offline **wlasnego**
artefaktu, nie test kanalu override albo stalego cache procesu.

### Wniosek implementacyjny i runtime gate

1. Nie implementowac w Meshy2Aurora domyslnej kolejnosci resolvera ani
   "clear cache"; bez dowodu runtime bylby to falszywy oracle.
2. Dla kazdego przyszlego proof packet ma deklarowac wymagane, unikalne
   resrefy MDL/TGA/HAK/MOD oraz caly zestaw hashy. Nie rozwiązuje to globalnego
   resrefu `appearance`, ale ogranicza kolizje modelu i tekstury.
3. Live proof musi miec read-only environment fingerprint: identyfikacje
   uruchomionego MOD/HAK, hash plikow, lista HAK z IFO oraz inventory konfliktow
   zasobow `appearance`/MDL/TGA w kanalach widocznych dla sesji. Brak takiego
   packetu oznacza `missing`, nie "model rejected".
4. Swiezosc cache jest hipoteza do rozdzielenia przez kontrolowana, autoryzowana
   sesje wedlug shared Aurora standardu; nie jest upowaznieniem do modyfikacji
   INI, override ani userpatch.

### Zrodla

- Aurora decomp: `C:\Projects\New Folder\export\decompiled_all.c`,
  `Mod_HakList` / `Mod_Hak` read/write paths.
- xoreos (reference-only):
  `C:\Projects\Claude\xoreos\src\engines\nwn\creature.cpp` i
  `modelloader.cpp`.
- Aktualny kod i testy: `crates/m2a-core/src/model_pipeline.rs`,
  `crates/m2a-core/src/proof_module.rs`,
  `crates/m2a-core/tests/model_pipeline.rs`,
  `crates/m2a-core/tests/binary_m0_vertical_slice_module.rs`.
- [appearance.2da](https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da)
  oraz [Models / NeverBlender](https://nwn.wiki/spaces/NWN1/pages/38175602/Models).

## 28. Animacje: widocznosc pozy bazowej nie jest routingiem stanow gry (2026-07-21)

### Werdykt

**Nie ma podstaw, aby dopisywac kolejne aliasy animacji jako naprawe
niewidocznosci M0.** Aktualny M0 ma niezalezna, niepusta geometrie pozy
bazowej oraz siedem binarnie odczytywanych stanow z poprawnym `animroot` i
drzewem placeholderow. To jest mocny gate strukturalny emitera, ale nie jest
dowodem, ze klient NWN wybiera w danej chwili `cpause1`, `cwalk` lub inna
nazwe.

Dlatego diagnoza musi rozdzielac dwa niezalezne przypadki:

```text
brak obrazu pozy bazowej
  -> geometria / material / resource resolver / bounds / eligibility

obraz bazowy jest, a znika lub deformuje sie po zmianie stanu
  -> routing stanu / drzewo animacji / kontrolery / skinning
```

Przy pierwszym przypadku lista animacji nie jest uzasadnionym pierwszym
fixem. Przy drugim trzeba najpierw zarejestrowac rzeczywista zmiane stanu w
kliencie, a nie zgadywac semantyke nazw z eksportera.

### Co potwierdza aktualny kod i jego own-readback

| Fakt | Dowod | Granica |
| --- | --- | --- |
| Statyczne M0 generuje dokladnie siedem stanow: `cpause1`, `cappear`, `cwalk`, `crun`, `ca1slashl`, `cdamagel`, `cdead`. Kazdy ma `length=1`, `transtime=0.25`, root rowny resrefowi modelu, bez eventow i trackow. | `static_direct_creature_runtime_clips()` w `crates/m2a-core/src/model_pipeline.rs`. | To jest profil produktu dla statycznego M0, nie znana kompletna tabela stanow klienta. |
| Pozy bazowej nie tworzy zadna z tych animacji: jest emitowana jako osobne drzewo modelu z rzeczywista geometria. | `write_binary_mdl.rs` planuje base geometry niezaleznie od animation nodes; wlasny reader xoreos widzi base state przed iteracja po offsetach animacji. | Jest to struktura binarnego artefaktu i wskazanie xoreos, nie proof rendereru Beamdog. |
| Kazdy stan M0 ma root oraz dziecko `m2a_seg_1`, `content_flags=0x21`, zerowa geometrie placeholdera i poprawny root. | `static_meshy_m0` w `crates/m2a-core/tests/model_pipeline.rs`; wykonano `cargo test -p m2a-core --test model_pipeline static_meshy_m0 --quiet` — 4/4 passed. | Potwierdza wlasny writer/readback, nie stan runtime. |
| Rigid placeholder `0x21` jest dopuszczony tylko na podstawie zarejestrowanego profilu rigid. Dla skin writer celowo pozostawia odrebny dummy, bo nie ma native witness. | `AnimationNodeKind::RigidMeshPlaceholder` / `SkinMeshDummy` w `write_binary_mdl.rs`. | Nie wolno przenosic wniosku o rigid na M4 skin. |

### Aurora, xoreos i NeverBlender

- **Aurora decomp — fakt.** Parser ASCII `newanim` rozpoznaje `animlength`,
  `event`, `length`, `transtime` i `animroot`, po czym przekazuje blok do
  parsera wezlow animacji (`C:\Projects\New Folder\export\decompiled_all.c`,
  okolice `FUN_00a5d758`). To potwierdza, ze te pola sa czescia kontraktu
  modelu Toolsetu. W dekompilacji nie znaleziono literalow siedmiu nazw M0;
  nie daje ona tabeli przejsc stanow klienta NWN.
- **xoreos — niezalezna implementacja, nie oracle Beamdog.** `Model_NWN` najpierw
  buduje i dodaje stan bazowego modelu, potem dla kazdego binary animation
  offset tworzy nowy `State`, czyta nazwe, dlugosc i transition time, laduje
  node tree i dodaje `Animation` pod ta nazwa
  (`C:\Projects\Claude\xoreos\src\graphics\aurora\model_nwn.cpp`). To
  wspiera podzial base-pose/animation-state, lecz nie ustala, ktory stan wybierze
  NWN:EE ani czy brak aliasu blokuje pierwszy draw.
- **NeverBlender/wiki — workflow, nie specyfikacja runtime.** Tutorial wskazuje
  zgodnosc skali/orientacji oraz pozy poczatkowej armatury jako warunek
  sensownego retargetingu, a takze ostrzega, ze eksport NeverBlender moze
  miec niepoprawne headery animacji. Wspiera to osobny gate binary readback i
  proofu runtime, ale nie dokumentuje listy wymaganych stanow gry.
  Zrodlo: [Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations).

### Najmniejsza poprawna drabina proofu

1. **A0 — base pose:** kontrolowany frame po zaladowaniu obiektu, przed
   wymuszona obserwacja przejscia stanu. Packet wiaze MOD/HAK/model/texture,
   placement, kamere i obraz. Brak geometrii tutaj kieruje diagnoze do
   poprzedniej galezi, nie do aliasow.
2. **A1 — idle:** zarejestrowac faktycznie aktywowany stan idle i porownac
   obraz z A0. Dla statycznego M0 brak ruchu jest oczekiwany; geometria nie
   moze zniknac.
3. **A2 — znane przejscie:** po ustaleniu sposobu wywolania w kliencie,
   zarejestrowac jedno przejscie, np. idle -> ruch, z tym samym packetem
   zasobow. Dopiero wtedy nazwa konkretnego clipu moze zostac uznana za
   runtime-required.
4. **A3 — skin/motion:** dla H1 dodac przynajmniej jeden zmieniajacy sie
   kontroler i osobny proof deformacji zgodny z drabina S1--S4 z sekcji 24.

W chwili tego wpisu A0--A3 sa `missing`; nie uruchamiano Aurora, NWN ani nie
modyfikowano INI. Jedyna uzasadniona przyszla zmiana kodu pojawi sie dopiero,
gdy packet A1/A2 wskaze konkretny brakujacy warunek. Obecny kod nie powinien
ani rozszerzac listy aliasow, ani deklarowac, ze `cpause1` jest przez silnik
wybierane przy starcie.

### Zrodla

- Aktualny generator/test: `crates/m2a-core/src/model_pipeline.rs`,
  `crates/m2a-core/src/mdl/write_binary_mdl.rs`,
  `crates/m2a-core/tests/model_pipeline.rs`.
- Aurora decomp: `C:\Projects\New Folder\export\decompiled_all.c`, stringi
  `newanim`, `animlength`, `transtime`, `animroot` i `FUN_00a5d758`.
- xoreos (reference-only):
  `C:\Projects\Claude\xoreos\src\graphics\aurora\model_nwn.cpp`.
- NeverBlender workflow: [Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations).

## 29. R21 material/alpha: negatywny wynik dla aktualnego M0 (2026-07-21)

### Pytanie zawężone przez r21

Fresh r21 `TScrollBox` pokazuje mala, jasna sylwetke przy
`Appearance_Type=848`, `MODELTYPE=S` i `RACE=m2a_m0p01`. W konsekwencji
aktualnej hipotezy nie nalezy szukac w zmianie na `P`, w nowym `animroot` ani
w dopisaniu aliasow. Zbadano jedna pozostala realistyczna galez: czy aktualny
eksport MDL/TGA ukrywa ta sama geometrie przez alpha lub transparency.

### Werdykt

**Alpha/transparency nie jest przyczyna niewidocznosci aktualnego r21 M0.**
To negatywny wynik dla konkretnego, zhashowanego artefaktu; nie jest dowodem
runtime NWN ani ogolna regulą dla przyszlych importow Meshy.

| Warstwa | Odczyt aktualnego artefaktu | Wniosek |
| --- | --- | --- |
| TGA `m2a_m0t01` | SHA-256 `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`; 2048 x 2048; `pixelDepth=24`; `descriptor=0`; 12 582 956 B. | TGA jest RGB8, wiec fizycznie nie ma kanalu alpha, ktory moglby byc zerowy. |
| Base mesh `m2a_m0p01` | SHA-256 `cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5`; `render=1`; `transparency=0`; `renderHint=0`; `meshType=3`; texture0 `m2a_m0t01`. | MDL nie wlacza lokalnej transparency ani nie wylacza draw przez `render=0`. |
| Vertex colors | 2 380 kolorow; range alpha `255..255`. | Vertex alpha nie moze w tym artefakcie wyzerowac siluety. |
| Dodatkowe materialy | r21 HAK ma tylko `appearance.2da`, MDL i TGA; nie zawiera MTR ani TXI. | Wygenerowany pakiet nie wprowadza wlasnego override `texture0` lub TXI alpha/blending. |

Dokladna sylwetka ma bounds okolo `1.09 x 0.64 x 1.89 m`, wiec jej maly
rozmiar na calym widoku Area jest zgodny z r21, a nie z calkowicie
przezroczystym modelem. Szczegoly packetu i viewportu r21 sa w
`documentation/evidence/m0-direct-resolver-and-native-admission-2026-07-21.md`.

### Zrodla i granice

- **Aurora decomp — fakt:** parser rozpoznaje `texture0`, `transparencyhint`,
  `renderhint`, `render`, `alpha` i kontrolery alpha
  (`C:\Projects\New Folder\export\decompiled_all.c`, okolice stringow i
  parserow `00cc7dbe..00cc8009`). To uzasadnia badanie tych pol, ale nie
  zastępuje capture'u klienta NWN.
- **xoreos — niezalezna implementacja:** binary `readMesh()` czyta
  `render`, transparency hint i cztery resrefy tekstur przed tworzeniem danych
  mesha (`C:\Projects\Claude\xoreos\src\graphics\aurora\model_nwn.cpp`).
  Jest to cross-check layoutu, nie oracle Beamdog.
- **NeverBlender/wiki — workflow:** [Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations)
  wymaga kontroli zgodnosci eksportu/armatury i ostrzega o wadliwych
  headerach, ale nie definiuje alpha/blendingu binary MDL. Nie nalezy z niego
  wyprowadzac zmiany materialu r21.

### Minimalny, powtarzalny test negatywny

Dla r21 test ma czytac istniejacy TGA i semantic readback MDL, a nie zmieniac
obrazu na wymuszony `alpha=255`:

```text
TGA: sha256 == 079bf...ce59b && header[16..18] == [24, 0]
MDL: render == 1 && transparency == 0 && renderHint == 0
     && texture0 == "m2a_m0t01" && vertexAlpha.min == vertexAlpha.max == 255
HAK: tylko appearance(2017), m2a_m0p01(2002), m2a_m0t01(3)
```

W tym przebiegu test wykonano read-only: TGA header spelnia `[24,0]`, a
`cargo run -p m2a-core --quiet --example inspect_mdl -- <exact-m0.mdl>`
zwrocil wszystkie powyzsze wartosci. `cargo test -p m2a-core --test tga_writer
--quiet` przeszedl 7/7 i potwierdza, ze writer zachowuje deklarowany RGB8/RGBA8
oraz semantic readback pikseli.

### Decyzja implementacyjna

Nie dodawac globalnego `alpha=255`, konwersji RGB do RGBA ani MTR/TXI jako
"fixu" M0. Jedyna uzasadniona przyszla poprawa offline to diagnostyka
materialu w runtime-fixture contract: format/depth/descriptor TGA, zakres
vertex-alpha, `render`, transparency/renderHint oraz inventory MTR/TXI. Ma to
byc obserwowalnosc z mutacyjnym testem, nie blokada legalnych przyszlych
 materialow z alpha. Pozostaje jedynie kontrolowany packet runtime **r27**;
 historyczne capture'y r21/r26 nie rozstrzygają skutku `type=5`.

## 30. R21 classification, root i animacje: negatywny wynik dla bledu eksportu (2026-07-21)

### Werdykt

**Nie ma obecnie dowodu na blad klasyfikacji modelu, korzenia ani wymaganej
animacji w statycznym M0 r21. Nie zmieniac `classification=4`,
`setsupermodel=NULL`, root node ani listy siedmiu stanow jako probnego fixu.**

Jest to negatywny wynik dla dokladnego pliku
`m2a_m0p01.mdl`, SHA-256
`cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5`.
Nie jest to dowod, ze klient NWN wybiera prawidlowy stan po wejsciu do Area:
taki packet runtime nadal nie istnieje. Jest za to wystarczajacy, aby nie
wprowadzac slepej mutacji eksportera w galezi, ktora juz renderuje w r21
Toolsecie.

### Dokladny odczyt artefaktu r21

| Warstwa | Odczyt own-reader | Znaczenie |
| --- | --- | --- |
| Header modelu | `geometryType=2`, `classification=4`, `fog=1`, `supermodelName="NULL"`, siedem wskaznikow animacji. | To jest kompletny profil domyslny wymagany przez semantic readback emitera; `4` nie jest przypadkowa wartoscia zastapcza. |
| Root pozy bazowej | `m2a_m0p01`, numer `0`, parent `null`, flags `0x01`; tylko position `[0,0,0]` i orientation `[0,0,0,1]`; jedno dziecko. | Korzen jest dummy/header node, a nie pustym meshem. |
| Geometria bazowa | Dziecko `m2a_seg_1`, flags `0x21`, z rzeczywistym meshem 2 380 vertexow. | `0x01 | 0x20`: header + mesh; to jest galez, ktora dala mala jasna sylwetke w r21 Toolsecie. |
| Stany | Dokladnie `cappear`, `cpause1`, `cwalk`, `crun`, `ca1slashl`, `cdamagel`, `cdead`; kazdy `length=1`, `animroot=m2a_m0p01`, dwa nody: root `0x01` i pusty rigid placeholder `m2a_seg_1` `0x21`. | Stany nie zawieraja geometrii bazowej ani kontrolerow, wiec nie sa kandydatem na usuniecie pierwszego draw pozy bazowej. |

### Triangulacja z trzech zrodel

- **Aurora decomp — fakt.** Handler ASCII `classification` w
  `FUN_00a5dc44` mapuje literal `character` na bajt `4` w naglowku modelu
  (`C:\Projects\New Folder\export\decompiled_all.c`, okolice
  `FUN_00a5dc44`). Zatem aktualne `classification=4` odpowiada
  `classification character`, a nie klasyfikacji `other`/`tile`.
  Ten sam parser rozpoznaje `newmodel`, `newanim`, `animroot`, `parent` i
  `setsupermodel`; nie zawiera tabeli, ktora wymuszalaby dla M0 kolejny alias
  albo niepusty track animacji.
- **xoreos — niezalezna implementacja, nie oracle Beamdog.**
  `Model_NWN::loadBinary()` odczytuje classification i fog, tworzy i dodaje
  base state z root node *przed* petla po offsetach animacji. Dopiero potem
  czyta poszczegolne clipy. Jego flagi oznaczaja `0x01` jako node header i
  `0x20` jako mesh, a `readAnimBinary()` zachowuje `animroot`, dlugosc i
  transition. To potwierdza layout oraz rozdzielenie pozy bazowej od clipow,
  ale nie opisuje wyboru stanu przez silnik Beamdog.
- **NeverBlender/wiki — workflow authoringu.**
  Lista diagnostyczna MDL wymaga poprawnego parentingu i dummy node z
  parent `null`; wskazuje tez poprawny `setsupermodel`. Aktualne drzewo r21
  spelnia te warunki. Tutorial animacji traktuje base/origin pose oraz
  poprawne headery jako oddzielny problem retargetingu — nie twierdzi, ze
  siedem konkretnych nazw jest konieczne do pierwszego renderu. Zrodla:
  [MDL](https://nwn.wiki/spaces/NWN1/pages/38175669/MDL?src=contextnavpagetreemode)
  i [Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations).

### Minimalny test i decyzja

Wlasciwy test jest juz mutacyjnie chroniony, wiec nie wymaga zmiany writera:

```text
cargo test -p m2a-core --test model_pipeline static_meshy_m0 --quiet
```

W tym przebiegu: **4/4 passed**. `semantic_readback.rs` odrzuca naglowek,
ktory nie ma `classification=4`, `fog=1`, `childModelCount=0`,
`animationScale=1` lub `supermodel="NULL"`; test M0 wymaga siedmiu
`animroot` rownych resrefowi modelu, po jednym root i placeholderze `0x21` na
stan. Odczyt read-only zhashowanego pliku r21 potwierdzil ponadto base root
`0x01` i jego jedyne mesh-child `0x21`.

Nie ma minimalnej poprawki eksportu: poprawka bylaby nieuzasadniona regresja.
Najmniejszy kolejny test **dopiero w runtime** to A0 z sekcji 28: kontrolowany
frame po zaladowaniu obiektu, z tym samym hashem MDL/TGA/HAK/MOD i kamera na
fixture. Jezeli A0 ma geometrie, ten watek jest definitywnie zamkniety dla
widocznosci poczatkowej; jezeli A0 jest pusty mimo r21 Toolsetu, porownac
wybor zasobu/caching przed zmiana classification albo animacji. W tym cyklu
nie uruchamiano Aurora ani NWN i nie modyfikowano zadnego INI.

### Zrodla

- Aktualny reader/writer/test: `crates/m2a-core/src/mdl/semantic_readback.rs`,
  `crates/m2a-core/src/mdl/write_binary_mdl.rs`,
  `crates/m2a-core/tests/model_pipeline.rs`.
- Aurora decomp: `C:\Projects\New Folder\export\decompiled_all.c`,
  `FUN_00a5dc44`, `parent`, `newmodel`, `newanim`, `animroot`,
  `setsupermodel`.
- xoreos (reference-only):
  `C:\Projects\Claude\xoreos\src\graphics\aurora\model_nwn.cpp`.
- NeverBlender/wiki: [MDL](https://nwn.wiki/spaces/NWN1/pages/38175669/MDL?src=contextnavpagetreemode),
  [Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations).

## 31. R21 winding i normalne: integralnosc potwierdzona, exterior-culling nadal wymaga runtime (2026-07-21)

### Werdykt

**Aktualny r21 M0 nie ma wykrywalnego rozjazdu indeksow, face-plane ani
normalnych, ktory uzasadnialby slepe odwracanie windingu.** Nie dodawac
drugiego reverse, nie wymuszac materialu two-sided i nie modyfikowac
normalnych jako "fixu" niewidocznosci.

Nie jest to rownoznaczne z dowodem, ze kazda powierzchnia jest widoczna z
zewnetrznej strony w rendererze NWN. Bez kontrolowanego kadru runtime nie ma
niezaleznego oracle'a "outside" dla dowolnej, potencjalnie otwartej siatki.
Rozroznienie jest istotne: **uszkodzony binary winding jest odrzucony**;
**globalny backface-culling klienta pozostaje nieudowodniony**.

### Exact r21 readback

Dla `m2a_m0p01.mdl`, SHA-256
`cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5`,
read-only analiza 1 569 faces daje:

| Kontrola | Wynik |
| --- | --- |
| Degenerate triangles | `0` |
| `rawIndices` kontra kolejne `face.vertexIndices` | `0` roznic |
| Dot(geometric cross-product, zapisany face normal) | min `0.999999999991299`, max `1.0000000000000002`; zadnego wyniku `< 0.999` |
| Dot(geometric cross-product, srednia normalnych wierzcholkow) | min `0.47547674934705042`, max `0.99999999998819622`; `0` wartosci ujemnych |
| Render profile | `render=1`, `transparency=0`, `meshType=3` |

Pierwsza relacja jest gate'em binary integrity: face normal jest liczony z
tej samej kolejnosci trzech vertexow, ktora zapisywana jest do face record i
raw index stream. Druga relacja nie wymaga idealnie plaskiej powierzchni,
dlatego ma nizsze minimum, lecz wyklucza odwrocone normalne lokalnie.

### Lancuch konwersji i trzy zrodla

- **Aktualny generator — fakt z kodu/testu.** Raport r21 ma determinant basis
  `-1` i polityke `orientationParity=NEGATIVE_FOR_POSITIVE_SOURCE_AND_RIG_PARITY`.
  `profile_a.rs` w takim przypadku emituje indeksy `[0,2,1]` dokladnie raz i
  transformuje normalna przez inverse-transpose. Test
  `basis_scale_alignment_winding_normal_tangent_and_uv_are_exact` sprawdza
  wynik `[0,2,1]` oraz dodatni dot cross-product/normal; przeszedl 1/1.
  Powtorzony `static_meshy_m0` przeszedl 4/4.
- **Aurora decomp — fakt o zakresie pol.** Parser rozpoznaje `twosided` w
  konfiguracji shaderowej i `twosidedtex` w node emittera
  (`C:\Projects\New Folder\export\decompiled_all.c`, okolice
  `s_twosided`, `s_twosidedtex`). Nie jest to dowod ogolnej flagi naprawczej
  dla direct-creature trimesh; przypisanie jej do M0 byloby spekulacja.
- **xoreos — niezalezna implementacja.** Binary reader czyta face normals i
  trzy indeksy face record, a nastepnie buduje jego vertex buffer w tej
  kolejnosci; smooth normals wyprowadza z face normals, nie z samego raw
  vertex-normal streamu. To czyni potwierdzona zgodnosc face records r21
  szczegolnie wazna. Nie jest to wyrok o ustawieniach backface culling
  Beamdog.
- **NeverBlender/wiki — granica workflow.** Dokumentacja MDL zaleca
  diagnozowac prawidlowy model binarny i nie traktowac eksportu jako proofu
  gry; nie udostepnia tabeli wartosci binary flagi, ktora moglaby legalnie
  zastapic kontrolowana probe widoku z obu stron.
  Zrodlo: [MDL](https://nwn.wiki/spaces/NWN1/pages/38175669/MDL?src=contextnavpagetreemode).

### Minimalny test i kolejny proof

Zachowac test konwersji powyzej i dodac w przyszlosci test diagnostyczny
readback M0: dla kazdego face ma sprawdzac niezerowy cross-product,
zgodnosc face-plane z indeksami oraz identycznosc face/raw index streamu.
Nie powinien wymagac, aby kazdy dot z vertex normals byl bliski `1`, bo
smooth shading legalnie rozni sie od pojedynczej normalnej face.

Jedyny rozstrzygajacy test exterior-culling to kontrolowany runtime A/B po
packetcie A0: dwa hashowane warianty rozniace sie **wylacznie** kolejnoscia
trzech indeksow i face normal sign, ten sam MOD/HAK/appearance/placement,
kamera z obu stron. Przed takim packetem nie zmieniac windingu M0.
W tym przebiegu nie uruchamiano Aurora ani NWN i nie modyfikowano INI.

### Zrodla

- Aktualny writer/converter/testy: `crates/m2a-core/src/profile_a.rs`,
  `crates/m2a-core/src/mdl/write_binary_mdl.rs`,
  `crates/m2a-core/tests/profile_a.rs`,
  `crates/m2a-core/tests/model_pipeline.rs`.
- Aurora decomp: `C:\Projects\New Folder\export\decompiled_all.c`,
  `s_twosided`, `s_twosidedtex` i odpowiadajace handlery parsera.
- xoreos (reference-only):
  `C:\Projects\Claude\xoreos\src\graphics\aurora\model_nwn.cpp`.
- NeverBlender/wiki: [MDL](https://nwn.wiki/spaces/NWN1/pages/38175669/MDL?src=contextnavpagetreemode).

## 31a. P1: gate renderability M0 nie sprawdza jeszcze zgodności face-plane (2026-07-21)

### Werdykt

**Nie jest to błąd aktualnego r21 ani uzasadnienie zmiany windingu.** Writer
odrzuca trójkąt zdegenerowany i wylicza `face.normal` oraz `face.distance` z
tej samej kolejności indeksów, którą serializuje do face record i raw streamu.
R21 przeszedł dodatkowy read-only audit wszystkich 1 569 face records (§31).

Publiczny gate `inspect_m0_runtime_mesh_eligibility_v1()` sprawdza obecnie
`render`, `meshType`, vertices, obecność faces oraz zgodność face/raw-index
streamu, ale nie weryfikuje, że zapisany face-plane nadal odpowiada tym
indeksom. Artefakt z odwróconym jednym `face.normal` albo zmienionym
`face.distance` pozostałby zatem *eligible* dla tego diagnostycznego API,
jeśli pozostałe pola byłyby spójne. To P1 hardening readbacku przeciw
uszkodzeniu artefaktu lub przyszłemu drugiemu emiterowi, nie dowód problemu
renderera NWN.

### Dowody i granice

| Źródło | Fakt | Granica |
| --- | --- | --- |
| Aktualny `write_binary_mdl.rs` | `checked_face_plane()` liczy unit normal i plane distance z indeksowanego trójkąta; wywołanie przed zapisem odrzuca degenerate/non-finite geometry. | Gwarantuje dzisiejszy output tego writera, nie późniejszą mutację binarnego MDL. |
| Aktualny `inspect_m0_runtime_mesh_eligibility_v1()` | Jego literalna reguła kończy się na `faceIndicesMatchSingleRawIndexStream`; nie porównuje `FaceReport.normal`/`distance` z pozycjami. | Jest to luka diagnostyczna, nie brak binary parsera: parser już odczytuje oba pola. |
| Aurora decomp `FUN_00a62b0c` | Widoczny native mesh-build wymaga positions, UV0, vertex normals oraz `meshType==3`, po czym korzysta z trójek indeksów. | Ten trace nie daje podstaw, by nazwać face-plane osobnym Beamdog draw predicate. |
| xoreos `ModelNode_NWN_Binary::readMesh()` | Czyta face normals i indeksy; buduje wygładzoną normalną wierzchołka przez agregację normalnych face o tym samym smooth group. | Potwierdza wartość spójności dla niezależnego readera, nie jest oracle'em Beamdog. |
| NeverBlender Manual 2.0 | Trimesh jest eksportowany jako triangulowana geometria. Manual nie opisuje skompilowanego binary face-plane ani flagi naprawiającej culling. | Nie uzasadnia zmiany two-sided ani odwracania M0. |

### Minimalny test-first, jeśli hardening zostanie autoryzowany

Rozszerzyć **wyłącznie** M0 eligibility readback o obliczenie face-plane z
`mesh.vertices[face.vertex_indices]` i porównanie z pola `FaceReport`, z tą
samą tolerancją f32, której używa obecny semantic readback. Test ma:

1. zachować baseline `eligible=true`;
2. zmutować wyłącznie semantyczne pole `face.normal` na jego negację,
   zachowując vertices, face indices i raw stream, i wymagać `eligible=false`;
3. zmutować wyłącznie `face.distance` i wymagać `eligible=false`.

W tym cyklu przeszły istniejące, wąskie gates: `m0_runtime_mesh_eligibility_requires_triangle_topology_and_nonempty_faces` oraz `basis_scale_alignment_winding_normal_tangent_and_uv_are_exact` (po `1 passed`). Nie tworzyć nowego M0, HAK-a ani packetu NWN dla tej P1 korekty; aktualny r21 wymaga nadal wyłącznie tego samego, kontrolowanego proofu runtime.

Aurora, NWN i INI nie były uruchamiane ani modyfikowane w tym cyklu.

## 32. R21 `appearance.2da`: poprawny binding M0, lecz skrocony zasob jest tylko vertical-slice (2026-07-21)

### Werdykt

**Nie znaleziono bledu indeksu, `MODELTYPE`, `RACE`, klasyfikacji MDL,
root-node, animacji ani alpha, ktory wyjasnialby niewidocznosc obecnego
statycznego M0.** Wlasciwy dla r21 jest nadal fizyczny wiersz `848`: jest
obecny w HAK, ma `MODELTYPE=S` i `RACE=m2a_m0p01`; Toolset rzeczywiscie
renderuje ten M0.

Jest natomiast osobny, materialny fakt wdrozeniowy: HAK r21 zawiera **caly
zasob** o nazwie `appearance`, ale celowo skrocony do pierwszych 848 wierszy
wejsciowych plus wiersz M0. Nie jest to scalanie wiersza z retail
`appearance.2da`. Taki HAK jest poprawnym, ograniczonym proof packetem M0,
ale nie moze byc bezwarunkowo dystrybuowany jako ogolny HAK runtime dla
modulu korzystajacego z appearance IDs powyzej 848.

### Fakty z artefaktu r21 i generatora

| Kontrola | Wynik | Znaczenie |
| --- | --- | --- |
| Wbudowany `appearance.2da` | SHA-256 `04f7933cd6cb67ae4d065a18c6bed9e18b8688bb97bd4cae5f9072e1d2792acb`, 388 194 B, 35 kolumn, 849 fizycznych wierszy | To kompletna skladniowo tabela dla tego pakietu, nie plik z samym jednym wierszem. |
| Zachowany prefiks | input ma 848 wierszy / 388 005 B / SHA-256 `83a043adaff35f6ce116a7d6c47078633fd6ff000c6db7a2e70d6ca4fabcd5af`; report podaje `sourcePrefixPreserved=true` | Wiersze 0..847 nie zostaly zreserializowane ani przesuniete. |
| Wiersz M0 | physical `848`, `LABEL=M2A_M0_MESHY_RIGID`, `MODELTYPE=S`, `RACE=m2a_m0p01`, `MOVERATE=NORM`, `BLOODCOLR=R`, `SIZECATEGORY=4` | GIT `Appearance_Type=848` wskazuje wiersz istniejacy i bezposrednio rozwiazuje jeden model. |
| Celowe obciecie | `build_meshy_m0_static_rigid_package_v1` wywoluje `retain_two_da_row_prefix_v1(..., 848, ...)`, a nastepnie dopisuje M0 | To swiadoma polityka Toolsetu, nie przypadkowe uszkodzenie emitera. |
| Lokalny retail reference | read-only `base_2da.bif` ma 15 100 fizycznych wierszy `appearance` | Skrocony HAK nie jest pelnym retail replacement table. |

Test `static_meshy_m0_uses_the_toolset_visible_appearance_prefix` przeszedl
`1/1`: z inputu o 15 105 wierszach usuwa tail od 848, wstawia M0 dokladnie na
physical 848 i readback widzi 849 wierszy. Jest to celowy negatywny test
przeciwko przypadkowemu powrotowi do 15109/15100 w lane Toolsetu.

### Co mowi resolver i czego nie mowi

- **xoreos, niezalezna implementacja.** `TwoDARegistry::load2DA` pobiera jeden
  stream `ResMan.getResource(name, kFileType2DA)` i tworzy z niego jeden
  `TwoDAFile`; nie wykonuje merge wierszy. `Creature::loadModel` pobiera
  physical row `_appearanceID`, a gdy `MODELTYPE != P`, laduje model o
  resrefie z `RACE`. To potwierdza kontrakt M0 `848 -> S -> m2a_m0p01`, ale
  nie jest proofem runtime Beamdog. Zrodla:
  [2DA registry](https://github.com/xoreos/xoreos/blob/master/src/aurora/2dareg.cpp),
  [NWN creature loader](https://github.com/xoreos/xoreos/blob/master/src/engines/nwn/creature.cpp)
  i lokalne kopie reference-only.
- **Aurora decomp.** Odczytuje i serializuje uporzadkowane `Mod_HakList` /
  `Mod_Hak` (`decompiled_all.c`, okolice 262516 oraz 264464). Nie znaleziono
  w niej dowodu, ze klient Beamdog scala `appearance.2da`; decomp jest tu
  faktem o module/Toolsecie, nie o priorytecie klienta.
- **Dokumentacja spolecznosci.** Opis override wskazuje, ze HAK z zasobem o
  tej samej nazwie zastepuje nizszy zasob; poradnik scalania HAKow zaleca
  reczne scalenie kolidujacych 2DA. To wspiera hipoteze pelnego replacementu,
  lecz nie zastepuje kontrolowanego capture NWN EE.
  Zrodla: [Override](https://nwn.fandom.com/wiki/Override),
  [Combining HAK Packs](https://neverwintervault.org/article/tutorial/combining-hak-packs),
  [2da Files](https://nwn.wiki/spaces/NWN1/pages/38174875/2da%2BFiles).

### Dokladny zakres diagnozy

Dla **jednego aktualnego M0** skrocenie nie tworzy lookup failure: indeks 848
jest w zakresie 0..848, jego `RACE` istnieje jako MDL w tym samym HAK, a
Toolset zobaczyl obiekt. Dlatego nie wolno zmieniac teraz `classification=4`,
roota, aliasow animacji, alpha, windingu ani numeru appearance jako "fixu".

Skrocenie jest mimo to prawdziwym ryzykiem dla **innego** contentu: gdy w
module, template albo przywolanym zasobie pojawi sie `Appearance_Type > 848`,
wybrany zasob HAK nie zawiera jego wiersza. To moze uszkodzic ten obiekt albo
jego dane appearance; nie jest dowodem o samym M0.

### Minimalny gate do implementacji

Przed wydaniem ogolnego runtime HAK wprowadzic jawny enum strategii zamiast
ukrytej polityki `retain...(..., 848)`:

1. `toolset_visible_slice`: zachowuje wiersze 0..847 i dodaje M0 na 848;
   manifest musi nosic `scope=isolated_toolset_vertical_slice` oraz zakaz
   `general_runtime_distribution`.
2. `full_runtime_append`: zachowuje caly wskazany input `appearance.2da`,
   dopisuje M0 na jego rzeczywistym physical index i zapisuje identyczny index
   do GIT/UTC. Nie moze obiecywac wyboru tego wiersza przez palette Toolsetu.

Minimalny test negatywny: input z 15 100 wierszami i strategia
`toolset_visible_slice` musi byc oznaczona scoped oraz odrzucic manifest
ogolnego runtime; `full_runtime_append` ma zachowac 15 100 prefixowych
wierszy i powiazac GIT/UTC z 15 100. Numer nie moze byc stala produktu --
wynika z physical row count wybranego inputu.

To jest **policy/gate do przyszlej implementacji**, nie zgoda na zmiane
obecnego r21. W tym przebiegu nie uruchamiano Aurora ani NWN i nie
modyfikowano INI. Runtime A0 nadal musi zarejestrowac hash faktycznie
zaladowanego HAK/2DA oraz obraz M0 w kliencie.

## 33. R21 naglowek MDL: geometry type, fog i brak supermodelu sa zgodne (2026-07-21)

### Werdykt

**Naglowek aktualnego, statycznego M0 nie zawiera pozostalego realistycznego
fixu niewidocznosci.** `geometryType=2`, `classification=4`, `fog=1`,
`animationScale=1.0` i `supermodelName="NULL"` sa spojną konfiguracja
samodzielnego direct creature. Nie zmieniac tych bajtow na `tile`, `other`,
`ignorefog`, niepusty supermodel ani inny scale bez odrebnego runtime A/B.

### Fakty

| Pole naglowka modelu | R21 | Dowod i wniosek |
| --- | --- | --- |
| `core+0x6c` geometry type | `2` | Lokalna specyfikacja binary MDL w xoreos-docs oznacza `0x02` jako model geometry header; `0x05` jest geometry headerem animacji. Own semantic readback M2A wymaga `2`. |
| `core+0x72` classification | `4` | Aurora `FUN_00a5dc44` mapuje ASCII `classification character` na bajt `4`. Referencyjny direct creature `c_squirrel` ma `classification Character`. |
| `core+0x73` fog | `1` | Aurora rejestruje osobny handler `ignorefog`; `FUN_00a5dcd8` ustawia ten bajt na `0` tylko po takiej dyrektywie. `1` jest wiec zwyklym wariantem fog-enabled, nie sygnalem ukrywania modelu. |
| `core+0xa4` animation scale | `1.0` | W xoreos skala jest mnozona tylko dla animacji odziedziczonych z supermodelu; wlasna animacja ma skale `1.0`. M0 ma wszystkie siedem rozpatrywanych stanow lokalnie oraz `NULL`, wiec ta wartosc nie moze usunac base pose. |
| `core+0xa8` supermodel | `NULL` | Aurora handler `setsupermodel` zapisuje wartosc w tym polu i rozwiazuje referencje; xoreos nie laduje zadnego parent modelu, gdy nazwa jest pusta albo rowna `NULL`. Ten sam sentinel jest w niezmodyfikowanym `c_squirrel`. |

`c_squirrel` jest jedynie witness reference-only, nie template do kopiowania:
jego `setanimationscale 0.5` ma sens dla jego wlasnego authoringu, ale nie
jest argumentem za zmiana samodzielnego M0. Jego `setsupermodel c_squirrel
NULL` i `classification Character` potwierdzaja natomiast semantyke dwoch
pol naglowka M0. NeverBlender/wiki traktuje supermodel i root hierarchy jako
kontrakt eksportu/animacji; nie podaje alternatywnej wartosci, ktora bylaby
fixem pierwszego draw callu. Zrodla: [MDL](https://nwn.wiki/spaces/NWN1/pages/38175669/MDL?src=contextnavpagetreemode),
[Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations).

### Test minimalny i granica dowodu

Istniejacy own-readback wymaga tych wartosci w `semantic_readback.rs`, a
`cargo test -p m2a-core --test model_pipeline static_meshy_m0 --quiet`
przeszedl `4/4`. To jest wystarczajacy negatywny wynik wobec propozycji
mutowania naglowka "w ciemno".

Przy przyszlym refaktorze gate ma dostac cztery celowe mutacje outputu:
`geometryType != 2`, `classification != 4`, `fog = 0` i `supermodel != NULL`.
Kazda musi byc rozpoznana osobnym, nazwanym semantic diff zanim artefakt
wejdzie do HAK. Nie jest to runtime renderer oracle: jedyny test, czy fog lub
supermodel zmieniaja obraz w konkretnym kliencie Beamdog, to hashowany A/B z
tymi samymi MDL/MDX/tekstura/placement poza jednym polem naglowka.

W tym przebiegu wykonano tylko odczyt dekompilacji, xoreos, reference-only
`c_squirrel` i wlasnych artefaktow/testow. Aurora, NWN i INI pozostaly
nietykane.

## 34. M0 puste aliasy animacji: brak offline przeslanek, ze ukrywaja base mesh (2026-07-21)

### Pytanie i werdykt

Pytanie dotyczylo jedynego nadal realistycznego eksportowego wariantu: Toolset
moze narysowac base pose, ale klient po wyborze `cpause1` moglby potraktowac
pusty alias animacji jako nowa, pusta siatke i przez to usunac obraz M0.

**Werdykt offline jest negatywny: nie ma dowodu, ze aktualny M0 wymaga
dopisywania identity trackow albo geometrii do aliasow. Nie zmieniac writer'a
w ciemno.** Pozostaje jedna granica: xoreos i parser Toolsetu nie sa
rendererem Beamdog, wiec tylko kontrolowany runtime A/B moze rozstrzygnac
wariant "brak kontrolerow w `cpause1`".

### Fakty rozdzielone wedlug zrodla

| Zrodlo | Fakt | Znaczenie dla M0 |
| --- | --- | --- |
| R21 M0 + own readback | Kazdy z 7 clipow ma dokladnie dwa węzly: root `m2a_m0p01` (`0x01`) i dziecko `m2a_seg_1` (`0x21`). Dziecko ma `vertexCount=0`, bez faces/indexes/tekstur/MDX i bez kontrolerow. | Alias zachowuje nazwe, numer i rodzine węzla bazowej siatki; nie jest zastepowany obcym dummy. |
| Wlasny writer/readback | `RigidMeshPlaceholder` ma naglowek mesh (`MESH_HEADER_SIZE`), flage `0x21`, zero geometrii i zero trackow. Semantic readback odrzuca kazde odstepstwo od tego kontraktu. | Jest to intencjonalny profil dla rigid creature, sprawdzany po zapisie, a nie niekontrolowany pusty wpis. |
| Referencyjny ASCII `c_squirrel` | `cpause1` i `cappear` maja po 28 wezlow: `6 dummy`, `21 trimesh`, `1 danglymesh`; obejmuja nie-skinowa topologie base modelu, w tym węzly `trimesh`, ale nie maja ponownie payloadu verts/faces. Posiadaja jednak rzeczywiste kontrolery ruchu. | Jest silnym swiadkiem **topologii/nazw/rodziny wezla**, lecz nie swiadkiem braku kontrolerow. Nie wolno rozszerzac go do wniosku o Beamdog bez A/B. |
| xoreos, `ModelNode_NWN_Binary::readMesh()` | Gdy `vertexCount==0`, `facesCount==0` lub `facesOffset==0`, loader wraca przed zbudowaniem render-data. | Pusty alias nie tworzy alternatywnego draw payloadu, ktory moglby zastapic bazowa siatke w tym rendererze. |
| xoreos, `ModelNode_NWN_Binary::load()` + `Animation::update()` | Brak position/orientation controllers dziedziczy transform base state; podczas update tylko niepuste listy frames zapisuja transform docelowego wezla. | W xoreos obecne aliasy M0 nie modyfikuja transformu renderowanej siatki bazowej. |
| Dekompilacja Aurory | Parser rozroznia `node trimesh` od `dummy` oraz osobno akceptuje `positionkey`, `orientationkey` i `alphakey`. | Potwierdza, ze node-kind i controllery sa niezaleznymi elementami kontraktu tekstowego; nie dostarcza dowodu, ze kontroler jest wymagany do renderu statycznego aliasu. |
| NeverBlender/wiki | Zalecenie importu animacji zaczyna od zgodnego base modelu/armature i opisuje dopasowanie nazw/originow. Nie ustanawia wymogu identity key dla kazdego wezla ani dla bezruchowego modelu. | Dokumentacja authoringu nie uzasadnia dopisania sztucznych trackow do M0. [Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations) |

`c_squirrel` pozostaje materialem reference-only. W szczegolnosci jego 22
deklaracje controllerow na clip dowodza ruchu, nie sa payloadem do kopiowania
ani normatywnym wymogiem dla statycznego M0.

### Minimalny test rozstrzygajacy -- dopiero w runtime

Nie nalezy implementowac tej mutacji jako poprawki. Jezeli powstanie
kontrolowany packet NWN, wykonac jedynie nastepujacy A/B:

1. **A:** aktualny M0 (identyczny HAK/modul/appearance/tekstura/placement),
   z bez-trackowym `cpause1` i obecnym `0x21` placeholderem.
2. **B:** identyczny packet, z jedyna semantyczna roznica: w root
   `m2a_m0p01` clipu `cpause1` dwa jawne identity controller streams
   (`position=(0,0,0)` oraz orientation identity) na klatce poczatkowej i
   koncowej. Child `m2a_seg_1` pozostaje tym samym zero-geometry `0x21`.
3. Zarejestrowac hash obu MDL/HAK/modulow, potwierdzic, ze klient wybral
   `cpause1`, oraz zrobic porownywalne capture po ustabilizowaniu idle.

Interpretacja jest jednoznaczna: A i B widoczne tak samo odrzuca hipoteze;
widoczne wylacznie B otwiera osobny bug "controllerless static alias"; brak
obrazu w obu kieruje dochodzenie poza ten wariant. Test B nie jest jeszcze
zaimplementowany, aby nie wprowadzic niepotwierdzonej semantyki do produktu.

### Biezacy gate i granice

Wykonano offline:

```text
cargo test -p m2a-core --test model_pipeline static_meshy_m0 --quiet
# 4 passed; obejmuje readback siedmiu aliasow, ich root/name/part number,
# flage 0x21 oraz zero vertices/faces/indexes/textures/MDX.
```

To zamyka blad serializacji placeholdera na obecnym poziomie dowodu. Nie
zamyka renderu NWN: nie wykonano Toolsetu ani klienta NWN i nie zmieniono INI.

## 35. M4/H1 `meshType=0`: potwierdzona rozbieznosc emitera, nie problem M0 (2026-07-21)

### Diagnoza

**To jest konkretny blad kontraktu obecnego writer'a dla
`M4DirectCreatureExtended64V1`: zapisuje `0` do mesh `+0x224`, mimo ze emituje
trojkatne face records i raw `u16` triangle indices. Dla tego profilu wartosc
ma byc `3`.** Statyczny M0 nie jest objety bledem -- jego osobny profil juz
emituje `3` i nie nalezy go zmieniac.

Dokladne miejsce: `crates/m2a-core/src/mdl/write_binary_mdl.rs`, funkcja
`plan()`, match `M4DirectCreatureExtended64V1 => 0`. Ten sam selector zasila
zarowno rigid segment M4, jak i segment skin `extended64`; wlasciwa poprawka
jest wiec jedna zmiana selektora, nie mutacja wybranych wygenerowanych MDL.

### Ponownie sprawdzone dowody

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp, `FUN_00a5e54c` | Przekazuje wartosc z mesh `+0x224` jako argument draw path `FUN_00a78128`. | Utracone typy dekompilatu nie nadaja publicznej nazwy enumowi, ale pole nie jest ignorowanym paddingiem. |
| Aurora decomp, `FUN_00a62954` | Po budowie listy z trojek `u16` zapisuje `3` do `+0x224`; kolejna galez sprawdza `+0x224 == 3` obok wymaganych offsets/counts. | Jest to dowod native dla trojkatnej sciezki, nie capture Beamdog wygenerowanego M4. |
| xoreos binary reader | Ten bajt jest komentowany jako `triangleMode`: `3` Triangle, `4` TriStrip. | Niezalezna implementacja, nie renderer Beamdog. |
| xoreos-docs `NWN1MDL.bt` | Enum koduje `0=point_list`, `3=triangle_list`, `4=triangle_strip`. | Dokumentacja formatu pomocnicza; zgodna z dekompilacja. |
| Wlasny kontrakt M4, tabela 5.2 | Zapisuje wprost `mesh type byte 0x224 = 3` jako canonical R1 observation. | Historyczny P-REF nie zastępuje fresh runtime proofu, ale jest zgodny z aktualnymi dwiema niezaleznymi obserwacjami. |

Wynik offline tego cyklu: oba obecne testy przechodza (`1/1` + `1/1`), ale
nie sa kontrdowodem. Rigidowy
`minimal_rigid_writer_roundtrips_every_locked_semantic_and_exact_eof` w
`crates/m2a-core/tests/mdl_writer.rs` wprost oczekuje `mesh.mesh_type == 0`.
Skinowy `extended64_skin_roundtrips_1_2_4_lanes_tree_ordinals_and_exact_layout`
nie sprawdza jeszcze `mesh_type`. Wlasny `semantic_diff` porownuje ten bajt,
ale oczekiwanie jest obecnie wyprowadzone z tego samego selektora `0`, wiec
readback jest tylko samospójny. To jest regresja obecnej implementacji, nie
kontrdowod formatowy.

### Minimalna poprawka test-first

1. Zmienic najpierw test rigid M4 z oczekiwania `0` na `3` i po znalezieniu
   `skin_node` dodac `assert_eq!(skin_node.mesh.as_ref().unwrap().mesh_type, 3)`
   dla `extended64`; oba maja byc czerwone na obecnym kodzie.
2. W `plan()` zmienic tylko
   `M4DirectCreatureExtended64V1 => 0` na `=> 3`, a komentarz przy zapisie
   `base + 0x224` na `triangle-list`; M0 pozostaje bez zmiany.
3. Dodac wewnetrzny test mutacyjny w `write_binary_mdl.rs`, oparty na
   istniejacym `semantic_diff`: po poprawce pobrac
   `node = artifact.report.layout.mesh_nodes[0].core_offset`, zapisac tylko
   `0_u32.to_le_bytes()` pod absolutnym offsetem `12 + node + 0x224`, a potem
   wymusic `inspect_binary_mdl(&mutated).unwrap()` i oczekiwac dokladnie
   `nodes[1].mesh.profileDefaults` (dla fixture rigid). To testuje rownoczesnie
   dwa warunki: ogolny reader nadal toleruje mozliwy primitive mode z korpusu,
   ale profilowy readback wlasnego direct-creature odrzuca wlasne `0` wobec
   wymaganych triangularnych danych. Analogiczna dodatnia asercja skinowa z
   punktu 1 zamyka wspolny selector dla obu typow segmentu.
4. Dopiero po zmianie kodu odswiezyc frozen payload SHA; nie wolno uznac
   nowego SHA za proof runtime.

### Zakres i granica

Ta poprawka jest uzasadniona niezaleznie od tego, czy naprawi widocznosc H1:
M4 skin ma osobne, niezamkniete ryzyka weights/bone refs/inverse binds oraz
runtime renderer. Kontrolowany A/B `M4=0` kontra `M4=3`, z identycznym
MDL/MDX/tekstura/HAK/MOD/placement poza tym polem, pozostaje konieczny do
przypisania skutku wizualnego w NWN. Nie dotyczy aktualnego, juz widocznego
w Toolsecie M0 (`Appearance=848`, `MODELTYPE=S`): M0 juz emituje `3`, wiec
nie zmieniac jego roota, klasyfikacji, aliasow, alpha ani tego profilu. W tym
przebiegu nie uruchamiano Aurora ani NWN i nie modyfikowano INI.

## 36. M4/H1 bez vertex colors: sentinel `-1` jest legalny, nie dodawac bialego RGBA (2026-07-21)

### Werdykt

**Brak streamu vertex RGBA w aktualnym profilu M4/H1 nie jest kolejnym
potwierdzonym bledem emitera ani poprawka widocznosci.** W `M4DirectCreatureExtended64V1`
pole mesh `+0x248` jest celowo zapisane jako `-1`, a own-readback zwraca
`vertexColors=[]`. Jest to legalny sentinel pola opcjonalnego. Nie wolno
dopisac do M4 tablicy `[255,255,255,255]` dla kazdego vertexa tylko po to,
aby "odroznic" go od M0.

M0 jest osobnym profilem: emituje biale vertex colors, co tworzy spojny
nieprzezroczysty profil jego aktualnego statycznego artefaktu. Nie wynika z
tego jednak, ze analogiczny stream jest wymogiem skinowanego M4/H1.

### Fakty i granice zrodel

| Zrodlo | Fakt | Wniosek ograniczony do tego faktu |
| --- | --- | --- |
| Aurora decomp, `FUN_00a5e54c` (okolice obslugi `verts`, `tverts`, `colors`, `normals`) | Parser obsluguje dyrektywe `colors` w osobnej, warunkowej galezi. Brak dopasowania tokenu nie wywoluje jej materializacji. | Kolory sa osobnym opcjonalnym elementem ASCII modelu; dekompilat nie ustanawia wymogu ich obecnosci dla draw path. |
| xoreos-docs `NWN1MDL.bt` i specyfikacja `torlack/binmdl.html` | `p_mdx_vertex_colors` jest polem `+0x248`; template czyta RGBA tylko gdy pointer nie jest `-1`, a spec wprost opisuje `-1` jako `not present`. | `-1` jest poprawnym layoutem binarnym, a nie uszkodzonym offsetem. |
| xoreos `ModelNode_NWN_Binary::readMesh()` | Czyta `colorOffset` jako "Vertex RGBA colors", lecz tworzy render-data po wymaganych vertices/faces; nie traktuje koloru jako warunku wejscia do tej galezi. | Niezalezny loader potwierdza, ze to nie jest jego minimalny gate mesha; nie jest rendererem Beamdog. |
| Wlasny artefakt H1 v19 | Skin `m2a_m6p01` ma `1334` vertexy, `vertexColors=[]`, a walidowany pointer `colors` jest null/length `0`; audyt odnotowal zgodnosc sentinela `-1` z obserwowanymi native skin nodes. | Aktualny profil M4 nie omija wymaganego streamu przez blad serializacji. |

Ostatni wiersz jest dowodem o modelu i o readbacku, nie proofem game
runtime. Rownolegle potwierdzony blad `meshType=0` z sekcji 35 pozostaje
osobnym P0 dla M4/H1 i nie moze zostac zamaskowany przez zmiane kolorow.

### Gate i decyzja implementacyjna

Obecne focused testy writera przechodza:

```text
cargo test -p m2a-core --test mdl_writer \
  extended64_skin_roundtrips_1_2_4_lanes_tree_ordinals_and_exact_layout --quiet
# 1 passed

cargo test -p m2a-core --test mdl_writer \
  minimal_rigid_writer_roundtrips_every_locked_semantic_and_exact_eof --quiet
# 1 passed
```

Przy nastepnym rozszerzeniu testow M4 nalezy dodac jawne asercje profilu:

1. rigid i skin `M4DirectCreatureExtended64V1` maja `vertexColors=[]` i raw
   pointer `colors=-1`;
2. M0 zachowuje pelny stream bialego RGBA;
3. parser odrzuca mutacje `+0x248` do nieujemnego offsetu, gdy wskazany
   strumien nie miesci sie w raw data.

Punkt 3 sprawdza bezpieczenstwo parsera; nie oznacza, ze poprawnie zapisana
tablica RGBA jest zabroniona przez format. Jezeli produkt kiedys bedzie
eksportowal autorskie vertex colors, musi to byc jawna cecha IR z jednym
RGBA na vertex i testem rozmiaru, a nie globalny filler. Runtime A/B
"brak streamu kontra bialy RGBA" ma sens dopiero po usunieciu `meshType=0`
i dopiero na kontrolowanym pakiecie NWN.

W tym przebiegu czytano tylko dekompilacje, xoreos, dokumentacje formatu oraz
wlasne artefakty/testy. Aurora, NWN i INI pozostaly nietkniete.

## 37. NeverBlender ASCII axis-angle a binarny controller `type=20`: nie zmieniac M4 na axis-angle (2026-07-21)

### Werdykt

**Nie znaleziono bledu konwencji orientacji w aktualnym binarnym writerze
M4/H1.** NeverBlender zapisuje w ASCII `orientation` jako os i kat (dlatego
czwarta wartosc moze byc np. `3.08670`, a nie skladnikiem unit quaternionu).
Nie jest to jednak reprezentacja pola binarnego controller `type=20`.
W binarnym MDL ten controller zawiera quaternion w kolejnosci `x,y,z,w`.

Nie zmieniac zatem `write_binary_mdl.rs` tak, aby zapisywal oś+kat w
controllerze `20`, ani nie przeliczac z powrotem obecnych q/t tylko z powodu
wygladu referencyjnego ASCII. Bylaby to sprzeczna z binarnym P-REF i z
niezaleznym loaderem.

### Fakty wedlug zrodla

| Zrodlo | Fakt | Znaczenie |
| --- | --- | --- |
| NeverBlender output, reference-only `incaxje.ascii.mdl` | Wiele wierszy `orientation` ma pierwsze trzy wartosci normy bliskiej `1`, a czwarta jest katem w radianach, np. `0.77748 -0.59163 0.21330 3.08670`. | To jest ASCII axis-angle; nie wolno go porownywac pozycja-po-pozycji z binarnym controllerem. |
| xoreos `ModelNode_NWN_Binary::readControllers()` | Dla orientation wymaga `columnCount==4`, odczytuje kolejno `q.x`, `q.y`, `q.z`, `q.q`, a dopiero potem wyprowadza kat `2*acos(q.q)`. | Bezposrednio potwierdza, ze cztery liczby w binarnym controllerze sa quaternionem `XYZW`. xoreos nie jest oracle'em Beamdog, ale jest niezaleznym dekoderem binary MDL. |
| Aurora decomp | Stan noda kopiuje cztery komponenty orientacji, a downstream composition operuje na surowym quaternionie; parser zna tez osobna dyrektywe ASCII `orientationkey`. | Potwierdza rozdzielenie reprezentacji stanowej od tekstowego authoringu. Sam dekompilat nie daje tu pelnego named linku `type=20 -> pole noda`. |
| R1/R4 local binary P-REF | Bind controller `type=20` ma `{rows=1,timeIndex=4,dataIndex=5,columns=4}` i canonical identity `[0,0,0,1]`. | Identity binarna rozni sie od typowej identity axis-angle `[0,0,0,0]`; jest to bezposredni witness binary konwencji. |

Referencyjny eksperyment `incaxje` nie podwaza tego wniosku: jego binarny
CleanModels wariant jest obecnie `UNSUPPORTED_BY_OWN_READER`, a jego proof
widocznosci jest niejednoznaczny/failed. Nie wolno wywodzic semantyki
binarnych controllerow z jego nieudanej obserwacji runtime.

### Aktualny gate i granica runtime

Writer zamienia proper-rigid bind matrix na unit `XYZW`, zapisuje ja w
controllerze `type=20`, a q/t skina wyprowadza z tych samych emitowanych
wartosci. W tym przebiegu przeszly:

```text
cargo test -p m2a-core --test mdl_writer \
  emitted_skin_inverse_bind_matches_worlds_rebuilt_only_from_inspection_controllers --quiet
# 1 passed

cargo test -p m2a-core --test mdl_writer \
  deep_raw_controller_drift_has_stable_inverse_bind_rejection --quiet
# 1 passed
```

Pierwszy test odbudowuje swiaty tylko z odczytanych binary controllers i
sprawdza q/t wobec `inverse(nodeWorld) * skinWorld`; drugi odrzuca glebiej
zlozony drift, ktory nie pozostaje proper rigid. To wyklucza blad serializacji
konwencji w obecnym profilu, nie dowodzi jeszcze deformacji w Beamdog. Po
usunieciu P0 `meshType=0` jedynym rozstrzygajacym testem pozostaje kontrolowany
runtime packet z widoczna, celowo poruszona koscia i porownywalnym capture.

W tym przebiegu nie uruchamiano Aurora ani NWN i nie modyfikowano INI.

## 38. Native mesh-build: `meshType=3` jest gate'em, surface/adjacency nim nie sa (2026-07-21)

### Werdykt

**P0 M4 `meshType=0` z sekcji 35 jest teraz potwierdzony bezposrednio przez
native gate renderera.** W `FUN_00a62b0c` Aurora kontynuuje budowe danych
mesha tylko, gdy ma pozycje (`+0x22c`), UV0 (`+0x234`), normalne (`+0x244`)
i `meshType == 3`. Aktualny M4 emituje wszystkie trzy wymagane streamy, lecz
wpisuje `0` do `+0x224`; zatem nie przechodzi ten native predicate.

Jednoczesnie nie ma podstaw, aby zmieniac face `surface_id=0` lub
`adjacent_faces=[-1,-1,-1]` jako "fix widocznosci". W widocznej galezi
Aurory raw index stream jest zbudowany z triples face i dalej konsumowany
przez mesh-build; face surface/adjacency nie pojawiaja sie w tym predicate ani
w petli z indeksami. M0 juz ma `meshType=3`, wiec ten P0 go nie dotyczy.

### Lancuch dowodowy

| Zrodlo | Fakt | Skutek |
| --- | --- | --- |
| Aurora decomp `FUN_00a62954` | Z trzech `u16` indeksow kazdej face buduje raw index data, ustawia `+0x224=3`, `+0x21c=-1` i `+0x220=0`. | Native triangle compile-path identyfikuje ten tryb jako `3`; obecne productowe `-1/0` dla pomocniczych pol sa zgodne z ta galezia. |
| Aurora decomp `FUN_00a62b0c` | Predicate wymaga `meshType==3`, positions, UV0 i normals przed alokacja/buildem mesh data; nastepnie czyta raw index list przez strukture przy `+0x210`. | `0` nie jest alternatywnym triangle-list mode dla emitowanego M4. Jest konkretnym kandydatem na odrzucenie mesha przed draw. |
| xoreos `readMesh()` | Czyta face normals i trzy indeksy, ale adjacency omija; do budowy geometrii uzywa indeksow. | Niezalezny loader wspiera rozdzielenie indeksow od adjacency; nie jest proofem Beamdog. |
| Current writer/readback | Zapisuje face triples i identyczny raw `u16` index stream, ale selector M4 nadal daje `0`; test literalnie oczekuje `mesh_type == 0`. | Nalezy poprawic selector i test, nie geometrie ani face fields. |

Read-only canonical P-REF CEP HAK ponownie przeszedl `1/1` przez istniejacy
test `canonical_cep_hak_builds_all_required_p_ref_packets_without_extracting_payloads`.
Potwierdza to, ze own reader nadal akceptuje kanoniczny corpus bez kopiowania
payloadow; sam wynik nie jest runtime proofem M4.

### Minimalna poprawka i test

Wlasciwa kolejnosc pozostaje test-first:

1. zmienic test rigid M4 z `mesh_type == 0` na `== 3` i dodac te sama
   asercje do skinowego extended64;
2. zmienic jedyny selector `M4DirectCreatureExtended64V1 => 0` na `=> 3`
   oraz komentarz, ktory blednie nazywa zero triangle-list;
3. dodac gate emitera: mutacja tylko `+0x224` z `3` na `0` ma zwrocic
   nazwany blad M4, przy zachowaniu tolerancji readera dla obcych modeli;
4. dopiero potem zaktualizowac frozen SHA i wykonac hashowany NWN A/B.

Nie wykonano tu poprawki kodu, Aurora, NWN ani INI nie byly uruchamiane lub
modyfikowane. Runtime A/B jest nadal konieczny do przypisania skutku
wizualnego, lecz nie jest potrzebny, aby stwierdzic blad native eligibility
emitowanego M4.

## 39. Globalnie zduplikowane nazwy nodow: P1 walidacji emitera, nie przyczyna aktualnego M0/H1 (2026-07-21)

### Werdykt

**Writer dopuszcza obecnie case-insensitive duplicate node names w roznych
galeziach drzewa. Jest to potwierdzony blad walidacji eksportu dla dowolnego
przyszlego modelu wielonodowego, lecz nie wyjasnia obecnego M0 ani H1.** M0 ma tylko
`m2a_m0p01` i `m2a_seg_1`; odczyt bazowych artefaktow H1 v20 i v21 daje 25
nodow, jeden root i zero nazw zduplikowanych po ASCII case-fold.

Nie nalezy wiec mutowac nazewnictwa aktualnego M0/H1 jako testu widocznosci.
Trzeba natomiast uszczelnic writer przed kolejnym rigiem, gdzie import moze
dostarczyc np. dwa `shared` albo `Bone`/`bone` w roznych poddrzewach.

### Lancuch dowodowy

| Zrodlo | Fakt | Skutek |
| --- | --- | --- |
| Aktualny writer i `duplicate_names_across_branches_are_legal_but_case_folded_siblings_are_ambiguous` | Test akceptuje dwa `shared` w roznych galęziach i zapisuje je jako dwa binary nody; kontroluje tylko kolizje bezposrednich dzieci przy animacji. | Emiter potrafi wytworzyc artefakt z globalnie niejednoznacznym identyfikatorem noda. |
| xoreos, `ModelNode_NWN_Binary::checkDuplicateNode()` | Przy wczytywaniu binary MDL skanuje **wszystkie** juz widziane nody `equalsIgnoreCase`; dla duplikatu ostrzega, przepina dzieci do nowego noda, usuwa stary node i go kasuje. | Niezalezny loader nie traktuje takiej struktury jako dwoch niezaleznych nodow. To realne ryzyko utraty mesha lub transformu, nie tylko kwestia estetyczna. |
| Aurora decomp `FUN_00a58384`, `FUN_00a57b58`, `FUN_00a57af4`, `FUN_00b8737c` i `FUN_00b8f008` | Dyrektywa `parent` jest czytana jako nazwa i rozwiazywana rekurencyjnie przed dolaczeniem dziecka. Porownanie normalizuje litery przez `LCMapStringA` z flaga `0x200` (lowercase); traversal zwraca pierwszy zgodny node. | Dla ograniczonego do ASCII emitera `Bone` i `bone` sa tym samym identyfikatorem. Kolejnosc wyboru wynika z traversalu, wiec nie wolno jej traktowac jako stabilnego kontraktu outputu. |
| Dokumentacja authoringu [MDL](https://nwn.wiki/spaces/NWN1/pages/38175669/MDL?src=contextnavpagetreemode) i reference-only CleanModels | Wiki ostrzega, ze dwa nody o tej samej nazwie moga wywolac "weirdness or crashes" po ograniczeniu do 32 znakow; CleanModels renumerowuje zwykle duplikaty, a animated/skin duplicate oznacza jako blad. | Daje niezalezna regule authoringu: eksport ma gwarantowac globalna unikalnosc, nie tylko unikalnosc rodzenstwa. |
| NeverBlender/NWN wiki [Models](https://nwn.wiki/spaces/NWN1/pages/38175602/Models?src=contextnavpagetreemode) | NeverBlender jest wspieranym eksporterem modelu NWN; te same reguly 32-bajtowego pola nazwy dotycza wynikowego MDL, a nie wyłącznie tekstowego authoringu. | Limit i sprawdzenie unikalnosci musza lezec w naszym binary writerze, przed serializacja. |

Wynik H1 pochodzi z read-only przejscia drzewa po `nodeHeadPointer` i tablicach
children: deklarowane `25` nodow zostalo odwiedzone `25`, root to
`m2a_m6p01`, a zbior `lowercase(ASCII name)` ma `25` elementow. Nie jest to
proof renderu H1; wylacza tylko ten konkretny wariant z listy przyczyn jego
aktualnych problemow.

### Minimalna poprawka i test-first

1. Przed planowaniem layoutu zbudowac jeden globalny zbior nazw ASCII
   case-fold dla wszystkich `creature.nodes` **oraz** wygenerowanych nazw
   `m2a_seg_<segment_id>`; kolizja ma zwrocic nowy, niespecyficzny dla
   animacji blad, np. `M4-NODE-NAME-DUPLICATE`. Nie jest to juz tylko policy
   xoreos: odpowiada case-fold w native resolverze `parent`.
2. Zastapic dodatnia czesc obecnego testu z dwoma `shared` w roznych galęziach
   asercja named error. Dodac osobny przypadek `Bone`/`bone` w roznych
   galęziach oraz kolizje wejściowego rig noda z `m2a_seg_5`.
3. Zachowac mozliwosc dwoch odmiennych nazw w roznych galęziach i nie
   normalizowac/nie przemianowywac automatycznie: nazwa jest kontraktem rig,
   skin i animacji, wiec cicha zmiana bylaby gorsza niz jawny blad eksportu.

To jest P1 hardening emitera. Nie blokuje P0 `M4 meshType=0 -> 3`, nie dotyczy
aktualnego statycznego M0 i nie wymaga testu Aurora/NWN, aby zapobiec emisji
jednoznacznie ryzykownego drzewa. Aurora, NWN i INI pozostaly nietkniete.

## 40. Skin tylko w base state: potwierdzona korekta M6 dla M4, negatywny wynik dla M0 (2026-07-21)

### Werdykt

**Nie ma tu poprawki dla aktualnego M0.** Jest jednym sztywnym segmentem bez
`skin`; jego lokalne clipy zawieraja tylko root i bezkontrolerowy rigid alias.
Zmiana policy dla skinowanych animation trees nie moze wiec wyjasnic jego
zachowania. Po swiezym r21 Toolset proofie widocznej sylwetki nie wolno na tej
podstawie mutowac klasyfikacji, roota, aliasow ani alpha M0. Runtime M0 nadal
ma status `not_tested`, a nie `not_visible`.

**Dla skinowanych modeli M4 jest to juz potwierdzona binarna rozbieznosc
emitera, ale P1 (nie przypisana jeszcze do runtime H1).** Aktualne
`plan_animations()` dodaje pod koscia rodzica `SkinMeshDummy`, ktory writer
serializuje jako zwykly node `0x01` o nazwie bazowego skina
`m2a_seg_<id>`. Skompilowany binary witness CEP `c_squirrel` ma dwa skiny
`0x61` tylko w base tree i **zero** skin nodes w kazdym ze swoich 43 states.
Nie wolno wiec utrzymywac pseudo-skinowego `0x01` jako native profile ani
zamieniac go na `0x61`: poprawny minimalny output nie emituje dla skina zadnego
animation child.

Status: `CONFIRMED_STRUCTURAL_M6 / P1`. Najpierw pozostaje P0
`M4 meshType=0 -> 3`. M6 nalezy zaimplementowac test-first jako lokalna
korekte kontraktu; nie wolno jej przedstawic jako potwierdzonej przyczyny
niewidocznosci lub deformacji H1 przed kontrolowanym runtime A/B.

### Binarna referencja i reprodukowalnosc

Read-only, in-place resource z lokalnej instalacji (niekopiowany do repo):

| Pole | Wartosc |
| --- | --- |
| HAK | `C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\cep3_core1.hak` |
| resref / resource | `c_squirrel`, id `6396`, type `2002` (MDL) |
| zakres payloadu | offset `1424455248`, dlugosc `949324` B; core `892084` B + raw `57228` B |
| SHA-256 calego payloadu | `fce071fcebc04d0ffce33b6ef48891af615e70dd402120554dcc963cca36f78b` |
| base tree | deklaruje i osiaga `30` nodow; dokladnie `2` skin: `sqrlback`, `sqrlfront` (flagi `0x61`) |
| own animation states | `43`; deklarowane i osiagniete lacznie `1204` nody |
| wynik kluczowy | w state trees `skin=0`; histogram flag to `0x01:258`, `0x21:903`, `0x121:43` |
| powtarzalny shape stanu | kazdy z 43: `28`/`28` nodow, w tym `6` x `0x01`, `21` x `0x21`, `1` x `0x121`; ani jednego `0x61` |

Ten witness jest silniejszy od poprzedniego ASCII licznika: sprawdza wlasnie
binary flags i drzewa, ktore ma odczytac runtime. Nie ustanawia sam pelnego ABI
kazdego typu noda ani nie dowodzi wizualnego skutku zmiany, ale bezposrednio
wyklucza obecne zalozenie writera, ze skin potrzebuje generic aliasu `0x01` w
kazdym state.

### Lancuch dowodowy z wymaganych zrodel

| Zrodlo | Fakt | Granica wniosku |
| --- | --- | --- |
| Aktualny writer `crates/m2a-core/src/mdl/write_binary_mdl.rs`, `plan_animations()` i `emit_animations()` | Kazdy rigid dostaje zero-geometry `0x21`, a kazdy skin `SkinMeshDummy`; dla drugiego `node_size=NODE_HEADER_SIZE` i flagi wynosza `0x01`. Bazowy skin ma osobno `0x61`. Aktualny test wprost oczekuje generic skina w state tree. | To jest dokladne miejsce roznicy i minimalny scope poprawki. Nie jest jeszcze dowodem, ze `0x01` niszczy render w grze. |
| Binary CEP `c_squirrel` opisany wyzej | Base tree ma dwa skiny `0x61`: `sqrlfront` (`371` vertices) i `sqrlback` (`245` vertices). Oba maja `render=1` i `meshType=3`; wszystkie 43 binary state trees maja lacznie 1204 nody i zero skina. Mesh topology w states jest reprezentowana przez `0x21` (oraz `0x121` dla dangly), nie przez alias skina. | Jeden model nie usprawiedliwia zgadywania innych node profiles, ale jest bezposrednim witnessiem przeciw `SkinMeshDummy` dla tego native patternu oraz potwierdza `meshType=3` dla prawdziwego skinmesha. |
| Aurora decomp `C:\\Projects\\New Folder\\export\\decompiled_all.c`: `FUN_00a5d224`, `FUN_00a4cba4`, `FUN_00a5d1d4` | Parser/factory rozroznia odrebny obiekt skin od generic noda; konstruktor skina przechodzi przez TriMesh i ma osobna obsluge. | Potwierdza semantyczna roznice skin/dummy; sam dekompilat nie narzuca, ze skin ma wystapic w state. |
| xoreos [`model_nwn.cpp`](https://github.com/xoreos/xoreos/blob/master/src/graphics/aurora/model_nwn.cpp) | `kNodeFlagHasSkin=0x40`, wiec `0x61` ma dodatkowy skin layout, a `0x01` go nie ma. Animacje sa czytane jako osobne states; gdy brak local position/orientation, loader dziedziczy wartosc z base node o tej samej nazwie. | Niezaleznie wyjasnia, dlaczego state nie potrzebuje skina do zachowania bind pose. xoreos nie jest oracle'em Beamdog. |
| NeverBlender `nvb_animnode.py` / `nvb_utils.py` i [NWN Wiki: Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations) | Eksporter ASCII bierze typ noda z obiektu bazowego, wiec moze wypisac `node skin` w animacji. Wiki ostrzega, ze eksportowane animacje moga miec wadliwe headery i zaleca weryfikacje wobec akceptowanego szablonu. | Jest kontrastem authoringu, nie policy dla binary outputu. Nie wolno traktowac jego `node skin` jako powodu do utrzymania pseudo-skina w naszym binary state. |

### Minimalna poprawka i test-first

1. Najpierw zmienic/dodac wlasna fixture: jedna hierarchia rig, jeden skin i
   jeden clip z ruchem kosci. Dla kazdego state tree asercja ma znalezc nody
   rigowe oraz tylko potwierdzone rigid placeholders, ale **nie**
   `m2a_seg_<skin-id>` i nie `SkinMeshDummy`/`0x01` o nazwie skina.
2. Zachowac oddzielna asercje base tree: skin pozostaje `0x61`, z tymi samymi
   wagami, refami, materialem i bind q/t. Test nie moze oslabiac istniejacego
   kontraktu rigid `0x21` ani ruchu kontrolera kosci.
3. Minimalna implementacja: usunac wariant `SkinMeshDummy` i jego dolaczanie
   do `mesh_children` w planie animacji (skin jest po prostu pomijany przy
   budowie state tree). Nie emitowac pseudo-`skin`, nie zmieniac base skina,
   tekstur, alpha, root classification ani hierarchy rig.
4. Dodac env-gated, read-only invariant referencji CEP z powyzszym hashem,
   metadanymi i licznikiem `base skin=2`, `states=43`, `state skin=0`. Fixture
   pozostaje jedynym materialem CI i nie wolno kopiowac payloadu CEP.
5. Po P0 i po swiezym, zwiazanym z kandydatem negatywnym proofie H1 wykonac
   runtime A/B na identycznych, hashowanych pakietach roznicy jednego aliasu.
   Do tego czasu wynik brzmi: poprawiony binary contract, **runtime impact
   not proven**.

W tym przebiegu czytano tylko dekompilacje Aurory, lokalny binary CEP, xoreos,
source NeverBlender i NWN Wiki. Aurora, NWN i INI pozostaly nietkniete.

## 41. `boneconstantindices`: zera pozostaja; 4-bajtowy odczyt P1 i negatywny wynik direct-palette (2026-07-21)

### Werdykt

**Nie zmieniac emitera M4, aby kopiowal niezerowe `boneconstantindices` z
`c_squirrel`.** Obecne zera pozostaja minimalnym, dobrze wspartym domyslnym
outputem: 22 niezalezne, skin-bearing binarne modele z lokalnego corpusu CEP
maja lacznie 2346 takich wpisow i wszystkie sa zerowe. `c_squirrel` jest
wyjatkiem danych, nie szablonem do klonowania.

Jest natomiast realna poprawka P1 w naszym parserze i IR: element tablicy ma
dokladnie 4 bajty i powinien byc zachowany jako nieinterpretowane `u32`, a nie
jako para `[i16; 2]`. To jest poprawa wierności round-trip, nie zmiana
semantyki skina ani odpowiedz na runtime H1.

Najwazniejszy wynik negatywny: Aurora materializuje te slowa przy ladowaniu,
ale prześledzona bezposrednia sciezka budowy palety skinu ich nie odczytuje.
Zera `bone_constants` nie sa zatem obecnie realistycznym izolowanym kandydatem
na blad pierwszego renderu bezposredniego skina. Nie rozstrzyga to wszystkich
mozliwych konsumentow engine'u ani deformacji runtime.

Zakres: brak wplywu na aktualny statyczny M0. M0 nie ma noda `skin`, tablic
skin ani tego pola; jego Toolset proof r21 pozostaje `visible`, a NWN
`not_tested`.

### Dowod binarny i corpus

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aktualny parser/writer | Parser czyta constants jako elementy 4 B, ale reprezentuje je jako `[i16; 2]`. Writer alokuje `mapCount * 4` B i wpisuje dwa zera `i16` na wiersz. | Output zero jest bajtowo rowny jednemu slowu `0u32`, lecz typ nieslusznie sugeruje dwie znane wartosci. |
| Aurora decomp | `FUN_00a5cc98` przekazuje `boneconstantindices` do storage noda; `FUN_00a5c84c` i `FUN_00a56ea4` obsluguja element jako atomowy 4-bajtowy wpis. Loader `FUN_00a3dd4c` kopiuje `count << 2` bajtow. | Tekstowy importer moze nadawac temu wartosci bool-podobne; nie definiuje znaczenia skompilowanego binary payloadu. |
| CEP `c_squirrel` | Dwa skiny maja po 30 wpisow; czesc slow jest niezerowa. Widok tych samych 4 bajtow jako dwoch `i16` daje arbitralne pary, a jako `f32` czasem liczby wygladajace jak floaty. | Zadna z tych reprezentacji nie jest dowodem, ze pole jest para indeksow albo floatem. |
| Skan CEP | Deterministyczny, read-only scan pierwszych 20 skin-bearing `c_*` modeli dal 2346/2346 zerowych slow; dodatkowo `c_vampire_f` (2 skiny, 28 wierszy kazdy) i `c_kocrachn` (3 skiny, 38 wierszy kazdy) maja same zera. | To szeroka konwencja, nie uniwersalna specyfikacja, bo `c_squirrel` jest obserwowanym wyjatkiem. |

Referencja wyjatku jest czytana in-place, bez kopiowania do repo:
`C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\cep3_core1.hak`,
`c_squirrel`, id `6396`, type `2002`, payload SHA-256
`fce071fcebc04d0ffce33b6ef48891af615e70dd402120554dcc963cca36f78b`.

### Negatywny trace renderera

Po odczytaniu mapy, q/t i constants loader Aurory alokuje i kopiuje constants
z dlugoscia `count << 2`. Nastepnie prześledzona sciezka direct skin
`FUN_00a936a4 -> FUN_00a9228c -> FUN_00a932cc` iteruje 64 wpisy inline map,
pobiera bone ordinals i q/t, buduje palete i przekazuje ja dalej do draw path.
Nie ma w niej odczytu zaalokowanej tablicy constants.

To jest **negatywny wynik dla konkretnej hipotezy**, a nie twierdzenie, ze
engine nigdy nie uzywa tego pola. Nie wolno usuwac diagnostyki
`M4-SKIN-CONSTANTS-MEANING-OPEN-M6`, ale nalezy obnizyc jej priorytet ponizej
P0 `meshType=0 -> 3` i P1 usuniecia `SkinMeshDummy` z animation states.

### Minimalna poprawka test-first

1. Najpierw zmienic syntetyczna extended-skin fixture, aby zawierala pelne
   slowo, np. `0x0008_0007`, a odczyt asercyjnie zwracal dokladnie `u32`.
2. Dodac mutacje calego slowa (w tym bitu wysokiego) i wymagac, aby
   `semantic_diff` wskazywal `nodes[...].skin.constants`, bez konwersji do
   float/bool/pary indeksow.
3. Dopiero potem zmienic `bone_constants: Vec<[i16; 2]>` na lossless
   `bone_constant_words: Vec<u32>` oraz writer na jeden zapis `u32`. Dla
   aktualnego emitera wartosc pozostaje `0u32`, wiec output danych M4 nie
   zmienia sie bajtowo.
4. Przyszly env-gated P-REF moze sprawdzac tylko count rowny `mapCount` i
   bezstratny odczyt surowych slow. Nie wolno oczekiwac rownosci z
   `c_squirrel` ani wprowadzac niezerowych slow do fixture bez osobnej,
   udokumentowanej semantyki.

Aurora, NWN i INI nie byly uruchamiane ani modyfikowane w tym przebiegu.

## 41a. M0: klasyfikacja, root i alpha sa zgodne; nie sa kolejna delta eksportu (2026-07-21)

### Werdykt

**Dla aktualnego, hash-bound `m2a_m0p01` nie zmieniac classification, roota,
`animroot`, `render`, transparency ani alpha tekstury.** Read-only audit
nie znalazl w tych polach roznicy uzasadniajacej kolejna iteracje eksportu:

| Pole M0 | Odczyt aktualnego MDL/TGA | Native porownanie / granica |
| --- | --- | --- |
| `classification` | `4` | `c_squirrel` ma rowniez `4`; w CEP `3368/3430` binary MDL i `1166/1182` modeli z local animations ma te wartosc. To mocne wsparcie dla profilu creature, nie publiczna definicja enumu. |
| model root i `animroot` | nazwa modelu `m2a_m0p01`; wszystkie 7 clipow maja dokladnie ten `animroot` | `c_squirrel` ma `c_squirrel` w kazdym z 43 clipow. W calym sweepie `20623/24707` local headers ma `animroot == model name`; pozostale `4084` pokazuja, ze nie wolno zrobic z tego globalnego wymogu readera. |
| draw / material | jedyny mesh `m2a_seg_1` ma flags `0x21`, `render=1`, `transparency=0`, `render_hint=0`, `meshType=3`, texture0 `m2a_m0t01` | `render=0` bylby bezposrednia negacja draw. Nie ma podstawy, by mutowac te pola przy widocznym Toolset M0. |
| alpha tekstury | `m2a_m0t01.tga`, SHA-256 `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`, `2048x2048`; alpha min/max `255/255`, `4194304` piksele opaque | Ten konkretny asset nie ma przezroczystych pikseli. Nie jest to dowod, ze wszystkie przyszle tekstury musza byc opaque. |

Aktualny MDL ma SHA-256
`cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5`.
Jego base tree to tylko root `0x01` i trimesh `0x21`; nie ma skina. Wobec
tego poprawka P1 state-skin nie dotyczy M0, a `meshType=3` jest juz poprawne.
Jedyna znana rozbieznosc w lokalnym animation ABI pozostaje `type=0` zamiast
native `type=5` z sekcji 44.

`modelVisibility=visible` w Toolsecie jest dowodem, ze ten konkretny chain
zostal tam narysowany; **nie** jest dowodem runtime NWN. Brak kontrolowanego
packetu NWN nadal ma `proofCompleteness=missing`, a nie wynik
`modelVisibility=not_visible`.

### Lancuch dowodowy

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aktualny packet M0, odczyt binary MDL/TGA in-place | Daje wszystkie wartosci z tabeli, hashe, osiem kontekstowych node/clip metadanych oraz histogram alpha. | Wlasny odczyt binarny potwierdza output, ale nie wykonuje gry. |
| Bounded sweep `cep3_core1.hak` | Daje histogram classification oraz licznik relacji `animroot` do nazwy modelu; `c_squirrel` jest matching witness `classification=4` i root/name. | Jeden HAK nie definiuje wszystkich legalnych klas ani wariantow animroot. |
| Aurora decomp `C:\\Projects\\New Folder\\export\\decompiled_all.c`, `FUN_00a3edc0`, `FUN_00a3ee64`, `FUN_00a3ed90` | Loader kopiuje z naglowkow pola classification/fog oraz dla local state name, root i byte `type`; nie pojawil sie tu predicate, ktory wymagalby dla M0 innej klasy lub innego roota. | Brak odczytanego predicate nie jest dowodem pelnej semantyki enumu; jest wynikiem negatywnym przeciw przypadkowej zmianie. |
| xoreos `model_nwn.cpp` | Czyta `type`, classification i fog z geometry headera; osobno czyta local state oraz jego `animroot`. | Niezalezny reader, nie oracle Beamdog. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Aurora Root ma miec nazwe pliku; Character obejmuje creatures/placeables. Manual opisuje tez `render` jako kontrolke rysowania, a transparency hint jako priorytet transparentnych obiektow. | Manual authoringu nie jest specyfikacja binary ABI ani proofem NWN. |

### Minimalny test i ograniczenie dalszej pracy

1. Dla fixture **M0** utrzymac preflight/readback: `classification=4`, base
   root name = model resref, kazdy z 7 `animroot` = model resref,
   `flags=[0x01,0x21]`, `render=1`, `transparency=0`, `render_hint=0`,
   `meshType=3` i texture alpha `255` dla kazdego piksela.
2. Negatywna mutacja `mesh.render: 1 -> 0` musi zostac odrzucona przez policy
   M0 jako oczywiscie niewidoczny profil. Nie dodawac testu, ktory wymusza
   root/name albo classification `4` na wszystkie modele referencyjne.
3. Nie tworzyc wariantu M0 ze zmieniona klasyfikacja/rootem/alpha. Jedyny
   uzasadniony test-first patch to nadal pojedynczy byte `+0x6c: 0 -> 5`, z
   testem negatywnym opisanym w sekcji 44; po nim potrzebny jest ten sam,
   hash-bound proof Toolset -> NWN.

Aurora, NWN i INI nie byly uruchamiane ani modyfikowane w tym przebiegu.

## 41b. Korekta P1: state-skin ma trzy native profile; `SkinMeshDummy` nie jest bledem sam w sobie (2026-07-21)

### Werdykt

**Sekcja 40 zostaje skorygowana w czesci postulatu „usunac
`SkinMeshDummy`”.** Jej obserwacja dla `c_squirrel` jest poprawna: ten model ma
base skiny `0x61` i zero skin node'ow w 43 state trees. Nie wolno jednak
przeksztalcac jednego witnessa w globalna regule writer'a.

Read-only, in-place sweep `cep3_core1.hak` (SHA-256 calego HAK-a
`6a8e6a64773a77fd46740cbcce19a708db6a70b4975732d0405978f3fbe8eb1a`) przeszedl
bez bledu po `3430` binary MDL (dalsze `87` type `2002` to ASCII) i pokazal
trzy odrebne native profile dla modeli z base skinem oraz local animations:

| Profil stanu | Zakres w corpusie | Co wolno z tego wywnioskowac |
| --- | --- | --- |
| Brak state-skin | `123` z `137` takich modeli, w tym `c_squirrel` | Jest legalnym profilem native, nie dowodzi zakazu aliasu. |
| `0x01` o nazwie base-skin | `3029` node'ow w `43` modelach; `3015` ma zero dzieci i zero kontrolerow | Dokladny minimalny layout `NODE_HEADER_SIZE`, ktory dzisiejszy writer nazywa `SkinMeshDummy`, ma realne native witnessy. Sam `0x01` nie jest wiec kandydatem na blad eksportu. |
| matching-name `0x61` state-skin | `1211` node'ow w `14` modelach; kazda nazwa odpowiada base skinowi | To osobny profil, ktorego nie wolno ani automatycznie generowac, ani uznawac za konieczny bez dalszego kontraktu. |

Pozostale `14` wsrod `137` modeli skinowanych i animowanych maja ten trzeci
profil; nie ma rozjazdu nazw: wszystkie `1211` state-skin node'ow maja nazwe
obecna w base skin. To koryguje poprzedni wniosek, ale **nie** jest dowodem,
ze dowolny `0x01` lub `0x61` zapewnia widocznosc albo deformacje w NWN.

Aktualny kandydat `m2a_m0p01` jest single rigid M0: raport materializacji
podaje `rigNodeCount=1` i `meshNodeCount=1`, bez skina. Ta korekta P1 nie
dotyczy wiec Appearance `848`, roota, klasyfikacji, tekstury ani runtime
widocznosci M0. Wobec siedmiu lokalnych clipow M0 z `type=0` na `+0x6c`
najmocniejszym pozostajacym offline kandydatem jest nadal P0 z sekcji 44.

### Lancuch dowodowy i granice

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aktualny writer `write_binary_mdl.rs`, `AnimationNodeKind::SkinMeshDummy` oraz `plan_animations()` | Dla skina planuje node `NODE_HEADER_SIZE`, flage `0x01`, bez dzieci i bez trackow. | Nie dowodzi jeszcze zgodnosci parent/part-number dla kazdego profilu, ale opisuje porownywany minimalny layout. |
| Bounded read-only sweep CEP opisany wyzej | Liczby `137/123/14`, `3029`, `3015` i `1211`; zero parse failures. Dla `0x01` porownano nazwe z base skinem oraz liczbe dzieci/kluczy/danych kontrolerow. | Jeden HAK nie jest cala specyfikacja NWN; wynik nie kopiuje zadnego payloadu do repo. |
| Aurora decomp `C:\\Projects\\New Folder\\export\\decompiled_all.c`, `FUN_00a3b874` i rekurencyjny `FUN_00a3b994` | Aurora tworzy osobne drzewo dla kazdego local state i rekurencyjnie materializuje jego node'y. | Dekompilacja nie ustanawia policy, ktory z trzech profilow writer ma wybrac. |
| xoreos `src/graphics/aurora/model_nwn.cpp` | `kNodeFlagHasMesh=0x20`, `kNodeFlagHasSkin=0x40`; `0x61` ma wiec skin layout, zas `0x01` jest header-only. Stany sa czytane jako osobne drzewa. | xoreos nie jest oracle'em Beamdog i nie interpretuje tu gameplayowego wyboru profilu. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Skinmesh sluzy do animacji szkieletowej, zas podstawowym typem geometrii jest triangulowany trimesh. | Manual authoringu nie definiuje binary policy dla state tree. |

### Negatywny wynik i minimalny test

1. **Nie implementowac usuniecia `SkinMeshDummy`.** Taki patch nie ma juz
   poprawnej przeslanki corpusowej; dla obecnego rigid M0 bylby ponadto
   zupelnie nieaktywny.
2. Dodac env-gated, read-only invariant corpusowy, ktory odroznia trzy profile:
   `c_squirrel` jako konkretny przypadek `state skin=0`; co najmniej jeden
   witness `0x01` matching-name/leaf/no-controller; oraz jeden witness `0x61`
   matching-name. Test ma porownywac metadane i hash HAK-a, nie kopiowac MDL.
3. Dla wlasnej skin+motion fixture utrzymac test, ze dzisiejszy `0x01` leaf
   ma flage `0x01`, zero dzieci i zero trackow; nie wolno zamieniac go na
   `0x61` ani oczekiwac globalnie `state skin=0`.
4. Osobny przyszly test moze wprowadzic profil `0x61` dopiero po rozpisaniu
   pelnego layoutu i policy wyboru. Nie mieszac go z P0 `type=5` ani z M4
   `meshType=3`.

Aurora, NWN i INI nie byly uruchamiane ani modyfikowane w tym przebiegu.

## 42. `animroot`: nie wymagac globalnie nazwy root noda; direct M0/M4 sa juz znormalizowane (2026-07-21)

### Werdykt

**Nie ma poprawki eksportu `animroot` dla aktualnego direct-creature M0 ani
M4/H1.** Oba produktowe pipeline'y przed serializacja nazywaja jedyny geometry
root resrefem modelu i ustawiaja taki sam `animroot` dla kazdego wlasnego
klipu. Odczyt binarny M0 i H1 to sprawdza.

Rownoczesnie **nie wolno dodac do ogolnego writera walidacji, ze `animroot`
musi wskazywac serializowany animation/root node**. Lokalny compiled witness
R3 pokazuje natywne, poprawnie odczytywane modele, w ktorych `animroot` ma
stala nazwe per model, inna niz nazwa serialized root noda. Pole jest
obecne w binarnym naglowku i parserze Aurora, lecz jego bezposredni consumer
runtime pozostaje nieodkryty.

To jest zatem negatywny wynik dla hipotezy „unmatched animroot wyjasnia
aktualny M0/H1”. Dla API ogolnego zachowac obecna walidacje skladni i jawne
odchylenie `M4A-DECOMP-ANIMROOT-CONSUMER-OPEN-M6`; dla produktu
direct-creature utrzymac juz istniejaca normalizacje do resrefu.

### Fakty wedlug zrodla

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp `FUN_00a5d758` | Parser ASCII rozpoznaje `animroot` i kopiuje string do naglowka animation pod `+0x78`. | Dowodzi serializacji/pola, nie odnaleziono binarnego konsumenta sterujacego draw lub routingiem. |
| xoreos `readAnimBinary()` | Czyta `animRoot` z 64-bajtowego pola obok `animLength` i `transTime`, ale lokalna zmienna nie jest dalej stosowana przez ten loader. | Nie wolno wnioskowac, ze Beamdog ignoruje pole; to tylko granica xoreos. |
| Native R3 `c_phod_horror_b` / `c_phod_horror_p` | `animroot` wynosi odpowiednio `phod_horror_blak` / `phod_horror_purg`, podczas gdy serialized animation root nody maja nazwy modeli. | Bezposredni kontrprzyklad dla globalnej reguly `animroot == node name`. Nie udostepnia semantyki consumer'a. |
| Native direct `c_squirrel` | Wszystkie 43 stany maja `animroot c_squirrel`, zgodny z modelem i jego rootem. | Wspiera konwencje self-contained direct-creature, lecz nie ustanawia jej jako uniwersalnego ABI. |
| Aktualny pipeline | `normalize_direct_creature_runtime_root()` wymaga jednego geometry root i ustawia jego nazwe oraz wszystkie `animation_root` na model resref. Testy M0 i H1 odczytuja ten invariant. | To correctness gate dla naszej waskiej sciezki produktu, nie definicja ogolnego binary MDL. |
| NeverBlender / NWN Wiki | Authoring workflow ostrzega, ze eksportowane przez NeverBlender naglowki animacji moga byc wadliwe i porownuje ASCII z zaakceptowanym szablonem. | Praktyczna ostroznosc dla authoringu; nie jest specyfikacja binary consumer'a `animroot`. |

### Test i decyzja implementacyjna

1. Zachowac test ogolnego writera z legalnym, niepowiazanym `animation_root`:
   rozroznia on poprawne przechowanie pola od nieudowodnionej semantyki
   runtime.
2. Zachowac produktowy test direct: dla kazdego klipu `animation_root ==
   M0_MODEL_RESREF` lub `M6_MODEL_RESREF`, a animation tree ma wlasny root o
   tej samej nazwie. To jest profile policy oparta na `c_squirrel`.
3. Nie dodawac nowej korekty MDL, aliasow ani node names na podstawie tej
   hipotezy. Rozstrzygajacy proof pozostaje runtime packet z celowym ruchem
   kosci po usunieciu P0 `meshType=0` i P1 `SkinMeshDummy`.

W tym przebiegu przeszly:

```text
cargo test -p m2a-core --test model_pipeline static_meshy_m0 --quiet
# 4 passed
cargo test -p m2a-core --test model_pipeline meshy_h1 --quiet
# 1 passed
```

Aurora, NWN i INI pozostaly nietkniete.

## 43. R21 M0: bezkontrolerowe aliasy animacji nie sa uzasadniona poprawka eksportera (2026-07-21)

### Pytanie i werdykt

Swiezy, zwiazany z kandydatem proof Toolsetu r21 pokazuje statyczny M0 jako
widoczna sylwetke przy `Appearance_Type=848`, `MODELTYPE=S` i
`RACE=m2a_m0p01`. Zatem dla **tego** M0 nie ma juz offline podstaw, aby
zmieniac `classification`, geometry root, `animroot`, rodzaj wezla ani alpha
tekstury jako domniemana poprawke niewidocznosci.

Jedyna pozostajaca waska hipoteza eksport/runtime brzmi: klient NWN po wyborze
stanu `cpause1` moglby wymusic controller, mimo ze Toolset narysowal base pose.
Wynik tego przebiegu jest **negatywny**: nie znaleziono takiego wymogu ani
mechanizmu, ktory zamienialby aktualny bezkontrolerowy alias w pusta siatke.
Nie implementowac identity keyframes w produkcyjnym M0. Status runtime M0
pozostaje `modelVisibility=not_tested`, `proofCompleteness=missing`, a nie
`not_visible`.

### Ponownie sprawdzone fakty

| Zrodlo | Fakt | Konsekwencja |
| --- | --- | --- |
| Aktualny offline packet `m0-direct-resolver-gate-20260721` | MDL `m2a_m0p01` ma SHA-256 `cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5`; raport deklaruje physical row `848`, siedem clipow, jeden root i jeden rigid segment. W kazdym clipie root i `m2a_seg_1` nie maja trackow. | To konkretny profil, a nie brakujaca wartosc przypadkowo opuszczona przez serializacje. |
| Wlasny test `static_meshy_m0` | Cztery testy przechodza: sprawdzaja kompletnosc siedmiu aliasow oraz binarny readback root/name/part/flag `0x21` i braku geometrii aliasu. | Gate wykryje utrate aliasu lub przypadkowa zmiane profilu, ale celowo nie twierdzi, ze test unit jest rendererem NWN. |
| xoreos `ModelNode_NWN_Binary::readMesh()` | Dla `vertexCount==0`, `facesCount==0` lub `facesOffset==0` wraca przed budowa render-data. `Animation::update()` zmienia target tylko, gdy lista position/orientation frames jest niepusta. | W niezaleznym loaderze pusty rigid alias ani nie zastępuje render-data base mesh, ani nie nadpisuje jego transformu. To nie jest dowod implementacji Beamdog. |
| Dekompilacja Aurora `FUN_00a5d758` | Parser rozpoznaje osobno `length`, `transtime` i `animroot`; tracki kluczy sa niezaleznym kontraktem noda. W tym offline trace nie ma warunku nakazujacego klucz dla statycznego aliasu. | Nie ma podstaw do generowania sztucznych controllerow tylko dlatego, ze stan nazywa sie `cpause1`. |
| NeverBlender / NWN Wiki | Instrukcja importu animacji omawia dopasowanie armatury i pozy domyslnej podczas retargetingu ruchu, a nie wymog identity trackow dla nieruchomego modelu. | Material authoringowy nie dostarcza przeslanki dla zmiany M0. [Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations) |

### Jedyny rozstrzygajacy test -- odlozony do kontrolowanego runtime

Nie tworzyc nowego `rNN` ani nie podmieniac aktualnego packetu. Gdy istnieje
autoryzowana, hash-bound droga NWN, przygotowac **dwa testowe MDL** z tym samym
HAK, MOD, `appearance.2da`, TGA, placementem i resrefem:

1. **A:** aktualny MDL bez zmian.
2. **B:** tylko w root `m2a_m0p01` clipu `cpause1` dodac dwa linearne,
   stale tracki: position `(0,0,0)` oraz orientation identity na `t=0` i
   `t=length`. Alias `m2a_seg_1` pozostaje `0x21`, bez vertices/faces/MDX i
   bez trackow.
3. Przed runtime asercyjnie porownac own semantic readback: dozwolona roznica
   to wyłącznie dwa root controller streams w `cpause1`; zarejestrowac SHA
   obu MDL/HAK/MOD oraz capture po ustabilizowaniu idlu.

Interpretacja: rowna widocznosc A/B odrzuca hipoteze; widocznosc tylko B
otwiera osobny blad profilu `controllerless static alias`; brak widocznosci w
obu wyklucza ja jako wystarczajaca przyczyne. Ten test jest runtime-only; nie
zostal teraz uruchomiony.

W tym przebiegu uzyto wylacznie odczytu dekompilacji Aurora, xoreos,
NeverBlender/Wiki oraz lokalnego packetu i testu. Aurora, NWN i INI pozostaly
nietykane.

## 44. P0: naglowek local-animation musi miec `type=5` pod `+0x6c`; obecny writer emituje zero (2026-07-21)

### Werdykt

**To jest potwierdzony, wspolny blad strukturalny emitera i najwazniejszy
pozostaly kandydat export/runtime dla aktualnego M0.** W lokalnym, read-only
corpusie `cep3_core1.hak` wszystkie `24 707` osiagalne binary local-animation
headers z `1 182` modeli maja ten sam `type=5`; nie jest to juz wniosek z
pojedynczego `c_squirrel`. Kazdy binary local animation ma pelny naglowek
`0xc4` B; jego poczatkowy prefix `GeometryHeader` ma `0x70` B. Pole
`GeometryHeader+0x68..+0x6b` pozostaje nieznanym 4-bajtowym polem, ale pierwszy
bajt kolejnego slowa, `+0x6c`, jest odrebnym `type`; trzy dalsze bajty
`+0x6d..+0x6f` sa paddingiem. Nasz
`emit_animations()` zapisuje name, root, node count, length, transition i
`animroot`, lecz pomija `+0x6c`. Poniewaz core jest inicjalizowany zerami,
wszystkie siedem klipow lokalnych M0, w tym `cpause1`, dostaje obecnie
`type=0` zamiast native `type=5`. Potwierdza to bezposredni odczyt aktualnego
hash-bound MDL `proof-output/m0-direct-resolver-gate-20260721/generated/m2a_m0p01.mdl`:
`cappear`, `cpause1`, `cwalk`, `crun`, `ca1slashl`, `cdamagel` i `cdead` maja
kolejno `type=0` i padding `00 00 00`.

Dotyczy to **rowniez statycznego M0**: jego widoczna w r21 geometria bazowa
nie zmienia faktu, ze model zawiera siedem lokalnych stanow, ktore runtime moze
materializowac przy spawnie, idle lub ruchu. Nie jest to jednak jeszcze dowod,
ze samo `type=0` wywolalo brak kontrolowanego proofu NWN; brak takiego proofu
pozostaje `proofCompleteness=missing`.

### Lancuch dowodowy

| Zrodlo | Fakt | Granica wniosku |
| --- | --- | --- |
| Aurora decomp `C:\\Projects\\New Folder\\export\\decompiled_all.c`, `FUN_00a3b874` -> `FUN_00a3ee64` -> `FUN_00a3ed90` | Loader tworzy obiekt stanu z binary headera. `FUN_00a3ee64` kopiuje `length` (`+0x70`), `transition` (`+0x74`) i `animroot` (`+0x78`), a `FUN_00a3ed90` kopiuje nazwe, root oraz **dokladnie jeden bajt** z `+0x6c`; nie kopiuje `+0x68` ani trzech wysokich bajtow slowa `+0x6c`. | To jest bezposredni dowod, ze `+0x6c` nie jest bezpiecznie calym „opaque u32” w runtime Aurory. Dekompilacja nie nadaje nazwy liczbie `5`. |
| xoreos `src/graphics/aurora/model_nwn.cpp`, `readAnimBinary()` | Po `name` i `root/count` loader omija `24 + 4` B (`+0x50..+0x6b`), czyta `uint8_t type`, omija 3 B paddingu, potem czyta `length` i `transTime`. | Niezaleznie potwierdza granice pola i padding; xoreos nie jest oracle'em implementacji Beamdog. |
| Native binary CEP `c_squirrel`, HAK `C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\cep3_core1.hak`, id `6396`, MDL SHA-256 `fce071fcebc04d0ffce33b6ef48891af615e70dd402120554dcc963cca36f78b` | Pierwszy stan `ca1slashl` pod core offsetem `404` ma `+0x68 = 0x9D529E65`; bytes `+0x6c..+0x6f = 05 7F 00 00`, czyli `type=5` oraz native padding `7F 00 00`. | Native padding nie jest wzorcem do kopiowania: Aurora go nie kopiuje, a wlasny output ma pozostac deterministyczny. Jeden witness sam nie definiuje pelnej taksonomii type. |
| Bounded read-only sweep `cep3_core1.hak` | Z `3517` MDL type `2002`, `1182` binarnych modeli ma lokalne animacje. Odczytano `24 707` naglowkow, ktore przeszly granice `core`/tablicy wskaznikow; histogram `+0x6c` wynosi dokladnie `5:24707`. Pozostale `87` zasobow type `2002` to ASCII MDL z magic/comment `# Ex` (64), `#Exp` (11) albo `#MAX` (12), wiec nie maja binary headera i nie weszly do histogramu. | To jest mocny fakt corpusowy dla naszej direct-creature sciezki, lecz nie uniwersalna specyfikacja wszystkich mozliwych typow MDL lub przyszlych wersji engine. |
| NeverBlender / [NWN Wiki: Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations) | Dokumentacja authoringu ostrzega, ze animacje wyeksportowane z NeverBlendera czesto zawodza przez naglowki; manual NeverBlendera opisuje ASCII clipy przez name/root/transition i nie eksponuje binary `type`. | Wspiera potrzebe osobnej walidacji binary emitera, ale nie definiuje liczby `5` ani nie jest zrodlem ABI. |
| Aktualny writer/parser/test | `write_binary_mdl.rs:emit_animations()` nie wykonuje zapisu pod `animation.header + 0x6c`. Parser zwraca cale `runtime_6c: u32`, a `semantic_readback.rs` i `tests/mdl_writer.rs` obecnie oczekuja zera. Ich zgodnosc jest wiec samopotwierdzajacym bledem. | Wskazuje minimalny, kontrolowany scope poprawki; nie wymaga zmiany root, controllerow, tekstur, geometrii ani `appearance.2da`. |

### Granica potwierdzenia lifecycle

Trace `FUN_00a3b874` doprecyzowuje, ze Aurora alokuje obiekt local state,
wywoluje `FUN_00a3ee64` (a wiec kopiuje `+0x6c`), nastepnie kopiuje events i
materializuje root state tree przez `FUN_00a3b994`. Jest to silniejszy fakt niz
sam odczyt parsera: byte dociera do konkretnego obiektu stanu, nie jest
pominietym paddingiem pliku.

Nie odnaleziono jednak w przejrzanej bezposredniej sciezce warunku lub switcha,
ktory nadaje liczbie `5` nazwe gameplayowa albo dowodzi, ze `0` samo powoduje
niewidocznosc. Wniosek implementacyjny pozostaje waski: writer musi emitowac
native `5`; twierdzenie o skutku renderowym wymaga osobnego runtime A/B.

### Minimalna poprawka -- test najpierw

1. Najpierw zmienic test readbacku writera: dla kazdego wlasnego local clipu
   oczekiwac `runtime_68 == 0` oraz `runtime_6c == 5`; dodatkowo sprawdzic
   surowe bytes `+0x6d..+0x6f == [0, 0, 0]` dla deterministycznego paddingu.
   Test M0 ma jawnie objac wszystkie 7 jego clipow.
2. Dodac negatywna mutacje do testu semantycznego: nadpisac tylko
   `animation.header + 0x6c` wartoscia `0` i wymagac diffa
   `animations[0].header`. Mutacja `+0x68` pozostaje osobnym testem pola
   opaque.
3. Dopiero potem w `emit_animations()` dodac pojedynczy zapis
   `write_u32(core, animation.header + 0x6c, 5)?;`. Nie zapisywac
   `0x00007F05`: trzy wysokie bajty maja pozostac zerowym paddingiem naszego
   outputu.
4. Uscislic parser/readback: tymczasowo `runtime_6c == 5` jako surowe `u32`
   jest wystarczajace do minimalnej poprawki; pozniejszy refactor moze
   rozdzielic `animation_type: u8` od trzech bajtow paddingu. Nie wolno nadal
   nazywac calego slowa „opaque runtime field”.

Po zaakceptowanej zmianie jedyny dopuszczalny runtime krok to proof na tym
samym hash-bound M0 packetcie po wymianie wyłącznie tego MDL i aktualizacji
manifestu/hashy. Nie tworzyc nowego `rNN`, nie zmieniac Appearance `848`,
MODELTYPE, RACE, root node, trackow ani tekstur w ramach tego testu. Aurora,
NWN i INI nie byly uruchamiane ani modyfikowane w tym przebiegu.

### Aktualizacja stanu implementacji -- 2026-07-21 (offline)

Powyższa diagnoza dotyczy hash-bound artefaktu r21, a nie aktualnego stanu
drzewa roboczego. W aktualnym snapshotcie `emit_animations()` **juz zapisuje**
`write_u32(core, animation.header + 0x6c, 5)`, czyli bytes
`05 00 00 00`. Wlasny parser nie nazywa juz calego slowa `runtime_6c`:
raportuje osobno `animation_type: u8` pod `+0x6c` oraz
`animation_type_padding: [u8; 3]` pod `+0x6d..+0x6f`. Semantic readback
wymaga `animation_type == 5` i paddingu `[0, 0, 0]`, a test
`static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice`
sprawdza wszystkie siedem clipow M0. W tym przebiegu przeszly dwa waskie testy:

1. `cargo test -p m2a-core --test model_pipeline static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice --quiet`;
2. `cargo test -p m2a-core --test model_pipeline static_meshy_m0_uses_the_toolset_visible_appearance_prefix --quiet`.

Wynik jest pozytywny tylko dla source/readbacku: stary MDL r21 o SHA-256
`cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5` nadal
ma `type=0`, a wiec nie jest artefaktem tej poprawki i nie moze byc jej
runtime proofem. Nalezy wygenerowac ponownie ten sam packet logiczny,
zarejestrowac nowe hashe MDL/HAK/MOD i dopiero wtedy wykonac kontrolowane
runtime A/B. Nie zmieniac przy tym `Appearance_Type=848`, `MODELTYPE=S`,
`RACE=m2a_m0p01`, klasyfikacji, rootow, controllerow ani tekstury.

Precyzyjny test negatywny ABI **juz istnieje** w module testowym
`write_binary_mdl.rs`, gdzie dostepne sa prywatne `expected_readback()` i
`semantic_diff()`. Buduje on lokalny clip, zapisuje zero pod
`payload[12 + clip.offset + 0x6c..+0x70]` i wymaga dokladnie diffa
`animations[0].header`. Poniewaz baseline ma juz padding `00 00 00`, ta
operacja materialnie zmienia tylko byte type `5 -> 0`. Osobny test parsera
mutuje `+0x6c` do `7` i potwierdza `animation_type == 7`; oddzielnie mutuje
`+0x6d` i potwierdza pierwszy byte paddingu. To zamyka source/readback dla
podzialu pola na `u8 + 3 B`, ale nadal nie jest testem renderera.

To jest test regresji serializacji, nie test renderera. Dekompilacja pokazuje,
ze Aurora kopiuje ten byte do obiektu stanu, a xoreos niezaleznie go odczytuje;
nie znaleziono natomiast switcha dowodzacego, ze sama wartosc `0` powoduje
niewidocznosc. Dlatego P0 jest **zamkniety jako blad source/readbacku**, lecz
NWN runtime nadal ma status `missing`.

## 45. Corpus base-mesh: `meshType=3` dla kazdej niepustej geometrii (2026-07-21)

### Werdykt

**P0 M4 `meshType=0 -> 3` ma teraz szeroki, binarny dowod corpusowy i jest
zamkniete w aktualnym source/readbacku.**
Read-only przejscie po base node trees `cep3_core1.hak` odczytalo `3430`
binarnych MDL (pozostale `87` wpisow type `2002` to ASCII). Daje ono `77 770`
mesh nodes, z czego `77 552` maja jednoczesnie niezerowe `vertexCount` i
`facesCount`. Histogram dla realnej geometrii wynosi dokladnie `3:77552`; w tym
`1437` niepustych skinmeshow rowniez ma `3`. Wszystkie `218` obserwacji
`meshType=0` wystapily w mesh nodes bez vertices i bez faces.

Wniosek jest waski: profil direct-creature, ktory emituje rzeczywiste trójkatne
faces, musi zapisywac `3`. Nie wolno z tego zrobic globalnego odrzucenia `0` w
readerze, bo corpus pokazuje je w pustych node'ach. Ten fakt **nie** dowodzi
jeszcze, ze sama zmiana naprawi runtime H1 ani nie zastepuje kontrolowanego A/B
w NWN.

### Lancuch dowodowy

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp `C:\\Projects\\New Folder\\export\\decompiled_all.c`, `FUN_00a62954`, `FUN_00a5e54c` | Kod budujacy indeksy z trojek wpisuje `3` pod `mesh+0x224`; draw path przekazuje to pole dalej do renderera. | Dekompilacja wiąże pole z topologia/draw, ale utracone typy nie nadaja enumowi publicznej nazwy. |
| xoreos `model_nwn.cpp`, `readMesh()` | Czyta pole jako `triangleMode`; komentarz identyfikuje `3` jako Triangle i `4` jako TriStrip. | Niezalezna implementacja, nie oracle Beamdog. |
| NeverBlender Manual 2.0 | Trimesh jest bazowym meshem i eksporter trianguluje faces; render jest oddzielna flaga. | Potwierdza semantyke authoringu trojkatow, nie binary enum. [Manual](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) |
| Bounded corpus CEP opisany wyzej | Wszystkie `77 552` niepuste meshe i `1437` niepustych skinmeshow maja `meshType=3`; `0` wystepuje tylko `218` razy i wylacznie w pustych node'ach. | Jeden HAK nie specyfikuje wszystkich przyszlych typow prymitywow, lecz bezposrednio obejmuje native direct-creature geometry. |
| Aktualny writer i own readback | Zarowno `M0StaticRigidNativeV1`, jak i `M4DirectCreatureExtended64V1` wybieraja `3`; writer zapisuje wartosc pod `mesh+0x224`. Test M4 rigid oczekuje `mesh_type=3`, a negatywna mutacja extended64 skin `3 -> 0` daje dokladnie semantic diff `nodes[3].profileDefaults`. | Zamyka regression source/readback, nie dowodzi skutku wizualnego w kliencie. |

### Aktualny gate M4 i granica runtime

1. Test writer/readbacku M4 rigid oczekuje `mesh_type == 3`; semanticzny test
   extended64 skin odrzuca mutacje jedynie `mesh+0x224: 3 -> 0`.
2. Referencyjny parser pozostaje tolerancyjny dla `0`, aby nadal czytac native
   puste nodes.
3. Nie wykonywac kolejnego A/B samego `0 -> 3`: aktualny writer juz ma `3`.
   Kontrolowany runtime M4 nadal moze sprawdzic caly aktualny output, ale nie
   wolno przypisac mu skutku tej juz usunietej roznicy ani przy okazji zmieniac
   skin weights, MDX, rootow, tekstur czy
 `appearance.2da`.

Aurora, NWN i INI nie byly uruchamiane ani modyfikowane w tym przebiegu.

## 46. R21 M0: brak controllerow nie jest bledem pozy bazowej; wdrozony HAK zawiera stare `type=0` (2026-07-21)

### Werdykt

**Nie ma kolejnej poprawki `classification`, root node, `animroot` ani alpha dla dokladnego M0 r21.** Ten kandydat jest juz widoczny w Toolsecie jako single rigid silhouette. Read-only audit jego faktycznie podpietego HAK-a pokazuje jednak inna, pojedyncza rozbieznosc: wszystkie siedem local-animation headers ma `type=0` pod `+0x6c`, podczas gdy aktualny writer i source/readback wymagaja `type=5`.

To dotyczy statycznego M0, bo nawet nieruchomy creature ma local states, ktore runtime moze zmaterializowac przy spawnie lub idle. Nie jest to jeszcze dowod, ze `0` powoduje niewidocznosc w NWN: r21 mial zly placement, a r26 z `type=0` nie byl widoczny mimo poprawionego fixture. Jest to natomiast dowod, ze zadna z tych obserwacji nie testowala aktualnego outputu r27 z `type=5`.

Rownoczesnie r21 ma w kazdym z siedmiu stanow root `0x01` i oba naglowki controller arrays `pointer, used, allocated = 0,0,0`; dotyczy to takze `cpause1`. Skoro ten sam hash-bound M0 jest widoczny w Toolsecie, brak dwoch identity tracks nie wyjasnia braku pozy bazowej. Aktualne source emituje takie tracki tylko w `cpause1`, ale bez runtime A/B sa one hipoteza, nie potwierdzona poprawka. Nie nalezy ich ani dodawac do kolejnych stanow, ani usuwac w ramach tego samego zadania.

### Tozsamosc i odczyt in-place

| Element | Fakt odczytany bez zapisu | Konsekwencja |
| --- | --- | --- |
| HAK r21 | `C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_m0r21.hak`; 13 124 342 B; SHA-256 `25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6`. | Jest to konkretny HAK, nie analogiczny artifact z `proof-output`. |
| Wpis ERF | key index `1`, `resourceId=1`, `resref=m2a_m0p01`, type `2002`; payload offset `388450`, size `152936` B. | Weryfikacja dotyczy wlasciwego binary MDL rozwiazywanego przez `RACE=m2a_m0p01`. |
| Payload MDL | SHA-256 `cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5`; animation table `232`, count `7`. | Hash zgadza sie z wczesniejszym stale-artifact witness, lecz teraz zostal potwierdzony wewnatrz faktycznego HAK-a r21. |
| Siedem headers | `cappear`, `cpause1`, `cwalk`, `crun`, `ca1slashl`, `cdamagel`, `cdead`: kazdy `type=0`, padding `00 00 00`. | r21 nie sprawdza wdrozonej korekty ABI `type=5`. |
| Rooty stanow | Kazdy local state: root flag `0x01`, controllers keys `0,0,0`, controller data `0,0,0`. | Negatywny wynik dla hipotezy, ze identity controller jest konieczny do narysowania pozy bazowej w Toolsecie. |

### Triangulacja zrodel

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp `FUN_00a3b874 -> FUN_00a3ee64 -> FUN_00a3ed90` | Loader przenosi do local-state name, root, `length`, `transition`, `animroot` i dokladnie byte `+0x6c`. | Nie znaleziono switcha dowodzacego, ze `0` sam powoduje invisibility. |
| xoreos `Model_NWN::readAnimBinary()` | Czyta osobno `uint8_t type` po `24 + 4` B oraz omija trzy bajty paddingu przed length/transition/animroot. | Niezaleznie potwierdza granice pola, ale xoreos nie jest oracle'em Beamdog. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Root Aurora musi miec nazwe pliku modelu; `Character` jest klasyfikacja dla creatures/placeables, a `Render` jest osobna flaga mesha. | Potwierdza authoringowy sens obecnego root/classification/render; nie definiuje binary `type=5` ani wymogu identity tracks. |
| Aktualny source/readback | `emit_animations()` zapisuje `05 00 00 00` pod `+0x6c`; test M0 wymaga type `5` dla 7 clipow, a test semantyczny odrzuca mutacje tylko do zera jako `animations[0].header`. | To zamyka serializacje w aktualnym drzewie, ale nie podmienia r21 ani nie jest proofem gry. |

### Najmniejszy test i granica kolejnego kroku

W tym cyklu wykonano dwa offline gates i oba przeszly:

1. `cargo test -p m2a-core --test model_pipeline static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice --quiet` — wymaga m.in. 7 clipow z `type=5`, prawidlowym `animroot` i obecnym profilem M0.
2. `cargo test -p m2a-core --lib semantic_animation_mutations_name_opaque_budget_and_padding --quiet` — zmiana tylko `+0x6c..+0x6f` do zera daje dokladnie diff `animations[0].header`.

Minimalny **przyszly runtime** test nie jest nowym eksperymentem root/alpha: ma najpierw zwiazac nowy, autoryzowany packet z hashem MDL/HAK/MOD, `Appearance_Type=848`, `MODELTYPE=S`, `RACE=m2a_m0p01`, tekstura i placementem r21, a nastepnie sprawdzic go w NWN. Nie wolno przy tym laczyc zmiany `type=5` z nowa klasyfikacja, innym rootem, innym `animroot`, alpha, tekstura lub dodatkowymi controllerami. Dopiero wynik runtime rozstrzyga, czy usuniety `type=0` byl przyczyna, czy jedynie konieczna korekta ABI.

Aurora, NWN i INI nie byly uruchamiane ani modyfikowane w tym cyklu.

## 47. R21 M0: base mesh i blok MDX przechodza znane bramki offline (2026-07-21)

### Werdykt

**Nie ma offline podstaw do zmiany binary base-mesh/MDX aktualnego r21 M0.** Dokladny payload `m2a_m0p01` z HAK-a r21 ma kompletna, spojna geometryczna sciezke: `render=1`, mesh `0x21`, `meshType=3`, niepuste faces, vertices, UV, normals, vertex colors i raw indices. Wszystkie wskazane strumienie mieszcza sie w zadeklarowanym MDX, a lista raw indices jest bajtowo rowna topologii faces.

`startMdx=0` nie jest tu bledem: odczytana dekompilacja odrzuca dopiero pare `vertexCount==0 && startMdx==0`, a r21 ma `vertexCount=2380`. Jedna czesc native gate pozostaje z natury poza offline MDL: bit runtime instance pod `param+0x7a`. Nie wolno mylic jej z kolejnym polem emitera ani twierdzic, ze sam ten audit jest proofem draw calla NWN.

### Odczyt hash-bound payloadu

Payload ma SHA-256 `cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5`.

| Warstwa | Odczyt | Wynik |
| --- | --- | --- |
| File layout | `p_start_mdx=57828`, `size_mdx=95096`; `12 + 57828 + 95096 = 152936` B payloadu. | Core/MDX pokrywaja dokladnie caly plik. |
| Base mesh | `flags=0x21`, `render=1`, `meshType=3`, `startMdx=0`, `vertexCount=2380`, `textureCount=1`. | Zgadza sie z direct rigid Trimesh, nie z pustym aliasem animacji. |
| Faces | core pointer `7612`, `1569` faces, cala tablica w core; `4707` face indices, min/max `0..2379`, zero poza vertex range. | xoreos nie odrzucilby mesha przez puste faces lub poza-zakresowe indeksy. |
| MDX streams | vertices `0..28559` (28 560 B), UV0 `28560..47599` (19 040 B), normals `47600..76159` (28 560 B), colors `76160..85679` (9 520 B), indices `85680..95093` (9 414 B). | Kazdy zakres jest w `0..95095`; koncowe 2 B sa paddingiem. |
| Raw indices | `4707` entries, min/max `0..2379`; liczba i kolejnosc sa identyczne z face indices. | Nie ma rozjazdu core topology kontra MDX index stream. |

### Triangulacja zrodel

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp `FUN_00a5e508`, `FUN_00a5e54c` | Draw eligibility wymaga aktywnego runtime state, runtime mesha, `render != 0` oraz nie obu zer: vertex count i start MDX; dalej przekazuje `mesh+0x224` do draw path. | Offline payload potwierdza trzy file-sourced warunki, ale nie stan runtime instance. |
| xoreos `ModelNode_NWN_Binary::readMesh()` | Rezygnuje z budowy render-data, gdy vertex count, face count albo face pointer jest zerowy; czyta vertices z MDX i faces z core. | Niezalezny reader, nie oracle Beamdog, lecz jego wszystkie odczytywane preconditions spelnia r21. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Trimesh jest podstawowym meshem Aurora, a exporter trianguluje faces. | Potwierdza znaczenie trojkatnej topologii authoringu, nie binary layout MDX ani runtime behaviour. |
| Aktualny product test | `m0_runtime_mesh_eligibility_requires_triangle_topology_and_nonempty_faces` odrzuca tylko `meshType=0`, puste faces oraz rozjazd raw/core indices; `static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice` sprawdza kompletny output. | Oba testy sa source/readback, nie wykonaniem r21 w grze. |

### Minimalny gate i decyzja

Nie zmieniac emitera M0, MDX, `startMdx`, vertex colors ani indeksow na podstawie wyniku r26. W przyszlym env-gated P-REF nalezy odczytac in-place HAK o powyzszym hashu i asercyjnie zweryfikowac: dokladny payload hash, exact file layout, wszystkie piec raw ranges, `faceIndices == rawIndices` i indeksy w `0..vertexCount-1`. Test ma pozostac read-only i nie wolno z niego kopiowac payloadu do fixture lub CI.

Wynik jest negatywny dla hipotezy „r21/r26 znikaja przez uszkodzony MDX/base mesh”. Toolset r21/r26 pozostaje `visible`; r26 NWN jest `not_visible`, lecz `proofCompleteness` runtime jest `missing`. R27 jest jedynym otwartym A/B i pozostaje `not_tested`.

Aurora, NWN i INI nie byly uruchamiane ani modyfikowane w tym cyklu.

## 48. R21 M0: texture resolver i material nie ukrywaja aktualnej geometrii (2026-07-21)

### Werdykt

**Nie zmieniac `texture0`, TGA, alpha, transparency ani nie dodawac MTR/TXI dla r21 M0.** Exact HAK ma tylko trzy zasoby: `appearance` (2017), `m2a_m0p01` (2002) i `m2a_m0t01` (3). Mesh wskazuje `texture0=m2a_m0t01`, wiec ref jest obecny pod tym samym resrefem i typem. Nie ma w pakiecie zadnego MTR, TXI, DDS ani drugiej tekstury, ktore moglyby nadpisac material wewnatrz tego HAK-a.

| Element | Odczyt hash-bound r21 | Konsekwencja |
| --- | --- | --- |
| TGA | SHA-256 `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`; type `2` (uncompressed truecolor), `2048x2048`, `24` bpp, descriptor `0`, poprawny footer TGA 2.0. | Plik nie ma kanalu alpha, ktory moglby wyzerowac widocznosc. |
| Kolory pikseli | Kazdy kanal BGR ma zakres `0..255`; srednie `76.829/88.202/83.926`; obraz nie jest ani caly czarny, ani caly bialy. | To nie jest proof renderera, ale odrzuca banalny przypadek pustej/zerowej mapy. |
| Material mesha | diffuse i ambient `(1,1,1)`, specular `(0,0,0)`, shininess `1`, shadow `1`, beaming `0`, render `1`, transparency `0`, renderHint `0`, tileFade `0`. | Lokalny MDL nie ustawia flagi wylaczajacej render lub lokalnej transparencji. |
| Inventory HAK | Trzy wpisy i tylko trzy: appearance/MDL/TGA. | Pakiet sam nie wprowadza material override; globalny cache lub zewnetrzny override pozostaje osobna, nieodczytana zmienna runtime. |

### Triangulacja zrodel

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp, ASCII parser mesh | Rozpoznaje osobno `texture0` pod mesh `+0xe8`, `transparencyhint` pod `+0xe0` i `render` pod `+0xdc`. | Potwierdza, ze te parametry sa rozdzielone; nie jest binarnym trace'm wyboru TGA w kliencie EE. |
| xoreos `ModelNode_NWN_Binary::readMesh()` | Odczytuje render/transparency/textures; dla niepustego mesha laduje tekstury po utworzeniu render-data. | Niezalezny reader, nie oracle Beamdog; spelniony jest jego lokalny kontrakt `textureCount=1` i `texture0`. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | `Render` kontroluje rysowanie obiektu, a transparency hint jest porzadkiem transparentnych obiektow i wedlug manuala nie dotyczy dynamic objects. | Wspiera pozostawienie `render=1`/hint `0`; nie definiuje TGA/DDS resolvera runtime. |

### Minimalny gate i granica runtime

Aktualny test M0 juz odrzuca model wskazujacy na teksture nieobecna w HAK-u (`M0-RUNTIME-CONTRACT-MESH-TEXTURE-RESREF-MISMATCH`). Przyszly env-gated P-REF powinien dodatkowo hash-bound odczytac TGA i wymagac: type `2`, `24` bpp, oczekiwane wymiary, `texture0` zgodne z ERF key oraz brak dodatkowych material resources w jednolitym HAK-u. Nie zamieniac RGB na RGBA ani nie generowac MTR/TXI jako zgadywanej poprawki.

Wynik jest negatywny dla hipotezy, ze **wewnetrzny** texture/material packet r21/r26 ukrywa M0. Nie wyklucza zewnetrznego cache/override, ktory wymaga kontrolowanego packetu NWN r27. R26 ma `modelVisibility=not_visible`; runtime `proofCompleteness` pozostaje `missing`, a r27 jest `not_tested`.

Aurora, NWN i INI nie byly uruchamiane ani modyfikowane w tym cyklu.

## 49. R21 M0: `MODELTYPE=S` jest wlasciwa klasyfikacja single-creature, nie kandydatem do zmiany na `P` (2026-07-21)

### Werdykt

**Nie zmieniac `MODELTYPE` z `S` na `P`, nie zmieniac `RACE=m2a_m0p01` i nie wprowadzac kolejnego appearance row.** Odczyt r21 wiaze fizyczny wiersz `848` z `LABEL=M2A_M0_MESHY_RIGID`, `MODELTYPE=S` i `RACE=m2a_m0p01`; ten dokladny kandydat jest widoczny w Toolsecie. `S` oznacza wlasnie prosty creature z pojedynczym MDL i pojedyncza tekstura, a nie model parts-based. Zmiana na `P` zmienilaby kontrakt resolvera na wieloczesciowy profil, ktorego M0 nie realizuje.

Wniosek nie usuwa otwartego watku runtime: typ `S` wykorzystuje creature aliases, w tym `cwalk` i `crun`. R21 ma siedem local states, lecz stale naglowki `type=0` opisane w §46; obecny writer wymaga `type=5`. To jest oddzielna niezgodnosc ABI wdrozonego HAK-a, **nie** uzasadnienie zmiany `S`/`P` ani dodania identity controllerow. Bez hash-bound packetu NWN nie wiadomo, czy gra odrzuci r21 na etapie local state, czy uruchomi go poprawnie.

### Triangulacja zrodel

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp `FUN_00463d54` | Dla wybranego appearance pobiera kolumne `MODELTYPE` przez `FUN_00af3568`; brak typu daje blad „Invalid Appearance / Model Type for appearance not found”. Po sukcesie rozpoznaje w tej funkcji tylko znak `P`, aby wlaczyc dodatkowa walidacje czesci/phenotype. | To sciezka Toolsetu, a nie trace renderera klienta NWN; nie dowodzi kompletnej listy runtime aliases. |
| xoreos `Model_NWN::loadBinary()` + `readAnimBinary()` | Najpierw materializuje base state, potem czyta wszystkie local-animation offsets i kazdy header jako osobny state; type to osobny byte po `24 + 4` B. | Niezalezny reader, nie oracle Beamdog; potwierdza rozdzielenie base geometry od lokalnych stanow. |
| [NWN Wiki: appearance.2da](https://nwn.wiki/spaces/NWN1/pages/38174941/appearance.2da) | `RACE` jest resrefem modelu; `S` to single MDL z jednym TGA/DDS i creature animations jak `crun`/`cwalk`, podczas gdy `P` jest modelem z czesciami. | Dokumentacja spolecznosciowa, ale jej semantyka zgadza sie z r21 oraz dekompilacja resolvera. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Model ma jeden Aurora Root o nazwie pliku, `Character` jest klasyfikacja dla creature/placeable, a animacje maja nazwe i zakres. | Opisuje authoring ASCII, nie binarny `animation type` ani decyzje renderera NWN. |

### Minimalny test i decyzja implementacyjna

Negatywny gate juz istnieje w `static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice`: zmienia tylko wiersz M0 z `MODELTYPE=S` na `P`, zachowuje HAK oraz kontrakt wzajemnie spojne i wymaga bledu `M0-RUNTIME-CONTRACT-APPEARANCE-MODELTYPE-INVALID` pod `appearance.rows[1].MODELTYPE`. Ten test nalezy zachowac; nie ma minimalnej poprawki source dla klasyfikacji, bo obecne `S`/`RACE` sa zgodne z profilem single rigid M0.

Najmniejszy przyszly test runtime pozostaje jeden: na autoryzowanym packetcie NWN uruchomic dokladny r27 z tym samym hash-bound `MOD/HAK/MDL/TGA`, Appearance `848`, `S`, `m2a_m0p01` i placementem `(10,14.5,0)`. Nie laczyc go ze zmiana na `P`, innym rootem, alpha, texture ani controllerami. Wynik offline jest zatem negatywny dla hipotezy „niewidocznosc wynika z blednej klasyfikacji `MODELTYPE=S`”; r26 jest `not_visible` przy niepelnej przyczynowosci, a r27 `not_tested` / `proofCompleteness=missing`.

Aurora, NWN i INI nie byly uruchamiane ani modyfikowane w tym cyklu.

## 50. R21 M0: `TemplateResRef` bazowego UTC nie nadpisuje `Appearance_Type` instancji; pozostaje mala luka w verifierze generowanego MOD (2026-07-21)

### Werdykt

**`TemplateResRef=nw_dwarfmerc001` w r21 nie jest bledem eksportu modelu ani powodem do zmiany root/classification/MDL.** Read-only odczyt dokladnego `m2a_m0r21.mod` (SHA-256 `d9286d469e049f032cf70e79b763ca4d314cae13ca414b8230b6b24bb81e49ec`) wykazuje jeden wpis `Mod_HakList`: `m2a_m0r21`. Powiazany HAK (SHA-256 `25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6`) ma wlasnie trzy zasoby: `appearance`/2017, `m2a_m0p01`/2002 i `m2a_m0t01`/3. Jedyna instancja GIT obszaru `m2a_m0a21` ma `TemplateResRef=nw_dwarfmerc001` **i wprost** `Appearance_Type=848`.

To jest poprawny rozdzial template od instancji. Dla loadera w stylu NWN template dostarcza domyslne dane creature, a GIT nastepnie naklada instancyjne `Appearance_Type`; r21 jest widoczny w Toolsecie wlasnie przy tym lancuchu. Nie wolno zamieniac template na `m2a_m0p01`: ta nazwa jest resrefem MDL z kolumny `appearance.RACE`, nie resrefem creature UTC.

Znaleziono jednak waska **luke offline, a nie dowod blednego r21**: `build_binary_m0_vertical_slice_module_v1()` tworzy wlasny UTC o resrefie `nw_dwarfmerc001` i wartosci `Appearance_Type=848`, ale jego szczegolowy `validate_binary_m0_vertical_slice_module_readback()` sprawdza tylko GIT. Weryfikuje obecne `TemplateResRef`, GIT `Appearance_Type`, pozycje i HAK-list, lecz nie parsuje rownoleglego UTC/2027, aby asercyjnie wymagac tego samego appearance. Konstruktor obecnie generuje zgodne dane; gate nie zabezpiecza jednak regresji, w ktorej UTC stalby sie rozbiezny.

### Triangulacja zrodel

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp `FUN_00532174` | Czyta z GFF odrebnie `Appearance_Type` (`FUN_00aeedc4 -> FUN_00512810`) oraz `TemplateResRef`; odpowiedni writer zapisuje `Appearance_Type` jako `ushort`. | Nazwy struktur/decyzja kolejnosci blueprint kontra instancja nie sa w tej dekompilacji w pelni zrekonstruowane. |
| xoreos `engines/nwn/creature.cpp` | Najpierw laduje UTC z `TemplateResRef`, nastepnie wykonuje `loadProperties(*blueprint)`, po czym `loadProperties(instance)`; to drugie nadpisuje `_appearanceID`. `loadModel()` wybiera `appearance` po `_appearanceID`, a dla `MODELTYPE != P` laduje `RACE`. | Niezalezna implementacja, nie oracle Beamdog; jest jednak bezposrednia i zgodna z widocznym r21 w Toolsecie. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Root MDL musi miec nazwe pliku modelu; klasyfikacja i animacje sa danymi MDL, nie polami creature GIT/UTC. | Manual nie dokumentuje merge GIT/UTC; wyklucza jedynie mylenie `TemplateResRef` z resrefem MDL. |
| Aktualny generator/readback | Binary vertical-slice zawiera lokalny UTC/2027 pod `nw_dwarfmerc001`, a jego konstrukcja przekazuje temu UTC ten sam `appearance_row`; specific readback nie odczytuje jednak tego zasobu. | Jest to sprawdzenie source/readback, nie proof runtime. |

### Minimalna poprawka i test, jezeli praca implementacyjna zostanie autoryzowana

Nie ma uzasadnionej zmiany r21 ani nowej iteracji modelu. Minimalna, niezalezna poprawka gate'a to rozszerzyc `validate_binary_m0_vertical_slice_module_readback()` o odczyt `archive.find(BINARY_M0_CREATURE_TEMPLATE_RESREF, UTC_RESOURCE_TYPE)`, parse GFF i wymaganie `utc.Appearance_Type == GffValueV1::Word(appearance_row)`. Rozbieznosc ma zwracac obecny typ bledu `M0-BINARY-VERTICAL-SLICE-SEMANTIC-DIFF` pod precyzyjna sciezka `binary_m0.utc.Appearance_Type`.

Najmniejszy test TDD: zbudowac binary vertical slice, odczytac jego GIT i UTC i asercyjnie wymagac obu wartosci `848`; nastepnie zmutowac tylko serialized UTC `Appearance_Type` do `847` i wymagac wskazanego bledu verifiera. Test nie moze zmieniac HAK, MDL, tekstury, rootow ani `MODELTYPE`. Do chwili takiej implementacji klasyfikacja statusu brzmi: r21 `modelVisibility` Toolset `visible`; `TemplateResRef` jako przyczyna niewidocznosci **odrzucony offline**; runtime NWN tego lancucha nadal wymaga kontrolowanego packetu (`proofCompleteness=missing`).

W tym cyklu nie uruchamiano Aurora ani NWN i nie modyfikowano INI.

## 56. I7: bounds/culling r21--r27 — negatywny wynik dla aktualnego static M0 (2026-07-21)

### Pytanie i wynik

Sprawdzony kandydat brzmial: **czy header AABB/radius lub transform drzewa
mogl calkowicie wyciac aktualny single static M0 w NWN, mimo malej sylwetki
widocznej w Toolsecie?** Wynik offline dla artefaktow r21/r27 jest
**negatywny**. Nie jest to dowod draw calla ani runtimeowego frustum-cullingu
Beamdog, ale eliminuje konkretna hipoteze, ze emitowana geometria M0 lezy poza
tym headerem, ktory sam writer wpisal do tego MDL.

Swiezy own-reader odczytal `r27` (`m2a_m0p01.mdl`, SHA-256
`bf7864f0ed541ed93c4075d167f4a2392eeb6ef37b4cdf444cbb74efd5731578`):

| Warstwa | X | Y | Z | Promien | Wynik |
| --- | --- | --- | --- | --- | --- |
| Model header | `[-5, 5]` | `[-5, 5]` | `[-1, 10]` | `7` | native-style staly header writera |
| Jedyny base trimesh `m2a_seg_1` | `[-0.54365003, 0.54365003]` | `[-0.31950998, 0.31950998]` | `[0, 1.892962]` | `1.8960401` | contained na wszystkich szesciu granicach |

W r27 root `m2a_m0p01` ma tylko identity `position=(0,0,0)` i
`orientation=(0,0,0,1)`, a mesh jest jego jedynym childem. Zatem jego local
mesh AABB jest rownoczesnie world AABB dla base state. Wczesniejszy own-read
r21 (MDL SHA-256 `cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5`)
podal te same granice mesha i headera. Roznica r26 -> r27 dotyczy wyłącznie
siedmiu byte values animation `type` (`0 -> 5`), nie geometrii ani bounds.

Nie interpretowac `radius=7` jako kuli zawierajacej kazdy rog header AABB:
sam header `[-5,-5,-1]..[5,5,10] / 7` tego nie spelnia. Semantyka tego pola w
kliencie pozostaje otwarta i nie jest dodatkowym gate'em.

### Triangulacja i korekta provenance

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp `FUN_00a5d69c` | Rozpoznaje tekstowe `radius`, `radiuskey` i `radiusbezierkey` dla controllera ASCII; zapisuje wartosc controllera pod `+0x58`. Nie znaleziono w tej funkcji predykatu model-header AABB/radius ani frustum cullingu. | Nie wolno opisywac tej funkcji jako potwierdzonego binary-header parsera lub dowodu algorytmu NWN. |
| xoreos `Model_NWN::loadBinary()` oraz `Model::createBound()` | Binary reader konsumuje model- i mesh-level min/max/radius, lecz `createBound()` buduje bound box z `createAbsoluteBound()` aktywnego drzewa nodow, a nastepnie go transformuje. | To niezalezna implementacja, nie oracle Beamdog; wzmacnia jedynie wymog liczenia finalnych transformow, ktory M0 spełnia. |
| NeverBlender `nvb_node.py`, `nvb_utils.py`, `nvb_ops.py` | Eksporter ASCII emituje pozycje/orientacje i tylko uniform `scale`; `getAuroraScale()` zwraca `1` dla skali niejednorodnej. Transform Helper aplikuje translacje i skale do calego modelu oraz jego animacji. | NeverBlender nie serializuje binary model-header AABB/radius. Jest zrodlem granicy authoringu: bounds trzeba liczyc po koncowych transformach, nie z surowego GLB. |
| Dostepna kopia `c_squirrel.mdl` | Zaczyna sie od `#NWmax MODEL ASCII` i `newmodel c_squirrel`; own binary inspector prawidlowo odrzuca ja jako niebinaryjna. | `c_squirrel` nie jest witnessem binary headera. Poprzednie sformulowanie o szesciu binarnych referencjach zostalo skorygowane w §14; zanim zostanie przywrocone, potrzebuje hashowanych binarnych wejsc i powtarzalnego odczytu. |

### Co (nie) implementowac

1. **Nie zmieniac** `classification`, roota, tekstury/alpha, skali ani
   `[-5,-5,-1]..[5,5,10] / 7` dla aktualnego r27. Taka zmiana mieszalaby
   negatywny wynik bounds z jedynym jeszcze kandydatem ABI (`animation_type=5`)
   i wymagalaby niedozwolonej nowej iteracji bez swiezego failure r27.
2. **Nie traktowac** obecnego stałego headera jako faktycznego rozmiaru modelu.
   `plan()` w `write_binary_mdl.rs` juz liczy world `model_min`, `model_max`
   i `model_radius`, ale odrzuca je na rzecz `DIRECT_CREATURE_MODEL_*`; nie
   raportuje ani nie gate'uje containmentu.
3. Prawidlowe przyszle hardening, a nie fix aktualnej niewidocznosci, to
   test-first preflight AABB: punkt dokladnie na kazdej granicy przechodzi;
   osobny mutant poza `min/max` kazdej z osi odrzuca writer przed MDL z
   `M4-NATIVE-ENVELOPE-OUTSIDE`, wskazujac segment i os. Test ma porownywac
   tylko szesc granic AABB, nie `radius=7`; header w przypadku pozytywnym musi
   pozostac bajtowo staly.

Ten minimalny test nie zostal w tym cyklu zaimplementowany, bo nie istnieje
fakt wskazujacy bounds jako przyczyne M0. Jest rozsadnym gate'em dla przyszlych
importow o duzych transformach; nie jest przepustka do r28 ani substytutem
hash-bound packetu runtime r27.

Aurora, NWN i INI nie byly uruchamiane ani modyfikowane w tym cyklu.

## 57. I8: `geometryType=2`, `classification=4`, `fog=1` — native creature header profile, nie blad static M0 (2026-07-21)

### Werdykt

**To jest negatywny wynik dla hipotezy, ze aktualny r27 znika przez model
classification, geometry type albo fog byte.** R27 ma dokladnie profil
binarnych native direct-creature models z kontrolowanego, read-only corpusu:
`geometryType=2`, `classification=4`, `fog=1`, `childModelCount=0`,
`[-5,-5,-1]..[5,5,10]` i `radius=7`. Nie zmieniac zadnego z tych byte'ow,
nie zmieniac `MODELTYPE=S` i nie tworzyc z tego A/B lub r28.

Wynik nie dowodzi, ze klient NWN narysowal r27: nadal brakuje
hash-bound runtime packetu. Dowodzi natomiast, ze nie ma podstaw, aby uznac
te konkretne wartosci headera za odstajace od rodziny natywnych creature.

### Read-only corpus i aktualny writer

Przebieg `cargo test -p m2a-core --test erf_reference_integration --quiet`
z `M2A_REFERENCE_CEP_HAK` wskazujacym lokalny `cep3_core1.hak` przeszedl
`1/1`. Own ERF reader zweryfikowal resource id, offset, byte length i SHA-256
szesciu MDL in-place; own MDL reader odczytal ich naglowki. Dodatkowy read-only
odczyt pierwszych `180` B kazdego payloadu dal ten sam profil:

| Binary native resref | `geometryType` | `classification` | `fog` | child models | Header AABB / radius |
| --- | ---: | ---: | ---: | ---: | --- |
| `c_kocrachn` | `2` | `4` | `1` | `0` | `[-5,-5,-1]..[5,5,10] / 7` |
| `c_phod_horror_b` | `2` | `4` | `1` | `0` | `[-5,-5,-1]..[5,5,10] / 7` |
| `c_phod_horror_p` | `2` | `4` | `1` | `0` | `[-5,-5,-1]..[5,5,10] / 7` |
| `c_nulltail` | `2` | `4` | `1` | `0` | `[-5,-5,-1]..[5,5,10] / 7` |
| `c_vampire_f` | `2` | `4` | `1` | `0` | `[-5,-5,-1]..[5,5,10] / 7` |
| `c_eye` | `2` | `4` | `1` | `0` | `[-5,-5,-1]..[5,5,10] / 7` |

R27 ma ten sam profil w fresh own-read. Aktualny writer emituje type `2`,
zapisuje byte `4` pod `core+0x72` i `1` pod `core+0x73`; semantic readback
wymaga odpowiednio `2`, `4` i `1`. Juz istniejacy
`minimal_rigid_writer_roundtrips_every_locked_semantic_and_exact_eof` sprawdza
`classification=4` i `fog=1`. Corpus-test weryfikuje dzis identyfikacje,
zakresy i parser, ale nie ma jeszcze osobnej asercji wspolnego profilu
headera z tabeli powyzej.

### Trzy wymagane zrodla i ich granice

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp `C:\\Projects\\New Folder\\export\\decompiled_all.c` | Audit znajduje parsery ASCII controllera i modelowego tekstu, lecz nie odtworzyl stabilnej, nazwanej mapy binary byte `classification=4` ani predykatu renderera dla tych trzech pol. | Nie wolno wymyslac znaczenia innych wartosci byte'ow ani twierdzic, ze dekompilacja dowodzi draw calla. Lokalny binarny corpus jest tu silniejszy dla konkretnego profilu creature. |
| xoreos `Model_NWN::loadBinary()` | Czyta byte `type`, potem `classification` i `fogged` z binary headera, przed root node, animation array i supermodel. | W tej implementacji lokalne zmienne nie ustanawiaja semantyki Beamdog; potwierdzaja jedynie granice i kolejnosc pól. |
| NeverBlender `nvb_def.py` / `nvb_mdl.py` | Authoring UI ma jawne tekstowe klasy `character`, `tile`, `door`, `effect`, `gui`, `item`, `other`; eksport ASCII wpisuje `classification <CLASS>`. | NeverBlender nie mapuje ich na binary byte i nie emituje binary MDL. [NWN Wiki: Models](https://nwn.wiki/spaces/NWN1/pages/38175602/Models?src=contextnavpagetreemode) rozroznia ASCII i compiled MDL, wiec nie wolno przenosic tekstu authoringu bezposrednio na ABI. |
| [NWN Wiki: TXI transparency for placeables](https://nwn.wiki/spaces/NWN1/pages/38176400/Placeable%2BModel%2BTXI%2BTransparency) | Classification ma szczegolne znaczenie w opisanym profilu transparentnego placeable z TXI/dummy node. | M0 jest opaque creature (`render=1`, RGB TGA, brak TXI w kandydacie), wiec ten wyjątek nie jest hipoteza dla r27. |

### Jedyna mala poprawka aplikacji

Nie jest potrzebna zmiana emitted bytes. Jest natomiast P1 w diagnostyce
own semantic readback: obecny warunek laczy `actual.model.name != expected`
z `actual.model.geometry_type != 2` i raportuje oba przypadki jako
`model.name`. Minimalna poprawka rozdziela je na `model.name` i
`model.geometryType`, z testem mutujacym wyłącznie `core+0x6c` z `2` na `1`
i oczekujacym tylko `model.geometryType`. Osobny, env-gated corpus assertion
powinien zamknac szesc entries na tabeli `2/4/1/0`; nie kopiuje payloadu i nie
staje sie proofem runtime.

To poprawia wiarygodnosc diagnosy future mutantow. Nie zmienia current M0 i
nie zastępuje packetu Toolset -> NWN.

Aurora, NWN i INI nie byly uruchamiane ani modyfikowane w tym cyklu.

## 55. R27: wielo-HAKowy resolver jest luka eligibility/proof, nie potwierdzona przyczyna renderera (2026-07-21)

### Werdykt

**Bieżący r27 nie może przejść własnego `M0RuntimeFixtureContractV1`, mimo że
ma hash-bound profil AUR-S07, ponieważ publiczny builder i verifier M0
wymagają dokładnie jednego HAK-a.** Zapisany MOD r25 ma trzy wpisy
`Mod_HakList` w kolejności `[m2a_m0r27, m2a_m0r26, m2a_m0r21]`. To jest
konkretny P0 aplikacji w warstwie eligibility/proof: obecny verifier nie
potrafi udowodnić, które z trzech kolidujących zasobów runtime rzeczywiście
rozwiązuje.

Nie jest to jednak dowód, że r27 jest niewidoczny ani uprawnienie do zmiany
MDL, `type=5`, Appearance `848`, alpha, roota czy utworzenia r28. Aktualny
stan r27 pozostaje `modelVisibility=not_tested`,
`proofCompleteness=missing`.

### Fakty z aktualnych artefaktów

Wszystkie trzy HAK-i zawierają identyczne klucze ERF, lecz różny payload MDL:

| HAK | SHA-256 HAK | `appearance`/2017 | `m2a_m0p01`/2002 | `m2a_m0t01`/3 |
| --- | --- | --- | --- | --- |
| r21 | `25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6` | `04f7933cd6cb67ae4d065a18c6bed9e18b8688bb97bd4cae5f9072e1d2792acb` | `cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5` | `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b` |
| r26 | `6f805873420b5279b45dadc26d913fa7692374df9e0582c23b42c8a4b42d6bb0` | ten sam | `b177f450f68a2e762b8c15886be17b4aaf7ccfe71da14d5ad0db9df8c5d01b77` | ten sam |
| r27 | `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd` | ten sam | `bf7864f0ed541ed93c4075d167f4a2392eeb6ef37b4cdf444cbb74efd5731578` | ten sam |

R27 jest pierwszym wpisem listy i zawiera oczekiwany `type=5`; r21 i r26
mają starsze MDL z `type=0`. `appearance.2da` oraz TGA są bajtowo identyczne,
więc przy tym konkretnym packetcie rozstrzygnięcie kolejności zmienia tylko
wybrany MDL, nie row `848` ani mapę tekstury.

### Granica źródeł

| Źródło | Fakt | Granica |
| --- | --- | --- |
| Aurora decomp `FUN_00aefb18` / pętla `Mod_HakList` | Toolset serializuje listę HAK-ów po kolei, nie jako nieuporządkowany set. | Odczyt nie pokazuje klientowego predykatu priorytetu zasobów EE. |
| xoreos `Module::loadHAKs()` + `ResourceManager` | Indeksuje element `i` z priorytetem `1002+i`, sortuje rosnąco i wybiera ostatni, więc jego implementacja preferuje **ostatni** HAK. | Jest to jawnie rozbieżne z bieżącą konwencją Toolset/EE; xoreos nie jest oracle'em Beamdog dla tego punktu. |
| [Beamdog forum: userpatch / Toolset order](https://forums.beamdog.com/discussion/69487/nwn-ee-userpatch-ini-unofficial-faq) | Wypowiedź developera opisuje, że w Toolsecie nowsze/nadpisujące HAK-i są na górze listy. | To dokumentacja społecznościowa, nie trace binarnego klienta; wspiera r27 jako pierwszy wpis, ale finalny dowód to kontrolowany r27 runtime. |
| [NWN Wiki: Models](https://nwn.wiki/spaces/NWN1/pages/38175602/Models) / NeverBlender | NeverBlender jest narzędziem authoringu/exportu MDL. | Nie rozwiązuje HAK, `appearance.2da` ani priorytetu zasobów, więc nie może naprawić tej luki. |

### Minimalna poprawka i testy TDD

Nie zmieniać `write_hak_v1`, nie scalać ręcznie HAK-ów i nie materializować
nowego kandydata. Minimalna poprawka to nowy, kompatybilny
`M0RuntimeFixtureContractV2` / verifier z wejściem:

`ordered_hak_layers = [{resref, sha256, bytes}, ...]`.

Ma on odczytać `Mod_HakList`, wymagać identycznej liczby, kolejności, resrefów
i hashów warstw oraz policzyć wybraną warstwę dla trzech kluczy M0. Reguła
priorytetu musi być nazwana w danych kontraktu, nie ukryta w kolejności pętli.
Dla aktualnego r27 spodziewanym zwycięzcą jest pierwszy wpis
`m2a_m0r27`; późniejszy kontrolowany run może tę regułę dopiero zatwierdzić
jako runtime fact.

Testy RED/GREEN:

1. pozytywny fixture `r27,r26,r21` wybiera r27 dla MDL i weryfikuje trzy
   podane hashe payloadów;
2. mutacja wyłącznie kolejności na `r26,r27,r21` daje stabilny
   `M0-RUNTIME-CONTRACT-HAK-RESOLUTION-MISMATCH` pod
   `resolver.m2a_m0p01.2002`;
3. zmiana jednego hasha/bytes warstwy daje błąd bindingu warstwy przed próbą
   parsowania modelu;
4. istniejący pojedynczy HAK V1 pozostaje zielonym przypadkiem kompatybilnym
   wstecz.

Po tej zmianie nadal potrzebny jest ten sam live proof r27: dokładny wybór i
readback obiektu plus świeży Toolset `TScrollBox`, potem NWN i log. `Focus on
Object` oraz kadrowanie są opcjonalne. Aktualna diagnoza kończy się offline;
Aurora, NWN i INI nie były uruchamiane ani modyfikowane.

## 54. Bieżąca decyzja offline: jedyny otwarty A/B to local-animation `type=5` r27 (2026-07-21)

### Status nadrzędny

Ta sekcja **zastępuje wyłącznie statusowe zdania `NWN=not_tested`** we
wcześniejszych, chronologicznych sekcjach §2a, §46-§53. Starsze odczyty
artefaktów pozostają ważne, ale późniejsze capture'y r21 i r26 muszą być
widoczne w ledgerze. Nie zmienia to faktu, że pełny packet przyczynowy dla
aktualnego r27 nadal jest `missing`.

| Kandydat | Świeży fakt | Co wynika / czego nie wolno z niego wywnioskować |
| --- | --- | --- |
| r21 | Toolset pokazuje M0; log NWN ładuje `m2a_m0r21`, a capture nie pokazuje M0. Fixture jest jednak przy `[5.0642,14.9358,0]`, poza później zamrożonym punktem `[10,14.5,0]`. | `NWN.modelVisibility=not_visible`, ale nie jest to test samego MDL. Dopuściło wyłącznie następcę z poprawnym placementem. |
| r26 na MOD r25 | Native geometry wiąże Area `m2a_m0a25`, entry `(10,10,0)` i fixture `(10,14.5,0)`; Toolset jest `visible`, a log NWN ładuje `m2a_m0r25` i capture nadal nie pokazuje M0. Jedyna zmiana r26 to identity controllers. | `NWN.modelVisibility=not_visible`; identity-controller-only jest negatywnym wynikiem. Runtime `proofCompleteness=missing`, więc nie przypisuje sam w sobie winy dokładnej strukturze MDL. |
| r27 na tym samym MOD r25 | `native-geometry-gate.json` i profil AUR-S07 wiążą stały fixture oraz module SHA-256 `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257`. HAK r27 ma SHA-256 `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd`. | `modelVisibility=not_tested`, `proofCompleteness=missing`: nie ma jeszcze obrazu Aurora/NWN r27. To jest bieżący kandydat; nie tworzyć r28. |

### Dlaczego `type=5` jest jedynym realistycznym eksportowym A/B

1. W dekompilacji Aurora `FUN_00a3b874 -> FUN_00a3ee64 -> FUN_00a3ed90`
   kopiuje **jeden bajt** local-animation header `+0x6c` do runtime state.
   Nie scala go z czterema bajtami `+0x68`.
2. `xoreos/src/graphics/aurora/model_nwn.cpp`,
   `Model_NWN::readAnimBinary()`, po `24 + 4` bajtach odczytuje `uint8_t
   type`, pomija trzy bajty paddingu i dopiero potem czyta length, transition
   oraz `animroot`. Jest to niezależny parser, nie oracle zachowania Beamdog.
3. Audyt corpusowy CEP zapisał `type=5` w każdym z `24 707` osiągalnych
   local-animation headers. R26 ma `0` w siedmiu nagłówkach M0; r27 różni się
   od r26 dokładnie siedmioma bajtami `00 -> 05`, po jednym dla `cappear`,
   `cpause1`, `cwalk`, `crun`, `ca1slashl`, `cdamagel`, `cdead`.
4. Aktualny writer już wykonuje `write_u32(core, animation.header + 0x6c, 5)`;
   semantic readback wymaga `animation_type == 5` i paddingu `[0,0,0]`.
   Nie ma więc następnej poprawki source do wdrożenia w tej warstwie.

NeverBlender pozostaje ważnym źródłem **authoringu** (Aurora root, klasyfikacja
i export), lecz nie specyfikuje tego binarnego bajtu. Aktualne materiały NWN
opisują go jako wspierany plugin Blenderowy, a osobny przewodnik ostrzega, że
animacje wyeksportowane z NeverBlendera mogą mieć problem z nagłówkami:
[Models](https://nwn.wiki/spaces/NWN1/pages/38175602/Models) i
[Importing Animations](https://nwn.wiki/spaces/NWN1/pages/195362863/Importing%2BAnimations).
Nie jest to dowód wartości `5`; wzmacnia tylko granicę, że r27 musi być
sprawdzony własnym binary readbackiem i runtime proofem.

### Wyniki negatywne: czego nie zmieniać

Nie zmieniać teraz `MODELTYPE=S`, `RACE=m2a_m0p01`, Appearance `848`,
classification `4`, root `0x01`, `animroot`, rodzaju mesh node, base geometry,
MDX, alpha/TGA ani texture resrefu. Ich kontrakty są już zgodne z odczytem
Aurora/xoreos i z widocznością Toolsetu; r26 dodatkowo obalił hipotezę, że
same identity controllers naprawią runtime.

### Najmniejszy test offline

Obecny test mutuje `+0x6c..+0x6f` zapisem `u32(0)`; przy bazowym paddingu
`[0,0,0]` materialnie zmienia on tylko type `5 -> 0`, ale minimalny test
powinien wyrazić to dosłownie. W `write_binary_mdl.rs` należy użyć wyłącznie:

```rust
let mut wrong_type = artifact.payload.clone();
wrong_type[absolute(clip.offset) + 0x6c] = 0;
let inspected = inspect_binary_mdl(&wrong_type).unwrap();
assert_eq!(inspected.animations[0].animation_type, 0);
assert_eq!(inspected.animations[0].animation_type_padding, [0, 0, 0]);
assert_eq!(
    semantic_diff(&expected, &inspected),
    ["animations[0].header"]
);
```

To jest test policy/readback, nie test renderera. W tym cyklu przeszły:

1. `cargo test -p m2a-core --lib semantic_animation_mutations_name_opaque_budget_and_padding --quiet` — `1 passed`.
2. `cargo test -p m2a-core --test model_pipeline static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice --quiet` — `1 passed`.

Jedyny test rozstrzygający pozostaje poza zakresem offline: dla dokładnego r27
wykonać po autoryzacji exact object selection/readback i świeży Toolset
`TScrollBox`, a następnie NWN na tym samym MOD/HAK/fixture/logu. `Focus on
Object` i kadrowanie są opcjonalne. Do tego czasu nie uruchamiać Aurora/NWN,
nie dotykać INI i nie tworzyć nowego kandydata.

## 50a. Current-source audit: `canonical runtime` M0 still emits a Toolset-only `appearance.2da` slice (2026-07-21)

### Verdict

**This is a packaging/eligibility logic gap in the application, not a new
export defect of the visible r21 M0.** The public function named
`build_meshy_m0_canonical_runtime_package_v1()` passes
`M0PackageContractV1::Canonical`, but both M0 contracts first execute
`retain_two_da_row_prefix_v1(..., 848, ...)`. Both subsequently report the
same string policy, `AURORA_VISIBLE_PREFIX_AND_ONE_ROW_APPENDED`.

Consequently, `canonical runtime` currently does *not* mean a generally safe
replacement `appearance.2da`: it is another 0..848 isolated slice. The exact
r21 proof packet remains internally consistent -- GIT/UTC use row 848 and
the Toolset has already shown this candidate -- so neither its MDL, texture,
root, animation tree, nor appearance row should change. The fault is that the
API/status does not make the deployment scope machine-enforceable.

### Current-state evidence and boundaries

| Source | Fact | Boundary |
| --- | --- | --- |
| Current generator `crates/m2a-core/src/model_pipeline.rs` | Lines 503-507 truncate before the `IsolatedControl`/`Canonical` match; lines 551-594 append to that same slice in both arms. `Canonical` therefore cannot preserve an input tail even when its name suggests a runtime lane. | Source/readback fact, not proof that NWN will fail the isolated r21 module. |
| Current contracts and tests | `M6MaterializationReportV1` and manifest expose only a free-form `appearance_payload_policy` string. `M0RuntimeFixtureContractV1` has no table scope or row-count fields, and is emitted only for `IsolatedControl`. The canonical test asserts the same slice-policy label, while the isolated test proves the 848-prefix readback. | A green test here proves the existing policy; it is not a general-runtime eligibility proof. |
| Aurora decomp + local r21 readback | `Appearance_Type` is consumed as a `ushort` lookup; r21 `848` exists in its 849-row HAK table and is visible in Toolset. | The decomp fragment does not prove every Beamdog client failure mode for another creature whose row is absent. |
| xoreos `TwoDARegistry` / `Creature::loadModel` | The registry opens one resolved `appearance` resource, and the creature loader looks up its physical appearance row before reading `MODELTYPE` and `RACE`; it does not merge a HAK row into a lower-priority table. | Independent implementation, not a Beamdog runtime oracle. |
| [NWN Wiki: 2da Files](https://nwn.wiki/spaces/NWN1/pages/38174875/2da%2BFiles) | Row numbers are physical, sequential entries; `appearance.2da` is client, server and Toolset loaded and its practical field limit is `uint16`. The planned full-input append at 15 100 is therefore representable. | Community documentation establishes format/limit context, not the current r21 runtime result. |
| [NWN Wiki: Models / NeverBlender boundary](https://nwn.wiki/spaces/NWN1/pages/38175602/Models?src=contextnavpagetreemode) | NeverBlender concerns authoring/export of the MDL; it has no authority over replacing or merging `appearance.2da`. | It rules out treating a Blender setting as a fix for this package-scope bug. |

The independent xoreos reader makes the concrete risk easy to state without
overclaiming: an object whose `Appearance_Type` lies outside the selected HAK
table gets an empty row from that implementation, rather than a row merged
from the base table. That is enough to require an application gate before a
general HAK is issued. It is not evidence that r21 row 848 is invalid.

### Minimal implementation and TDD gate

Do not change the r21 packet. For future packaging, make the table strategy a
typed, persisted capability, for example:

1. `IsolatedToolsetVerticalSlice`: may call `retain_two_da_row_prefix_v1`,
   must carry `scope=isolated_toolset_vertical_slice`, and must be rejected by
   any general-runtime distribution entrypoint.
2. `FullRuntimeAppend`: must append directly to the caller-supplied complete
   table, preserve its entire prefix, use the resulting physical row in both
   GIT and UTC, and carry `scope=full_runtime_append` plus input/output row
   counts in the manifest and runtime contract.

The smallest correction is to move the `retain...` call inside only the
isolated arm, route `build_meshy_m0_canonical_runtime_package_v1()` through
`FullRuntimeAppend`, and replace the untyped policy string with a serializable
enum. Reuse the already-existing general M6 `PRESERVED_AND_APPENDED` path as a
behavioral boundary, not by copying payload or reference code.

TDD must first add a full input with 15 100 physical rows. The canonical
function must emit 15 101 rows, preserve rows 0..15 099 byte-for-byte in the
2DA append report, and bind `Appearance_Type=15100` in both GIT and UTC. A
negative test must prove that an `IsolatedToolsetVerticalSlice` artifact is
rejected by the general-runtime entrypoint with a named scope error (for
example `M0-APPEARANCE-SCOPE-GENERAL-RUNTIME-INVALID`). The existing isolated
test must remain and continue to require exactly 849 rows and row 848.

This closes an ambiguity in application eligibility only. A hash-bound NWN
packet for r21 is still the only test that can change its current runtime
status: `modelVisibility=not_tested`, `proofCompleteness=missing`.

Aurora, NWN and INI were not launched or modified in this cycle.

## 50b. Current-source correction: global case-fold node-name gate is already implemented (2026-07-21)

### Verdict

**The implementation task proposed in section 39 is already closed as
source/readback.** The current `plan()` builds one global
`HashSet<to_ascii_lowercase(name)>` across all rig nodes and all generated
`m2a_seg_<segment_id>` names. It rejects either collision with the stable
error `M4-NODE-NAME-DUPLICATE` before binary layout or HAK generation.

This is not a new explanation for the visible r21 M0: its root and one mesh
name are distinct. It is also not an NWN runtime proof for historically
ambiguous external models; the product no longer emits that ambiguous shape.

### Evidence and correction boundary

| Source | Fact | Boundary |
| --- | --- | --- |
| Current writer `write_binary_mdl.rs`, `plan()` | Checks rig--rig and rig--generated-mesh name collisions after ASCII case-fold and reports the exact node or segment path. | This is the current product contract, not a guarantee for arbitrary already-built MDLs. |
| Current `duplicate_output_node_names_are_rejected_globally_after_ascii_case_fold` test | Rejects two `shared` nodes in separate branches, `Bone`/`bone`, and a rig node colliding with `M2A_SEG_5`; the focused test passed `1/1` in this cycle. | Own readback/gate proof only; no Aurora or NWN process was launched. |
| Aurora decomp `FUN_00a57af4` / `FUN_00a57b58` / `FUN_00b8737c` / `FUN_00b8f008` | Parent-name resolution recurses through the model tree. Its string comparison lowercases each byte via `LCMapStringA(..., 0x200, ...)` before comparing. | It establishes the native case-insensitive lookup risk for ASCII output; it does not prescribe automatic renaming. |
| xoreos `ModelNode_NWN_Binary::checkDuplicateNode()` | Scans previously read nodes with `equalsIgnoreCase`; on collision it reparents children to the later node and deletes the earlier one. | Independent implementation, not a Beamdog renderer oracle, but directly demonstrates why duplicate names are unsafe. |
| [NWN Wiki: MDL](https://nwn.wiki/spaces/NWN1/pages/38175669/MDL?src=contextnavpagetreemode) and [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | The authoring guidance warns that colliding/truncated node names can cause strange behavior or crashes; NeverBlender is an authoring/export tool, not a reason to weaken the binary writer gate. | Community/tool documentation supports the authoring boundary but does not replace the local decomp evidence. |

Section 39 remains useful as the original diagnosis and TDD design, but its
phrasing that the writer "currently permits" duplicate names is superseded by
this current-source audit. Do not add a second gate, silently rename nodes, or
change M0/H1 names. Preserve the existing named rejection and move the open
work to skin deformation/runtime proof, not duplicate-node remediation.

Aurora, NWN and INI were not launched or modified in this cycle.

## 50c. `boneconstantindices`: current `[i16; 2]` is bit-preserving; the missing high-word test is P1 (2026-07-21)

### Verdict

**Do not describe the current pair representation as a proven loss of binary
data or as a skin-rendering fix.** `read_core_i16x2()` reserves exactly four
bytes per row and reads offsets `+0` and `+2`; together, the two little-endian
`i16` values preserve all 32 input bits. `semantic_diff` compares both pair
members. The existing mutation changes only the low half, so it is incomplete
coverage of the 32-bit field, not evidence that the high half would be lost.

The field is still semantically opaque. A future `Vec<u32>` readback API would
name it more honestly, but the present writer deliberately emits `0u32` as two
zero halfwords, producing the same four bytes. This is therefore a P1
representation/test correction, not a reason to change a generated model or
to assign a runtime visibility failure to constants.

### Evidence and boundaries

| Source | Fact | Boundary |
| --- | --- | --- |
| Current parser and semantic readback | `read_core_i16x2()` validates `count * 4`, reads both halfwords, and semantic diff compares the complete `[i16; 2]`. A high-half mutation is representable (including its sign bit). | The JSON/API value does not expose one named opaque word, so it should not be given invented pair semantics. |
| Current writer | Allocates `mapCount * 4` and writes two zero `i16` values at `+0` and `+2`; this is byte-identical to writing `0u32`. | The writer has no sourced semantic input for nonzero constants and must not copy them from a reference model. |
| Aurora decomp `FUN_00a3dd4c` | For the constants array it copies exactly `count << 2` bytes from binary model data. | This proves element width/copy behavior, not what every later renderer path means by a nonzero word. |
| xoreos `model_nwn.cpp` | Recognizes the skin node flag but currently skips the binary skin layout as `TODO: Skin`; it has no named constants interpretation. | xoreos cannot decide the field's semantics or validate a pair-vs-word claim. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Documents skinmesh authoring through vertex groups/pseudo-bones, not this compiled binary array. | NeverBlender cannot justify a nonzero constant value or a runtime workaround. |

The earlier direct-palette trace remains a negative result for the narrow
hypothesis that emitted zero constants cause the first skin draw: it found no
constants-array read on that palette path. It does not establish that no other
engine consumer exists.

### Minimal test and implementation boundary

Add one regression before any API retype: mutate only bytes `+2..+4` of a
known constants row to `0x8001` and require precisely
`nodes[...].skin.constants` from semantic diff. A parser fixture with raw word
`0x8001_0007` should prove the current bit-preserving representation is
`[7, -32767]` without claiming that either half has independent meaning.

Only after that test may an explicitly backward-compatible API rename to
`bone_constant_words: Vec<u32>` be considered. It must preserve the current
all-zero output byte-for-byte, add no nonzero generated values, and remain
separate from the hash-bound Toolset/NWN skin-deformation proof. The static
r21 M0 has no skin node, so this has no effect on Appearance 848 or its
observed Toolset silhouette.

Aurora, NWN and INI were not launched or modified in this cycle.

## 51. R21 M0: retailowy template `nw_dwarfmerc001` nie wypada poza skrocone `appearance.2da` (2026-07-21)

### Werdykt

**To jest negatywny wynik dla ostatniej realistycznej hipotezy export/runtime: bazowy UTC nie kieruje r21 na nieobecny wiersz `appearance.2da`.** Odczyt in-place retailowego `nw_dwarfmerc001.utc` z lokalnego `nwn_base.key` zwraca `Appearance_Type=249` (GFF field type `2`, Word). R21 HAK ma fizyczne wiersze `0..848`; zarowno domyslne `249`, jak i jawne nadpisanie instancji GIT `848`, sa wiec w jego zakresie.

Dokladny locator retailowego faktu: `nwn_base.key` SHA-256 `09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935`, wpis KEY `57698`, `resource_id=0x01B0031D`, BIF index `27`, resource index `797`; wskazuje `data/xp2_templates.bif` SHA-256 `5214439d872ab6854fbe1b4339b5e00e95df65f9aa044086a846f3da3efba07b`, offset `2247546`, `3603` B, `UTC V3.2`. Odczyt byl tylko w pamieci: nie wypakowano ani nie skopiowano retailowego payloadu.

To rozstrzyga waski przypadek `TemplateResRef=nw_dwarfmerc001` plus skrocona tabela. Nie rozstrzyga samodzielnie renderingu NWN, lecz usuwa powod do zmiany MDL, root node, animation, `MODELTYPE`, alpha albo tekstury. Status aktualny jest w §54: r21 mial placement-confounded `not_visible`, r26 `not_visible`, a r27 pozostaje `not_tested` / `proofCompleteness=missing`.

### Triangulacja kolejnosci i granic

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Local retail KEY/BIF + own in-memory GFF read | Bazowy UTC jest faktycznie zasobem `2027` i ma `Appearance_Type` Word `249`, a nie wartosc powyzej `848`. | To fakt o lokalnej instalacji referencyjnej, nie o ostatecznej kolejnosci scalania w kliencie Beamdog. |
| xoreos `engines/nwn/creature.cpp`, `Creature::load` / `loadProperties` / `loadModel` | Otwiera UTC z `TemplateResRef`; wykonuje `loadProperties(*blueprint)`, potem `loadProperties(instance)`, a dopiero pozniej `loadModel()` robi `appearance.getRow(_appearanceID)`. Dla `MODELTYPE != P` wybiera `RACE`. | Niezalezna implementacja, nie oracle Beamdog; wprost przedstawia jednak semantyke nadpisania zgodna z obserwacja r21 w Toolsecie. |
| Aurora decomp `FUN_00532174 -> FUN_00512810` | Loader czyta `Appearance_Type`, przekazuje je jako `ushort` do setter'a, ktory pyta globalne `appearance.2da` o `MODELTYPE`; `TemplateResRef` jest odczytywany osobno. | Dekompilacja nie daje w tym fragmencie nazwanej rekonstrukcji pary blueprint/GIT, wiec sama nie jest dowodem kolejnosci merge. |
| NeverBlender Manual 2.0 | Opisuje kontrakt authoringu MDL (root, classification, animacje), a nie pliki UTC/GIT. | Nie moze byc zrodlem dla tezy o merge template/instance; potwierdza tylko, ze nie nalezy mieszac resrefu UTC z MDL `RACE`. |

### Jedyny minimalny gate

Nie ma poprawki do emitera ani nowej iteracji r21. Pozostaje tylko niezalezna luka generatora z §50: konkretny verifier `validate_binary_m0_vertical_slice_module_readback()` ma odczytac lokalny UTC/2027 i wymagac `utc.Appearance_Type == GffValueV1::Word(appearance_row)`, obok obecnego wymagania GIT. Minimalny test negatywny mutuje wyłącznie serialized UTC z `848` na `847` i oczekuje `M0-BINARY-VERTICAL-SLICE-SEMANTIC-DIFF` pod `binary_m0.utc.Appearance_Type`.

Ten test zabezpiecza generowany, samowystarczalny vertical slice przed przyszla rozbieznoscia template/GIT. Nie jest uzasadnieniem zmiany exportu aktualnego M0 i nie zastępuje kontrolowanego packetu NWN.

W tym cyklu nie uruchamiano Aurora ani NWN i nie modyfikowano INI.

## 52. R21 M0: nie ma aktywnego shadowingu r21, ale stale resrefy nie sa bezpieczne dla dowolnego multi-HAK (2026-07-21)

### Werdykt

**Dla dokladnego modulu r21 hipoteza „NWN laduje obcy `m2a_m0p01` zamiast wpisu HAK-a r21” ma wynik negatywny offline.** MOD ma jeden `Mod_HakList=m2a_m0r21`; jego HAK zawiera wlasnie `appearance`/2017, `m2a_m0p01`/2002 i `m2a_m0t01`/3. Retailowy `nwn_base.key` (113 489 wpisow, SHA-256 `09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935`) nie ma zadnego z tych trzech resrefow. Read-only scan obu lokalnych katalogow `override` nie znalazl `appearance.2da`, `m2a_m0p01.mdl`/`.mdx` ani `m2a_m0t01.tga`/`.dds`.

Jednoczesnie read-only indeksowanie key tables wszystkich 126 lokalnych HAK-ow i 83 MOD-ow pokazalo dziewiec historycznych HAK-ow zawierajacych te same pary `(m2a_m0p01, 2002)` i `(m2a_m0t01, 3)`. Same pliki historyczne nie sa aktywnymi zasobami r21: nie wystepuja w jego jednoelementowym `Mod_HakList`. Jest to jednak realny limit **stalych nazw kontrolnego M0**: modul dolaczajacy dwa takie HAK-i bylby zależny od kolejnosci resolvera, nie od semantyki MDL.

Nie zmieniac resrefow r21 ani nie tworzyc nowego `rNN`: Toolset juz pokazal ten konkretny kandydat. Wniosek dotyczy przyszlego wielo-modelowego pakietowania aplikacji, nie przyczyny niewidocznosci r21. Status aktualny jest w §54: r26 `not_visible`, r27 `not_tested`, a runtime `proofCompleteness` pozostaje `missing`.

### Triangulacja zrodel

| Zrodlo | Fakt | Granica |
| --- | --- | --- |
| Local retail KEY + local override scan | Nie ma retailowego wpisu o resrefach M0 ani pasujacego loose override; r21 HAK jest jedynym aktywnym nosnikiem tych trzech zasobow w jego MOD. | Scan nie jest inventory zdalnego serwera, cache NWSync ani uruchomionego procesu gry. |
| Local HAK/MOD metadata sweep | Wszystkie 209 kontenerow V1.0 przeszly odczyt metadanych; dziewiec historycznych HAK-ow posiada ten sam MDL i TGA resref M0. | Obecnosc w katalogu nie oznacza dolaczenia do r21; sweep nie jest dowodem priorytetu Beamdog. |
| xoreos `IFOFile::load` + `Module::loadHAKs` + `ResourceManager` | Odczytuje `Mod_HakList`, indeksuje HAK-i jako zasoby modulu, a manager sortuje kolizje po priorytecie (wyzszy numer ma wyzszy priorytet). | Niezalezna implementacja, nie oracle Beamdog. Dla jednego HAK-a r21 kolejnosc i tak nie ma alternatywy. |
| Aurora decomp | Zna i serializuje `Mod_HakList` oraz `Mod_Hak`; lokalna dekompilacja nie daje wystarczajacego, nazwanego trace'u pelnego priorytetu resource managera klienta EE. | Nie wolno wyprowadzac z niej reguly dla dowolnego multi-HAK bez proofu runtime. |
| [NeverBlender Manual 2.0](https://neverwintervault.org/sites/neverwintervault.org/files/project/895/files/neverblender_manual_2.0.pdf) | Kontrakt authoringu wiaze Aurora root z nazwa pliku modelu; nie opisuje priorytetu HAK/override. | Nie jest zrodlem dla wyboru jednego z kolidujacych kontenerow. |

### Najwezszy przyszly gate pakietowania

Aktualny writer HAK odrzuca duplicate `(resref, type)` **wewnatrz jednego** archiwum; nie zna listy zewnetrznych HAK-ow, wiec nie moze dowiesc braku kolizji calego modulu. Gdy aplikacja dostanie funkcje skladania wielu wygenerowanych modeli w jednym module, ma ona najpierw przyjac jawny, stabilny namespace zasobow dla kazdego modelu i walidowac unikalnosc par `(MDL resref, 2002)` oraz `(TGA/DDS resref, type)` w calym ordered HAK manifest.

Minimalny test TDD dla tej przyszlej funkcji: dwa model slots z tym samym namespace musza zostac odrzucone przed zapisem HAK-a z nazwanym bledem collision manifestu; dwa rozne namespaces musza zachowac spojnosc `appearance.RACE -> MDL root -> HAK key` oraz `texture0 -> TGA/DDS key`. Nie zmieniac obecnego izolowanego M0, jego `appearance` ani HAK listy, aby symulowac ten przypadek.

W tym cyklu nie uruchamiano Aurora ani NWN i nie modyfikowano INI.

## 53. R21 M0: zamkniecie offline — `type=0` jest starym artefaktem, nie bledem aktualnego emitera (2026-07-21)

### Werdykt

**Dla pojedynczego statycznego M0 nie ma juz kolejnej potwierdzonej poprawki
exportu do zaimplementowania offline.** Jedyna pozostala realistyczna
niezgodnosc binarnego ABI znajduje sie w *wdrozonym* MDL r21: wszystkie jego
siedem local-animation headers ma `animation_type=0` pod `core+0x6c`.
Aktualny writer zapisuje natomiast `05 00 00 00` w tym miejscu, a dwa waskie
testy source/readback przechodza. Jest to zatem rozjazd artefaktu wdrozonego z
aktualnym kodem, nie otwarty bug implementacji `MODELTYPE`, root node,
klasyfikacji, renderingu mesh'a, alpha ani tekstury.

R21 pozostaje konkretnym, widocznym w Toolsecie kandydatem przy
`Appearance_Type=848`, `MODELTYPE=S` i `RACE=m2a_m0p01`. Jego późniejszy run
NWN jest `not_visible`, ale placement był poza kontraktem. R26 z poprawnym
fixture jest również `not_visible`, lecz z niepełnym packetem przyczynowym.
Wobec tego nie wolno twierdzić, że `type=0` powoduje niewidoczność; jedynym
aktualnym rozstrzygającym kandydatem jest r27 z `type=5`, obecnie
`modelVisibility=not_tested`, `proofCompleteness=missing`.

### Dowody i wyniki negatywne

| Warstwa | Fakt | Wniosek ograniczony do M0 |
| --- | --- | --- |
| HAK r21, MDL SHA-256 `cae00c4a8fe42ce0bd6b0dcbd3e7621b5e673727f55c5bb9c548fa17ee87d7a5` | `cappear`, `cpause1`, `cwalk`, `crun`, `ca1slashl`, `cdamagel` i `cdead` maja `type=0` oraz zerowy padding. | R21 nie jest artefaktem aktualnej korekty ABI. To nie dowod render failure. |
| Aktualny `emit_animations()` | Zapisuje `write_u32(... + 0x6c, 5)`, tj. byte type `5` i trzy deterministyczne bajty paddingu `0`. | Aktualny export nie wymaga patcha tego pola. |
| Aurora decomp `FUN_00a3b874 -> FUN_00a3ee64 -> FUN_00a3ed90` | Lokalny state przenosi do runtime dokladnie jeden byte spod `+0x6c`. | Pole jest realnym kontraktem ABI, ale odczyt nie pokazal predykatu, ze `0` sam ukrywa model. |
| xoreos `Model_NWN::readAnimBinary()` | Po `24 + 4` B czyta `uint8_t type`, omija trzy bajty paddingu, a nastepnie odczytuje dlugosc, transition i `animroot`. | Niezaleznie potwierdza granice pola; xoreos nie jest oracle'em Beamdog. |
| NeverBlender Manual 2.0 | Wymaga Aurora Root o nazwie pliku oraz opisuje `Character` i `Render`; nie opisuje binary `animation_type`. | Nie ma podstaw do wyprowadzania z NeverBlendera innej wartosci type, zmiany roota lub alpha. |

Odrzucone pozostaja wiec cztery hipotezy dla aktualnego single rigid M0:
`MODELTYPE=S` jest poprawnym direct-model routingiem, root `0x01` i lokalne
zero-geometry state leafs sa zgodne z profilem, tekstura jest RGB bez kanalu
alpha i material ma `render=1`, a brak identity controllerow w r21 nie
uniemozliwil jego obserwacji w Toolsecie. Nie zmieniac zadnej z tych warstw na
podstawie brakującego runtime packetu r27.

### Najmniejszy wykonany test i granica dalszego kroku

W tym cyklu, bez uruchamiania Aurora/NWN, przeszly:

1. `cargo test -p m2a-core --test model_pipeline static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice --quiet` — `1 passed`; wymaga siedmiu lokalnych clipow M0 z type `5`.
2. `cargo test -p m2a-core --lib semantic_animation_mutations_name_opaque_budget_and_padding --quiet` — `1 passed`; zmiana bajtow `+0x6c..+0x6f` do zera daje tylko `animations[0].header` w semantic diff.

Drugi test jest wystarczajacym minimalnym negatywnym gate'em, bo baseline
paddingu jest `00 00 00`: zapis `u32(0)` zmienia materialnie tylko byte type
`5 -> 0`. Przyszly refactor moze zawęzic mutacje do jednego bajtu `+0x6c`, ale
nie jest to poprawka renderera ani uzasadnienie nowej iteracji M0.

Jedyny test, ktory moze rozstrzygnac przyczyne runtime, jest poza zakresem
offline: po osobnej autoryzacji trzeba zwiazac z hashami dokładny r27 i
wykonać kontrolowany packet NWN bez jednoczesnej zmiany `848`, `S`, `RACE`,
roota, controllerow, tekstury lub placementu. Do tego czasu nie tworzyc r28,
nie podmieniac artefaktow r27 i nie interpretowac braku packetu jako nowego
bledu modelu.

W tym cyklu nie uruchamiano Aurora ani NWN i nie modyfikowano INI.
