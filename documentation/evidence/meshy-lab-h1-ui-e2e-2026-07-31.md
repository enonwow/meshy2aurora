# Meshy Lab H1 UI E2E — wynik testu 2026-07-31

Status: `FAILED / NIE JEST GOTOWE JAKO PELNY PIPELINE`

## Cel

Sprawdzic realnie, przez UI aplikacji Studio, a nie przez runner albo reczny
writer, sciezke:

`Meshy Lab -> platny Image-to-3D -> rig -> piec animacji -> zweryfikowany GLB -> Source -> Inspect -> Build -> binary readback -> MOD/HAK`.

Test mial dodatkowo uzyc innego idle niz poprzednie modele, aby sprawdzic
Creature stojace w innej pozie.

## Wejscie i konfiguracja

- Studio: `http://localhost:5174/`, Compose z `VITE_MESHY_LAB=1`.
- Local Bridge: polaczony przez jawne `Connect local bridge`.
- Referencja obrazu:
  `C:\Users\enonw\AppData\Local\Temp\codex-clipboard-1694ba6a-a988-4ab6-b6a4-05e54622214c.png`.
- SHA-256 referencji:
  `96b784db771791391d5c57da5317762b5072448ac19382f1176e49dbf96a64f2`.
- Profil: `H1 Humanoid Animated`.
- Zrodlo: `IMAGE`.
- Meshy model: `Meshy 6`.
- Pose: `T-Pozycja`.
- Target polycount: `300000`.
- Rig: standard humanoid.
- Mapowanie zamowionych akcji:
  - `244 Idle_4 -> cpause1`;
  - `198 -> ca1slashl`;
  - `30 Casual_Walk -> cwalk`;
  - `16 RunFast -> crun`;
  - `8 Dead -> cdead`.
- Ekran review deklarowal `38 credits maximum`.

## Fakty potwierdzone przez UI

1. Platny run zostal utworzony dopiero po ekranie review i kliknieciu
   `Generate model`.
2. UI przeszlo przez `PREVIEW -> REFINE -> RIG -> ANIMATE` i osiagnelo
   `Run ready`, `100%`.
3. Meshy viewport zaladowal rzeczywisty humanoidalny model zgodny kierunkowo z
   referencja.
4. Viewport podal:
   - `296280` trojkatow;
   - `193528` wierzcholkow.
5. `Import verified GLB to Source` przekazal plik
   `meshy-h1-humanoid-animated.glb`, `21450080` bajtow, SHA-256
   `8ac6bf95436e59a2e3c2bac3d1fae46af1df7f40fb40fdce67fbce19acffbf4f`.
6. Source Inspect potwierdzil:
   - `1` mesh;
   - `296280` trojkatow;
   - `24` bones;
   - conversion eligible.
7. Build zakonczyl sie binary readbackiem `PASS`, pustym semantic diffem i
   zachowal dokladnie `296280` trojkatow.
8. Build pokazal gotowe artefakty, zanim dev UI zostalo przeladowane:
   - `chvuhcjygz5fchl5.hak`, SHA-256
     `50f50fa40c6afdf09dc9ec25b8903be858a93da92b80adf69344586e31457e71`;
   - `cmvuhcjygz5fchl5.mdl`, SHA-256
     `433501fa415de3d6c4a570b6ec8c5e7836f6f360beee332461110397f3f466cc`;
   - `ctvuhcjygz5fchl5.tga`, SHA-256
     `4aba083ad36eaa919d8e9cb7f5dde7ad19431adf12969149387c7405887f0d2b`;
   - `cdvuhcjygz5fchl5.mod`, SHA-256
     `1f481c74be01bd515097feeac18f2654f8cd6f1dc03b27afa0e3a42d86262e03`.

## Blad P0 — zamowione animacje nie dotarly do Source

Fakt:

- review wyslalo piec jawnych mapowan;
- run osiagnal `ANIMATE` i `READY`;
- Source Inspect zobaczyl tylko jeden klip:
  `Armature|Idle|baselayer`, `4.03 s`;
- readback MDL pokazal `42` stany, ale sa to stany wypelnione przez pipeline z
  jednego klipu zrodlowego;
- m.in. `cdead` ma `0.03 s`, wiec nie jest zamowionym Meshy action `8 Dead`;
- statystyka porownawcza UI podala wprost `Animation clips: Source 1,
  Converted 42`.

Ocena:

To nie jest piecioanimacyjny model Meshy przepuszczony do NWN. Sam Build jest
poprawny strukturalnie, ale cztery z pieciu platnych klipow nie sa obecne w
wejsciu Studio. Dokladna granica awarii pozostaje do ustalenia pomiedzy:

- scaleniem action GLB w Local Bridge;
- wyborem canonical artifact przez `/artifact`;
- importem artifactu do Source;
- odczytem wielu animacji przez GLB inspector.

Nie wolno wskazywac jednego z tych komponentow jako przyczyny bez zachowanego
GLB/provenance i porownania zawartosci `animations[]`.

## Blad P0 — utrata gotowego handoffu po reloadzie dev UI

Fakt:

- Build zakonczyl sie i wyswietlil MOD/HAK oraz ich hashe;
- przed kliknieciem Download Studio zostalo ponownie zainicjalizowane o
  `2026-07-31T20:44:46.045Z` wedlug logu przegladarki;
- po inicjalizacji Source byl pusty i artefakty w pamieci przegladarki nie byly
  juz dostepne;
- pliki nie zostaly pobrane ani zainstalowane, wiec nie istnieje handoff
  `ready_for_owner_proof`.

Hipoteza:

Najbardziej prawdopodobnym wyzwalaczem byl HMR/reload Vite podczas rownoleglych
zmian w aktywnym, brudnym worktree. Sam fakt reloadu i utraty stanu jest
potwierdzony; dokladny plik albo proces, ktory go wyzwolil, nie zostal
zidentyfikowany.

## Blad P1 — brak odzyskania runu Image-to-3D

Fakt:

- po ponownym polaczeniu `Assets / Meshy history` pokazalo `0` zadan;
- kod Local Bridge pobiera na tej stronie tylko
  `/openapi/v2/text-to-3d` i filtruje `text-to-3d-preview/refine`;
- zakonczony run Image-to-3D nadal moze istniec w pamieci Bridge, ale UI po
  reloadzie nie zna jego lokalnego `run.id`, a Bridge nie udostepnia listy
  lokalnych runow.

Wniosek:

Po reloadzie nie da sie przez obecna aplikacje bezkosztowo odzyskac dokladnego
zweryfikowanego artifactu tego runu Image-to-3D. Nie nalezy generowac drugiej
platnej iteracji jako obejscia.

## Obserwacja billingowa do izolowanego audytu

- saldo widoczne przed runem: `456`;
- saldo po ponownym polaczeniu: `408`;
- obserwowana roznica w oknie testu: `48`;
- review deklarowal maksimum `38`.

Nie jest to jeszcze dowod, ze pojedynczy run zuzyl `48`, poniewaz workspace i
konto mogly byc uzywane rownolegle przez inne zadanie. Niezbedny jest
izolowany test salda powiazany z jednym `run.id` i faktycznym
`consumedCredits` z provenance.

## Werdykt i kolejny bezpieczny krok

`Generation path = PASS`, `Source/Build geometry path = PASS`,
`requested Meshy animation preservation = FAIL`,
`durable demo handoff = FAIL`.

Przed kolejnym platnym runem:

1. dodac regresje, w ktorej piec mapowan daje dokladnie piec nazwanych klipow
   w zaimportowanym GLB;
2. zachowac i porownac `animations[]` po merge oraz po pobraniu `/artifact`;
3. dodac odzyskiwanie lokalnych runow Image-to-3D po reloadzie albo trwaly,
   bezpieczny identyfikator wznowienia;
4. powtorzyc E2E dopiero po zielonych testach offline;
5. dopiero potem pobrac i zainstalowac exact MOD/HAK do proofu wlasciciela.
