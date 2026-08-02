# Audyt funkcjonalności ustawiania przodu Creature

Data: 2026-07-31
Status: `P0 IMPLEMENTED OFFLINE / OWNER VISUAL PROOF REQUIRED / NO NEW PAID RUN`

## Aneks wdrożeniowy — 2026-07-31

Zakres P0 z tego audytu został zaimplementowany w pełnym torze produktu:

- Creature ma jawny, wersjonowany wybór przodu źródła: `+Z`, `-Z`, `+X` albo
  `-X`; każda opcja jest mapowana do natywnego kierunku Aurora/NWN `-Y`
  macierzą o wyznaczniku `+1`;
- wybór jest dostępny w Source, widoczny jako strzałka w podglądzie i wchodzi
  do tożsamości artefaktu, żądania Workera, WASM, rdzenia Rust, raportu,
  summary i manifestu;
- zmiana kierunku unieważnia poprzedni Build, więc nie można pobrać artefaktu
  zbudowanego dla innego ustawienia;
- Studio i WASM wykonują dokładny handshake
  `M2A_STUDIO_WASM_2026_07_31_V1`; niezgodna paczka jest blokowana;
- Studio i Local Bridge wykonują handshake protokołu `2` oraz capability
  `M2A_MESHY_BRIDGE_2026_07_31_V2`; stary Bridge jest blokowany przed płatnym
  uruchomieniem;
- po scaleniu animacji Bridge ponownie odczytuje finalny GLB i wymaga dokładnej
  kolejności oraz nazw klipów; import do Source wiąże plik z SHA-256 i
  provenance konkretnego runu;
- gotowe runy Bridge są zapisywane bez sekretów i podpisanych URL-i w lokalnym
  journalu, a Studio odzyskuje aktywny run po przeładowaniu;
- gotowe artefakty Build są zapisywane w IndexedDB i przed odzyskaniem mają
  ponownie sprawdzany SHA-256 oraz rozmiar.

Stan weryfikacji offline po wdrożeniu:

- `cargo clippy --workspace --all-targets -- -D warnings` — PASS;
- `cargo fmt --all -- --check` — PASS;
- Studio typecheck — PASS;
- Studio unit/component tests — 244/244 PASS;
- Worker/WASM integration — 9 PASS, 2 środowiskowe SKIP;
- Local Bridge — 28/28 PASS;
- lokalny proces Bridge został odtworzony i zweryfikowany jako protokół `2`,
  `READY`, z pełnym capability contract V2 i trwałym wolumenem journalu;
- canonical-workspace i Meshy asset-layout preflight — PASS.

Pełne `cargo test --workspace --quiet` nie zgłosiło błędu, ale przekroczyło
limit wykonania 300 sekund na istniejących ciężkich testach korpusu. Testy
bezpośrednio obejmujące Basis V2, rdzeń pipeline'u i kompilację workspace były
zielone; timeout nie jest klasyfikowany jako niepowodzenie funkcjonalności.

Nie utworzono nowej płatnej generacji Meshy ani nowej iteracji MOD/HAK. Ostatnim
kryterium pozostaje właścicielski test dokładnego przyszłego artefaktu w Aurora
Toolset i NWN; implementacja nie może sama podnieść statusu do runtime proof.

## Werdykt

Funkcjonalność, którą użytkownik miał przetestować, nie jest obecnie kompletną
funkcją ustawiania przodu Creature. Kod zawiera jedną stałą politykę
`source +Z -> Aurora -Y`, ale Studio nie pozwala wskazać, w którą stronę jest
zwrócony konkretny model źródłowy. Raport potwierdza zastosowanie tej stałej
macierzy, a nie rozpoznanie faktycznego przodu postaci.

Próba E2E z 2026-07-31 nie mogła udowodnić poprawki z trzech niezależnych,
potwierdzonych powodów:

1. uruchomiony Local Bridge był starszy od implementacji wieloanimacyjnej i
   zrealizował tylko jedną akcję;
2. testowana paczka WASM nie była dokładną, aktualną kompilacją końcowego kodu
   Creature Basis V2;
3. reload Vite usunął stan Source/Build i gotowe MOD/HAK przed pobraniem, a
   aplikacja nie ma trwałego odzyskiwania lokalnego runu Image-to-3D ani
   wyników Build.

Nie ma więc podstaw do twierdzenia, że ustawianie przodu zostało sprawdzone
przez aplikację albo że powstało demo gotowe do proofu właściciela.

## Potwierdzone dowody

### 1. Obecny kontrakt nie jest ustawieniem użytkownika

- `ProfileABasisPolicyV1` zawiera jedną nową politykę Creature:
  `GltfYUpPositiveZForwardToAuroraZUpNegativeYForwardV2`.
- `direct_creature_profile_a_options_v2()` wybiera ją bezwarunkowo.
- `ProceduralCreatureBuildOptionsV1` zawiera tylko czyszczenie tekstury i
  stabilizację skinned accessories; nie zawiera source-forward, yaw ani
  wyboru osi.
- `SourceStep`, `App`, kontrakt Workera i WASM nie udostępniają kontrolki ani
  parametru kierunku Creature.
- `CREATURE_BASIS_V2_RESOLVED` oraz
  `GLTF_POSITIVE_Z_TO_AURORA_NEGATIVE_Y` są stałymi etykietami wynikającymi z
  wybranej polityki. Pipeline nie mierzy semantycznego przodu siatki.

Wniosek: dla źródła rzeczywiście skierowanego w `+Z` macierz może być
poprawna. Dla źródła skierowanego w `-Z`, `+X` albo `-X` wynik nadal będzie
obrócony, a raport może mimo to wyglądać poprawnie.

### 2. Local Bridge wykonywał stary kontrakt jednej animacji

- kontener `meshy2aurora-meshy-bridge-1` rozpoczął proces
  `2026-07-29T18:37:08Z`;
- merge obsługi nazwanych zestawów animacji wszedł w commit
  `f28361984dd6913a4a9a678d20036d3e29c8bb66` z datą
  `2026-07-30T23:40:03Z`, już po uruchomieniu procesu;
- Bridge jest zwykłym procesem Node i nie ma hot reloadu;
- stara implementacja liczyła `35 + 3 * liczba akcji`, a przy nieznanym polu
  `animationActions` wracała do jednej akcji `0`; dlatego review pokazało
  dokładnie `38` kredytów;
- stara implementacja wybierała `run.artifacts[0].bytes`, zamiast scalać
  wszystkie action GLB;
- zaimportowany GLB miał dokładnie jeden klip
  `Armature|Idle|baselayer`, co zgadza się z tym zachowaniem.

To zamyka wcześniejszą niepewność: utrata czterech animacji nastąpiła w
niezrestartowanym, starym procesie Bridge, przed importem do Source. Inspector
GLB i writer NWN nie usunęły tych czterech klipów, ponieważ nigdy ich nie
dostały.

### 3. Test nie był związany z dokładną wersją WASM

- kontener Studio działa od `2026-07-29T18:37:08Z`;
- `docker-compose.yml` buduje WASM tylko wtedy, gdy `pkg/m2a_wasm.js` nie
  istnieje; zmiany Rust wymagają osobnego `RebuildWasm`;
- końcowy plik `profile_a.rs` z Basis V2 został zmodyfikowany po ostatnim
  rebuildzie WASM poprzedzającym test;
- log Vite pokazuje późniejsze przebudowy `m2a_wasm.js` i
  `m2a_wasm_bg.wasm`, już po utracie wyniku testu.

Wniosek: zielone testy aktualnego źródła nie dowodzą, że przeglądarka używała
tej samej implementacji podczas runu. Brakuje runtime handshake wiążącego UI,
Bridge i WASM z jednym kontraktem/buildem.

### 4. HMR usunął jedyny wynik testu

- Build zakończył się i pokazał hashe MOD/HAK/MDL/TGA;
- log Studio potwierdza reload Workera o `2026-07-31T20:44:44Z`;
- Source, wynik Build i blob-y artefaktów są stanem pamięciowym Reacta;
- Bridge trzyma `runs` wyłącznie w pamięci i udostępnia GET tylko po znanym
  `run.id`;
- ekran History odpytuje wyłącznie historię Text-to-3D Meshy, więc nie
  odzyskuje lokalnego runu Image-to-3D z animacjami.

Wynik nie został pobrany ani zainstalowany. Nie istnieje handoff
`ready_for_owner_proof`.

## Dlaczego testy offline tego nie złapały

Aktualne testy potwierdzają, że:

- macierz V2 mapuje matematyczne `+Z` na `-Y` i ma determinant `+1`;
- aktualny kod Bridge potrafi scalić do dziesięciu zgodnych action GLB;
- geometria, rig i źródłowe animacje są zachowywane przez te jednostkowe
  ścieżki.

Nie sprawdzają jednak:

- czy przód rzeczywistego źródła jest zgodny z założonym `+Z`;
- czy UI przekazało wybraną przez użytkownika oś;
- czy działający proces Bridge/WASM ma ten sam kontrakt co UI;
- czy żądana liczba akcji jest równa liczbie klipów w canonical GLB;
- czy run i Build można odzyskać po reloadzie.

Targetowane bramki uruchomione podczas audytu:

- `node --test tools/meshy-local-bridge/bridge.test.mjs tools/meshy-local-bridge/merge-animation-glbs.test.mjs` — 26/26 PASS;
- `cargo test -p m2a-core --test profile_a creature_basis_v2 -- --nocapture` — 2/2 PASS.

To potwierdza rozjazd środowiska/kontraktu, a nie błąd samych obecnych testów
macierzy albo funkcji merge.

## Plan implementacji

### P0. Jawny kontrakt source-forward

1. Wprowadzić wersjonowane `CreatureSourceForwardV1` z wartościami
   `POSITIVE_Z`, `NEGATIVE_Z`, `POSITIVE_X`, `NEGATIVE_X`.
2. Dodać je do nowego, jawnie wersjonowanego build-options; historyczny
   domyślny wariant zachować jako `POSITIVE_Z`, bez mutowania zamrożonych
   lineage.
3. Dla każdej osi wyliczać właściwy obrót, który mapuje wybrany source-forward
   na Aurora `-Y`, zachowuje source-up `+Y -> Aurora +Z` i ma determinant `+1`.
4. Tę samą bazę stosować do geometrii, normalnych, tangentów, bounds, bind
   pose, translacji i rotacji wszystkich klipów.
5. Związać wybór z reportem, manifestem, summary i hashem opcji artefaktu.

Nie należy implementować pozornego `AUTO`, dopóki nie istnieje wiarygodny,
testowalny detektor semantycznego przodu. Dla humanoida wybór operatora jest
bezpieczniejszy od zgadywania z bounds albo PCA.

### P0. Studio i podgląd

1. Dodać w kroku Source kontrolkę `Przód modelu w źródle` z czterema osiami i
   czytelnym opisem celu `Aurora -Y`.
2. Przekazać wybór przez `App -> StudioWorkerRequest -> WASM -> m2a-core`.
3. W Source/Converted viewport pokazać gizmo osi i strzałkę `Front`; zmiana
   wyboru ma natychmiast unieważniać stary Build.
4. W Review pokazywać wybraną oś, wynikowe mapowanie, determinant i status
   `OWNER_PROOF_REQUIRED`.

### P0. Ochrona przed starym Bridge i WASM

1. Podnieść wersję protokołu po semantycznej zmianie wieloanimacyjnej.
2. Rozszerzyć `/v1/health` o capabilities/build fingerprint, co najmniej:
   `multiAnimationMerge`, `namedClipMapping`, `maxAnimationActions`,
   `creditFormulaVersion` i `artifactRecovery`.
3. Studio ma zablokować płatne `Confirm`, jeśli wymagane capabilities albo
   fingerprint są niezgodne, i wskazać bezpieczny restart Bridge.
4. Worker/WASM ma zwracać swój ABI/capability fingerprint; Build ma być
   zablokowany, jeżeli UI oczekuje innego kontraktu facing.
5. Dev workflow ma przebudowywać WASM deterministycznie lub wykrywać różnicę
   hashy źródło/pkg przed testem E2E.

### P0. Integralność animacji

1. Provenance ma deklarować żądane mapowania i każdy action artifact.
2. Po merge Bridge ma ponownie sparsować canonical GLB i wymagać dokładnie
   oczekiwanych nazw oraz liczby klipów.
3. Po pobraniu Studio ma porównać provenance z Source Inspect i zablokować
   Build przy `requested != imported`.
4. Review kosztów musi pochodzić z tego samego capability/version contract co
   wykonawca runu.

### P0. Odzyskiwanie po reloadzie

1. Bridge ma trwale przechować redacted run receipt, task IDs, provenance,
   canonical GLB i hash do jawnego wyczyszczenia/wygaśnięcia.
2. Udostępnić listę lokalnych runów obejmującą Image-to-3D/Rig/Animation, bez
   API key i bez signed URLs.
3. Studio ma zapisać aktywne `run.id` i po ponownym połączeniu odzyskać exact
   artifact bez tworzenia płatnego zadania.
4. Wyniki Build wraz z blobami i hashami przechować w IndexedDB albo innym
   trwałym, limitowanym magazynie lokalnym, aby HMR nie niszczył handoffu.

### P1. E2E i handoff

1. Najpierw wykonać testy syntetyczne oraz test uruchomionego stacku bez
   płatnego Meshy.
2. Użyć asymetrycznego humanoidalnego fixture skierowanego kolejno w każdą z
   czterech osi i potwierdzić jednakowy wynik Aurora `-Y`.
3. Przeprowadzić test reloadu na etapie READY, po imporcie i po Build.
4. Dopiero po zielonych bramkach i zgodnie z model-iteration gate przygotować
   jeden exact MOD/HAK, zainstalować go hash-identycznie w native katalogach i
   przekazać właścicielowi. Agent nie uruchamia Aurory ani NWN.

## Kryteria ukończenia

### Kontrakt kierunku

- [ ] Użytkownik może wskazać `+Z`, `-Z`, `+X` albo `-X` jako przód źródła.
- [ ] Każda opcja mapuje wskazany wektor dokładnie na Aurora `[0,-1,0]`.
- [ ] Source-up `[0,1,0]` mapuje się dokładnie na Aurora `[0,0,1]`.
- [ ] Determinant pełnej transformacji wynosi `+1`.
- [ ] Geometria, rig, bind pose i każdy klip używają tej samej transformacji.
- [ ] Liczba trójkątów, UV, materiały i czasy klipów nie zmieniają się przez
      sam wybór przodu.
- [ ] Report/manifest/summary podają wybraną oś i wynikowe mapowanie; zmiana
      opcji zmienia tożsamość Build.

### UI i fail-closed

- [ ] Kontrolka jest widoczna wyłącznie dla Creature i unieważnia stary wynik.
- [ ] Viewport pokazuje strzałkę Front zgodną z wybraną osią.
- [ ] Nie istnieje status `RESOLVED` bez jawnego source-forward w artefakcie.
- [ ] Stary Bridge albo stary WASM blokuje płatny run/Build przed wykonaniem,
      zamiast cicho wracać do domyślnej funkcji.

### Animacje i recovery

- [ ] Żądanie pięciu mapowań daje canonical GLB z dokładnie pięcioma nazwami.
- [ ] Rozbieżność action count/provenance/Source Inspect zatrzymuje Build.
- [ ] Reload w READY pozwala pobrać ten sam GLB o tym samym SHA-256.
- [ ] Reload po Build pozwala pobrać te same MOD/HAK/MDL/TGA o tych samych
      hashach.
- [ ] Odzyskanie nie tworzy nowego płatnego zadania.

### Dowód końcowy

- [ ] Offline: pełne Rust/WASM/Studio/Bridge gates są zielone.
- [ ] Running-stack E2E zapisuje fingerprint UI/Bridge/WASM i exact input/output
      hashes.
- [ ] Exact MOD/HAK są zainstalowane z weryfikacją byte-for-byte.
- [ ] Właściciel potwierdza ten sam model przodem w Aurora Toolset i NWN.

Do spełnienia ostatniego punktu status pozostaje `OWNER_VISUAL_PROOF_REQUIRED`.
