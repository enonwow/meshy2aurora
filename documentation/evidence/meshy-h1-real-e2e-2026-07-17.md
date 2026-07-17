# Meshy H1 real E2E — 2026-07-17

Status: `OPEN / BUILD BLOCKED`.

## Cel testu

Przeprowadzono lokalny, przegladarkowy workflow Studio dla rzeczywistych wejsc:

- GLB: `test-assets/meshy/incoming/h1-humanoid-1500.glb` (lokalny, ignorowany
  przez Git), SHA-256 `3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f`;
- wybrana przez wlasciciela tabela `local-reference-assets/appearance.2da`
  (lokalna, ignorowana przez Git), SHA-256
  `815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a`.

Test uruchomil prawdziwy local-file workflow. Nie przeslal plikow do sieci.

## Dowody lokalne

Wszystkie screenshoty i raporty sa poza Gitem w:

`C:\Projects\meshy2aurora\proof-output\meshy-h1-e2e-2026-07-17`

| Etap | Dowod | Wynik |
| --- | --- | --- |
| Source | `01-source-files-loaded.png` | PASS — oba realne pliki wybrane i krok Inspect aktywny. |
| Inspect | `02-inspect-real-h1-ready.png` | PASS — model: 1 mesh, 1 334 vertices, 1 556 triangles, 1 material, 2 textures, 24 bones, 1 animation clip; `conversionEligible=PASS`. Tabela: 35 kolumn, 15 100 physical rows. |
| Build start | `03-build-ready.png` | PASS — Build Package zostal odblokowany po realnym Inspect. |
| Build | `failure-build-or-review.png` | FAIL — `M4A-MAPPER-SKIN-INVALID` przed emisja artefaktow. |

## Znaleziona blokada

Realny rig Meshy jest poprawnym wariantem glTF, ale obecny `M6` jest proofem
dla wlasnego, syntetycznego riga i mapowania dwoch kosci. Nie jest jeszcze
ogolnym importerem H1.

Konkretne roznice realnego H1:

1. `skin.skeleton` jest opcjonalne i nie wystepuje; Hips jest jedynym root
   jointem, a Armature jest zewnetrznym kontenerem transformu.
2. Rig ma 24 jointy, podczas gdy domyslne M6 mapowanie ma dwa source nodes.
3. Idle ma 72 kanaly: translation i rotation LINEAR, ale rowniez kanały SCALE
   (w tym niejednostkowy Hips) oraz trzy SCALE/STEP. Obecny M4A obsluguje
   tylko translation/rotation LINEAR.

Nastepny etap nie moze udawac konwersji przez usuniecie animacji lub zmiane
wejscia. Wymaga jawnego kontraktu dla inferencji root skina, generacji riga i
mapowania H1 oraz potwierdzonej Aurora-first polityki dla animowanego scale.

## Aktualizacja po implementacji H1

Status poprzedniej sekcji zostal zastapiony wynikiem `PACKAGE MATERIALIZED`.
Nie zmieniano wejsc ani nie usuwano animacji.

| Etap | Dowod | Wynik |
| --- | --- | --- |
| Source | `04-source-real-h1-ready.png` | PASS |
| Inspect | `05-inspect-real-h1-ready.png` | PASS: 1 334 vertices, 1 556 triangles, 24 bones, 1 clip; tabela 35 kolumn / 15 100 rows. |
| Build | `06-build-real-h1-ready.png` | PASS: rzeczywiste wejscia odblokowaly pakietowanie. |
| Review | `07-review-real-h1-materialized.png` | PASS: binary readback i writer semantic diff; HAK, MDL i raporty widoczne w aplikacji. |

Wynik realnej materializacji:

- MDL `204312 B`, SHA-256 `8674b732347376f00aaad58a31e2a4b430582b8822de66a1638f97453658a957`;
- HAK `19688878 B`, SHA-256 `c116611095f30bfc8fde9eeaa914375ba74612416193a39c939506b9eb16d553`;
- wynikowa `appearance.2da` ma dopisany physical row `15100`;
- `Armature|Idle|baselayer` zostal zmapowany do `cpause1`, `4.0333333 s`, `hasMotion=true`.

Wizualny follow-up jest jawnie otwarty: preview Three.js (Y-up) pokazuje
Aurora MDL (Z-up) obrocony. Nie jest to runtime/engine PASS; binary readback i
pakiet przechodza, ale adapter osi preview oraz niezalezny runtime proof
pozostaja osobnym gate'em.

### Korekta preview

Adapter preview zostal poprawiony w tym samym przebiegu: ostatni zapis
`07-review-real-h1-materialized.png` pokazuje stojacy, wygenerowany model po
odczycie binarnego MDL. Runtime/engine proof pozostaje niezaleznym gate'em;
nie jest jednak juz blokada wizualizacji Studio.

### Modul proof

Po materializacji rozszerzono realny przebieg o osobny, generowany artefakt
`m2a_h1proof.mod`. Ekran `08-review-real-h1-proof-module.png` pokazuje w jednym
przebiegu realny model H1, binary readback `PASS`, HAK `m2a_m6p01.hak` oraz
MOD `m2a_h1proof.mod` (2 033 B, SHA-256
`52732975d10bb3718a86dc657a9ead71775eb65fbfe3fe2ed1a727845445a4f7`). MOD ma
przejsc structural own-readback: `module.ifo`, area `ARE/GIC/GIT`, `UTC`, wpis
`Mod_HakList` i `Appearance_Type=15100`. Szczegoly kontraktu i granicy runtime
sa w `suplement-meshy-h1-proof-module-2026-07-17-codex.md`.
