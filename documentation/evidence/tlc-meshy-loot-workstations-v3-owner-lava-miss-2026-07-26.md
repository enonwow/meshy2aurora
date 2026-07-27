# Odrzucenie układu V3: lawa omija wnętrze dolnej misy

Dokładny kandydat:

- MOD: `m2a_tlcw3_mod.mod`
- nazwa modułu: `Meshy2Aurora TLC Loot Workstations V3`
- Area: `Meshy2Aurora TLC Loot Workstations V3`
- SHA-256 MOD: `06e51513f8e7543ab3d8d9d6b1fa571f8383cd80378e025ce38ec3816b4789fd`
- HAK: `m2a_tlcw3_hak.hak`
- SHA-256 HAK: `3906389593039f96118a1ef393cfe2bcc1f1456f6385491a50d3f90e63f41055`
- model: `m2a_tlcw3_upg`
- blueprint: `m2a_tlcw3_uu`
- Appearance: `16502`

## Decyzja właściciela

- [x] Układ V3 został odrzucony.
- [x] Na dostarczonym obrazie strumień lawy trafia w lewy/skrajny rant, a nie w użytkową powierzchnię dolnej misy.
- [x] Zachowano dokładny obraz wejściowy: `tlc-meshy-loot-workstations-v3-owner-lava-miss-2026-07-26.png`.
- [x] SHA-256 obrazu: `cf4ab69b08bcae2aabe5f60b549a4c83440b48c6ab7934829fcb0b1d4589f563`.

To jest odrzucenie układu autorskiego, nie wynik live proof w Aurorze/NWN:

- `modelVisibility = not_tested`
- `proofCompleteness = missing`

## Przyczyna

Warunek V3 sprawdzał jedynie, czy punkt strumienia mieści się w osiowym bounds powierzchni misy. Punkt na skrajnym rancie spełniał ten warunek, chociaż wizualnie lawa omijała wnętrze zbiornika.

## Minimalna poprawka V4

- [x] Zmienić wyłącznie grupę dolnej podstawy.
- [x] Zachować lawę bez lokalnego przesunięcia, obrotu, skali i zmiany pivota.
- [x] Zachować skalę X podstawy `0.54`, aby nogi pozostały na zewnątrz.
- [x] Przesunąć podstawę w osi Z z `0.08` na `0.14`.
- [x] Poszerzyć podstawę w osi Z z `1.05` na `1.50`.
- [x] Pozostawić dolny pierścień `C37` razem z nogami i z transformacją lokalną identity.
- [x] Zachować przesunięcie Y `0.05`, skalę Y `1.0` i skalę całego modelu `2.5`.
- [x] Wymagać, aby punkt strumienia mieścił się w wewnętrznej części powierzchni misy z co najmniej 40-procentowym marginesem radialnym.
- [x] Sprawdzić układ w webowym viewportcie 3D przed materializacją.
- [x] Wygenerować V4 wspólnym pipeline’em MDL/PWK/2DA/UTP/HAK/MOD.
- [x] Zainstalować dokładne V4 MOD/HAK do natywnych katalogów NWN i potwierdzić zgodność hashy.
