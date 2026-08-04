# ModelType 2: pełna orientacja i natywne ikony — implementacja 2026-08-04

## Wynik audytu V5

Wynik właścicielski dla `m2atgls5.mod` potwierdził dwie niezależne wady:

- fitter wiązał wyłącznie oś długości z Aurora `+Y`, pozostawiając obrót wokół tej osi bez kontraktu; szeroka płaszczyzna miecza trafiała w głębokość kamery i model był prezentowany krawędzią;
- ikony Bottom/Middle/Top były generowane projekcją izometryczną i mimo wspólnego płótna składały się w krótką ukośną linię.

Źródłem dopuszczenia iteracji jest `documentation/evidence/tlc-guard-longsword-v5-owner-item-properties-result-2026-08-04.json`.

## Zaimplementowane poprawki

1. `ItemFitReportV3` zapisuje kompletną prawoskrętną ramę orientacji, a nie tylko oś długości.
2. Deterministyczna klasyfikacja wspólnych osi trzech GLB mapuje:
   - źródłową oś długości na Aurora `+Y`;
   - źródłową szerokość na Aurora `+Z`;
   - źródłową głębokość/front na Aurora `+X`.
3. Automatyczny fit przechodzi tylko, gdy szerokość jest jednoznacznie większa od głębokości (`widthToDepthRatio >= 1.10`) i wyznaczona macierz ma wyznacznik `+1`.
4. Dotychczasowy kontrakt V5 został zachowany: trzy part-y, długości `0.22/0.08/0.90`, kontrolery translacji na dzieciach Trimesh, jawny overlap sąsiadujących łączników i pełny odczyt złożonego MDL.
5. Projekcja ikon jest ortograficzna od frontu ramy Aurora. Każdy z trzech partów emituje niezależny RGBA TGA na dokładnie tym samym płótnie `32x128`.
6. Walidator `ITEM_MODELTYPE2_ICON_LAYER_COMPOSITE_V3` składa warstwy wyłącznie w kolejności `ModelPart1/ModelPart2/ModelPart3`, bez ponownego centrowania i skalowania poszczególnych warstw. Blokuje pusty part, zły rozmiar, złą kolejność, clipping i niepełną pionową sylwetkę.
7. Ten sam kontrakt jest aktywny w core, WASM, Workerze oraz raporcie Studio. Eksport pakietu nie przejdzie po samym sukcesie konwersji: wymaga fitu pełnej ramy, conformance MDL i conformance ikony dla wszystkich czterech wariantów koloru.

## Kryteria ukończenia i stan

- [x] Kąt wokół osi długości wynika z geometrii trzech GLB, a nie ze stałej dobranej wizualnie.
- [x] Docelowa rama to głębokość `X`, długość `Y`, szerokość `Z`; wyznacznik wynosi `+1`.
- [x] Rzeczywiste źródła dają `widthToDepthRatio=3.6271276474` i `status=PASSED`.
- [x] Wszystkie 16 356 trójkątów pozostaje w modelu; limit produktu 300 000 nie jest przekroczony.
- [x] Bottom/Middle i Middle/Top mają wymagany jawny overlap oraz przechodzą powierzchniowy seam gate.
- [x] Wszystkie cztery colorwaye przechodzą node-aware append conformance.
- [x] Wszystkie cztery colorwaye przechodzą trzywarstwowy pionowy icon conformance.
- [x] Wybrany kolor `3` ma złożony bbox `[2,11]..[30,116]`, axial fill `0.8203125` i 497 widocznych pikseli; krótka linia z V5 jest osobnym negatywnym testem regresji.
- [x] UTI odczytuje się jako `Identified=true`, BaseItem `1`, `ModelPart1=23`, `ModelPart2=63`, `ModelPart3=23`.
- [x] MOD i HAK zostały odczytane semantycznie, zamrożone, zainstalowane do katalogów użytkownika NWN i zweryfikowane byte-for-byte.
- [ ] Właściciel potwierdzi prezentację w Aurora Item Properties. Jest to celowe, jedyne pozostające kryterium wizualne; agent nie uruchamiał Toolsetu ani NWN.

## Poprawka po wyniku właścicielskim V6

Właściciel potwierdził, że model w dużym lewym viewportcie V6 jest poprawny, ale mała ikona jest obrócona o 180°. Wynik zapisano w `documentation/evidence/tlc-guard-longsword-v6-owner-item-properties-result-2026-08-04.json`.

Aktywna ścieżka ikon została zaktualizowana do `LONG_VERTICAL_PART_ORDER_V2`:

- wspólna rama 2D wszystkich trzech warstw jest obracana o 180°;
- walidator wymaga kolejności Top nad Middle nad Bottom w odczytanym TGA;
- kompletna, lecz odwrócona ikona V6 jest negatywnym testem regresji;
- kandydat V7 zachowuje wszystkie MDL i tekstury V6 byte-for-byte, a zmienia wyłącznie warstwy ikon oraz opakowanie kandydata.

## Walidacja automatyczna

- `cargo test -p m2a-core --test item`: 44/44 PASS;
- `cargo check -p m2a-wasm`: PASS;
- `cargo fmt --all -- --check`: PASS;
- `npm run typecheck`: PASS;
- `npm test -- src/features/item`: 28/28 PASS;
- worker integration dla `item-worker.integration.ts`: 2/2 PASS.

Dokładny raport V6 znajduje się w `C:\Projects\meshy2aurora\proof-output\tlc-guard-longsword-item-properties-v6-20260804\ready-for-owner-proof.json`.
