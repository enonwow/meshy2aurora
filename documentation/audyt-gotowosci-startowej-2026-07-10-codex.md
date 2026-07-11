# audyt-gotowosci-startowej-2026-07-10-codex.md

Data: 2026-07-10 | Aktualizacja: 2026-07-11 | Autor: Codex | Status: AKTYWNY GATE PRZED IMPLEMENTACJA

## 1. Werdykt

Projekt jest dobrze przygotowany koncepcyjnie, ale nie jest jeszcze gotowy do uruchomienia implementacji bez warunkow.

```yaml
readiness:
  product_contract: "READY"
  architecture: "READY"
  M1A_scope: "READY"
  canonical_repository: "READY"
  documentation_inventory: "READY after fixes recorded in this audit"
  full_pipeline_knowledge_direction: "READY; runtime evidence remains staged"
  local_toolchain: "READY: Rust/Cargo 1.96.1, wasm32 target, rustfmt, clippy and wasm-pack 0.15.0 verified 2026-07-11"
  repository_scaffold: "NOT_STARTED"
  native_runtime_proof_environment: "READY for later manual proof"
  M1A_start: "READY_AFTER_P0_ACCEPTANCE"
  M4_writer: "DIRECTION READY; implementation/readback/runtime evidence pending"
  M4A_animations: "DIRECTION READY; state/event/runtime evidence pending"
  M5_2DA_HAK: "DIRECTION READY; generated module runtime evidence pending"
```

Initial documentation baseline i kolejne knowledge updates zostaly zapisane oraz wypchniete na `codex/docs-readiness-baseline`. Stan `main` nie zawiera jeszcze tej serii dokumentacji; przed M1A wlasciciel wybiera merge do `main` albo jawnie zatwierdza kontynuacje implementacji na obecnej galezi. Nie zmieniamy `main` bez osobnego polecenia.

M1A moze zaczac po jednym pozostalym gate'ie:

1. jawnej zgodzie wlasciciela `start M1A` po przegladzie aktualnej dokumentacji;

Lokalna weryfikacja z 2026-07-11 potwierdza: `rustc`/`cargo` 1.96.1, `rustfmt`, `clippy`, target `wasm32-unknown-unknown`, `wasm-pack` 0.15.0, Node 24.15.0 oraz npm 11.12.0. Instalacja Rust/WASM nie jest juz gate'em startowym.

Minimalny Cargo workspace z sekcji 6 jest pierwszym krokiem M1A po zgodzie, a nie czynnoscia wykonywana teraz w ramach dokumentacji.

Kierunek pelnego writera, 2DA/HAK/GFF i animacji jest zapisany w macierzy wiedzy i kontraktach przekrojowych. Otwarte proofy nie blokuja strukturalnego parsera M1A, ale nie wolno promowac ich do faktow runtime przed przejsciem nazwanych testow.

## 2. Kanoniczne repozytorium i granice

```yaml
canonical:
  repository: "C:\\Projects\\meshy2aurora"
  default_branch: "main"
  current_documentation_branch: "codex/docs-readiness-baseline"
  current_documentation_branch_merge_to_main: "PENDING OWNER DECISION"
  remote: "https://github.com/enonwow/meshy2aurora.git"
  documentation: "C:\\Projects\\meshy2aurora\\documentation"
non_canonical:
  path: "C:\\Users\\enonw\\Documents\\meshy2aurora"
  observed_state: "empty Git repository without commits"
  rule: "do not implement or duplicate documentation here"
reference_only:
  aurora_decompilation: "C:\\Projects\\New Folder"
  aurora_web: "C:\\Projects\\aurora-web"
```

`C:\Projects\New Folder` jest glownym zrodlem Aurora First. `C:\Projects\aurora-web` pozostaje osobnym projektem i nie moze wejsc do dependency, runtime, testow, fixture ani walidacji `meshy2aurora`.

## 3. Stan repo przed tym audytem

```yaml
repository_snapshot:
  commits: 1
  tracked_branch: "main...origin/main"
  implementation_files: 0
  documentation_files: 62
  modified_documentation_files: 11
  untracked_documentation_entries: 8
  sample_2d_payloads: 0
  sample_3d_payloads: 0
  root_gitignore: false
  root_gitattributes: false
  root_license: false
```

Nie wolno rozpoczynac szerokiej implementacji na niezapisanym baseline dokumentacji. Na osobne polecenie wlasciciela zestaw zmian z tego audytu zostal przeznaczony do jednego, intencjonalnego commita i pushu przed implementacja. Publikacja dokumentacji nie jest jeszcze zgoda na start M1A.

## 4. Zweryfikowane srodowisko lokalne

Stan sprawdzony 2026-07-10:

```yaml
installed:
  git: "present"
  node: "24.15.0"
  npm: "11.12.0"
  python: "3.12"
  visual_studio: "Community 18"
  msvc_tools: "14.50.35717"
  windows_sdk: "10.0.26100.0"
  nwn_ee_toolset: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwtoolset.exe"
  nwn_ee_game: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwmain.exe"
  nwn_ee_server: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32\\nwserver.exe"
  nwn_user_hak: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak"
  nwn_user_modules: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\modules"
missing:
  - "rustup"
  - "rustc"
  - "cargo"
  - "wasm-pack"
  - "wasm-bindgen CLI"
```

Rust `1.96.1` jest realnym, dostepnym toolchainem: oficjalny manifest dystrybucji odpowiada HTTP 200 i ma date 2026-06-30. Target `wasm32-unknown-unknown` ma dostepny komponent `rust-std`.

## 5. Bootstrap srodowiska

Instalacja jest osobna czynnoscia operacyjna; nie zostala wykonana w ramach audytu dokumentacji.

### 5.1 Instalacja

1. Pobrac i uruchomic `rustup-init.exe` z oficjalnej strony Rust.
2. Otworzyc nowa sesje PowerShell.
3. Wykonac w kanonicznym repo:

```powershell
rustup toolchain install 1.96.1 --profile minimal --component rustfmt --component clippy --target wasm32-unknown-unknown
rustup override set 1.96.1
cargo install wasm-pack --version 0.15.0 --locked
```

`wasm-pack 0.15.0` jest przypieta wersja narzedzia deweloperskiego dla M1A. Zmiana wersji wymaga jawnej aktualizacji tego dokumentu, CI i evidence; nie jest to dependency runtime produktu.

### 5.2 Weryfikacja po instalacji

```powershell
rustup show
rustc --version --verbose
cargo --version
rustup target list --installed
rustup component list --installed
wasm-pack --version
node --version
npm --version
```

Oczekiwane minimum:

```yaml
expected:
  rustc: "1.96.1"
  cargo: "toolchain 1.96.1"
  components: ["rustfmt", "clippy"]
  targets: ["x86_64-pc-windows-msvc", "wasm32-unknown-unknown"]
  wasm_pack: "0.15.0"
  node: ">=24.0.0 for the M1A Node smoke test"
```

MSVC i Windows SDK sa zainstalowane. `cl.exe` oraz `link.exe` nie sa na zwyklym `PATH`, co jest normalne poza Developer PowerShell. Jesli Cargo nie odnajdzie linkera, nalezy uruchomic `VsDevCmd.bat` z `C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools`, zapisac dokladny blad w evidence i nie zmieniac ABI na GNU bez osobnej decyzji.

## 6. Minimalny scaffold repozytorium

Pierwsza implementacja ma utworzyc tylko fundament potrzebny M1A:

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
.gitignore
.gitattributes
.github/workflows/ci.yml
crates/
  m2a-core/
    Cargo.toml
    src/
    tests/
  m2a-wasm/
    Cargo.toml
    src/
    tests/
documentation/
  evidence/
```

Reguly scaffoldingu:

- `Cargo.lock` jest commitowany, bo repo buduje aplikacje, a nie publikuje samodzielna biblioteke ogolnego przeznaczenia.
- `rust-toolchain.toml` przypina `1.96.1`, `rustfmt`, `clippy` i `wasm32-unknown-unknown`.
- `.gitattributes` wymusza tekstowe LF dla kodu i oznacza generowane binaria jako binary.
- `.gitignore` wyklucza `target/`, `pkg/`, `node_modules/`, raporty tymczasowe i lokalne proof payloads, ale nie wyklucza dokumentacji evidence.
- Nie dodajemy jeszcze React/Vite `package.json`; web UI zaczyna sie zgodnie z etapem S1. M1A potrzebuje Node tylko do testu granicy WASM.
- Nie tworzymy lokalnej kopii assetow retail/CEP.
- Decyzja o licencji repo musi zostac zamknieta przed publicznym wydaniem/GitHub Pages, ale nie blokuje lokalnego M1A z wlasnym kodem.

## 7. M1A: kontrakt startowy

M1A jest parserem strukturalnym i testem granicy Rust/WASM. Nie jest jeszcze parserem calego modelu ani writerem.

### 7.1 Dozwolony zakres

- checked little-endian reader;
- file header i zakres appended MDX;
- potwierdzony zakres model header/root pointer;
- node tree, parent/children, content flags;
- cycle detection i limit glebokosci;
- deterministyczny, wersjonowany JSON report;
- stabilne kody diagnostyczne;
- adapter `Uint8Array -> JSON` bez DOM, filesystemu i sieci;
- syntetyczny fixture builder w testach.

### 7.2 Obowiazkowe zabezpieczenia

```yaml
parser_safety:
  unsafe_rust: "forbidden in M1A"
  arithmetic: "checked_add/checked_mul before every range calculation"
  allocation: "never allocate from an untrusted count before validating count, element size and remaining bytes"
  traversal: "visited-offset set plus explicit depth/node budget"
  recursion: "prefer iterative traversal; recursive code must prove bounded depth"
  input_mutation: false
  panic_policy: "invalid input returns a diagnostic; it must not panic"
  diagnostics:
    required_fields: ["schema_version", "code", "severity", "offset", "context"]
  parser_limits:
    rule: "named defaults and boundary tests are required"
    classification: "product safety guardrails, not claimed Aurora engine limits"
```

Nie wpisujemy arbitralnych wartosci limitow jako faktow Aurory. Implementacja ma jednak miec jawny `ParserLimits`, test przekroczenia kazdego limitu i kod `M2A-LIMIT-EXCEEDED`.

### 7.3 Definition of Done

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p m2a-wasm --target wasm32-unknown-unknown
wasm-pack test crates/m2a-wasm --node
git diff --check
```

Wymagane dowody:

- test prawidlowej syntetycznej fixture;
- test pustego inputu;
- invalid header;
- pointer/array OOB;
- arithmetic overflow;
- node cycle;
- limit exceeded;
- brak paniki dla obcietego inputu;
- dwa uruchomienia tej samej fixture daja bajtowo identyczny JSON;
- test `wasm-bindgen-test` uruchamia publiczne API w Node, a nie tylko kompiluje `.wasm`;
- `documentation/prototyp-parsera-m1a-claude.md` i `documentation/evidence/M1A-evidence.md` zawieraja komendy, actual result, wersje i changed files.

Samo `cargo build --target wasm32-unknown-unknown` nie dowodzi dzialania granicy JavaScript/WASM.

## 8. CI wymagane od pierwszego kodu

Minimalny workflow CI powinien miec:

```yaml
ci:
  native_quality:
    matrix: ["windows-latest", "ubuntu-latest"]
    commands:
      - "cargo fmt --all --check"
      - "cargo clippy --workspace --all-targets -- -D warnings"
      - "cargo test --workspace"
  wasm_boundary:
    runner: "ubuntu-latest"
    commands:
      - "cargo build -p m2a-wasm --target wasm32-unknown-unknown"
      - "wasm-pack test crates/m2a-wasm --node"
  hygiene:
    commands:
      - "git diff --check"
```

CI nie uzywa assetow gry, sekretow ani lokalnych sciezek. Env-gated reference tests sa pomijane w CI, dopoki nie powstanie osobna, legalna polityka prywatnych test assets.

## 9. Aurora First: stan dowodow przed M1A

```yaml
evidence_classification:
  file_header:
    status: "retail/reference binary confirmed"
    evidence: "c_kocrachn: bin_mdl_id=0, p_start_mdx=76048, size_mdx=87132"
  node_and_model_layout:
    status: "secondary layout reference plus local binary observations; not fully loader-confirmed"
    action: "implement only fields needed by M1A and preserve unsupported diagnostics"
  writer_contract:
    status: "DIRECTION_DEFINED_EVIDENCE_OPEN GB-001"
    contract: "documentation/mdl-binary-crosswalk-codex.md"
  mdx_initial_profile:
    status: "CONFIRMED for c_kocrachn/cep3_core1 reference"
    evidence:
      - "cep3_core1.hak: 3517 MDL entries (type 2002), zero MDX entries (type 2003)"
      - "c_kocrachn has one type-2002 entry and no type-2003 entry"
      - "12 + 76048 + 87132 = 163192, exactly the MDL resource size"
    implementation_decision: "profile A emits one MDL resource 2002 with appended MDX block; no separate 2003 resource"
    final_gate: "own readback plus NWN EE Toolset/game proof"
  two_da_edit_contract:
    status: "DIRECTION_DEFINED_RUNTIME_OPEN GB-004"
    contract: "documentation/hak-2da-gff-crosswalk-codex.md"
  animation_contract:
    status: "DIRECTION_DEFINED_RUNTIME_OPEN GB-005"
    contract: "documentation/animacje-kontrakt-profil-a-codex.md"
```

Wynik dla MDX nie jest uniwersalnym twierdzeniem o kazdym modelu NWN. Jest wystarczajacym, lokalnym dowodem dla pierwszego profilu `direct creature`. Inny profil moze zmienic polityke dopiero po osobnym Aurora First evidence.

## 10. Proof i test assets

Srodowisko do pozniejszego manualnego proofu jest dostepne: NWN EE Toolset, gra, `hak/` i `modules/` istnieja lokalnie.

Obowiazuje jednak rozdzial:

```yaml
unit_tests: "synthetic fixtures generated in test code"
integration_reference: "optional read-only local assets via env; never committed"
proof_assets: "generated by meshy2aurora: HAK + target template + minimal module/GIT"
final_proof: "manual NWN EE Toolset/game screenshots and exact hashes"
forbidden:
  - "CEP/retail payload as repository fixture"
  - "aurora-web as validator or oracle"
  - "Three.js preview as final proof"
```

## 11. Otwarte decyzje i ich realny wplyw

| ID | Decyzja/brak | Blokuje teraz | Owner | Warunek zamkniecia |
|---|---|---|---|---|
| P0-ACCEPT | Akceptacja aktualnego planu i gate M1A | start M1A | Mateusz | jawne `start M1A` po przegladzie dokumentacji |
| GB-003 | Rust/WASM/wasm-pack | zamkniete 2026-07-11 | Mateusz + Codex | zweryfikowano: Rust/Cargo 1.96.1, rustfmt, clippy, `wasm32-unknown-unknown`, `wasm-pack` 0.15.0; Node 24.15.0 |
| GB-001 | Runtime evidence binary writera i wariant skin 17/64 | M4+ | Codex | M1B corpus report + semantic write/readback + Toolset/game |
| GB-004 | Runtime proof append-only `appearance.2da` i generated HAK/module | M5+ | Codex | own readback + `Appearance_Type`/RACE resolution w grze |
| GB-005 | Runtime proof self-contained animacji wybranego profilu | M4A+ | Codex | motion/state/event proof bez kopiowania external payloads |
| P-LICENSE | Licencja wlasnego repo | publiczne wydanie | Mateusz | dodany LICENSE albo jawna decyzja private/proprietary |
| P-NODE | Pin Node/package manager dla React/Vite | S1 | Mateusz + implementacja | `.node-version`/`packageManager` i lockfile |
| P-PROOF | Pierwszy proof na proxy `c_kocrachn` | M6 plan | Mateusz | potwierdzony profil proofu |

Brak prawdziwego Meshy GLB, prompty 2D i budzet API nie blokuja M1A.

## 12. Kolejnosc startowa

```text
1. Review the committed readiness and knowledge contracts
2. Receive explicit owner approval: start M1A
3. Record the already verified tool versions in M1A evidence
4. Scaffold only Cargo workspace, CI and repository hygiene files
5. Write failing synthetic M1A tests
6. Implement minimum checked parser
7. Pass native + WASM/Node gates
8. Update orchestrator-state.yaml and M1A evidence
9. Only then open M1B/M1C
```

Nie zaczynac rownolegle writera, HAK writera, React UI ani integracji Meshy API.

## 13. Zrodla operacyjne

- Rustup installation: https://rust-lang.github.io/rustup/installation/
- Rustup Windows MSVC prerequisites: https://rust-lang.github.io/rustup/installation/windows-msvc.html
- Rust `wasm32-unknown-unknown`: https://doc.rust-lang.org/stable/rustc/platform-support/wasm32-unknown-unknown.html
- Rust 1.96.1 distribution manifest: https://static.rust-lang.org/dist/channel-rust-1.96.1.toml
- `wasm-bindgen-test`: https://wasm-bindgen.github.io/wasm-bindgen/wasm-bindgen-test/
- `wasm-bindgen-test` usage with `wasm-pack`: https://wasm-bindgen.github.io/wasm-bindgen/wasm-bindgen-test/usage.html
