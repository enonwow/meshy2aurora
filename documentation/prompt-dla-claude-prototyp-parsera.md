# prompt-dla-claude-prototyp-parsera.md

Data: 2026-07-10 | Autor: Codex | Status: GOTOWE DO PRZEKAZANIA CLAUDE

## Cel promptu

Ten prompt zleca Claude'owi waski prototyp `M1a`: bezpieczny, samodzielny parser struktury binary MDL jako rdzen aplikacji webowej Rust/WASM. Nie zleca writera, konwertera Meshy, HAK/2DA, viewportu ani przyszlych feature'ow Studio.

## Prompt do Claude

```text
Pracujesz nad samodzielnym projektem meshy2aurora. Zbuduj M1a: minimalny, bezpieczny prototyp parsera struktury binary MDL Aurora/NWN.

KANONICZNY WORKSPACE
- Pracuj tylko w C:\Projects\meshy2aurora.
- Nie uzywaj C:\Users\enonw\Documents\meshy2aurora; to pusty, niekanoniczny klon.
- Cala dokumentacja projektu nalezy do C:\Projects\meshy2aurora\documentation.

NAJPIERW PRZECZYTAJ
1. C:\Projects\meshy2aurora\documentation\PROJECT_RULES.md
2. C:\Projects\meshy2aurora\documentation\architektura-meshy2aurora-codex.md
3. C:\Projects\meshy2aurora\documentation\architektura-web-wasm-codex.md
4. C:\Projects\meshy2aurora\documentation\audyt-gotowosci-startowej-2026-07-10-codex.md
5. C:\Projects\meshy2aurora\documentation\standalone-odpowiedz-codex.md, szczegolnie Q2 i m1_parser_scope
6. C:\Projects\meshy2aurora\documentation\aurora-mdl-format-codex.md
7. C:\Projects\meshy2aurora\documentation\engine-mdl-pytania-cloud.md
8. C:\Projects\meshy2aurora\documentation\engine-mdl-odpowiedz-codex.md
9. C:\Projects\meshy2aurora\documentation\decyzje-i-zadania-cloud.md
10. C:\Projects\New Folder jako Aurora First, gdy dokumentacja nie daje potwierdzonego faktu.

ZASADY NIEPODLEGAJACE DYSKUSJI
- Aurora First. Bez zgadywania offsetow, flag albo semantyki pol.
- C:\Projects\aurora-web jest tylko reference-only. Nie wolno go importowac, wywolywac przez subprocess, traktowac jako oracle, walidator, fixture source ani dependency.
- Nie kopiuj kodu z zewnetrznych repozytoriow. C:\Projects\Claude\xoreos-docs, borealis_nwn_mdl, rollnw i Radoub wolno czytac tylko jako cross-check po Aurorze First.
- Nie commituj ani nie wypakowuj do repo assetow retail/CEP, c_kocrachn ani pochodnych ASCII MDL.
- TDD: najpierw test lub gate, potem minimalny kod, potem refactor.
- Gdy layout nie jest potwierdzony: zwroc structured unsupported/error, opisz blocker w dokumentacji. Nie wymyslaj formatu.
- Produkt jest aplikacja webowa local-first. Parser nie moze wymagac lokalnej sciezki, procesu CLI ani serwera; wejscie dostaje jako bytes wybranego przez uzytkownika pliku.

CEL M1a
Napisz prototyp, ktory przyjmuje bytes binary .mdl i zwraca deterministyczny raport JSON o jego strukturze. Rdzen ma dzialac natywnie w testach Cargo i byc wystawiony do JavaScript przez WASM. To ma byc parser do inspekcji referencji, nie writer i nie runtime gry.

M1a MUSI OBSLUZYC
1. File header:
   - little-endian binary reader;
   - walidacja bin_mdl_id == 0 dla binary MDL;
   - p_start_mdx i size_mdx, wraz z bounds check.
2. Model header i root geometry pointer w zakresie potwierdzonym przez dokumentacje.
3. Node tree:
   - node name, parent/children relationship, content flags;
   - bezpieczne przejscie po child arrays;
   - bounds checks dla kazdego pointera i rozmiaru;
   - wykrywanie cykli i limit bezpiecznej glebokosci.
4. Deterministyczny raport inspekcji z diagnostyka.
5. WASM API:
   - przyjmuje `Uint8Array`/bytes pliku wybranego przez uzytkownika;
   - zwraca JSON raportu albo stabilny JSON bledu;
   - nie czyta DOM, filesystemu ani sieci.

M1a NIE OBEJMUJE
- binary MDL writer;
- ASCII emitter poza ewentualnym minimalnym debug stringiem;
- MDX policy/emisji osobnego zasobu;
- parsowania mesh vertices, faces, skin weights, controllers i animacji, jezeli nie sa konieczne do bezpiecznego przejscia node tree;
- HAK/ERF reader/writer;
- 2DA;
- GLB/Meshy;
- GUI React/Three.js, viewportu i przyszlych feature'ow Studio;
- automatyzacji Toolset/NWN.

ARCHITEKTURA KODU
- Uzyj Rust 1.96.1 i Cargo. Projekt ma byc workspace z rdzeniem niezaleznym od DOM, przystosowanym do WebAssembly; bez zaleznosci od Node.js, .NET ani aurora-web w rdzeniu Rust.
- Utworz w root `Cargo.toml` jako workspace, `rust-toolchain.toml` z `channel = "1.96.1"`, komponentami `rustfmt` i `clippy` oraz targetem `wasm32-unknown-unknown`, a takze co najmniej dwa crate'y:
  crates/m2a-core/src/mdl/binary_reader.rs
  crates/m2a-core/src/mdl/errors.rs
  crates/m2a-core/src/mdl/types.rs
  crates/m2a-core/src/mdl/parse_binary_mdl.rs
  crates/m2a-core/src/lib.rs
  crates/m2a-wasm/src/lib.rs
  crates/m2a-core/tests/mdl/*.rs
  crates/m2a-core/tests/fixtures/build_minimal_binary_mdl.rs
- Nie dodawaj ciezkich frameworkow ani zaleznosci runtime bez jasnego uzasadnienia. W M1a preferuj standardowa biblioteke w `m2a-core`; `m2a-wasm` moze uzyc `wasm-bindgen` wylacznie jako granicy JavaScript/WASM.
- `BinaryReader` ma byc jedynym miejscem odczytu little-endian primitive values, strings i sprawdzania zakresow. Nie wolno uzywac niekontrolowanych rzutowan, `unsafe` ani indeksowania bufora bez sprawdzenia granic.
- Kazde dodawanie/mnozenie offsetu, count i element size ma uzywac checked arithmetic przed alokacja albo slice. Nieprawidlowy input ma zwracac diagnostyke, nie panic.
- Dodaj jawny `ParserLimits` z nazwanymi guardrails dla inputu, node count, depth i diagnostics. Sa to limity bezpieczenstwa produktu, nie rzekome limity silnika Aurora.
- Parser ma zwracac struktury danych, nie wypisywac ich sam. Adapter WASM tylko serializuje wynik do deterministycznego JSON.
- Kod produktu nie moze czytac globalnych pathow NWN, uruchamiac procesu CLI ani wysylac inputu po sieci.

KONTRAKT RAPORTU JSON
Utworz typowany raport z co najmniej:
{
  "format": "nwn1-binary-mdl",
  "byteLength": 0,
  "fileHeader": {
    "binaryMdlId": 0,
    "mdxStart": 0,
    "mdxSize": 0,
    "mdxRangeInBounds": true
  },
  "model": {
    "name": "...",
    "rootNodeOffset": 0
  },
  "nodeTree": {
    "nodeCount": 0,
    "maxDepth": 0,
    "roots": []
  },
  "unsupported": [],
  "diagnostics": []
}

Nie wpisuj wartosci modelu, offsetow ani nazw na sztywno. Raport musi byc stabilny dla tego samego inputu.

TDD - WYMAGANE TESTY SYNTETYCZNE
Stworz fixture builder, ktory programowo buduje minimalny binary MDL w testach. Nie dodawaj do repo cudzych modeli ani binarnych payloadow gry.

Testy musza pokrywac co najmniej:
1. poprawny minimalny file header i pojedynczy root node;
2. odrzucenie bin_mdl_id innego niz 0;
3. odrzucenie pointera lub tablicy poza zakresem pliku;
4. odrzucenie cyklu node tree albo bezpieczne zgloszenie M2A-MDL-NODE-CYCLE;
5. deterministyczny JSON raportu dla tej samej fixture;
6. adapter WASM zwraca stabilny blad dla pustego inputu.
7. arithmetic overflow i count*element_size overflow sa odrzucane stabilnym kodem;
8. obciety input nie powoduje panic;
9. przekroczenie `ParserLimits` zwraca `M2A-LIMIT-EXCEEDED`;
10. `wasm-bindgen-test` uruchamiany przez `wasm-pack test crates/m2a-wasm --node` rzeczywiscie wywoluje publiczne API adaptera.

Uzyj czytelnych, stabilnych kodow diagnostycznych, np.:
- M2A-MDL-HEADER-INVALID
- M2A-MDL-POINTER-OOB
- M2A-MDL-NODE-CYCLE
- M2A-MDL-UNSUPPORTED

KRYTERIA AKCEPTACJI
- `cargo fmt --all --check` przechodzi;
- `cargo clippy --workspace --all-targets -- -D warnings` przechodzi;
- `cargo test --workspace` przechodzi;
- `cargo build -p m2a-wasm --target wasm32-unknown-unknown` przechodzi;
- `wasm-pack test crates/m2a-wasm --node` przechodzi i wywoluje publiczny adapter WASM;
- publiczny adapter WASM dla syntetycznej fixture zwraca poprawny JSON albo czytelny kod bledu;
- parser nie wykonuje zapisu do inputu;
- brak importow, CLI/subprocess, dostepu do DOM/filesystemu/sieci i fixture payloadow z aurora-web;
- brak retail/CEP payloadow w git;
- git diff --check przechodzi.

RAPORT KONCOWY I DOKUMENTACJA
Po implementacji utworz C:\Projects\meshy2aurora\documentation\prototyp-parsera-m1a-claude.md. Zawrzec:
- status POTWIERDZONE/HIPOTEZA/NIE WIEM dla kazdego zaimplementowanego pola;
- sciezki zmienionych plikow;
- wersje `rustc` i `cargo`, komendy weryfikacyjne i ich wynik;
- znane ograniczenia M1a;
- kolejne, osobne kroki: M1b mesh/skin/controllers/animations oraz HAK reader;
- wszystkie nowe pytania do Codexa/Cloud, jesli bez potwierdzenia nie wolno kontynuowac.

Dodatkowo utworz lub dopisz `C:\Projects\meshy2aurora\documentation\evidence\M1A-evidence.md` wedlug szablonu `documentation/evidence/README.md`. Zapisz osobno wyniki native testow, WASM build i `wasm-pack` Node smoke test.

Nie rob commit ani push. Na koniec podaj zwiezle: co dziala, czego parser celowo jeszcze nie obsluguje, oraz liste zmienionych plikow.
```

## Oczekiwany rezultat

Claude ma dostarczyc tylko prototype `M1a`. Nastepne kroki `M1b` i HAK reader sa osobnymi zadaniami, nie dopuszczalnym scope creepem tego zlecenia.
