# TLC Ship Placeable — aged-wood texture offline preview

Data: 2026-08-01

Status: `OFFLINE_PREVIEW_VERIFIED / NOT_A_PROOF_CANDIDATE / OWNER_PROOF_UNCHANGED`

## Cel i zakres

Zastąpienie diagnostycznego teal override teksturą, która czytelnie koduje
statek w budowie: ciepłe postarzane drewno kadłuba i pokładu, ciemniejsze
wiązania oraz jaśniejsze brudne płótno. Zmieniono wyłącznie obraz base-color.
GLB, geometria, UV0, material slot, hierarchia i triangle count pozostały
bez zmian.

Przebieg jest porównaniem offline w Studio. Nie utworzono MOD/HAK/MDL/UTP,
nie uruchamiano Aurora Toolset ani NWN i nie powstała nowa iteracja lineage
proof. Istniejący kandydat zachowuje `owner_visual_result: not_tested`.

## Wejście

- asset id: `tlc-ship-under-construction-s1-p150k-v1`;
- źródło: `sample-3d/tlc-ship-under-construction-s1-p150k-v1/source.glb`;
- source byte length: `11 498 076`;
- source SHA-256 przed i po przebiegu:
  `61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278`;
- triangles z manifestu: `152 574`;
- material slots: `1`;
- UV: `UV0 present`;
- embedded source base-color: `image/jpeg`, SHA-256
  `21c7620419351a1b927958f71c32a181bab576bc8db94e3227c98fc3f3f17c43`;
- `baseColorFactor: [1, 1, 1, 1]`, `metallicFactor: 0`,
  `roughnessFactor: 0.8`.

## Recipe tekstury

Tryb: built-in image editing na oryginalnym atlasie base-color.

Kontrakt promptu: zachować pozycję, granice, skalę i orientację wszystkich
fragmentów atlasu; wykonać tylko wyraźny warm color-grade. Drewno otrzymuje
paletę chestnut/dark oak, płótno warm ivory/dirty canvas, liny dark tan, a
okucia charcoal brown. Wykluczone są: zmiana atlasu, nowa ilustracja, tło,
światło kierunkowe, cienie, AO, połysk, napisy i przezroczystość.

Wybrany wariant: `aged-ship-wood-v2.png`:

- byte length: `2 967 040`;
- SHA-256:
  `faf87999e85e72af2b55610640abbd3ac4342408650bb20aab2ae7a58605efbb`;
- obraz fully opaque PNG;
- ograniczenie: model ma jeden material slot, więc pipeline nie może sterować
  drewnem i płótnem niezależnymi suwakami; rozdział zachodzi wewnątrz atlasu.

## Wynik resolvera

- Worker response: `PLACEABLE_TEXTURES_RESOLVED`;
- resolved authoring SHA-256:
  `07e2b48c9cd7669f01a8b19d2f962c665671d28c06f9e23a2d43c52a931f1b5b`;
- output TGA resref: `pt59529201`;
- output TGA SHA-256:
  `43933a0f2a1cb9b0a6d47d088ad06d15ab917d2077bfbcd042a6726e16ba904e`;
- console errors: `0`;
- UI readback: `texture EDITED`, `UV0 preserved`, `alpha OPAQUE`,
  `diffuse only`.

## Artefakty lokalne

Katalog: `output/playwright/ship-texture-copy/`

- `aged-ship-v2-original-source.png` — SHA-256
  `4acb6256d223c74f93aa00335e945fc2da7c1c8a40c93fe327fa84ec755c3073`;
- `aged-ship-v2-edited.png` — SHA-256
  `1ce0caa148d1512c139b6cfce1cfd46a3eccffc06e7235114182d4f826d8fd33`;
- `aged-ship-v2-pipeline-panel.png` — SHA-256
  `95bc860043320cb981d6ec65dd494502ea2d8ed35ed57bacc3c695873e260abe`;
- `pipeline-state-aged-ship-v2.json` — maszynowy readback resolvera.

Kadry Source i Edited używają byte-identical GLB, tej samej kamery, orientacji
i zoomu. `Show PWK` jest wyłączone. Jedyną zmienną jest aktywna tekstura.

## Werdykt

Wersja `aged-ship-wood-v2` jest czytelniejsza od źródła jako drewniany statek:
kadłub i pokład są wyraźnie cieplejsze, a płótno pozostaje jasne. Podmiana
przeszła pełną ścieżkę Studio -> Worker -> WASM/Core -> exact TGA bez zmiany
geometrii i UV. Jest to zakończony wynik offline, nie proof wizualny w
Aurora/NWN.
