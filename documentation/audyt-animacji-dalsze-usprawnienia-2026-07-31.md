# Audyt animacji: dalsze usprawnienia i nowe funkcje — 2026-07-31

Status: `AUDYT ZAKOŃCZONY / ROADMAPA GOTOWA / NOWE FUNKCJE NIEZAIMPLEMENTOWANE`

Branch/worktree: `animation` / `C:\Projects\meshy2aurora\.worktrees\animation`

Dokument wykonawczy:
[Animation Authoring V2 — plan implementacji i kryteria ukończenia](plan-implementacji-animation-authoring-v2-2026-07-31.md).

## 1. Wniosek

Obecny pipeline animacji jest już technicznie mocny. Potrafi:

- tworzyć i edytować klipy `Custom`;
- płynnie odtwarzać je w viewportcie bez renderowania całego Reacta co klatkę;
- kopiować animację z modelu o identycznym rigu;
- retargetować różny rest pose przy tej samej nazwanej hierarchii;
- używać repozytoryjnej biblioteki presetów z tagami i provenance;
- przeprowadzić klip przez Worker/WASM/Core do binary MDL;
- zablokować Download, gdy writer albo packaged readback nie daje `MATCH`.

Największa luka przesunęła się z eksportu do authoringu i oceny jakości. Studio
jest dziś dobrym edytorem klatek, ale nie jest jeszcze wygodnym narzędziem do
świadomego tworzenia atrakcyjnego ruchu. Użytkownik może zmienić wartości, lecz
aplikacja nie pomaga mu odpowiedzieć na pytania: czy stopy się ślizgają, czy
pętla ma szew, czy root nie dryfuje, czy dłoń naprawdę trzyma broń i czy
viewport pokazuje dokładnie tę samą pozę, którą zapisze writer.

Rekomendowany następny etap to `Animation Authoring V2`, w tej kolejności:

1. jedno źródło prawdy dla operacji edycyjnych i pose parity;
2. automatyczna analiza jakości ruchu;
3. prawdziwy gizmo 3D i narzędzia pozy;
4. pętle, root motion i kontakty z podłożem;
5. trzymanie broni i kontrola chwytu;
6. szerszy retarget i operacje wsadowe;
7. warstwy, krzywe i redukcja klatek.

## 2. Zakres i źródła dowodu

Audyt objął aktualne moduły:

- `crates/m2a-core/src/animation_studio.rs`;
- `crates/m2a-core/src/animation_retarget.rs`;
- `crates/m2a-core/src/animation_library.rs`;
- `crates/m2a-core/src/model_pipeline.rs`;
- `crates/m2a-wasm/src/lib.rs`;
- `apps/studio-web/src/features/animation-editor`;
- `apps/studio-web/src/features/preview`;
- Worker, persistence projektu, Review i testy exact.

Klasyfikacja twierdzeń:

- opis istniejących funkcji jest faktem z aktualnego kodu i testów repo;
- informacja o native Base 42 i formacie eventów pochodzi z lokalnych ustaleń
  Aurora First zapisanych w dokumentacji projektu;
- proponowane funkcje i kolejność są wnioskami implementacyjnymi;
- zachowanie nowej funkcji w Toolset/NWN pozostaje osobnym owner proofem.

Nie użyto internetu ani zewnętrznego kodu/payloadów.

## 3. Stan obecny

| Obszar | Stan | Ocena audytu |
|---|---|---|
| Studio document, autosave, undo/redo | działa | mocna podstawa |
| Keyframe CRUD, trim, retime, events | działa | dobre MVP |
| Płynny playback | działa i ma exact performance gate | nie przebudowywać zegara |
| Biblioteka Built-in/Community/Custom | działa | MVP techniczne ukończone |
| Kopiowanie exact-rig | działa | zachować bez regresji |
| Retarget same-hierarchy | działa na Fogbound → Void Knight | za wąska kompatybilność |
| Custom/Base 42 routing | działa | zachować jeden Core contract |
| Binary MDL writer/readback | działa, `MATCH` jest bramką | mocna podstawa |
| Manipulacja kością w viewportcie | suwak jednego delta axis | niewystarczająca ergonomia |
| Interpolacja authoringu | tylko `LINEAR` | brak kontroli rytmu/easingu |
| Quality analysis | głównie schema/limity | brak oceny ruchu |
| Pętle/root motion | transfer i proste retime | brak narzędzi naprawczych |
| Broń w dłoni | brak kontraktu | luka dla animacji walki |
| Retarget różnych hierarchii | brak | istotne ograniczenie biblioteki |
| Batch import/retarget | UI wybiera jeden klip | niepotrzebna praca ręczna |
| Runtime event semantics | eventy są free-form | brak zamkniętej tabeli Aurory |

## 4. Potwierdzone ograniczenia

### A1 — operacje Core i TypeScript rozwijają się równolegle

Core ma kanoniczne operacje tworzenia, klatek, trim/retime, eventów i
walidacji. `editing.ts` zawiera równoległe operacje potrzebne interaktywnemu UI.
Istnieją golden/parity testy, ale każda nowa zaawansowana funkcja zwiększy
ryzyko rozjazdu.

Wniosek: gest może być podglądany lokalnie, lecz commit dokumentu powinien być
wynikiem jednej wersjonowanej komendy Core.

### A2 — viewport nie ma prawdziwego gizmo 3D

Widoczny „gizmo” jest suwakiem. Dla rotacji zapisuje jedną deltę osi, a dla
translacji jedną deltę kierunku. Nie ma wybierania kości na modelu, osi X/Y/Z,
trybu local/world, manipulatora translate/rotate ani widocznego szkieletu.

Wniosek: obecny interfejs wystarcza do testu kontraktu, ale jest zbyt wolny i
mało intuicyjny do ręcznego tworzenia pełnych animacji.

### A3 — brak obiektywnej kontroli jakości ruchu

Walidator wykrywa m.in. brak kości, niepoprawne czasy, nieunit quaternion,
duplikaty, brak ruchu i limity. Nie mierzy jednak:

- ślizgania stóp;
- penetracji podłoża;
- nagłych skoków prędkości/rotacji;
- szwu pierwsza–ostatnia poza;
- niezamierzonego root drift;
- bezruchu viewportu przy postępie czasu;
- odległości dłoni od chwytu broni;
- różnicy pozy preview względem binary readback.

To dokładnie ta klasa problemów, która wcześniej pozwoliła nagrać film z
poruszającym się czasem, ale praktycznie nieruchomą pozą.

### A4 — retarget V1 wymaga tego samego zestawu nazw i parentów

`build_semantic_rig_mapping_v1` odrzuca brakującą, dodatkową albo inaczej
nazwaną kość oraz zmianę parenta. To bezpieczne i deterministyczne, ale nie
obsłuży dwóch humanoidów różniących się aliasami, twist bones albo opcjonalnymi
kośćmi akcesoriów.

Biblioteka presetów jest jeszcze bardziej restrykcyjna: `STRICT_RIG_V1`
wymaga exact rig signature i świadomie nie uruchamia istniejącego retargetu.

### A5 — authoring ma tylko translację, rotację i LINEAR

To wystarcza do natywnego outputu, lecz rytm ruchu trzeba uzyskiwać ręcznie
przez kolejne klatki. Brakuje easingów, graph editora, tangents, warstw,
maskowania kości i automatycznej redukcji klatek.

Wniosek: krzywe powinny być formatem edycyjnym, a przed MDL deterministycznie
bake'ować się do obecnego liniowego kontraktu. Nie należy zgadywać nowej
semantyki runtime Aurory.

### A6 — porównanie Source/Edited jest przełącznikiem

Nie ma synchronizowanego split view, ghost pose, motion trail ani heatmapy
różnic kości. Przy subtelnych poprawkach użytkownik musi zapamiętywać poprzedni
kadr.

### A7 — import z modelu jest pojedynczy

Dialog przygotowuje i commit'uje jeden wybrany klip. Core potrafi przeprowadzić
wszystkie dziewięć klipów Fogbounda, ale produkt nie ma wygodnego zaznaczenia
wielu klipów, wspólnego raportu i transakcyjnego batch commit.

### A8 — eventy nie mają potwierdzonego słownika gameplay

Format `event <time> <name>` jest znany, lecz lokalny audyt Aurory nie zamknął
pełnej listy znaczeń hit/footstep/sound. UI nie powinno przedstawiać wymyślonej
listy jako standardu NWN. Można dodać własne etykiety authoringowe, ale native
event presets wymagają osobnego evidence.

## 5. Priorytety funkcji

### P0 — bezpieczeństwo authoringu i jakość wyniku

#### P0.1 Kanoniczne komendy edytora

- [ ] `AnimationEditCommandBatchV1` — wersjonowany zestaw operacji;
- [ ] `apply_animation_edit_command_batch_v1` — wykonanie w Core;
- [ ] `reconcile_animation_edit_preview_v1` — porównanie optimistic UI z Core;
- [ ] race guard: source, document revision, clip revision i command identity;
- [ ] jeden commit Core po zakończeniu gestu, bez Worker round-trip co piksel;
- [ ] golden parity Rust/WASM/Worker/Studio dla każdej komendy.

#### P0.2 Pose parity preview → writer → package

- [ ] `sample_animation_world_pose_v1`;
- [ ] `evaluate_preview_binary_pose_parity_v1`;
- [ ] próbki `t=0`, każdy event, wszystkie key times i równomierna siatka;
- [ ] raport różnic translacji i kąta quaternionu per bone;
- [ ] blokada Download dla różnicy przekraczającej wersjonowany epsilon;
- [ ] MP4 proof gate sprawdzający także różnorodność pozy, nie tylko FPS/czas.

#### P0.3 Motion Quality Analyzer

- [ ] `analyze_animation_motion_quality_v1`;
- [ ] `detect_static_playback_v1`;
- [ ] `measure_loop_discontinuity_v1`;
- [ ] `measure_root_drift_v1`;
- [ ] `detect_ground_penetration_v1`;
- [ ] `detect_foot_sliding_v1` po jawnie wybranych kościach kontaktu;
- [ ] `detect_motion_spikes_v1` dla prędkości i prędkości kątowej;
- [ ] raport `INFO | WARNING | BLOCKING` z przedziałem czasu i akcją naprawczą;
- [ ] overlay znaczników problemów na timeline i w viewportcie.

Nie wszystkie odchylenia powinny blokować. Próg blokujący musi dotyczyć
integralności/rozjazdu, a ocena artystyczna domyślnie ma być ostrzeżeniem.

### P1 — wygodne tworzenie dobrej animacji

#### P1.1 Prawdziwy gizmo i szkielet w viewportcie

- [ ] `ViewportBonePicker` — wybór kości z modelu;
- [ ] `SkeletonOverlay` — kości, parenty i zaznaczenie;
- [ ] `ThreeBoneTransformControls` — translate/rotate X/Y/Z;
- [ ] local/world space, snapping i zerowanie delty;
- [ ] `solve_bone_local_transform_from_gizmo_v1`;
- [ ] `insert_pose_keyset_v1` — klucz dla zaznaczonej grupy kości;
- [ ] drag preview bez mutacji, Core commit na pointer-up;
- [ ] pełna obsługa klawiatury i pól numerycznych.

#### P1.2 Narzędzia pozy

- [ ] `copy_animation_pose_v1` / `paste_animation_pose_v1`;
- [ ] `mirror_humanoid_pose_v1` z jawną mapą Left↔Right;
- [ ] `reset_bone_to_rest_v1` i `reset_chain_to_rest_v1`;
- [ ] grupy: whole body, root, spine, left/right arm, left/right leg;
- [ ] zapisywanie własnej pozy w bibliotece projektu;
- [ ] ghost poses previous/next oraz motion trails dłoni i stóp.

#### P1.3 Pętle, kontakty i root motion

- [ ] `inspect_loop_seam_v1`;
- [ ] `blend_animation_loop_seam_v1` z preview before/after;
- [ ] `extract_root_motion_v1`;
- [ ] `lock_root_motion_in_place_v1`;
- [ ] `restore_root_motion_v1` z zachowanym provenance;
- [ ] `suggest_contact_intervals_v1`;
- [ ] `lock_contact_bone_interval_v1` i bake do zwykłych tracków;
- [ ] preset operacji: idle loop, walk/run in-place, one-shot attack.

#### P1.4 Trzymanie broni

Dla obecnego direct creature `MODELTYPE=S` widoczna broń musi być baked do
modelu; zwykłe `Equip_ItemList` nie daje wymiennej widocznej broni. Funkcja ma
więc jawnie rozróżniać `BAKED_DIRECT_CREATURE` od przyszłego wyposażenia
`MODELTYPE=P`.

- [ ] `HeldWeaponAttachmentV1` — źródło, ręka, target bone i local transform;
- [ ] `inspect_held_weapon_source_v1`;
- [ ] `resolve_hand_attachment_target_v1` bez globalnego hardcode nazwy;
- [ ] `compose_held_weapon_preview_v1`;
- [ ] `solve_weapon_grip_offset_v1`;
- [ ] `measure_primary_hand_grip_error_v1`;
- [ ] `measure_secondary_hand_grip_error_v1` jako guide dla broni dwuręcznej;
- [ ] `bake_held_weapon_attachment_v1` jako rigid segment parented do dłoni;
- [ ] shared triangle/material/texture gates i binary hierarchy readback;
- [ ] odtwarzanie każdego przypisanego klipu z widoczną bronią przed buildem.

V1 nie powinno obiecywać pełnego IK dwóch dłoni. Broń ma parent do jednej
głównej dłoni, a druga dłoń dostaje mierzalny guide. Pełne IK można bake'ować w
P2 po udowodnieniu stabilnego solvera.

#### P1.5 Sekwencje i przejścia

- [ ] `AnimationSequencePreviewV1`;
- [ ] preview `idle → attack → idle`, `walk → stop`, phased fire-and-forget;
- [ ] synchronizacja eventów i phase boundaries;
- [ ] raport skoku pozy między klipami;
- [ ] narzędzie blend transition, którego wynik znów bake'uje się do tracków.

### P2 — szerszy zakres i narzędzia profesjonalne

#### P2.1 Retarget V2 i batch

- [ ] `HumanoidSemanticBoneMapV2` z aliasami zatwierdzanymi przez użytkownika;
- [ ] optional bones, twist bones i jawne ignored donor bones;
- [ ] mapowanie chain/root/rest pose zamiast wymogu identycznego parent graph;
- [ ] `inspect_humanoid_retarget_compatibility_v2`;
- [ ] `retarget_animation_clip_humanoid_v2`;
- [ ] `prepare_animation_transfer_batch_v1`;
- [ ] transakcyjny batch commit: wszystko albo zero;
- [ ] konflikt nazw, per-clip preview i wspólny raport 9/9;
- [ ] użycie retargetu przy `Use as template`, ale wyłącznie po jawnym wyborze.

#### P2.2 Warstwy i maski kości

- [ ] `AnimationEditLayerV1` — base, additive i override;
- [ ] `AnimationBoneMaskV1`;
- [ ] warstwa poprawki dłoni/tułowia bez niszczenia donor motion;
- [ ] mute/solo/weight warstwy;
- [ ] `bake_animation_layers_v1` z deterministycznym wynikiem;
- [ ] provenance zachowujące źródło i listę warstw.

#### P2.3 Krzywe i redukcja klatek

- [ ] graph editor dla komponentów translacji i krzywych rotacji;
- [ ] editor-only easing/tangents;
- [ ] `resample_animation_curve_to_linear_v1`;
- [ ] `reduce_animation_keyframes_v1` z tolerancją pozy/kąta;
- [ ] raport before/after: rows, max error, MDL size;
- [ ] brak zmiany eventów, duration i pose fingerprint ponad tolerancję.

#### P2.4 Event presets po Aurora First

- [ ] zamknąć lokalny audyt nazw eventów z dekompilacji/retail;
- [ ] oddzielić `AUTHORING_MARKER` od `AURORA_NATIVE_EVENT`;
- [ ] katalog potwierdzonych nazw z provenance;
- [ ] preset hit/footstep/sound tylko dla nazw potwierdzonych;
- [ ] owner runtime proof eventów niezależny od proofu samego ruchu.

#### P2.5 Biblioteka i dystrybucja

- [ ] signed release manifest;
- [ ] ręczne `Check for updates`;
- [ ] cache, rollback i revoke;
- [ ] warianty presetu na wiele zatwierdzonych rig profiles;
- [ ] quality metrics i wymagany pose-diversity proof w contribution CI.

## 6. Rekomendowany plan etapów

### F0 — kontrakt i regresje

- [ ] zamrozić obecne golden fixtures i exact wyniki;
- [ ] dodać czerwone testy command parity, static playback i pose parity;
- [ ] nie zmieniać istniejących Base 42 ani writer semantics.

### F1 — Core command boundary i pose sampler

- [ ] command batch, canonical result i fingerprint;
- [ ] world-pose sampler;
- [ ] preview/writer/package pose reconciliation;
- [ ] Worker/WASM parity i stale-result rejection.

### F2 — Quality Analyzer

- [ ] syntetyczne fixture każdego problemu;
- [ ] timeline/viewport diagnostics;
- [ ] exact Void Knight i Fogbound corpus;
- [ ] motion-diversity gate dla MP4.

### F3 — gizmo, skeleton i pose tools

- [ ] prawdziwe kontrolki 3D;
- [ ] select/multi-select/copy/paste/mirror/reset;
- [ ] accessibility i 60 FPS interaction gate.

### F4 — loop/root/contact

- [ ] inspect i repair pętli;
- [ ] root motion policies;
- [ ] contact intervals i foot lock;
- [ ] sekwencja preview przed zapisem.

### F5 — held weapon vertical slice

- [ ] jedna broń, prawa dłoń, direct creature baked attachment;
- [ ] transform chwytu i animation preview;
- [ ] binary hierarchy/readback;
- [ ] exact demo z pipeline i owner handoff dopiero po właściwej zgodzie/gate.

### F6 — retarget V2 i batch

- [ ] semantic map z zatwierdzeniem użytkownika;
- [ ] multi-select import i transakcyjny commit;
- [ ] library preset → explicit retarget → Custom;
- [ ] real multi-rig corpus bez kopiowania payloadów do repo.

### F7 — layers/curves/optimization

- [ ] editor-only authoring layer;
- [ ] deterministic linear bake;
- [ ] key reduction z error report;
- [ ] pełny binary MDL readback `MATCH`.

### F8 — runtime i dystrybucja

- [ ] moduł/HAK tylko zgodnie z model-iteration gate;
- [ ] handoff zaczyna się od exact `.mod`, module name i Area;
- [ ] owner-reported Toolset/NWN result;
- [ ] signed library updates jako osobny release gate.

## 7. Kryteria ukończenia następnego milestone'u

`Animation Authoring V2` jest ukończone dopiero, gdy wszystkie poniższe
kryteria są binarnie spełnione:

- [ ] każda trwała edycja ma wynik Core i revision-bound fingerprint;
- [ ] optimistic preview oraz Core commit są zgodne albo UI cofa gest;
- [ ] użytkownik wybiera kość w viewportcie i manipuluje X/Y/Z w local/world;
- [ ] analyzer wykrywa przygotowane fixture: static playback, loop seam, root
  drift, ground penetration, foot slide i motion spike;
- [ ] exact Void Knight przechodzi preview → Core → binary MDL → packaged
  readback z pose parity w ustalonej tolerancji;
- [ ] pętla może zostać zbadana i naprawiona bez zmiany source GLB;
- [ ] root motion może być jawnie zachowany, skalowany albo zamieniony na
  in-place, z pełnym provenance;
- [ ] broń jest widoczna w aplikacji, parented do wybranej dłoni i porusza się
  przez cały klip bez utraty geometrii modelu;
- [ ] batch transfer pokazuje raport per clip i nie zostawia połowy wyniku;
- [ ] viewport zachowuje obecny gate p95 `<=25 ms` desktop i `<=50 ms` przy
  throttle 4× dla exact Void Knighta;
- [ ] source/donor GLB pozostają byte-identical;
- [ ] wszystkie zmiany są local-first i działają offline;
- [ ] cargo fmt, clippy, workspace tests, WASM Node, Worker browser, Studio,
  production build, bundle budgets i `git diff --check` są zielone;
- [ ] demo MP4 powstaje z normalnego flow aplikacji i przechodzi automatyczny
  test różnorodności pozy;
- [ ] claim Toolset/NWN pojawia się wyłącznie po wyniku właściciela dla exact
  immutable lineage.

## 8. Co wdrażać najpierw

Najwyższy zwrot da połączenie `P0.2 + P0.3 + P1.1`: pose parity, quality
analyzer i prawdziwy gizmo. Dzięki temu następna animacja nie będzie tworzona
„na oko” ani oceniana dopiero po eksporcie. Aplikacja pokaże, co faktycznie
zapisze, wskaże konkretne klatki z problemem i pozwoli poprawić je bez
opuszczania viewportu.

Trzymanie broni powinno wejść bezpośrednio po tej podstawie. Bez pose parity i
miernika chwytu można narysować broń przy dłoni, ale nie da się systemowo
udowodnić, że pozostaje w niej przez cały atak.

## 9. Bramki uruchomione podczas audytu

- [x] canonical workspace preflight: PASS;
- [x] canonical Meshy asset layout: PASS;
- [x] Core animation library: 15/15 PASS;
- [x] Core retarget aktywne: 5/5 PASS, 2 exact ignored w zwykłym przebiegu;
- [x] Core Animation Studio: 19/19 PASS;
- [x] Core Animation Studio V5: 8/8 aktywnych PASS;
- [x] exact Void Knight V5: 2/2 PASS;
- [x] exact Fogbound → Void Knight, w tym wszystkie klipy: 2/2 PASS;
- [x] Studio targeted editor/viewport/library: 50/50 PASS;
- [x] katalog biblioteki: 7 presetów, hash zgodny;
- [x] Worker/WASM contract: 20 request types, PASS.

Audyt nie tworzył nowej iteracji modelu, MOD/HAK ani proofu Toolset/NWN.
