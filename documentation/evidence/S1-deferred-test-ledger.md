# S1 deferred test ledger

Status: `CODE_WAVE_PASS_REVIEW_CLEAN_REAL_E2E_DEFERRED`

Ten ledger jest evidence PASS dla kodowej fali S1-V1--S1-V5. Nie jest jeszcze
realnym browser E2E na oryginalnym modelu Meshy ani finalnym S1 DONE.

Shared wave 2026-07-14:

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS, 302 testy.
- `wasm-pack test --node crates/m2a-wasm`: PASS, 20 testow, w tym publiczny
  Studio model-package boundary z exact core parity.
- `npm test`: PASS, 4 pliki / 8 testow Studio.
- `npm run build`: PASS; static Vite bundle zawiera osobny Worker i WASM.
- Build sam generuje swiezy `wasm-pack --target web`, a CI wykonuje ten sam flow.
- Niezalezne review po poprawkach: P1=0, P2=0.
- Build raportuje nieblokujace ostrzezenie o glownym chunku JS 822.74 kB;
  code-splitting pozostaje optymalizacja przed publikacja, nie blad outputu.

## S1-V1 shell and local files

- Shell przyjmuje tylko pliki jawnie wybrane przez uzytkownika.
- Sesja rozroznia `EMPTY`, `READY`, `WORKING`, `COMPLETE` i `ERROR`.
- Zmiana inputu usuwa poprzednie artefakty, uniewaznia trwajacy request i nie
  zapisuje danych poza browserem.
- Do MVP nie nalezy backend, upload, odczyt instalacji NWN ani File System
  Access API wymagajace dodatkowych uprawnien.

## S1-V2 canonical WASM Worker

- UI wysyla do Workera transferable `ArrayBuffer`, a nie sciezke lokalna.
- Worker laduje publiczny `m2a-wasm` i nie implementuje formatu MDL/HAK/2DA.
- `buildM6ModelPackageV1` deleguje caly model-only pipeline do `m2a-core`.
- HAK i MDL sa pobierane z WASM tylko raz; JSON pozostaje dokladnym outputem
  adaptera.
- Worker zwraca jawna provenance `M2A_WASM_WORKER`, dlugosc i SHA-256.
- Blad WASM jest propagowany do stanu sesji bez mock outputu.
- TypeScript i Rust przechodza wspolny kontrakt typow podczas test wave.

## S1-V3 source viewport

- Widok ma jawna provenance `SOURCE` oraz ostrzezenie, ze nie jest proofem
  outputu Aurory.
- Renderuje tylko lokalny GLB wybrany przez uzytkownika.
- External resource URLs sa odrzucane; preview nie pobiera zaleznosci sieciowych.
- SHA-256 provenance pochodzi z canonical `ingestGlbJson`, nie z mocka UI.
- Blad loadera viewportu przechodzi do stanu `ERROR`.
- Zmiana pliku niszczy poprzednia scene i zwalnia geometrie, materialy,
  tekstury oraz zasoby skeletonu.

## S1-V4 Aurora/readback and validation

- Output viewport ma jawna provenance `READBACK` i przyjmuje tylko raport
  `inspectBinaryMdl` zwrocony przez canonical Rust/WASM pipeline.
- Geometria, indeksy, hierarchia i bind controllers pochodza z raportu parsera.
- Diagnostyka jest mapowana do noda tylko przez raportowany offset.
- Klikniecie noda filtruje diagnostyke, a wybor diagnostyki podswietla ten sam
  node w readback viewport.
- UI nie czyta naglowkow MDL, offsetow tabel ani bajtow MDX samodzielnie.

## S1-V5 artifact downloads

- Download przyjmuje tylko provenance `M2A_WASM_WORKER`.
- HAK, MDL i JSON sa pobierane z dokladnych `ArrayBuffer` zwroconych przez
  Worker; UI nie buduje ani nie serializuje artefaktow ponownie.
- Filename, extension, byte length i lowercase SHA-256 sa walidowane przed
  utworzeniem obiektu `Blob`; SHA-256 jest ponownie liczony z realnych bajtow.
- Object URL jest zwalniany po zainicjowaniu downloadu.

## Nadal odroczone

- Realny Meshy browser E2E nalezy do pozniejszej bramki wejsc.
- Source/Aurora/readback viewport oraz download UX sa S1-V3--S1-V5.
- Finalny real browser E2E zaczyna sie po dostarczeniu zatwierdzonego inputu.
