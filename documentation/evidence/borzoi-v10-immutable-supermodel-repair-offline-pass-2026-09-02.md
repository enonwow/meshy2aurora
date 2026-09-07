# `m2aborzmod10.mod` — naprawa immutable-supermodel V10

Moduł wyświetlany w Toolsecie: `Meshy2Aurora Borzoi c_wolf Demo V10`.

Dokładny Area: `Meshy2Aurora Borzoi Test Area V10` (`m2aborzarea10`).

Status: `offline_product_pass_native_hak_collision`. Produkt jest poprawnie
zmaterializowany w repozytorium, ale nie jest oznaczony jako
`ready_for_owner_proof`, ponieważ natywny docelowy HAK istnieje i ma inny hash.
Agent nie uruchamiał Toolsetu ani NWN i nie nadpisał kolidującego pliku.

## Naprawiona semantyka supermodelu

- wybrany supermodel: `c_wolf`;
- retail MDL SHA-256:
  `a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726`;
- 30/30 carrierów i 1088 kontrolerów;
- hierarchia, nazwy, rodzice i lokalne macierze bind są niezmienne;
- `jointOverrideCount=0`;
- bazowy i wynikowy rig SHA-256 są identyczne:
  `85a1ae7e1355922c4a603d3883823df054fc8b3b7076375a6d2040dfaa9a6069`;
- model zawiera 0 lokalnych animacji i dziedziczy 42/42 klipy z `c_wolf`;
- tylko siatka jest rejestrowana względem referencji; wagi są generowane i
  refinowane względem dokładnych jointów wybranego supermodelu.

## Pełny oracle ruchu

Raport:
`artifacts/diagnostics/borzoi-v10-immutable-c-wolf-bind-v7/base-preview-report.json`.

- status diagnostyki: `APPLIED_PREVIEW_OFFLINE_PASS`;
- status produktu: `REFERENCE_SUPERMODEL_CREATURE_PRODUCT_MATERIALIZED`;
- admission: `PASS`;
- motion quality: `PASS`;
- wadliwe komponenty: 0;
- hard edges: 152754 / 307262;
- triangle collapse: 404 / 5121;
- triangle expansion: 148 / 10242;
- min/max edge ratio: 0.0625522956 / 14.5134801865;
- min/max triangle-area ratio: 0.0080598202 / 63.7927474976;
- appendage/tail relative-motion pairs: 38/38, PASS.

## Artefakty produktu

Katalog:
`C:\Projects\meshy2aurora\artifacts\borzoi-v10-immutable-product-v6`.

- MOD `m2aborzmod10.mod`, 15295 B, SHA-256
  `0a97d8fd3429fc359b7d56bd36b3b7a3927f557af3e8f073c06233f24e234456`;
- HAK `m2aborzhak10.hak`, 40011886 B, SHA-256
  `6c696f6664cd17a0912d818a8bc5a049012f7bed6ee77395b9cd9609f8fd485f`;
- MDL `m2aborzcre10.mdl`, SHA-256
  `97b79753bf716036bb814931c79ac61156d8559a5a249931bfbdf792aa1f2ea1`;
- diffuse `m2aborztex10.tga`, SHA-256
  `cdd9ac9310740952f1f1c804c6ec66283f185be0c5d15a549873ed4ad3d15cbe`;
- MTR `m2aborztex10_m0.mtr`, SHA-256
  `7dd199ba5f24144860d826528b231ef015a0c8629f793501f5dc55195aa6cb7f`.

## Natywna instalacja

- `Documents/Neverwinter Nights/modules/m2aborzmod10.mod` istnieje i jest
  byte-for-byte identyczny z nowym źródłem (SHA-256
  `0a97d8fd3429fc359b7d56bd36b3b7a3927f557af3e8f073c06233f24e234456`),
  więc może zostać ponownie użyty;
- `Documents/Neverwinter Nights/hak/m2aborzhak10.hak` istnieje jako stary HAK:
  41150702 B, SHA-256
  `8a168fdc4cdb4b3439f08940d53c3e774cf23c13979c58064aafc379abfd3186`;
- nowy HAK ma inny hash i nie został nadpisany, usunięty ani przemianowany;
- warunek wznowienia: właściciel usuwa albo przenosi stary natywny HAK poza
  katalog `hak`, po czym agent może skopiować dokładny nowy HAK i zweryfikować
  byte-for-byte SHA-256 bez tworzenia kolejnej iteracji modelu.

## Testy

- integracyjne Rust supermodel: 37/37 PASS;
- testy jednostkowe nowych reguł komponentowych: PASS;
- Studio supermodels: 27/27 PASS;
- Studio TypeScript typecheck: PASS;
- `cargo check -p m2a-wasm`: PASS.
