# Plan implementacji: Creature Animation Mapping

Data: 2026-07-28
Branch: `animation`
Status: AKTYWNY PLAN WYKONAWCZY
Interfejs referencyjny:
[`mockups/creature-animation-mapping-v2-2026-07-28/02-aurora-state-mapping-ux-corrected.png`](mockups/creature-animation-mapping-v2-2026-07-28/02-aurora-state-mapping-ux-corrected.png)

## 1. Cel

Zaimplementować w Studio etap importu creature, w którym gracz:

1. widzi standardowe stany/akcje Aurory i ich wynikowe sloty animacji;
2. przypisuje klipy źródłowe z GLB lub świadomie akceptuje fallback;
3. może dodawać animacje niestandardowe;
4. podgląda wynik mapowania przed zbudowaniem pakietu NWN;
5. nie musi znać surowych ścieżek do modeli ani samodzielnie wybierać
   technicznych nazw slotów.

Ten plan dotyczy warstwy aplikacji, kontraktu WASM/core oraz integracji z
istniejącym buildem. Nie zastępuje końcowego proofu wizualnego w Aurora Toolset
i NWN, który zgodnie z decyzją właściciela wykonuje człowiek.

## 2. Jak aktualizować checklistę

- `[ ]` — funkcja nieukończona;
- `[-]` — funkcja rozpoczęta lub częściowa;
- `[x]` — funkcja ukończona wraz z testami i wymaganym dowodem;
- etap można oznaczyć jako ukończony dopiero po odznaczeniu wszystkich jego
  pozycji `Definition of Done`;
- samo istnienie kodu nie wystarcza do zaznaczenia `[x]`;
- przy zmianie statusu należy dopisać krótki link do testu, raportu albo
  commitu w sekcji `Dziennik postępu`.

GitHub renderuje natywnie tylko `[ ]` i `[x]`. W razie aktualizacji planu przez
UI GitHub częściowe `[-]` można pozostawić jako tekstowy stan roboczy.

## 3. Niezmienny kontrakt domenowy

- [x] Udokumentowano, że dla bezpośredniego creature `MODELTYPE=S/L`
  obowiązuje katalog 42 bazowych slotów.
- [x] Udokumentowano, że liczba 42 nie oznacza wszystkich stanów całego
  silnika Aurora.
- [x] Udokumentowano, że `ANIMATION_*` jest selektorem NWScript, a nie pełnym
  katalogiem stanów silnika.
- [x] Udokumentowano, że `Loop`, `Fire-and-forget`, `Start` i `End` opisują
  sposób odtwarzania lub fazę, a nie stan Aurory.
- [x] Istnieje kanoniczna lista 42 slotów w Rust i TypeScript.
- [x] Istnieje generator pełnego natywnego profilu 42 oraz ścieżka readback.
- [x] Ustanowić jeden wersjonowany kontrakt danych używany przez Rust, WASM,
  worker i UI bez ręcznego powielania znaczeń.
- [x] Potwierdzić testem parytet kolejności, nazw i liczby slotów pomiędzy Rust
  i TypeScript.

Obowiązujący przepływ:

```text
stan/akcja Aurory
-> MODELTYPE
-> bazowy slot modelu
-> realizacja slotu: lokalna | odziedziczona | wygenerowana | użytkownika
-> walidacja i jawna akceptacja fallbacku
-> build i binary readback
```

Pola po lewej stronie tego przepływu są wyliczane i tylko do odczytu.
Edytowalne jest źródło realizujące wynikowy slot oraz jawne decyzje użytkownika.

## 4. Stan całości

- [x] E0 — zamknięty kontrakt danych i integracji
- [x] E1 — katalog stanów i bazowych slotów
- [x] E2 — inspekcja klipów źródłowych i automatyczne mapowanie
- [x] E3 — stan sesji, rewizje i zapis projektu
- [x] E4 — nowy krok workflow i szkielet ekranu
- [x] E5 — edytor mapowania, fallbacki i animacje custom
- [x] E6 — podgląd animacji i diagnostyka
- [x] E7 — gotowość, walidacja i dostępność UX
- [x] E8 — kontrakt WASM, worker i m2a-core
- [x] E9 — build, readback, Review i eksport
- [x] E10 — pełna weryfikacja offline
- [x] E11 — pakiet handoff do proofu właściciela; authored V4 odtwarza
  bajtowo identyczny, zainstalowany candidate r46 bez tworzenia nowej iteracji

## 5. Etapy implementacji

### E0. Kontrakt danych i granice rozwiązania

Cel: przed implementacją UI ustalić nazwy, wersjonowanie i odpowiedzialność
każdej warstwy.

#### Funkcje i typy

- [x] Dodać `CreatureAnimationAuthoringV1` — trwały dokument decyzji
  użytkownika.
- [x] Dodać `AuroraAnimationStateDefinitionV1` — semantyka stanu/akcji i reguła
  rozwiązania slotu.
- [x] Dodać `DirectCreatureBaseSlotV1` — jeden z 42 kanonicznych slotów `S/L`.
- [x] Dodać `SourceAnimationClipV1` — znormalizowany opis klipu wejściowego.
- [x] Dodać `AnimationSourceAssignmentV1` — przypisanie źródła do slotu.
- [x] Dodać `AnimationFallbackDecisionV1` — fallback, jego cel, powód i decyzja
  `PENDING | ACCEPTED | REJECTED`.
- [x] Dodać `CustomAnimationDefinitionV1` — nazwa custom, selektor i fazy
  `START | LOOP | END`.
- [x] Dodać `AnimationMappingProvenanceV1`, rozdzielając:
  `provider`, `asset` i `ownership`.
- [x] Dodać `AnimationMappingDiagnosticV1` z poziomami
  `INFO | WARNING | BLOCKING`.
- [x] Dodać `CreatureAnimationMappingStatusV1` z wartościami
  `READY | NEEDS_REVIEW | BLOCKED`.

#### Decyzje do zapisania w kodzie i testach

- [x] Zakazać edycji wynikowego `MODELTYPE` i bazowego slotu.
- [x] Zakazać podawania przez użytkownika surowej ścieżki do supermodelu.
- [x] Nie uznawać fallbacku za gotowy bez wskazanego celu i jawnej akceptacji.
- [x] Nie dopuścić konfiguracji `customXlp + ONE_SHOT + wymagany END`.
- [x] Zachować 42 bazowe sloty niezależnie od liczby dodanych animacji custom.
- [x] Zachować kompatybilność istniejących API V2/V3 i dodać nowy,
  wersjonowany kontrakt zamiast zmieniać ich znaczenie.

#### Definition of Done

- [x] Typy mają komentarze opisujące granicę stan/slot/klip/playback.
- [x] Przykładowy dokument JSON przechodzi deserializację w Rust i TypeScript.
- [x] Niepoprawne kombinacje mają stabilne kody diagnostyczne.
- [x] Kontrakt został podlinkowany z
  `documentation/aurora-animation-system-codex.md`.

---

### E1. Katalog stanów i bazowych slotów

Cel: jedna kanoniczna projekcja dla panelu `Base 42`, wyszukiwarki, filtrów i
resolvera.

Docelowy moduł UI:
`apps/studio-web/src/features/animation-mapping/catalog.ts`

#### Funkcje

- [x] `getDirectCreatureBaseCatalogV1()` — zwraca dokładnie 42 rekordy w
  stabilnej kolejności.
- [x] `getAuroraAnimationStateCatalogV1()` — zwraca semantyczne stany/akcje
  wspierane przez profil, bez mieszania selektorów `ANIMATION_*`.
- [x] `resolveBaseSlotForStateV1(stateId, modelType)` — wylicza właściwy slot
  dla wybranego stanu i `MODELTYPE`.
- [x] `getPlaybackPolicyForStateV1(stateId)` — zwraca wyliczoną politykę
  playback bez tworzenia fikcyjnego pola `loop` w MDL.
- [x] `isDirectCreatureBaseSlotV1(value)` — type guard kanonicznej nazwy.
- [x] `findBaseSlotDefinitionV1(slot)` — bezpieczny lookup definicji slotu.
- [x] `filterAnimationCatalogV1(rows, query, filter)` — obsługa widoków
  `Needs attention`, `Base 42`, `Custom`.
- [x] `projectAnimationCatalogRowsV1(authoring, inspection)` — buduje wiersze
  gotowe do wyświetlenia.
- [x] `assertDirectCreatureCatalogParityV1()` — testowa kontrola zgodności
  katalogu TS z eksportem core/WASM.

#### Zawartość jednego wiersza katalogu

- [x] stabilne `stateId`;
- [x] etykieta i krótki opis dla gracza;
- [x] wyliczony `MODELTYPE`;
- [x] wyliczony bazowy slot;
- [x] wyliczona polityka playback;
- [x] źródło realizacji;
- [x] status i diagnostyki;
- [x] provenance: provider, asset, ownership.

#### Definition of Done

- [x] Test potwierdza dokładnie 42 unikalne sloty bazowe.
- [x] Test odrzuca duplikaty, brak slotu i zmianę kolejności.
- [x] Żaden filtr ani animacja custom nie zmienia licznika `Base 42`.
- [x] Snapshot przykładowych stanów pokazuje poprawne rozwiązanie zależne od
  `MODELTYPE`.

---

### E2. Inspekcja źródła i automatyczne mapowanie

Cel: po załadowaniu GLB aplikacja rozpoznaje klipy i proponuje mapowanie, ale
nie ukrywa niepewnych decyzji.

Docelowe moduły:

- `apps/studio-web/src/features/animation-mapping/sourceClips.ts`
- `apps/studio-web/src/features/animation-mapping/autoMap.ts`
- `apps/studio-web/src/features/animation-mapping/fallbacks.ts`

#### Funkcje inspekcji

- [x] `normalizeSourceAnimationNameV1(name)` — normalizacja wyłącznie do
  porównania, bez zmiany nazwy artefaktu.
- [x] `inventorySourceAnimationClipsV1(gltfAnimations)` — indeks nazw, czasu,
  tracków i targetów.
- [x] `classifySourceAnimationClipV1(clip)` — kandydaci znaczenia wraz z
  confidence i uzasadnieniem.
- [x] `detectSourceClipDuplicatesV1(clips)` — konflikt nazw lub znaczeń.
- [x] `validateSourceClipRigTargetsV1(clip, rig)` — brakujące albo obce kości.
- [x] `summarizeSourceAnimationCoverageV1(clips)` — pokrycie, braki i konflikty.

#### Funkcje auto-mapowania

- [x] `rankSourceClipCandidatesV1(state, clips)` — deterministyczny ranking
  kandydatów.
- [x] `proposeCreatureAnimationMappingV1(catalog, clips)` — propozycje bez
  automatycznej akceptacji niepewnych fallbacków.
- [x] `applyHighConfidenceAssignmentsV1(authoring, proposal)` — automatycznie
  przypisuje tylko jednoznaczne przypadki.
- [x] `createFallbackProposalV1(targetSlot, sourceSlot, reason)` — jawny cel,
  źródło i powód.
- [x] `approveFallbackV1(authoring, fallbackId)` — zapisuje decyzję i rewizję.
- [x] `rejectFallbackV1(authoring, fallbackId)` — usuwa fallback z gotowego
  rozwiązania i przywraca blocker.
- [x] `resetAssignmentToProposalV1(authoring, slot)` — bezpieczny powrót do
  rekomendacji.
- [x] `explainMappingProposalV1(proposal)` — tekst UX wyjaśniający regułę.

#### Definition of Done

- [x] Ten sam input zawsze daje tę samą propozycję i kolejność kandydatów.
- [x] Niski confidence nigdy nie daje statusu `READY`.
- [x] Brak źródła dla wymaganego slotu daje `BLOCKING`, a nie cichy idle
  fallback.
- [x] Testy obejmują nazwy Meshy, nazwy slotów Aurory, duplikaty, brak tracków
  oraz klip z niezgodnym szkieletem.

---

### E3. Stan sesji, rewizje i zapis projektu

Cel: mapowanie jest częścią projektu, unieważnia się po zmianie źródła i może
zostać wznowione po przeładowaniu aplikacji.

Docelowe integracje:

- `apps/studio-web/src/app/studioSession.ts`
- `apps/studio-web/src/app/studioSelectors.ts`
- nowy `apps/studio-web/src/features/animation-mapping/state.ts`

#### Stan i reducer

- [x] Dodać `animationMapping` do `StudioSessionState`.
- [x] Dodać `sourceRevision` i `authoringRevision` do mapowania.
- [x] `createCreatureAnimationAuthoringV1(...)` — inicjalizacja dokumentu.
- [x] `reduceCreatureAnimationAuthoringV1(state, event)` — czysty reducer.
- [x] `invalidateAnimationMappingAfterSourceChangeV1(...)` — reset zależnych
  wyników bez utraty niezależnych danych projektu.
- [x] `isAnimationMappingCurrentV1(session)` — kontrola związania z bieżącym
  źródłem, wyglądem i inspekcją.
- [x] `canContinueToAnimationMapping(state)` — tylko creature po poprawnej
  inspekcji.
- [x] `canContinueFromAnimationMapping(state)` — brak blockerów i zakończony
  review fallbacków.

#### Zdarzenia sesji

- [x] `ANIMATION_MAPPING_INITIALIZED`
- [x] `ANIMATION_SOURCE_ASSIGNED`
- [x] `ANIMATION_SOURCE_CLEARED`
- [x] `ANIMATION_FALLBACK_APPROVED`
- [x] `ANIMATION_FALLBACK_REJECTED`
- [x] `CUSTOM_ANIMATION_ADDED`
- [x] `CUSTOM_ANIMATION_UPDATED`
- [x] `CUSTOM_ANIMATION_REMOVED`
- [x] `ANIMATION_MAPPING_VALIDATED`
- [x] `CONTINUE_TO_ANIMATION_MAPPING`

#### Persistencja

- [x] `serializeCreatureAnimationAuthoringV1(authoring)` — stabilny JSON.
- [x] `parseCreatureAnimationAuthoringV1(json)` — walidacja wersji i danych.
- [x] `saveCreatureAnimationDraftV1(projectId, authoring)` — autosave.
- [x] `loadCreatureAnimationDraftV1(projectId)` — wznowienie pracy.
- [x] `migrateCreatureAnimationDraftV1(document)` — jawne migracje kolejnych
  wersji.
- [x] Pokazywać `Saving`, `Saved` albo błąd zapisu; nie wyświetlać
  bezwarunkowo napisu `Autosaved`.

#### Definition of Done

- [x] Test reducerów obejmuje każde zdarzenie i unieważnienie downstream.
- [x] Zmiana GLB unieważnia stare przypisania zależne od jego klipów.
- [x] Odświeżenie strony odtwarza zaakceptowane decyzje i animacje custom.
- [x] Uszkodzony lub nowszy dokument nie wywraca aplikacji i daje diagnostykę.

---

### E4. Workflow i szkielet ekranu

Cel: wprowadzić mapowanie jako rzeczywisty krok pipeline’u creature, a nie
osobny, oderwany konfigurator.

Docelowe komponenty:

- `apps/studio-web/src/features/animation-mapping/CreatureAnimationMappingStep.tsx`
- `apps/studio-web/src/features/animation-mapping/AnimationMappingStatusBar.tsx`
- `apps/studio-web/src/features/animation-mapping/CreatureAnimationMappingStep.css`

#### Funkcje workflow

- [x] Dodać `ANIMATION_MAPPING` między `INSPECT` i `BUILD`.
- [x] `getWorkflowStepsForTarget(target)` — krok występuje tylko dla
  `CREATURE`.
- [x] Zaktualizować `compareWorkflowSteps`, aby działało dla targetowego
  workflow, a nie jednej globalnej tablicy.
- [x] Zaktualizować `canNavigateToStep` i `getUnlockedWorkflowSteps`.
- [x] Zaktualizować statusy kroków i nawigację wstecz.
- [x] Zachować dotychczasowy przebieg dla `PLACEABLE` i `TILE`.

#### Układ ekranu

- [x] Pasek postępu i nazwa kroku.
- [x] Lewy panel katalogu z wyszukiwarką i filtrami.
- [x] Centralny panel szczegółów wybranego stanu/slotu.
- [x] Prawy panel podglądu.
- [x] Dolny pasek statusu i przycisk `Continue to Build`.
- [x] Responsywny tryb dla węższego okna bez utraty kolejności tabulatora.
- [x] Stany: loading, empty, ready, validation error i autosave error.

#### Definition of Done

- [x] Krok nie pojawia się dla placeable i tile.
- [x] Creature nie może ominąć obowiązkowego mapowania przez ręczną nawigację.
- [x] Powrót do `INSPECT` nie kasuje mapowania bez zmiany danych wejściowych.
- [x] Podstawowy ekran przechodzi test renderowania i klawiatury.

---

### E5. Edycja mapowania i animacje custom

Cel: użytkownik edytuje wyłącznie decyzje, które rzeczywiście należą do niego.

Docelowe komponenty:

- `AnimationCatalogPanel.tsx`
- `AnimationStateMappingPanel.tsx`
- `AnimationAttentionList.tsx`
- `AnimationSourcePicker.tsx`
- `FallbackReview.tsx`
- `CustomAnimationEditor.tsx`

#### Katalog i wybór

- [x] `selectAnimationCatalogRow(id)` — stan wyboru bez mutowania mapowania.
- [x] `getAnimationAttentionItems(authoring)` — stabilna lista blockerów i
  review.
- [x] Liczniki filtrów odzwierciedlają pełny dokument, a nie tylko aktualny
  wynik wyszukiwania.
- [x] Wybrany wiersz pozostaje widoczny po zmianie statusu, o ile nadal spełnia
  filtr; w przeciwnym razie UI wyjaśnia zmianę.

#### Panel mapowania

- [x] Pokazać read-only: stan/akcję, `MODELTYPE`, wynikowy slot i playback.
- [x] `getAvailableAnimationSourcesV1(slot, inspection)` — wyłącznie
  kompatybilne źródła.
- [x] `assignAnimationSourceV1(authoring, slot, source)` — jedna atomowa
  zmiana.
- [x] `clearAnimationSourceV1(authoring, slot)` — jawny powrót do braku
  przypisania.
- [x] Pokazać provider, asset i ownership jako osobne pola.
- [x] Pokazać przyczynę odrzucenia niekompatybilnego klipu.

#### Fallback

- [x] Wyświetlić źródło fallbacku, docelowy slot i przewidywaną różnicę.
- [x] Wymagać `Accept fallback` lub `Reject`.
- [x] Nie pozwalać zaakceptować fallbacku, który tworzy cykl.
- [x] `detectFallbackCyclesV1(assignments)` — diagnostyka stabilnego kodu.
- [x] `resolveEffectiveAnimationSourceV1(slot, assignments)` — rozwiązanie
  łańcucha z limitem głębokości.

#### Animacje custom

- [x] `createCustomAnimationV1(input)` — nowa definicja z unikalnym ID.
- [x] `validateCustomAnimationNameV1(name)` — format, długość i konflikt.
- [x] `validateCustomAnimationPhasesV1(custom)` — zgodność faz
  `START | LOOP | END`.
- [x] `addCustomAnimationV1(authoring, custom)`.
- [x] `updateCustomAnimationV1(authoring, id, patch)`.
- [x] `removeCustomAnimationV1(authoring, id)` z potwierdzeniem, jeżeli jest
  używana.
- [x] Oddzielić animacje custom od licznika i kompletności `Base 42`.

#### Definition of Done

- [x] Każda akcja użytkownika jest odwracalna albo wymaga potwierdzenia.
- [x] UI nie eksponuje ścieżki systemowej do supermodelu.
- [x] Fallback nie może zniknąć z `Needs attention` bez decyzji użytkownika.
- [x] Czytnik ekranu otrzymuje nazwę stanu, status i przyczynę problemu.

---

### E6. Podgląd i diagnostyka animacji

Cel: użytkownik porównuje źródło z wynikiem pipeline’u i rozumie ograniczenia
podglądu.

Docelowe komponenty:

- `AnimationPreviewPanel.tsx`
- istniejące `features/preview/animationPlayback.ts`
- istniejące `features/preview/readbackAnimationPlayback.ts`
- istniejący `AuroraReadbackViewport.tsx`

#### Funkcje

- [x] `createAnimationPreviewSelectionV1(row, authoring)` — wybór źródła i
  wynikowego klipu.
- [x] `loadSourceAnimationPreviewV1(source, clip)` — podgląd GLB.
- [x] `loadReadbackAnimationPreviewV1(readback, slot)` — podgląd wyniku MDL.
- [x] `compareAnimationPreviewTimingsV1(source, result)` — długość, eventy,
  transtime i odchylenia.
- [x] `getAnimationPreviewDiagnosticsV1(selection)` — brak wyniku, stary
  readback, niezgodny rig i inne ograniczenia.
- [x] Połączyć wybór w katalogu z `AnimationPlaybackRuntime.selectClip`.
- [x] Obsłużyć play/pause, seek, prędkość i krok klatki.
- [x] Politykę pętli ustawiać z domeny stanu; nie zapisywać jej jako
  niepotwierdzonego pola MDL.
- [x] Oznaczać podgląd jako `Source preview` albo `Built readback preview`.
- [x] Po zmianie mapowania oznaczać wcześniejszy wynik builda jako stale.

#### Definition of Done

- [x] Zmiana wiersza zatrzymuje poprzednią akcję i zwalnia zasoby mixera.
- [x] Brak klipu daje czytelny placeholder, a nie pusty viewport.
- [x] UI nie przedstawia preview jako proofu działania w NWN.
- [x] Testy obejmują play, pause, seek, loop policy, zmianę klipu i dispose.

---

### E7. Walidacja, gotowość i dostępność UX

Cel: status ekranu jest jednoznaczny i nie może jednocześnie mówić „gotowe”
oraz pokazywać blocker.

Docelowy moduł:
`apps/studio-web/src/features/animation-mapping/readiness.ts`

#### Funkcje

- [x] `validateCreatureAnimationAuthoringV1(authoring, inspection)` — pełny
  zestaw diagnostyk.
- [x] `getCreatureAnimationMappingStatusV1(diagnostics)` — priorytet
  `BLOCKED > NEEDS_REVIEW > READY`.
- [x] `countAnimationMappingStatusesV1(rows)` — liczniki mapped/review/blocker.
- [x] `canBuildCreatureAnimationPackageV1(status)` — `true` wyłącznie dla
  `READY`.
- [x] `groupAnimationDiagnosticsV1(diagnostics)` — kategorie: source, rig,
  base coverage, fallback, custom, readback.
- [x] `focusFirstBlockingAnimationIssueV1()` — przejście do pierwszego błędu.
- [x] `formatAnimationDiagnosticV1(diagnostic)` — komunikat z działaniem
  naprawczym.

#### Reguły statusu

- [x] `BLOCKED`: brak obowiązkowego źródła, konflikt, stary dokument, zły rig
  albo niepoprawna konfiguracja custom.
- [x] `NEEDS_REVIEW`: kompletne technicznie, ale istnieje niezaakceptowany
  fallback lub niepewna propozycja.
- [x] `READY`: wszystkie wymagane sloty mają rozwiązanie, brak blockerów i
  wszystkie fallbacki zostały jawnie zaakceptowane.
- [x] W licznikach osobno pokazywać mapped, review, blockers i custom.

#### Dostępność i UX

- [x] Pełna obsługa klawiatury w katalogu i formularzach.
- [x] Widoczny focus i logiczna kolejność tabulatora.
- [x] Status nie opiera się wyłącznie na kolorze.
- [x] Błędy są połączone z odpowiednimi polami przez ARIA.
- [x] Zmiana statusu ma nieinwazyjny komunikat live region.
- [x] Potwierdzenia są wymagane tylko przy realnej utracie decyzji.

#### Definition of Done

- [x] Macierz testów statusu nie zawiera kombinacji `READY + BLOCKING`.
- [x] Każdy blocker ma kod, lokalizację i proponowaną akcję.
- [x] Test komponentowy przechodzi podstawowy audyt dostępności.
- [x] Użytkownik może znaleźć i rozwiązać każdy blocker bez opuszczania kroku.

---

### E8. WASM, worker i m2a-core

Cel: decyzje z UI docierają do istniejącego generatora i wracają jako
kanoniczny raport, bez dublowania logiki w workerze.

#### m2a-core

- [x] Dodać serializowalne odpowiedniki typów z E0.
- [x] `validate_creature_animation_authoring_v1(...)` — kanoniczna walidacja.
- [x] `resolve_creature_animation_mapping_v1(...)` — wynikowe przypisania 42
  slotów i custom.
- [x] `materialize_authored_direct_creature_clips_v1(...)` — wykorzystuje
  istniejącą materializację natywnych klipów.
- [x] `evaluate_authored_animation_conformance_v1(...)` — raport gotowości.
- [x] Połączyć event authoring z istniejącym
  `apply_direct_creature_event_authoring_v1`.
- [x] Wykorzystać istniejący
  `author_procedural_humanoid_full_native_42_v1`, gdy źródłem jest generator.
- [x] Nie przywracać cichego idle fallbacku.

#### WASM

- [x] Dodać `validateCreatureAnimationAuthoringV1(...)`.
- [x] Dodać `resolveCreatureAnimationMappingV1(...)`.
- [x] Dodać addytywny `buildMeshyH1ModelPackageV4(...)`, przyjmujący
  wersjonowany dokument authoring.
- [x] Zachować działanie `buildMeshyH1ModelPackageV2/V3`.
- [x] Eksportować kanoniczny katalog lub jego fingerprint do testu parytetu.
- [x] Zwracać diagnostyki jako dane, nie jako nieustrukturyzowany tekst błędu.

#### Worker i klient

- [x] Dodać request `VALIDATE_CREATURE_ANIMATION_MAPPING`.
- [x] Dodać response `CREATURE_ANIMATION_MAPPING_VALIDATED`.
- [x] Dodać wersjonowany `animationAuthoringJson` do nowej ścieżki builda.
- [x] Dodać lane `H1_SKINNED_FULL_42_AUTHORED` albo równoważny, jawnie
  wersjonowany wariant bez zmiany semantyki istniejących lane.
- [x] `validateCreatureAnimationMapping(...)` w kliencie workera.
- [x] `buildAuthoredCreatureModelPackage(...)` w kliencie workera.
- [x] Obsłużyć anulowanie, progress, błąd parsowania i niezgodną wersję.

#### Definition of Done

- [x] Ten sam dokument daje zgodny status w UI i core.
- [x] Worker nie zawiera własnej kopii reguł mapowania.
- [x] Stare requesty i lane nadal przechodzą testy regresji.
- [x] Nowy build emituje pełny raport pokrycia, provenance i readback.

---

### E9. Build, Review i eksport pakietu

Cel: `Continue to Build` używa zaakceptowanego mapowania, a Review pokazuje
dokładnie to, co weszło do artefaktu.

#### Funkcje i integracje

- [x] `createCreatureAnimationBuildInputV1(session)` — zamrożony snapshot
  authoringu i rewizji.
- [x] `assertCreatureAnimationBuildInputCurrentV1(session, input)` — blokada
  starego snapshotu.
- [x] Przekazać dokument do nowej lane workera.
- [x] Zapisać fingerprint mapowania w raporcie i manifest.
- [x] `projectBuiltAnimationCoverageV1(buildResult)` — faktyczny wynik builda,
  nie stan formularza.
- [x] `reconcileAuthoredAndBuiltAnimationsV1(authoring, readback)` — różnice
  expected/actual.
- [x] Rozszerzyć `projectConversionReadiness(...)` o:
  mapowanie, fallback review i pełne pokrycie 42.
- [x] Dodać sekcję `Animation Mapping` do Review.
- [x] Pokazać listę custom oraz ich fazy.
- [x] Zablokować Download przy rozbieżności authoring/readback.
- [x] Do eksportu dołączyć wersjonowany manifest decyzji animacyjnych.

#### Definition of Done

- [x] Build nie startuje dla `BLOCKED` ani `NEEDS_REVIEW`.
- [x] Readback potwierdza exact 42 bazowe klipy profilu.
- [x] Review pokazuje rzeczywiste źródło każdego slotu.
- [x] Zmiana mapowania po buildzie jednoznacznie oznacza wynik jako stale.
- [x] Eksportowane artefakty przechodzą istniejące bramki pakietu.

---

### E10. Weryfikacja offline

Cel: zamknąć implementację aplikacji przed przekazaniem artefaktu do proofu
właściciela.

#### Testy jednostkowe

- [x] katalog 42 i parytet Rust/TS;
- [x] resolver stan -> `MODELTYPE` -> slot;
- [x] ranking i auto-mapowanie;
- [x] fallbacki, cykle i akceptacja;
- [x] walidacja custom i faz;
- [x] status readiness;
- [x] serializacja, migracja i invalidacja rewizji;
- [x] source/readback playback.

#### Testy integracyjne

- [x] Studio session: creature przechodzi przez `ANIMATION_MAPPING`;
- [x] placeable/tile pomijają nowy krok;
- [x] inspekcja GLB -> propozycja -> review -> build input;
- [x] UI -> worker -> WASM -> core;
- [x] authoring -> build -> binary readback -> Review;
- [x] regresja istniejących buildów V2/V3 i lane.

#### Fixture scenarios

- [x] pełne 42 lokalne klipy;
- [x] część lokalna plus jawnie zaakceptowane fallbacki;
- [x] generowany pełny profil 42;
- [x] brak obowiązkowego klipu;
- [x] niezgodny rig;
- [x] duplikaty nazw;
- [x] custom loop bez start/end;
- [x] custom z poprawnymi start/loop/end;
- [x] zakazana kombinacja custom one-shot/end;
- [x] uszkodzony/stary dokument authoring.

#### Jakość i bezpieczeństwo zakresu

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] testy TypeScript/React dla zmienionych pakietów;
- [x] produkcyjny build Studio;
- [x] brak odwołań produktu do `test-assets\meshy`;
- [x] brak nowych trwałych payloadów modelu poza kanonicznym `sample-3d`;
- [x] brak zmian w instalacji Aurora/NWN poza osobno dozwolonym handoffem
  dokładnych artefaktów proof.

#### Definition of Done

- [x] Wszystkie powyższe testy przechodzą na tym samym stanie drzewa.
- [x] Znane ograniczenia są zapisane jako diagnostyki albo backlog, nie ukryte
  w implementacji.
- [x] Dokumentacja użytkowa zgadza się z finalnym UI.
- [x] Audit diff nie wykazuje przypadkowych zmian poza zakresem animacji.

---

### E11. Handoff do proofu właściciela

Cel: przygotować dokładny, nieruchomy kandydat do ręcznego sprawdzenia w Aurora
Toolset i NWN bez przypisywania proofu agentowi.

#### Artefakty i instalacja

- [x] Zamrozić jeden exact candidate i zapisać hashe źródeł oraz wyników.
- [x] Przygotować exact `.mod` i wszystkie wymagane `.hak`.
- [x] Podać jako pierwszą informację dokładną nazwę pliku test-module `.mod`.
- [x] Podać nazwę modułu widoczną w Toolset.
- [x] Podać dokładną nazwę Area.
- [x] Podać obiekt, HAK, wiersz Appearance i placement.
- [x] Przed instalacją rozwiązać i zahashować każde źródło oraz cel.
- [x] Kopiować tylko do nieistniejącego celu; identyczny istniejący plik można
  wyłącznie ponownie użyć.
- [x] Po instalacji potwierdzić byte-for-byte identyczność.
- [x] Przy kolizji z innym hashem zatrzymać handoff bez nadpisywania i bez
  tworzenia nowej iteracji.

#### Granica odpowiedzialności

- [x] Oznaczyć status `ready_for_owner_proof` dopiero po poprawnej instalacji i
  weryfikacji hashy.
- [x] Nie uruchamiać ani nie przejmować Aurora Toolset/NWN.
- [x] Nie deklarować sukcesu wizualnego na podstawie preview lub readback.
- [x] Po wyniku właściciela zapisać osobno:
  `modelVisibility` i `proofCompleteness`.
- [x] Nową iterację dopuścić tylko po świeżym, candidate-bound wyniku
  `modelVisibility=not_visible`.

#### Definition of Done

- [x] Właściciel otrzymał kompletny, jednoznaczny handoff.
- [x] Exact artefakty są zainstalowane i zweryfikowane.
- [x] Status końca pracy agenta to `ready_for_owner_proof`, nie niepotwierdzony
  sukces w Toolset/NWN.

## 6. Proponowana kolejność commitów

- [ ] `animation: add versioned creature animation mapping contract`
- [ ] `animation: add catalog resolver and source clip automapping`
- [ ] `studio: persist creature animation authoring in session`
- [ ] `studio: add creature animation mapping workflow step`
- [ ] `studio: implement mapping fallback and custom animation editor`
- [ ] `studio: integrate source and readback animation preview`
- [ ] `core: validate and materialize authored creature animations`
- [ ] `wasm: expose authored creature animation build v4`
- [ ] `studio: connect animation mapping to build and review`
- [ ] `animation: complete offline verification and owner proof handoff`

Każdy commit powinien zawierać spójny vertical slice, jego testy i aktualizację
tej checklisty. Nie należy zaznaczać pozycji `[x]` w osobnym, późniejszym
commicie bez wskazania dowodu.

## 7. Dziennik postępu

| Data | Etap | Zmiana statusu | Dowód |
|---|---|---|---|
| 2026-07-28 | Kontrakt wiedzy | Udokumentowano rozróżnienie: stan, slot, `ANIMATION_*`, playback i custom | `documentation/aurora-animation-system-codex.md` |
| 2026-07-28 | UX | Przygotowano i skorygowano mockup ekranu mapowania | `documentation/mockups/creature-animation-mapping-v2-2026-07-28/` |
| 2026-07-28 | Plan | Utworzono etapową checklistę implementacji | ten dokument |
| 2026-07-28 | E0/E1 | Dodano wersjonowane typy Rust/TS, współdzielony kontrakt katalogu, resolver, projekcję i filtry; 5 testów Rust + 6 testów Vitest + typecheck przechodzą | `contracts/creature-animation-catalog-v1.json`, `crates/m2a-core/tests/creature_animation_mapping.rs`, `apps/studio-web/src/features/animation-mapping/catalog.test.ts` |
| 2026-07-28 | E2 | Dodano inventory, klasyfikację exact/alias/Meshy, diagnostykę duplikatów i rigu, deterministyczny auto-map oraz jawny review fallbacków; 14 testów Vitest w trzech plikach + typecheck przechodzą | `apps/studio-web/src/features/animation-mapping/sourceClips.ts`, `autoMap.ts`, `fallbacks.ts` |
| 2026-07-28 | E3–E7 | Dodano stan sesji, rewizje, persistencję, krok workflow, edytor, preview, readiness i obsługę klawiatury/ARIA | `apps/studio-web/src/features/animation-mapping/`, `apps/studio-web/src/app/` |
| 2026-07-28 | E8–E9 | Dodano core resolver/materializację/conformance, WASM V4, worker lane oraz report/readback/Review | `crates/m2a-core/src/creature_animation_mapping.rs`, `crates/m2a-wasm/src/lib.rs`, `apps/studio-web/src/worker/` |
| 2026-07-28 | E10 | 243 testy Studio, build, clippy, 10/10 testów real Worker/WASM integration oraz pełne `cargo test --workspace` przechodzą; testy w worktree czytają Git-ignored dane wyłącznie z canonical root bez kopiowania payloadów | `documentation/raport-implementacji-creature-animation-mapping-2026-07-28.md` |
| 2026-07-28 | E11 | Dodano V4 z jawną immutable identity, naprawiono zachowanie proceduralnego `cpause1` i potwierdzono, że authored V4 odtwarza byte-identical MDL/TGA/2DA/HAK/MOD r46; ponownie użyto istniejącej, hash-verified instalacji bez r47 | `documentation/evidence/creature-animation-mapping-v4-r46-ready-for-owner-proof-2026-07-28.md` |

## 8. Dokumenty powiązane

- [`aurora-animation-system-codex.md`](aurora-animation-system-codex.md) —
  kontrakt techniczny i pochodzenie wiedzy;
- [`animacje-kontrakt-profil-a-codex.md`](animacje-kontrakt-profil-a-codex.md) —
  profil produktu i reguły animacji;
- [`macierz-gotowosci-wiedzy-codex.md`](macierz-gotowosci-wiedzy-codex.md) —
  gotowość wiedzy i otwarte dowody runtime;
- [`mockups/creature-animation-mapping-v2-2026-07-28/README.md`](mockups/creature-animation-mapping-v2-2026-07-28/README.md) —
  decyzje UX pokazane na mockupie;
- [`raport-implementacji-creature-animation-mapping-2026-07-28.md`](raport-implementacji-creature-animation-mapping-2026-07-28.md) —
  zakres implementacji, pełne dowody offline i handoff E11;
- [`PROJECT_RULES.md`](PROJECT_RULES.md) i
  [`CANONICAL_WORKSPACE.md`](CANONICAL_WORKSPACE.md) — nadrzędne reguły
  workspace, proofu i artefaktów.
