# M1A Evidence

Ten plik jest append-only dziennikiem dowodow dla etapu M1A. Biezacy stan etapu pozostaje w `documentation/orchestrator-state.yaml`.

## M1A-20260711-01 - 2026-07-11

Status: IN_PROGRESS
Owner: Codex orchestrator + M1A implementation subagents
Stage: M1A

### Cel proby

Zbudowac minimalny workspace Rust/WASM oraz bezpieczny parser strukturalny binary MDL, ktory przechodzi wszystkie natywne i Node/WASM gate'y M1A.

### Aurora First / provenance

- `documentation/PROJECT_RULES.md` - zasada Aurora First i granice standalone.
- `documentation/prompt-dla-claude-prototyp-parsera.md` - zaakceptowany zakres M1A.
- `documentation/mdl-binary-crosswalk-codex.md` - potwierdzony kierunek layoutu binary MDL.
- Syntetyczne fixture generowane w kodzie testowym; bez retail/CEP payloadow.

### Zmienione pliki

- `documentation/plan-implementacji-orkiestrator-codex.md` - odnotowano start M1A i regule checkpointow Git.
- `documentation/orchestrator-state.yaml` - P0 ustawiono na DONE, M1A na IN_PROGRESS.
- `documentation/evidence/M1A-evidence.md` - utworzono dziennik proby.

### Weryfikacja

| Command or action | Expected | Actual | Status |
|---|---|---|---|
| Jawna zgoda wlasciciela na rozpoczecie | P0 acceptance i start M1A | Otrzymano 2026-07-11 | PASS |
| `git switch -c codex/m1a-structural-parser` | Osobna galaz od baseline dokumentacji | Utworzono galaz | PASS |

### Problemy i bledy

```yaml
current_problems:
  - "Implementacja oraz pelne gate'y M1A sa w toku."
bugs: []
```

### Evidence artifacts

- `documentation/orchestrator-state.yaml` - aktywny etap i attempt id.

### Nastepny krok

Zintegrowac rdzen parsera i adapter WASM, a nastepnie uruchomic komplet polecen Definition of Done.

## M1A-20260711-01 - final verification

Status: DONE
Owner: Codex orchestrator + M1A implementation and audit subagents
Stage: M1A

### Cel proby

Zamknac pelny Definition of Done M1A po niezaleznym security/code review oraz zapisac zweryfikowany checkpoint do publikacji.

### Aurora First / provenance

- `documentation/mdl-binary-crosswalk-codex.md` - file/core/MDX pointer contract i rozmiary struktur.
- `documentation/m1a-kontrakt-suplement-codex.md` - stabilny JSON/error envelope i named fixed-layout constants.
- Syntetyczne fixture w `crates/m2a-core/tests/fixtures/build_minimal_binary_mdl.rs`; brak zewnetrznych payloadow.

### Zmienione pliki

- Pelna lista znajduje sie w `documentation/prototyp-parsera-m1a-claude.md` oraz `documentation/orchestrator-state.yaml`.

### Weryfikacja

| Command or action | Expected | Actual | Status |
|---|---|---|---|
| `rustc --version --verbose` | Rust 1.96.1 | `rustc 1.96.1 (31fca3adb 2026-06-26)` | PASS |
| `cargo --version` | Cargo dla 1.96.1 | `cargo 1.96.1 (356927216 2026-06-26)` | PASS |
| `wasm-pack --version` | pinned 0.15.0 | `wasm-pack 0.15.0` | PASS |
| `node --version` | Node >=24 | `v24.15.0` | PASS |
| `cargo fmt --all --check` | no formatting diff | exit 0 | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | no warnings | exit 0 | PASS |
| `cargo test --workspace` | all native tests pass | 20 passed, 0 failed | PASS |
| `cargo build -p m2a-wasm --target wasm32-unknown-unknown` | WASM target builds | exit 0 | PASS |
| `wasm-pack test crates/m2a-wasm --node` | public Node adapter executes | wasm-pack 0.15 rejected argument order | FAIL |
| `wasm-pack test --node crates/m2a-wasm` | public Node adapter executes | 2 passed, 0 failed | PASS |
| `git diff --check` | no whitespace errors | exit 0 | PASS |
| independent final re-review | no unresolved P1/P2 findings | brak findings | PASS |

### Problemy i bledy

```yaml
current_problems: []
bugs:
  - id: "M1A-BUG-001"
    severity: "P1"
    status: "FIXED"
    repro: "core pointer wskazuje do appended MDX"
    expected: "M2A-MDL-POINTER-OOB"
    actual: "przed poprawka payload MDX mogl zostac sparsowany jako node"
    next_action: "zamkniete przez core-relative lower/upper range validation"
  - id: "M1A-BUG-002"
    severity: "P1"
    status: "FIXED"
    repro: "root lub child node nachodzi na ModelHeader"
    expected: "M2A-MDL-POINTER-OOB"
    actual: "przed poprawka overlap mogl zostac zaakceptowany"
    next_action: "zamkniete przez minimalny node offset 0xE8"
  - id: "M1A-BUG-003"
    severity: "P2"
    status: "FIXED"
    repro: "normalny contentFlags 0x001"
    expected: "brak unsupported warning"
    actual: "przed poprawka zuzywal diagnostic budget"
    next_action: "zamkniete przez jawne rozpoznanie header flag"
  - id: "M1A-BUG-004"
    severity: "P2"
    status: "FIXED"
    repro: "caller podnosi max_depth dla glebokiego drzewa"
    expected: "brak ryzyka stack overflow"
    actual: "pierwsza wersja traversalu byla rekurencyjna"
    next_action: "zamkniete przez iteracyjny traversal i iteracyjne skladanie drzewa"
```

### Evidence artifacts

- `documentation/prototyp-parsera-m1a-claude.md` - raport implementacji i klasyfikacja pol.
- `crates/m2a-core/tests/mdl/parser.rs` - negatywne, graniczne i deterministyczne testy parsera.
- `crates/m2a-wasm/src/lib.rs` - test publicznego adaptera uruchamiany w Node.

### Nastepny krok

Wykonac checkpoint commit i push M1A, nastepnie aktywowac M1B bez uruchamiania M1C/M2 rownolegle.
