# Creature — wdrożenie etapów 1, 4, 5, 6 i 7 (2026-08-19)

## Granica wykonania

Zmiany dotyczą produktu, API WASM/Worker i Studio. Nie utworzono nowej iteracji
MOD/HAK, nie uruchamiano Aurora Toolset ani NWN i nie zmieniano istniejącego
kandydata proof. Końcowy proof wizualny pozostaje własnością właściciela.

## Etap 1 — pełne wyposażenie Creature

- Aktywna ścieżka Studio nie używa już `EquippedRes=nw_wswss001`.
- `CreatureDemoAuthoringV1` zapisuje jeden wspólny kontrakt gameplay do GIT i
  UTC, a `CreatureEquipmentLoadoutV2` osadza kompletne struktury itemów.
- Domyślny przepis V10 to `nw_wswbs001`, `BaseItem=3`, części
  `[41,11,11]`, feat `44`, koszt `70` i strref `168`.
- Dostępne są sloty `RIGHT_HAND`, `LEFT_HAND` i `BOTH_HANDS`; walidator blokuje
  kolizje slotów oraz broń dwuręczną z zajętym offhandem.
- Readback odrzuca każdy powrót `EquippedRes` i obecność zastępczego UTI w MOD.
- Zmiana wyposażenia otrzymuje osobną tożsamość MOD/UTC i nie zmienia
  tożsamości MDL/HAK produktu.

## Etap 4 — runtime envelope i authoring UTC

- `CreatureRuntimeEnvelopePolicyV1` obsługuje profil średniego humanoida,
  wartości wyprowadzone z bounds oraz jawny envelope.
- Wartości `HEIGHT`, `HITDIST`, `PERSPACE`, `CREPERSPACE`, `PREFATCKDIST`,
  `TARGETHEIGHT`, `WALKDIST`, `RUNDIST`, `SIZECATEGORY`, `PERCEPTIONDIST` i
  `FOOTSTEPTYPE` trafiają do nowego wiersza `appearance.2da`.
- Pełny authoring blueprintu obejmuje statystyki, klasę, poziom, featy,
  faction, portrait, soundset, flagi i komplet hooków skryptowych.
- Edycja gameplay zmienia MOD/UTC, ale builder tej operacji nie przyjmuje ani
  nie przebudowuje MDL.

## Etap 5 — profile chwytu rodzin itemów

- Kontrakt posiada rodziny `SWORD`, `AXE_MACE`, `SPEAR_POLEARM`,
  `BOW_CROSSBOW` i `SHIELD`.
- Każda rodzina ma inną właściwą macierz obrotu o wyznaczniku `+1`.
- Automatyczna baza rodziny jest stosowana przed niezależnymi ręcznymi
  offsetami R/P/Y prawej i lewej dłoni.
- Studio udostępnia wybór rodziny. Kalibracje pozostają oznaczone
  `OFFLINE_INITIAL_CALIBRATION_OWNER_PROOF_REQUIRED`; zgodnie z decyzją
  właściciela korekta obrotu broni jest osobnym późniejszym tematem.

## Etap 6 — MotionPack i quadruped intake

- `CreatureMotionPackV1` jest związany z dokładnym `sha256:<GLB>`.
- Deklarowane jointy i klipy muszą istnieć dokładnie raz w realnym inventory
  źródła; sama deklaracja nazw nie wystarcza.
- Humanoidowy builder filtruje warianty ataku według wybranej rodziny broni,
  dzięki czemu mapowania miecza, włóczni i łuku nie nadpisują się.
- Profil `QUADRUPED` wymaga root/spine/head oraz czterech jointów kontaktowych
  łap. Osobne API Core/WASM/Worker wykonuje source-bound intake bez budowania
  lub mutowania modelu.
- Humanoidowy emitter nadal jawnie odrzuca MotionPack quadrupeda. Oznacza to,
  że nie ma cichego użycia humanoidowego rigu dla zwierzęcia; pełny emitter
  quadrupeda wymaga odrębnego kontraktu rigu i appearance.

## Etap 7 — materiały i wydajność

- Animowany Creature obsługuje `AURORA_CLASSIC_SAFE` i `NWN_EE_MTR`.
- Profil MTR używa istniejącego kompilatora PBR, generuje tangenty, rozszerza
  gotowy animowany MDL, ponownie parsuje finalne bajty oraz pakuje TGA/TXI/MTR.
- Manifest model package akceptuje natywne typy TXI `2022` i MTR `2072`.
- Raport produktu V4 zawiera runtime envelope, profil materiału, readback
  rozszerzenia materiałowego, MotionPack oraz raport wydajności.
- Presety `COMPACT`, `STANDARD`, `HIGH`, `MAXIMUM` są celami ostrzegawczymi
  50k/100k/200k/300k. Jedyny hard limit pozostaje wspólnym limitem produktu
  `300_000`; dokładnie 300k przechodzi, 300001 jest blokowane.

## Automatyczne kryteria zakończenia

- Pełny loadout przechodzi round-trip GIT/UTC i mutacja do `EquippedRes` jest
  odrzucana.
- Wszystkie pięć baz chwytu jest różnych i ma wyznacznik `+1`.
- Bounds-derived runtime envelope przechodzi walidację zależności odległości.
- Zmiana HP zmienia MOD/UTC przy niezmienionym resrefie zewnętrznego HAK.
- Quadruped intake przechodzi tylko z czterema rzeczywistymi kontaktami.
- Animowany produkt H2 z `NWN_EE_MTR` zachowuje animacje i zawiera MTR/TXI.
- Studio typecheck i testy aktywnej ścieżki wyposażenia przechodzą.

Wynik tych bramek jest dowodem offline, nie werdyktem wizualnym Toolset/NWN.
