# Audyt 10 funkcji rozwoju animacji — 2026-08-01

Status: `IMPLEMENTACJA OFFLINE ZAKOŃCZONA / K0–K12 SPEŁNIONE / HUMAN-OWNED TOOLSET/NWN NIEURUCHAMIANE`

Branch/worktree: `animation` / `C:\Projects\meshy2aurora\.worktrees\animation`

> Zamknięcie implementacji 2026-08-01: sekcja 3 pozostaje historycznym
> baseline audytu sprzed wdrożenia. Aktualny stan funkcji, odznaczenie faz
> F0–F9, kryteriów K0–K12, komendy bramek, budżety i dowody znajdują się w
> [raporcie implementacji](evidence/animation-10-functions-implementation-2026-08-01.md).
> Nie utworzono nowego MOD/HAK i nie wykonano claimu Toolset/NWN.

Dokument rozwija bieżący stan `Animation Authoring V2` o dziesięć funkcji
wybranych po ostatniej implementacji. Checkbox oznacza faktycznie dostępny i
zweryfikowany element produktu, a nie sam typ, prototyp Core albo plan.

## 1. Wynik audytu

| # | Funkcja | Stan 2026-08-01 | Najważniejsza luka |
|---|---|---|---|
| 1 | Finalny pipeline broni | `CZĘŚCIOWO` | brak persistence i wejścia broni do finalnego build/package/readback |
| 2 | Kreator faz ataku | `CZĘŚCIOWO` | istnieje jeden stały slash, ale brak ogólnego modelu faz i edytora |
| 3 | Narzędzia pozy | `BRAK` | brak copy/paste/mirror/reset/preset i multi-select kości |
| 4 | Pełny graph editor | `CZĘŚCIOWO` | jest resampling krzywych, ale nie ma wykresu ani edycji tangentów |
| 5 | Motion trails / onion skin | `BRAK` | viewport nie wizualizuje pozy sąsiednich ani trajektorii |
| 6 | Pełny sequence composer | `CZĘŚCIOWO` | jest nietrwały preview `idle → action → idle`, bez montażu i blendów |
| 7 | Ręczny mapper retargetu | `CZĘŚCIOWO` | Core przyjmuje override, UI potrafi tylko pokazać blokadę |
| 8 | Batch import animacji | `CZĘŚCIOWO` | Core V1 działa dla exact/same-hierarchy, brak Worker/UI i semantic V2 |
| 9 | Warianty proceduralne i tagi | `CZĘŚCIOWO` | tagi działają; brak wersjonowanej recepty wariantu |
| 10 | Combat Quality Analyzer | `CZĘŚCIOWO` | działa analyzer techniczny, brak semantyki faz i jakości ataku |

Wniosek: nie należy tworzyć dziesięciu osobnych wysp UI. Funkcje układają się
w cztery wspólne pionowe przekroje:

1. `weapon product`: źródło → preview → zapis projektu → Core bake → package → readback;
2. `authoring`: pose tools → fazy ataku → graph editor → wizualizacja ruchu;
3. `composition`: sekwencje, przejścia i analiza jakości walki;
4. `reuse`: manual retarget → batch → wariant → biblioteka i tagi.

## 2. Zakres i klasyfikacja dowodu

Audyt objął aktualny kod Core, WASM, Worker i Studio, w szczególności:

- `crates/m2a-core/src/held_weapon.rs`;
- `crates/m2a-core/src/animation_authoring_v2.rs`;
- `crates/m2a-core/src/animation_curves.rs`;
- `crates/m2a-core/src/animation_sequence.rs`;
- `crates/m2a-core/src/animation_retarget.rs`;
- `crates/m2a-core/src/animation_library.rs`;
- `crates/m2a-core/src/model_pipeline.rs`;
- `apps/studio-web/src/features/animation-editor`;
- `apps/studio-web/src/features/animation-library`;
- kontrakt Workera/WASM i testy odpowiadających modułów.

Klasyfikacja:

- opis istniejącego kodu i testów: fakt z repozytorium;
- wymagania binary MDL, direct creature i owner proof: istniejąca decyzja projektu;
- proponowane API, UX i kolejność: wniosek implementacyjny;
- jakość artystyczna ruchu: metryka pomocnicza, nie fakt Aurory;
- runtime Toolset/NWN: nie jest objęty tym offline audytem i pozostaje human-owned.

Audyt nie tworzył modelu, MOD/HAK, nowego resrefu ani iteracji proofowej.

## 3. Audyt funkcji

### 3.1 Finalny pipeline broni — `CZĘŚCIOWO`

Istnieje:

- [x] inspekcja sztywnego GLB broni z SHA-256, geometrią i materiałami;
- [x] kontrakt `HeldWeaponAttachmentV1`, wybór ręki/kości i local transform;
- [x] pomiar błędu głównego oraz pomocniczego chwytu;
- [x] deterministyczny `bake_held_weapon_attachment_v1` na poziomie `AuroraModelIrV1`;
- [x] ochrona sumy trójkątów i test braku utraty geometrii;
- [x] preview broni przy kości w Animation Studio.

Brakuje:

- [ ] trwałego attachmentu w schemacie projektu i migracji;
- [ ] bezpiecznego przechowywania/reconnect exact pliku broni po reloadzie;
- [ ] publicznej ścieżki GLB broni → rigid IR → creature product;
- [ ] scalenia materiałów i tekstur broni z finalnym pakietem;
- [ ] segmentacji po granicy jednego strumienia writera;
- [ ] hierarchy/material/texture readback po zapisaniu binary MDL;
- [ ] przekazania attachmentu przez App → Worker → WASM → Core build;
- [ ] Review pokazującego tożsamość broni i status `MATCH`;
- [ ] blokady Download dla stale/missing/mismatch weapon payload.

Ryzyko: `File` trzymany tylko w stanie Reacta nie jest trwałą tożsamością
projektu. Nie wolno po reloadzie udawać, że broń nadal jest podłączona. Projekt
powinien zachować metadane i wymagać reconnectu pliku o identycznym SHA-256 albo
użyć jawnie wersjonowanego local blob store.

Kryterium lokalne tej funkcji: exact model + exact broń przechodzą jednym
normalnym buildem aplikacji do binary MDL i package readback `MATCH`, bez utraty
trójkątów, materiałów, UV, tekstur, animacji albo source identity.

### 3.2 Kreator faz ataku — `CZĘŚCIOWO`

Istnieje:

- [x] proceduralny `HUMANOID_SWORD_SLASH` z siedmioma punktami czasu;
- [x] czytelna antycypacja, ruch ataku, impact i recovery w jednym konkretnym presetcie;
- [x] testy wymaganych kości, amplitudy łuku i ruchu bioder;
- [x] osobny kreator Aurora `START/LOOP/END` dla phased Custom.

To nie jest jeszcze ogólny kreator faz ataku. `START/LOOP/END` opisuje sposób
odtwarzania phased Custom, a nie semantykę pojedynczego ciosu. Stała tablica
czasów slashu nie pozwala zbudować innego ataku bez pisania kodu.

Docelowy kontrakt:

- [ ] `CombatAttackPhaseTimelineV1` z fazami `READY`, `WIND_UP`, `STRIKE`,
  `IMPACT`, `RECOVERY`;
- [ ] czasy faz ściśle rosnące, revision-bound i mieszczące się w klipie;
- [ ] marker `IMPACT` powiązany z authoring markerem, a nie udawanym eventem Aurory;
- [ ] tworzenie klipu od rest/current/source pose;
- [ ] edycja czasu faz przez timeline bez utraty klatek i eventów;
- [ ] opcjonalne szablony: punch, slash, overhead, thrust;
- [ ] bake faz do zwykłych tracków `LINEAR`; writer nie dostaje nowej semantyki;
- [ ] preview każdej fazy oraz pełnego one-shotu.

Kryterium lokalne: użytkownik tworzy nowy atak bez zmiany kodu, przesuwa pięć
faz, widzi wynik w viewportcie, a zapisany klip ma tę samą pozę po Core i
packaged readback w ustalonej tolerancji.

### 3.3 Narzędzia pozy — `BRAK`

Stan: Studio ma wybór jednej kości i prosty transform. Nie znaleziono kontraktu
copy/paste/mirror/reset ani wielokrotnego wyboru kości. Biblioteka przechowuje
animacje, nie pozy użytkownika.

Do wdrożenia:

- [ ] `AnimationPoseSnapshotV1` związany z source revision, rig signature i czasem;
- [ ] `copy_animation_pose_v1` i `paste_animation_pose_v1`;
- [ ] paste modes: whole body, selected bones, named chain;
- [ ] `HumanoidMirrorMapV1` z jawnymi parami Left↔Right i osią odbicia;
- [ ] `mirror_humanoid_pose_v1` bez zgadywania niejednoznacznych kości;
- [ ] `reset_bone_to_rest_v1` oraz `reset_chain_to_rest_v1`;
- [ ] multi-select w bone tree, viewportcie i dope sheet;
- [ ] zapis local pose preset w projekcie;
- [ ] komendy Core, undo/redo, race guards i optimistic-preview reconciliation;
- [ ] pełna obsługa klawiatury i pól numerycznych.

Ryzyko: mirror quaternionów nie może polegać na negacji przypadkowego
komponentu. Transform trzeba odbić jako macierz w jawnie wybranej przestrzeni,
a potem zdekomponować i znormalizować.

Kryterium lokalne: skopiowana, wklejona, odbita i zresetowana poza ma
deterministyczny fingerprint, zachowuje niezaznaczone kości i przechodzi parity
TypeScript preview ↔ Core commit.

### 3.4 Pełny graph editor — `CZĘŚCIOWO`

Istnieje:

- [x] editor-only curve track;
- [x] tangenty Hermite dla translacji;
- [x] shortest-arc interpolacja quaternionów bez niebezpiecznych tangentów komponentów;
- [x] adaptacyjny, ograniczony błędem resampling do `LINEAR`;
- [x] panel „Smooth selected track” i testy Core/Worker/Studio.

Brakuje:

- [ ] wykresu time/value z zoom, pan i fit selection;
- [ ] osobnych kanałów X/Y/Z i bezpiecznej reprezentacji rotacji;
- [ ] wyboru wielu kluczy, box select i snap;
- [ ] uchwytów tangentów, broken/aligned/auto/flat;
- [ ] bezpośredniej edycji czasu i wartości;
- [ ] podglądu krzywej przed bake oraz raportu błędu na wykresie;
- [ ] trwałego zapisu editor curve albo jawnego bake przy zamknięciu;
- [ ] spójnego undo/redo i command batch.

Decyzja: V1 graph editora powinien edytować translację i skalarne kanały
diagnostyczne. Rotacja pozostaje reprezentowana jako bezpieczny quaternion
shortest-arc; nie pokazujemy czterech niezależnych kanałów quaternionu jako
czterech swobodnych krzywych.

Kryterium lokalne: użytkownik zmienia krzywą wizualnie, Core bake zachowuje
błąd poniżej wybranej tolerancji, a eventy, duration i niezaznaczone tracki są
identyczne.

### 3.5 Motion trails i onion skin — `BRAK`

Istniejący world-pose sampler daje dobry fundament, ale viewport nie rysuje
trajektorii ani ghost poses.

Do wdrożenia:

- [ ] `MotionTrailSampleRequestV1` i deterministyczne próbkowanie wybranych kości;
- [ ] trail dłoni, stóp, broni i opcjonalnie root;
- [ ] onion skin dla N poprzednich/następnych próbek;
- [ ] kolory previous/current/next i czytelna legenda;
- [ ] ograniczenie liczby punktów/ghostów i adaptacja do zoomu czasu;
- [ ] cache związany z clip revision, rig i sampling policy;
- [ ] wyłączenie pickingu dla obiektów pomocniczych;
- [ ] reduced-motion oraz możliwość całkowitego wyłączenia;
- [ ] performance gate bez alokacji React per frame.

Kryterium lokalne: trail aktualizuje się po edycji bez stale result, nie zmienia
dokumentu, nie przechwytuje wyboru kości i zachowuje p95 playbacku w obecnym
budżecie.

### 3.6 Pełny sequence composer — `CZĘŚCIOWO`

Istnieje:

- [x] Core łączy 2..=16 klipów w nietrwały preview;
- [x] eventy są przesuwane razem z klipami;
- [x] raportowane są skoki translacji i rotacji na granicach;
- [x] UI buduje jeden wariant `idle → action → idle`.

Brakuje:

- [ ] trwałego `AnimationSequenceDocumentV1` w projekcie;
- [ ] dodawania, usuwania i re-order segmentów;
- [ ] trim/offset/repeat per segment;
- [ ] `walk → stop`, chained combo i phased preview;
- [ ] overlap/crossfade/blend transition z deterministycznym bake;
- [ ] widoku wielu klipów na jednej osi czasu;
- [ ] konfliktów eventów i phase boundaries;
- [ ] zapisu wyniku jako nowy Custom albo pozostawienia jako preview-only;
- [ ] provenance wszystkich źródeł i transition recipe.

Kryterium lokalne: użytkownik układa co najmniej cztery segmenty, zmienia ich
kolejność i przejścia, odtwarza całość płynnie oraz zapisuje wynik jako nowy
Custom bez mutowania klipów źródłowych.

### 3.7 Ręczny mapper retargetu — `CZĘŚCIOWO`

Istnieje:

- [x] `HumanoidSemanticOverrideV2` w Core;
- [x] wersjonowany słownik aliasów i required/optional semantics;
- [x] status `MANUAL_CONFIRMATION_REQUIRED`;
- [x] potwierdzenie manual mapping jest częścią fingerprintu;
- [x] retarget fail-closed dla niepotwierdzonej lub stale mapy;
- [x] Studio pokazuje diagnostykę i blokuje niejednoznaczny import.

Brakuje:

- [ ] UI donor bone ↔ semantic ↔ target bone;
- [ ] filtrowania kości już użytych i ostrzeżeń o duplikatach;
- [ ] wizualnego podświetlenia obu rigów;
- [ ] jawnego ignore dla optional/twist/accessory bones;
- [ ] zapisu mapy dla pary rig signatures;
- [ ] ponownego potwierdzenia po zmianie któregokolwiek SHA/revision;
- [ ] preview pojedynczej pozy i ruchu przed importem;
- [ ] przekazania overrides przez pełny App/Worker/WASM flow.

Kryterium lokalne: niejednoznaczny rig można zmapować bez edycji JSON, a każda
niepełna, duplikująca albo stale mapa pozostaje zablokowana. Zapisana mapa nie
jest automatycznie używana dla innej pary rig signatures.

### 3.8 Batch import animacji — `CZĘŚCIOWO`

Istnieje:

- [x] `prepare_animation_transfer_batch_v1` w Core;
- [x] limit 1..=256 i unikalność ID/nazw;
- [x] exact donor/target source revisions;
- [x] `ALL_OR_NOTHING` i wspólny fingerprint;
- [x] offline test pełnego zestawu klipów dla zgodnej hierarchii.

Ograniczenie: batch V1 korzysta z compatibility exact/same-hierarchy. Nie jest
jeszcze ścieżką semantic V2. Nie ma też eksportu WASM/Worker ani produktu UI.

Do wdrożenia:

- [ ] `PREPARE_ANIMATION_TRANSFER_BATCH_V2` przez WASM i Worker;
- [ ] multi-select, select all/none i wyszukiwanie donor clips;
- [ ] reguła nazewnictwa i rozwiązywanie konfliktów przed prepare;
- [ ] per-clip status, diagnostyka i preview;
- [ ] jedna zatwierdzona semantic map dla całego batcha;
- [ ] atomic commit dokumentu Studio albo zero zmian;
- [ ] progress i bezpieczne cancel/retry bez częściowego commit;
- [ ] raport 9/9 lub jawna lista blokad, bez ukrywania błędu jednego klipu.

Kryterium lokalne: exact dziewięć klipów można zaznaczyć, przygotować,
przejrzeć i zatwierdzić jednym flow; wymuszony błąd jednego klipu zostawia
dokument byte-identical względem stanu sprzed commit.

### 3.9 Warianty proceduralne i tagi — `CZĘŚCIOWO`

Tagi są wdrożone:

- [x] stabilny słownik 17 tagów;
- [x] tagi w manifestach biblioteki;
- [x] wyszukiwanie po tagu, autorze, etykiecie i ID;
- [x] łączone filtry tagów w UI;
- [x] walidacja kontribucji i katalogu.

Wariantów proceduralnych nie ma. Istnieje jeden kodowany template slash i
siedem gotowych presetów, ale nie ma wersjonowanej recepty „utwórz wariant”.

Docelowy kontrakt:

- [ ] `AnimationVariantRecipeV1` z source clip/preset fingerprint;
- [ ] operacje V1: retime/speed, mirror przez zatwierdzoną mapę, amplitude
  delta-from-rest, root-motion policy i optional bone mask;
- [ ] preview before/after i quality report;
- [ ] wariant zawsze tworzy nowy Custom, nigdy nie mutuje Built-in/Community;
- [ ] provenance przechowuje pełną kolejność i parametry operacji;
- [ ] dziedziczenie tagów plus jawne tagi wariantu;
- [ ] możliwość eksportu wariantu jako contribution z nowym motion hash;
- [ ] nazwy/ID deterministyczne i bez konfliktu resref.

Ryzyko: „strength 150%” nie może mnożyć komponentów quaternionu. Należy
skalować rotacyjną deltę od rest pose przez shortest-arc interpolation.

Kryterium lokalne: z jednego immutable presetu powstają co najmniej trzy
odróżnialne Custom variants, każdy z własnym fingerprintem, tagami i pełnym
provenance; preset źródłowy i jego hash pozostają identyczne.

### 3.10 Combat Quality Analyzer — `CZĘŚCIOWO`

Istniejący analyzer techniczny wykrywa:

- [x] static playback;
- [x] loop discontinuity;
- [x] root drift;
- [x] ground penetration;
- [x] foot sliding;
- [x] motion spike;
- [x] distinct pose count i skok do czasu problemu;
- [x] narzędzia repair dla loop/root/contact/key reduction.

Nie ocenia jeszcze semantyki ataku. Do wdrożenia:

- [ ] `CombatQualityContextV1`: semantic rig map, attack phases, side/weapon;
- [ ] kontrola kolejności i minimalnego czasu pięciu faz;
- [ ] miara anticipation displacement względem `READY`;
- [ ] peak hand/weapon speed i jego odległość czasowa od `IMPACT`;
- [ ] attack arc length oraz nagłe odwrócenia kierunku;
- [ ] udział bioder/tułowia/ramienia jako raport, nie arbitralna ocena stylu;
- [ ] recovery distance do pozy docelowej;
- [ ] guard-hand i secondary-grip error, gdy mają zastosowanie;
- [ ] stabilność root/foot contact podczas impact;
- [ ] timeline markers i overlay problematycznego łańcucha;
- [ ] severity domyślnie `INFO/WARNING`; `BLOCKING` tylko dla integralności,
  stale identity albo niemożliwej struktury faz.

Kryterium lokalne: syntetyczne fixture osobno wywołują każdy rodzaj problemu,
a poprawny atak przechodzi bez fałszywych `BLOCKING`. Analyzer nie przedstawia
subiektywnej oceny stylu jako standardu Aurory/NWN.

## 4. Zależności i kolejność

```text
Final weapon product ───────────────┐
                                    ├─> Combat Quality Analyzer
Pose tools ─> Attack phases ────────┤
    └───────> Graph editor          ├─> Sequence composer
             └─> Motion trails ─────┘

Manual retarget mapper ─> Batch import ─> Procedural variants ─> Library/tags
```

Najważniejsze zależności:

- analyzer walki potrzebuje jawnych faz, nie powinien zgadywać ich z samego ruchu;
- batch semantic V2 potrzebuje zatwierdzonego ręcznego mappera;
- wariant mirror potrzebuje tej samej bezpiecznej mapy Left↔Right co pose tools;
- graph editor i trails powinny korzystać z obecnego world-pose/curve Core;
- finalny weapon build może ruszyć niezależnie, bo inspekcja, preview i IR bake już istnieją.

## 5. Plan implementacji

### F0 — kontrakty, baseline i migracje

- [ ] zamrozić aktualne testy i fingerprinty funkcji częściowych;
- [ ] dodać wersjonowane schema dla attachmentu, pose, phase, sequence i variant recipe;
- [ ] zdefiniować migracje projektu i fail-closed reconnect local assets;
- [ ] spisać progi wydajności i tolerancje parity przed implementacją;
- [ ] dodać czerwone testy dla każdego kryterium K0–K12.

### F1 — finalny weapon product vertical slice

- [ ] persistence attachmentu i exact asset identity;
- [ ] GLB broni → rigid IR → `bake_held_weapon_attachment_v1` w product builderze;
- [ ] merge materiałów/tekstur i bezstratna segmentacja;
- [ ] App/Worker/WASM build contract;
- [ ] binary/package readback i Review/Download gate;
- [ ] syntetyczny exact test bez tworzenia MOD/HAK.

### F2 — pose tools i bezpieczny mirror

- [ ] pose snapshot, copy/paste/reset;
- [ ] wersjonowana humanoid mirror map;
- [ ] multi-select kości i grupy/łańcuchy;
- [ ] local pose presets;
- [ ] command parity, undo/redo i accessibility.

### F3 — kreator faz ataku i combat analyzer

- [ ] phase timeline i markery `READY/WIND_UP/STRIKE/IMPACT/RECOVERY`;
- [ ] atak od current/source pose;
- [ ] metryki łuku, prędkości, impact, recovery, root/contact i grip;
- [ ] timeline/viewport diagnostics;
- [ ] fixture dobrych i celowo wadliwych ataków.

### F4 — graph editor i wizualizacja ruchu

- [ ] canvas/SVG graph z kanałami translacji;
- [ ] edycja tangentów i multi-key selection;
- [ ] bezpieczny quaternion preview;
- [ ] trails i onion skin z cache revision-bound;
- [ ] error overlay resamplingu i performance gate.

### F5 — sequence composer

- [ ] trwały sequence document i UI wielosegmentowe;
- [ ] reorder, trim, offset, repeat i transitions;
- [ ] event/phase reconciliation;
- [ ] preview-only oraz bake-to-Custom;
- [ ] test `idle → attack A → attack B → recovery` i `walk → stop`.

### F6 — manual retarget mapper

- [ ] dual-rig mapping UI i semantic table;
- [ ] overrides/ignore/confirmation;
- [ ] zapis mapy związany z dwoma rig signatures;
- [ ] pose/motion preview i stale-map rejection;
- [ ] pełna granica App/Worker/WASM/Core.

### F7 — batch import V2

- [ ] multi-select i plan nazw;
- [ ] exact/same-hierarchy/semantic V2 w jednym raporcie;
- [ ] transactional prepare/preview/commit;
- [ ] progress/cancel/retry bez częściowych wyników;
- [ ] exact 9/9 test oraz test rollback 8+1 failure.

### F8 — procedural variants i biblioteka

- [ ] variant recipe i deterministyczny operator stack;
- [ ] speed/mirror/amplitude/root/mask;
- [ ] tag inheritance i variant search;
- [ ] export contribution z nowym motion hash;
- [ ] immutability presetów źródłowych.

### F9 — integracja produktu i dowody offline

- [ ] wspólne revision/race guards dla wszystkich nowych operacji;
- [ ] packaged pose/material/texture/readback `MATCH`;
- [ ] pełne testy Rust/WASM/Worker/Studio/browser/build/budgets;
- [ ] MP4 z normalnego flow aplikacji z mierzalną różnorodnością pozy;
- [ ] dokument evidence i owner handoff tylko dla zamrożonego exact lineage;
- [ ] MOD/HAK wyłącznie po osobnej autoryzacji i zgodnie z model-iteration gate.

## 6. Kryteria ukończenia całego zakresu

### K0 — workspace i provenance

- [ ] wszystkie zmiany są w zatwierdzonym worktree branchu `animation`;
- [ ] source/donor/weapon GLB pozostają byte-identical;
- [ ] każda operacja ma source revision, fingerprint i jawne provenance.

### K1 — finalna broń

- [ ] broń wybrana w Studio trafia do finalnego binary MDL/package;
- [ ] hierarchy, geometria, UV, materiał i tekstury przechodzą readback `MATCH`;
- [ ] suma trójkątów jest zachowana i respektuje wspólny limit 300 000;
- [ ] reload wymaga identycznego assetu albo zachowanego local blob, nigdy zgadywania.

### K2 — fazy ataku

- [ ] pięć faz można utworzyć i edytować bez zmiany kodu;
- [ ] phase markers są zachowane w authoringu i prawidłowo bake'owane;
- [ ] impact preview, Core commit i package pose są zgodne.

### K3 — narzędzia pozy

- [ ] copy/paste/mirror/reset działa dla jednej kości, grupy i całego ciała;
- [ ] multi-select nie zmienia kości spoza wyboru;
- [ ] mirror jest deterministyczny i dwukrotne mirror wraca do pozy wejściowej
  w ustalonej tolerancji.

### K4 — graph editor

- [ ] klucze i tangenty można edytować wizualnie;
- [ ] bake do LINEAR nie przekracza wybranej tolerancji;
- [ ] eventy, duration i niezaznaczone tracki nie zmieniają się.

### K5 — trails/onion

- [ ] trajektorie i ghost poses zgadzają się z world-pose samplerem;
- [ ] nie mutują dokumentu i nie przechwytują pickingu;
- [ ] viewport utrzymuje dotychczasowy gate p95 `<=25 ms` desktop i `<=50 ms`
  przy throttle 4× dla exact Void Knighta.

### K6 — sequence composer

- [ ] co najmniej cztery dowolne segmenty można ułożyć, przestawić i zblendować;
- [ ] eventy i fazy mają prawidłowe czasy po montażu;
- [ ] zapis do Custom nie mutuje klipów źródłowych.

### K7 — manual retarget

- [ ] niejednoznaczne mapowanie można zakończyć w UI bez ręcznego JSON;
- [ ] incomplete/duplicate/stale map jest blokowana;
- [ ] mapy są związane z exact donor/target rig signatures.

### K8 — batch

- [ ] zestaw 9/9 przechodzi prepare/preview/commit jednym flow;
- [ ] jeden wadliwy klip powoduje zero zmian dokumentu;
- [ ] cancel/retry nie tworzy duplikatów ani częściowego stanu.

### K9 — variants i tagi

- [ ] warianty speed/mirror/amplitude mają nowe fingerprinty i pełne recipe;
- [ ] źródłowy preset, manifest i motion hash pozostają identyczne;
- [ ] wariant można znaleźć po odziedziczonych i dodanych tagach.

### K10 — combat quality

- [ ] fixture wykrywają phase order, słabą antycypację, rozjazd impact,
  odwrócenie łuku, złą recovery, niestabilny root/contact i grip error;
- [ ] poprawny atak nie dostaje fałszywego `BLOCKING`;
- [ ] ocena artystyczna pozostaje `INFO/WARNING` i nie udaje standardu Aurory.

### K11 — kontrakty i regresje

- [ ] każdy nowy request jest obecny w Core/WASM/Worker contract gate;
- [ ] optimistic UI i Core commit mają parity albo gest jest cofany;
- [ ] race guard odrzuca stale source/document/clip/asset result;
- [ ] istniejące Base 42, Custom, biblioteka i retarget nie mają regresji.

### K12 — bramki końcowe

- [ ] `cargo fmt --all -- --check`;
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`;
- [ ] `cargo test --workspace`;
- [ ] WASM Node i Worker browser integration;
- [ ] pełne testy Studio i browser persistence;
- [ ] production build i bundle budgets;
- [ ] animation catalog/immutability gates;
- [ ] `git diff --check`;
- [ ] MP4 powstaje z normalnego flow produktu, nie z bocznego generatora;
- [ ] claim Toolset/NWN dopiero po wyniku właściciela dla exact lineage.

## 7. Rekomendowany priorytet

Kolejność o najwyższym zwrocie:

1. `F1 finalny weapon product` — zamyka największą różnicę między preview a eksportem;
2. `F2 pose tools` — przyspiesza każdą kolejną animację;
3. `F3 attack phases + combat analyzer` — zamienia tworzenie ataku z „na oko” w iteracyjny proces;
4. `F4 graph + trails` — daje kontrolę rytmu i czytelność ruchu;
5. `F6 manual mapper → F7 batch` — odblokowuje realne ponowne użycie biblioteki;
6. `F5 sequence composer` — składa działające klipy w zachowania;
7. `F8 variants` — skaluje bibliotekę dopiero po bezpiecznym mirrorze i retargecie.

Nie rekomenduje się zaczynania od wariantów proceduralnych. Bez pose tools,
bezpiecznego mirroru i quality report łatwo wygenerować wiele technicznie
poprawnych, ale słabych lub uszkodzonych klipów.

## 8. Definition of Done

Cały zakres jest `DONE` dopiero wtedy, gdy K0–K12 są zaznaczone, wszystkie
dziesięć funkcji jest dostępne z normalnego flow aplikacji local-first, finalny
build przechodzi przez Worker/WASM/Core do własnego binary MDL/package, a
Review pokazuje exact lineage i readback. Zielony unit test samego Core nie
oznacza ukończenia funkcji produktu.
