# Creature weapon grip V7 — ready for owner proof — 2026-08-04

## Handoff

1. plik modułu: `m2aweapdemo7.mod`;
2. nazwa modułu w Toolsecie: `Meshy2Aurora Creature Weapon Grip V7`;
3. Area: `Meshy2Aurora Creature Weapon Test V7`;
4. HAK: `m2aweaphak7.hak`;
5. Creature: `V7 RIGHT HAND - module-local UTI longsword - native basis`;
6. blueprint Creature: `m2awrhand7`;
7. modułowy przedmiot UTI: `m2aweapitem7`.

## Co zmieniono względem V6

V7 nie polega na zewnętrznym resrefie przedmiotu. MOD zawiera własny zasób
`m2aweapitem7` typu `2025/UTI`, a `Equip_ItemList` Creature wskazuje dokładnie
ten sam resref w slocie `right_hand`. Jest to clean-room zwykły longsword:

- `BaseItem=1`;
- `ModelPart1/2/3=61/11/11`;
- `PaletteID=36`;
- `Cost=30`;
- bez historycznych pól `xModelPart*` i `Cursed`;
- exact zestaw 19 zwykłych pól UTI sprawdzany regresją.

Model części longsworda został niezależnie porównany z lokalnym indeksem
retailowego `nw_wswls001`; payload retailowy nie został skopiowany do projektu.
Starszy writer V1 pozostał osobnym profilem, więc V7 nie zmienia historycznych
lineage.

## Offline proof

- targeted named-owned-UTI regression: pass;
- weapon-demo regressions: `4/4` pass;
- creature-equipment regressions: `6/6` pass;
- `cargo fmt --all -- --check`: pass;
- materializer example compile check: pass;
- animacje zawierające `rhand` i `lhand`: `42/42`;
- trójkąty: `297190`;
- Appearance row: `15104`, `MODELTYPE=L`;
- grip calibration: `MESHY_H1_PALM_CENTER_NATIVE_ITEM_BASIS_V6`;
- source GLB SHA-256:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`.

Niezależny odczyt gotowego MOD wykrył dokładnie:

- `m2aweapitem7`, type `2025`, 867 bytes, SHA-256
  `b53cbde7f9cbd8d7536652533b470ee29dbdbcede73a665d53ae4d65ecb3b62b`;
- `m2awrhand7`, type `2027`, 3599 bytes, SHA-256
  `09f2c3b5efa55bd314eae5a3ee8ddbd5543cd7b709181b100c462021a8401c81`.

Readback modułu potwierdza `m2aweapitem7` zarówno jako istniejący UTI, jak i
`equippedItemResref` prawej dłoni.

## Exact artifacts i instalacja

- MOD:
  `proof-output/creature-weapon-grip-v7/m2aweapdemo7.mod`, SHA-256
  `d0d04789dbebc1b523c072506b18d0d168e55af08e5fc64afb90b6bac8d810d0`;
- HAK:
  `proof-output/creature-weapon-grip-v7/m2aweaphak7.hak`, SHA-256
  `edc366fbbd18a0e5b707caa78658de9e7dcbd5db8556d0ddc85a21397e6f885d`;
- handoff:
  `proof-output/creature-weapon-grip-v7/handoff.json`, SHA-256
  `3a1c46c2ad14f093f801ceb04b683a56a5d0fdf76f7906dc07b105e58751c4e8`.

Wszystkie pliki z `handoff.generatedFiles` ponownie sprawdzono pod względem
długości i SHA-256: wszystkie zgodne. Exact MOD i HAK skopiowano wyłącznie do
wcześniej nieistniejących celów i potwierdzono byte-for-byte:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aweapdemo7.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aweaphak7.hak`.

Agent nie uruchamiał ani nie kontrolował Toolsetu/NWN. Stan V7 pozostaje
`modelVisibility=not_tested`, `proofCompleteness=missing` do testu właściciela.
