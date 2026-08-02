# Plan implementacji: Animation Studio i wybór animacji Custom

Data: 2026-07-28
Branch: `animation`
Baseline: `8bd5e0d` (`feat: implement creature animation mapping`)
Status: `F1-F10 UKOŃCZONE / F11 ZABLOKOWANE BRAMĄ ITERACJI MODELU`

Aktualizacja 2026-07-30: wdrożono fazę F13 — poprawki po audycie rozwiązań
animacji. Arbitrary Custom jest teraz jawnie `LIBRARY_ONLY`, dopóki nie zostanie
związany z Base 42; attack demo zachowuje `cpause1` i routuje ten sam Custom do
`ca1slashl`, `ca1slashr`, `ca1stab`; binary readback i owner runtime proof są
osobnymi osiami; Meshy Bridge zachowuje 1–10 osobnych animacji i przekazuje je
do Studio jako zweryfikowanych donorów; UX inspektora, timeline i Save validation
został domknięty. Pełny raport:
[`audyt-poprawki-rozwiazan-animacji-2026-07-30.md`](audyt-poprawki-rozwiazan-animacji-2026-07-30.md).

Aktualizacja 2026-07-28: implementacja offline została ukończona i
zweryfikowana. F11 nie może otrzymać statusu `ready_for_owner_proof`, ponieważ
aktywny dokładny kandydat r46 ma właścicielski wynik `visible`, a projekt nie
ma świeżego, związanego z kandydatem wyniku `modelVisibility=not_visible`,
który dopuszczałby nową iterację V5. Nie utworzono ani nie zainstalowano nowego
MOD-a/HAK-a. Szczegóły i warunek wznowienia:
[`animation-studio-v5-owner-proof-gate-2026-07-28.md`](evidence/animation-studio-v5-owner-proof-gate-2026-07-28.md).

Interfejsy referencyjne:

- [Animation Studio wewnątrz kroku Animation Mapping](mockups/animation-studio-v1-2026-07-28/02-animation-studio-within-mapping.png)
- [Wybór zapisanej animacji Custom dla slotu Base 42](mockups/animation-studio-v1-2026-07-28/03-custom-animation-picker.png)
- [Kopiowanie klipu ze zgodnego modelu](mockups/animation-studio-v1-2026-07-28/04-copy-animation-from-model-compatible.png)
- [Blokada importu z innym rigiem](mockups/animation-studio-v1-2026-07-28/05-copy-animation-rig-mismatch.png)
- [Bazowy ekran mapowania 42 slotów](mockups/creature-animation-mapping-v2-2026-07-28/02-aurora-state-mapping-ux-corrected.png)

Pierwszy mockup `01-animation-studio-create-edit.png` pozostaje zapisem
odrzuconego wariantu IA. Osobny krok `Create & Edit` jest błędny i nie może
zostać zaimplementowany.

## 1. Cel

Dodać do istniejącego kroku `Animation Mapping` drugi tryb pracy:

`Map animations | Create & edit (Beta)`

Użytkownik ma móc:

1. utworzyć nową animację z pozy bazowej;
2. utworzyć edytowalną kopię klipu źródłowego;
3. edytować rotację i translację kości w klatkach kluczowych;
4. przycinać i przeskalowywać czas animacji;
5. dodawać eventy animacji;
6. zapisać animację jako trwały, lokalny klip projektu;
7. zobaczyć ją automatycznie na liście `Custom`;
8. przypisać poprawny klip `Custom` do slotu Base 42;
9. wrócić z mapowania do edytora bez utraty decyzji;
10. zbudować pakiet tylko po walidacji i binarnym readbacku.

Funkcja jest lokalnym, niedestrukcyjnym edytorem. Meshy API pozostaje źródłem
gotowych presetów, a nie usługą tworzenia własnych keyframe'ów.

## 2. Jak aktualizować checklisty

- `[ ]` — funkcja niezaimplementowana;
- `[-]` — istnieje część fundamentu, ale funkcja użytkowa nie jest ukończona;
- `[x]` — funkcja ukończona, ma testy i wymagany readback albo dowód;
- etap jest ukończony dopiero po zaznaczeniu wszystkich pozycji jego
  `Definition of Done`;
- samo istnienie typu, przycisku albo mockupu nie wystarcza do `[x]`;
- przy każdej zmianie statusu trzeba dopisać datę oraz link do testu, raportu
  albo commitu w sekcji `Dziennik postępu`.

GitHub natywnie obsługuje `[ ]` i `[x]`. `[-]` jest tekstowym statusem
częściowym i nie może być uznany za wykonanie.

## 3. Niezmienne decyzje produktowe

- [x] `Create & edit` nie jest osobnym krokiem workflow.
- [x] Aktywnym krokiem pozostaje `3 Animation Mapping`.
- [x] Tryb `Map animations` służy do realizacji 42 bazowych slotów.
- [x] Tryb `Create & edit` służy do lokalnego authoringu klipów.
- [x] Źródłowy GLB pozostaje niezmieniony.
- [x] Liczba i znaczenie 42 bazowych slotów `S/L` pozostają niezmienne.
- [x] Animacje Custom są dodatkowymi klipami i nie zwiększają licznika
  `Base 42`.
- [x] Przypisanie klipu Custom do Base 42 nie usuwa go z biblioteki Custom.
- [x] Klip `Draft` jest widoczny w bibliotece, ale nie można go przypisać ani
  zbudować.
- [x] Tylko klip `Valid` może zostać przypisany do Base 42.
- [x] Mapowania używają stabilnych ID, a nie ścieżek plików.
- [x] Aktualne API i build V4 pozostają kompatybilne i nie zmieniają znaczenia.
- [x] Nowa funkcja otrzyma addytywny kontrakt i osobną ścieżkę builda.
- [x] Agent nie wykonuje końcowego proofu wizualnego Toolset/NWN; granicą jest
  `ready_for_owner_proof`.

## 4. Zakres MVP

### 4.1 W zakresie

- tworzenie klipu z bieżącej pozy szkieletu;
- edytowalna kopia istniejącego klipu;
- rotacja kości jako quaternion;
- translacja kości;
- interpolacja `LINEAR`;
- dodawanie, przesuwanie i usuwanie keyframe'ów;
- zmiana długości i `transtime`;
- trim i retime;
- eventy z czasem i nazwą;
- timeline/dope sheet;
- playback, seek i krok po keyframe'ach;
- undo/redo;
- autosave projektu;
- statusy `Draft | Valid | Invalid`;
- biblioteka Custom;
- przypisanie Custom do Base 42;
- build, binary readback, manifest i Review.

### 4.2 Poza MVP

- Jednolita skala istnieje w części niskopoziomowego writera, ale nie
  będzie edytowalną ścieżką pierwszego MVP.
- `STEP` i `CUBICSPLINE` nie będą emitowane; wejście musi zostać
  zlinearyzowane albo odrzucone.
- IK, constraints i animation layers są poza MVP.
- Retargeting pomiędzy różnymi szkieletami jest poza MVP.
- Edycja wag skina i morph weights jest poza MVP.
- Graph editor i krzywe Bezier są poza MVP.
- Facial animation i lip sync są poza MVP.
- Nadpisywanie źródłowego GLB jest zabronione.
- Kopiowanie retail/CEP keyframe'ów, eventów albo szkieletów jest
  zabronione.
- Meshy API nie jest używane jako edytor własnego ruchu.

## 5. Stan wyjściowy brancha

Ta sekcja jest historycznym snapshotem baseline `8bd5e0d`, zachowanym dla
porównania. Nie opisuje aktualnego stanu po implementacji; bieżący status
znajduje się w sekcjach 8-11 i w raporcie implementacji.

### 5.1 Fundament ukończony

- [x] Istnieje katalog 42 bazowych slotów `S/L`.
- [x] Istnieje `CreatureAnimationAuthoringV1`.
- [x] Istnieje `CustomAnimationDefinitionV1` dla klipu źródłowego albo faz
  `START | LOOP | END`.
- [x] Istnieją stabilne operacje add/update/remove dla definicji Custom.
- [x] Core rozwiązuje `sourceKind=CUSTOM` i `customAnimationId`.
- [x] Core materializuje definicje Custom przez klonowanie wskazanych klipów
  źródłowych.
- [x] Istnieje trwały zapis i odczyt mappingu V1.
- [x] Istnieje build lane `H1_SKINNED_FULL_42_AUTHORED`.
- [x] Istnieje WASM `buildMeshyH1ModelPackageV4`.
- [x] Istnieje ogólny `writeBinaryMdlWithAnimations`.
- [x] `MdlAnimationClipV1` przechowuje długość, `transtime`, `animroot`,
  eventy i tracki.
- [x] `MdlAnimationTrackV1` przechowuje target, path, interpolację, czasy i
  wartości.
- [x] Writer waliduje finite values, arność, zakres czasu i ściśle rosnące
  czasy tracku.
- [x] Writer normalizuje i kanonizuje quaterniony.
- [x] Preview obsługuje play/pause, seek oraz poprzedni/następny keyframe.
- [x] Istnieje core event authoring i opcjonalny JSON sidecar w Studio.
- [x] Istnieje authored V4 -> binary readback -> Review.

### 5.2 Częściowe

- [-] UI tworzy definicję Custom, ale tylko z istniejących nazw klipów GLB.
- [-] UI potrafi zmienić nazwę definicji Custom, ale nie jej ruch.
- [-] Timeline odtwarza i przeskakuje po keyframe'ach, ale nie edytuje ich.
- [-] Core potrafi zapisać ręcznie skonstruowany animation-set, ale nie ma
  wersjonowanego dokumentu operacji edytora.
- [-] Reducer potrafi przypisać `sourceKind=CUSTOM`, ale obecny
  `AnimationSourcePicker` wystawia wyłącznie `SOURCE_CLIP`.
- [-] Eventy są wspierane przez core i plik JSON, ale nie mają edytora w UI.

### 5.3 Brakujące

- [ ] Brak trwałego `AuthoredAnimationClipV1`.
- [ ] Brak kolekcji `authoredClips`.
- [ ] Brak edytora kości i keyframe'ów.
- [ ] Brak biblioteki zapisanych klipów z `Draft | Valid | Invalid`.
- [ ] Brak selektora Custom pokazanego w mockupie 03.
- [ ] Brak powrotu `Open selected in editor`.
- [ ] Brak projekcji edytowanego klipu do builda V5.
- [ ] Brak zintegrowanych eventów edytowanego klipu.
- [ ] Brak readback reconciliation dla dokumentu edytora.

## 6. Docelowy model danych

### 6.1 Rozdzielenie odpowiedzialności

Nie wolno wkładać surowych keyframe'ów do istniejącego
`CustomAnimationDefinitionV1`.

Docelowe byty:

```text
AnimationStudioDocumentV1
  └─ authoredClips: AuthoredAnimationClipV1[]
       ├─ tracks: AuthoredAnimationTrackV1[]
       │    └─ keyframes: AnimationKeyframeV1[]
       └─ events: AuthoredAnimationEventV1[]

CreatureAnimationAuthoringV2
  ├─ assignments: Base 42 -> źródło
  └─ customAnimations: CustomAnimationDefinitionV2[]
       └─ ONE_SHOT albo START / LOOP / END -> authoredClipId
```

`AuthoredAnimationClipV1` odpowiada za dane ruchu.
`CustomAnimationDefinitionV2` odpowiada za nazwę wynikową, playback i routing.
`AnimationSourceAssignmentV2` nadal wskazuje stabilne `customAnimationId`.

### 6.2 Planowane typy TypeScript

- [x] `AnimationStudioDocumentV1`
- [x] `AnimationStudioDocumentStatusV1`
- [x] `AuthoredAnimationClipV1`
- [x] `AuthoredAnimationClipKindV1 = "MOTION" | "STATIC_POSE"`
- [x] `AuthoredAnimationClipStatusV1 = "DRAFT" | "VALID" | "INVALID"`
- [x] `AuthoredAnimationSourceV1`
- [x] `AuthoredAnimationSourceKindV1 =
  "BLANK_POSE" | "SOURCE_CLIP_COPY" | "PROCEDURAL_TEMPLATE"`
- [x] `AuthoredAnimationTrackV1`
- [x] `AuthoredAnimationTrackPathV1 = "TRANSLATION" | "ROTATION"`
- [x] `AnimationKeyframeV1`
- [x] `AuthoredAnimationEventV1`
- [x] `CustomAnimationClipReferenceV2`
- [x] `CustomAnimationPhaseV2`
- [x] `CustomAnimationDefinitionV2`
- [x] `AnimationSourceAssignmentV2`
- [x] `CreatureAnimationAuthoringV2`
- [x] `CustomAnimationLibraryItemV1`
- [x] `AnimationStudioDiagnosticV1`
- [x] `AnimationStudioReadbackV1`

### 6.3 Minimalny kontrakt klipu

Planowana postać logiczna:

```json
{
  "id": "authored_attack_01",
  "name": "custom_attack_overhead",
  "kind": "MOTION",
  "status": "DRAFT",
  "source": {
    "kind": "SOURCE_CLIP_COPY",
    "sourceRevision": "<sha256>",
    "sourceClipName": "Attack"
  },
  "lengthSeconds": 0.9,
  "transitionSeconds": 0.1,
  "animationRoot": "torso",
  "tracks": [],
  "events": [],
  "revision": 1
}
```

To jest przykład projektowy, nie ukończony kontrakt wire. Dokładny JSON zostanie
zamrożony testem parytetu Rust/TypeScript w fazie F1.

### 6.4 Tożsamość i provenance

- [x] `id` jest stabilne i nie zależy od nazwy wynikowej.
- [x] Zmiana `name` nie zrywa mapowania.
- [x] `sourceRevision` jest SHA-256 dokładnego GLB.
- [x] Kopia klipu zapisuje nazwę i fingerprint źródłowego klipu.
- [x] Dokument nie zapisuje absolutnych ścieżek.
- [x] Dokument zapisuje `authoringRevision`.
- [x] Każdy build zapisuje fingerprint dokumentu edytora.
- [x] Manifest rozdziela `source`, `authored`, `generated` i `mapped`.
- [x] Zmiana źródłowego GLB unieważnia zależne drafty do czasu reconciliation.

## 7. Przepływ danych

```text
exact source GLB
  -> own GLB inspection
  -> source animation inventory
  -> Create & edit
  -> AnimationStudioDocumentV1
  -> core validation
  -> materialized MdlAnimationSetV1
  -> CustomAnimationDefinitionV2
  -> Base 42 assignment
  -> build V5
  -> own binary MDL readback
  -> authored/readback reconciliation
  -> Review
  -> Download
```

Reguła fail-closed:

```text
Draft | Invalid | stale source | readback mismatch
  -> nie można przypisać albo zbudować
```

## 8. Stan całości

- [x] F0 — audyt, decyzje UX i mockupy
- [x] F1 — kontrakty danych V1/V2 i migracja
- [x] F2 — kanoniczne operacje edycji w `m2a-core`
- [x] F3 — WASM i worker
- [x] F4 — sesja, autosave i undo/redo
- [x] F5 — tryb `Create & edit` i biblioteka klipów
- [x] F6 — viewport, kości i dope sheet
- [x] F7 — eventy, trim i retime
- [x] F8 — selektor Custom i przypisanie Base 42
- [x] F9 — build V5, readback, Review i manifest
- [x] F10 — testy, dostępność i wydajność
- [-] F11 — handoff `ready_for_owner_proof` (HARD STOP: model iteration gate)

## 9. Fazy implementacji

### F0. Audyt, decyzje UX i mockupy

Cel: zamknąć różnicę pomiędzy obecnym „Custom mapping” a prawdziwym edytorem
ruchu.

#### Funkcje i rezultaty

- [x] Potwierdzono, że obecny `CustomAnimationDefinitionV1` wskazuje istniejące
  klipy i nie zawiera keyframe'ów.
- [x] Potwierdzono, że obecny picker mapuje tylko `SOURCE_CLIP`.
- [x] Potwierdzono możliwość emisji tracków przez core writer.
- [x] Potwierdzono dostępność playback, seek i step-keyframe.
- [x] Potwierdzono, że Meshy API wybiera preset `action_id`, a nie edytuje ruch.
- [x] Przygotowano mockup edytora.
- [x] Odrzucono osobny workflow step.
- [x] Przygotowano poprawny sub-mode `Map animations | Create & edit`.
- [x] Przygotowano mockup selektora Custom.
- [x] Ustalono automatyczne dodanie zapisanego klipu do biblioteki Custom.

#### Definition of Done

- [x] Dokument rozróżnia fakty repo, funkcje częściowe i funkcje brakujące.
- [x] Mockupy nie zmieniają pięcioetapowego workflow.
- [x] V4 pozostaje bazą kompatybilności.

### F1. Kontrakty danych i migracja

Cel: zamrozić addytywne schematy zanim powstanie UI edytora.

#### Rust

- [x] Dodać `AnimationStudioDocumentV1`.
- [x] Dodać `AuthoredAnimationClipV1`.
- [x] Dodać `AuthoredAnimationTrackV1`.
- [x] Dodać `AnimationKeyframeV1`.
- [x] Dodać `AuthoredAnimationEventV1`.
- [x] Dodać `CustomAnimationDefinitionV2`.
- [x] Dodać `CreatureAnimationAuthoringV2`.
- [x] Dodać `AnimationStudioDiagnosticV1`.
- [x] Dodać `AnimationStudioReadbackV1`.

#### TypeScript

- [x] Odwzorować dokładnie typy Rust.
- [x] Wygenerować albo współdzielić katalog enumów bez ręcznej rozbieżności.
- [x] Dodać strict parser dokumentu Studio.
- [x] Dodać stable serializer.

#### Funkcje

- [x] `serializeAnimationStudioDocumentV1(document)`
- [x] `parseAnimationStudioDocumentV1(json)`
- [x] `fingerprintAnimationStudioDocumentV1(document)`
- [x] `migrateCreatureAnimationAuthoringV1ToV2(v1)`
- [x] `validateAnimationStudioSchemaV1(document)`
- [x] `validateCreatureAnimationAuthoringV2(authoring, studio)`

#### Reguły

- [x] Nie dodawać pól do strict V1.
- [x] Migracja V1 -> V2 jest deterministyczna i bezstratna.
- [x] Odczyt V1 nie zapisuje automatycznie V2 bez jawnej zmiany użytkownika.
- [x] Nie używać nazwy klipu jako klucza relacyjnego.
- [x] Nie przechowywać `THREE.AnimationClip` jako formatu projektu.
- [x] Nie przechowywać binarnego GLB w dokumencie edytora.

#### Testy

- [x] Rust serde round-trip.
- [x] TypeScript parse/serialize round-trip.
- [x] Rust/TypeScript fixture parity.
- [x] Unknown field fail-closed.
- [x] V1 -> V2 migration golden.
- [x] Stabilny fingerprint niezależny od kolejności map.
- [x] Duplicate ID i duplicate output name są blokowane.

#### Definition of Done

- [x] Obie warstwy akceptują ten sam zestaw fixture.
- [x] Migracja nie zmienia istniejącego mappingu 42 slotów.
- [x] Dokument ma jawny version gate.

### F2. Kanoniczne operacje edycji w `m2a-core`

Cel: każda zmiana ruchu ma jedną implementację referencyjną w Rust.

#### Tworzenie klipu

- [x] `create_blank_pose_clip_v1(rig, input)`
- [x] `clone_source_clip_for_editing_v1(source, clip_name, input)`
- [x] `create_procedural_template_clip_v1(rig, template, input)`
- [x] `rename_authored_clip_v1(document, clip_id, name)`
- [x] `duplicate_authored_clip_v1(document, clip_id, new_name)`
- [x] `remove_authored_clip_v1(document, clip_id, policy)`

#### Tracki i keyframe'y

- [x] `add_animation_track_v1(clip, target_node_id, path)`
- [x] `remove_animation_track_v1(clip, target_node_id, path)`
- [x] `insert_animation_keyframe_v1(track, time, value)`
- [x] `update_animation_keyframe_v1(track, key_id, patch)`
- [x] `move_animation_keyframe_v1(track, key_id, time)`
- [x] `remove_animation_keyframe_v1(track, key_id)`
- [x] `sample_animation_track_linear_v1(track, time)`
- [x] `normalize_animation_quaternions_v1(track)`
- [x] `sort_and_deduplicate_keyframes_v1(track, policy)`
- [x] `detect_authored_motion_v1(clip)`

#### Czas

- [x] `trim_authored_animation_clip_v1(clip, start, end)`
- [x] `retime_authored_animation_clip_v1(clip, new_length)`
- [x] `shift_authored_animation_keys_v1(clip, delta)`
- [x] `clamp_authored_animation_keys_v1(clip)`
- [x] `set_authored_animation_transition_v1(clip, transition)`

#### Eventy

- [x] `add_authored_animation_event_v1(clip, event)`
- [x] `update_authored_animation_event_v1(clip, event_id, patch)`
- [x] `move_authored_animation_event_v1(clip, event_id, time)`
- [x] `remove_authored_animation_event_v1(clip, event_id)`
- [x] `sort_authored_animation_events_stable_v1(clip)`

#### Walidacja

- [x] `validate_authored_animation_clip_v1(clip, rig)`
- [x] `validate_authored_track_target_v1(track, rig)`
- [x] `validate_authored_track_times_v1(track, clip_length)`
- [x] `validate_authored_track_values_v1(track)`
- [x] `validate_authored_event_v1(event, clip_length)`
- [x] `evaluate_authored_clip_status_v1(clip, rig)`
- [x] `materialize_animation_studio_document_v1(document, rig)`

#### Stabilne diagnostyki

- [x] `M2A-ANIMATION-EDIT-SCHEMA`
- [x] `M2A-ANIMATION-EDIT-SOURCE-STALE`
- [x] `M2A-ANIMATION-EDIT-CLIP-NAME`
- [x] `M2A-ANIMATION-EDIT-CLIP-DUPLICATE`
- [x] `M2A-ANIMATION-EDIT-BONE-MISSING`
- [x] `M2A-ANIMATION-EDIT-PATH-UNSUPPORTED`
- [x] `M2A-ANIMATION-EDIT-TIME-NOT-STRICT`
- [x] `M2A-ANIMATION-EDIT-TIME-OOB`
- [x] `M2A-ANIMATION-EDIT-VALUE-NONFINITE`
- [x] `M2A-ANIMATION-EDIT-QUATERNION`
- [x] `M2A-ANIMATION-EDIT-NO-MOTION`
- [x] `M2A-ANIMATION-EDIT-EVENT`
- [x] `M2A-ANIMATION-EDIT-ROW-LIMIT`

#### Testy

- [x] Blank pose fixture.
- [x] Source clip copy fixture.
- [x] Translation insert/update/delete.
- [x] Rotation insert/update/delete.
- [x] Quaternion normalization i sign canonicalization.
- [x] Trim na granicach keyframe'ów.
- [x] Trim pomiędzy keyframe'ami z próbkowaniem liniowym.
- [x] Retime zachowuje względne pozycje kluczy i eventów.
- [x] Equal time fail-closed albo jawna replace policy.
- [x] Nonfinite i out-of-range fail-closed.
- [x] Foreign bone fail-closed.
- [x] `MOTION` wymaga jawnie zmieniającego się tracku.
- [x] `STATIC_POSE` wymaga jawnej klasyfikacji i poprawnej pozy.
- [x] Deterministyczny wynik dla identycznego dokumentu.

#### Definition of Done

- [x] UI nie implementuje własnej, rozbieżnej matematyki authoringu.
- [x] Materializacja daje poprawny `MdlAnimationSetV1`.
- [x] Wszystkie błędy mają stabilny code/path/message/action.

### F3. WASM i worker

Cel: udostępnić core przeglądarce bez duplikowania walidacji.

#### WASM

- [x] `inspectEditableAnimationSourceV1(sourceGlb, clipName)`
- [x] `validateAnimationStudioDocumentV1(documentJson, sourceGlb)`
- [x] `materializeAnimationStudioDocumentV1(documentJson, sourceGlb)`
- [x] `previewAuthoredAnimationClipV1(documentJson, clipId)`
- [x] `buildMeshyH1ModelPackageV5(...)`

#### Worker

- [x] Dodać lane `H1_SKINNED_FULL_42_EDITED`.
- [x] Dodać request z `animationStudioDocumentJson`.
- [x] Dodać `CreatureAnimationAuthoringV2`.
- [x] Dodać cancel/supersede zgodny z aktualną ochroną build race.
- [x] Nie przenosić źródłowego GLB do głównego wątku więcej razy niż potrzeba.
- [x] Transferować binarne wyniki przez transferable objects.

#### Klient

- [x] `inspectEditableAnimationSource(...)`
- [x] `validateAnimationStudioDocument(...)`
- [x] `buildEditedCreatureModelPackage(...)`
- [x] `cancelAnimationStudioPreviewBuild(...)`

#### Testy

- [x] Native core/WASM parity.
- [x] Malformed JSON daje stabilny błąd.
- [x] Worker materializuje exact keyframe values.
- [x] Superseded preview nie nadpisuje nowszego.
- [x] Build V4 pozostaje byte-identical dla dotychczasowej fixture.
- [x] V5 bez `authoredClips` jest semantycznie zgodne z V4.

#### Definition of Done

- [x] Worker nie zawiera logiki matematycznej edytora.
- [x] V4 nie zostało zmienione.
- [x] V5 ma pełny report i readback.

### F4. Sesja, autosave i undo/redo

Cel: edycja ma być odporna na odświeżenie strony i bezpieczna dla dużych
dokumentów.

`CreatureAnimationAuthoringV1` może pozostać w obecnym storage. Keyframe'y nie
powinny być dokładane do tego samego wpisu `localStorage`. Dokument Studio
powinien korzystać z IndexedDB albo równoważnego magazynu dużych danych.

#### Store

- [x] `openAnimationStudioDatabaseV1()`
- [x] `saveAnimationStudioDocumentV1(projectId, document)`
- [x] `loadAnimationStudioDocumentV1(projectId)`
- [x] `deleteAnimationStudioDocumentV1(projectId, confirmed)`
- [x] `migrateAnimationStudioStorageV1(database)`
- [x] `listAnimationStudioRecoveryRecordsV1()`

#### Stan i reducer

- [x] `createAnimationStudioStateV1(document)`
- [x] `reduceAnimationStudioStateV1(state, event)`
- [x] `commitAnimationStudioEditV1(state, command)`
- [x] `undoAnimationStudioEditV1(state)`
- [x] `redoAnimationStudioEditV1(state)`
- [x] `markAnimationStudioBuildRevisionV1(state, revision)`
- [x] `isAnimationStudioBuildCurrentV1(state)`

#### Eventy sesji

- [x] `ANIMATION_STUDIO_MODE_SELECTED`
- [x] `AUTHORED_CLIP_CREATED`
- [x] `AUTHORED_CLIP_UPDATED`
- [x] `AUTHORED_CLIP_REMOVED`
- [x] `AUTHORED_CLIP_SELECTED`
- [x] `BONE_SELECTED`
- [x] `KEYFRAME_INSERTED`
- [x] `KEYFRAME_UPDATED`
- [x] `KEYFRAME_REMOVED`
- [x] `EVENT_INSERTED`
- [x] `EVENT_UPDATED`
- [x] `EVENT_REMOVED`
- [x] `ANIMATION_STUDIO_UNDO`
- [x] `ANIMATION_STUDIO_REDO`

#### Reguły

- [x] Jedna logiczna operacja zwiększa rewizję dokładnie raz.
- [x] Drag gizma nie tworzy setek wpisów undo.
- [x] Autosave zapisuje tylko zatwierdzony commit edycji.
- [x] Crash recovery nie oznacza automatycznego `Valid`.
- [x] Zmiana source revision oznacza `STALE_SOURCE`.
- [x] Usunięcie klipu używanego przez mapping wymaga potwierdzenia.

#### Testy

- [x] Refresh odtwarza klipy i wybrany tryb.
- [x] Undo/redo przywraca exact JSON.
- [x] IndexedDB failure pokazuje blocker i nie udaje autosave.
- [x] Stary schema version nie jest cicho nadpisywany.
- [x] Usunięcie używanego klipu czyści mapping tylko po potwierdzeniu.

#### Definition of Done

- [x] Odświeżenie strony nie traci zatwierdzonych klipów.
- [x] Użytkownik widzi `Autosaved`, `Saving` albo błąd zapisu.
- [x] Build korzysta z dokładnej zapisanej rewizji.

### F5. Tryb `Create & edit` i biblioteka klipów

Cel: wdrożyć hierarchię z mockupu 02 bez tworzenia nowego workflow step.

#### Komponenty

- [x] `AnimationMappingModeSwitch.tsx`
- [x] `AnimationStudioWorkspace.tsx`
- [x] `AnimationClipLibrary.tsx`
- [x] `AnimationClipLibraryRow.tsx`
- [x] `NewAnimationMenu.tsx`
- [x] `AnimationClipStatusBadge.tsx`
- [x] `AnimationStudioAutosaveStatus.tsx`

#### Funkcje projekcji

- [x] `projectAnimationStudioLibraryV1(document, sourceInventory)`
- [x] `filterAnimationStudioLibraryV1(items, query, filter)`
- [x] `sortAnimationStudioLibraryV1(items)`
- [x] `getAnimationStudioClipActionsV1(item, usage)`
- [x] `getAuthoredClipUsageV1(clipId, authoring)`

#### UX

- [x] Przełącznik `Map animations | Create & edit`.
- [x] `Create & edit` ma badge `Beta`.
- [x] `+ New animation`.
- [x] `Edit copy`.
- [x] `Duplicate`.
- [x] Zakładki `Base 42 | Custom`.
- [x] Status `Source | Edited | Generated`.
- [x] Status `Draft | Valid | Invalid`.
- [x] Informacja `Source GLB stays unchanged`.
- [x] Informacja o użyciu przez Base 42.
- [x] Potwierdzenie usunięcia używanego klipu.

#### Testy

- [x] Tryb nie zmienia aktywnego workflow step.
- [x] Deep-link/refresh wraca do poprawnego sub-mode.
- [x] Klip po Save pojawia się bez ponownego importu.
- [x] Draft jest widoczny.
- [x] Invalid pokazuje pierwszą akcję naprawczą.
- [x] Rename nie zmienia stabilnego ID.

#### Definition of Done

- [x] UI zgadza się z mockupem 02.
- [x] Nie istnieje szósty krok workflow.
- [x] Biblioteka jest projekcją source/generated/authored, nie drugim rootem
  assetów.

### F6. Viewport, kości i dope sheet

Cel: umożliwić bezpośrednią edycję transformacji w czasie.

#### Komponenty

- [x] `AnimationEditorViewport.tsx`
- [x] `AnimationBoneTree.tsx`
- [x] `BoneTransformInspector.tsx`
- [x] `AnimationDopeSheet.tsx`
- [x] `AnimationTrackRow.tsx`
- [x] `AnimationKeyframeMarker.tsx`
- [x] `AnimationTransportControls.tsx`
- [x] `AnimationTimelineRuler.tsx`
- [x] `AnimationEditorDiagnostics.tsx`

#### Runtime/projekcja

- [x] `projectAuthoredClipToThreeV1(clip, rig)`
- [x] `applyEditorPoseAtTimeV1(root, clip, time)`
- [x] `collectAnimationKeyTimesV1(clip)`
- [x] `selectAnimationEditorBoneV1(nodeId)`
- [x] `beginBoneTransformGestureV1(input)`
- [x] `updateBoneTransformGestureV1(input)`
- [x] `commitBoneTransformGestureV1(input)`
- [x] `cancelBoneTransformGestureV1()`

#### Dope sheet

- [x] `projectDopeSheetRowsV1(clip, rig)`
- [x] `insertKeyAtPlayheadV1(clip, selection, time)`
- [x] `moveSelectedKeysV1(clip, keyIds, delta)`
- [x] `deleteSelectedKeysV1(clip, keyIds)`
- [x] `selectKeysInRangeV1(clip, range)`
- [x] `snapAnimationTimeV1(time, snapPolicy)`
- [x] `zoomAnimationTimelineV1(view, delta)`
- [x] `panAnimationTimelineV1(view, delta)`

#### Reguły

- [x] Edycja odbywa się w local space kości output rig.
- [x] Gizmo jest wyłączone dla nieobsługiwanego path.
- [x] Rotation zapisuje quaternion, nie Euler.
- [x] Pola Euler w inspectorze są wyłącznie kontrolką prezentacyjną.
- [x] Commit normalizuje quaternion.
- [x] Jeden playhead jest źródłem czasu dla viewportu i dope sheet.
- [x] Snap `1/30 s` jest opcją UX, nie polem formatu MDL.
- [x] Loop w preview nie jest zapisywany jako niepotwierdzone pole MDL.

#### Dostępność

- [x] Wszystkie operacje gizma mają równoważne pola liczbowe.
- [x] Keyframe można dodać/usunąć z klawiatury.
- [x] Timeline ma czytelny focus.
- [x] Kolor nie jest jedynym nośnikiem statusu.
- [x] Screen reader dostaje czas, kość, path i status klucza.

#### Testy

- [x] Viewport i dope sheet pokazują ten sam czas.
- [x] Previous/next keyframe działa po wszystkich trackach.
- [x] Drag + commit tworzy jeden wpis undo.
- [x] Cancel gesture nie zmienia dokumentu.
- [x] Quaternion z UI po round-trip jest kanoniczny.
- [x] Pola numeryczne działają bez myszy.

#### Definition of Done

- [x] Można utworzyć ruch bez ręcznej edycji JSON.
- [x] Każda zmiana UI daje ten sam wynik po core validation.
- [x] Preview nie jest źródłem danych builda; źródłem jest dokument.

### F7. Eventy, trim i retime

Cel: zakończyć podstawowy authoring czasu i callbacków.

#### Komponenty

- [x] `AnimationEventTrack.tsx`
- [x] `AnimationEventEditor.tsx`
- [x] `AnimationTrimDialog.tsx`
- [x] `AnimationRetimeDialog.tsx`

#### Funkcje UI

- [x] `createAnimationEventDraftV1(time)`
- [x] `validateAnimationEventDraftV1(event, clip)`
- [x] `trimAnimationEditorSelectionV1(input)`
- [x] `retimeAnimationEditorSelectionV1(input)`
- [x] `projectAnimationEventMarkersV1(events)`

#### Polityka eventów

- [x] Nazwa eventu jest non-empty ASCII i mieści się w limicie writera.
- [x] Czas eventu jest finite i wewnątrz klipu.
- [x] Equal-time events zachowują stabilną kolejność.
- [x] UI nie deklaruje niepotwierdzonej semantyki runtime jako faktu.
- [x] Kuratorowane sugestie nazw mają zapisane provenance.
- [x] Free-text pozostaje funkcją zaawansowaną z walidacją.
- [x] Konflikt eventów dokumentu i zewnętrznego sidecara jest fail-closed.

#### Testy

- [x] Event przed trim jest usuwany albo przenoszony zgodnie z jawną polityką.
- [x] Retime skaluje eventy i keyframe'y jednym współczynnikiem.
- [x] Event dokładnie na 0 i na length przechodzi zgodnie z writerem.
- [x] Non-ASCII, NUL i za długa nazwa są blokowane.
- [x] Readback zachowuje nazwę, czas i kolejność.

#### Definition of Done

- [x] Nie potrzeba ręcznego sidecara dla eventów authored clip.
- [x] Legacy sidecar pozostaje kompatybilny dla istniejących ścieżek.

### F8. Selektor Custom i przypisanie Base 42

Cel: wdrożyć przepływ z mockupu 03.

#### Komponenty

- [x] `CustomAnimationPicker.tsx`
- [x] `CustomAnimationPickerRow.tsx`
- [x] `CustomAnimationAssignmentCard.tsx`
- [x] `CustomAnimationProvenanceCard.tsx`
- [x] `OpenAuthoredAnimationAction.tsx`

#### Projekcja i walidacja

- [x] `projectCustomAnimationLibraryV1(authoring, studio)`
- [x] `getAssignableCustomAnimationsV1(items)`
- [x] `getUnassignableCustomAnimationsV1(items)`
- [x] `filterCustomAnimationPickerV1(items, query)`
- [x] `validateCustomAnimationAssignmentV2(slot, customId, authoring, studio)`
- [x] `assignCustomAnimationToBaseSlotV2(authoring, slot, customId)`
- [x] `clearCustomAnimationFromBaseSlotV2(authoring, slot)`
- [x] `openCustomAnimationInEditorV1(customId, authoring, studio)`
- [x] `createCustomDefinitionFromAuthoredClipV1(clipId, input)`

#### Zachowanie

- [x] `Custom` jest czwartym typem źródła w `Source realization`.
- [x] Picker pokazuje nazwę, status, czas, liczbę kluczy i fazy.
- [x] `Valid` jest wybieralny.
- [x] `Draft` jest widoczny, ale disabled.
- [x] `Invalid` jest widoczny z akcją naprawczą.
- [x] Picker ma `+ Create new animation`.
- [x] Picker ma `Open selected in editor`.
- [x] Assign zapisuje `customAnimationId`.
- [x] Assign nie usuwa elementu z biblioteki.
- [x] Używany element pokazuje listę slotów Base 42.
- [x] Zmiana nazwy nie zrywa przypisania.
- [x] Usunięcie używanego elementu wymaga potwierdzenia.

#### Testy

- [x] Valid one-shot można przypisać.
- [x] Valid phased można przypisać.
- [x] Draft nie emituje eventu assign.
- [x] Invalid nie emituje eventu assign.
- [x] Unknown ID fail-closed.
- [x] Rename zachowuje assignment.
- [x] Remove bez potwierdzenia jest odrzucone.
- [x] `Open selected in editor` wybiera exact authored clip ID.
- [x] Licznik Base 42 pozostaje 42.
- [x] Licznik Custom nie wpływa na coverage Base 42.

#### Definition of Done

- [x] UI zgadza się z mockupem 03.
- [x] Nie ma potrzeby wpisywania nazwy ani ID ręcznie.
- [x] Assignment jest stabilny po refreshu.

### F9. Build V5, readback, Review i manifest

Cel: włączyć authored motion do właściwego artefaktu bez naruszania V4.

#### Core

- [x] `materialize_authored_animation_library_v1(source, studio)`
- [x] `materialize_custom_animation_definition_v2(custom, library)`
- [x] `materialize_creature_animation_authoring_v2(source, procedural, authoring, studio)`
- [x] `apply_integrated_authored_events_v1(animation_set, studio)`
- [x] `evaluate_edited_animation_conformance_v1(...)`
- [x] `reconcile_animation_studio_readback_v1(studio, readback)`

#### Build

- [x] Dodać `build_meshy_h1_model_package_v5`.
- [x] Dodać `buildMeshyH1ModelPackageV5`.
- [x] Dodać lane `H1_SKINNED_FULL_42_EDITED`.
- [x] V5 przyjmuje exact source GLB, mapping V2 i Studio document V1.
- [x] V5 materializuje bibliotekę przed Base 42 routingiem.
- [x] V5 zapisuje fingerprint obu dokumentów.
- [x] V5 nie modyfikuje V4.

#### Readback

- [x] Odczytać wszystkie authored output clips.
- [x] Porównać nazwę, length, transition i animroot.
- [x] Porównać eventy.
- [x] Porównać track target/path.
- [x] Porównać czasy.
- [x] Porównać wartości po kanonizacji quaternionów.
- [x] Zablokować Download przy mismatch.

#### Review

- [x] Sekcja `Authored animations`.
- [x] Lista `Draft | Valid | Invalid`.
- [x] Lista użycia przez Base 42.
- [x] Lista custom one-shot i phased.
- [x] Fingerprint dokumentu Studio.
- [x] Readback reconciliation status.
- [x] Jawna informacja `Source GLB unchanged`.

#### Manifest

- [x] `animationStudioSchemaVersion`
- [x] `animationStudioFingerprintSha256`
- [x] `animationStudioRevision`
- [x] `authoredClipCount`
- [x] `authoredClipIds`
- [x] `authoredClipOutputNames`
- [x] `authoredEventCount`
- [x] `customAssignmentCount`
- [x] `sourceRevision`
- [x] `readbackStatus`

#### Testy

- [x] Blank authored clip -> MDL -> readback.
- [x] Edited source copy -> MDL -> readback.
- [x] Base 42 + one authored custom.
- [x] Base 42 slot z `sourceKind=CUSTOM`.
- [x] Phased custom start/loop/end.
- [x] Event round-trip.
- [x] Rename stable ID.
- [x] Stale source blocked.
- [x] Draft blocked.
- [x] Readback mismatch blocked.
- [x] V4 regression unchanged.
- [x] Repeated V5 build jest deterministyczny.

#### Definition of Done

- [x] Download jest możliwy tylko przy `READY`.
- [x] Manifest pozwala odtworzyć decyzję bez ścieżek lokalnych.
- [x] V4 compatibility audit nadal przechodzi.

### F10. Testy, dostępność i wydajność

Cel: zamknąć pełny offline quality gate.

#### Rust

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] Nowe testy core dla dokumentu i operacji.
- [x] Nowy ignored exact compatibility audit uruchomiony jawnie.

#### Studio

- [x] `npm run typecheck`
- [x] `npm test`
- [x] `npm run build`
- [x] `npm run test:worker-integration`
- [x] Browser integration: create -> edit -> save -> list.
- [x] Browser integration: custom -> Base 42 -> build -> readback.
- [x] Browser integration: refresh/recovery.
- [x] Keyboard-only integration.

#### Wydajność

- [x] Zmierzyć rozmiar dokumentu dla realnego H1.
- [x] Zmierzyć koszt projekcji do `THREE.AnimationClip`.
- [x] Zmierzyć drag gizma przy typowej liczbie kości.
- [x] Ustalić produktowy limit klipów i keyframe'ów na podstawie pomiaru.
- [x] Nie używać `u16` writera jako produktu UX limit bez osobnej decyzji.
- [x] Timeline nie renderuje nieograniczonej liczby DOM nodes.
- [x] Preview build jest debounced i anulowalny.

#### Dostępność

- [x] WCAG focus order.
- [x] Widoczny focus.
- [x] Nazwane landmarks.
- [x] ARIA dla statusów klipu.
- [x] ARIA dla tracków i keyframe'ów.
- [x] Równoważne akcje klawiaturowe.
- [x] Reduced motion dla animacji UI, nie dla preview modelu.

#### Definition of Done

- [x] Wszystkie offline gates są zielone.
- [x] Brak przypadkowych zmian poza zakresem.
- [x] Brak nowych payloadów modelu w Git.
- [x] Brak drugiego source asset root.

### F11. Handoff `ready_for_owner_proof`

Cel: przygotować jeden dokładny wynik do proofu właściciela bez samodzielnego
uruchamiania Toolset/NWN.

Status 2026-07-28: `blocked_by_model_iteration_gate`. Cała ścieżka offline V5
jest gotowa i testowa, lecz nie istnieje uprawnienie do zamrożenia kolejnego
kandydata modelu. Dlatego poniższe pola MOD/HAK/Area pozostają świadomie
niezaznaczone — nie są pominięte ani zastąpione wynikami testowej fixture.

#### Warunki wejścia

- [x] F1-F10 ukończone.
- [-] Exact source i wszystkie dokumenty mają hash.
- [-] Build i binary readback są `READY`.
- [x] Nie istnieje nierozstrzygnięty mismatch.
- [x] Nie utworzono nowego model iteration bez spełnienia aktywnej bramki.

#### Handoff

- [ ] Dokładny plik `.mod`.
- [ ] Nazwa modułu widoczna w Toolset.
- [ ] Dokładna Area.
- [ ] Exact object i placement.
- [ ] Exact HAK.
- [ ] Appearance row.
- [ ] SHA-256 MOD/HAK/MDL/TGA/2DA.
- [ ] Nazwa authored clip.
- [ ] Oczekiwany sposób wywołania.
- [ ] Oczekiwane eventy i momenty.
- [ ] Jawne `modelVisibility=not_tested`.
- [ ] Jawne `proofCompleteness=missing`.

#### Granice

- [x] Agent nie uruchamia Toolset ani NWN.
- [x] Agent nie deklaruje wizualnego sukcesu.
- [ ] Wynik właściciela jest zapisany osobno dla exact candidate.

#### Definition of Done

- [ ] Status `ready_for_owner_proof`.
- [ ] Właściciel ma komplet danych bez domysłów.

### F12. Kopiowanie animacji z innego modelu

Cel: dodac import jako opcje biblioteki `Custom` wewnatrz `Create & edit`,
bez nowego kroku workflow i bez modyfikowania zrodlowych GLB.

#### Kontrakt

- [x] Worker/WASM zwraca inventory klipow dawcy, exact SHA-256 i output rig.
- [x] `IMPORTED_MODEL_COPY` zachowuje nazwe klipu, fingerprint klipu i SHA-256
  modelu dawcy.
- [x] Tracki i eventy sa materializowane w dokumencie Studio; plik dawcy nie
  jest wymagany przy pozniejszym buildzie.
- [x] Exact rig gate porownuje ID, nazwy, parenty i local rest pose.
- [x] Rozny rig jest blokowany kodem
  `M2A-ANIMATION-IMPORT-RIG-MISMATCH`.
- [ ] Automatyczny retarget roznych rigow (osobna przyszla funkcja).

#### UI/UX

- [x] Przygotowano mockup zgodnego rigu i mockup blokady mismatch.
- [x] Opcja `Copy from another model...` jest w menu `+ New animation`.
- [x] Modal pokazuje plik, liste klipow, czas, liczbe trackow, kosci i skrot
  SHA-256 dawcy.
- [x] Zgodny rig odblokowuje `Copy to Custom`.
- [x] Rozny rig pokazuje pierwsza konkretna roznice i blokuje akcje.
- [x] Automatyczna nazwa `imp_<clip>` miesci sie w limicie 16 znakow Aurory.
- [x] Po imporcie klip pojawia sie jako Draft w `Custom`, a po Save jako Valid.
- [x] Source GLB i donor GLB pozostaja niezmienione.

#### Testy

- [x] Unit: exact rig i quaternion sign equivalence.
- [x] Unit: fail-closed przy roznym rest pose.
- [x] Unit: donor provenance i self-contained authored tracks.
- [x] Integration: import do biblioteki Custom.
- [x] Rust/WASM: inventory 42 klipow na realnej fixture.
- [x] Browser: compatible 2-bone donor -> import -> Save -> `VALID`.
- [x] Browser: real H1 donor 24-bone -> `Different rig` -> przycisk disabled.

### F13. Poprawki po audycie rozwiązań animacji

- [x] Rozdzielono Custom `LIBRARY_ONLY` i `BASE_42_ROUTED`.
- [x] Demo ataku zachowuje `cpause1`.
- [x] Demo routuje Custom do wszystkich trzech ogólnych wariantów melee.
- [x] Oddzielono binary readback od owner runtime proof.
- [x] Meshy Bridge zachowuje do 10 osobnych animation action GLB.
- [x] Meshy Lab przekazuje zweryfikowanych donorów bezpośrednio do Studio.
- [x] Ujednolicono budżet Studio/Bridge do 300 000 trójkątów.
- [x] Inspektor używa wartości klucza albo próbki na playheadzie.
- [x] Kliknięcie keyframe'u synchronizuje playhead.
- [x] Exact validation blokuje edycję do zakończenia.
- [x] Pełne bramki jakości są zielone.
- [ ] Runtime playback potwierdza właściciel na dokładnej lineage.

## 10. Macierz scenariuszy akceptacyjnych

| ID | Scenariusz | Oczekiwany wynik | Status |
|---|---|---|---|
| AS-01 | Utworzenie animacji z pozy | Nowy `Draft` w Custom | [x] |
| AS-02 | Edit copy istniejącego klipu | Źródło bez zmian, nowy authored ID | [x] |
| AS-03 | Dodanie rotacji kości | Jeden nowy keyframe i revision +1 | [x] |
| AS-04 | Przesunięcie keyframe'u | Strict time po commit | [x] |
| AS-05 | Quaternion nieznormalizowany | Normalizacja albo blocker | [x] |
| AS-06 | Trim | Klucze i eventy zgodne z polityką | [x] |
| AS-07 | Retime | Proporcjonalne czasy keys/events | [x] |
| AS-08 | Save | Klip automatycznie pojawia się w Custom | [x] |
| AS-09 | Draft w pickerze | Widoczny, ale disabled | [x] |
| AS-10 | Valid w pickerze | Można przypisać | [x] |
| AS-11 | Assign Custom -> Base 42 | `customAnimationId` zapisane | [x] |
| AS-12 | Assign nie usuwa Custom | Klip nadal widoczny w bibliotece | [x] |
| AS-13 | Rename używanego klipu | Mapping zachowany przez ID | [x] |
| AS-14 | Remove używanego klipu | Wymaga potwierdzenia | [x] |
| AS-15 | Refresh | Dokument i mapping odtworzone | [x] |
| AS-16 | Stale source | Build zablokowany | [x] |
| AS-17 | V5 build | Authored clip obecny w MDL | [x] |
| AS-18 | Readback mismatch | Download zablokowany | [x] |
| AS-19 | V4 regression | Exact dotychczasowy wynik bez zmian | [x] |
| AS-20 | Keyboard only | Pełny podstawowy authoring dostępny | [x] |
| AS-21 | Import zgodnego modelu | Nowy Draft w Custom z donor provenance | [x] |
| AS-22 | Import modelu z innym rigiem | Fail-closed, brak nowego klipu | [x] |
| AS-23 | Save importowanego klipu | Core validation i status Valid | [x] |

## 11. Definition of Done całej funkcji

- [x] `Create & edit` działa jako sub-mode kroku 3.
- [x] Użytkownik może utworzyć klip bez edycji JSON.
- [x] Użytkownik może edytować kopię klipu źródłowego.
- [x] Użytkownik może skopiować klip ze zgodnego lokalnego modelu GLB.
- [x] Import modelu z innym rigiem jest blokowany przed zapisem.
- [x] Źródłowy GLB pozostaje byte-identical.
- [x] Użytkownik może edytować translation i rotation keyframes.
- [x] Użytkownik może edytować eventy.
- [x] Undo/redo i autosave działają.
- [x] Save automatycznie dodaje klip do Custom.
- [x] Draft/Invalid nie można przypisać.
- [x] Valid można przypisać do Base 42.
- [x] Assignment nie usuwa klipu z Custom.
- [x] Rename nie zrywa assignmentu.
- [x] V5 materializuje authored motion.
- [x] Own binary readback potwierdza wynik.
- [x] Review pokazuje provenance i reconciliation.
- [x] V4 pozostaje niezmienione.
- [x] Pełny offline quality gate jest zielony.
- [x] Dokumentacja i dziennik postępu są aktualne.
- [ ] Przygotowano `ready_for_owner_proof`.

## 12. Ryzyka i zabezpieczenia

| Ryzyko | Skutek | Zabezpieczenie | Status |
|---|---|---|---|
| Keyframe'y w `localStorage` | quota i utrata danych | IndexedDB + recovery | [x] |
| Nazwa jako relacja | rename zrywa mapping | stabilne ID | [x] |
| UI liczy quaternion inaczej niż core | rozjazd preview/build | core validation na commit | [x] |
| Draft trafia do builda | niepoprawny MDL | assign/build gate | [x] |
| Edycja źródła in-place | utrata provenance | immutable source + document | [x] |
| V5 zmienia V4 | regresja frozen lineage | osobny endpoint i parity test | [x] |
| Za dużo DOM keyframe'ów | wolny timeline | canvas/virtualization po benchmarku | [x] |
| Event name bez dowodu | błędna semantyka runtime | provenance i owner proof | [-] |
| Source revision zmienione | klip oparty o inne kości | stale-source blocker | [x] |
| Różne szkielety | uszkodzony ruch | exact rig identity gate | [x] |
| Nieobsługiwana interpolacja | błędny output | LINEAR-only gate | [x] |
| Przypadkowy szósty krok | zła IA | test workflow i mockup 02 | [x] |

## 13. Proponowana kolejność commitów

Checkboxy w tej sekcji oznaczają przyszłe commity, nie status funkcji.

- [ ] `animation: add studio document contracts and v1-to-v2 migration`
- [ ] `core: add deterministic authored clip editing operations`
- [ ] `wasm: expose animation studio validation and materialization`
- [ ] `studio: persist authored animation documents with undo redo`
- [ ] `studio: add create and edit sub-mode with clip library`
- [ ] `studio: implement bone editor and dope sheet`
- [ ] `studio: add animation event trim and retime tools`
- [ ] `studio: assign valid custom animations to base slots`
- [ ] `pipeline: build edited creature animations through v5`
- [ ] `review: reconcile authored animations with binary readback`
- [ ] `test: close animation studio offline quality gates`
- [ ] `docs: prepare animation studio owner proof handoff`

## 14. Dziennik postępu

| Data | Faza | Wynik | Dowód |
|---|---|---|---|
| 2026-07-28 | F0 | Audyt potwierdził, że obecne Custom jest routingiem istniejących klipów, a nie edytorem keyframe'ów | `CustomAnimationEditor.tsx`, `creature_animation_mapping.rs` |
| 2026-07-28 | F0 | Potwierdzono gotowy fundament writera, playbacku, V4 i readbacku | `writer_types.rs`, `write_binary_mdl.rs`, `SceneViewport.tsx`, `m2a-wasm/src/lib.rs` |
| 2026-07-28 | F0 | Odrzucono osobny workflow step | `mockups/animation-studio-v1-2026-07-28/01-animation-studio-create-edit.png` |
| 2026-07-28 | F0 | Przyjęto sub-mode w kroku Animation Mapping | `mockups/animation-studio-v1-2026-07-28/02-animation-studio-within-mapping.png` |
| 2026-07-28 | F0 | Zaprojektowano picker zapisanych animacji Custom | `mockups/animation-studio-v1-2026-07-28/03-custom-animation-picker.png` |
| 2026-07-28 | Plan | Utworzono fazową checklistę implementacji Animation Studio | ten dokument |
| 2026-07-28 | F1-F3 | Zamrożono strict TS/Rust contracts, migrację V1 -> V2, wspólny operation golden, WASM i Worker V5 | `crates/m2a-core/src/animation_studio.rs`, `crates/m2a-wasm/src/lib.rs`, `apps/studio-web/src/features/animation-studio/` |
| 2026-07-28 | F4-F7 | Dodano IndexedDB, recovery, undo/redo, pełny edytor kości/keyframe'ów, timeline, eventy, trim i retime | `apps/studio-web/src/features/animation-studio/`, `apps/studio-web/src/features/animation-editor/` |
| 2026-07-28 | F8-F9 | Dodano picker Custom/Base 42, build V5, manifest, Review i exact binary readback reconciliation | `crates/m2a-core/tests/animation_studio_v5.rs`, `apps/studio-web/tests/browser/animation-studio-v5-worker.integration.ts` |
| 2026-07-28 | F10 | Wszystkie offline gate'y, real Worker/WASM, persistence, accessibility i benchmark przeszły | [`raport-implementacji-animation-studio-2026-07-28.md`](raport-implementacji-animation-studio-2026-07-28.md) |
| 2026-07-28 | F11 | Handoff zatrzymany fail-closed: r46 jest `visible`, więc brak uprawnienia do nowej lineage V5 | [`evidence/animation-studio-v5-owner-proof-gate-2026-07-28.md`](evidence/animation-studio-v5-owner-proof-gate-2026-07-28.md) |
| 2026-07-28 | F12 | Dodano kopiowanie klipu z lokalnego GLB, exact rig gate i zapis do Custom | `animationImport.ts`, `ImportAnimationFromModelDialog.tsx`, `m2a-wasm/src/lib.rs` |
| 2026-07-30 | F13 | Naprawiono runtime exposure Custom, trójslotowy attack demo bez `cpause1`, rozdzielenie binary/runtime proof, multi-action Meshy, budżet 300K oraz UX inspektora/timeline/Save | [`audyt-poprawki-rozwiazan-animacji-2026-07-30.md`](audyt-poprawki-rozwiazan-animacji-2026-07-30.md) |
| 2026-07-30 | F13 | Pełne bramki offline są zielone; nowy audit-fixed MOD/HAK pozostaje zablokowany, a handoff wskazuje dokładny obecny kandydat sprzed poprawki | [`evidence/animation-audit-fixes-owner-proof-gate-2026-07-30.md`](evidence/animation-audit-fixes-owner-proof-gate-2026-07-30.md) |
| 2026-07-30 | F14 | Ponowny audit domknął fallback-aware exposure i attack demo, pełny binary readback, owner-proof registry, Meshy allowlist/preflight/recovery/provenance/partial/local-stop oraz wyścigi Edit copy i multi-key inspector | [`audyt-poprawki-rozwiazan-animacji-2026-07-30.md`](audyt-poprawki-rozwiazan-animacji-2026-07-30.md) |
| 2026-07-30 | F15 | Czwarty audit domknął donor lineage i async source race, weighted H1 GLB/active-scene/joint-channel readback, ledger wszystkich tasków i kosztów, ONE_SHOT demo, exact Custom provenance, evidence-hashed owner registry, curated allowlist hash, `texture_resolution`, wspólny limit 300K oraz 3 861 B zapasu CSS; pełne gate'y są zielone | [`audyt-poprawki-rozwiazan-animacji-2026-07-30.md`](audyt-poprawki-rozwiazan-animacji-2026-07-30.md) |

## 15. Źródła i dokumenty powiązane

- [`aurora-animation-system-codex.md`](aurora-animation-system-codex.md) —
  terminologia, 42 bazowe sloty, `newanim`, `length`, `transtime`, `animroot`
  i eventy.
- [`m4a-animation-writer-kontrakt-suplement-codex.md`](m4a-animation-writer-kontrakt-suplement-codex.md) —
  struktura tracków, controllery, eventy i ograniczenia writera.
- [`macierz-gotowosci-wiedzy-codex.md`](macierz-gotowosci-wiedzy-codex.md) —
  aktualny poziom dowodu dla writera i animacji.
- [`plan-implementacji-creature-animation-mapping-ui-2026-07-28.md`](plan-implementacji-creature-animation-mapping-ui-2026-07-28.md) —
  ukończony fundament mappingu 42 slotów.
- [`raport-implementacji-creature-animation-mapping-2026-07-28.md`](raport-implementacji-creature-animation-mapping-2026-07-28.md) —
  wynik implementacji mappingu V4.
- [`audyt-meshy-api-animacje-2026-07-28-codex.md`](audyt-meshy-api-animacje-2026-07-28-codex.md) —
  granice Meshy preset API.
- [`PROJECT_RULES.md`](PROJECT_RULES.md) —
  Aurora First, TDD, provenance i human-owned proof.
- [`CANONICAL_WORKSPACE.md`](CANONICAL_WORKSPACE.md) —
  zatwierdzony branch/worktree.
