# Poprawki po audycie rozwiązań animacji

Data: 2026-07-30  
Branch: `animation`  
Worktree: `C:\Projects\meshy2aurora\.worktrees\animation`  
Status: implementacja offline ukończona; pełne bramki jakości zielone

## Czwarty audyt i stan autorytatywny — 2026-07-30

Czwarty przegląd sprawdził nie tylko happy path, ale także fałszywe artefakty,
wyścigi asynchroniczne, pełną płatną lineage Meshy i zgodność dowodów. Ten
rozdział zastępuje starsze liczby testów i węższe kryteria z dalszej części
dokumentu.

- [x] import animacji z innego modelu działa od inspekcji dawcy do Custom;
  zapisuje `IMPORTED_MODEL_COPY`, dokładny donor `sourceRevision` i nazwę klipu;
- [x] zmiana bieżącego modelu podczas asynchronicznego importu unieważnia wynik,
  a równoległe zmiany dokumentu są scalane z najnowszą rewizją;
- [x] GLB przechodzi pełny lokalny readback kontenera, JSON/BIN, buforów,
  accessorów, sceny aktywnej, geometrii, indeksów, skinów, wag i animacji;
- [x] H1 wymaga nie tylko `skins[]`, lecz renderowanej weighted binding
  `JOINTS_0/WEIGHTS_0` i klipu z kanałem animującym joint;
- [x] martwy skin, geometria poza aktywną sceną, indeks jointa poza skórą,
  klip bez joint channel i header-only GLB są odrzucane;
- [x] limit H1 i wspólny budżet produktu wynoszą `300_000`; dokładnie 300 000
  przechodzi, a 300 001 jest blokowane;
- [x] `taskLedger` obejmuje Preview, Refine, Rig i każdą Animation, także taski
  zakończone błędem; `consumedCredits` sumuje cały ledger;
- [x] maksymalny koszt przed płatnym POST-em zależy od źródła, modelu,
  teksturowania i `texture_resolution`, w tym Smart Topology T2 oraz 8K;
- [x] requesty używają `texture_resolution=2k|4k|8k`; `hd_texture` nie jest
  wysyłane; Text-to-3D jawnie wymaga płatnego Refine, Meshy 5 jest ograniczone
  do 2K, a 8K do triangle topology;
- [x] hash kuratowanej allowlisty jest liczony z `actions`, sprawdzany przy
  starcie Bridge i wiązany z requestem niezależnie od hasha upstream katalogu;
- [x] attack demo przyjmuje wyłącznie semantyczne `ONE_SHOT`; phased loop musi
  zostać jawnie wyodrębniony jako osobny klip;
- [x] Base 42 assignment dziedziczy dokładne provenance definicji Custom, a Core
  odrzuca każdą rozbieżność;
- [x] owner proof registry wymaga Toolset `visible/verified`, NWN
  `visible/verified`, `animationPlayback=verified`, czasu UTC, fingerprintu,
  hashy MDL/HAK/MOD oraz SHA-256 istniejącego pliku evidence;
- [x] CSS po usunięciu wyłącznie martwych i kaskadowo przesłoniętych reguł ma
  `146_139 / 150_000 B`, czyli `3_861 B` zapasu zamiast 32 B;
- [x] nie uruchomiono Aurora/NWN i nie utworzono nowego MOD/HAK.

Aktualne bramki tego audytu:

- [x] canonical workspace i canonical Meshy asset layout;
- [x] `cargo fmt --check`;
- [x] `cargo clippy --workspace --all-targets -- -D warnings`;
- [x] `cargo test --workspace`;
- [x] exact local Void Crystal Knight replay — 1/1;
- [x] Studio — 383 passed, 3 skipped;
- [x] Worker/WASM — 23 passed, 2 environment-skipped;
- [x] browser persistence — 2/2;
- [x] Local Bridge — 35/35;
- [x] produkcyjny build, contract checks i bundle budgets;
- [x] `git diff --check`.

Końcowy build:

- main JS: `459_007 / 460_000 B`;
- main JS gzip: `126_610 / 130_000 B`;
- maksymalny chunk JS: `601_685 / 620_000 B`;
- total JS: `1_440_249 / 1_450_000 B`;
- total CSS: `146_139 / 150_000 B`;
- WASM: `3_261_882 / 4_000_000 B`;
- WASM gzip: `1_201_105 / 1_420_000 B`.

## Ponowny audit i hardening — 2026-07-30

Drugi i trzeci przegląd wykazały, że pierwsze zamknięcie faz A1–A7 było zbyt
optymistyczne. Poniższe poprawki są częścią tego samego celu i zastępują
wcześniejsze, węższe kryteria akceptacji:

- [x] runtime exposure Custom uwzględnia każdy zaakceptowany łańcuch fallback,
  a nie tylko bezpośrednie assignments;
- [x] binary readback porównuje pełny materialized clip: root, długość,
  transition, eventy, controllery, czasy i wartości;
- [x] owner runtime playback ma śledzony, domyślnie pusty rejestr, związany
  jednocześnie z fingerprintem Animation Studio oraz hashami MDL/HAK/MOD;
- [x] attack demo jest najpierw preview-only, wymaga osobnego Apply i pozwala
  odwrócić dokładnie zastosowaną zmianę;
- [x] attack demo działa także z zaakceptowanymi fallbackami i zachowuje
  efektywne źródło `cpause1`, materializując je bezpiecznie, gdy było
  dziedziczone przez slot ataku;
- [x] opóźniony `Edit copy` scala wynik z najnowszym dokumentem, nie ze starym
  closure, oraz czeka na zakończenie exact validation;
- [x] multi-key inspector wybiera zaznaczony keyframe pod playheadem;
- [x] Meshy używa śledzonego katalogu/allowlisty zamiast surowego pola ID;
- [x] H1 ma cztery jawne potwierdzenia; limit input task Meshy oraz wspólny
  budżet produktu mają aktualnie tę samą wartość 300 000, ale pozostają
  osobnymi kontraktami;
- [x] rigged base, basic walk/run i każdy udany action GLB są zachowywane
  niezależnie od późniejszych błędów;
- [x] historia odzyskuje Text-to-3D, Rigging oraz Animation wraz z wyborem
  konkretnej roli artefaktu;
- [x] zdalne `CANCELED` jest terminalne, a lokalna akcja nazywa się
  `Stop tracking locally` i nie udaje anulowania po stronie Meshy;
- [x] manifest zawiera source/rig/action identity, katalog, wersję API gdy
  zwrócona, timestamps, credits, skeleton signature oraz clip/root-motion
  inventory;
- [x] każdy pobrany GLB przechodzi strukturalny readback; klipy H1 z odmienną
  sygnaturą szkieletu są odrzucane;
- [x] bezpieczne GET-y Meshy mają ograniczony retry dla `429`; płatne POST-y
  nie są automatycznie ponawiane.

Rejestr
`contracts/owner-animation-playback-proofs-v1.json` pozostaje pusty do czasu
świeżego wyniku właściciela. Sam binary readback nie dodaje wpisu i nie może
zmienić `OWNER_PROOF_REQUIRED` na `OWNER_VERIFIED`.

## Cel

Dokument śledzi poprawki wynikające z audytu animacji. Rozdziela fakty
potwierdzone przez kod i binary readback od zachowania, które nadal wymaga
właścicielskiego proofu w Aurora Toolset/NWN.

## Stan faz

- [x] A1 — poprawny kontrakt runtime animacji Custom
- [x] A2 — deterministyczny kontrakt demo bez podmiany `cpause1`
- [x] A3 — osobne osie binary readback i runtime playback proof
- [x] A4 — wieloklipowy Meshy Bridge → Animation Studio
- [x] A5 — wspólny budżet produktu 300 000 trójkątów
- [x] A6 — poprawki UX edytora
- [x] A7 — pełne bramki jakości i dokumentacja
- [-] A8 — właścicielski proof runtime

## A1. Kontrakt runtime Custom

### Problem

Sama obecność dowolnie nazwanej animacji Custom w binarnym MDL nie dowodzi,
że NWScript lub AI Aurory potrafi ją wywołać. W runtime gwarantowany kontrakt
stanowią sloty Base 42.

### Zaimplementowane funkcje

- [x] `CustomAnimationRuntimeExposureStatusV1`
- [x] `CustomAnimationRuntimeExposureV1`
- [x] `inspect_custom_animation_runtime_exposure_v1`
- [x] raportowanie `LIBRARY_ONLY` dla Custom bez routingu Base 42
- [x] raportowanie `BASE_42_ROUTED` z dokładną listą slotów
- [x] przeniesienie ekspozycji do audytu builda i canonical Review
- [x] UI rozróżniające „library only” od „Base 42 routed”

### Reguła akceptacji

- [x] Binary readback może potwierdzić, że klip istnieje i ma kontrolery.
- [x] Tylko przypisanie do Base 42 daje zadeklarowaną ekspozycję runtime.
- [ ] Faktyczne odtworzenie przez AI/skrypt wymaga owner proofu.

## A2. Kontrakt attack demo

### Problem

Poprzedni packet Void Crystal Knight wymusił ruch przez podmianę `cpause1`.
To demonstrowało automatyczne odtwarzanie, ale fałszowało produkcyjny idle i
nie dowodziło prawdziwego wywołania ataku.

### Zaimplementowane funkcje

- [x] `CUSTOM_ATTACK_DEMO_BASE_SLOTS_V1`
- [x] `CustomAttackDemoContractV1`
- [x] `CustomAttackDemoAuthoringV1`
- [x] `apply_custom_attack_demo_route_v1`
- [x] odpowiednik TypeScript `applyCustomAttackDemoRouteV1`
- [x] akcja UI `Prepare attack demo (3 slots)`
- [x] akcja UI rozdzielona na `Preview attack demo`, `Apply` i `Revert`
- [x] jeden Custom routowany do `ca1slashl`, `ca1slashr`, `ca1stab`
- [x] dokładne zachowanie istniejącego `cpause1`
- [x] zachowanie efektywnego `cpause1` także przy zaakceptowanym fallbacku
- [x] jedna deterministyczna zmiana revision
- [x] jawny profil triggera: aktywny potwór wybiera wariant ataku przez combat AI

### Granica dowodu

Routing do trzech wariantów usuwa losową lukę „AI wybrało inny wariant”, ale
nie gwarantuje momentu wykonania ataku. Runtime nadal wymaga właścicielskiego
testu walki. Nie utworzono nowego MOD/HAK dla tej poprawki.

## A3. Binary readback a runtime playback

### Zaimplementowane funkcje

- [x] `CanonicalCustomAnimationRuntimeExposureV1`
- [x] `CanonicalAnimationPlaybackAcceptanceV1`
- [x] status downloadu `BINARY_READY`
- [x] niezależny status `OWNER_PROOF_REQUIRED`
- [x] `playbackProofStatus=not_tested`
- [x] `proofCompleteness=missing`
- [x] exact owner-proof registry związany z fingerprintem Studio i hashami
  `model` / `hak` / `proofModule`
- [x] osobne sekcje Review: `Binary MDL readback` i `Runtime playback proof`
- [x] poprawna nazwa `Animation-root translation`

### Zakaz nadinterpretacji

`MATCH`, obecne kontrolery i changing motion nie mogą zmienić statusu runtime
na verified. Tylko świeży, związany z dokładnym kandydatem wynik właściciela
może zamknąć tę oś.

## A4. Wieloklipowy Meshy Bridge → Studio

### Zaimplementowane funkcje

- [x] wejście `animationActionIds` z 1–10 unikalnymi ID
- [x] koszt `35 + 3 × liczba animacji`
- [x] osobne task ID `ANIMATE_<actionId>`
- [x] osobny SHA-256 i `byteLength` dla każdego GLB
- [x] endpoint artefaktu związany z dokładnym `actionId`
- [x] walidacja długości, nagłówka GLB i SHA-256 w kliencie Studio
- [x] katalog `Verified animation GLBs` w Meshy Lab
- [x] dodawanie wielu donorów bez zastępowania źródłowego modelu
- [x] bezpośrednia lista donorów w `Copy animation from another model`
- [x] ponowna kontrola zgodności rigu przed `Copy to Custom`
- [x] własny strukturalny GLB readback i skeleton equality przed zachowaniem
  klipu
- [x] manifest action/catalog/API/task/skeleton/clip/root-motion
- [x] lokalny file picker pozostaje dostępny

## A5. Budżet geometrii

- [x] `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000`
- [x] dokładnie 300 000 jest dozwolone
- [x] 300 001 jest odrzucane
- [x] Studio i Local Bridge używają tej samej wartości
- [x] limit Meshy Smart Topology T2 15 000 pozostaje osobnym limitem API
- [x] granica jednego streamu binary MDL 21 845 trójkątów pozostaje osobną
  granicą writera i nie jest limitem produktu

## A6. UX Animation Studio

- [x] inspektor inicjalizuje wartości z wybranego keyframe'u
- [x] bez wybranego keyframe'u inspektor próbkuje track na playheadzie
- [x] brak tracku daje neutralną rotację lub zerową translację
- [x] UI rozróżnia wartość bieżącą od wartości zapisywanej
- [x] kliknięcie markera keyframe'u ustawia dokładny playhead
- [x] `Save to Custom` pokazuje trwającą exact validation
- [x] dokument, timeline, inspektor, undo/redo i dialogi są zablokowane podczas
  walidacji
- [x] mutacja wywołana programowo podczas blokady nie zmienia dokumentu

## A7. Weryfikacja

### Zaliczone bramki cząstkowe

- [x] canonical workspace assertion
- [x] core `animation_studio` — 16 testów
- [x] core `animation_studio_v5` — 8 testów oraz 1/1 exact local replay
- [x] Studio targeted Meshy/Animation/Review — zielone
- [x] Studio typecheck — zielony
- [x] Local Bridge — 35/35

### Pełne bramki przed zamknięciem

- [x] canonical Meshy asset layout
- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] exact local Void Crystal Knight replay — 1/1
- [x] pełne testy Studio — 383 passed, 3 skipped
- [x] Worker/WASM integration — 23 passed, 2 environment-skipped
- [x] browser persistence — 2/2
- [x] produkcyjny build Studio
- [x] bundle/WASM budgets
- [x] Local Bridge — 35/35
- [x] `git diff --check`

Release `opt-level="s"` dla `m2a-core` i `m2a-wasm` oraz końcowe
`wasm-opt -Oz --converge` dały:

- WASM raw: 3 261 882 / 4 000 000 B;
- WASM gzip: 1 201 105 / 1 420 000 B;
- main JS: 459 007 / 460 000 B;
- total JS: 1 440 249 / 1 450 000 B;
- total CSS: 146 139 / 150 000 B.

## A8. Handoff i proof właściciela

Dokładny, już zainstalowany kandydat sprzed poprawki kontraktu:

- MOD: `m2c7a58c250ba516.mod`
- nazwa modułu w Toolset:
  `Meshy2Aurora procedural humanoid proof`
- Area: `Meshy2Aurora M0 binary vertical-slice area`
- HAK: `m2c7a58c250ba516.hak`
- Appearance: `15101`
- `modelVisibility=not_tested`
- `proofCompleteness=missing`

Ten packet nadal zastępuje `cpause1` historycznym `vck_showcase`. Właściciel
może ocenić tylko tę dokładną lineage i zgłosić wynik. Nie wolno traktować go
jako proofu nowego, trójslotowego kontraktu ataku.

Nowa materializacja poprawionego kontraktu jest wstrzymana przez model
iteration gate. Agent nie uruchamia Aurora Toolset/NWN, nie tworzy kolejnego
resrefu i nie nadpisuje zainstalowanych plików.

Dokładny handoff bramy:
[`evidence/animation-audit-fixes-owner-proof-gate-2026-07-30.md`](evidence/animation-audit-fixes-owner-proof-gate-2026-07-30.md).

## Pliki implementacji

- `crates/m2a-core/src/animation_studio.rs`
- `crates/m2a-core/src/model_pipeline.rs`
- `apps/studio-web/src/features/animation-editor/editing.ts`
- `apps/studio-web/src/features/animation-editor/CustomAnimationMappingPanel.tsx`
- `apps/studio-web/src/features/animation-editor/AnimationStudioWorkspace.tsx`
- `apps/studio-web/src/features/animation-editor/BoneTransformInspector.tsx`
- `apps/studio-web/src/features/animation-editor/ImportAnimationFromModelDialog.tsx`
- `apps/studio-web/src/features/meshy/bridge.ts`
- `apps/studio-web/src/features/meshy/MeshyLab.tsx`
- `apps/studio-web/src/features/results/projectCanonicalResult.ts`
- `apps/studio-web/src/features/review/reconcileAnimationStudioReadback.ts`
- `apps/studio-web/src/features/review/AuthoredAnimationReview.tsx`
- `tools/meshy-local-bridge/index.mjs`
- `tools/meshy-local-bridge/glb-artifact-inspection.mjs`
- `contracts/meshy-animation-catalog-v1.json`
- `contracts/owner-animation-playback-proofs-v1.json`
