# Instrukcja użytkownika: Creature, Animation Studio i Placeable

Stan: MVP przed końcowym gate E7
Data: 2026-07-29

## Wspólna zasada projektu

1. Utwórz lub otwórz projekt.
2. Wybierz target: `Creature` albo `Placeable`.
3. Przejdź kolejno przez `Source → Inspect → ... → Build → Review → Download`.
4. Po zmianie źródła, mappingu albo authoringu wykonaj nowy Build. Aplikacja
   blokuje pobieranie artefaktów ze starej rewizji.
5. Zapis projektu przechowuje ustawienia i referencje do plików, ale po
   ponownym otwarciu trzeba wskazać lokalne payloady GLB/2DA o tych samych SHA.

Modele źródłowe są przetwarzane lokalnie w przeglądarce przez Worker/WASM.
Plik modelu nie jest wysyłany do backendu aplikacji.

## Creature

### 1. Source

- wybierz właścicielski plik Meshy GLB;
- wybierz bazowy `appearance.2da`;
- potwierdź prawa i provenance;
- aplikacja obliczy SHA źródła i przypisze je do bieżącej rewizji projektu.

### 2. Inspect

Sprawdź:

- liczbę trójkątów — wspólny limit produktu wynosi 20 000;
- mesh, skin, kości, tekstury i inventory klipów;
- błędy blokujące oraz ostrzeżenia wymagające decyzji.

### 3. Animation Mapping

Katalog zawiera 42 bazowe stany bezpośredniego creature Aurory. To standard
wyjściowy silnika, a nie lista stanów pobrana z Meshy. Dla każdego stanu:

- przypisz klip źródłowy;
- wybierz świadomy fallback;
- albo użyj wspieranej animacji proceduralnej.

Sekcja `Custom` służy do animacji definiowanych przez użytkownika. Nazwa
wyjściowa musi mieć od 1 do 16 znaków ASCII: litery, cyfry lub `_`.

### 4. Build, Review i Download

Build działa tylko dla kompletnej, zwalidowanej rewizji. W Review porównaj:

- Source, Mapping i Binary readback;
- komplet 42 stanów;
- events, root motion i deformację;
- project ID/revision, fingerprint mappingu i SHA artefaktów.

Przejdź do osobnego kroku Download i pobierz `.mod`, `.hak`, `.mdl` oraz
raporty JSON. Są to artefakty dokładnie tej rewizji — nie mieszaj plików
z różnych buildów.

## Animation Studio

Animation Studio jest trybem authoringu w projekcie Creature, a nie osobnym
krokiem całego workflow.

### Utworzenie animacji

1. Otwórz `Animation Studio`.
2. Dodaj klip pusty, proceduralny albo skopiowany z modelu.
3. Wybierz kość i edytuj klatki na timeline.
4. Ustaw długość, transition, animation root oraz eventy.
5. Użyj Trim/Retime lub normalizacji quaternionów, jeśli są potrzebne.
6. Uruchom Validate. Tylko klip `VALID` może wejść do końcowego mappingu.

### Kopiowanie animacji z innego modelu

1. Wybierz `Import from model`.
2. Wskaż lokalny GLB dawcy — nie podajesz ścieżki do instalacji NWN ani nazwy
   supermodelu.
3. Aplikacja porówna dokładny output rig: ID i nazwy kości, hierarchy oraz rest
   translation/rotation.
4. Jeśli rig jest zgodny, wybierz klip i nazwę wyjściową.
5. Zaimportowany klip pojawi się w bibliotece `Custom` z SHA dawcy,
   nazwą/fingerprintem klipu i może zostać wybrany w Animation Mapping.

Import dowolnego rigu z automatycznym retargetem nie należy do MVP. Niezgodny
rig jest blokowany przed kopiowaniem, więc aplikacja nie pobiera ciężkich
supermodeli i nie próbuje zgadywać mapy kości.

### Powrót do mappingu

Po zapisaniu klipu wybierz go jako `AUTHORED_CLIP` w definicji Custom albo
przypisaniu stanu. Review pokaże porównanie:

`Source → Edited → Binary readback`.

Utworzenie klipu i jego materializacja w pamięci nie są jeszcze proofem w
Aurorze/NWN. Nowy `.mod`/`.hak` wolno zamrozić wyłącznie w dopuszczonej iteracji.

## Placeable

### 1. Source i Inspect

- wybierz GLB oraz bazowy `placeables.2da`;
- sprawdź wspólny limit 20 000 trójkątów;
- w authoringu oznacz elementy renderowane, collision, shadow i walkmesh;
- skontroluj bounds i preview.

### 2. Build, Review i Download

Build zapisuje project ID/revision, SHA źródła, fingerprint authoringu,
placement, Appearance row oraz SHA zasobów MDL/PWK/UTP/2DA i paczek.

W Review wymagaj wszystkich statusów offline `passed`. Status proof pozostaje:

```text
modelVisibility=not_tested
proofCompleteness=missing
```

do chwili przekazania wyniku przez właściciela. Następnie przejdź do Download
i pobierz pliki dokładnie tej rewizji.

## Test właścicielski Aurora/NWN

Agent przygotowuje exact handoff i hashe, ale nie uruchamia Toolsetu ani NWN.
Handoff musi zaczynać się od:

1. dokładnej nazwy pliku `.mod`;
2. nazwy modułu wyświetlanej w Toolset;
3. dokładnej nazwy Area.

Następnie zawiera HAK, Appearance row, resrefy, obiekt, placement i sposób
wywołania animacji. Status runtime pochodzi wyłącznie z raportu właściciela.

Aktualny handoff E7:
`documentation/evidence/e7-owner-proof-handoff-2026-07-29.md`.
