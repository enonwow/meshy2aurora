# Dokładny bind supermodelu V3 — implementacja 2026-08-21

## Zakres i decyzja właściciela

Właściciel zlecił wdrożenie wspólnej funkcji dla każdego supermodelu, a nie
specjalnego wyjątku dla `c_wolf`. Model wynikowy ma używać dokładnej hierarchii
i neutralnych lokalnych transformacji carrierów odczytanych z wybranego
supermodelu. Geometria, tekstury i wagi skina pozostają własnością modelu
źródłowego; pipeline nie kopiuje geometrii retail, wag retail, kluczy
kontrolerów ani payloadu animacji.

## Potwierdzone przyczyny poprzedniego błędu

Fakty z naszego readera i poprzedniego pipeline:

- kontrakt V2 `c_wolf` używał proceduralnej bryły 1,0 × 1,5 × 1,0 zamiast
  neutralnych macierzy zapisanych w `c_wolf.mdl`;
- bezpośredni carrier kopiował macierze targetu i raportował zerowy błąd przez
  porównanie targetu z nim samym;
- `bindPoseCompatible` nie oznaczało porównania z immutable inspekcją
  supermodelu;
- bramka ważonych regionów blokowała brak klastra tylko dla ról zawierających
  `paw`, więc nieruchomy lub nieważony `tail_tip` nie blokował wyniku;
- base tree i animation tree mogą zawierać render-only mesh nodes. Sama
  obecność w animation tree nie wystarcza do uznania węzła za carrier.

## Zaimplementowany kontrakt V3

### Core

- `build_exact_reference_supermodel_motion_contract_v3` buduje kontrakt z
  exact lokalnych macierzy position/orientation/scale odczytanych przez nasz
  binary MDL reader.
- Inventory carrierów powstaje z węzłów sterowanych w wymaganych klipach,
  domknięcia ich przodków oraz jawnie zadeklarowanych węzłów semantycznych.
  Render-only mesh node nie staje się przypadkowo kością.
- `build_exact_motion_carrier_rig_v3` zachowuje geometrię i wagi targetu, lecz
  zamienia nazwy, parenty i local bind carrierów na dokładny kontrakt
  supermodelu.
- Walidacja porównuje immutable inspekcję, kontrakt i wyemitowany carrier.
  Rozjazd kończy się `M2A-SUPERMODEL-REFERENCE-BIND-MISMATCH`.
- Raport rozdziela `sourceBindMaxAbsError`,
  `referenceBindMaxAbsError` i `referenceBindVerified`. Legacy route bez
  inspekcji nie może już twierdzić, że bind jest zweryfikowany.
- Publiczne ścieżki direct material/classic/minimal korzystają z exact
  carriera. `motionCompatible` jest wyliczane z lookupu supermodelu, exact
  bindu, readbacku skina i bramki motion; nie jest stałą `true`.

### Ogólna bramka ważonych regionów

- Wszystkie semantyczne regiony poza strukturalnymi `root` i `motion_root`
  wymagają niepustego klastra ważonych vertexów.
- Brak np. `tail_tip`, `head`, `pelvis`, `neck` albo łapy blokuje pipeline jako
  `M2A-SUPERMODEL-MOTION-ANCHOR-CLUSTER-MISSING`.
- Raport zawiera `anchorClusterMissingCount`; profil jakości ma nazwę
  `REFERENCE_SUPERMODEL_SAMPLED_SURFACE_AND_WEIGHTED_ANCHORS_V4`.
- Reguła jest niezależna od resrefu i działa dla każdego kontraktu
  supermodelu.

### c_wolf i API

- Materializer borzoja używa teraz
  `build_c_wolf_motion_contract_exact_v3(..., &retail_inspection)`.
- Stary builder V2 pozostaje wyłącznie kompatybilnym szablonem
  topology/semantics dla starszych wywołań; nie jest ścieżką produkcyjną demo.
- WASM udostępnia ogólne funkcje:
  `buildExactReferenceSupermodelMotionContractV3Json` oraz
  `buildExactMotionCarrierRigV3Json`.
- Worker Studio ma żądania
  `BUILD_EXACT_REFERENCE_SUPERMODEL_MOTION_CONTRACT` i
  `BUILD_EXACT_REFERENCE_SUPERMODEL_CARRIER`.
- Legacy `buildMotionCorrectedRigV1Json` raportuje obecnie tylko
  `TOPOLOGY_ONLY`, ponieważ bez bytes referencji nie może udowodnić exact
  bindu.

## Weryfikacja offline

Wyniki 2026-08-21:

- `cargo test -p m2a-core --test reference_supermodel_motion` — PASS, 9/9;
- test regresyjny generic tail cluster — PASS;
- `cargo test -p m2a-core --test c_wolf_rig` — PASS, 3/3;
- `cargo test -p m2a-wasm --lib` — PASS, 48/48 dla pełnego zestawu przed
  dodaniem nowego boundary testu; nowy test exact contract Core/WASM — PASS
  (pakiet zawiera obecnie 49 testów);
- `cargo check -p m2a-core --example materialize_borzoi_c_wolf_demo_v1` — PASS;
- `npm run typecheck` w `apps/studio-web` — PASS;
- `npm run build:wasm` — PASS;
- Node WASM boundary używany przez CI — PASS (`M5/M7 Node boundary PASS`);
- realny read-only audit `nwn_base.key` dla `c_wolf` — PASS: 30 carrierów,
  8 wymaganych klipów, 6 wymaganych eventów, exact contract SHA-256
  `1142432ba999a5a027d25bce7a97df3e314b4a51612c2c2dd3e78ae37ed758c8`.

## Kryteria zakończenia i aktualny status

- [x] Wspólny kontrakt exact działa bez warunku na `c_wolf`.
- [x] Rozjazd neutralnego bindu jest blokujący.
- [x] Render-only node nie trafia automatycznie do szkieletu.
- [x] Raport nie może fałszywie oznaczyć niezweryfikowanego bindu jako zgodny.
- [x] Brak ważonego ogona lub innego regionu semantycznego jest blokujący.
- [x] `c_wolf` przechodzi realny offline audit z zasobu retail.
- [x] Core, WASM i worker Studio mają tę samą ogólną ścieżkę API.
- [ ] Ostateczny wynik wizualny wymaga nowego, jawnie dopuszczonego kandydata
  oraz proofu właściciela w Toolset/NWN.

W tej implementacji nie utworzono nowego MOD/HAK/MDL i nie uruchomiono
Toolsetu ani NWN. Zmiana Core nie jest sama w sobie wizualnym proofem ruchu
ogona. Następna materializacja podlega model-iteration gate i osobnemu
lineage/hash handoffowi.
