# Placeable V9 — checkpoint wstrzymanej implementacji

Data: 2026-08-12\
Status: `PAUSED_BY_OWNER / WORKTREE_PRESERVED / DO_NOT_CLAIM_COMPLETE`

## Wznowienie 2026-08-13

Checkpoint został wznowiony i zamknięty pełną walidacją offline. Aktualny
raport implementacyjny znajduje się w
[`placeable-v9-material-pipeline-implementation-2026-08-13.md`](placeable-v9-material-pipeline-implementation-2026-08-13.md).
Ten dokument pozostaje historycznym zapisem stanu w chwili zatrzymania.

## Powód checkpointu

Właściciel polecił zatrzymać zadanie i wrócić do implementacji następnego dnia.
Bieżący proces pełnych regresji został przerwany. Nie uruchamiano ani nie
sterowano Aurora Toolset/NWN, nie zamrażano nowego kandydata i nie tworzono
nowej iteracji modelu.

## Stan wykonania sześciu etapów

1. **Kontrakt i kontrola jakości — zaimplementowane.** V9 kompiluje materiały
   dla `AURORA_CLASSIC_SAFE` albo `NWN_EE_MTR`; source-quality gate ocenia UV,
   gęstość texeli, fragmentację oraz czytelność mipów i blokuje wynik
   niespełniający progu.
2. **Integracja kompilatora materiałów z Placeable — zaimplementowana.** Nowa
   publiczna trasa `build_meshy_static_placeable_package_v9` korzysta z
   Material Separation V2, opcjonalnego UV projection i model texture
   authoring.
3. **MDL/TGA/MTR/TXI — zaimplementowane.** Produkcyjny bridge tworzy diffuse,
   normal/specular bake, TXI i MTR, uzupełnia tangenty oraz zapisuje materiał
   w rozszerzonym binary MDL z semantycznym readbackiem.
4. **HAK/WASM/Worker — zaimplementowane.** Zasoby materiałowe są pakowane,
   odczytywane i porównywane po HAK readbacku; WASM i Worker eksportują ich
   dokładne bajty jako TGA/DDS/MTR/TXI.
5. **Studio — zaimplementowane w kodzie.** Placeable ma wybór profilu materiału.
   `Aurora Export` używa finalnego readbacku MDL, zweryfikowanych SHA-256 TGA i
   MTR, w tym `twoSided`, transparency oraz punch-through. Downloader akceptuje
   TGA/DDS/MTR/TXI.
6. **Regresje i handoff — częściowo ukończone.** Test pionowy
   `Studio -> Worker -> WASM -> Core V9 -> HAK/MOD` przeszedł. Pełny przebieg
   wszystkich testów Core/Studio został uruchomiony, ale świadomie przerwany na
   polecenie właściciela, więc nie wolno raportować pełnego PASS ani ukończenia.

## Potwierdzone wyniki przed zatrzymaniem

- `cargo test -p m2a-core --test placeable_pipeline placeable_v9 -- --nocapture`
  — PASS, 3/3:
  - pozytywny EE MTR z MTR/TXI i semantic readback;
  - klasyczny profil fail-closed dla `doubleSided`;
  - quality gate fail-closed dla nieczytelnej gęstości tekstury.
- `npm run typecheck` — PASS.
- testy jednostkowe Studio dla `AuroraExportViewport` i
  `projectPlaceableResult` — PASS, 9/9.
- test przeglądarkowy `materializes Placeable V9 materials through the real
  Worker and WASM boundary` — PASS, 1/1.
- `npm run build:wasm` — PASS przed testem przeglądarkowym.

## Pierwsze kroki po wznowieniu

1. Uruchomić guard kanonicznego workspace.
2. Sprawdzić `git diff --check` i `cargo fmt --all -- --check`.
3. Dokończyć przerwany pełny zestaw:
   - `cargo test -p m2a-core --no-fail-fast`;
   - `cargo test -p m2a-wasm --no-fail-fast`;
   - `cargo check -p m2a-core -p m2a-wasm`;
   - `npm test` oraz `npm run typecheck` w `apps/studio-web`;
   - pełny `npm run test:worker-integration`, jeśli budżet czasu na to pozwala.
4. Naprawić wyłącznie wykryte regresje, bez zmiany zamrożonych artefaktów
   statku i bez osłabiania quality gate.
5. Po pełnym PASS uzupełnić dokumentację wyniku i dopiero wtedy ocenić, czy
   etap implementacyjny jest zakończony. Owner proof pozostaje osobnym etapem.

## Ważne granice

- Worktree ma wiele wcześniejszych zmian właściciela; nie resetować ani nie
  usuwać plików niezwiązanych z V9.
- Nie stage'owano i nie commitowano zmian w tym checkpointcie.
- Nie utworzono MOD/HAK przeznaczonego do nowego owner proof.
- Końcowy wizualny proof w Aurora/NWN pozostaje własnością człowieka.
