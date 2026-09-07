# Placeable V9 — implementacja pipeline'u materiałów Aurora

Data: 2026-08-13\
Status: `IMPLEMENTED_OFFLINE / 6_OF_6_COMPLETE / OWNER_PROOF_SEPARATE`

## Wynik

Plan sześciu etapów został wykonany. Produkcyjna ścieżka Placeable potrafi
przenieść source-bound Material Separation przez kompilator materiałów,
binary MDL, TGA/MTR/TXI, HAK, WASM, Worker i Studio. Każda warstwa wykonuje
własny readback i blokuje rozjazd semantyczny.

Nie utworzono nowej iteracji statku ani nowego kandydata proof. Finalny proof
wizualny dokładnego modelu w Aurora Toolset/NWN pozostaje osobnym krokiem
wykonywanym przez właściciela.

## Zrealizowane etapy

### 1. Kontrakt i quality gate

- profile `AURORA_CLASSIC_SAFE` oraz `NWN_EE_MTR`;
- source-bound ledger kanałów PBR;
- fail-closed dla niewspieranych alpha/two-sided w profilu klasycznym;
- kontrola UV, texel density, fragmentacji oraz czytelności mipów;
- quality gate nie zmienia geometrii ani tekstur.

### 2. Integracja z Placeable

- publiczna trasa `build_meshy_static_placeable_package_v9`;
- Material Separation V2 i opcjonalny Material UV Projection;
- niezależne tekstury per material slot z zachowaniem source fallback;
- brak geometry cleanup domyślnie i wspólny limit 300 000 trójkątów.

### 3. Zasoby oraz binary MDL

- rzeczywisty diffuse i specular/gloss bake;
- normal map, diffuse/specular TGA oraz canonical TXI;
- canonical MTR dla profilu EE;
- MDL zapisuje diffuse/ambient/specular/shininess, alpha,
  self-illumination, `texture1`, `texture2`, `materialname`, UV1..UV3 i
  tangenty;
- finalne bajty MDL są ponownie parsowane i porównywane semantycznie.

### 4. HAK, WASM i Worker

- HAK zawiera dokładne zasoby TGA/DDS/MTR/TXI;
- każdy zasób jest odczytywany z archiwum i porównywany byte-for-byte;
- parsery TGA/MTR/TXI potwierdzają semantykę wygenerowanych zasobów V9;
- WASM i Worker zwracają dokładne bajty oraz descriptor SHA-256;
- legacy `build_static_placeable_package_v1` zachowuje wcześniejszy kontrakt
  i nie jest błędnie przepuszczany przez parser zasobów produkowanych tylko
  przez V9.

### 5. Studio

- wybór profilu materiału dla Placeable;
- downloader obsługuje TGA, DDS, MTR i TXI;
- projekcja wyniku wymaga kompletnego raportu V9 i zgodności hashy artefaktów;
- `Aurora Export` rekonstruuje materiał wyłącznie z finalnego readbacku MDL i
  hash-verified zasobów;
- MTR steruje `twoSided`, transparency oraz punch-through;
- normal/specular mapy są traktowane jako data maps, diffuse jako sRGB.

### 6. E2E i regresje

Test przeglądarkowy przechodzi cały pion:

`Studio -> Worker -> WASM -> Core V9 -> MDL/TGA/MTR/TXI -> HAK/MOD -> Studio readback`

Fixture sukcesu korzysta z czytelnej tekstury 512x512. Oddzielny fixture
potwierdza, że zbyt mała gęstość texeli zostaje zablokowana zamiast cicho
przechodzić.

## Regresja wykryta podczas walidacji

Pierwszy pełny przebieg wykrył jeden błąd w
`full_static_placeable_package_is_deterministic_and_cross_resource_consistent`.
Nowy parser TGA był wywoływany również dla historycznej niskopoziomowej trasy,
której test używa celowo atrapowego payloadu tekstury. Poprawka ograniczyła
semantyczny readback TGA do pakietów z zasobami materiałowymi V9. Po poprawce:

- legacy output nadal jest deterministyczny i byte-compatible;
- V9 nadal wymaga poprawnych TGA/MTR/TXI;
- cały `placeable_pipeline` przechodzi 25/25, z jednym testem env-gated
  oznaczonym `ignored`.

## Końcowa macierz walidacji

| Bramka | Wynik |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |
| `cargo test -p m2a-core --no-fail-fast` | PASS; brak failed, tylko jawne testy env-gated `ignored` |
| `cargo test -p m2a-wasm --no-fail-fast` | PASS, 41/41 |
| `cargo check -p m2a-core -p m2a-wasm` | PASS |
| `npm test` | PASS, 49/49 plików, 281/281 testów |
| `npm run typecheck` | PASS |
| `npm run test:worker-integration` | PASS, 3/3 pliki, 13 PASS, 2 jawnie pominięte kosztowne replaye |

## Kryteria ukończenia

- [x] Placeable V9 używa wspólnego kompilatora materiałów.
- [x] Materiały są source/recipe/hash-bound i fail-closed.
- [x] TGA/MTR/TXI powstają jako realne, parsowalne zasoby.
- [x] Binary MDL zachowuje i odczytuje rozszerzoną semantykę materiału.
- [x] HAK readback potwierdza dokładne payloady każdego zasobu.
- [x] WASM/Worker zachowują parity z Core i eksportują dokładne bajty.
- [x] Studio rozróżnia finalny eksport od podglądu source.
- [x] Pełny pion przeglądarkowy i regresje przechodzą.
- [x] Legacy Placeable pozostaje zgodny wstecznie.
- [ ] Finalny wizualny wynik konkretnego kandydata w Toolset/NWN — osobny
  owner proof, poza granicą tej implementacji.

## Stan handoffu

Implementacja jest gotowa do użycia przy następnym dozwolonym kandydacie.
Nie ma podstaw do deklarowania `ready_for_owner_proof` dla nowego statku,
ponieważ ten etap nie zamrażał ani nie instalował żadnego konkretnego MOD/HAK.
