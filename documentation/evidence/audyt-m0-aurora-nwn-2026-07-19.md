# Audyt M0 Meshy, Aurora Toolset i NWN:EE — 2026-07-19

Status: `RUNTIME PROOF MISSING`.

Ten audyt był odczytowy. Nie uruchamiał Aurora Toolsetu ani NWN i nie zmieniał instalacji gry.

> Korekta skilli 1.0.2, odczytana po sporządzeniu pierwszej wersji audytu:
> granica `reference-only` nie ogranicza wspólnych skilli Aurora, ich
> kanonicznego runnera ani zweryfikowanych atomów. Dla pracy Toolsetowej trzeba
> używać tego centralnego tooling bezpośrednio, a nie pisać lokalnego runnera
> albo adaptera. Poniższe sformułowania o konieczności utworzenia lokalnego
> adaptera są zastąpione przez tę korektę.

## Konkluzja

Nie ma dowodu, że ostatni model wygenerowany w Meshy działa w Aurora Toolset ani w NWN:EE.
Zweryfikowano łańcuch offline (`GLB -> własny writer -> MDL/TGA/2DA/HAK/MOD`) oraz zgodność zainstalowanych plików z aktualnym pakietem.
Brakuje natomiast bramek live: Area viewport, `Build -> Test Module` oraz runtimeowego capture NWN.
Import GLB, hash, otwarcie modułu ani test jednostkowy nie są proofem działania modelu w grze.

## Jaki to model

Aktywny kandydat runtime to `M0_MESHY_RIGID_CONTROL`, resref `m2a_m0p01`, tekstura `m2a_m0t01`.
Jest to nowy, statyczny model Meshy: mała niskopoligonowa, malowana kamienna statua golema, stojąca centralnie, z jednym nieprzezroczystym materiałem, bez broni i części ruchomych.
Nie jest to referencyjny `c_tortoise` ani starszy H1.
M0 jest celowo prostą kontrolą RIGID: źródło nie ma skina ani animacji źródłowej, a siedem nieruchomych klipów direct-creature generuje Meshy2Aurora.

Źródłowy GLB: SHA-256 `aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1`, 8 581 684 B, 2 380 wierzchołków, 1 569 trójkątów.
Najnowszy materializowany wariant to `proof-output/m0-meshy-golem-20260719-v5`.

| Artefakt | SHA-256 | Status |
| --- | --- | --- |
| `m2a_m0p01.mdl` | `971a15b0990fb0f308be929e23fc3fb70c8c7342c0c39246613687f6c6c84e66` | `verified` offline |
| `m2a_m0_proof.hak` | `7211c1a016c2b36320f7ce2a399d2833b8f2d7901d2896261d01a93161600200` | zgodny z zainstalowanym HAK |
| `m2a_m0_proof.mod` | `d63b9b71c421ee07ad678619be92b3f624febfecdcd4b09ebd9fdcaa2e13c753` | zgodny z zainstalowanym MOD |

Aktualny `appearance.2da` dopisuje M0 do wiersza `15109`.
Starszy opis z 2026-07-18 podawał wiersz `15101`, więc aktualny manifest musi wskazywać v5.

## Stan dowodu

| Punkt akceptacji | Status | Dowód albo powód |
| --- | --- | --- |
| Meshy `REFINE` i odzyskanie GLB | `verified` | dokumenty M0 z 2026-07-18 |
| Własny MDL/TGA/2DA/HAK/MOD i readback | `verified` | 3 targetowane testy M0 przeszły w tym audycie |
| Instalacja bieżącego HAK/MOD | `verified` | bieżące hashe plików gry są identyczne z v5 |
| Otwarcie modułu | `verified` historycznie | `m0-resume-20260719/module-open.json`; nie dowodzi viewportu |
| Poprawny viewport Toolsetu dla v5 | `missing` | brak zatwierdzonego adaptera i capture |
| `Build -> Test Module` | `missing` | lokalne CLI nie udostępnia tej akcji |
| Log, widoczność i sylwetka w NWN | `missing` | brak uruchomienia NWN i packetu runtime |

W chwili audytu nie było procesu `nwtoolset.exe` ani `nwmain.exe`.
Jest to czysty preflight, a nie wynik proofu.

## Rola skilli i rzeczywista luka

`aurora-toolset-operate` i `aurora-toolset-prove` są obowiązującym standardem Aurora Web.
Wymagają jednej sesji, PID-first recovery, braku globalnego inputu, natychmiastowego readbacku, niezależnego capture i fail-closed po modalu lub Access Violation.
Nie są jednak gotową biblioteką sterującą konkretnymi HWND, menu i kontrolkami bieżącej sesji Toolsetu.

Zgodny podział odpowiedzialności:

1. Skill dostarcza nieosłabialny kontrakt i bramki.
2. Projekt ma mały, wersjonowany i przetestowany adapter tylko dla własnej akcji native.
3. Adapter zapisuje manifest, a proof przechodzi jego walidator.

`PROJECT_RULES.md` i `orchestrator-state.yaml` oznaczają `C:\Projects\aurora-web` jako `reference-only` dla kodu produktu, assetów, fixture i walidacji produktu.
Skill 1.0.2 rozstrzyga, że nie dotyczy to wspólnego operating skill, kanonicznego runnera ani zweryfikowanych atomów.
Trzeba wykonywać centralny runner bezpośrednio; tworzenie albo kwalifikowanie project-local Toolset runnera lub UI adaptera jest teraz wyraźnie zakazane.

## Dlaczego nie ma obecnie obsługi Aurora/NWN

### P0 — lokalny adapter nie może być drogą proofu

Lokalny `tools/m2a-aurora-proof.mjs` jest plikiem nieśledzonym przez Git i nie może być użyty jako droga proofu po korekcie skilli 1.0.2.
Publiczny CLI wystawia wyłącznie `doctor`, `plan`, `open`, `inspect-open-dialog` i `complete-open-dialog`.
Pomoc CLI wprost mówi, że tylko adapter otwarcia jest live, a `Viewport` i `Test Module` pozostają niedostępne.

Nie istnieje lokalna komenda ani packet dla:

- wyboru Area i pełnego readbacku `TfrmViewerArea`;
- niezależnego capture `TScrollBox` z manifestem i inspekcją obrazu;
- weryfikacji fixture: najpierw `c_tortoise`, potem M0;
- natywnego `Build -> Test Module`, obserwacji startu NWN i logu ładowania;
- runtimeowego PNG/MP4/readbacku z tożsamością modułu, Area i creatury.

W tekście PowerShell dla `complete-open-dialog` jest szkic `viewM0Area`, ale znajduje się pomiędzy `<#` i `#>`.
Jest komentarzem PowerShell, nie wykonywalnym adapterem JavaScript; nie ma komendy CLI, testu ani proofu live.
Nie należy go naprawiać ani rozwijać; trzeba użyć centralnych atomów.

### P1 — centralna droga Toolsetu istnieje, runtime NWN jeszcze nie

Lokalny otwieracz wywołuje `SetWindowPos` w liniach 369 i 412, mimo że aktualny standard zakazuje takiego przenoszenia Toolsetu.
To dodatkowy powód, aby go wycofać z live route.

Centralny `run-aurora-toolset-module-proof.mjs` jest parametryzowany i może tworzyć świeży, nieistniejący moduł proof, przyłączyć `m2a_m0_proof.hak` i ustawić `M2A_M0_MESHY_RIGID`.
Centralny `aurora-toolset-viewport-proof.mjs` dostarcza capture zweryfikowanego `TScrollBox` dla zadanego modułu i Area.
Po poprawce skilli jest to jedyna dopuszczona droga do proofu Toolsetu.

Końcowy runtime NWN pozostaje jednak realnie `missing`: mapa centralnego standardu ma `AUR-S07-04 Test Module = NIE` oraz `AUR-S07-05 Runtime proof = NIE`.
Zasada skilla nakazuje w tej sytuacji zatrzymać tylko tę lane i zapisać brakujący centralny atom; nie wolno tworzyć lokalnego launchera NWN ani adaptera zastępczego.

### P1 — poprzedni pakiet naprawiono, ale nie dowiedziono nowego

Pierwszy M0 otwierał Area, lecz viewport był jednolicie pusty.
Potwierdzoną przyczyną była syntetyczna lista `tdc01` niebędąca renderowalnym precedensem.
Zmiana na cztery tile `5` utworzyła v2–v5, ale dla v5 nie ma świeżego capture viewportu.
Wyniku błędnej paczki nie wolno przenosić na nowy hash MOD/HAK.

Dokumentacja nie określa jednego aktualnego stanu:

- `m0-meshy-runtime-preflight-2026-07-18.md` nazywa lokalny runner niezatwierdzonym i wykluczonym z live route.
- `m0-toolset-recovery-blocker-2026-07-19.md` nazywa ten sam nieśledzony plik „approved”, a zarazem potwierdza brak adaptera viewportu i `Test Module`.
- Aktualny v5 ma wiersz 15109, gdy starsze packet-y używały 15101 i niższych.
- `orchestrator-state.yaml` ma `last_updated: 2026-07-14` i nie odzwierciedla zdarzeń M0 z 18–19 lipca.

To błąd ciągłości/provenance, nie wada skilla.

## Stan projektu w obecnym worktree

Audyt dotyczy bieżącego, nieczystego worktree: `git status --short` wskazuje 70 zmienionych lub nieśledzonych ścieżek.
`git diff --check` nie wykrył błędów białych znaków, ale nie ma jednej zatwierdzonej rewizji M0 ani adaptera.

| Sprawdzenie | Wynik |
| --- | --- |
| `cargo fmt --all -- --check` | `failed` — zmodyfikowane pliki Rust nie są sformatowane |
| `cargo clippy --workspace --all-targets -- -D warnings` | `failed` — `clippy::filter_map_bool_then` w `crates/m2a-core/src/profile_a.rs:890` |
| `cargo test --workspace` | `failed` — 2 testy `crates/m2a-core/tests/gff.rs` |
| Targetowane testy M0 | `passed` — 3/3 w `model_pipeline.rs` |
| `node --test tools/m2a-aurora-proof.test.mjs` | `passed` — 6/6 kontraktów CLI, nie proof live |
| `npm run typecheck` | `passed` |
| `npm test` | `passed` — 28 plików / 164 testy |
| `npm run test:worker-integration` | `passed` — 2 pliki / 5 testów; pozostaje ostrzeżenie o `@vitest/browser/context` |

Własny pipeline i Studio są częściowo sprawdzalne, lecz całość nie jest zielona, a native runtime pozostaje `missing`.

## Kolejność naprawy

1. Ustalić jedną zatwierdzoną rewizję M0: v5, jego hashe, wiersz `15109` i jeden manifest.
   Zaktualizować ledger/orchestrator i wycofać sprzeczne określenie „approved” dla nieśledzonego pliku.
2. Naprawić offline: formatowanie, Clippy, dwa testy GFF oraz przestarzały import Vitest.
   Nie mieszać tej naprawy z proofem runtime.
3. Uruchomić tylko centralne testy kontraktu i dry-run dla parametryzowanego disposable-module proof z HAK-iem `m2a_m0_proof` oraz appearance `M2A_M0_MESHY_RIGID`, a następnie jeden świeży live Toolset run przez kanoniczny runner.
4. Zakończyć Toolset lane capturem centralnego `aurora-toolset-viewport-proof.mjs`; oznaczyć wynik wyłącznie jako `verified`, `failed` albo `missing`.
5. Dla NWN zapisać blocker `missing_central_AUR-S07-04_AUR-S07-05_atoms` i czekać na centralny standard/atom albo zaakceptowany user-provided runtime capture ingestion. Nie tworzyć lokalnego launchera.

## Materiał dowodowy

- `documentation/evidence/m0-meshy-app-generation-2026-07-18.md`
- `documentation/evidence/m0-meshy-runtime-preflight-2026-07-18.md`
- `documentation/evidence/m0-area-tileset-correction-2026-07-18.md`
- `documentation/evidence/m0-toolset-recovery-blocker-2026-07-19.md`
- `proof-output/m0-meshy-golem-20260719-v5/reports/summary.json`
- `proof-output/m0-resume-20260719/module-open.json`
- `tools/m2a-aurora-proof.mjs` i `tools/m2a-aurora-proof.test.mjs`
- Bieżące skille `aurora-toolset-operate` i `aurora-toolset-prove` oraz ich kanoniczne standardy Aurora Web, odczytane wyłącznie jako reference contract.

## Follow-up: luki w skillach 1.0.2 zgłoszone do Aurora Web

W dniu 2026-07-19 wysłano podsumowanie do tasku `019f6203-f618-7b82-8800-6c6a3e8c044f`.
Korekta 1.0.2 właściwie wybiera centralny runner zamiast lokalnego adaptera, ale pozostawia następujące otwarte kontrakty:

- fast module runner ma dokumentowany profil M6, mimo że kod dopuszcza parametry M0; potrzebny jest formalny profil i manifest M0;
- runner obsługuje tylko jeden HAK i jedną creature, a proof M0 wymaga ordered HAK list oraz kontroli `c_tortoise` i M0 w jednym Area;
- nie ma decyzji, czy świeży disposable MOD z identycznym HAK/model hash zastępuje dokładny `m2a_m0_proof.mod`;
- AUR-S07-04 (`Test Module`) i AUR-S07-05 (runtime proof) nadal są `NIE`, więc nie ma centralnej ścieżki finalnego proofu NWN;
- preflight nie zapisuje/nie weryfikuje pełnej tożsamości HAK/MDL/TGA/2DA/hashów wymaganych przez proof M0;
- capture pełnego frame'u używa tymczasowego `SetWindowPos(HWND_TOPMOST)` i `CopyFromScreen`, co wymaga jawnego rozstrzygnięcia względem polityki z-order i exact viewport capture;
- brak AUR-SNN-NN, funkcjonalnego manifestu oraz validatora dla importowanego modelu w HAK;
- trzeba określić project-local `--artifactDir`/`--out`, aby centralny tooling nie mieszał artefaktów proofu produktu z domyślnym `.codex-tmp` Aurora Web.

Warunkiem odblokowania M0 jest centralny profil/manifest, proof Toolsetu z kontrolą dodatnią i exact viewport artifact oraz centralny atom runtime albo zatwierdzona schema ingestu capture dostarczonego przez użytkownika.

## Finalna korekta shared proof contract 1.0.3

Po follow-upie task Aurora zaktualizował shared proof contract do `1.0.3` i
zwalidował zmianę. Wersja jest jednakowa w routing references oraz w
`aurora-toolset-skill-consistency-audit-2026-07-19.md`.

`aurora-toolset-operate` preflight precyzuje teraz, że projekt może tylko
wybierać spośród kanonicznych tras i wzmacniać safety/proof. Nie może zakazać,
zastąpić ani wymagać lokalnego substytutu dla shared operator tooling,
kanonicznego runnera ani verified native atoms.

Kontrakt 1.0.3 zamienia wcześniejsze niedopowiedzenia M0 w fail-closed bramki:
bez centralnego, wersjonowanego profilu M0/manifestu z jego dry-run contract
testem nie uruchamia się live Toolsetu. Finalny runtime NWN jest nadal
`missing` do czasu centralnego atomu AUR-S07 albo zwalidowanej schemy ingestu
capture dostarczonego przez użytkownika.

## Live follow-up: centralny profil M0 wykonał się do Area, 2026-07-19

Po wdrożeniu centralnego profilu `m0-static-rigid-v5` przeprowadzono jego
kanoniczne testy kontraktu, `m0-dry-run` i live preflight. Preflight potwierdził
dokładny hash `m2a_m0_proof.mod`, oba hashe HAK i brak procesów Toolset/NWN.
Jedno otwarcie Toolsetu zakończyło się prawidłowo: PID `14628`, dokładny
moduł `m2a_m0_proof.mod`, brak Access Violation i brak zmiany `nwtoolset.ini`.

Etap `run-queue` nie przeszedł. Jest to **błąd centralnej trasy, nie dowód
wady modelu**: kolejka M0 podała `name: m2a_m0proof_area`, natomiast aktualny
native `TTreeView` Toolsetu zawiera nazwę wyświetlaną `Meshy2Aurora M0 static
runtime proof area` dla tego Area. Runner zatrzymał się fail-closed z
`M0_TOOLSET_QUEUE_UNVERIFIED` / `toolset_area_queue_exception` przed otwarciem
Area i przed capturem `TScrollBox`.

Stan po odczycie: jedna responsywna sesja PID `14628`, bez widocznego modalu;
pozostawiona otwarta zgodnie ze standardem. Nie powtarzano niezmienionej akcji
i nie utworzono lokalnego adaptera. Zgłoszono do tasku standardu
`019f6203-f618-7b82-8800-6c6a3e8c044f` wymaganie: centralna kolejka musi wiązać
resref Area z dokładnym tekstem natywnego węzła oraz mieć test tego mapowania.

Aktualne statusy:

| Punkt dowodu | Status | Powód |
| --- | --- | --- |
| Profil, hashe i preflight M0 | `verified` | centralny walidator i preflight przeszły |
| Live otwarcie Toolsetu | `verified` | jedna sesja, dokładny MOD, brak AV |
| Toolset Area / `TScrollBox` | `failed` | centralny mismatch resref–nazwa drzewa zatrzymał kolejkę |
| Widoczność modelu M0 w Toolsecie | `missing` | Area i capture nie powstały |
| NWN runtime AUR-S07 | `missing` | brak user-provided packetu runtime / artefaktów |

Artefakty przebiegu: `proof-output/m0-static-rigid-v5-toolset-20260719/session-state.json`
i `proof-output/m0-static-rigid-v5-toolset-20260719/manifest.json`.

## Live follow-up: Area otwarte, viewport jednolity, 2026-07-19

Centralny standard naprawił mapowanie resref–nazwa węzła i jego dwa testy
kontraktu przeszły. Ta sama zweryfikowana sesja otworzyła następnie właściwe
Area: manifest kolejki ma `status: completed`, wpis `opened`, resref
`m2a_m0proof_area`, native nazwę `Meshy2Aurora M0 static runtime proof area`
i viewer ` - m2a_m0proof_area`.

Następna bramka, centralny `aurora-toolset-viewport-proof.mjs`, odrzuciła
capture jako `toolset_viewport_scene_pixels_unavailable`. `GetWindowDC`
walidowanego `TScrollBox`, fizyczny capture po weryfikacji właściciela okna i
crop `PrintWindow` zwróciły jednolite piksele. Nie użyto trybu
`allowUniformScene`, nie zmieniono kamery ani nie próbowano lokalnego obejścia.

Wynik jest zatem: otwarcie modułu i Area `verified`; wizualny proof Toolsetu
`failed`; twierdzenie, że model M0 renderuje się poprawnie, nadal `missing`.
NWN/AUR-S07 pozostaje `missing` — brak runtime packetu użytkownika.

Zgłoszono też błąd ciągłości centralnego managera do tasku standardu
`019f6203-f618-7b82-8800-6c6a3e8c044f`: po takim błędzie `runM0Queue` ma już
zweryfikowane otwarcie Area, ale nie zapisuje `area_ready` ani lane failure do
session ledgeru przed rethrow. Wymagana jest centralna poprawka: trwały
area-readback i blocker manifest z dokładnym błędem capture oraz test negatywny
tej ciągłości. PID `14628` pozostał otwarty i responsywny zgodnie ze standardem.

## Finalizacja centralnej ciągłości runu, 2026-07-19

Agent standardu poprawił manager i dostarczył publiczną, niemutującą komendę
`m0-reconcile-capture-failure` dla capture’ów, które zakończyły się przed
wdrożeniem poprawki ledgeru. Jej test publicznego entrypointu oraz dwa kontrakty
M0 przeszły. Komenda została wykonana raz dla tego runu z dosłownym błędem
`toolset_viewport_scene_pixels_unavailable: physicalFallback=validated_viewport_but_uniform_pixels`.

Centralny ledger jest teraz spójny: `phase: area_ready`, Area
`m2a_m0proof_area`, jedna responsywna sesja PID `14628` oraz trwały lane failure
`m0_tscrollbox_capture` / `M0_TOOLSET_TSCROLLBOX_CAPTURE_FAILED`. Manifest
blockera `proof-output/m0-static-rigid-v5-toolset-20260719/m0-toolset-capture-blocker.json`
wiąże dokładny MOD, hash, Area, native nazwę węzła, manifest kolejki i warunek
wznowienia: nie ponawiać niezmienionego capture’u.

Wszystkie znalezione braki standardu zostały zgłoszone, poprawione centralnie
i zwalidowane kontraktami. Nie zmienia to wyniku produktu: Toolset nie dostarczył
niejednolitego obrazu sceny, zatem nie ma dowodu poprawnego renderu M0; runtime
NWN również nie ma packetu i pozostaje `missing`.

## Krytyczna korekta: wygenerowana Area jest nieprawidłowa, 2026-07-19

Po ręcznym otwarciu dokładnie tego samego artefaktu użytkownik ustalił przyczynę,
której poprzednia próba capture nie mogła rozstrzygnąć. Native Toolset odrzucił
zapis Area komunikatem:

```text
This area contains 2 creature(s) located in invalid locations.
Area was NOT saved!!
```

Są to oba fixture'y M0 widoczne w drzewie Area: `Reference tortoise control` i
`Meshy M0 static rigid control`. Widok Area jest czarny, a okno
`Inaccessable Objects` wskazuje niedostępne obiekty. Usunięcie ich przez
użytkownika w bieżącej, niezapisanej sesji nie naprawia wygenerowanego MOD ani
nie stanowi dowodu; Toolset wprost odmówił zapisu tego stanu.

To wiąże czarny viewport z **nieprawidłową geometrią/generated tile layoutem
Area**, a nie z samym modelem `m2a_m0p01`. Preflight przed otwarciem sesji
potwierdził, że wybrany MOD odpowiada pakietowi
`m0-meshy-golem-20260719-v5`:

```text
m2a_m0_proof.mod
SHA-256 d63b9b71c421ee07ad678619be92b3f624febfecdcd4b09ebd9fdcaa2e13c753
```

Po otwarciu bieżącej sesji plik MOD jest zablokowany przez Toolset, więc nie
odczytano jego hasha ponownie. Ten stan oznacza `unreadable/unknown`, nie
potwierdzony drift ani nowy hash. Niezapisane zmiany użytkownika nie stanowią
artefaktu do proofu.

### Przyczyna w generatorze

`proof_module.rs` tworzy syntaktycznie poprawne `ARE`: `tin01`, `8x8`, 64
wpisy `Tile_List`, każdy `Tile_ID=94`, z czterema cyklicznymi orientacjami.
Następnie bez sprawdzenia semantyki Area wpisuje do `GIT` fixture'y na
`(12,10)` i `(16,10)`. `module.ifo` zawiera `Mod_Entry_Area` oraz `(5,5)`,
ale nie sprawdza, czy punkt startowy i dwa punkty GIT znajdują się na
dostępnej/przechodniej powierzchni wygenerowanych tile'ów.

Własny test readback potwierdzał jedynie, że liczba w `GIT.XPosition/YPosition`
jest równa uprzednio zaprogramowanej liczbie oraz że `Tile_List` ma 64 rekordy.
Nie testował native Toolset `Save`, dostępności (`Inaccessable Objects`),
spójności sąsiedztwa tile'ów, walkability ani prawidłowości punktu startowego.
To był błąd krytyczny implementacji i wiedzy proceduralnej o Aurorze: format
GFF i poprawne hashe nie są dowodem poprawnej Area.

Poprzednia korekta Area również miała tę samą lukę: dokument z 2026-07-18
zastąpił nieudany układ `tdc01` innym, syntetycznym układem po testach parsera,
ale bez pozytywnego zapisu/odczytu w Toolsecie. Bieżący kod generatora jest
niezatwierdzoną zmianą roboczą i nadal stosuje kolejną heurystykę (`tin01`,
powielony tile 94), a nie zwalidowany kontrakt native Area.

### Skutek dla dowodu i wymagane bramki

Wynik nie jest już wyłącznie `TScrollBox capture failed`. Poprawny status to:

| Punkt | Status | Powód |
| --- | --- | --- |
| Semantyka Area / fixture placement | `failed` | Toolset odrzucił zapis: 2 creatures in invalid locations |
| Prawidłowy start Area | `missing` | Pole IFO istnieje, lecz nie ma dowodu prawidłowego startu na dostępnej geometrii |
| Widoczność/render modelu M0 | `missing` | Nie wolno wnioskować o modelu z nieprawidłowej Area |
| NWN runtime AUR-S07 | `missing` | Nie uruchomiono prawidłowego modułu/Area ani capture runtime |

Centralny task standardu `019f6203-f618-7b82-8800-6c6a3e8c044f` wdrożył
poprawkę i zwalidował oba kontrakty M0. Dokładny profil v5 ma teraz jawny stan
`area_geometry_invalid`; centralny manager odrzuca go w preflight i na początku
`run-queue`, więc nie może dojść ani do capture, ani do runtime proof.

Nowa kanoniczna bramka geometrii wymaga: udanego native Toolset `Save` (`Area
was saved`), braku dialogów invalid-locations/`Inaccessable Objects`, startu
IFO `[5,5]` na dostępnej powierzchni oraz pozytywnego potwierdzenia powierzchni
dla obu fixture'ów. Test negatywny potwierdza, że samo `8x8`/64 `Tile_List`
z `Tile_ID=94` i GIT `[12,10]`/`[16,10]` jest tylko wejściem syntaktycznym i
zostaje odrzucone jako `area_geometry_invalid` bez takiego dowodu native.
Hash/preflight oraz syntaktyczny readback GFF nie mogą już oznaczać gotowości
do proofu. Nie wykonywano retry capture, zapisu Area ani sterowania PID `14628`.
