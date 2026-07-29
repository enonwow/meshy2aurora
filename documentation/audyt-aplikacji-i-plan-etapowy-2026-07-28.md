# Audyt aplikacji Meshy2Aurora i plan etapowy

Data: 2026-07-28  
Branch: `animation`  
Worktree: `C:\Projects\meshy2aurora\.worktrees\animation`  
Zakres: Studio Web, Worker, WASM, `m2a-core`, przepływy Creature, Placeable,
Tile, Animation Studio, persistence, build, review, download, CI i aktywna
dokumentacja.

Poza zakresem audytu: uruchamianie Aurora Toolset lub NWN. Końcowy proof
wizualny pozostaje własnością właściciela projektu.

## 1. Werdykt

Stan aplikacji:

`MOCNY CORE / DZIAŁAJĄCE STUDIO BETA / PRODUKT JESZCZE NIEGOTOWY DO WYDANIA`

Najważniejsze elementy pipeline już istnieją i są połączone:

- lokalny ingest GLB i `appearance.2da` / `placeables.2da`;
- Rust/WASM Worker z bezpiecznymi kontraktami;
- Creature, Placeable oraz ukryty flagą lane Tile;
- mapowanie 42 stanów Creature;
- tworzenie, edycja, zapis i ponowne otwieranie animacji Custom;
- kopiowanie animacji z innego GLB przy dokładnie zgodnym rigu;
- binary MDL, readback, TGA, 2DA, HAK i pakiet proof;
- Review oraz pobieranie artefaktów z kontrolą SHA-256;
- szeroki zestaw testów jednostkowych i integracyjnych.

Do wydania nie brakuje kolejnego wielkiego writera. Brakuje domknięcia produktu:
jednej prawdy w konfiguracji, spójnego UX całego workflow, pełnego cyklu życia
projektu local-first, kompletnego evidence w Review, zielonego realnego testu
Worker/WASM oraz owner proof na dokładnym kandydacie.

## 2. Jak używać dokumentu

- `[x]` — potwierdzone jako istniejące lub ukończone.
- `[ ]` — wymagane albo świadomie odłożone.
- Każdy etap można zamknąć dopiero po spełnieniu całego Definition of Done.
- Funkcje oznaczone `P0` blokują wydanie lub główny gate.
- Funkcje `P1` są wymagane dla spójnego MVP.
- Funkcje `P2` są ważne po MVP albo wymagają osobnej decyzji produktowej.

## 3. Stan potwierdzony podczas audytu

| Obszar | Stan | Wniosek |
|---|---|---|
| Rust Core | `PASS` | `fmt`, `clippy -D warnings` i wszystkie nieignorowane testy workspace przeszły. |
| TypeScript | `PASS` | `npm run typecheck` przeszedł. |
| Testy Studio | `PASS` | 60 plików testowych i 324 testy przeszły; jeden test środowiskowy jest pomijany w tym zestawie. |
| Browser persistence | `PASS` | Rzeczywisty IndexedDB round-trip przeszedł. |
| Production build | `PASS Z OSTRZEŻENIEM` | Build działa, ale główny JS ma 1,304 MB, a WASM 3,764 MB przed gzip; Vite zgłasza zbyt duży chunk. |
| Worker + web-WASM | `FAIL 1/16` | 15 testów przeszło, jeden test E2E nadal oczekuje starego `.inputs-panel`. |
| Aktualne UI | `DZIAŁA` | Source, Inspect, Animation Mapping i Animation Studio są renderowane i dostępne semantycznie. |
| Runtime Aurora/NWN | `OTWARTE` | `modelVisibility=not_tested`, `proofCompleteness=missing` dla niezamkniętego kandydata Animation Studio. |

Dokładny powtarzalny błąd integracyjny:

```text
tests/browser/worker-wasm.integration.ts:720
expected container.querySelector(".inputs-panel") to be truthy
```

Nowy ekran Source używa `SourceStep`, a nie starego panelu bocznego. Test nie
został przeprowadzony na nowy kontrakt UI i obecnie blokuje pełny gate CI.

## 4. Mapa funkcjonalna aplikacji

| Funkcja | Stan | Uwagi |
|---|---|---|
| Wybór lokalnego GLB | Gotowe | Plik pozostaje lokalny i ma SHA-256. |
| Wybór bazowego 2DA | Gotowe | Creature i Placeable mają odpowiedni kontrakt wejścia. |
| Inspect source | Gotowe częściowo | Pokazuje statystyki i gate'y generyczne; nie pokazuje całej gotowości profilu Creature przed buildem. |
| Source preview | Gotowe | Odtwarzanie klipów, seek, speed, keyframe stepping i debug overlays. |
| Base 42 mapping | Gotowe | Stabilne sloty i core validation. |
| Custom animation library | Gotowe | Draft/Valid/Invalid, stabilne ID, autosave i przypisanie do Base 42. |
| Create & edit | Gotowe jako Beta | Rotation, translation, keyframes, trim, retime, eventy, undo/redo. |
| Copy from another model | Gotowe dla exact rig | Niezgodny rig jest blokowany; automatyczny retarget nie istnieje. |
| V5 edited build | Gotowe offline | Materializacja, readback reconciliation i blokada downloadu przy mismatch. |
| Build UI | Częściowe | Brak prawdziwego postępu etapów; anulowanie restartuje cały Worker. |
| Review | Częściowe | Pokazuje mapping i authored evidence, ale nie pełną macierz behavior/deformation/event conformance. |
| Download | Gotowe technicznie | Hash jest sprawdzany, ale Creature nadal używa stałych nazw artefaktów. |
| Project recovery | Częściowe | Animation Studio używa IndexedDB, mapping localStorage; brak eksportu/importu całego projektu. |
| Placeable | Zaawansowane | Ma authoring, build, review i historyczne owner proof. Wymaga ujednolicenia shell/project UX. |
| Tile | Eksperymentalne | Istnieje za flagą `VITE_TILE_TARGET`; wymaga decyzji, czy wchodzi do MVP. |
| Meshy Lab | Eksperymentalne | Istnieje za flagą `VITE_MESHY_LAB` i wymaga Local Bridge. |
| Help / Theme / Settings | Brak funkcji | Kontrolki są widoczne, lecz zawsze disabled bez przekazanych callbacków. |

## 5. Najważniejsze ustalenia audytu

### P0.1. Pełny test Worker/web-WASM jest czerwony

Po zmianie globalnego layoutu scenariusz prawdziwego App + Worker + WASM nadal
szuka starej klasy `.inputs-panel`. Test należy przepisać na role i etykiety
nowego `SourceStep`, a następnie uruchomić cały `test:worker-integration`.

To jest regresja testu, nie potwierdzona regresja konwersji, ale pełny gate
wydania pozostaje czerwony.

### P0.2. Budżet trójkątów nadal łamie decyzję właściciela

Obowiązuje jedna wspólna granica:

- warning powyżej `10_000`;
- dokładnie `20_000` jest akceptowane;
- ponad `20_000` jest blokowane.

Kod nadal zawiera profile `5_000 / 10_000`, a Meshy Lab i Local Bridge używają
domyślnie `30_000` oraz opcjonalnie `60_000`. To jest aktywny rozjazd między
produktem, generatorem i właścicielskim kontraktem.

### P0.3. Creature nadal używa stałych nazw artefaktów

Worker emituje między innymi:

- `m2a_codex_aproof.hak`;
- `m2a_m6p01.mdl`;
- `m2a_codex_aproof.mod`.

Kolejne buildy mogą dostać suffix przeglądarki albo zostać pomylone z innym
lineage. Potrzebna jest caller-owned `ProjectIdentityV1`, która generuje i
waliduje wszystkie resrefy oraz nazwy plików jednego builda.

### P0.4. Aktywny stan dokumentacji nie odpowiada kodowi

`orchestrator-state.yaml` nadal opisuje `S2` jako `NOT_STARTED`, a
`documentation/README.md` mówi, że Studio powstanie po M6. Tymczasem Animation
Studio F1-F10, V5 i import zgodnego modelu są już zaimplementowane.

Starszy audyt Creature/Animation zawiera również luki, które zostały już
częściowo zamknięte: jawny mapping, wieloklipową bibliotekę Custom oraz exact-rig
import. Stan maszynowy i indeks dokumentacji muszą zostać zrekonsyliowane bez
przepisywania historycznych dowodów.

### P0.5. Wydanie pozostaje zablokowane przez realne wejście i owner proof

Offline readback nie jest proofem runtime. Dla aktualnego niezamkniętego
kandydata właściwy stan to:

- `modelVisibility=not_tested`;
- `proofCompleteness=missing`.

Agent może przygotować dokładny, immutable handoff. Toolset/NWN i końcowy
werdykt pozostają po stronie właściciela.

### P1.1. Nawigacja Creature używa błędnej etykiety

W `Inspect` przycisk nazywa się `Continue to Build`, ale dla Creature faktycznie
otwiera krok `Animation Mapping`. Opis ekranu także mówi o przejściu do builda.
Komponent powinien otrzymywać label i opis zależny od targetu.

### P1.2. Licznik `Custom 0` ma inną semantykę niż biblioteka Custom

Po zapisaniu prawidłowego klipu biblioteka pokazuje klip, ale katalog mapowania
nadal może wyświetlać `Custom 0`. Licznik oznacza wiersze Custom w katalogu
realizacji, nie liczbę zapisanych klipów. Użytkownik nie ma skąd znać tej
różnicy.

Należy rozdzielić etykiety:

- `Custom library: N`;
- `Base slots using Custom: N`.

### P1.3. Help, Theme i Settings są atrapami

Trzy nieaktywne przyciski zajmują stałe miejsce w globalnym nagłówku. Do MVP
należy albo zaimplementować minimum, albo ukryć je do czasu udostępnienia.
Widoczna nieaktywna kontrolka nie może udawać gotowej funkcji.

### P1.4. Brakuje pełnego cyklu życia projektu local-first

Istnieją trzy osobne mechanizmy:

- pliki źródłowe tylko w pamięci;
- mapping w `localStorage`;
- Animation Studio w IndexedDB.

Brakuje jednego projektu, który można:

- nazwać;
- wyeksportować jako manifest/JSON bez binarnych źródeł;
- ponownie zaimportować;
- powiązać z ponownie wybranym GLB po SHA-256;
- zduplikować;
- bezpiecznie usunąć;
- odzyskać po błędzie quota lub uszkodzeniu storage.

Diagnostyka persistence zaleca „export a project backup”, ale aplikacja nie ma
takiej akcji.

### P1.5. Anulowanie preview nie zatrzymuje pracy Workera

`StudioWorkerClient.cancel()` usuwa Promise z mapy i odrzuca go jako
`AbortError`, ale nie wysyła anulowania do Workera. Obliczenie WASM nadal może
zużywać czas i blokować kolejny request. Dla pełnego builda aplikacja anuluje
przez zakończenie całego Workera.

Potrzebny jest jawny kontrakt:

- najnowszy request wygrywa dla preview;
- ciężkie zadanie ma worker generation/token albo osobny Worker;
- UI nie deklaruje anulowania, dopóki stare obliczenie realnie nie przestało
  blokować kolejki.

### P1.6. Build ledger jest deklaratywny, nie rzeczywisty

Podczas builda UI ustawia:

```text
completedStages = []
Per-stage progress is unavailable
```

Mapa błędów rozpoznaje tylko sześć historycznych etapów i nie ma pełnych
odpowiedników między innymi dla Animation, Texture, Package i Proof Module.
Należy zwracać zdarzenia etapów z Workera albo pokazywać uczciwy pojedynczy stan
bez pozorowanego ledgera.

### P1.7. Review nie pokazuje pełnego evidence, które posiada Core

Typed projection pokazuje authored mapping i event conformance, ale nadal nie
prezentuje kompletnej macierzy:

- animation completeness;
- behavior conformance;
- loop/terminal policy;
- per-clip active joints;
- root motion;
- non-rigid SkinMesh deformation;
- pochodzenie `SOURCE / IMPORTED / AUTHORED / GENERATED`;
- pierwsza różnica source vs binary readback.

### P1.8. Kontrakt Base 42 jest powielony

Nazwy stanów są utrzymywane w TypeScript oraz w kilku modułach Rust. Aktualne
testy zmniejszają ryzyko, ale nie zapewniają jednej prawdy. Katalog powinien
pochodzić z jednego wersjonowanego kontraktu Core/WASM albo z generowanego
artefaktu.

### P1.9. Rozmiar aplikacji wymaga podziału

Aktualny build:

- JS: około `1.304 MB`, gzip `346 KB`;
- WASM: około `3.764 MB`, gzip `1.339 MB`;
- CSS: około `133 KB`, gzip `22 KB`.

`App.tsx` ma około 2,373 linii, `m2a-wasm/src/lib.rs` około 5,892 linii i
97 eksportów `wasm_bindgen`, a CSS Animation Studio około 1,880 linii.

Nie jest to blocker poprawności, ale utrudnia kolejne funkcje, testy i lazy
loading. Meshy Lab, Tile, Placeable authoring i Animation Studio powinny być
ładowane według aktywnego targetu/funkcji.

## 6. Plan etapowy

### E0. Przywrócić jedną prawdę i zielony gate

Priorytet: `P0`  
Zależności: brak

- [x] Przepisać czerwony test Worker/web-WASM na role/label nowego Source UI.
- [x] Usunąć pozostałe testowe zależności od `.inputs-panel`.
- [x] Uruchomić cały `npm run test:worker-integration`.
- [x] Wprowadzić jedną stałą `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 20_000`.
- [x] Ustawić warning na `10_000` i blokadę powyżej `20_000`.
- [x] Zsynchronizować Core, Studio, Meshy Lab i Local Bridge.
- [x] Dodać testy granic `10_000`, `10_001`, `20_000`, `20_001`, `21_845`,
  `21_846`.
- [x] Zrekonsyliować `orchestrator-state.yaml` z faktycznym stanem Animation
  Studio, bez zmiany historycznych proof packets.
- [x] Poprawić aktywne linki/statusy w indeksie dokumentacji.

Definition of Done:

- [x] CI przechodzi w całości.
- [x] Nie istnieje drugi budżet produktu.
- [x] Aktywny stan maszynowy i aktywne dokumenty nie przeczą kodowi.

Evidence 2026-07-28:

- `cargo fmt --all --check` — PASS;
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS;
- `cargo test --workspace --quiet` — PASS dla wszystkich nieignorowanych testów;
- `npm run typecheck` — PASS;
- `npm test` — PASS: 60 plików, 325 testów; 1 plik i 1 test świadomie
  pominięte;
- `npm run build` — PASS;
- `npm run test:worker-integration` — PASS: 6 plików, 16 testów oraz 1/1
  browser persistence;
- `node --test bridge.test.mjs` — PASS: 16/16.

### E1. Domknąć globalny shell i nawigację

Priorytet: `P1`  
Zależności: E0

- [x] Dodać target-aware tekst w Inspect:
  `Continue to Animation Mapping` dla Creature i `Continue to Build` dla
  Placeable/Tile.
- [x] Ujednolicić nazwy sześciu kroków we wszystkich aktywnych ekranach i
  dokumentach; historyczne mockupy V1 zachowują własne, jawnie wersjonowane
  nazwy.
- [x] Rozdzielić `Custom library: N` od `Base slots using Custom: N`.
- [x] Ustalić los Help/Theme/Settings: minimum funkcji albo ukrycie.
- [x] Dodać globalny status projektu: nazwa, target, source SHA i dirty/saved.
- [x] Sprawdzić fokus, scroll i modal layering w 1600×1000, 1366×768,
  1024×768 oraz przy powiększeniu 200%.
- [x] Dodać browser test przejścia Source → Inspect → Mapping → Build dla
  Creature oraz Source → Inspect → Build dla Placeable.

Definition of Done:

- [x] Każdy przycisk zapowiada dokładnie ekran, który otwiera.
- [x] Żadna widoczna kontrolka nagłówka nie jest atrapą.
- [x] Globalny shell zachowuje ten sam język wizualny dla wszystkich targetów.

Evidence 2026-07-28:

- wspólna prawda nazw kroków:
  `Source`, `Inspect`, `Animation Mapping`, `Build`, `Review`, `Download`;
- target-aware CTA sprawdzone dla Creature i Placeable, w tym poprawne
  `Back to Animation Mapping` z Creature Build;
- widoczny nagłówek zawiera nazwę projektu, target, source SHA oraz stan
  `Not saved`/`Dirty`/`Saved`; Help/Theme/Settings nie są renderowane bez
  prawdziwych callbacków;
- Mapping rozdziela liczbę pozycji biblioteki Custom od liczby bazowych slotów
  używających Custom;
- `npm run typecheck` — PASS;
- `npm test` — PASS: 60 plików, 327 testów; 1 plik i 1 test świadomie
  pominięte;
- `npm run build` — PASS;
- `npm run test:worker-integration` — PASS: 7 plików i 19 testów browser
  Worker/WASM oraz 1/1 browser persistence;
- browser responsive contract — PASS dla 1600×1000, 1366×768, 1024×768 i
  efektywnego viewportu 512×384 odpowiadającego 200% zoom; modal pozostaje
  w viewport, ponad shellem i utrzymuje focus trap.

### E2. Wprowadzić prawdziwy projekt local-first

Priorytet: `P1`  
Zależności: E1

- [x] Zdefiniować `Meshy2AuroraProjectV1`.
- [x] Włączyć target, source SHA, base 2DA SHA, mapping V2, Animation Studio,
  placeable/tile options i project identity.
- [x] Dodać `New project`, `Export project`, `Import project`,
  `Duplicate project`, `Delete project`.
- [x] Projekt nie zapisuje absolutnych ścieżek ani binarnych payloadów GLB.
- [x] Po imporcie aplikacja prosi o ponowne wskazanie plików i wiąże je tylko
  po exact SHA-256.
- [x] Dodać ekran recovery rekordów IndexedDB.
- [x] Dodać obsługę quota, uszkodzonego rekordu i niezgodnej wersji schematu.
- [x] Dodać ostrzeżenie przed opuszczeniem strony przy niezapisanej rewizji.
- [x] Zdefiniować migrację V1 → przyszłe wersje bez cichego przepisywania
  nieznanych danych.

Definition of Done:

- [x] Użytkownik może zamknąć kartę, wrócić i odzyskać projekt po ponownym
  wskazaniu exact plików.
- [x] Backup można przenieść między przeglądarkami bez prywatnych ścieżek.
- [x] Build zawsze wskazuje jedno project ID i jedną rewizję.

Evidence 2026-07-28:

- dodano ścisły `Meshy2AuroraProjectV1` z identity, rewizją, targetem,
  referencjami plików po SHA-256, mapping V2, Animation Studio, Placeable
  authoring i Tile options;
- serializacja odrzuca ścieżki, payloady binarne, pola nieznane, nowszy schemat
  i niespójne source-bound authoring;
- Project Manager zapewnia New/Export/Import/Duplicate/Delete, nazwę projektu,
  recovery rekordów oraz wymagany exact-SHA rebinding;
- IndexedDB weryfikuje fingerprint i metadane indeksu; quota, rekord uszkodzony
  oraz wersja niezgodna mają odrębne diagnostyki; audit migracyjny V1 nie
  wykonuje zapisów;
- `beforeunload` chroni każdą rewizję bez stanu `SAVED`;
- Build i Review pokazują caller-owned project ID oraz project revision;
- `npm run typecheck` — PASS;
- `npm test` — PASS: 63 pliki, 338 testów; 2 browser-only pliki i 2 testy
  świadomie pominięte w środowisku unit;
- `npm run build` — PASS;
- `npm run test:worker-integration` — PASS: 8 plików i 21 testów browser
  Worker/WASM; 2/2 natywne browser IndexedDB;
- browser lifecycle — PASS: import backupu, odrzucenie niezgodnego GLB,
  exact-SHA rebind GLB/2DA, autosave, ponowne uruchomienie UI i recovery;
- browser responsive contract — PASS także dla Project Managera przy
  efektywnym viewport 200%, z focus trap i poprawnym layeringiem.

### E3. Domknąć Creature i Animation Studio jako jeden produkt

Priorytet: `P1`  
Zależności: E2

- [x] Base 42 jest stałym katalogiem slotów Aurora.
- [x] Valid Custom można przypisać do Base 42.
- [x] Draft/Invalid nie można przypisać.
- [x] Source GLB pozostaje niezmieniony.
- [x] Można utworzyć i edytować keyframes oraz eventy.
- [x] Można skopiować animację ze zgodnego GLB.
- [x] Niezgodny rig jest blokowany przed zapisem.
- [x] Wyświetlać pełne provenance klipu bezpośrednio w pickerze.
- [x] Udostępnić preview wybranego Custom jeszcze przed przypisaniem.
- [x] Dodać szybki filtr braków `gameplay floor 7` oraz `full 42`.
- [x] Pokazać wymagania profilu H1 już w Inspect, nie dopiero po buildzie.
- [x] Generować katalog Base 42 z jednej prawdy Core/WASM.
- [x] Przeprowadzić caller-owned `ProjectIdentityV1` przez Studio, Worker,
  WASM, Core, manifest i nazwy pobrań.
- [x] Zdefiniować uczciwe anulowanie/debounce preview.
- [x] Dodać test wielu kolejno importowanych donorów z tym samym rigiem.
- [x] Dodać test odrzucenia klipu z tym samym bone count, ale inną hierarchią.
- [x] Zdecydować, czy `Beta` pozostaje po zamknięciu owner proof.

Decyzja `Beta`:

- podczas oczekiwania na human-owned owner proof oznaczenie `Beta` pozostaje;
- po pozytywnym zamknięciu exact owner proof oznaczenie zostanie usunięte;
- wynik negatywny albo niekompletny pozostawia `Beta` do czasu kolejnego
  dopuszczonego i pozytywnie zamkniętego proofu.

Poza MVP tego etapu:

- [ ] Automatyczny retarget między różnymi rigami (`P2`).
- [ ] Import animacji z FBX (`P2`).
- [ ] Pełny edytor wag/skinningu (`P2`).
- [ ] Quadruped/N1 authoring (`P2`, osobny profil).

Definition of Done:

- [x] Jeden projekt może zebrać wiele klipów z wielu zgodnych GLB bez utraty
  provenance.
- [x] Wszystkie przypisania są stabilne po rename i recovery.
- [x] Każdy Creature build V4/V5 ma własne, deterministyczne i kolizyjnie
  bezpieczne nazwy artefaktów pochodzące z `ProjectIdentityV1`.

Evidence 2026-07-28:

- picker Custom pokazuje provider, asset, ownership i lineage, a preview przed
  przypisaniem nie mutuje mappingu; przypisanie wymaga osobnej akcji;
- Core definiuje pełne Base 42 i dokładny gameplay floor 7, a śledzony kontrakt
  JSON jest generowany z Core oraz sprawdzany przez `catalog:check`;
- Inspect pokazuje wymagania profilu H1 przed buildem;
- `ProjectIdentityV1` przechodzi przez Studio, Worker, WASM i Core do manifestu;
  ten sam projekt i rewizja dają identyczne nazwy i SHA, a zmiana rewizji daje
  inny 16-znakowy namespace;
- osobny Worker preview zapewnia rzeczywiste przerwanie obliczenia, last-request
  wins i nie przerywa głównego Workera buildów;
- regresje donorów potwierdzają wiele zgodnych źródeł w jednym projekcie oraz
  blokadę tej samej liczby kości z inną hierarchią;
- `npm run typecheck` — PASS;
- `npm test` — PASS: 64 pliki, 343 testy; 2 browser-only pliki i 2 testy
  świadomie pominięte w środowisku unit;
- `npm run build` — PASS;
- browser Worker/WASM — PASS: 8 plików i 21 testów;
- browser IndexedDB — PASS: 2 pliki i 2 testy;
- `cargo test -p m2a-wasm --lib` — PASS: 32 testy;
- celowo nie wykonano ani nie zadeklarowano proofu Aurora Toolset/NWN; granicą
  pozostaje późniejszy handoff do proofu wykonywanego przez właściciela.

### E4. Domknąć Build, Review i Download

Priorytet: `P1`  
Zależności: E3

- [x] Rozszerzyć kontrakt Workera o rzeczywiste etapy albo uprościć UI do
  uczciwego pojedynczego progress state.
- [x] Pokryć wszystkie etapy błędów Core w typed Build UI.
- [x] Dodać prawdziwą semantykę cancel/retry bez pomieszania odpowiedzi
  poprzedniej rewizji.
- [x] Pokazać pełne animation completeness/behavior/event/deformation evidence.
- [x] Dodać tabelę wszystkich Base 42 i Custom z pochodzeniem oraz readbackiem.
- [x] Dodać zsynchronizowane porównanie source/edited/binary readback.
- [x] Pokazać root motion i markery eventów.
- [x] Dodać `First mismatch` z linkiem do klipu, tracka albo artefaktu.
- [x] Utworzyć jeden manifest pobrania z project ID, rewizją, SHA wszystkich
  wejść/wyjść i statusem owner proof.
- [x] Rozważyć pojedynczy ZIP tylko po zachowaniu indywidualnych SHA i typów
  artefaktów.

Definition of Done:

- [x] Użytkownik przed pobraniem rozumie, co zostało wygenerowane, skąd
  pochodzi każda animacja i które dowody są tylko offline.
- [x] Download jest możliwy wyłącznie dla exact current revision z pełnym
  readback reconciliation.

Evidence 2026-07-29:

- Build pokazuje jeden uczciwy, nieprocentowy stan
  `Canonical local package build`; błędy wszystkich 14 aktualnych etapów Core
  mają ścisłą projekcję typed UI;
- cancel kończy fizycznie starego Workera, retry tworzy nowy request i nową
  generację Workera, a odpowiedź poprzedniej rewizji jest ignorowana;
- Review pokazuje pełne Base 42 i Custom wraz z providerem, assetem,
  ownership, fallbackiem, pochodzeniem oraz stanem binary readback;
- Review pokazuje completeness, behavior, events, deformation, root motion
  oraz porównanie Source → Edited → Binary readback; `First mismatch` prowadzi
  do dokładnego klipu/tracka albo artefaktu;
- manifest pobrania wiąże project ID i rewizję, SHA wejść, fingerprinty
  mappingu/Animation Studio, wszystkie artefakty Workera oraz jawny stan
  `PENDING_OWNER`, `modelVisibility=not_tested`,
  `proofCompleteness=missing`;
- download jest blokowany dla starej rewizji, niepełnego readbacku albo
  niezgodnego inventory; pojedynczy ZIP świadomie nie wchodzi do E4, aby nie
  tworzyć drugiej warstwy pakowania bez nowej wartości dowodowej;
- `npm run typecheck` — PASS;
- `npm test` — PASS: 66 plików, 353 testy; 2 browser-only pliki i 2 testy
  świadomie pominięte w środowisku unit;
- `npm run build` — PASS;
- browser Worker/WASM — PASS: pełny gate 8 plików i 21 testów, a ponowny
  celowany test dokładnej granicy pakietu 9/9;
- browser IndexedDB — PASS: 2 pliki i 2 testy;
- celowo nie wykonano ani nie zadeklarowano proofu Aurora Toolset/NWN.

### E5. Ujednolicić pozostałe targety i funkcje opcjonalne

Priorytet: `P1/P2`  
Zależności: E2, może być realizowane niezależnie od E3-E4

- [x] Przenieść Placeable na wspólny project lifecycle, identity i build
  evidence.
- [x] Zachować potwierdzony wspólny budżet `20_000` dla Placeable.
- [x] Dodać browser E2E Placeable z realnym, właścicielskim sample.
- [x] Podjąć decyzję: Tile w MVP czy nadal eksperymentalny feature flag.
- [x] Jeśli Tile wchodzi do MVP, domknąć WOK/SET/MDL Review i owner proof.
- [x] Podjąć decyzję: Meshy Lab jest częścią dystrybucji czy osobnym trybem
  developerskim.
- [x] Jeśli Meshy Lab wchodzi do produktu, zsynchronizować budżety, provenance,
  recovery i komunikację błędów Bridge.
- [x] Nie wysyłać sekretów Meshy do aplikacji webowej; Bridge pozostaje lokalną
  granicą zaufania.

Definition of Done:

- [x] Każdy widoczny target ma kompletny Source → Inspect → Build → Review →
  Download.
- [x] Funkcje eksperymentalne są ukryte lub jawnie oznaczone i nie udają MVP.

Decyzje produktowe:

- Creature i Placeable tworzą pierwsze MVP;
- Tile pozostaje poza MVP, jest ukryty bez `VITE_TILE_TARGET=1`, a po włączeniu
  ma widoczne oznaczenie `Experimental`; warunek owner proof Tile nie ma
  zastosowania do tego MVP;
- Meshy Lab pozostaje ukrytym domyślnie `Developer tool` wymagającym lokalnego
  Bridge; nie jest podstawową ścieżką dystrybucji;
- pełna decyzja i granica zaufania:
  `documentation/zakres-produktu-mvp-2026-07-29.md`.

Evidence 2026-07-29:

- Placeable V3 przyjmuje dokładny `ProjectBuildIdentityV1` w
  App → Worker → WASM → Core, wyprowadza kolizyjnie bezpieczny 16-znakowy
  namespace i zapisuje identity w raporcie;
- ta sama rewizja daje identyczne nazwy i SHA, a kolejna rewizja nowe
  `.mod`, `.hak`, resrefy oraz hashe;
- Review pokazuje project ID/revision, komplet statusów MDL/PWK/2DA/UTP,
  zasoby, readback i jawny stan owner proof; manifest pobrania zawiera również
  SHA dokumentu Placeable authoring;
- realny browser E2E przeszedł pełne
  Source → Inspect → Build → Review → Download dla
  `sample-3d/s1-placeable-ritual-pedestal-1500/source.glb`, SHA-256
  `dad22a5c3490242458cb7a81e50c53e265abf75886f57f6bd8a770938c2f7372`;
- wspólny budżet 20 000 dla Placeable pozostaje sprawdzany przez Core/Studio;
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
  warnings` oraz wszystkie nieignorowane `cargo test --workspace` — PASS;
- `npm run typecheck` — PASS;
- `npm test` — PASS: 66 plików i 354 testy; 2 browser-only pliki i 2 testy
  świadomie pominięte w środowisku unit;
- `npm run build` — PASS;
- browser Worker/WASM — PASS: 8 plików i 22 testy;
- browser IndexedDB — PASS: 2 pliki i 2 testy;
- Toolset/NWN nie został uruchomiony; historyczny owner proof Placeable nie
  został przepisany na nową identity.

### E6. Wydajność, architektura i jakość utrzymania

Priorytet: `P1` przed większymi nowymi funkcjami  
Zależności: E0-E4

- [x] Podzielić `App.tsx` na kontrolery targetów i hooki sesji/builda.
- [x] Podzielić publiczną granicę `m2a-wasm` na moduły domenowe bez zmiany ABI.
- [x] Wygenerować lub współdzielić typed schematy Worker/WASM.
- [x] Podzielić CSS Animation Studio na shell, library, viewport, inspector,
  timeline i dialogs.
- [x] Wprowadzić lazy loading Animation Studio, Placeable, Tile i Meshy Lab.
- [x] Zmierzyć cold start, parse, preview i build na słabszym laptopie.
- [x] Ustalić budżet JS/WASM oraz egzekwować go w CI.
- [x] Dodać testy race: replace source, cancel build, rapid preview i stale
  response.
- [x] Dodać pełny keyboard/a11y smoke dla wszystkich kroków, nie tylko
  Animation Studio.
- [x] Przypiąć świadomie wersje zależności zamiast utrzymywać wszystkie główne
  paczki jako `latest`.

Definition of Done:

- [x] Brak ostrzeżenia o niekontrolowanym głównym chunku albo istnieje
  zaakceptowany, zmierzony budżet.
- [x] Zmiana jednego targetu nie wymaga edycji monolitycznego `App.tsx`.
- [x] Każdy request ma test własności rewizji i stale-response rejection.

Evidence 2026-07-29:

- targetowe requesty i transferable buffers są w
  `features/build/targetBuildRequest.ts`, a cykl życia sesji rewizyjnej i
  Workera w `app/useStudioControllers.ts`; zmiana targetowego buildu nie
  wymaga dopisywania kolejnej gałęzi requestu w `App.tsx`;
- publiczne ABI WASM zachowało istniejące `js_name`, a Placeable i Tile mają
  osobne moduły domenowe `placeable_boundary.rs` i `tile_boundary.rs`;
- klient i Worker współdzielą `worker/types.ts`; wykonywalny
  `contract:check` potwierdza 14/14 request discriminants, 14/14
  wyczerpujących handlerów i 23 typed importy z generowanego
  `m2a_wasm.d.ts`;
- 1 895 linii CSS Animation Studio rozdzielono na 15 plików domenowych przy
  zachowaniu kolejności reguł; modal importu ładuje własny arkusz również
  poza pełnym workspace;
- produkcyjny build tworzy osobne lazy chunks dla Animation Studio, Placeable
  authoring, Tile Review i Meshy Lab; główny JS ma 439 222 B
  (gzip 122 088 B);
- gate `budget:check` przechodzi dla main JS 439 222/460 000 B, największego
  chunku 601 685/620 000 B, całego JS 1 393 517/1 450 000 B, CSS
  147 038/150 000 B i WASM 3 798 736/3 820 000 B
  (gzip 1 340 912/1 380 000 B);
- pomiar realnego H1 w headless Chrome z CPU throttling 4x:
  cold Worker/WASM 41,7 ms, parse 318,4 ms, preview 24,1 ms i build
  696,7 ms; są to jawne sufity regresji profilu referencyjnego, nie deklaracja
  konkretnego fizycznego laptopa;
- race matrix obejmuje replace source, cancel/retry build, rapid preview
  last-request-wins oraz odrzucenie starego request ID i rewizji;
- realny keyboard smoke przechodzi przez Source → Inspect → Animation Mapping
  → Build → Review → Download; ujawniony w audycie niedostępny krok Download
  został domknięty osobnym ekranem i rewizyjnym
  `CONTINUE_TO_DOWNLOAD`;
- wszystkie wersje React, Three, TypeScript, Vite i Vitest są przypięte do
  dokładnych wersji w `package.json`;
- `cargo fmt --all --check` i
  `cargo clippy --workspace --all-targets -- -D warnings` — PASS;
- wszystkie nieignorowane testy Rust workspace uruchomione w rozliczalnych
  grupach — PASS, 0 failures; ignored testy wymagają jawnych zewnętrznych
  witnessów i nie były wymuszane;
- `npm run typecheck` — PASS;
- `npm test` — PASS: 67 plików, 357 testów; 2 browser-only pliki i 2 testy
  pominięte w środowisku unit;
- `npm run build` wraz z `contract:check` i `budget:check` — PASS;
- browser Worker/WASM — PASS: 8 plików i 22 testy;
- browser IndexedDB — PASS: 2 pliki i 2 testy;
- Toolset/NWN nie został uruchomiony; human-owned owner proof pozostaje
  granicą E7.

### E7. Realne E2E, handoff i wydanie

Priorytet: `P0 release gate`  
Zależności: E0-E6

- [x] Uruchomić realne browser E2E na właścicielskich modelach z
  `sample-3d/<asset-id>/manifest.yaml`.
- [ ] Minimum: humanoid Creature, Creature z klipami donorów oraz Placeable.
- [ ] Dla każdego wyniku zapisać source SHA, project revision, mapping
  fingerprint i SHA wszystkich artefaktów.
- [ ] Zamrozić jeden exact candidate Animation Studio bez tworzenia nowej
  iteracji poza model iteration gate.
- [x] Przygotować handoff zaczynający się od dokładnego pliku `.mod`, następnie
  nazwy modułu w Toolset i dokładnej Area.
- [x] Dołączyć exact object, placement, HAK, Appearance row, resrefy i
  oczekiwany sposób wywołania animacji.
- [x] Zapisać `modelVisibility=not_tested` i
  `proofCompleteness=missing` przed wynikiem właściciela.
- [x] Po wyniku właściciela zapisać osobno Toolset i NWN.
- [x] Przygotować krótką instrukcję użytkownika dla Creature, Animation Studio
  i Placeable.
- [ ] Nadać wersję produktu dopiero po zielonym CI, realnym E2E i wymaganym
  owner proof.

Definition of Done:

- [x] Exact artefakty są odtwarzalne z manifestu.
- [x] Właściciel otrzymuje kompletny handoff bez domysłów.
- [x] Status runtime pochodzi wyłącznie z owner proof.

Evidence 2026-07-29:

- realny browser Worker/WASM potwierdził źródło H2
  `h2-clockwork-sentinel-1500`, SHA-256
  `f8cf0af21c8143a62b64c490a81dd2855ad3c3f9865922e3854f84b714dec3a3`,
  materializację profilu 42 stanów i SHA sześciu artefaktów;
- realny H1, SHA-256
  `3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f`,
  przeszedł exact-rig import odrębnego klipu dawcy, walidację, preview i
  materializację 48 tracków w projekcie rewizji `3`; fingerprint Animation
  Studio:
  `1750fdf134dd472916d02ffd004a0a502ce8c605a0ca5ac542bac05b0382ab0a`;
- ten sam projekt ma pełny dokument mappingu V2 z `cpause1` wskazującym
  stabilne ID donor Custom; deterministyczny SHA canonical serialized mapping:
  `b7a426f3b7c7dd5a3e2a12dce68b660a8b99d2cb9d9f6645d3fecedbb7074c8e`;
- materializacja donor JSON ma 173 500 B i SHA-256
  `0abfd4ce8e41cf2756e1f6c793aac05f780bb74cd932bc22d44875bf0076488e`;
  drugi przebieg zwrócił identyczne bajty;
- donor payload powstał wyłącznie w pamięci przez deterministyczną zmianę nazwy
  klipu w canonical H1, nie został zapisany jako drugi asset root, a jego SHA
  wynosi
  `298d3418c460aada80e3868f95ed99e8a768e74579c94f4034731e7e7f16f3e5`;
- realny Placeable
  `s1-placeable-ritual-pedestal-1500`, SHA-256
  `dad22a5c3490242458cb7a81e50c53e265abf75886f57f6bd8a770938c2f7372`,
  z project revision `1` i authoring fingerprint
  `6acb06c903391a77ea4c2e04662b928e1d1a393f7ec5b0c8c93e5030b6c93db8`
  dał dwukrotnie identyczne nazwy, długości i SHA czterech artefaktów;
- realny H2 również został zbudowany dwukrotnie z identycznymi nazwami,
  długościami i SHA sześciu artefaktów;
- wszystkie istniejące exact artefakty z manifestu mają repeat evidence:
  H2 i Placeable przez dwa buildy, donor przez dwie materializacje, a r46
  przez ignored compatibility audit; brakujący V5 MOD/HAK nie został
  zastąpiony fikcyjnym wpisem;
- pełny zapis wejść, rewizji, fingerprintów, artefaktów i statusów:
  `documentation/evidence/e7-release-evidence-manifest-2026-07-29.json`;
- kompletny exact handoff r46 zaczyna się od `m2a_h2r46.mod`, nazwy modułu
  i Area oraz zawiera object, placement, HAK, Appearance row, resrefy i
  oczekiwane wywołanie `cpause1`:
  `documentation/evidence/e7-owner-proof-handoff-2026-07-29.md`;
- ponowny odczyt z 2026-07-29 potwierdził byte identity canonical i już
  zainstalowanych MOD/HAK r46; niczego nie kopiowano i Toolset/NWN nie został
  uruchomiony;
- instrukcja użytkownika:
  `documentation/instrukcja-uzytkownika-creature-animation-placeable-2026-07-29.md`;
- E7 pozostaje otwarte: donor jest `READY_MATERIALIZED_NO_PACKAGE`, ponieważ
  model iteration gate nie dopuszcza zamrożenia nowego exact V5 MOD/HAK.
  H2/r46 poprzedza project lifecycle i nie ma project revision, dlatego pełny
  wymóg manifestu nie może być oznaczony jako spełniony przez dopisywanie
  fikcyjnej wartości;
- wersja produktu nie została nadana, a zmiany są niecommitowane zgodnie
  z brakiem polecenia commit/push.
- `npm run typecheck` wraz z katalogiem i typed Worker/WASM contractem — PASS;
- `npm test` — PASS: 67 plików i 357 testów; 2 browser-only pliki i 2 testy
  świadomie pominięte w środowisku unit;
- `npm run build` wraz z WASM, contract i bundle budgets — PASS;
- pełny browser Worker/WASM — PASS: 9 plików i 23 testy;
- browser IndexedDB — PASS: 2 pliki i 2 testy;
- oficjalny złożony `npm run test:worker-integration` po świeżym WASM i
  regeneracji repo-owned fixtures — PASS 9/23 + IndexedDB 2/2;
- `git diff --check` — PASS; testy Rust pozostają zielone z zamknięcia E6,
  a E7 nie zmienia kodu Rust.

## 7. Rekomendowana kolejność

```text
E0 → E1 → E2 → E3 → E4 → E6 → E7
               ↘ E5 ↗
```

Najbliższy sensowny pakiet pracy:

1. E0: czerwony test, budżet 20k i dokumentacja;
2. E1: poprawna nawigacja oraz semantyka Custom;
3. E2: jeden eksportowalny projekt;
4. dopiero potem dalsze rozbudowywanie Animation Studio.

## 8. Czego nie potrzebujemy przed MVP

- pełnego modelera 3D zastępującego Blendera;
- automatycznego retargetu dowolnego rigu;
- backendu do przechowywania modeli;
- agentowego sterowania Toolset/NWN;
- bezpośredniej edycji binary MDL w UI;
- obsługi wszystkich rodzin modeli NWN naraz;
- mobilnego edytora keyframe'ów jako głównego interfejsu.

## 9. Kryterium gotowości całej aplikacji

Aplikacja jest gotowa do pierwszego wydania dopiero, gdy:

- [x] cały CI, Worker/web-WASM i browser persistence są zielone;
- [x] obowiązuje jeden budżet 20k;
- [x] każdy krok ma poprawne etykiety i brak atrap;
- [x] projekt można wyeksportować, odzyskać i związać po SHA;
- [x] każdy build targetów MVP ma własną identity i niekolizyjne nazwy;
- [x] Creature mapping i Custom library mają spójną semantykę;
- [x] Review pokazuje pełne evidence, a nie tylko skrót;
- [x] realne właścicielskie sample przechodzą browser E2E;
- [x] istnieje exact handoff do owner proof;
- [x] wymagany wynik Toolset/NWN został zapisany przez właściciela.

Te trzy kryteria odnoszą się do istniejącego, immutable r46 i osobno
przetestowanych ścieżek browser. Nie otwierają nowej iteracji V5 i nie
zastępują nadal niespełnionych wymagań E7: exact candidate Animation Studio,
pełny manifest tej iteracji oraz wersja wydania.

## 10. Powiązane dokumenty

- [Plan Animation Studio](plan-implementacji-animation-studio-2026-07-28.md)
- [Raport Animation Studio](raport-implementacji-animation-studio-2026-07-28.md)
- [Audyt Creature i animacji](audyt-aplikacji-creature-animacje-2026-07-28-codex.md)
- [Audyt Meshy API i animacji](audyt-meshy-api-animacje-2026-07-28-codex.md)
- [Architektura web/WASM](architektura-web-wasm-codex.md)
- [Orchestrator state](orchestrator-state.yaml)
- [Reguły projektu](PROJECT_RULES.md)
