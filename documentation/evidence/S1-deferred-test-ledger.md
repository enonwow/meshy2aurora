# S1 deferred test ledger

Status: `THRESHOLD_REACHED_TEST_WAVE_PENDING`

Ten ledger nie jest evidence PASS ani S1 DONE. Rejestruje testy dla wspolnej
fazy po pierwszej implementacji vertical slices.

## S1-V1 shell and local files

- Shell przyjmuje tylko pliki jawnie wybrane przez uzytkownika.
- Sesja rozroznia `EMPTY`, `READY`, `WORKING`, `COMPLETE` i `ERROR`.
- Zmiana inputu usuwa poprzednie artefakty i nie zapisuje danych poza browserem.
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

## Nadal odroczone

- Realny Meshy browser E2E nalezy do pozniejszej bramki wejsc.
- Source/Aurora/readback viewport oraz download UX sa S1-V3--S1-V5.
- Full browser build, Worker runtime test i independent review zaczynaja sie po
  integracji pierwszej implementacji zgodnie z suplementem.
