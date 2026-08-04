# TLC Guard Longsword V3 — diagnoza orientacji i poprawka pipeline'u

Data: 2026-08-03

## Dokładny kandydat

- moduł testowy: `m2atgls3.mod`
- nazwa modułu w Toolsecie: `Meshy2Aurora TLC Guard Longsword item v3`
- Area: `TLC Guard Longsword Item Model Color V3`
- SHA-256 MOD: `7ca0760645d88660eaec96c524adf0d3c7dbe823aa68c4280c2e43487efe7748`
- HAK: `m2atglh3.hak`
- SHA-256 HAK: `00f40b6585c0942e95b03feb0ec6f588427ca2420e19f46e1ac23b02c0325c2d`
- blueprint: `m2atglu3`
- SHA-256 UTI: `5456b41c32e410231d7a5810981e3e02d9fc4f09262e144c3648e987f26af9a0`

Właściciel potwierdził dla tego dokładnego lineagu:

- Toolset `modelVisibility=visible`;
- Toolset `proofCompleteness=verified`;
- akceptacja wizualna nie przeszła: model leży poziomo, podczas gdy detaliczny WSwLs stoi pionowo.

Dowód i jego hashe są związane w `tlc-guard-longsword-v3-owner-toolset-orientation-result-2026-08-03.json`.

## Przyczyna

Referencyjne party `wswls_b_023`, `wswls_m_063` i `wswls_t_023` składają się w lokalnej osi `+Y`. Ich kontrolery pozycji oraz obwiednie geometrii przesuwają kolejne segmenty Bottom/Middle/Top właśnie po Y.

Zamrożone V3 zostało dopasowane przez `ORDERED_SLOT_AXIAL_ENVELOPE_SEAM_GATE_V1`. Ten algorytm obracał długą oś każdego źródła do `+Z`, centrował X/Y i przesuwał segmenty kursorem Z. Dla trzech źródeł Meshy, których długą osią już było Z, rotacja pozostała identycznością. Aurora złożyła więc poprawne trzy modele, ale cały miecz miał błędną orientację o 90 stopni.

Klasa błędu: `ITEM_PART_COMPOSER_AXIS_MISMATCH`.

## Zaimplementowana poprawka offline

Nowa ścieżka produktu używa algorytmu:

`ORDERED_SLOT_AXIAL_ENVELOPE_SEAM_GATE_V2_AURORA_Y`

Kontrakt tej wersji:

1. rozpoznaje wspólną długą oś źródeł;
2. obraca ją deterministycznie do aurorowej osi `+Y`;
3. centruje segment w osiach poprzecznych X/Z;
4. składa Bottom/Middle/Top rosnąco po Y;
5. zapisuje `axialTargetAxis=1` w hashowanym raporcie;
6. nadal wymaga dotknięcia sąsiednich partów i braku przecięcia niesąsiednich partów.

WASM, Worker i UI przyjmują do nowego builda wyłącznie raport `V2_AURORA_Y` z `axialTargetAxis=1`. Zamrożona ścieżka `V1` pozostaje dostępna tylko dla byte-for-byte reprodukcji V3; jej znany hash rozwiązania nadal wynosi:

`ab335afb5b1b6c73611ffd7243201455e07b29e6fe256d0344bfc9aec829e15a`

Test na dokładnych trzech kanonicznych GLB potwierdza dla nowej ścieżki:

- oś źródłowa Z i oś docelową Y dla każdego partu;
- długości Y `0.22`, `0.08`, `0.90`;
- centrowanie w X/Z;
- seam gate `PASSED`;
- zachowanie wszystkich `16 356` trójkątów podczas materializacji MDL.

## Korekta granicy pipeline'u po wyniku właściciela

Właściciel doprecyzował, że kryterium dotyczy modelu w oknie `Item Properties`, a nie orientacji instancji w Area. Pipeline został więc dodatkowo domknięty na dwóch granicach:

1. Dla ModelType 2 Worker wymaga raportu `V2_AURORA_Y` i po zapisaniu każdego MDL porównuje obwiednię z binarnego readbacku z dokładną obwiednią zapisaną w hashowanym raporcie fit. Wynik jest raportowany jako `BINARY_MDL_BOUNDS_MATCH_FIT_V1`. Rotacja samego placementu w Area nie może spełnić tego warunku.
2. Standardowy MOD dla ModelType 2 używa profilu `ITEM_ONLY_GROUND_ITEM_V1`: zawiera dokładnie jeden prawdziwy item, zero wpisów `Creature List` i zero zasobów UTC. ModelType 2 nie alokuje już resrefu creature i nie wymaga `equippedProofContext`.

Profile equipped pozostają wyłącznie dla kompozytorów, których prezentacja rzeczywiście wymaga właściciela modelu postaci, obecnie CAPART armor oraz cloak. Historyczny V3 zachowuje creature witness wyłącznie jako element zamrożonego, odtwarzalnego lineagu; nie jest wzorcem dla następnego dema.

## Bramka następnego artefaktu

Nie utworzono ani nie zainstalowano V4. Ponieważ dokładne V3 jest widoczne w Toolsecie, właściciel musi najpierw sprawdzić ten sam `m2atgls3.mod` z tym samym `m2atglh3.hak` w NWN. Stan NWN pozostaje:

- `modelVisibility=not_tested`;
- `proofCompleteness=missing`.

Dopiero świeży werdykt NWN dla dokładnego V3 może dopuścić wygenerowanie nowego lineagu z poprawką +Y.
