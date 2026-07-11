# Prototyp parsera M1A

Data: 2026-07-11 | Owner: Codex orchestrator + subagenci M1A | Status: DONE

Nazwa pliku pozostaje zgodna z historycznym kontraktem M1A. Implementacja zostala wykonana przez orkiestratora Codex i wyspecjalizowanych subagentow, nie przez zewnetrzny runtime ani dependency.

## 1. Wynik

Powstal standalone workspace Rust/WASM z bezpiecznym parserem strukturalnym binary MDL. Publiczne API `inspectBinaryMdl(Uint8Array) -> JSON String` wykonuje parser `m2a-core`, a serializacja pozostaje w `m2a-wasm`.

Parser obsluguje file header, core/MDX boundary, pelny minimalny ModelHeader profilu M1A, root node, relacje parent/children, content flags oraz strukturalna walidacje odroczonych pointerow i tablic. Drzewo jest budowane iteracyjnie.

## 2. Klasyfikacja pol i decyzji

| Pole lub zachowanie | Status | Podstawa i ograniczenie |
|---|---|---|
| `bin_mdl_id == 0` | POTWIERDZONE | Lokalny binary/reference i aktywne kontrakty projektu. |
| `p_start_mdx`, `size_mdx`, core `[12, 12 + p_start_mdx)` | POTWIERDZONE DLA PROFILU A | Lokalny model `c_kocrachn` i crosswalk; nie jest to uniwersalne twierdzenie o kazdej rodzinie modelu. |
| Core pointer `12 + offset` | POTWIERDZONE DLA AKTYWNEGO PROFILU | Kazdy odczyt jest dodatkowo ograniczony do core blocku, nigdy do calego pliku. |
| ModelHeader `0xE8`, name i root pointer | HIPOTEZA Z SILNYM CROSS-CHECKIEM | Lokalna obserwacja binary i niezalezne opisy layoutu; M1B ma potwierdzic na wielomodelowym corpusie P-REF. |
| NodeHeader `0x70`, name, parent, children i content | HIPOTEZA Z SILNYM CROSS-CHECKIEM | Zaimplementowane tylko strukturalnie; runtime loader proof pozostaje pozniejszym gate'em. |
| `contentFlags & 0x001` jako obslugiwany header | POTWIERDZONE W KONTRAKCIE M1A | Nie generuje falszywego `unsupported`; pozostale bity sa jawnie odroczone. |
| `p_geometry`, controller keys/data | POTWIERDZONE STRUKTURALNIE | Pointery i zakresy sa walidowane, payloady nie sa parsowane w M1A. |
| `ParserLimits` | POTWIERDZONE JAKO DECYZJA PRODUKTOWA | Guardraile bezpieczenstwa, nie deklarowane limity engine Aurora. |
| Schemat JSON `schemaVersion: 1`, camelCase | POTWIERDZONE JAKO KONTRAKT API | Zdefiniowane w `m1a-kontrakt-suplement-codex.md`, testowane natywnie i przez Node/WASM. |
| Mesh/skin/controllers/animations payload | NIE WIEM W M1A | Celowo poza zakresem; M1B. |
| Akceptacja wygenerowanego modelu przez engine | NIE WIEM W M1A | Koncowy proof nalezy do M6, nie do parsera inspekcyjnego. |

## 3. Bezpieczenstwo

- Brak `unsafe` w obu crate'ach; workspace lint `unsafe_code = "forbid"` jest dziedziczony przez core i WASM.
- Checked arithmetic dla zakresow i rozmiarow tablic.
- Brak alokacji z niezaufanego count przed walidacja zakresu oraz limitu.
- Iteracyjny traversal eliminuje ryzyko stack overflow od dowolnego caller-supplied `max_depth`.
- Core pointery nie moga wejsc w appended MDX.
- Niepoprawny lub obciety input zwraca stabilny `ParseError`, nie panic.
- Input bytes nie sa modyfikowane.

## 4. Testy i wynik

Zweryfikowane lokalnie na:

- `rustc 1.96.1 (31fca3adb 2026-06-26)`;
- `cargo 1.96.1 (356927216 2026-06-26)`;
- `wasm-pack 0.15.0`;
- `Node v24.15.0`.

| Komenda | Wynik |
|---|---|
| `cargo fmt --all --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS: 20 testow natywnych, 0 failures |
| `cargo build -p m2a-wasm --target wasm32-unknown-unknown` | PASS |
| `wasm-pack test --node crates/m2a-wasm` | PASS: 2 testy publicznego adaptera w Node |
| `git diff --check` | PASS |

Po naprawie findingow P1/P2 wykonano niezalezny re-review aktualnego diffu. Wynik koncowy: brak findings.

Pierwotna kolejnosc argumentow `wasm-pack test crates/m2a-wasm --node` nie jest akceptowana przez lokalny `wasm-pack 0.15.0`. Dokumentacja i CI zostaly poprawione na zweryfikowana kolejnosc `wasm-pack test --node crates/m2a-wasm`.

## 5. Pokryte przypadki

- poprawny minimalny binary MDL i root node;
- deterministyczny JSON;
- pusty i obciety input;
- niepoprawny `bin_mdl_id`;
- MDX range OOB;
- root i child-array OOB;
- core pointer probujacy wejsc w MDX;
- root nachodzacy na ModelHeader;
- cykl/repeated node;
- `used > allocated` oraz nieograniczony allocated count;
- OOB `p_geometry`, controller keys i controller data;
- overflow `offset + size` i `count * element_size`;
- przekroczenie i dokladna granica kazdego `ParserLimits`;
- niemutowalnosc inputu;
- publiczny blad i poprawny raport przez Node/WASM.

## 6. Zmienione pliki

- `.gitattributes`
- `.github/workflows/ci.yml`
- `.gitignore`
- `Cargo.lock`
- `Cargo.toml`
- `rust-toolchain.toml`
- `crates/m2a-core/Cargo.toml`
- `crates/m2a-core/src/lib.rs`
- `crates/m2a-core/src/mdl/mod.rs`
- `crates/m2a-core/src/mdl/binary_reader.rs`
- `crates/m2a-core/src/mdl/errors.rs`
- `crates/m2a-core/src/mdl/types.rs`
- `crates/m2a-core/src/mdl/parse_binary_mdl.rs`
- `crates/m2a-core/tests/mdl.rs`
- `crates/m2a-core/tests/fixtures/build_minimal_binary_mdl.rs`
- `crates/m2a-core/tests/mdl/parser.rs`
- `crates/m2a-wasm/Cargo.toml`
- `crates/m2a-wasm/src/lib.rs`
- `documentation/README.md`
- `documentation/architektura-web-wasm-codex.md`
- `documentation/audyt-gotowosci-startowej-2026-07-10-codex.md`
- `documentation/m1a-kontrakt-suplement-codex.md`
- `documentation/orchestrator-state.yaml`
- `documentation/plan-implementacji-orkiestrator-codex.md`
- `documentation/prompt-dla-claude-prototyp-parsera.md`
- `documentation/evidence/M1A-evidence.md`
- `documentation/prototyp-parsera-m1a-claude.md`

## 7. Znane ograniczenia i kolejne kroki

- M1A nie parsuje mesh, skin, controller payloadow ani animacji.
- Layout model/node wymaga wielomodelowego P-REF i env-gated regresji w M1B.
- Reader HAK/ERF jest osobnym etapem M1C.
- GLB i `AuroraAssetIR` zaczynaja sie dopiero w M2.
- Writer, native MDL/MDX output i engine proof pozostaja M4-M6.
- Nie ma pytania blokujacego zamkniecie M1A; nowe fakty formatu musza byc zamykane dopiero w przypisanym etapie i bez zgadywania.
