# Plan: definicja i rozwiazanie runtimeowego problemu modelu H1 (2026-07-18)

Status: `ACTIVE / ROOT CAUSE NIEZAMKNIETE`.

## 1. Jednoznaczna definicja problemu

Problemem nie jest juz ogolne zdanie "modelu nie widac". Aktywna definicja
brzmi:

> Meshy2Aurora nie ma jeszcze proofu, ze wygenerowany native binary MDL H1
> jest w kliencie NWN:EE jednoczesnie **latwy do zidentyfikowania**,
> **geometrycznie poprawny** i **poprawnie animowany** jako direct creature.

Znane fakty rozdzielaja objaw na trzy niezalezne tory:

| Tor | Fakt | Co trzeba rozstrzygnac |
| --- | --- | --- |
| Proof/runtime | `c_squirrel` byl widoczny w Toolsecie, ale nie uzyskal bramki widocznosci w kliencie. | Czy kontrola, HAK, pozycja instancji i kamera proofu dowodza rzeczywistej widocznosci w NWN. |
| Mesh/MDX | Wariant H1 v20 (`RIGID`) zostal wyrenderowany w NWN jako duza, zdeformowana figura. | Ktore pole wspolnej sciezki binarnego mesha/MDX zmienia znaczenie geometrii w runtime. |
| Skin/animacja | H1 v19 nie przeszedl widocznosci; skin layout ma poprawne dotychczas sprawdzone zakresy. | Czy po naprawie RIGID skin deformuje sie tak samo jak geometria bazowa, a potem jak rig. |

Nie wolno uznac Toolsetu, own-readbacku ani samego logu `Loading Module` za
zamkniecie ktoregokolwiek toru.

## 2. Kryteria zakonczenia

Cel zostanie uznany za osiagniety tylko, gdy ten sam hash wygenerowanego H1:

1. przejdzie deterministic writer/readback i nowe gate'y binary MDL;
2. bedzie widoczny i rozpoznawalny w Aurora Toolset;
3. bedzie widoczny i rozpoznawalny w kliencie NWN w tej samej fixture;
4. zachowa poprawna sylwetke przy 30 sekundach idle oraz przy kontrolowanym
   ruchu; dla wariantu skin dodatkowo przy jednym ruchu kosci;
5. otrzyma packet proofu z hashami MDL/MDX/HAK/MOD, screenshotami z obu
   aplikacji, tozsamoscia modulu/obszaru/creature oraz wynikiem logow.

Przez "rozpoznawalny" rozumiemy sylwetke H1, a nie marker zaznaczenia,
niezidentyfikowana plame, cien, fragment kafla ani sam fakt istnienia draw
calla.

## 3. Kolejnosc dzialania

### Faza 0 - nowy testowy model Meshy w tym samym proofie co kontrola dodatnia

Testowym assetem ma byc **nowy, niezalezny model wygenerowany bezposrednio w
Meshy**, a nie kolejny przebieg H1 ani zasob referencyjny NWN. `c_tortoise`
pozostaje tylko kontrola dodatnia tego samego uruchomienia: pozwala stwierdzic,
czy obserwacja klienta jest wiarygodna. Oba modele maja byc w jednym HAK-u,
MOD-zie, obszarze i proofie.

Nowy model Meshy przechodzi w calosci przez:

```text
nowy Meshy GLB -> own ingest -> own IR/writer -> native binary MDL/MDX
-> own 2DA/HAK/MOD -> Aurora -> NWN
```

Pierwszy model `M0_MESHY_RIGID_CONTROL` ma celowo ograniczac zmienne:

- wyrazna, stojaca sylwetka (rekomendowany prompt: prosty low-poly kamienny
  golem lub statua), bez broni, efektow, przezroczystosci i ruchomych dodatkow;
- jedna dominujaca, nieprzezroczysta tekstura; docelowo jeden RIGID trimesh;
- inna topologia, tekstura i skala niz H1;
- brak zewnetrznej armatury i animacji; minimalne runtime clips tworzy tylko
  nasz profil direct creature;
- caly model jest nowym materialem wejsciowym z Meshy, a nie przerobionym
  assetem NWN, Blenderem ani syntetyczna bryla z kodu.

| Wynik jednego proofu | Interpretacja |
| --- | --- |
| tortoise PASS, M0 poprawny w NWN | bledy H1 sa zrodlo- lub rig-specyficzne; RIGID writer nie jest globalnie odrzucony |
| tortoise PASS, M0 zdeformowany jak H1 v20 | problem jest wspolny dla Meshy ingest/writera RIGID mesh/MDX |
| tortoise PASS, M0 niewidoczny | bledy sa w pipeline nowego assetu Meshy albo RIGID writerze; proof jest nadal wiarygodny |
| tortoise FAIL | nie interpretowac wyniku M0; najpierw naprawic proof/resource-resolution lane |

Po zamknieciu M0 nalezy dodac drugi, odmienny model Meshy `M1` dopiero jako
regresje ogolnosciowa. M1 nie moze wejsc do tych samych A/B co naprawa H1,
aby nie mnozyc zmiennych przed znalezieniem root cause.

### Faza A - zamrozenie kontraktu proofu

1. Ustalic jeden kanoniczny modul, HAK, obszar i pozycje fixture. Kazdy wariant
   ma zmieniac wylacznie jeden opisany element modelu.
2. Do manifestu proofu dopisac: hash kazdego zasobu, `Appearance_Type`, row
   `appearance.2da`, wspolrzedne creature, pozycje/kierunek kamery i wersje
   klienta NWN:EE.
3. Bramka wizualna musi pokazywac jednoczesnie tozsamosc instancji w Toolsecie
   oraz obiekt w viewport. Dla NWN: capture startowy i capture po kontrolowanym
   zblizeniu do tej samej pozycji fixture.
4. Nie zmieniac jeszcze writera ani listy animacji.

**Wynik rozstrzygajacy:** od tej chwili kazde `PASS` lub `FAIL` daje sie
przypisac dokladnie do wersji artefaktu i ustawienia proofu.

### Faza B - dodatnia kontrola wlaczona do tego samego proofu

1. Umiescic `c_tortoise` i M0 w **tym samym** HAK-u, MOD-zie i obszarze, ale
   w dwu jasno roznych, zapisanych pozycjach fixture. Kazdy otrzymuje wlasny
   wiersz `appearance.2da`. Tortoise pozostaje kopia proofowa z pelnym P-REF;
   nie staje sie fixture ani zasobem produktu.
2. Wykonac jeden standardowy proof Aurora -> NWN na drugim ekranie, bez nowego
   adaptera i bez zmiany konfiguracji gry.
3. Najpierw ocenic tortoise, a potem M0, zgodnie z tabela w Fazie 0.

### Faza C - Aurora First: wspolny kontrakt RIGID mesh/MDX

1. Z dekompilacji Aurora wyprowadzic jawna tabele znaczenia tylko pol
   wykorzystywanych przez `FUN_00a3e19c`, `FUN_00a5e508` i dalsza sciezke
   przygotowania geometrycznego: naglowek MDX, strumienie POSITION/UV/NORMAL,
   `indexCounts/indexOffsets`, indeksy, node transform i lokalne bounds.
2. Porownac te pola niezaleznym od own-readback dekoderem bajtow dla:
   - native binary reference,
   - H1 v20 RIGID,
   - minimalnej clean-room fixture z jednym trojkatem.
3. Zdefiniowac konkretny invariant dla kazdej roznicy, np. poprawny stride,
   kolejnosc/typ indeksow, poczatek MDX, znaczenie node transform albo
   interpretacja bounds. Nie kopiowac zadnego referencyjnego payloadu.
4. Najpierw napisac test dla wykrytej roznicy, potem najmniejsza poprawke
   writera i readbacku.

**Zakaz:** nie zmieniac rownoczesnie MDX, skin map, inverse-bind i animacji.
Taka proba nie identyfikuje przyczyny.

### Faza D - proof najprostszego RIGID

1. Wygenerowac H1 RIGID z tym samym GLB, tekstura, bounds i fixture co v20.
2. Wykonac Toolset -> NWN proof zgodnie z Faza A.
3. Jezeli sylwetka jest bledna, pozostac w Fazie C i porownac tylko nowa,
   widoczna rozbieznosc binarna. Jezeli jest poprawna, zamrozic hash jako
   `RIGID-RUNTIME-BASELINE-PASS`.

To jest bramka bezwzgledna: nie wolno wlaczac skinu, dopoki RIGID nie jest
poprawny w kliencie gry.

### Faza E - izolacja skina i animacji

Po RIGID PASS wykonac ponizsza macierz, po jednym wariancie naraz:

| Wariant | Zmiana wzgledem poprzedniego | Warunek PASS |
| --- | --- | --- |
| S1 | jeden skin, jeden node/kosc, waga `1.0`, identity inverse-bind, bez ruchu | obraz identyczny z RIGID |
| S2 | aktywny root/base-pose jednej kosci | obraz identyczny z S1 |
| S3 | kontrolowany ruch jednej kosci w `cpause1` | deformacja jest lokalna i przewidywalna |
| S4 | docelowy rig oraz wagi H1 | poprawna sylwetka idle/ruch |
| S5 | minimalne native lifecycle clips potrzebne dla direct creature | poprawne spawn/idle/ruch, bez regresji geometrii |

Kazdy wariant dostaje niezalezny test strukturalny, hash i proof. Nie dodajemy
kolejnych aliasow animacji bez dowodu z resolvera stanu w dekompilacji.

### Faza F - kontrola kompilatora gry (opcjonalna, po zgodzie wlasciciela)

`compileloadedasciimodels` jest wartosciowym zewnetrznym A/B: moze wyprodukowac
binary MDL zgodny z aktualnym klientem dla celow **read-only porownania
strukturalnego**. Nie jest czescia produktu ani sposobem na maskowanie bledu
wlasnego writera.

Ta faza wymaga osobnej zgody, poniewaz klient zapisuje wynik do
`Documents\\Neverwinter Nights\\modelcompiler`. Po zgodzie artefakty sa
kopiowane wylacznie do `proof-output`, hashowane i oznaczone provenance; nie
trafiaja do zrodel, fixture, HAK-a produktu ani commita.

## 4. Regula decyzji po kazdej probie

1. Najpierw sprawdzic, czy dodatnia kontrola proofu przeszla.
2. Potem sprawdzic, czy zmienil sie tylko jeden czynnik A/B.
3. Następnie porownac log, hashe, screenshot Toolsetu i screenshot NWN.
4. Dopiero wtedy oznaczyc hipoteze jako potwierdzona albo odrzucona w audycie.
5. Gdy wynik jest niejednoznaczny, poprawic capture/bramke proofu, nie writer.

## 5. Oczekiwany pierwszy krok

Pierwsza implementacyjna akcja to **wygenerowanie prostego M0 w Meshy i
spakowanie go razem z dodatnia kontrola tortoise do jednego proofu**. Tortoise
nie zastepuje M0 i M0 nie jest odlozony na pozniej: oba sa konieczne do
jednoznacznej interpretacji pierwszego runu.

Po wyniku wspolnego proofu plan przejdzie albo do naprawy proofu, albo
bezposrednio do Fazy C/D i porownania wspolnej sciezki RIGID mesh/MDX z
dekompilacja Aurora.
