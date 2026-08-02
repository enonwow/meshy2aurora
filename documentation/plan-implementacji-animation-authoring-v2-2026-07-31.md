# Animation Authoring V2 — plan implementacji i kryteria ukończenia

Data: 2026-07-31  
Status: `PLAN GOTOWY / IMPLEMENTACJA NIEZACZĘTA`  
Branch/worktree: `animation` / `C:\Projects\meshy2aurora\.worktrees\animation`

Dokument bazowy:
[audyt animacji — dalsze usprawnienia](audyt-animacji-dalsze-usprawnienia-2026-07-31.md).

## 1. Cel

Rozwinąć obecne Animation Studio z działającego edytora klatek w narzędzie,
które pozwala świadomie tworzyć, poprawiać i weryfikować animacje przed
eksportem do Aurory. Użytkownik ma widzieć dokładnie ten ruch, który zostanie
zapisany do binary MDL, otrzymywać mierzalne informacje o problemach ruchu i
poprawiać je w viewportcie bez zgadywania.

Końcowy przepływ produktu:

`GLB/preset/donor → Custom → edit commands → pose/quality analysis → preview → Base 42 → Core → binary MDL → own readback → demo → owner proof`

## 2. Granice zakresu

### W zakresie

- kanoniczne komendy edycji wykonywane przez Core;
- world-pose sampler i parity preview/writer/package;
- Motion Quality Analyzer;
- prawdziwy gizmo 3D, wybór kości i narzędzia pozy;
- loop seam, root motion i contact tools;
- podgląd i baked attachment broni dla direct creature `MODELTYPE=S`;
- sequence preview;
- semantic retarget V2 i batch transfer;
- warstwy, maski kości, editor curves i deterministic linear bake;
- redukcja klatek z raportem błędu;
- integracja biblioteki i contribution quality gates;
- pełny offline proof oraz handoff do proofu właścicielskiego.

### Poza zakresem tego planu

- runtime IK wykonywane przez NWN;
- pełny solver dwuręczny utrzymujący dwa parenty jednocześnie;
- wymienna widoczna broń przez `Equip_ItemList` dla `MODELTYPE=S`;
- konwersja direct creature do humanoid part model `MODELTYPE=P`;
- automatyczne publikowanie albo pobieranie niepodpisanego kodu/payloadów;
- zgadywanie nazw gameplay eventów bez Aurora First evidence;
- agent-run Toolset/NWN proof bez nowej, bezpośredniej zgody właściciela.

### Dwie granice ukończenia

1. `OFFLINE_PRODUCT_COMPLETE` — aplikacja, Core, writer, readback, testy i demo
   spełniają wszystkie kryteria offline.
2. `OWNER_RUNTIME_VERIFIED` — właściciel potwierdzi ten sam immutable lineage
   w Toolset/NWN. Brak wyniku runtime nie unieważnia ukończenia technicznego,
   ale zabrania claimu zgodności runtime.

## 3. Inwarianty, których nie wolno naruszyć

- source i donor GLB pozostają byte-identical;
- produkt pozostaje webowy, local-first i offline-capable;
- Core jest jedynym właścicielem kanonicznej matematyki i materializacji;
- TypeScript może wykonywać optimistic preview, ale nie ustanawia trwałego
  wyniku bez reconciliation z Core;
- Base 42, istniejący exact-copy i same-hierarchy retarget V1 pozostają
  kompatybilne;
- writer emituje native binary MDL, nie ASCII runtime asset;
- każdy packaged klip ma own binary readback `MATCH`;
- wspólny limit modelu to 300 000 trójkątów, a per-stream boundary pozostaje
  osobną granicą writera;
- eventy zachowują istniejący kontrakt, dopóki Aurora First nie potwierdzi
  kolejnego;
- MP4 musi pochodzić z normalnego pipeline aplikacji i pokazywać rzeczywisty
  ruch, nie tylko postęp zegara;
- MOD/HAK i runtime proof respektują model-iteration gate.

## 4. Docelowy podział odpowiedzialności

| Warstwa | Odpowiedzialność |
|---|---|
| Core | komendy, sampling, jakość, retarget, bake, attachment, fingerprints |
| WASM | ścisła serializacja wejścia/wyjścia Core, bez własnej matematyki |
| Worker | wykonanie poza main thread, cancel i stale-result guards |
| Studio | interakcja, optimistic preview, wizualizacja raportów |
| Three.js viewport | render i gest, bez ustanawiania canonical output |
| Model pipeline | Custom/Base 42, MDL, package i readback |
| Review | parity/quality/provenance i blokady Download |
| Evidence | immutable wyniki offline i owner-reported runtime |

## 5. Zależności faz

| Faza | Zależy od | Otwiera |
|---|---|---|
| F0 baseline | — | wszystkie kolejne fazy |
| F1 commands | F0 | F3, F4, F5, F7 |
| F2 pose parity | F0 | F3, F5, F6, F9 |
| F3 quality | F1, F2 | F5, F6, F9 |
| F4 gizmo/pose | F1, F2 | F5, F6 |
| F5 loop/root/contact | F3, F4 | F9 |
| F6 held weapon/sequence | F2, F3, F4 | F9 |
| F7 retarget V2/batch | F1, F2 | F8, F9 |
| F8 layers/curves | F1, F2, F7 | F9 |
| F9 integration/proof | F0–F8 | release/handoff |

F1 i F2 mogą być rozwijane niezależnie po F0. Implementacja UI nie powinna
wyprzedzać zatwierdzonego kontraktu Core.

## 6. Plan implementacji

### F0 — baseline, decyzje i czerwone testy

Cel: zamrozić działający stan oraz określić polityki matematyczne przed zmianą
kontraktów.

#### Funkcje i kontrakty

- [ ] dodać `AnimationAuthoringCapabilityReportV1`;
- [ ] dodać `AnimationPoseParityPolicyV1`;
- [ ] dodać `AnimationQualityPolicyV1` z progami normalizowanymi wysokością rigu;
- [ ] związać polityki z `schemaVersion` i fingerprintem;
- [ ] zapisać exact baseline Void Knight oraz Fogbound→Void Knight;
- [ ] zapisać listę obecnych publicznych WASM/Worker requestów i bundle budgets.

#### Testy najpierw

- [ ] fixture, w którym czas rośnie, ale wszystkie pozy są identyczne;
- [ ] fixture z kontrolowaną różnicą preview/binary pose;
- [ ] fixture ze szwem pętli;
- [ ] fixture z root drift;
- [ ] fixture ślizgającej się stopy i penetracji podłoża;
- [ ] fixture motion spike;
- [ ] fixture poprawnego i błędnego chwytu broni;
- [ ] test braku regresji wszystkich obecnych wire fixtures.

#### Gate F0

- [ ] nowe testy są czerwone z oczekiwanym kodem diagnostycznym;
- [ ] stare testy nadal są zielone;
- [ ] polityki nie mają wartości rozproszonych między Rust i TypeScript.

### F1 — kanoniczne komendy edytora

Cel: każda trwała edycja jest deterministycznym wynikiem Core. UI może
podglądać gest lokalnie, lecz nie zapisuje własnej wersji matematyki.

#### Core

- [x] `AnimationEditCommandV1`;
- [x] `AnimationEditCommandBatchV1`;
- [x] `AnimationEditCommandContextV1` z source/document/clip revision;
- [x] `AnimationEditCommandResultV1` z canonical document i fingerprintem;
- [x] `apply_animation_edit_command_batch_v1`;
- [x] `fingerprint_animation_edit_command_batch_v1`;
- [ ] `reconcile_animation_edit_preview_v1`;
- [ ] objąć komendami istniejące keyframe/event/trim/retime/rename/duplicate;
- [x] atomowe wykonanie: cały batch albo zero mutacji;
- [ ] idempotentny replay jednego command identity.

#### WASM/Worker

- [x] `applyAnimationEditCommandBatchV1`;
- [x] typowany request/response i exhaustive switch gate;
- [ ] cancel przed commit;
- [x] odrzucenie wyniku po zmianie source, project, document albo clip revision;
- [ ] Native/WASM/Worker byte-identical result JSON.

#### Studio

- [ ] optimistic gesture state jest ephemeral;
- [ ] pointer-up/keyboard commit wysyła jeden batch;
- [ ] reconciliation mismatch cofa preview i pokazuje diagnostykę;
- [ ] undo/redo zapisuje kanoniczne wyniki lub komendy, nie obce snapshoty;
- [ ] autosave zaczyna się dopiero po Core commit.

#### Gate F1

- [ ] wszystkie dotychczasowe operacje edycyjne przechodzą przez nowy boundary;
- [ ] TypeScript nie zawiera drugiego canonical algorytmu operacji;
- [ ] race/cancel/failure pozostawiają projekt bez zmian.

### F2 — world-pose sampler i preview/output parity

Cel: udowodnić, że viewport, writer readback i package readback opisują tę samą
pozę w tych samych chwilach.

#### Core

- [x] `AnimationWorldPoseV1` i `AnimationBoneWorldTransformV1`;
- [x] `sample_animation_local_pose_v1`;
- [x] `sample_animation_world_pose_v1`;
- [x] `animation_pose_fingerprint_v1`;
- [x] `AnimationPoseParityReportV1`;
- [x] `evaluate_animation_pose_parity_v1`;
- [x] sampled times: start, end, eventy, wszystkie keys oraz równomierna siatka;
- [x] quaternion shortest-arc comparison;
- [ ] jawne basis/profile mapping, bez ukrytej korekty w UI.

#### Studio/Review

- [ ] viewport udostępnia canonical sampled pose telemetry;
- [ ] Review pokazuje max translation/angular delta, bone i time;
- [ ] pose mismatch jest `BLOCKING` dla Download;
- [ ] `MATCH` wymaga writer oraz packaged readback;
- [ ] raport i fingerprint trafiają do canonical build result.

#### Gate F2

- [ ] exact Void Knight daje parity zgodne z jedną wersjonowaną polityką;
- [ ] celowo zmieniona kość/czas/basis daje stabilny mismatch;
- [ ] q i -q są uznawane za tę samą rotację;
- [ ] source GLB pozostaje byte-identical.

### F3 — Motion Quality Analyzer

Cel: znaleźć problemy, których poprawny schema/readback nie wykryje.

#### Core

- [x] `AnimationQualityIssueKindV1`;
- [x] `AnimationQualityIssueV1` z bone, interval, metric i action;
- [x] `AnimationQualityReportV1` z policy/fingerprint;
- [x] `analyze_animation_motion_quality_v1`;
- [x] `detect_static_playback_v1`;
- [x] `measure_loop_discontinuity_v1`;
- [x] `measure_root_drift_v1`;
- [x] `detect_ground_penetration_v1`;
- [x] `detect_foot_sliding_v1` dla jawnych contact bones;
- [x] `detect_motion_spikes_v1`;
- [ ] `measure_pose_transition_jump_v1`;
- [ ] klasyfikacja schema/parity jako blocking, jakości artystycznej jako warning.

#### Studio

- [x] panel `Animation quality`;
- [ ] filtrowanie INFO/WARNING/BLOCKING;
- [ ] marker zakresu na timeline;
- [x] `Jump to issue` ustawia playhead i zaznacza kość;
- [ ] overlay ground/contact/trail w viewportcie;
- [ ] before/after comparison po naprawie.

#### MP4 integrity

- [ ] `AnimationMotionDiversityReportV1`;
- [ ] minimum dwóch różnych pose fingerprints dla klipu `MOTION`;
- [ ] telemetry czasu i pozy pochodzi z tego samego viewport runtime;
- [ ] MP4 manifest zawiera clip, source SHA, document fingerprint i quality hash;
- [ ] nieruchomy film z poruszającym się zegarem fail closed.

#### Gate F3

- [ ] każde fixture F0 jest wykrywane bez false positive na poprawnym fixture;
- [ ] exact `m2a_voidcleave` ma zapisany raport, nie tylko subiektywny opis;
- [ ] analiza działa poza main thread i jest deterministyczna.

### F4 — gizmo 3D, szkielet i narzędzia pozy

Cel: użytkownik tworzy ruch bez ręcznego wpisywania większości quaternionów.

#### Viewport

- [ ] `ViewportBonePicker`;
- [ ] `SkeletonOverlay`;
- [x] `ThreeBoneTransformControls` dla translate/rotate;
- [ ] osie X/Y/Z, local/world i snapping; _(XYZ/local/world gotowe; snapping pozostaje)_
- [ ] widoczny pivot oraz parent chain;
- [x] camera controls nie przechwytują aktywnego gestu;
- [ ] Escape anuluje, Enter/pointer-up commit'uje;
- [x] pola numeryczne zapewniają równoważną obsługę klawiaturą.

#### Core/commands

- [ ] `solve_bone_local_transform_from_gizmo_v1`;
- [ ] `insert_pose_keyset_v1`;
- [ ] `copy_animation_pose_v1`;
- [ ] `paste_animation_pose_v1`;
- [ ] `reset_bone_to_rest_v1`;
- [ ] `reset_chain_to_rest_v1`;
- [ ] `HumanoidMirrorMapV1` z jawną kompatybilnością;
- [ ] `mirror_humanoid_pose_v1`;
- [ ] groups/multi-select oraz stable command ordering.

#### Porównanie

- [ ] synchronized split Source/Edited;
- [ ] ghost previous/next key;
- [ ] motion trails dłoni, stóp i wybranej kości;
- [ ] heatmapa bone pose delta włączana przez użytkownika.

#### Gate F4

- [ ] pełna edycja XYZ local/world na mouse i keyboard;
- [ ] jeden gest tworzy jedną pozycję w undo history;
- [ ] brak trwałej mutacji przed Core commit;
- [ ] exact Void Knight zachowuje obecne p95 playbacku;
- [ ] interaction long task nie przekracza 100 ms.

### F5 — loop, root motion i contact tools

Cel: naprawiać najczęstsze problemy idle/walk/run/one-shot bez ręcznego
przesuwania dziesiątek klatek.

#### Loop

- [x] `inspect_loop_seam_v1`;
- [x] `blend_animation_loop_seam_v1`;
- [x] użytkownik wybiera szerokość blend window;
- [ ] first/last pose i velocity raport before/after;
- [x] eventy poza zakresem blend pozostają niezmienione.

#### Root motion

- [x] `AnimationRootMotionPolicyV1`: Preserve, InPlace, Scale;
- [x] `extract_root_motion_v1`;
- [x] `lock_root_motion_in_place_v1`;
- [x] `scale_root_motion_v1`;
- [x] `restore_root_motion_v1`;
- [x] pełne provenance transformacji i reversible source layer.

#### Contacts

- [x] `AnimationContactBoneSetV1`;
- [x] `suggest_contact_intervals_v1`;
- [x] `lock_contact_bone_interval_v1`;
- [ ] ground plane jest jawny i widoczny;
- [x] wynik bake'uje się do zwykłych translation/rotation tracks;
- [x] before/after foot-slide metric.

#### Gate F5

- [ ] synthetic loop seam zostaje zmniejszony zgodnie z policy;
- [ ] in-place usuwa root displacement bez zmiany ruchu kończyn ponad tolerance;
- [ ] Preserve jest byte/fingerprint stable;
- [ ] foot lock zmniejsza metrykę na fixture i nie zmienia source GLB.

### F6 — held weapon i sequence preview

Cel: tworzyć i oceniać atak razem z widoczną bronią w aplikacji.

#### Kontrakt attachment

- [x] `HeldWeaponAttachmentV1`;
- [x] `HeldWeaponModeV1::BakedDirectCreature`;
- [x] source filename/size/SHA/provenance;
- [x] primary hand, target node ID/name i local transform;
- [x] pivot policy oraz material/texture policy;
- [x] opcjonalny secondary-hand guide;
- [x] attachment revision i canonical fingerprint;
- [ ] projekt/persistence/backup migration.

#### Core/model pipeline

- [x] `inspect_held_weapon_source_v1`;
- [x] `resolve_hand_attachment_target_v1`;
- [x] `compose_held_weapon_attachment_v1`;
- [ ] `solve_weapon_grip_offset_v1`;
- [x] `measure_primary_hand_grip_error_v1`;
- [x] `measure_secondary_hand_grip_error_v1`;
- [x] `bake_held_weapon_attachment_v1` jako rigid segment dłoni; _(IR-level; product-builder integration pozostaje)_
- [ ] geometry/material/texture/triangle/stream gates;
- [ ] writer hierarchy i packaged binary readback;
- [ ] model oraz weapon inputs pozostają byte-identical.

#### Studio

- [x] panel `Held equipment` wewnątrz Animation Studio, nie nowy workflow step;
- [x] wybór źródła, ręki, bone i transformu;
- [ ] gizmo grip/pivot;
- [x] preview całego klipu z bronią;
- [x] warning, że baked weapon nie jest wymiennym `Equip_ItemList`;
- [ ] secondary hand guide i wykres grip error;
- [x] usunięcie attachment przywraca model bez broni.

#### Sequence preview

- [x] `AnimationSequencePreviewV1`;
- [x] idle→attack→idle;
- [ ] walk→stop;
- [ ] phased fire-and-forget;
- [x] transition jump quality report;
- [ ] event/phase markers na wspólnym timeline.

#### Gate F6

- [ ] broń pozostaje rigid-parented do primary hand przez cały klip;
- [ ] secondary guide raportuje odległość, ale nie udaje pełnego IK;
- [ ] binary MDL zachowuje model, broń, tekstury i animację;
- [ ] direct creature UI nie obiecuje runtime swap;
- [ ] exact demo powstaje wyłącznie z normalnego pipeline aplikacji.

### F7 — semantic retarget V2 i batch transfer

Cel: obsłużyć typowe różnice humanoidalnych rigów oraz wiele klipów w jednej
kontrolowanej operacji.

#### Compatibility

- [x] `HumanoidBoneSemanticV2`;
- [x] `HumanoidSemanticBoneMapV2`;
- [x] aliasy nazw są wersjonowane i widoczne;
- [x] required/optional/ignored donor bones;
- [x] chain/root/rest/scale diagnostics;
- [x] manual mapping wymaga jawnego zatwierdzenia; _(Core; UI override editor pozostaje)_
- [x] `inspect_humanoid_retarget_compatibility_v2`;
- [x] nigdy nie obniżać zachowania fail-closed V1.

#### Retarget

- [x] `retarget_animation_clip_humanoid_v2`;
- [x] chain-aware rest delta;
- [ ] optional twist distribution jako osobna jawna policy;
- [ ] root motion policy współdzielona z F5;
- [ ] quality report przed i po;
- [x] pełne donor/target/map/policy/output provenance.

#### Batch

- [x] `AnimationTransferBatchRequestV1`;
- [x] `prepare_animation_transfer_batch_v1`;
- [x] per-clip mode, rename, mapping i diagnostics;
- [x] transakcyjny commit `ALL_OR_NOTHING`;
- [ ] race/cancel nie zostawia części Custom;
- [ ] UI multi-select, progress i retry jednego niezatwierdzonego klipu;
- [ ] preset biblioteki może użyć retargetu tylko po jawnym wyborze użytkownika.

#### Gate F7

- [ ] exact V1 copy i same-hierarchy retarget pozostają identyczne;
- [ ] Fogbound 9/9 przechodzi batch i daje ten sam wynik co dziewięć operacji;
- [ ] synthetic alias/optional/twist cases mają golden results;
- [ ] incompatible/manual-unconfirmed nie mutuje projektu.

### F8 — warstwy, krzywe i redukcja klatek

Cel: umożliwić korektę ruchu bez niszczenia źródła oraz kontrolować rozmiar
outputu.

#### Warstwy

- [x] `AnimationEditLayerV1`;
- [x] `AnimationLayerModeV1`: Base, Additive, Override;
- [x] `AnimationBoneMaskV1`;
- [x] mute/solo/weight i stable layer ordering;
- [x] `bake_animation_layers_v1`;
- [x] warstwa poprawki ręki/tułowia zachowuje donor provenance;
- [ ] persistence i migration kolejnej wersji Studio document.

#### Krzywe

- [x] editor-only curve/tangent contract;
- [ ] graph editor translation i bezpieczna reprezentacja rotation; _(bezpieczny Core + panel smooth gotowe; pełny graph editor pozostaje)_
- [x] `resample_animation_curve_to_linear_v1`;
- [x] error-bounded adaptive sampling;
- [x] wynik MDL nadal korzysta z potwierdzonego kontraktu liniowego.

#### Optymalizacja

- [x] `AnimationKeyReductionPolicyV1`;
- [x] `reduce_animation_keyframes_v1`;
- [x] max position/angular error report;
- [ ] rows i rozmiar MDL before/after;
- [x] event times, duration i transition pozostają niezmienione;
- [ ] row limit jest sprawdzany przed materializacją.

#### Gate F8

- [ ] mute layer daje fingerprint bazowego klipu;
- [ ] bake jest deterministyczny native/WASM;
- [ ] curve→linear i key reduction pozostają w policy tolerance;
- [ ] writer/package readback ma `MATCH` dla każdego klipu.

### F9 — integracja, biblioteka, demo i handoff

Cel: domknąć produkt, regresje i uczciwie oddzielić offline proof od runtime.

#### Integracja produktu

- [ ] project schema migrations round-tripują wszystkie nowe kontrakty;
- [ ] Review pokazuje command, pose parity, quality, attachment i batch evidence;
- [ ] Download blockuje tylko udokumentowane blocking diagnostics;
- [ ] backup/restore zachowuje provenance, ale nie duplikuje donor GLB;
- [ ] source/donor/weapon nie trafiają do bundle ani zdalnego backendu;
- [ ] browser działa offline po załadowaniu aplikacji.

#### Biblioteka

- [ ] contribution zawiera quality summary i pose-diversity evidence;
- [ ] compatibility obejmuje strict-rig oraz jawny retarget mode;
- [ ] nowe tagi tylko przez wersjonowany słownik;
- [ ] root LICENSE/public contribution policy pozostaje owner decision;
- [ ] signed update channel pozostaje osobnym release gate.

#### Demo

- [ ] normalny flow aplikacji od wejścia do Review;
- [ ] exact model, clip, source SHA i document fingerprint są widoczne/zapisane;
- [ ] MP4 H.264, stały FPS i hash;
- [ ] motion-diversity gate PASS;
- [ ] klatki kontrolne pokazują różne fazy ruchu;
- [ ] dołączone quality/parity reports;
- [ ] demo nie jest nazywane runtime proofem.

#### Owner handoff

- [ ] nowa iteracja tylko po spełnieniu model-iteration gate;
- [ ] exact `.mod` filename jako pierwsza linia handoffu;
- [ ] module display name i exact Area;
- [ ] object, placement, HAK, Appearance row i resref;
- [ ] SHA-256 MOD/HAK/MDL/textures/2DA;
- [ ] sposób wywołania animacji i oczekiwane zdarzenia;
- [ ] `modelVisibility=not_tested` i `proofCompleteness=missing` przed wynikiem;
- [ ] wynik runtime wpisuje wyłącznie właściciel.

## 7. Kryteria ukończenia

Każde kryterium jest binarne. Screenshot, poprawny build albo sam binary
readback nie zastępują całego kryterium.

### K0 — zakres i bezpieczeństwo

- [ ] wszystkie nowe kontrakty są wersjonowane i strict-deserialized;
- [ ] unknown fields, nonfinite values, limits i stale revisions fail closed;
- [ ] source/donor/weapon GLB są byte-identical przed i po;
- [ ] nie powstał drugi asset root ani zależność od `aurora-web`;
- [ ] nie utworzono nieautoryzowanej iteracji MOD/HAK.

### K1 — jedno źródło prawdy

- [ ] Core posiada canonical commands, sampling, quality, retarget i bake;
- [ ] WASM i Worker są cienkimi adapterami;
- [ ] TypeScript nie posiada niezależnej canonical matematyki;
- [ ] optimistic preview jest zawsze reconciliation-bound;
- [ ] native/WASM/Worker wyniki mają tę samą treść i fingerprint.

### K2 — atomowe komendy i race safety

- [ ] batch wykonuje się w całości albo nie mutuje nic;
- [ ] source/document/clip revision mismatch odrzuca wynik;
- [ ] cancel, Worker failure i component unmount nie zapisują dokumentu;
- [ ] jeden gest daje jeden undo entry;
- [ ] replay tej samej identity jest deterministyczny/idempotentny.

### K3 — pose parity

- [ ] sampled local/world poses mają golden fixture;
- [ ] preview, writer i package są porównywane w tych samych chwilach;
- [ ] q/-q nie tworzy false mismatch;
- [ ] celowa zmiana basis/bone/time jest wykrywana;
- [ ] exact Void Knight ma `MATCH` według jednej wersjonowanej policy;
- [ ] mismatch blokuje Download i wskazuje bone/time/metric.

### K4 — jakość ruchu

- [ ] static playback, loop seam, root drift, ground penetration, foot slide i
  motion spike są wykrywane na osobnych fixture;
- [ ] poprawne fixture nie dają blocking false positives;
- [ ] progi są normalizowane i należą do Core policy;
- [ ] UI potrafi przejść do dokładnego problemu;
- [ ] quality report ma stable fingerprint i trafia do build evidence.

### K5 — viewport i UX authoringu

- [ ] kość można wybrać z modelu i drzewa;
- [ ] translate/rotate X/Y/Z działa local/world;
- [ ] snapping, cancel, commit, undo i numeric input działają;
- [ ] Source/Edited split, ghost i trails nie zmieniają dokumentu;
- [ ] klawiatura, focus i screen reader przechodzą testy;
- [ ] playback p95 pozostaje `<=25 ms` desktop i `<=50 ms` przy CPU throttle 4×;
- [ ] interaction nie tworzy main-thread long task >100 ms.

### K6 — loop/root/contact

- [ ] seam report istnieje before/after;
- [ ] repair zmniejsza wybraną metrykę bez zmiany source;
- [ ] Preserve root motion jest stabilny;
- [ ] InPlace usuwa displacement według policy;
- [ ] contact lock zmniejsza foot slide na fixture;
- [ ] wszystkie operacje są command/provenance-bound i bake'ują się do tracków.

### K7 — broń w dłoni

- [ ] źródło broni ma SHA/provenance i przechodzi limity;
- [ ] target hand/bone jest jawny, nie globalnie hardcoded;
- [ ] preview pokazuje broń podczas całego klipu;
- [ ] primary attachment jest rigid i ma stabilny local transform;
- [ ] secondary hand jest raportowany jako guide, nie fałszywe IK;
- [ ] binary hierarchy/material/texture/readback są zgodne;
- [ ] UI jawnie mówi, że `MODELTYPE=S` ma baked, niewymienną broń;
- [ ] model, animacje i geometria nie tracą danych.

### K8 — sequence preview

- [ ] idle→attack→idle i phased custom można odtworzyć przed buildem;
- [ ] phase/event markers są zsynchronizowane;
- [ ] transition jump jest raportowany;
- [ ] preview używa tych samych klipów i policy co materializacja.

### K9 — retarget V2 i batch

- [ ] strict V1 pozostaje bez regresji;
- [ ] alias/optional/ignored bones są jawne i wersjonowane;
- [ ] manual mapping wymaga potwierdzenia;
- [ ] incompatible nie mutuje projektu;
- [ ] batch `ALL_OR_NOTHING` przechodzi 9/9 Fogbound;
- [ ] batch i pojedyncze operacje dają identyczne klipy/fingerprinty;
- [ ] pełne donor/target/map/policy provenance jest zachowane.

### K10 — warstwy, krzywe i optymalizacja

- [ ] warstwy Base/Additive/Override i maski mają golden tests;
- [ ] deterministic bake przechodzi native/WASM parity;
- [ ] editor curves nie zmieniają formatu runtime bez evidence;
- [ ] linear resample mieści się w policy error;
- [ ] key reduction raportuje max error i row delta;
- [ ] events/duration/transition są zachowane;
- [ ] finalny binary MDL readback ma `MATCH`.

### K11 — persistence i kompatybilność

- [ ] starsze Studio V1/V2/V3 migrują bezstratnie;
- [ ] nowy dokument round-tripuje przez project JSON, IndexedDB i backup;
- [ ] undo/redo oraz autosave zachowują attachment/layers/policies;
- [ ] donor payload nie trafia do persistence;
- [ ] stare projekty bez nowych pól budują ten sam output.

### K12 — testy i wydajność

- [ ] `cargo fmt --all -- --check`;
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`;
- [ ] `cargo test --workspace`;
- [ ] exact local Void Knight/Fogbound regressions;
- [ ] WASM Node i generated boundary;
- [ ] browser Worker/WASM oraz persistence;
- [ ] Studio typecheck i pełny test suite;
- [ ] accessibility i performance browser gates;
- [ ] production build i wszystkie bundle budgets;
- [ ] animation library catalog/immutability/CI gates;
- [ ] canonical workspace i Meshy asset layout;
- [ ] `git diff --check`.

### K13 — demo i dowód offline

- [ ] demo powstało z normalnego flow aplikacji;
- [ ] source, clip i document identity są związane hashami;
- [ ] MP4 ma zweryfikowany codec/FPS/duration/hash;
- [ ] motion-diversity report potwierdza rzeczywisty ruch;
- [ ] pose parity i quality report są dołączone;
- [ ] source/donor/weapon pozostają byte-identical;
- [ ] status nie udaje Toolset/NWN proofu.

### K14 — owner runtime

- [ ] exact lineage ma kompletny handoff;
- [ ] MOD/HAK zostały przygotowane/umieszczone wyłącznie w autoryzowanym
  zakresie i zgodnie z aktualnymi regułami workspace;
- [ ] właściciel przekazał osobny wynik Toolsetu;
- [ ] właściciel przekazał osobny wynik NWN;
- [ ] `modelVisibility` i `proofCompleteness` są zapisane niezależnie;
- [ ] claim `OWNER_RUNTIME_VERIFIED` dotyczy wyłącznie tego samego lineage.

K14 nie blokuje `OFFLINE_PRODUCT_COMPLETE`, lecz blokuje claim runtime.

## 8. Definition of Done

### Offline Definition of Done

Fazy F0–F9 oraz K0–K13 są ukończone. Użytkownik może w aplikacji utworzyć lub
zaimportować animację, edytować ją prawdziwym gizmo, wykryć i naprawić typowe
problemy, zobaczyć broń w dłoni, przetestować sekwencję, zapisać Custom,
przypisać Base 42 i otrzymać binary MDL z package readback oraz pose parity
`MATCH`. Demo pokazuje rzeczywisty ruch i ma immutable evidence.

### Runtime Definition of Done

K14 jest ukończone na tym samym immutable lineage. Tylko wtedy status może
brzmieć `OWNER_RUNTIME_VERIFIED`.

## 9. Proponowane checkpointy commitów

Commity powstają dopiero na jawne polecenie właściciela, po zielonych bramkach
danego checkpointu.

- [ ] `animation: add canonical authoring commands and pose sampler`
- [ ] `animation: add pose parity and motion quality analysis`
- [ ] `studio: add true bone gizmo and pose tools`
- [ ] `animation: add loop root motion and contact tools`
- [ ] `animation: add baked held weapon preview and export`
- [ ] `animation: add sequence preview and semantic batch retarget`
- [ ] `animation: add layers curves and error-bounded key reduction`
- [ ] `test: close animation authoring v2 offline gates`
- [ ] `docs: prepare exact animation authoring owner handoff`

## 10. Pierwszy vertical slice

Pierwszy użyteczny slice powinien objąć F0–F4, bez broni i retargetu V2:

1. edycja kości prawdziwym gizmo;
2. Core command commit;
3. world-pose sampler;
4. quality report wykrywający static playback i motion spike;
5. binary/package pose parity;
6. MP4 z motion-diversity PASS.

Ten slice usuwa ryzyko ponownego tworzenia animacji „na ślepo” i stanowi
bezpieczną podstawę dla held weapon, loop tools oraz retargetu V2.
