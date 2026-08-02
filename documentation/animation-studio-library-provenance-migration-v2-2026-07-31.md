# Migracja Animation Studio V1 → V2 dla biblioteki animacji

Data: 2026-07-31  
Status: `IMPLEMENTED / BACKWARD-COMPATIBLE`

## Powód wersjonowania

`LIBRARY_PRESET_COPY` zapisuje więcej proweniencji niż źródła dostępne w
pierwotnym dokumencie Animation Studio V1. Dodanie tego wariantu do zapisanego
V1 bez migracji zmieniłoby istniejący wire contract pod tą samą wersją.
Dlatego:

- dokument V1 nadal jest odczytywany i serializowany bez zmian;
- V1 z `LIBRARY_PRESET_COPY` jest odrzucany z diagnostyką blokującą;
- `Use as template` wywołuje deterministyczną migrację i zapisuje dokument
  jako `schemaVersion: 2` przed dodaniem klipu;
- V2 zachowuje wszystkie pola, identyfikatory, klatki, wydarzenia, status i
  rewizję V1; zmienia wyłącznie numer wersji kontraktu;
- dokument V2 może przechowywać `LIBRARY_PRESET_COPY` z pełnym
  `presetId`, `presetVersion`, `presetMotionSha256`, `catalogSha256`,
  `rigSignatureSha256`, autorami, licencją, źródłem katalogu i trybem bindu;
- nieznana wersja inna niż 1 lub 2 nadal fail-closed.

## Implementacja

- Core: `migrate_animation_studio_document_v1_to_v2()` i walidator wersji w
  `crates/m2a-core/src/animation_studio.rs`.
- Studio: `migrateAnimationStudioDocumentV1ToV2()` i automatyczna migracja w
  akcji `Use as template`.
- Persistence i backup: parser akceptuje V1/V2, zachowuje V2 i pełną
  proweniencję bez degradacji do V1.
- Wynik produktu: canonical result akceptuje i raportuje
  `animationStudioSchemaVersion` równe 1 albo 2.

## Reguły integralności V2

`sourceClipFingerprint` kopii bibliotecznej musi być identyczny z
`libraryPreset.presetMotionSha256`. Core, parser Studio oraz parser canonical
result odrzucają rozbieżność. Inne rodzaje źródła nie mogą zawierać
`libraryPreset`.

## Testy migracji

- V1 bez biblioteki nadal przechodzi dotychczasowe testy i zachowuje frozen
  fingerprint.
- V1 z `LIBRARY_PRESET_COPY` jest odrzucany ze wskazaniem
  `authoredClips[n].source.kind`.
- migracja V1→V2 jest deterministyczna;
- `Use as template` podnosi wersję do 2 i dodaje jeden Draft bez modyfikowania
  presetu;
- V2 round-tripuje przez serializację, IndexedDB i backup projektu;
- niezgodny motion hash w proweniencji jest blokowany.
