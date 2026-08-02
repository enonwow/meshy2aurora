# Biblioteka animacji — stan implementacji i evidence

Data: 2026-07-31  
Branch/worktree: `animation` / `C:\Projects\meshy2aurora\.worktrees\animation`  
Kontrakt wykonawczy: `audyt-plan-biblioteki-animacji-2026-07-31.md`  
Status: `TECHNICAL MVP IMPLEMENTED / OWNER GATES OPEN`

## Wynik

Repozytorium zawiera działającą, offline-first bibliotekę animacji. Preset
jest przenośnym i immutable zasobem związanym z nazwami kości. `Use as
template` sprawdza zgodność w Core i tworzy lokalny Draft `Custom` związany z
dokładnym source GLB. Dalej używany jest istniejący pipeline
Studio → Worker → WASM → Core → binary MDL → readback `MATCH`.

Nie są ukończone dwa kroki należące do właściciela:

- wybór i dodanie root `LICENSE` oraz zatwierdzenie publicznej polityki
  contribution;
- owner-reported NWN result przypisany do jednego z siedmiu dokładnych
  presetów startowych.

Brak tych decyzji nie osłabia implementacji. Publiczne zewnętrzne PR-y
animacyjne pozostają zamknięte, a statusy presetów pozostają uczciwie
`PIPELINE_VERIFIED`, nie `OWNER_NWN_VERIFIED`.

## Fazy F0–F8

### F0 — decyzje i testy kontraktowe

- [ ] Root `LICENSE` i publiczna polityka contribution — wymaga decyzji
  właściciela; nie została zgadnięta.
- [x] Techniczna allowlista V1: `CC0-1.0`, `CC-BY-4.0` i wewnętrzny
  `LicenseRef-Meshy2Aurora-Project-Generated`.
- [x] Stabilny słownik 17 tagów w `animation-library/tags-v1.json`.
- [x] `STRICT_RIG_V1`, bez ukrytego retargetingu.
- [x] Limity presetów, tracków, events, keyframes, payloadu, preview i
  katalogu.
- [x] Pozytywne i negatywne testy kontraktu przed domknięciem integracji.

### F1 — Core portable preset

- [x] Strict serde i unknown-field rejection.
- [x] Canonical JSON, motion hash niezależny od source/node ID.
- [x] Rig signature z name/parent/rest transforms; rest transforms są
  kanonizowane do 6 miejsc dziesiętnych, aby semantycznie identyczny rig po
  round-tripie GLB nie zmieniał tożsamości przez 1–2 ULP `f32`.
- [x] Fail-closed compatibility inspection.
- [x] `requiredBones` jest dokładnie wyliczane z root bone i targetów tracków;
  brakująca albo nadmiarowa kość blokuje preset.
- [x] Instantiation do source-bound Draft z pełną provenance.
- [x] Rzeczywista migracja Animation Studio V1→V2 przed
  `LIBRARY_PRESET_COPY`.
- [x] Eksport i walidacja contribution.
- [x] Atomowy, no-clobber install contribution z immutable versioningiem.

### F2 — repozytorium i generator

- [x] `animation-library`, tags, rig profile, schematy i tracked catalog.
- [x] Generator przez Core z `--check` i `--write`.
- [x] Core wymaga kanonicznego katalogu `presetId` / `presetId-vN`, pliku
  `README.md` oraz dokładnie stabilnego słownika tagów V1; jego cicha zmiana
  wymaga nowego wersjonowanego kontraktu.
- [x] Deterministyczne sortowanie, rozmiary i SHA-256.
- [x] Lazy asset map; payloady ruchu nie są w initial/main JS.
- [x] GitHub Actions ma macierz `windows-latest` + `ubuntu-latest`.
- [x] Ten sam read-only `--check` przechodzi na Windows oraz w lokalnym
  kontenerze Linux z Rust 1.96.1 i Node 24.15.0, zwracając identyczny logical
  catalog SHA-256.
- [ ] Zewnętrzny wynik nowego joba GitHub Actions nie jest jeszcze dostępny
  bez pushu brancha; workflow jest gotowy do uruchomienia.

### F3 — Worker/WASM i pipeline projektu

- [x] Cztery publiczne granice validate/compatibility/instantiate/export.
- [x] 18/18 typów request ma exhaustywną obsługę Workera.
- [x] Race guard odrzuca stary wynik po zmianie projektu/source/presetu.
- [x] `Use as template` tworzy Draft Custom i podnosi dokument do V2.
- [x] IndexedDB i project backup zachowują provenance.
- [x] Tracked `animation-library/presets/m2a_right_cross` przechodzi prawdziwy
  Worker/WASM: validate → compatibility → instantiate → Custom/Base 42 →
  build → binary MDL readback `MATCH`; czasy i wartości translacji zgadzają
  się klatka po klatce, a source bytes pozostają niezmienione.

### F4 — UX biblioteki

- [x] Rozdzielone `Base 42`, `Built-in`, `Community`, `Custom`.
- [x] Search po label, preset ID, autorze i tagach.
- [x] Łączalne filtry tagów, klawiatura i screen-reader labels.
- [x] Preview, licencja, wersja, rig, proof i kompatybilność.
- [x] Presety repo są immutable; akcja zawsze tworzy Custom.
- [x] Loading/empty/error/404/offline mają kontrolowane stany i testy.
- [x] Wyszukiwanie obejmuje maksymalny kontrakt 2048 wpisów, a progressive
  rendering ogranicza pojedyncze okno DOM do 100 wyników z dostępną akcją
  `Show more presets`.
- [x] Hardcoded `Void crystal cleave` i `Root motion pulse` usunięto z menu.

### F5 — export contribution

- [x] Eksport wyłącznie z aktualnego klipu `VALID`.
- [x] Dialog autora, licencji, tagów, label, summary i oświadczenia o prawach.
- [x] Portable JSON bez GLB, lokalnych ścieżek, sekretów i binariów.
- [x] `scripts/import-animation-contribution.mjs` instaluje V1/Vn bez
  nadpisywania opublikowanej wersji.
- [x] Preview generowane z własnego profilu rigu i tracków przez repozytoryjny
  pipeline canvas, bez Meshy/retail modelu.

### F6 — CI i workflow PR

- [x] Osobny job biblioteki, schema/hash/path/size/license/immutability gates.
- [x] Core, WASM Node, prawdziwy browser Worker, Studio, build i budget gates.
- [x] PR template wymaga praw do ruchu i jawnego provenance.
- [x] CODEOWNERS obejmuje bibliotekę i kontrakty.
- [x] Diagnostyka Core zawiera kod, path, przyczynę i akcję naprawczą.
- [x] Contribution nie może samodzielnie nadać sobie
  `OWNER_NWN_VERIFIED`; promocja katalogu wymaga dokładnego wpisu rejestru
  proofów z tym samym preset ID, version, motion SHA i rig SHA.

### F7 — biblioteka startowa

- [x] `m2a_right_cross`
- [x] `m2a_left_jab`
- [x] `m2a_right_hook`
- [x] `m2a_uppercut`
- [x] `m2a_combat_guard`
- [x] `m2a_dodge_left`
- [x] `m2a_dodge_right`
- [x] Każdy ma osobny wielofazowy ruch, tagi, README, preview i
  `PIPELINE_VERIFIED`.
- [ ] Owner-reported NWN result dla jednego dokładnego presetu startowego.

Wszystkie startery używają kanonicznego rig signature
`e5b907dc8d0c86e62232744fc593ec6655ea2cb560fced56706e5a57c8fe2a5e`.
Logiczny catalog SHA-256 wynosi
`6a0ff531039d7c6d977f95c8df90295c45d7cb510f7a53102e6cf36b1cfadb97`.

### F8 — signed updates

- [ ] Signed release manifest, manual update, cache/rollback i revoke tests.

F8 jest zgodnie z planem poza MVP i nie blokuje biblioteki wbudowanej w
wydanie.

## Kryteria K0–K10

| Kryterium | Stan | Dowód / pozostały warunek |
|---|---|---|
| K0 prawa | `OWNER_OPEN` | PR rights gate i blokady payloadów są gotowe; brak root `LICENSE` i zatwierdzonej polityki publicznej. |
| K1 source of truth | `PASS_LOCAL_CROSS_OS` | Core-owned parse/validate/hash/catalog; Worker/WASM przekazują ten sam kontrakt; Windows i read-only Linux zwracają ten sam catalog SHA; macierz obu systemów jest w CI. |
| K2 integralność | `PASS` | Testy deterministycznego export/catalog, keyframe/preview hash separation, wszystkich byte/hash bindings, dokładnego `requiredBones`, kanonicznej ścieżki/README i stabilnego słownika tagów; produkcyjny preview jest sprawdzany po rozmiarze i SHA-256 przed pokazaniem. |
| K3 rig | `PASS` | Inne node ID i semantycznie identyczny round-trip z sub-mikro szumem `f32` przechodzą; brak kości, inny parent lub zmiana rest o `0.01` fail closed; brak mutacji przy race/incompatible. |
| K4 pipeline | `PASS_OFFLINE` | Browser Worker/WASM na tracked `m2a_right_cross`: preset → V2 Draft → Custom/Base42 → binary MDL readback `MATCH`, source unchanged, keyframe times/values zgodne. |
| K5 UX | `PASS` | Search/tags/details/compatibility/immutable copy/accessibility/error-state tests. |
| K6 contributor DX | `LOCAL_PASS / PUBLIC_CLOSED` | Export i importer są gotowe; publiczny PR czeka na K0. |
| K7 safety | `PASS` | Unknown fields, traversal, symlink, extension, executable, duplicate, stale hash i wszystkie limity fail closed. |
| K8 performance/offline | `PASS` | Main 411403 B < 460000 B; animation payload JS 602 B; payload JSON i preview są lazy/static/offline; test 2048 wpisów przeszukuje cały katalog przy oknie DOM ograniczonym do 100. |
| K9 starter quality | `PIPELINE_PASS / OWNER_OPEN` | 7 wielofazowych presetów i right-cross E2E; wszystkie uczciwie `PIPELINE_VERIFIED`; brak owner NWN result startera. |
| K10 regression/docs | `PASS_OFFLINE` | Format, Clippy, workspace, Studio, Node/browser, production build, budget, migration i contributor docs; final `git diff --check` w bramce końcowej. |

## Podglądy starterów

| Preset | Bajty | SHA-256 |
|---|---:|---|
| `m2a_combat_guard` | 15296 | `f57c62f4be01e53361ce04a1224d23ac1fc66758c89881b887d5c62f98ff20b7` |
| `m2a_dodge_left` | 11930 | `5a573f797a461ebe94735e0664e14935c7e2332c22856e5b353ec77636db8dbb` |
| `m2a_dodge_right` | 12198 | `27e4a49279dfcca697354df9b0ecea8c7e8ddf03218ca4da157726b283592177` |
| `m2a_left_jab` | 13092 | `6db23cdf6a04758023a8a537a64851a8d39353cf3a75f5c9b42954322b0709c5` |
| `m2a_right_cross` | 16066 | `d3c735a2860ce3b04ff0fe9d27c801a53b1279651cda41d3aa678fdb74e58f44` |
| `m2a_right_hook` | 12632 | `9c85bf0bcdb30d4e4561b3e922f8df08ea360b82a12f052da93ae192af6d30e2` |
| `m2a_uppercut` | 13126 | `76a17e9579134f38213dfb7cb6902b6b2b3fe68c8bdfbad0d6dfb7cd92e8206f` |

## Ostatnie lokalne bramki

- `assert-canonical-workspace.ps1`: PASS.
- `assert-meshy-asset-layout.ps1`: PASS.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS; testy wymagające jawnie lokalnych,
  Git-ignorowanych assetów/runtime witnesses pozostają oznaczone `ignored`.
- Core animation library: 15/15 PASS.
- Owner proof registry/catalog binding self-tests: PASS; bieżący rejestr ma
  uczciwie `0` wpisów.
- Studio: 408 PASS, 3 jawnie skipped.
- WASM Node tests: 20/20 PASS; generated Node boundary: PASS.
- Browser Worker/WASM: 25 PASS, 2 environment-skipped.
- Browser persistence: 2/2 PASS.
- Local Bridge i inspekcja GLB: 33/33 PASS.
- Production build: PASS.
- Bundle: main 411403 B; main gzip 115030 B; total JS 1476021 B;
  CSS 150213 B; animation payload JS 602 B; WASM 3443445 B.
- Catalog gate: 7 presetów, logical SHA-256
  `6a0ff531039d7c6d977f95c8df90295c45d7cb510f7a53102e6cf36b1cfadb97`.
- Linux cross-OS gate: read-only Docker/WSL, Rust 1.96.1, Node 24.15.0,
  7 presetów i ten sam logical catalog SHA-256 — PASS.

Pełne bramki są uruchamiane ponownie po każdej zmianie kontraktu przed
oznaczeniem celu jako ukończonego.
