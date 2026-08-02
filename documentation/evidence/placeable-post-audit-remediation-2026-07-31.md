# Remediacja po audycie modeli Placeable — 2026-07-31

Status: `OFFLINE_IMPLEMENTATION_COMPLETE / TESTS_PASS / VISUAL_PROOF_STATE_UNCHANGED`

## Zakres

Ten zapis domyka implementacyjne i testowe punkty audytu Placeable bez
tworzenia kolejnego kandydata modelu. W tej pracy nie uruchamiano ani nie
adoptowano Aurora Toolset lub NWN, nie generowano nowego `rNN`, MOD-a, HAK-a,
resrefu ani pakietu proof i nie zmieniano żadnego zamrożonego artefaktu.

## Zrealizowane zmiany

1. **Spójna nazwa Area.** Generator ARE używa teraz nazwy przekazanej w
   `PlaceableIdentityV1.area_name`. Readback pakietu parsuje ARE i wymaga
   dokładnej zgodności nazwy zamiast przyjmować stałą nazwę fixture M0.
2. **Wiele materiałów i tekstur.** Pipeline rozwiązuje obraz bazowego koloru
   dla każdego używanego slotu materiału. Różne obrazy dostają deterministyczne
   resrefy i osobne TGA, a wspólny obraz jest emitowany raz i raportuje wszystkie
   korzystające z niego sloty. Walidacja odrzuca duplikaty tożsamości zasobu,
   duplikaty powiązań slotu oraz brak powiązania używanego materiału.
3. **Pusta kolizja blokuje build.** Rdzeń zwraca stabilny błąd
   `PLACEABLE-COLLISION-EMPTY`, gdy autorska projekcja nie zawiera żadnego
   elementu kolizji. Studio pokazuje błąd przed wysłaniem zadania do Workera.
4. **PWK jest pełnym komponentem produktu.** Projekcja wyniku Studio wymaga
   komponentu `pwk`, jego SHA-256 oraz dokładnie jednego zasobu HAK typu
   `PLACEABLE_WALKMESH` (`2053`) o tym samym resrefie i hashu. Raport
   kompletności nazywa wykonany gate precyzyjnie:
   `ascii_pwk_emitted_offline_readback_passed`.
5. **Skalowanie edytora komponentów.** Podział geometrii jest cache'owany per
   `BufferGeometry`, a wycięty komponent zawiera tylko własne, przemapowane
   wierzchołki. Outliner jest wirtualizowany, masowy podział wymaga potwierdzenia,
   a kosztowna analiza par luk jest pomijana powyżej 512 aktywnych źródeł z
   diagnostyką `GAP_ANALYSIS_SKIPPED`.
6. **Jeden limit produktu.** Placeable nadal używa wspólnego
   `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000`, z ostrzeżeniem przy `150_000`.
   Niezależna granica jednego strumienia binary MDL pozostaje równa `65 535`
   indeksów, czyli `21 845` trójkątów; większa geometria jest dzielona, a nie
   usuwana.

## Weryfikacja

| Gate | Wynik |
|---|---|
| `assert-canonical-workspace.ps1` | PASS — `C:\Projects\meshy2aurora` |
| `assert-meshy-asset-layout.ps1` | PASS — kanoniczne `sample-3d` |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy -p m2a-core -p m2a-wasm --all-targets -- -D warnings` | PASS |
| `git diff --check` | PASS |
| `cargo test -p m2a-core --no-fail-fast` | PASS — wszystkie wykonywalne testy; wyłącznie oczekiwane testy env-gated pozostały pominięte |
| `cargo test -p m2a-core --test placeable_pipeline --no-fail-fast` | PASS — 12 passed, 1 oczekiwany ignored |
| Kanoniczny S1, env-gated inspekcja źródła | PASS — SHA-256 `dad22a5c3490242458cb7a81e50c53e265abf75886f57f6bd8a770938c2f7372`, 1 mesh, 1 primitive, 1 material, 4 obrazy, 0 skins, 0 animations, 1477 trójkątów |
| `cargo test -p m2a-wasm --no-fail-fast` | PASS — 33 passed |
| `npm test -- --run` | PASS — 38 plików, 234 testy |
| `npm run typecheck` | PASS |
| Realny Worker WASM: statyczny Placeable | PASS — 1 passed, 9 poza filtrem |

Testy regresyjne obejmują dwa różne obrazy materiałów, deduplikację obrazu
współdzielonego, pustą kolizję, zgodność nazwy ARE, obecność PWK i zgodność jego
hashu, kompaktowe wydzielanie komponentu 4000-elementowego oraz wirtualizację
outlinera.

## Stan proof po remediacji

Ta remediacja dowodzi kontraktów i zachowania offline. Nie zmienia żadnego
`modelVisibility` ani `proofCompleteness`:

- wcześniejszy owner proof S1 pozostaje zachowany monotonicznie;
- kandydaci bez werdyktu właściciela nadal mają `modelVisibility=not_tested`;
- brak dowodu wizualnego nadal oznacza `proofCompleteness=missing`, nigdy
  `modelVisibility=not_visible`;
- nie ma podstaw do nowej iteracji modelu;
- TLC Meshy P20K V1 pozostaje dodatkowo zablokowany przez kolizję natywnego
  MOD-a opisaną w
  [osobnym amendmencie](tlc-meshy-p20k-placeables-v1-native-collision-amendment-2026-07-31.md).

Następny krok wizualny należy do właściciela i musi użyć dokładnie zamrożonego,
hash-bound kandydata. Agent nie może uznać widoczności w Toolset/NWN na podstawie
samych testów offline.
