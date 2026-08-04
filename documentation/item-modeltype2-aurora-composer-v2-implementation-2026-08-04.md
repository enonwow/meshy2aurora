# Item ModelType 2 — implementacja kontraktu Aurora Composer V2

## Wynik

Pipeline nie kwalifikuje już modelu długiego miecza na podstawie samej zgodności obwiedni. Dla `ModelType 2` zapisuje trzy niezależne MDL-e zgodne ze strukturą retail, emuluje ich składanie w kolejności `ModelPart1`, `ModelPart2`, `ModelPart3` i blokuje Build, jeżeli wynik binarnego readbacku nie spełnia kontraktu.

Wynik właścicielski V4 został zapisany w `documentation/evidence/tlc-guard-longsword-v4-owner-item-properties-result-2026-08-04.json` jako `modelVisibility=visible`, `proofCompleteness=verified`, `visualAcceptance=failed`. To jest podstawa dopuszczenia jednej iteracji V5.

## Przyczyna V4

- V4 zapisywała pozycję i orientację na root node każdego modelu. Retailowe party `WSwLs` mają controllerless model root, natomiast kontrolery transformacji należą do bezpośrednich dzieci Trimesh.
- Stara bramka porównywała boundsy MDL z raportem fit, ale nie odtwarzała semantyki appendu Aurory dla Bottom, Middle i Top.
- Luz mniejszy od tolerancji był uznawany za wystarczające połączenie, mimo że retailowe party celowo zachodzą na siebie osiowo.
- Nazwy dzieci geometrii nie były wymagane jako globalnie unikalne między trzema dołączanymi modelami.

## Zaimplementowany kontrakt

### Core i writer MDL

- Profil `ItemPartStaticRigidAuroraComposerV2` zapisuje controllerless model root.
- Obrót źródła Meshy z osi `+Z` do osi itemu `+Y` jest wypiekany w pozycje, normalne i tangenty geometrii.
- Każde bezpośrednie dziecko Trimesh ma kontrolery `position` i identycznościowy `orientation`; translacja z raportu fit należy do Trimesh, nie do root node.
- Dzieci mają flagi `0x21`, root ma `0x01`, a nazwy są deterministycznie wyprowadzane z resrefu: `g_<resref>` albo `g_<resref>_NN`.
- Nagłówek modelu używa retailowej obwiedni broni `[-5,-5,-1]` / `[5,5,10]`; właściwe boundsy są liczone node-aware z geometrii i kontrolerów.

### Fit i złącza

- `ItemFitReportV2` opisuje kotwice Bottom/Top oraz dwa sąsiednie złącza.
- Algorytm `ITEM_MODELTYPE2_CONNECTOR_OVERLAP_FIT_V3_AURORA_Y` wymaga dodatniego overlapu w dozwolonym przedziale.
- Bottom–Top nadal muszą pozostać bez niepożądanego overlapu.
- Raport fit jest hashowany i ponownie walidowany przed Buildem.

### Bramka kompozycji

- `ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V2` parsuje gotowe binarne MDL-e.
- Weryfikuje kolejność partów, tożsamość resrefów, hierarchię, właściciela kontrolerów, flagi node, unikalne nazwy, retailowy nagłówek, node-aware boundsy oraz liczbę trójkątów.
- Worker uruchamia bramkę osobno dla colorwayów 1, 2, 3 i 4, czyli łącznie dla 12 MDL-i.
- Stary model V4 z kontrolerami na root node jest jawnie odrzucany kodem `ITEM-MODELTYPE2-ROOT-CONTROLLERS`.

### WASM i Studio

- Dodano granice WASM V3/V2 dla budowy partu, fitu, walidacji raportu i walidacji spakowanego zestawu MDL.
- Worker używa nowego profilu wyłącznie dla `ModelType 2`; pozostałe przypadki Item zachowują wcześniejszą trasę.
- Ekran Review pokazuje wynik kontraktu appendu, kolejność partów, właściciela transformacji oraz wynik każdego colorwayu.
- Relacja sąsiednich partów nazywa się `ADJACENT_CONNECTED` i dopuszcza wymagany overlap; overlap niesąsiednich partów nadal blokuje Build.

## Kryteria ukończenia i wynik

- [x] Controllerless root i transformacje na bezpośrednich dzieciach Trimesh.
- [x] Obrót osi wypieczony w geometrię.
- [x] Unikalne nazwy node wyprowadzane z resrefu.
- [x] Dodatnie, jawne nakładanie złączy Bottom–Middle i Middle–Top.
- [x] Brak overlapu Bottom–Top.
- [x] Node-aware append conformance dla wszystkich czterech colorwayów.
- [x] Zachowane 16 356 trójkątów trzech rzeczywistych źródeł Meshy.
- [x] UTI: istniejący `BaseItem 1`, `Identified=true`, wartości `23/63/23`.
- [x] Demo: jeden prawdziwy Item, zero creatures.
- [x] MOD/HAK zainstalowane i zweryfikowane bajt w bajt.
- [ ] Ocena wizualna w Aurora Toolset przez właściciela.

## Weryfikacja

- `cargo test -p m2a-core --test item`: 41 passed.
- `M2A_RUN_ITEM_MESHY_CORPUS=1 cargo test -p m2a-core --test item env_gated_real_longsword_parts`: 2 passed na kanonicznych GLB-ach.
- `cargo check -p m2a-wasm`: passed.
- `cargo check -p m2a-core --example materialize_tlc_guard_longsword_v3`: passed.
- `npm run typecheck`: passed.
- `npm test -- src/features/item`: 28 passed.
- `vitest ... tests/browser/item-worker.integration.ts`: 2 passed w Chromium.

Pełny niezwiązany zestaw `cargo test -p m2a-wasm` ma dwa istniejące błędy z powodu brakującego lokalnego payloadu `sample-3d/h2-clockwork-sentinel-1500/source.glb`; nowe granice Item są zbudowane i wykonane przez przechodzący test Worker/WASM.
