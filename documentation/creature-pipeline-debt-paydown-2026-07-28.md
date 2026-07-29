# Spłata długu technicznego pipeline creature

Data: 2026-07-28
Status: `IMPLEMENTED / OFFLINE_GATES_PASS / V4_OWNER_RUNTIME_PROVED`

## Wynik

Linia proceduralnego humanoida ma teraz rozdzielone kontrakty produktu i
demonstracji:

```text
GLB + appearance.2da + ProceduralCreatureProductIdentityV2
  -> MDL + TGA + pełny appearance.2da + HAK
  -> ProceduralCreatureProductArtifactV2

ProceduralCreatureProductArtifactV2
  + BinaryCreatureModuleIdentityV1
  + creature resref
  -> opcjonalny fixture-only MOD/UTC
```

Pierwsza operacja nie przyjmuje `moduleResref`, `areaResref` ani
`creatureResref`, więc nie może utworzyć MOD, Area lub UTC. Druga operacja nie
przebudowuje produktu i odrzuca próbę podpięcia innego HAK.

Historyczne funkcje V1 pozostają adapterami zgodności dla zamrożonych packetów.
Studio używa nowej granicy V2 dla lane
`SKINNED_PROCEDURAL_HUMANOID_42`.

## Owner proof V4

Właściciel potwierdził dla exact V4:

```yaml
module_file: tlcpowdemo4.mod
hak_file: tlcpowhak4.hak
model_resref: tlcpow_m4
texture_resref: tlcpow_t4
appearance_row: 15100
owner_result: "v4 działa"
nwn:
  modelVisibility: visible
  proofCompleteness: verified
deathSequence: passed
deathEntryJump: passed
toolset:
  modelVisibility: not_tested
  proofCompleteness: missing
```

SHA-256:

```text
MOD 5b0fd9713a0644e0d95d45e20d4bc3c971b1adea5b64428f2752dc347461624a
HAK 926da4db4f64226f1e2163c05b58547f966bfa8b79c32718f1b2fd5cd6751598
```

Źródło:
[exact V4 owner result](evidence/tlc-powrotnik-death-family-v4-ready-for-owner-proof-2026-07-28.md).

Nie uruchomiono nowej iteracji modelu i nie zmieniono V4. Zmiany opisane w tym
dokumencie są zmianami kodu, kontraktów i testów; nie stanowią V5 ani nowego
claimu wizualnego.

## Pochodzenie animacji

`DirectCreatureAnimationCompletenessV2` zastępuje nieprecyzyjne raportowanie
V1. Każdy z 42 klipów ma teraz dokładne pochodzenie:

- `PRESERVED_SOURCE` — zachowany klip wejściowy;
- `SOURCE_DERIVED` — klip utworzony z jawnie wskazanego klipu źródłowego,
  na przykład terminalny hold;
- `PROCEDURAL` — ruch utworzony przez clean-room authoring;
- osobna lista `discardedSourceClips` rejestruje wejściowe klipy świadomie
  wyłączone z końcowej rodziny.

Raport nie może już nazywać proceduralnych klipów „explicit”. Kontrakt zachowań
ma `schemaVersion=2` (`DirectCreatureAnimationBehaviorV2`).

## Przejścia, root motion i eventy

Każdy track jest normalizowany do klucza w czasie `0`. Pipeline zamyka i
sprawdza sześć granic:

```text
ckdbck    -> ckdbckps
ckdbckps  -> cguptokdb
cguptokdb -> cgustandb
cgustandb -> cpause1
ckdbck    -> ckdbckdie
ckdbckdie -> cdead
```

`ProceduralHumanoidKinematicsConformanceV2` blokuje produkt, gdy:

- track nie zaczyna się od `t=0`;
- wymagana granica pozycji/rotacji/skali jest nieciągła;
- `cwalk`, `crun`, `ccwalkf`, `ccwalkb`, `ccwalkl`, `ccwalkr` lub `ccturnr`
  nie jest animacją in-place w tolerancji 0,05 m.

Przemieszczenie klipów upadku, uników i obrażeń jest raportowane, ale nie jest
automatycznie zabronione.

Automatyczne eventy mają politykę `KINEMATIC_PEAK_V2`. Moment ataku, kroku lub
uderzenia o ziemię pochodzi z najsilniejszego zaobserwowanego przedziału ruchu
odpowiednich jointów, a nie ze stałego procentu długości klipu. Jawny sidecar
użytkownika zachowuje odrębną politykę `CALLER_OWNED_EXPLICIT_V1`.

## Appearance i Hook Horror

Wiersz 102 `c_horror` pozostaje wyłącznie strukturalnym dawcą kompletnego
wiersza `MODELTYPE=S`. Nie jest już semantycznym profilem produktu.

`DirectCreatureAppearanceSemanticProfileV2::HumanoidMediumV1` jawnie nadpisuje
35 pól runtime. Wartości ruchu, rozmiaru, przestrzeni, head trackingu i
pozostałych cech pochodzą ze stockowego średniego Human; bezpośredni model
zachowuje `MODELTYPE=S` i własny `RACE=<model resref>`. Najważniejsze
odcięcia od Hook Horrora:

```text
BLOODCOLR=R
PORTRAIT=****
SIZECATEGORY=3
FOOTSTEPTYPE=0
SOUNDAPPTYPE=0
NAME=<generated appearance label>
RACIALTYPE=11
```

Legacy/M0 zachowuje
`LegacyDirectMonsterDonorV1`, aby nie zmieniać zamrożonych packetów.

Produkcyjny artifact V2 nie zawiera UTC. Obecny
`ActiveMonsterBaseline` jest profilem wyłącznie demonstracyjnego fixture MOD,
nie profilem danych gameplayowych produktu.

## Świeża tożsamość w Studio

Studio przekazuje do WASM:

```json
{
  "modelResref": "m2c2m<sha8>",
  "textureResref": "m2c2t<sha8>",
  "hakResref": "m2c2h<sha8>",
  "appearanceLabel": "M2A_CREATURE_V2_<SHA8>"
}
```

Resrefy są walidowane jako 1–16 znaków `[a-z0-9_]`; label jako 1–64 znaki
alfanumeryczne lub `_`. `<sha8>` jest pierwszymi ośmioma znakami kanonicznego
SHA-256 wejściowego GLB, a `c2` wersjonuje kontrakt produktu. Zmiana źródła lub
przyszła zmiana wersji kontraktu nie wraca do historycznych stałych resrefów.

Worker zwraca dla tej lane dokładnie:

1. HAK;
2. MDL;
3. report JSON;
4. manifest JSON;
5. summary JSON.

Nie zwraca pustego ani domniemanego `proof-module`. Projector kanonicznego
wyniku rozpoznaje schema V2, uzgadnia wszystkie hashe i odrzuca obecność MOD w
produkcie.

## Bramki

Wykonane 2026-07-28:

```text
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
npm run typecheck
npm test
npm run test:worker-integration
node --test tools/meshy-local-bridge/bridge.test.mjs \
  tools/meshy-local-bridge/merge-animation-glbs.test.mjs
```

Wyniki:

- Rust workspace: PASS; zwykłe testy bez porażek, istniejące
  environment-bound `#[ignore]` pozostały jawnie pominięte;
- WASM native adapter: 31/31 PASS;
- Studio: 201/201 PASS;
- browser Worker/WASM integration: 9/9 PASS;
- Local Bridge: 19/19 PASS;
- clippy `-D warnings`: PASS;
- typecheck: PASS.

## Granice

- Nie wykonano agentowego Toolset/NWN proofu; końcowy proof jest własnością
  człowieka.
- V4 zachowuje już pozytywny owner verdict NWN. Oś Toolset V4 pozostaje
  `not_tested/missing` i nie jest fałszywie promowana.
- Utworzenie kolejnego runtime kandydata nadal podlega model iteration gate.
- Jeżeli produkt ma kiedyś dostarczać produkcyjny UTC, potrzebny jest osobny,
  wersjonowany kontrakt gameplayowy. Nie wolno promować fixture
  `ActiveMonsterBaseline` do tej roli.

## P0 V5 ExactFiniteNonCollinear — 2026-07-29

Status: `IMPLEMENTED / CORPUS_REPLAY_PASS / V5_OWNER_TOOLSET_AND_NWN_VERIFIED`

Po owner proof Stonebacka V5 zamknięto P0 geometrii aktywnych humanoidalnych
Creature:

```text
Product 20K
P100K target class (bounded Meshy envelope <= 110K)
P300K segmented experiment
  -> source sanitation: ExactFiniteNonCollinear
  -> rig/reference surface: ExactFiniteNonCollinear
  -> runtime face planes: ExactFiniteNonCollinear
  -> binary MDL writer: ExactFiniteNonCollinear
```

Reguła odrzuca tylko:

- wartości niefinitywne;
- pole dokładnie równe zero;
- dokładną współliniowość.

Nie istnieje już absolutny próg pola, który mógłby uznać poprawną
mikrogeometrię za błąd. Frozen legacy Product bundle zachowuje dawną politykę,
aby nie przepisywać historycznych packetów.

Canonical source replay:

```text
H1 humanoid       1,556 -> exact 1,556 -> legacy 1,556
H2 sentinel       1,543 -> exact 1,543 -> legacy 1,543
Powrotnik P20K   19,892 -> exact 19,892 -> legacy 19,892
Powrotnik P100K 103,290 -> exact 103,290 -> legacy 97,633
Veiled P100K    102,335 -> exact 102,335 -> legacy 99,812
Stoneback P300K 296,276 -> exact 296,276 -> legacy 290,319
```

Pełne replaye `GLB -> conversion -> runtime IR -> binary MDL`:

```text
Product 20K  19,892 -> 19,892 -> 19,892
P100K       102,335 -> 102,335 -> 102,335
P300K       296,276 -> 296,276 -> 296,276
```

Dokładny owner verdict V5:

```yaml
module_file: m2p3jd0eeb135c.mod
hak_file: m2p3jh0eeb135c.hak
model_sha256: c8b173a59a576911ff938b7287f823694474be838a81cb55c370c8c3258eb6e9
hak_sha256: 90d79114f0bc725e91f54657dc39f7491862ab4c0f54e032703ac4fddd6c9853
module_sha256: da60bc35b45f7ce53753ed91d6c2c13ce09192b6d63143acf2fffca2c94cf1df
toolset: { modelVisibility: visible, proofCompleteness: verified }
nwn: { modelVisibility: visible, proofCompleteness: verified }
geometryArtifactResult: fixed
```

Studio rozpoznaje ten wynik tylko przez pełne dopasowanie hashy MDL, TGA, HAK
i MOD do wersjonowanego rejestru owner proof. Inne wyniki nadal pokazują
`OPEN_M6`; współdzielenie źródła albo nazwy trasy nie promuje ich
automatycznie.

Dokładne evidence:
[Stoneback Geometry A/B V5](evidence/tlc-stoneback-brute-p300k-geometry-ab-v5-ready-for-owner-proof-2026-07-29.md).

Opcjonalny texture cleanup pozostaje dostępny dla faktycznych wad texeli, ale
nie jest przyczyną ani obowiązkową naprawą jasnych punktów/ubytków Stonebacka.

### Bramki zamknięcia P0

Wykonane 2026-07-29:

```text
assert-canonical-workspace.ps1
assert-meshy-asset-layout.ps1
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p m2a-core --lib \
  canonical_humanoid_creature_corpus_preserves_every_exact_finite_non_collinear_face \
  -- --ignored --nocapture
cargo test -p m2a-core --test creature_exact_geometry_corpus \
  product_20k_replays_through_exact_geometry_policy_without_face_loss \
  -- --ignored
cargo test -p m2a-core --test creature_exact_geometry_corpus \
  p100k_replays_bounded_meshy_overshoot_without_face_loss \
  -- --ignored
cargo test -p m2a-core --test creature_exact_geometry_corpus \
  p300k_replays_owner_verified_v5_geometry_without_face_loss \
  -- --ignored
npm run typecheck
npm test
npm run test:worker-integration
npm run build
node --test tools/meshy-local-bridge/bridge.test.mjs \
  tools/meshy-local-bridge/merge-animation-glbs.test.mjs
git diff --check
```

Wynik:

- canonical workspace i asset layout: PASS;
- format i clippy `-D warnings`: PASS;
- pełny Rust workspace: PASS, bez porażek;
- local source corpus replay: 6/6 PASS;
- pełny exact geometry replay: Product 20K, P100K i P300K — 3/3 PASS;
- Studio typecheck: PASS;
- Studio Vitest: 36 plików, 209/209 PASS, w tym projekcja exact lokalnego
  packetu Stoneback V5 jako `OWNER_VERIFIED`;
- Worker/WASM integration: 2 pliki, 9 PASS, 2 jawnie pominięte przez brak
  osobnego environment switcha;
- production build: PASS;
- Local Bridge: 21/21 PASS;
- kontrola whitespace/diff: PASS.

Ostrzeżenie Vite o chunku JavaScript większym niż 500 kB pozostaje
wydajnościowym zadaniem poza P0 Creature i nie wpływa na poprawność pipeline'u
ani wynik owner proof V5.

## Aktualizacja limitu produktu — 2026-07-29

Nazwy Product 20K/P100K/P300K w macierzy wyników pozostają nazwami
historycznych corpusów i profili replay. Bieżący profil produktu to
`PRODUCT_300K`: wspólny limit wynosi 300 000, warning 150 000, a każdy
przekroczony per-mesh limit binary MDL jest obsługiwany przez wspólną
bezstratną segmentację.
