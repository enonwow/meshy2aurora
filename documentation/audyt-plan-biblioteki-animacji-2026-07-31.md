# Audyt, plan implementacji i kryteria ukończenia biblioteki animacji — 2026-07-31

Status: `IMPLEMENTED / OFFLINE GATES PASS / OWNER GATES OPEN`

Stan wykonania, dowody dla faz F0–F8 i kryteriów K0–K10 oraz jawne
ograniczenia właścicielskie są prowadzone w
[`animation-library-implementation-evidence-2026-07-31.md`](animation-library-implementation-evidence-2026-07-31.md).
Migrację kontraktu dokumentu Animation Studio V1→V2 opisuje
[`animation-studio-library-provenance-migration-v2-2026-07-31.md`](animation-studio-library-provenance-migration-v2-2026-07-31.md).

Dokument uszczegóławia zaakceptowany kierunek z
[`community-animation-library-2026-07-31.md`](community-animation-library-2026-07-31.md).
Jest wykonawczym kontraktem implementacji repozytoryjnej biblioteki animacji,
tagów, contribution przez PR oraz integracji `Built-in` / `Community` /
`Custom`.

## 1. Wynik audytu

Biblioteka jest wykonalna bez przebudowy writera MDL i bez zmiany podstawowego
pipeline'u modelu. Obecna aplikacja ma już działający editor, lokalny zapis,
walidację, Worker/WASM, materializację i readback. Nie ma natomiast
przenośnego kontraktu presetu, metadanych katalogowych, tagów, repozytoryjnego
generatora ani eksportu contribution.

Najważniejszy wniosek: obecny `AuthoredAnimationClipV1` nie może być formatem
repozytoryjnego presetu. Jest związany z SHA-256 dokładnego źródłowego GLB oraz
numerycznymi `targetNodeId` jego rigu. Biblioteka potrzebuje osobnego formatu
przenośnego, którego ścieżki wskazują stabilne nazwy/role kości. Dopiero
kontrolowana operacja `Use as template` wiąże preset z aktualnym GLB i tworzy
zwykły, lokalny klip `Custom`.

### Ocena gotowości

| Obszar | Stan | Ocena |
|---|---|---|
| Edycja animacji | klatki, wydarzenia, timeline, undo/redo i preview działają | `READY` |
| Lokalna biblioteka projektu | `Save to Custom`, duplikacja i mapowanie Base 42 działają | `READY` |
| Persistence | dokument jest autosave'owany w IndexedDB i w backupie projektu | `READY` |
| Core/WASM/Worker | ścisła walidacja, materializacja oraz readback `MATCH` działają | `READY` |
| Import donor modelu | działa tylko dla dokładnie zgodnego rigu | `PARTIAL` |
| Repozytoryjne presety | brak katalogu i przenośnego payloadu | `MISSING` |
| Tagi i filtry | brak pól oraz UI | `MISSING` |
| Contribution export | brak funkcji | `MISSING` |
| Contribution CI | ogólne CI istnieje, brak bramek paczki animacji | `MISSING` |
| Licencja contribution | brak licencji root i zaakceptowanej polityki | `BLOCKED_BY_OWNER_DECISION` |
| Aktualizacje sieciowe | brak; produkt pozostaje offline | `DEFERRED` |

Wniosek końcowy: nie ma blokera technicznego dla biblioteki wbudowanej w
wydanie. Publiczne przyjmowanie PR-ów jest zablokowane do czasu jawnej decyzji
o licencji repozytorium i licencji animacji contributorów.

## 2. Dowody ze stanu aplikacji

### 2.1 Mocne fundamenty

- `AnimationStudioDocumentV1` zapisuje dokładny `sourceRevision`, rewizję,
  status oraz pełne authored clips.
- Core używa `deny_unknown_fields`, waliduje unikalność ID/nazw, limity czasu,
  wartości, quaterniony, tracki, wydarzenia oraz wykrywa brak rzeczywistego
  ruchu.
- Limity produktu są jawne: 64 authored clips, 10 000 klatek na klip, 100 000
  klatek w dokumencie i 65 535 wierszy MDL.
- Nazwa wyjściowa animacji Aurory jest ograniczona do 1–16 znaków ASCII
  `[A-Za-z0-9_]`.
- `Save to Custom` tworzy dokładną definicję Custom z authored clip ID i
  provenance. Klip `VALID`, który nie ma użycia Custom, blokuje build zamiast
  znikać z paczki.
- Arbitrary Custom jest zasobem bibliotecznym. Runtime NWN jest deklarowany
  dopiero przez routing do standardowego slotu Base 42.
- Persistence używa IndexedDB; projekt zachowuje dokument Animation Studio i
  rozróżnia błędy recovery.
- Import z innego GLB inspektuje donor w pamięci, zapisuje hash i kopiuje tylko
  tracki/provenance. Nie przechowuje GLB.
- Obecne testy Core z audytu: `19/19` Animation Studio i `8/8` aktywnych V5;
  dwa testy exact GLB są jawnie ignored bez lokalnego assetu.
- Obecne testy Studio z audytu: `40/40` dla editing, import, schema i
  persistence; generation/contract preflight jest zielony.

### 2.2 Luki potwierdzone kodem

1. `NewAnimationMenu.tsx` ma presety wpisane w kod (`Root motion pulse`,
   `Void crystal cleave`).
2. `AnimationClipLibrary.tsx` ma wyłącznie zakładki `Base 42` i `Custom`.
3. `filterAnimationStudioLibraryV1()` wyszukuje wyłącznie po nazwie; item nie
   ma autora, tagów, licencji, wersji ani preset ID.
4. `AuthoredAnimationSourceV1` rozróżnia blank/source/import/procedural, ale nie
   ma provenance repozytoryjnego presetu.
5. `AuthoredAnimationTrackV1` wskazuje `targetNodeId`, więc jego JSON nie jest
   niezależny od modelu.
6. `compareAnimationImportRigsV1()` wymaga dokładnej zgodności ID, nazw,
   rodziców i rest transforms. Komunikat jawnie mówi, że automatyczny retarget
   nie istnieje.
7. Istniejący generator katalogu dotyczy 42 standardowych stanów Aurory.
   `meshy-animation-catalog-v1.json` jest osobnym snapshotem zewnętrznych
   akcji Meshy. Żaden z nich nie jest biblioteką authored presets.
8. Brak `animation-library`, schema contribution, generatora indeksu,
   manifestu plików oraz PR template.
9. Brak funkcji `Use as template` i `Export contribution`.
10. Brak root `LICENSE`; istniejące reguły projektu wymagają decyzji
    licencyjnej przed publicznym wydaniem i zabraniają kopiowania retail/CEP
    oraz zewnętrznych animacji bez prawa do eksportu.
11. Budżet głównego JS wynosi 460 000 B i jest ciasny. Payloadów ruchu nie
    wolno dołączyć do initial chunk; muszą być ładowane leniwie.
12. Obecne CI uruchamia Core, WASM, Studio i integration tests, ale nie ma
    negatywnych bramek contribution, kontroli SPDX ani dozwolonych typów
    plików.

## 3. Priorytety problemów

| ID | Priorytet | Problem | Skutek bez poprawki |
|---|---|---|---|
| AL-P0-01 | P0 | brak przenośnego formatu presetu | animacja działa tylko w projekcie źródłowym |
| AL-P0-02 | P0 | brak bezpiecznego bindu preset → aktualny rig | błędne kości albo cicha deformacja |
| AL-P0-03 | P0 | brak decyzji licencyjnej | nie wolno przyjmować publicznych PR-ów |
| AL-P0-04 | P0 | brak core-owned validatora i motion hash | niespójne wyniki TS/Rust/CI |
| AL-P0-05 | P0 | brak deterministycznego generatora katalogu | ręczny drift i nieweryfikowalne wydania |
| AL-P1-01 | P1 | presety hardcoded w React | każda animacja wymaga zmiany kodu |
| AL-P1-02 | P1 | brak tagów i źródeł Built-in/Community | katalog szybko stanie się nieczytelny |
| AL-P1-03 | P1 | brak eksportu contribution | contributor musi ręcznie rekonstruować JSON |
| AL-P1-04 | P1 | brak CI paczek i provenance | ryzyko złych/licencyjnie niedozwolonych danych |
| AL-P1-05 | P1 | ryzyko wzrostu bundle | regresja startu aplikacji i przekroczenie budżetu |
| AL-P1-06 | P1 | brak badge poziomu proof | offline validation może być mylona z NWN proofem |
| AL-P2-01 | P2 | brak automatycznego retargetingu różnych rigów | V1 obsłuży tylko ścisłe profile zgodności |
| AL-P2-02 | P2 | brak podpisanych aktualizacji | nowy katalog wymaga nowego wydania aplikacji |

## 4. Docelowa architektura

```text
animation-library/presets + tags
              │
              ▼
Core validator / canonical hash / catalog generator
              │
              ├── contracts/community-animation-catalog-v1.json
              └── osobne, hash-bound payloady ruchu
                                   │
                                   ▼ lazy
Studio Built-in / Community browser
              │
              ▼
Worker → WASM → Core compatibility + instantiate
              │
              ▼
source-bound AuthoredAnimationClip → Save to Custom
              │
              ▼
Base 42 mapping → Build → binary MDL readback
```

Źródłem prawdy jest Core. TypeScript nie może mieć łagodniejszego, niezależnego
parsera. Worker/WASM przekazują ten sam JSON i tę samą diagnostykę.

### 4.1 Dwa różne typy danych

1. `AnimationPresetV1` — przenośny, repozytoryjny, immutable `id + version`,
   bez source GLB SHA i bez numerycznych node ID.
2. `AuthoredAnimationClip` — lokalny, edytowalny i związany z dokładnym
   projektem, source GLB oraz node ID aktualnego rigu.

`Use as template` nigdy nie edytuje presetu. Tworzy nowy authored clip i
zapisuje w nim pełne provenance presetu.

### 4.2 Przenośność rigu — decyzja V1

Pierwsza wersja ma być bezpieczna i ścisła:

- preset wskazuje kości przez stabilne nazwy, nie `targetNodeId`;
- `rigSignatureSha256` jest liczony z nazw, relacji parent-name i
  skanonizowanych rest transforms, bez node ID;
- przy zastosowaniu Core rozwiązuje nazwy do node ID aktualnego rigu;
- zgodność wymaga identycznej sygnatury i wszystkich required bones;
- różne node ID przy tej samej semantycznej strukturze są dozwolone;
- różny rest pose albo hierarchia daje `INCOMPATIBLE` i nie mutuje projektu;
- automatyczny retarget do innego rigu jest osobną fazą P2, nie ukrytą częścią
  V1.

Wartości tracków V1 pozostają lokalnymi transformami dla ścisłego profilu
rigu. Dzięki fail-closed zgodności nie są stosowane do podobnego, lecz innego
szkieletu.

### 4.3 Provenance po zastosowaniu

Instancja w `Custom` zapisuje co najmniej:

- `presetId`, `presetVersion`, `presetMotionSha256`;
- `catalogSha256` i source `BUILT_IN` albo `COMMUNITY`;
- autora i SPDX license ID;
- docelowy `sourceRevision` aktualnego GLB;
- `rigSignatureSha256` użyty podczas zgodności;
- informację `instantiationMode=STRICT_RIG_V1`;
- lokalny authored clip ID i revision.

Obecny source kind powinien otrzymać nowy wariant
`LIBRARY_PRESET_COPY`. Ponieważ zmienia to wire contract, trzeba przygotować
wersjonowaną migrację dokumentu zamiast dopisywać nieudokumentowane pola do
V1.

### 4.4 Rozdzielenie katalogów

- `creature-animation-catalog-v1` nadal opisuje 42 stany Aurory;
- `meshy-animation-catalog-v1` nadal opisuje allowlistę zewnętrznych akcji
  Meshy;
- `community-animation-catalog-v1` opisuje nasze przenośne authored presets.

Nie wolno scalać tych kontraktów tylko dlatego, że wszystkie dotyczą animacji.

### 4.5 Kanoniczny układ repozytorium po audycie

```text
animation-library/
├── tags-v1.json
└── presets/
    └── m2a_right_cross/
        ├── manifest.json
        ├── animation.json
        ├── README.md
        └── preview.webp

contracts/
├── community-animation-manifest-v1.schema.json
├── community-animation-payload-v1.schema.json
├── community-animation-contribution-v1.schema.json
└── community-animation-catalog-v1.json       # generated, tracked
```

Korekta względem dokumentu koncepcyjnego: agregat `catalog.json` nie jest
ręcznie edytowanym drugim źródłem. Generator skanuje presety i zapisuje jeden
śledzony, deterministyczny kontrakt w `contracts`.

### 4.6 Ładowanie w aplikacji

- indeks metadanych jest mały i trafia do lazy chunk Animation Studio;
- `animation.json` jest osobnym assetem/chunkiem ładowanym dopiero po otwarciu
  szczegółów albo wybraniu `Use as template`;
- preview jest osobnym assetem;
- hash payloadu jest sprawdzany przed użyciem;
- zablokowana sieć nie zmienia działania katalogu wbudowanego w wydanie;
- żaden payload biblioteki nie może zwiększyć initial main JS ponad istniejący
  budżet.

## 5. Kontrakt presetu V1

Rekomendowany minimalny kształt manifestu:

```json
{
  "schemaVersion": 1,
  "presetId": "m2a_right_cross",
  "presetVersion": 1,
  "outputName": "m2a_right_cross",
  "label": "Right cross",
  "summary": "Compact boxing guard, straight right and recoil.",
  "source": "BUILT_IN",
  "authors": [{ "name": "Meshy2Aurora contributors" }],
  "license": "SPDX-ID-REQUIRES-OWNER-DECISION",
  "tags": ["attack", "unarmed", "boxing", "right-hand", "one-shot"],
  "playback": "ONE_SHOT",
  "durationSeconds": 1.0,
  "rigProfile": "M2A_HUMANOID_STRICT_V1",
  "rigSignatureSha256": "<sha256>",
  "requiredBones": ["Hips", "Spine", "RightArm", "RightForeArm", "RightHand"],
  "animationPath": "animation.json",
  "animationSha256": "<sha256>",
  "motionSha256": "<sha256>",
  "previewPath": "preview.webp",
  "previewSha256": "<sha256>",
  "validationStatus": "PIPELINE_VERIFIED"
}
```

`presetId` jest tożsamością biblioteki. `outputName` jest osobną nazwą MDL i
podlega limitowi 16 znaków. Wersja jest dodatnią liczbą całkowitą. Raz
opublikowanej pary `presetId + presetVersion` nie wolno zmienić bez zmiany
hasha katalogu i odrzucenia przez gate immutability.

Payload zawiera tracki związane z `targetBoneName`, kanoniczne quaterniony,
rosnące czasy i wydarzenia. Nie zawiera source GLB, ścieżek systemowych,
tokenów, skryptów ani binarnego MDL.

## 6. Funkcje do zaimplementowania

Nazwy są kontraktem kierunkowym; przy implementacji można je dopasować do
konwencji modułu bez zmiany odpowiedzialności.

### Core/Rust

- [x] `parse_animation_preset_v1(json)` — strict serde, unknown fields fail;
- [x] `validate_animation_preset_v1(preset, tags)` — schema, limity, tagi,
  SPDX, pliki i hashe;
- [x] `canonical_animation_preset_json_v1(preset)` — stabilna serializacja;
- [x] `animation_preset_motion_sha256_v1(payload)` — hash ruchu niezależny od
  node ID i source GLB;
- [x] `animation_rig_signature_v1(rig)` — name/parent/rest signature;
- [x] `inspect_animation_preset_compatibility_v1(preset, rig)` —
  `COMPATIBLE | INCOMPATIBLE` z diagnostyką;
- [x] `instantiate_animation_preset_v1(preset, rig, source_revision, id,
  output_name)` — source-bound authored copy;
- [x] `export_animation_contribution_v1(clip, rig, metadata)` — przenośna
  contribution bez payloadu modelu;
- [x] `validate_animation_contribution_v1(contribution)` — pełny gate eksportu;
- [x] contribution może deklarować wyłącznie `PIPELINE_VERIFIED`; promocja do
  `OWNER_NWN_VERIFIED` jest związana z dokładnym wpisem owner proof registry;
- [x] `build_community_animation_catalog_v1(root)` — deterministyczny agregat;
- [x] `verify_community_animation_catalog_v1(catalog, assets)` — wszystkie
  ścieżki, rozmiary i SHA-256;
- [x] migracja dokumentu/source provenance do `LIBRARY_PRESET_COPY`;
- [x] negatywne testy: stale hash, duplicate ID/version, wrong rig, unknown
  tag, non-unit quaternion, payload GLB/MDL, path traversal i zbyt duży plik.
- [x] negatywne testy: rozbieżne `requiredBones`, niekanoniczny katalog,
  brak `README.md`, zmieniony bez wersjonowania słownik tagów i fałszywa
  deklaracja owner proof.

### WASM i Worker

- [x] `validateAnimationPresetV1`;
- [x] `inspectAnimationPresetCompatibilityV1`;
- [x] `instantiateAnimationPresetV1`;
- [x] `exportAnimationContributionV1`;
- [x] dodać cztery typowane request/response do wspólnego worker contract;
- [x] zachować cancel/race guards przy zmianie projektu, source GLB i presetu;
- [x] test Node boundary i rzeczywisty browser Worker/WASM.

### Generator i repozytorium

- [x] `scripts/generate-community-animation-catalog.mjs`;
- [x] tryby `--check` i `--write`;
- [x] generowanie przez Core, nie przez niezależną logikę JS;
- [x] `npm run animation-library:check` w pretest/pretypecheck/prebuild;
- [x] kontrola dozwolonych ścieżek i rozszerzeń;
- [x] blokada plików GLB/FBX/MDL/HAK/MOD oraz kodu wykonywalnego;
- [x] limit rozmiaru jednego presetu i całego katalogu;
- [x] PR template i `CONTRIBUTING_ANIMATIONS.md`;
- [x] CODEOWNERS dla kontraktów i biblioteki.

### Studio/UI

- [x] zakładki `Built-in`, `Community`, `Custom`;
- [x] wyszukiwanie po label, preset ID, autorze i tagach;
- [x] klikalne chipy i wielokrotne filtry tagów;
- [x] panel szczegółów: preview, autor, licencja, wersja, rig i proof status;
- [x] badge `Compatible` / `Incompatible` z wyjaśnieniem;
- [x] `Use as template` wywołujące Worker/WASM/Core;
- [x] niezmienność wpisów Built-in/Community;
- [x] `Export contribution` dostępne tylko dla aktualnego klipu `VALID`;
- [x] czytelne rozróżnienie `PIPELINE_VERIFIED` od
  `OWNER_NWN_VERIFIED`;
- [x] lazy load payloadu i preview;
- [x] pełna klawiatura, focus management, screen reader names i empty/error
  states;
- [x] usunąć hardcoded `Void crystal cleave` z menu po migracji do katalogu.

## 7. Plan implementacji

### Faza 0 — decyzje i testy kontraktowe

- [ ] właściciel wybiera licencję repozytorium i contribution;
- [x] ustalić dozwoloną listę SPDX oraz sposób deklarowania współautorów;
- [x] zatwierdzić stabilny słownik tagów V1;
- [x] zatwierdzić `STRICT_RIG_V1` i brak automatycznego retargetingu w MVP;
- [x] zatwierdzić limity rozmiaru/klatek/presetów;
- [x] najpierw dodać failing tests schematów, hashy i niezgodnego rigu.

Rezultat: brak otwartych decyzji, które zmieniałyby format repozytorium.

### Faza 1 — Core portable preset

- [x] dodać struktury Rust i strict serde;
- [x] dodać tag validator, rig signature i canonical motion hash;
- [x] dodać compatibility inspection;
- [x] dodać instantiation preset → project-bound authored clip;
- [x] dodać source provenance i migrację dokumentu;
- [x] potwierdzić deterministyczność na synthetic humanoid fixture.

Rezultat: Core potrafi bez UI bezpiecznie zastosować preset albo zwrócić
blocking diagnostic bez mutacji projektu.

### Faza 2 — Repozytorium i generator katalogu

- [x] utworzyć `animation-library`, schematy i tags V1;
- [x] dodać jeden minimalny syntetyczny preset testowy;
- [x] generator skanuje, sortuje i hashuje wpisy;
- [x] `--check` wykrywa każdą ręczną/stale zmianę;
- [x] osobny index i lazy payload map nie naruszają bundle budget;
- [x] Windows i Linux generują byte-identical katalog.

Rezultat: świeży checkout odtwarza śledzony katalog bez diffu.

### Faza 3 — Worker/WASM i integracja projektu

- [x] dodać publiczne funkcje WASM i kontrakt Workera;
- [x] dodać race/cancel guards;
- [x] `Use as template` tworzy Draft związany z bieżącym source GLB;
- [x] `Save to Custom` przechowuje provenance presetu;
- [x] mapowanie Base 42, Build i binary readback zachowują istniejący kontrakt;
- [x] projekt/IndexedDB/backup round-tripują nowe provenance.

Rezultat: jeden preset przechodzi pełny Studio → Worker → WASM → Core → MDL.

### Faza 4 — Library UX, tagi i lazy loading

- [x] dodać trzy źródła biblioteki;
- [x] dodać search/tags/filter/details/preview;
- [x] dodać kompatybilność przed akcją;
- [x] Built-in/Community są immutable, edycja zawsze tworzy Custom;
- [x] payload nie jest pobierany przed wyborem presetu;
- [x] offline i dostępność przechodzą browser tests.

Rezultat: użytkownik znajduje preset po tagu i tworzy z niego własną kopię bez
znajomości plików repozytorium.

### Faza 5 — Export contribution

- [x] Core eksportuje pojedynczy contribution JSON bez modelu;
- [x] UI zbiera autora, licencję, tagi, label i opis;
- [x] eksport wylicza rig signature i motion hash;
- [x] aplikacja pokazuje listę braków przed pobraniem;
- [x] skrypt repo przyjmuje contribution i tworzy kanoniczny katalog presetu;
- [x] preview generuje pipeline na własnej syntetycznej fixture, nie na
  commitowanym Meshy/retail modelu.

Rezultat: plik wyeksportowany z aplikacji jest bez ręcznej edycji przyjmowany
przez lokalny validator i gotowy do PR.

### Faza 6 — CI i workflow pull requestów

- [x] dodać contribution job do GitHub Actions;
- [x] schema/hash/immutability/path/license/size gates;
- [x] Core materialization i binary MDL readback na fixture;
- [x] Studio catalog/search/use-as-template tests;
- [x] Worker/WASM integration;
- [x] build i bundle budget;
- [x] PR template wymaga provenance i oświadczenia o prawach;
- [x] raport CI wskazuje dokładną ścieżkę oraz naprawę każdego błędu.

Rezultat: zły lub niedozwolony preset nie może zostać zmergowany przy zielonym
CI.

### Faza 7 — biblioteka startowa

- [x] `m2a_right_cross` wyprowadzony z obecnego ruchu bez zmiany zamrożonego
  `vckcleave2`;
- [x] `m2a_left_jab`;
- [x] `m2a_right_hook`;
- [x] `m2a_uppercut`;
- [x] `m2a_combat_guard`;
- [x] `m2a_dodge_left`;
- [x] `m2a_dodge_right`;
- [x] każda animacja ma tagi, preview, pipeline proof i jawny NWN proof status;
- [ ] co najmniej jedna pełna ścieżka ma owner-reported NWN result.

Rezultat: funkcja nie jest demonstracją jednego hardcoded presetu.

### Faza 8 — podpisane aktualizacje, poza MVP

- [ ] signed GitHub Release manifest;
- [ ] ręczne `Check for updates`;
- [ ] cache, rollback i wbudowany fallback;
- [ ] hash/signature verification przed zapisem;
- [ ] testy offline, uszkodzonej paczki, downgrade i revoked release.

Ta faza nie blokuje ukończenia pierwszej repozytoryjnej biblioteki.

## 8. Kryteria ukończenia

Poniższe kryteria są binarne. Sam screenshot, build albo poprawny JSON nie
wystarcza.

### K0 — decyzje i prawa

- [ ] repozytorium ma jawny `LICENSE`;
- [ ] contribution ma zatwierdzoną politykę licencji i provenance;
- [x] PR wymaga oświadczenia, że autor ma prawa do ruchu;
- [x] retail/CEP/nieautoryzowane external animation payloads są blokowane.

### K1 — jedno źródło prawdy

- [x] Core definiuje i waliduje format;
- [x] TS/WASM/Worker używają dokładnego wyniku Core;
- [x] generator `--check` na Windows i Linux daje identyczne bajty;
- [x] tracked catalog nie może być ręcznie rozbieżny z presetami.

### K2 — deterministyczność i integralność

- [x] dwa eksporty tego samego ruchu dają ten sam `motionSha256`;
- [x] dwa buildy katalogu dają ten sam catalog SHA-256;
- [x] zmiana jednego keyframe'u zmienia motion i catalog hash;
- [x] zmiana preview nie zmienia motion hash, ale zmienia asset/catalog hash;
- [x] wszystkie deklarowane rozmiary i SHA-256 są sprawdzane przed użyciem.

### K3 — bezpieczna zgodność rigu

- [x] preset z tym samym rig signature i innymi node ID materializuje się
  poprawnie;
- [x] brak kości, inny parent albo rest pose daje `INCOMPATIBLE`;
- [x] przypadek niezgodny nie dodaje klipu, nie zmienia rewizji i nie zapisuje
  projektu;
- [x] brak automatycznego retargetingu jest jawny w UI.

### K4 — pełny pipeline produktu

- [x] `Use as template` tworzy Draft w `Custom`;
- [x] provenance zawiera preset/version/motion/catalog/rig/source hashes;
- [x] Save, IndexedDB i project backup round-tripują dane;
- [x] mapowanie do Base 42 działa;
- [x] Build przechodzi Studio → Worker → WASM → Core;
- [x] binary MDL readback ma `MATCH` i ten sam ruch;
- [x] źródłowy GLB pozostaje byte-identical.

### K5 — UX biblioteki

- [x] Built-in, Community i Custom są jednoznacznie rozróżnione;
- [x] search działa po nazwie, ID, autorze i tagu;
- [x] filtry tagów są łączalne, dostępne z klawiatury i czytelne dla screen
  readera;
- [x] szczegóły pokazują licencję, wersję, kompatybilność i proof status;
- [x] edycja repozytoryjnego presetu zawsze tworzy kopię Custom;
- [x] empty/loading/error/offline states mają testy.

### K6 — contribution developer experience

- [x] użytkownik eksportuje contribution z klipu `VALID`;
- [x] contribution nie zawiera GLB, ścieżki lokalnej ani sekretów;
- [x] lokalny check uruchamia dokładnie ten sam walidator co CI; zewnętrzny
  wynik joba czeka na push brancha;
- [x] poprawny plik wymaga co najwyżej skopiowania do wskazanego katalogu i
  otwarcia PR;
- [x] błędny plik daje kod, ścieżkę, przyczynę i naprawę;
- [x] README i PR template prowadzą autora od eksportu do zielonego CI.

### K7 — bezpieczeństwo i limity

- [x] unknown fields, path traversal, duplicate IDs, stale hashes i
  niedozwolone rozszerzenia fail closed;
- [x] status `OWNER_NWN_VERIFIED` jest fail-closed i wymaga dokładnej
  tożsamości preset/version/motion/rig w rejestrze dowodów;
- [x] paczka nie wykonuje żadnego kodu contributora;
- [x] limity rozmiaru, czasu, liczby tracków/klatek/eventów są testowane na
  granicy i powyżej granicy;
- [x] JSON parse i preview load nie blokują głównego UI;
- [x] katalog nie osłabia istniejących provenance/model iteration gates.

### K8 — wydajność i offline

- [x] initial main JS pozostaje w budżecie 460 000 B;
- [x] pełny payload presetu ładuje się dopiero na żądanie;
- [x] aplikacja działa z całkowicie zablokowaną siecią;
- [x] brak payloadu/404 daje lokalny błąd, nie uszkadza Custom;
- [x] katalog setek wpisów zachowuje responsywne wyszukiwanie i wirtualizację
  listy, jeśli pomiar jej wymaga.

### K9 — biblioteka startowa i jakość ruchu

- [x] co najmniej siedem presetów startowych przechodzi wszystkie offline
  gates;
- [x] każdy ma różne fazy ruchu potwierdzone klatkami aplikacji;
- [x] każdy ma jawny status `PIPELINE_VERIFIED` albo
  `OWNER_NWN_VERIFIED`, nigdy domyślne twierdzenie o runtime;
- [x] right cross przechodzi jako pierwszy end-to-end przykład;
- [x] żadna animacja startowa nie pochodzi z niedozwolonego zewnętrznego
  payloadu.

### K10 — regresja i dokumentacja

- [x] `cargo fmt --all -- --check`;
- [x] `cargo clippy --workspace --all-targets -- -D warnings`;
- [x] `cargo test --workspace`;
- [x] Studio typecheck i wszystkie testy;
- [x] Worker/WASM Node i browser integration;
- [x] production build i bundle budget;
- [x] `git diff --check`;
- [x] dokumentacja użytkownika, contributora, schematów i migration;
- [x] evidence zapisuje komendy, wyniki, hashe i pozostałe ograniczenia.

## 9. Definition of Done MVP

Repozytoryjna biblioteka jest ukończona, gdy świeży checkout może offline i
deterministycznie zbudować katalog, Studio potrafi znaleźć preset po tagu,
sprawdzić zgodność rigu, utworzyć immutable-derived kopię `Custom`, zapisać ją,
zmapować do Base 42 i wyeksportować do binary MDL z readbackiem `MATCH`.

Contributor musi móc wyeksportować z aplikacji deklaratywny contribution,
otworzyć PR i otrzymać identyczny lokalnie oraz w CI wynik walidacji. Biblioteka
startowa ma zawierać co najmniej siedem własnych presetów. Publiczne PR-y nie
są otwierane przed decyzją licencyjną.

Podpisane aktualizacje sieciowe oraz automatyczny retarget między różnymi
rigami są świadomie poza MVP i nie mogą opóźniać bezpiecznej biblioteki
wbudowanej w wydanie.

## 10. Zalecana kolejność rozpoczęcia

1. Decyzja licencyjna i słownik tagów.
2. Failing tests portable preset + strict rig signature.
3. Core contract, validator, hash i instantiation.
4. Generator repozytorium i jeden syntetyczny preset.
5. Worker/WASM i pełny pipeline bez UI.
6. Library UX i lazy loading.
7. Export contribution oraz CI.
8. `m2a_right_cross` i pozostała biblioteka startowa.

Nie należy zaczynać od dodania tagów bezpośrednio do obecnego
`AnimationStudioLibraryItemV1`. Taki skrót poprawiłby wyszukiwanie, ale nie
rozwiązałby przenośności, wersjonowania, provenance ani bezpiecznego PR flow.
