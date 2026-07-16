# S1 deferred test ledger

## Studio V1 refactor note - 2026-07-15

- Aktualne `npm run test:worker-integration`: 4/4 PASS.
- Prawdziwy native-owned Worker -> web-WASM model build, produkcyjny App oraz oba testy M7 przechodza.
- Historyczny test zostal zmigrowany do flow Source -> Inspect -> Build -> Review i ponownie uruchomiony z prawdziwym Workerem/publicznym web-WASM.
- Kontrakt readbacku jest jawny: Rust emituje `nwn1-binary-mdl`, a projektor Studio go waliduje jako jedyny akceptowany format.
- S1-V5: wynik `MODEL_PACKAGE_BUILT` renderuje teraz canonical Worker artifact downloads w glownym ekranie Review, a nie tylko w panelu M7. App i realny Worker/web-WASM integration test potwierdzaja widocznosc kontroli pobierania.

Status: `CODE_WAVE_PASS_REVIEW_CLEAN_REAL_E2E_DEFERRED`

Ten ledger jest evidence PASS dla kodowej fali S1-V1--S1-V5. Nie jest jeszcze
realnym browser E2E na oryginalnym modelu Meshy ani finalnym S1 DONE.

Shared wave 2026-07-14:

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS, 319 testow.
- `wasm-pack test --node crates/m2a-wasm`: PASS, 20 testow, w tym publiczny
  Studio model-package boundary z exact core parity.
- `npm test`: PASS, 23 pliki / 141 testow Studio.
- `npm run test:worker-integration`: PASS, 4 realne testy Chrome, w tym
  produkcyjny App -> Worker -> web-WASM -> widoczny canonical result.
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
- Realna bramka integracyjna tworzy lokalne obiekty `File` z repo-owned
  synthetic fixture, wywoluje produkcyjny `StudioWorkerClient`, realny module
  Worker i web-WASM, a nastepnie porownuje HAK byte-for-byte z canonical native
  outputem oraz sprawdza report, manifest, readback i transferable buffers.
- Test nie korzysta z fake Workera, backendu, zewnetrznej sieci, Aurory/NWN ani
  konfiguracji uzytkownika; synthetic fixture nie jest Meshy acceptance evidence.
- Completion flow `M7-V4S` dodaje typed validate/intake/batch przez ten sam
  realny Worker i publiczny M7 WASM. Manifest jest canonical validation gate,
  payloady tworza jeden transferable blob, a exact JSON artifacts maja
  zweryfikowane bajty, dlugosc, SHA-256 i provenance.
- M7 revision guards uniewazniaja stale odczyty/odpowiedzi i czyszcza artefakty
  po zmianie inputu; bledy panelu M7 nie zmieniaja statusu sesji M6.

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

## S1-V5R canonical result workspace

- Produkcyjny App po realnym `MODEL_PACKAGE_BUILT` pokazuje strict snapshot
  exact `reportJson`, `summaryJson`, `manifestJson`, `readbackJson` i artifacts.
- Widoczne sa realne KPI geometrii, animation mapping/duration/motion,
  tekstura, resrefy, appended 2DA row, HAK size/resource inventory i byte hashes.
- Projectory wymagaja schemaVersion 1, exact materialized status i reconciled
  identities miedzy reportem, summary, manifestem, readbackiem i artefaktami.
- Snapshot/readback/artifacts sa commitowane atomowo; malformed lub identity
  mismatch daje ERROR bez czesciowego wyniku i bez crash renderu.
- Source inspection oraz build maja osobne revision guards; late response nie
  nadpisuje WORKING/COMPLETE, a zmiana inputu usuwa wynik i downloady.
- Karta wyniku jest przed viewportami i panelem M7 oraz jawnie pokazuje
  `Canonical structural output` i `Runtime acceptance: OPEN_M6`.
- Real Chrome test montuje produkcyjny App, wybiera lokalne pliki, uruchamia
  realny module Worker/public WASM i sprawdza exact widoczne KPI, kolejnosc,
  brak runtime PASS oraz reset po zmianie source.
- Finalne niezalezne review: P1=0, P2=0.

## Nadal odroczone

- Realny browser Worker/WASM E2E na repo-owned synthetic fixture jest PASS.
- Realny Meshy browser E2E nalezy do pozniejszej bramki wejsc.
- Source/Aurora/readback viewport oraz download UX sa S1-V3--S1-V5.
- Finalny real browser E2E zaczyna sie po dostarczeniu zatwierdzonego inputu.
