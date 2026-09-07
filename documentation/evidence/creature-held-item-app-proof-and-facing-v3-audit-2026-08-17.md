# Creature held-item app proof i korekta Creature Basis V3

Data: 2026-08-17

Status: `APP_EQUIPMENT_READBACK_VISIBLE_PROXY_PASS / FACING_PIPELINE_ROOT_CAUSE_CONFIRMED / V3_IMPLEMENTED_OFFLINE / V9_NWN_HELD_ITEM_OWNER_RESULT_PENDING`

## Werdykt

Powtarzające się poruszanie Creature tyłem nie wynika z wyboru drugiego
wadliwego modelu. Kanoniczny Void Crystal Knight jest semantycznie skierowany
w źródłowe `+Z`, zgodnie z ustawieniem użytym przez V9. Wspólny pipeline
mapował jednak wybrany przód na Aurora `-Y`. Dwa niezależne, dokładne modele
retail direct-Creature wskazują przód hierarchii w Aurora `+Y`. To wspólna
regresja bazowa pipeline'u.

Kod produktu używa teraz Creature Basis V3: wybrany przód źródła mapuje się na
Aurora `+Y`, source-up `+Y` mapuje się na Aurora-up `+Z`, a wyznacznik
transformacji pozostaje `+1`. Historyczne V2 i zamrożone artefakty V8/V9 nie
zostały przepisane.

Proof aplikacji dla trzymanego przedmiotu jest pozytywny w węższym, uczciwie
zdefiniowanym zakresie: aktualny raport MOD potwierdza wyposażenie
`nw_wswss001` w `right_hand`, binarny MDL zawiera `rhand`, a Studio pokazuje
widoczny short-sword basis proxy przy tej ręce. Nie jest to proof, że NWN
odnalazł i narysował dokładną stockową geometrię `nw_wswss001`.

## Proof aplikacji — broń

Test wykonano przez rzeczywisty interfejs Studio w przeglądarce i skompilowany
WASM/Worker, używając:

- źródła `sample-3d/void-crystal-knight-h1-v1/source.glb`, 9 951 624 B,
  SHA-256 `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`;
- dokładnego `proof-output/creature-held-item-v9/appearance.2da`, SHA-256
  `3fde7fafa7e232e1b1cd0ba0fa9383612ef6d9a5658f00909d7ccd39ed9a1f51`;
- `sourceForward=POSITIVE_Z`;
- `heldWeapon=RIGHT_HAND_BASE_SWORD`;
- `weaponGrip=AUTO_PLUS_OFFSETS`, right roll `+90°`.

Studio zbudowało świeżą, aplikacyjną tożsamość proofu, niezależną od
zamrożonego V9:

- MOD `cd277i4dixonofaq.mod`;
- HAK `ch277i4dixonofaq.hak`;
- MDL `cm277i4dixonofaq.mdl`;
- TGA `ct277i4dixonofaq.tga`;
- hash raportu demo
  `c1e176740c21e474b400d304421e1abd5263936fbd88a8efb7d9c1a093f4ef6b`;
- 19 704 trójkąty i 42 animacje.

Zweryfikowany łańcuch:

1. canonical demo report ma `heldStockWeaponReadback.schemaVersion=2`;
2. weapon ma `resref=nw_wswss001`, `resourceType=2025` i
   `resourceScope=NWN_BASE_GAME`;
3. istnieje dokładnie jeden fixture: `right_hand`, wyposażony resref jest
   identyczny z resrefem broni;
4. binarny readback MDL wskazuje węzeł `rhand`;
5. Studio pokazuje widoczny basis proxy przy `rhand` i jawnie opisuje go jako
   `NWN SHORTSWORD BASIS PROXY` oraz `Not exact item geometry or Toolset proof`.

Regresja UI została usunięta: bez `heldStockWeaponReadback` podgląd broni jest
domyślnie wyłączony. Studio nie może już pokazywać miecza tylko dlatego, że MDL
ma węzeł `rhand`. Parser odrzuca historyczne `heldWeaponReadback`, zły typ
resource, zły scope, liczbę fixture różną od jednego oraz rozbieżny equipped
resref.

Dowody obrazu:

- `output/playwright/creature-held-item-v9-app-proof/source-positive-z-front-audit.png`,
  SHA-256 `a8dc50742c6f913b7baf1b0e111a70398b79e7b965def803763a245798ab539b`;
- `output/playwright/creature-held-item-v9-app-proof/held-item-readback-and-visible-proxy.png`,
  SHA-256 `4fab9cb56ec27670881dc6a310b0e72940f6cea50bf24432f05de8905ef2b8a1`;
- `output/playwright/creature-held-item-v9-app-proof/held-item-readback-and-visible-proxy-close.png`,
  SHA-256 `1f8324b36b3226119c9b942246ec22822212bfe7183b1fad810f1cb466bcee28`.

## Root cause orientacji

### Źródło Void

Viewport pokazuje rzeczywisty model Void oraz strzałkę `+Z` po stronie jego
widocznego przodu. Bezpośredni odczyt JSON z GLB dodatkowo pokazuje:

- `Head/headfront` local translation
  `[0.02057577, 7.3206029, 12.71131]`;
- `head_end` ma lokalne `z=-12.71135`.

Wybór `POSITIVE_Z` dla V9 był więc poprawny. Zmiana wyboru modelu nie usuwała
problemu, bo błąd znajdował się po stronie docelowej osi Aurora.

### Dwa niezależne świadki retail Aurora

- exact retail `c_Direwolf`, SHA-256
  `121b63cd51ff46c3633c740951752110bebdeab7db139719ff6ff7db077f0fd9`:
  `headconjure` ma local Y `0.508174`, a kolejne elementy łańcucha głowy
  rozwijają się ku dodatniemu Y;
- exact retail `c_horror` z read-only `models_01.bif`, offset `29 729 840`,
  długość `393 692`, SHA-256
  `2faf553a0665da200b232bd52d03c0e1d79b88959cabdbe840f35f16e5878c8e`:
  `neck2` Y `0.733923`, `neck1` Y `0.306283`, head Y `0.11389`,
  `headconjure` Y `0.567614`.

Obie niezależne rodziny direct-Creature wskazują semantyczny przód w `+Y`.
Historyczna etykieta V2
`SOURCE_HEADFRONT_AND_RETAIL_NATIVE_NEGATIVE_Y` była błędnym wnioskiem.

## Zakres implementacji V3

- wszystkie cztery wybory source-forward (`+Z`, `-Z`, `+X`, `-X`) mapują
  wybrany wektor dokładnie na Aurora `[0,+1,0]`;
- source-up `[0,+1,0]` mapuje się na `[0,0,+1]`;
- każda macierz ma determinant `+1`;
- rig, geometria, bind pose i animacje korzystają z tej samej polityki V3;
- ścieżki Product, P100K i P300K respektują teraz tę samą wybraną oś podczas
  rzeczywistej konwersji, a nie tylko w summary/manifest;
- report używa `CREATURE_BASIS_V3_RESOLVED`, mapowań
  `*_TO_AURORA_POSITIVE_Y` i evidence
  `SOURCE_HEADFRONT_AND_RETAIL_NATIVE_POSITIVE_Y`;
- handshake Studio/WASM został podniesiony do
  `M2A_STUDIO_WASM_2026_08_17_V2` z capability
  `CARDINAL_XZ_TO_AURORA_POSITIVE_Y_V2`.

## Weryfikacja offline

- `cargo fmt --all -- --check` — PASS;
- `m2a-core` Profile A — 51/51 PASS;
- `m2a-core` model pipeline — 29/29 wykonywanych PASS, 1 witness test ignored;
- rzeczywisty lokalny Void product replay — PASS, 19 704 trójkąty, 42
  animacje, V3 `+Z -> +Y`, determinant `+1`;
- `m2a-wasm --lib` — 45/45 PASS;
- Studio typecheck — PASS;
- Studio unit/component — 296/296 PASS;
- testy bezpośrednio związane z proofem — 53/53 PASS;
- produkcyjny build WASM/Vite — PASS;
- skompilowany Worker/WASM integration — 14/14 wykonywanych PASS, 2
  środowiskowe SKIP.

## Granica V9 i następny krok

Zamrożony V9 pozostaje niezmieniony i nadal zawiera historyczne mapowanie
V2 do `-Y`. Dokładny handoff to:

- test-module: `m2aweapdemo9.mod`;
- module name: `Meshy2Aurora procedural humanoid item placement`;
- Area: `Meshy2Aurora procedural humanoid item placement area`;
- MOD SHA-256
  `da0a5872a3c82e7701696baf789c54a7b6b181e35099c1754c4ce475d2ff3d18`;
- HAK `m2aweaphak9.hak`, SHA-256
  `f3f8691f1a00fd5ef6798cd7e833c540d4852c9ced8ff34c0e9f9eb692a89e1b`.

Wynik właścicielski widoczności trzymanego itemu w dokładnym V9 nadal jest
`not_tested/missing`. Nie utworzono V10 ani nowych MOD/HAK/resrefów. Proof
aplikacji nie zastępuje werdyktu NWN; po zgłoszeniu dokładnego wyniku V9 można
zamknąć diagnozę held-item albo przygotować dozwoloną następną iterację.
