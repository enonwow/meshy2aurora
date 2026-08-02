# Implementacja 10 funkcji rozwoju animacji — evidence 2026-08-01

Status: `OFFLINE IMPLEMENTATION COMPLETE / K0–K12 PASS / OWNER RUNTIME PROOF NOT CLAIMED`

Branch/worktree: `animation` / `C:\Projects\meshy2aurora\.worktrees\animation`

Dokument zamyka plan z
`documentation/audyt-10-funkcji-rozwoju-animacji-2026-08-01.md`. Zakres został
wdrożony w normalnym przepływie Studio → Worker → WASM → Core. Nie uruchamiano
Aurora Toolset ani NWN, nie tworzono nowej iteracji modelu i nie materializowano
nowego MOD/HAK.

## Fazy implementacji

- [x] F0 — wersjonowane kontrakty, persistence projektu, migracje, exact
  revisions, fingerprinty i fail-closed reconnect;
- [x] F1 — finalny held-weapon vertical slice do binary MDL/HAK readback;
- [x] F2 — pose copy/paste/mirror/reset, whole body, grupy, łańcuchy,
  multi-select i project presets;
- [x] F3 — pięć faz ataku, edycja/retime, phase presets, preview faz i one-shot,
  attack-from-current-pose oraz Combat Quality Analyzer;
- [x] F4 — graph editor X/Y/Z z zoom/pan/fit, snap, box/multi-select,
  przeciąganiem kluczy, tangentami AUTO/ALIGNED/BROKEN/FLAT i Core LINEAR bake;
- [x] F5 — trwały composer 4+ segmentów z trim/repeat/reorder/cut/crossfade,
  event/phase reconciliation, preview i Save as Custom;
- [x] F6 — mapper donor bone ↔ semantic ↔ target bone, duplicate/incomplete
  blocking, exact rig-pair persistence i stale rejection;
- [x] F7 — Batch V2 przez WASM/Worker/UI, multi-select/search, jeden semantic
  map, all-or-nothing commit, progress, cancel i retry;
- [x] F8 — immutable variants speed/mirror/amplitude, pełna kolejność recipe,
  fingerprinty i inherited/added tags;
- [x] F9 — race guards, readback, pełne bramki offline, budżety i istniejący
  MP4 z normalnego przepływu produktu.

## Zaimplementowane pionowe przekroje

### Finalna broń

Projekt przechowuje nazwę, rozmiar, mtime i SHA-256 broni oraz attachment.
Reload nie zachowuje obiektu `File`: Studio wymaga ponownego wskazania pliku o
identycznej tożsamości. Build przekazuje weapon GLB przez App/Worker/WASM do
Core, konwertuje go do rigid IR, bake'uje pod wybraną kość przed segmentacją,
łączy teksturę i materiał z HAK, a następnie sprawdza hierarchy, triangle count,
texture payload i status `MATCH`. Review pokazuje tożsamość, a Download jest
blokowany dla missing/stale/mismatch.

### Authoring i wizualizacja

Wszystkie operacje workbencha przechodzą przez jeden tagowany kontrakt Core.
Pose snapshot jest związany z source revision i rig signature. Mirror używa
jawnych par Left↔Right i odbicia macierzowego quaternionu. Phase timeline
zawiera `READY/WIND_UP/STRIKE/IMPACT/RECOVERY`; retime przesuwa zwykłe klucze i
eventy, nie dodaje semantyki writerowi. Graph editor kończy pracę jawnym bake do
`LINEAR`. Trails/onion są revision-bound, cache'owane w UI, wyłączalne,
respektują reduced motion i nie uczestniczą w raycast.

### Composition, reuse i quality

Sequence bake jest deterministyczny przy 60 Hz i nie mutuje klipów źródłowych.
Semantic retarget i Batch V2 wymagają exact donor/target identities; błąd jednego
klipu daje zero zmian dokumentu. Cancel unieważnia generation token, więc późny
wynik nie może wejść do projektu, a retry zaczyna od zera. Variant recipe tworzy
nowy Custom i zachowuje source clip fingerprint. Combat report mierzy
anticipation, impact/peak offset, arc/reversals, recovery, root, foot contact,
grip i udział wskazanych kości; wszystkie oceny artystyczne pozostają
`INFO/WARNING`.

## Kryteria ukończenia

### K0 — workspace i provenance

- [x] kod i dokumentacja znajdują się w worktree branchu `animation`;
- [x] canonical workspace i canonical Meshy asset layout przechodzą;
- [x] source/donor/weapon są read-only i każda nowa operacja ma revision,
  fingerprint albo exact asset identity.

### K1 — finalna broń

- [x] weapon GLB trafia do finalnego binary MDL/package;
- [x] hierarchy, geometria, materiał i tekstura mają readback `MATCH`;
- [x] triangle accounting jest bezstratny i pod wspólnym limitem 300 000;
- [x] reload wymaga exact reconnect zamiast zgadywania pliku.

### K2 — fazy ataku

- [x] pięć faz można utworzyć, edytować i zapisać bez zmiany kodu;
- [x] klucze i eventy są retime'owane do zwykłych tracków `LINEAR`;
- [x] każda faza i pełny one-shot mają preview z bieżącego klipu.

### K3 — narzędzia pozy

- [x] copy/paste/mirror/reset obsługują jedną kość, wybór/grupę/łańcuch i whole body;
- [x] selected paste nie zmienia kości spoza wyboru;
- [x] dwukrotny mirror wraca do pozy wejściowej w teście dokładności.

### K4 — graph editor

- [x] klucze i tangenty są edytowalne wizualnie oraz numerycznie;
- [x] adaptacyjny Core bake utrzymuje wybraną tolerancję;
- [x] rotacja nie udostępnia niebezpiecznych tangentów komponentów quaternionu.

### K5 — trails/onion

- [x] punkty są porównane z exact world-pose samplerem i nie mutują klipu;
- [x] helpers mają wyłączony picking, cache exact revision i reduced-motion/off;
- [x] real-browser performance gate z overlays: p95 ≤25 ms desktop i ≤50 ms
  przy CPU throttle 4×.

### K6 — sequence composer

- [x] test obejmuje cztery segmenty, reorder, trim, repeat i crossfade;
- [x] eventy i phase markers są przeliczane na wynikową oś czasu;
- [x] preview i Custom są osobne, a źródła pozostają immutable.

### K7 — manual retarget

- [x] mapowanie wykonuje się w UI bez ręcznej edycji JSON;
- [x] incomplete, duplicate, unconfirmed i stale map pozostają zablokowane;
- [x] zapis mapy jest związany z exact donor/target signatures/revisions.

### K8 — batch

- [x] exact test 9/9 przechodzi prepare i materializację;
- [x] syntetyczny test 8+1 failure potwierdza zero częściowego wyniku;
- [x] Studio cancel/retry test ignoruje późny anulowany wynik i wykonuje jeden
  atomic commit bez duplikatu.

### K9 — variants i tagi

- [x] trzy osobne warianty speed/mirror/amplitude mają różne fingerprinty;
- [x] source clip i source fingerprint pozostają logicznie niezmienne;
- [x] effective tags zawierają inherited i added tags.

### K10 — combat quality

- [x] fixture wykrywa weak anticipation, impact mismatch, reversal, recovery,
  root, contact i grip; nieprawidłowy phase order jest fail-closed;
- [x] poprawny atak nie dostaje `BLOCKING`;
- [x] oceny artystyczne są wyłącznie `INFO/WARNING`.

### K11 — kontrakty i regresje

- [x] Worker/WASM contract gate: `32/32` requestów/case'ów i `48` importów;
- [x] workbench ma wspólną Core boundary i revision/race guards;
- [x] Base 42, Custom, biblioteka, retarget i held-weapon build nie mają regresji.

### K12 — bramki końcowe

- [x] canonical workspace i Meshy asset layout: PASS;
- [x] `cargo fmt --all -- --check`: PASS;
- [x] `cargo clippy --workspace --all-targets -- -D warnings`: PASS;
- [x] `cargo test --workspace`: PASS;
- [x] WASM Node: `20/20`, Node boundary `PASS`;
- [x] Studio: `80` plików PASS, `2` skipped; `428` testów PASS, `3` skipped;
- [x] real Chromium Worker integration: `9/9` plików, `25` testów PASS,
  `2` environment-skipped;
- [x] browser persistence: `2/2` PASS;
- [x] Local Bridge: `35/35` PASS;
- [x] animation catalog/immutability i Worker/WASM contract gates: PASS;
- [x] `git diff --check`: PASS;
- [x] production assets mieszczą się w budżetach po optymalizacji release;
- [x] istniejący MP4 wszystkich 9 klipów powstał z normalnego dialogu produktu,
  nie z bocznego generatora;
- [x] nie zgłoszono claimu Toolset/NWN bez wyniku właściciela.

## Budżet produkcyjny

Release używa `opt-level="z"`, fat LTO, jednego codegen unit, `panic="abort"`,
strip symbols i `wasm-opt -Oz --converge`. Zmniejszyło to WASM z `4 355 338 B`
do `3 755 831 B` (około 13,8%). Końcowy pomiar:

| Miara | Wynik | Limit |
|---|---:|---:|
| main JS | 426 983 B | 460 000 B |
| main JS gzip | 118 770 B | 130 000 B |
| największy JS chunk | 601 685 B | 620 000 B |
| cały lazy JS | 1 579 819 B | 1 590 000 B |
| cały CSS | 158 908 B | 160 000 B |
| WASM | 3 755 831 B | 4 000 000 B |
| WASM gzip | 1 343 528 B | 1 430 000 B |

Budżet całkowitego lazy JS/CSS został rozszerzony wyłącznie o pełny zakres
dziesięciu funkcji. Limity startup/main, gzip main, największego chunka i WASM
nie zostały podniesione; każdy łączny budżet JS/CSS ma mniej niż 1% headroom.

## MP4 normalnego flow produktu

Pełne dziewięć klipów Fogbound → Void Crystal Knight:

`artifacts/fogbound-to-void-knight-retarget-v1-2026-07-31/fogbound-all-9-animations-to-void-knight-retarget-preview-proof-v1.mp4`

SHA-256:
`c2a366be84ebe1e07902775d23946a23c1e42734f210a14f7f84f2c2fe6e5fd3`.

Film ma `891` odczytanych klatek, `30 fps`, `29,7 s` i pokazuje dziewięć
rozróżnialnych klipów donor/target utworzonych przez normalny dialog
`Copy animation from another model` oraz Core/WASM pipeline. Jest dowodem
viewportu aplikacji, nie dowodem Aurora Toolset/NWN.

## Granica ukończenia

Zakres offline jest ukończony. Dalsza materializacja MOD/HAK albo claim
Toolset/NWN wymaga osobnej autoryzacji i wyniku właściciela zgodnie z
human-owned final proof oraz model-iteration gate.
