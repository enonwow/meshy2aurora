# Studio V1 - suplement implementacji frontendu

Data: 2026-07-14
Autor: Codex
Status: `AKTYWNY KONTRAKT IMPLEMENTACYJNY`
Zakres: `apps/studio-web`, piecioetapowy workflow Studio V1 i Debug Drawer

Workspace gate: implementacja, testy, buildy, mockupy robocze i pliki
tymczasowe powstaja wylacznie w `C:\Projects\meshy2aurora`.
`C:\Users\enonw\Documents\meshy2aurora` jest zakazane nawet jako staging lub
fallback. Niezgodny workspace oznacza HARD STOP bez zapisu.

## 1. Cel i pierwszenstwo

Ten suplement przeklada zaakceptowany kierunek wizualny Studio V1 na konkretna implementacje React/TypeScript. Jest podporzadkowany:

- [PROJECT_RULES.md](PROJECT_RULES.md),
- [architektura-web-wasm-codex.md](architektura-web-wasm-codex.md),
- [suplement-implementation-first-m7-s1-codex.md](suplement-implementation-first-m7-s1-codex.md),
- [studio-v1-debug-implementation-suplement-codex.md](studio-v1-debug-implementation-suplement-codex.md),
- [mockups/studio-v1-2026-07-14/README.md](mockups/studio-v1-2026-07-14/README.md).

Status obrazu w macierzy mockupow ma pierwszenstwo przed samym obrazem. Widokow oznaczonych `REWORK_REQUIRED` lub `OBSOLETE_DO_NOT_IMPLEMENT` nie wolno kopiowac 1:1.

Suplement nie zmienia formatu Aurora/NWN, nie otwiera S2 ani backlogu F1-F10 i nie pozwala modyfikowac konfiguracji Aurory. Dotyczy powloki i prezentacji realnych danych istniejacego pipeline'u.

## 2. Wynik produktu V1

Frontend ma prowadzic uzytkownika przez jeden stan sesji:

`Source -> Inspect -> Build -> Review Output -> Download`

Uzytkownik ma zawsze wiedziec:

- jakie lokalne pliki wybral,
- co zostalo sprawdzone,
- ktory etap pipeline jest wykonywany albo zakonczyl sie bledem,
- czy Converted Model pochodzi z binary readbacku,
- jaki wiersz zostal dodany do `appearance.2da`,
- co znajduje sie w HAK,
- jakie artefakty moze pobrac,
- jak wygenerowac raport sukcesu albo bledu.

Frontend nie ocenia jakosci artystycznej modelu i nie tworzy drugiego parsera, konwertera, writera, readbackera ani walidatora.

## 3. Stan zastany w repo

Stan z 2026-07-14:

| Obszar | Istniejaca baza | Wniosek implementacyjny |
|---|---|---|
| Shell | `src/App.tsx`, 185 linii | Zachowac przeplyw i rozbic monolityczny render na shell, reducer i ekrany krokow. |
| Sesja | `EMPTY`, `READY`, `WORKING`, `COMPLETE`, `ERROR` | Zastapic sam status jawna maszyna workflow i podstanami operacji. |
| Worker | `StudioWorkerClient` z request/response | Zachowac jeden Worker i revision guards; rozszerzyc protokol, nie budowac alternatywnego konwertera w UI. |
| Source | `INSPECT_SOURCE`, `SourceViewport` | Wykorzystac jako baze Inspect; potrzebna typowana projekcja statystyk i klipow. |
| Build | `BUILD_MODEL_PACKAGE` zwraca dopiero wynik finalny | Bez rozszerzenia protokolu Build jest indeterminate; UI nie moze wymyslac etapow ani procentow. |
| Readback | `AuroraReadbackViewport`, `projectCanonicalReadback` | Baza Converted Model istnieje; nalezy dodac przejscie do Binary Inspectora i brakujace dane animacji. |
| Wynik | `CanonicalResultSummary`, strict `projectCanonicalResult` | Zachowac strict projection i atomowosc; rozdzielic dane na ekrany Review. |
| Walidacja | `ValidationPanel`, readback diagnostics | Zachowac selection linkage; readiness musi pochodzic z prawdziwych invariantow. |
| Download | `ArtifactDownloads`, hash verification, Blob | Zachowac walidacje i pobieranie; dodac Result Package i jawny updated 2DA. |
| Viewport | wspolny `SceneViewport` i Three.js | Rozszerzyc o kontrolowany scene handle, animacje i overlays bez duplikowania rendererow. |
| M7 | `M7CorpusPanel` renderowany w glownym App | Przeniesc poza glowny workflow V1 do narzedzi developerskich; nie usuwac dzialajacej funkcji. |
| CSS | pojedynczy 33-liniowy `styles.css` | Zastapic tokenami i warstwami layout/komponenty; bez zmiany frameworka. |

### 3.1. Istniejace gwarancje, ktorych nie wolno zgubic

- lokalne pliki nie sa wysylane,
- Worker otrzymuje transferable `ArrayBuffer`,
- zmiana inputu uniewaznia wynik i stare odpowiedzi przez revision guards,
- projekcje JSON failuja na brakujacym lub blednym polu zamiast wstawiac fikcyjne dane,
- download weryfikuje provenance, nazwe, rozmiar i SHA-256,
- SourceViewport blokuje zewnetrzne URL-e zasobow GLB,
- Converted Model jest budowany z kanonicznego Rust binary-MDL readbacku,
- `OPEN_M6` pozostaje jawne.

## 4. Docelowa maszyna stanu

Nie dodajemy biblioteki state-management bez potrzeby. V1 moze uzyc `useReducer`, jawnych eventow i selektorow.

```ts
type WorkflowStep = "SOURCE" | "INSPECT" | "BUILD" | "REVIEW" | "DOWNLOAD";

type BuildState =
  | { kind: "IDLE" }
  | { kind: "RUNNING"; requestId: string; progress?: BuildProgressSnapshot }
  | { kind: "FAILED"; requestId: string; failure: BuildFailureSnapshot }
  | { kind: "SUCCEEDED"; requestId: string; result: StudioResultSnapshot };

type ReviewTab = "MODEL_DETAILS" | "APPEARANCE_2DA" | "PACKAGE_CONTENTS";
type ModelView = "SOURCE_MODEL" | "CONVERTED_MODEL";
type DebugTab = "DIAGNOSTICS" | "BINARY" | "PIPELINE_DATA" | "EXPORT";
```

Minimalny `StudioSessionState` przechowuje:

- aktualny krok i ostatni dostepny krok,
- wybrane pliki oraz ich revision,
- identyfikacje SHA-256 i stan parsowania,
- `SourceInspectionSnapshot`,
- `BuildState`,
- atomowy `StudioResultSnapshot`,
- aktywny Review tab oraz model view,
- zaznaczona encje, zasob i offset,
- stan Animation Player i Debug Overlays,
- stan Debug Drawera oraz aktywna zakladke,
- komunikaty UI niezbedne do file/Worker errors.

### 4.1. Dozwolone przejscia

| Zdarzenie | Warunek | Wynik |
|---|---|---|
| select/replace/remove file | zawsze przez gest uzytkownika | increment revision, wyczyszczenie inspection/build/result/download |
| continue to Inspect | oba pliki wybrane | `INSPECT`; uruchomienie/wykorzystanie source inspection |
| continue to Build | source i base 2DA sa poprawnie sparsowane | `BUILD/IDLE` |
| start Build | brak aktywnego requestu | `BUILD/RUNNING` |
| Build success | requestId i revision nadal aktualne | zapis atomowego result, checkmark Build, automatyczne `REVIEW` |
| Build failure | requestId i revision nadal aktualne | pozostanie w `BUILD/FAILED`, odblokowanie raportu |
| select Review tab | Build success | zmiana zakladki bez utraty result |
| continue to Download | wszystkie wymagane artefakty istnieja | `DOWNLOAD` |
| back | krok juz odwiedzony | powrot bez ponownego Build, dopoki input sie nie zmienil |
| start new conversion | jawny gest | pelne wyczyszczenie sesji i `SOURCE` |

Krokow zablokowanych nie mozna otworzyc przez sam click ani reczna zmiane URL. Stepper jest nawigacja po stanach sesji, nie ozdobnym paskiem.

## 5. Docelowe drzewo komponentow

```text
App
`- StudioSessionProvider / useStudioSession
   `- StudioShell
      |- StudioHeader
      |- WorkflowStepper
      |- InputsPanel
      |- StepWorkspace
      |  |- SourceStep
      |  |- InspectStep
      |  |- BuildStep
      |  `- ReviewDownloadWorkspace
      |     |- ModelViewportPanel
      |     |- ReviewTabs
      |     |  |- ModelDetailsTab
      |     |  |- AppearanceTwoDaTab
      |     |  `- PackageContentsTab
      |     `- DownloadArtifactsPanel
      |- ConversionReadinessPanel
      |- ValidationPanel
      |- ContextActionBar
      `- DebugDrawer
         |- DiagnosticsTab
         |- BinaryTab
         |- PipelineDataTab
         `- ExportTab
```

Wspolne komponenty:

- `SceneViewport`,
- `AnimationPlayer`,
- `DebugOverlaysMenu`,
- `ReferenceTraceBar`,
- `StatusBadge`,
- `FileIdentity`,
- `DiagnosticList`,
- `ArtifactTable`,
- `EmptyState`,
- `Disclosure`,
- `Tabs`.

Komponenty prezentacyjne nie wywoluja bezposrednio WASM. Efekty Worker, revision guards, projekcje JSON i tworzenie Blobow pozostaja w warstwie session/services.

## 6. Kontrakt poszczegolnych krokow

### 6.1. Source

Stany:

- empty: dwa wymagane drop zones, Continue disabled,
- partially selected: brakujacy input jawnie wskazany,
- ready: filename, size, skrocony SHA-256 i neutralny `Selected`; Continue enabled,
- file error: blad przypisany do konkretnej karty.

Reguly:

- V1 przyjmuje source GLB oraz bazowy `appearance.2da`; tekst `GLTF` nie wystepuje,
- `Selected` nie jest rownoznaczne z `Valid`,
- drag/drop i file input wywoluja ten sam handler,
- remove/replace uniewaznia wszystkie dalsze wyniki,
- brak uploadu jest widoczny, ale nie dominuje interfejsu.

### 6.2. Inspect

Inspect pokazuje tylko dane zrodlowe:

- Source Model viewport,
- statystyki meshes/vertices/triangles/materials/textures/bones/clips,
- source validation i prawdziwe warningi,
- Animation Player dla klipow z GLB,
- read-only Debug Overlays,
- jawny stan `No animation clips`, jezeli klipow nie ma.

`Continue to Build` jest dostepne tylko po poprawnej projekcji wymaganych danych. Inspect nie pokazuje Binary Readback ani artefaktow wynikowych.

### 6.3. Build

Build nie zawiera viewportu. Caly workspace zajmuja pipeline i stan operacji.

Docelowe etapy:

1. ingest source,
2. normalize canonical IR,
3. write binary MDL,
4. canonical binary readback,
5. update `appearance.2da`,
6. package HAK.

Stany:

- `IDLE`: podsumowanie wejsc i akcja Build,
- `RUNNING`: zakonczone/aktywne/oczekujace etapy oraz Cancel,
- `FAILED`: ten sam layout, czerwony etap, dalsze `Not Run`, diagnostyka i raport,
- success: brak osobnego pelnego ekranu; wynik jest zapisywany atomowo, Build dostaje checkmark i otwiera sie Review.

Do czasu wdrozenia prawdziwych eventow progress UI pokazuje `RUNNING - indeterminate`. Zabronione sa wymyslone procenty, czasy pozostale i nazwa aktualnego etapu wywnioskowana przez timer.

### 6.4. Review Output - Model Details

- viewport toggle: `Source Model | Converted Model`,
- Converted Model jest renderowany wylacznie z `BinaryMdlInspectionReport`,
- badge `Verified by binary readback - Inspect Binary` otwiera Debug Drawer/Binary,
- Source vs Converted pokazuje tylko metryki obecne w obu snapshotach,
- semantic equivalence pochodzi z raportu/invariantow, nie z porownania zaokraglonych liczb w UI,
- Animation Player dziala tylko dla rzeczywistych klipow udostepnionych przez kontrakt,
- Reference Trace Bar pokazuje GLB -> IR -> normalized -> MDL -> offset.

### 6.5. Review Output - appearance.2da

- tabela jest read-only,
- naglowki i komorki pochodza z typowanego diffu zwroconego przez Worker,
- default pokazuje `Relevant columns`, toggle wlacza `All columns`,
- Row i Label sa zamrozone, pozostale kolumny przewijane poziomo,
- widoczne sa base/output SHA-256, Added/Modified/Removed, collision i reference validation,
- dodany wiersz jest pokazany z kontekstem wiersza przed i po,
- wybor row/resref aktualizuje staly Reference Trace Bar,
- frontend nie implementuje drugiego parsera 2DA i nie dopisuje wiersza samodzielnie.

### 6.6. Review Output - Package Contents

- tabela jest projekcja rzeczywistego `packageManifest.resources`,
- mockupowe liczby i rodzaje zasobow nie sa fixture produkcyjnym,
- summary pokazuje nazwe, entry count, byte length, SHA-256, missing dependencies i duplicate resrefs,
- `Package Assembly` oraz `Runtime Proof` sa rozdzielone,
- wybor zasobu aktualizuje Reference Trace Bar,
- frontend nie otwiera ani nie reparsuje HAK w celu uzyskania danych, ktore powinien zwrocic kanoniczny manifest.

### 6.7. Download

- jedna glowna akcja `Download Result Package`,
- `Individual Artifacts (n)` otwiera kompaktowa liste faktycznie dostepnych plikow,
- kazdy download zachowuje obecna walidacje provenance, filename, byte length i SHA-256,
- `Open Export tools` otwiera Debug Drawer/Export,
- Result Package i Debug Report sa roznymi paczkami,
- `OPEN_M6` pozostaje widoczne i nie blokuje pobrania strukturalnie poprawnych artefaktow,
- `Start New Conversion` czysci sesje dopiero po jawnym potwierdzeniu, jezeli istnieje wynik niepobrany.

## 7. Conversion Readiness i Validation

Nie obliczamy wyniku 0-100.

```ts
type CheckStatus = "PASS" | "WARNING" | "FAIL" | "NOT_CHECKED" | "OPEN";
type ReadinessCategory =
  | "GEOMETRY"
  | "MATERIALS_TEXTURES"
  | "RIG"
  | "ANIMATIONS"
  | "BINARY_READBACK"
  | "PACKAGE_ASSEMBLY"
  | "RUNTIME_PROOF";
```

Reguly:

- kategoria agreguje prawdziwe invarianty z `DebugSnapshot`/validation report,
- `FAIL` > `WARNING` > `PASS`,
- brak wymaganych regul daje `NOT_CHECKED`, nigdy PASS,
- brak klipow nie daje `Animations 4/4`; pokazuje `No clips` albo status adekwatny do profilu,
- kazdy warning/fail ma wpis w Validation i action target,
- `Runtime Proof = OPEN_M6` jest osobnym stanem informacyjnym,
- frontend nie interpretuje estetyki, polycountu ani rozmiaru tekstury bez jawnej wersjonowanej reguly.

## 8. Viewport, animacje i overlays

### 8.1. Jeden renderer, rozne zrodla

`SceneViewport` pozostaje wspolna baza. Nalezy wydzielic kontrolowany scene adapter zamiast kopiowac setup Three.js dla source i readback.

Adapter udostepnia:

- fit/reset camera,
- selection linkage,
- screenshot,
- overlays,
- animation mixer/clip inventory,
- dispose wszystkich geometry/material/texture/skeleton/helper.

Zmiana model view musi anulowac poprzedni load i zwolnic GPU resources.

### 8.2. Animation Player

- stan playera nalezy do aktywnego viewportu,
- zmiana klipu zatrzymuje poprzednia akcje,
- loop, speed, seek i frame step nie zmieniaja danych modelu,
- ukryta karta lub background tab nie moze bez potrzeby konsumowac render loop,
- `prefers-reduced-motion` zatrzymuje autoplay,
- Source i Converted nie udaja zsynchronizowanych, dopoki kontrakt nie daje wspolnego clip mappingu.

### 8.3. Overlays

Grid, axes, wireframe, bounds, normals, skeleton, bone names, selected bone i skin weights sa niezaleznymi warstwami. Brak wymaganych danych blokuje toggle z wyjasnieniem; nie renderujemy przyblizonego szkieletu.

## 9. Debug Drawer

Jedyny uklad zakladek:

`Diagnostics | Binary | Pipeline Data | Export`

Drawer jest domyslnie zwiniety i nie zmienia glownej akcji workflow.

### 9.1. Diagnostics

- severity filters,
- code, stage, entity, expected, actual i evidence,
- `Show in viewport`, `Show offset` tylko gdy target istnieje,
- selection jest wspolne z viewportem, Binary i Reference Trace Bar.

### 9.2. Binary

- czyta bytes wybranego `MODEL` albo `HAK` artifact,
- 16 bajtow na wiersz,
- nie renderuje calego wielomegabajtowego artefaktu do DOM,
- uzywa okna wierszy wokol offsetu i jawnego jump-to-offset,
- interpretacje korzystaja z `DataView` z jawna endianness,
- field/section/offset pochodzi z writer layout/readback,
- zero writebacku, zero edycji i zero mutacji ArrayBuffer,
- Object URL powstaje tylko dla downloadu i jest zwalniany.

### 9.3. Pipeline Data

- Stage Ledger,
- canonical input/output hashes,
- Data Lineage,
- dependency graph,
- worker trace,
- oddzielenie canonical od runtime.

`Log` i `IR Graph` sa widokami danych wewnatrz Pipeline Data, nie osobnymi glownymi zakladkami.

### 9.4. Export

- preview zawartosci debug bundle,
- opcje artefaktow i standardowych screenshotow,
- privacy scrub status,
- source GLB nie jest dolaczany automatycznie,
- dziala po sukcesie i bledzie,
- nieobecne wyniki sa jawnie `missing/not produced`,
- generowanie ZIP odbywa sie w Workerze lub dedykowanej operacji, nie blokuje UI thread.

## 10. Brakujace kontrakty danych

Te luki sa jawne. Nie blokuja implementacji shell/layout, ale blokuja prawdziwe wypelnienie odpowiednich funkcji.

| ID | Brak | Potrzebny kontrakt | Zachowanie frontendu przed kontraktem |
|---|---|---|---|
| FE-D1 | etapy i progress Build | eventy Worker `MODEL_PACKAGE_BUILD_PROGRESS` ze stage/status, opcjonalnie completed/total | indeterminate Build bez procentow |
| FE-D2 | pelne source inspection | wersjonowany `SourceInspectionSnapshot` ze statystykami, diagnostics i clip inventory | pokazac tylko pola faktycznie obecne; brak = NOT_CHECKED |
| FE-D3 | animacja Converted Model | readback clip/controller mapping z duration i tracks | Animation Player disabled z wyjasnieniem |
| FE-D4 | diff `appearance.2da` | kolumny, wiersz before/added/after, base/output hashes, collision i refs | pokazac appended row index i `Details unavailable`, bez parsowania w UI |
| FE-D5 | updated 2DA artifact | osobny artifact kind/role i bytes | nie pokazywac indywidualnego downloadu 2DA |
| FE-D6 | Result Package ZIP | kanoniczny artifact z inventory i SHA-256 | indywidualne downloady pozostaja dostepne |
| FE-D7 | DebugSnapshot/report ZIP | wersjonowany snapshot, writer layout, lineage, invariants, stage ledger i bundle artifact | Debug Drawer pokazuje tylko dostepne readback diagnostics |
| FE-D8 | cancellation | jawne cancel/abort acknowledgement Workera | Cancel uniewaznia revision; UI opisuje, ze wynik spozniony zostanie odrzucony |

Frontend nigdy nie wypelnia luk wartosciami z mockupu.

## 11. Struktura plikow docelowych

Proponowany podzial, bez narzucania nowych zaleznosci:

```text
src/
|- app/
|  |- StudioShell.tsx
|  |- studioSession.ts
|  |- studioSelectors.ts
|  `- workflow.ts
|- features/
|  |- source/
|  |- inspect/
|  |- build/
|  |- review/
|  |  |- model-details/
|  |  |- appearance-2da/
|  |  `- package-contents/
|  |- downloads/
|  |- debug/
|  |  |- diagnostics/
|  |  |- binary/
|  |  |- pipeline-data/
|  |  `- export/
|  `- preview/
|- worker/
|- styles/
|  |- tokens.css
|  |- shell.css
|  |- components.css
|  `- utilities.css
|- App.tsx
`- main.tsx
```

Nie wykonujemy masowego przenoszenia przed pierwszym dzialajacym slicem. Pliki wydzielamy w momencie przejecia realnej odpowiedzialnosci z `App.tsx`.

## 12. CSS i responsywnosc

### 12.1. Tokeny

Kolory, spacing, radius, border, typography i statusy sa CSS custom properties. PASS/WARNING/FAIL/OPEN maja ikone i tekst; kolor nie jest jedynym nosnikiem informacji.

### 12.2. Layout

- `>= 1280px`: trzy kolumny Inputs / workspace / readiness,
- `960-1279px`: Inputs zwezone, readiness pod workspace albo w bocznym disclosure,
- `< 960px`: jedna kolumna, stepper przewijany, tabele przewijane poziomo, Debug Drawer na pelna szerokosc,
- viewport zachowuje minimalna uzyteczna wysokosc,
- Build bez viewportu wykorzystuje pelna szerokosc workspace,
- overlaye nie zaslaniaja modelu przy waskim widoku.

V1 jest desktop-first, ale nie moze tracic funkcji przy zoom 200% ani szerokosci tabletu.

## 13. Dostepnosc

- stepper jako `ol`, aktywny krok z `aria-current="step"`,
- zablokowane kroki nie sa focusable,
- drop zones maja prawdziwy input/label i obsluge klawiatury,
- tabs uzywaja `tablist/tab/tabpanel`,
- Debug Drawer jest disclosure z `aria-expanded`,
- canvas ma tekstowe podsumowanie i nie jest jedynym zrodlem wyniku,
- selection w viewportcie jest dostepne rowniez z tabel/list,
- statusy maja tekst i ikone,
- focus jest widoczny,
- animacja respektuje reduced motion,
- tabele zachowuja `th`, caption i powiazania naglowkow,
- bledy sa oglaszane przez odpowiednie live region, bez spamowania progress eventami.

## 14. Wydajnosc i lifecycle

- nie trzymac zduplikowanych kopii duzych ArrayBuffer bez potrzeby,
- nie serializowac bytes do base64,
- utrzymac jeden Worker i jawnie czyscic pending requests,
- revision/request ID chroni przed stale response,
- viewport i screenshoty zwalniaja GPU resources,
- Binary Inspector uzywa windowed rows,
- duze tabele 2DA/manifest maja stabilne keys i moga uzyc windowingu dopiero po pomiarze,
- Debug Drawer nie parsuje i nie renderuje ciezkich danych przed otwarciem zakladki,
- generowanie ZIP i screenshot packu nie moze blokowac interakcji.

## 15. Kolejnosc vertical slices frontendu

Ponizsze batche sa podzialem implementacyjnym wewnatrz S1, nie nowymi milestone'ami projektu.

| Slice | Zakres | Koniec slice |
|---|---|---|
| FE-V1 | design tokens, StudioShell, stepper, reducer, InputsPanel | Source empty/selected i bezpieczne invalidation dzialaja |
| FE-V2 | Inspect, source projection, Source Model, validation | realne source dane bez fikcyjnych metryk |
| FE-V3 | Animation Player i Debug Overlays dla source | read-only playback i overlays z poprawnym dispose |
| FE-V4 | Build workspace running/failed, progress boundary | brak viewportu i brak fake progress; error/report path dziala |
| FE-V5 | Review Model Details i Conversion Readiness | Converted Model pochodzi z readbacku, selection jest wspolne |
| FE-V6 | appearance.2da, Package Contents, Reference Trace Bar | realne dane diff/manifest; brak parsera w UI |
| FE-V7 | Download workspace i Result Package | hash-verified package oraz individual artifacts |
| FE-V8 | Debug Diagnostics i Binary | powiazanie diagnostic -> entity -> offset, windowed bytes |
| FE-V9 | Pipeline Data i Export | DebugSnapshot, privacy preview i raport success/failure |
| FE-V10 | responsive/a11y/polish oraz usuniecie M7 z glownego flow | cala seria ekranow spojna z kontraktem wizualnym |

Po kazdym zamknietym slice: review zmian, adekwatna minimalna kontrola, commit i push. Pelna fala testow i wizualnego proofu pozostaje po pierwszej implementacji wiekszosci slice'ow zgodnie z aktywnym suplementem implementation-first.

## 16. Ledger testow do pozniejszej fali

Zapisywac podczas implementacji, wykonac w fali integracyjnej:

- reducer transitions i invalidation po zmianie pliku,
- stale Worker response po replace/cancel,
- step gating i back navigation,
- source empty/partial/ready/error,
- Build indeterminate/progress/failure/success auto-review,
- brak fake PASS i brak fake progress,
- source/converted viewport oraz animation lifecycle,
- diagnostic -> viewport -> Binary offset,
- dynamiczna 2DA table i manifest,
- SHA mismatch i nieprawidlowa nazwa downloadu,
- Debug Report po sukcesie i bledzie,
- privacy scrub,
- keyboard/tabs/disclosure/live regions,
- 200% zoom i breakpoints,
- real browser Worker/WASM smoke,
- screenshot comparison dla finalnych poprawionych mockupow.

## 17. Definition of Done frontendu V1

Frontend V1 jest gotowy dopiero, gdy:

- realizuje piec krokow i jawnie blokuje niedostepne przejscia,
- nie renderuje M7 Corpus jako glownej funkcji workflow,
- wszystkie wartosci sa realne albo jawnie niedostepne,
- Build running/failed ma wspolny layout bez viewportu,
- sukces Build atomowo otwiera Review,
- Converted Model jest renderowany z kanonicznego readbacku,
- `Inspect Binary` otwiera zsynchronizowana macierz read-only,
- Conversion Readiness ma evidence i nie uzywa score 0-100,
- Inspect i Model Details maja prawdziwy Animation Player/overlays albo jawny brak danych,
- trzy zakladki Review maja staly Reference Trace Bar,
- dodany wpis 2DA i manifest HAK pochodza z Workera,
- Download oferuje Result Package i prawdziwe individual artifacts,
- Debug Drawer ma dokladnie cztery ustalone zakladki,
- raport dziala po sukcesie i bledzie oraz usuwa dane prywatne,
- zmiana inputu usuwa wszystkie zalezne wyniki i stale odpowiedzi ich nie odtwarzaja,
- UI thread nie implementuje logiki formatow i nie blokuje sie na ciezkich operacjach,
- poprawione mockupy Build/Binary/Export/Pipeline Data sa zapisane jako final visual evidence,
- finalny typecheck, testy, browser Worker/WASM oraz screenshot proof sa zielone.

## 18. Granice i zakazy

- brak zmian w Aurorze, NWN i ich konfiguracji,
- brak automatycznego czytania instalacji gry,
- brak uploadu i backendu w V1,
- brak alternatywnego JS convertera/parsera/writera,
- brak udawanych procentow, PASS, clipow, zasobow, wierszy 2DA i hashy,
- brak edycji modelu, animacji, rigu, wag, 2DA albo binarki,
- brak runtime proof screen w glownym V1,
- brak wdrazania F1-F10 pod pozorem dopracowania mockupu,
- brak kopiowania danych lub kodu z `aurora-web` do produktu.

## 19. Wniosek wykonawczy

Mockupy sa wystarczajace do rozpoczecia FE-V1--FE-V3 oraz statycznego ukladu dalszych ekranow. Pelne dane FE-V4--FE-V9 wymagaja jawnych rozszerzen Worker/WASM wymienionych w sekcji 10. Nie sa one powodem do zatrzymania powloki, ale sa twardym zakazem zastapienia brakow atrapami.

## 20. Ledger implementacji

### FE-V1 - 2026-07-14 - DONE_FIRST_PASS

- dodano `StudioShell`, semantyczny stepper pieciu krokow i blokowanie niedostepnych przejsc,
- dodano reducer sesji z revision-based invalidation wynikow po zmianie inputu,
- dodano `InputsPanel` oraz stany Source: empty, partial, ready i error,
- wybor GLB uruchamia realne `INSPECT_SOURCE` przez istniejacego Workera; UI nie tworzy fikcyjnych metryk,
- SHA-256 pliku `appearance.2da` jest liczony lokalnie, a nazwy obu wymaganych plikow sa walidowane,
- ekran Source zostal sprawdzony w przegladarce na `127.0.0.1:4173`; brak bledow i ostrzezen konsoli,
- targeted verification: 19/19 testow oraz typecheck przechodza,
- dotychczasowy ekran po przejsciu do Inspect jest tylko jawnym rusztowaniem `FE-V2 next`; nie jest zaliczony jako FE-V2,
- pelna aktualizacja dawnych testow `App` i cala fala integracyjna pozostaja odroczone zgodnie z trybem implementation-first.

Nastepny batch: FE-V2 - realny Inspect, source projection, Source Model i validation bez fikcyjnych danych.

### FE-V2 i FE-V3 - 2026-07-15 - DONE_FIRST_PASS

- `SOURCE_INSPECTED.ingestJson` ma scisla, typowana projekcje wariantow `READY` i `FAILED`,
- projekcja zachowuje realne identity, inventory, statistics, gates, diagnostics i clip inventory oraz odrzuca brakujace albo sprzeczne pola,
- wynik inspekcji jest zwiazany z revision sesji; wymiana inputu nie pozwala spoznionej odpowiedzi odtworzyc danych,
- `appearance.2da` przechodzi osobny preflight przez istniejacy `inspectTwoDaV2Json`; sama nazwa pliku i SHA nie odblokowuja Build,
- Inspect pokazuje realne meshes, vertices, triangles, materials, textures, unikalne bone node IDs i animation clips,
- Validation pokazuje `conversionEligible` oraz rzeczywiste gates; brak evidence ma jawny status `UNAVAILABLE`,
- Source viewport przekazuje wylacznie `gltf.animations` i obsluguje prawdziwy `THREE.AnimationMixer`, clip select, play/pause, loop i timeline,
- brak klipow ma jawny stan `No animation clips`; nie sa tworzone fikcyjne klipy,
- realne overlays V1 to grid, axes, skeleton, bounds i wireframe, z dostepnoscia wyprowadzona ze sceny i pelnym dispose,
- bone names, selected bone, skin weights i normals pozostaja niewdrozone zamiast byc symulowane,
- targeted verification: 36/36 testow, typecheck i pelny production build z Worker/WASM przechodza,
- Source zostal sprawdzony wizualnie w in-app browser; automatyczny lokalny file chooser nie jest wspierany przez ten browser, dlatego pelny browser Worker/WASM smoke Inspect pozostaje w odroczonej fali integracyjnej,
- ekran Build po `Continue to Build` jest jawnym rusztowaniem `FE-V4 next`; nie jest zaliczony jako FE-V4.

Nastepny batch: FE-V4 i FE-V5 - prawdziwy Build state oraz Review Model Details z kanonicznego readbacku.
