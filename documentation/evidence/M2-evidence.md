# M2 Evidence

Append-only evidence log dla M2 GLB ingest i canonical AuroraAssetIR.

## M2-20260711-01 - contract lock

Status: IN_PROGRESS
Owner: Codex orchestrator + M2 implementation/review subagents
Stage: M2

### Cel proby

Zamknac implementowalny schema/API/fixture/error contract przed rozpoczeciem TDD, bez zgadywania target Aurora transform i bez zewnetrznych GLB payloadow.

### Wynik

- Utworzono `documentation/m2-aurora-asset-ir-glb-kontrakt-suplement-codex.md`.
- M2 zachowuje source glTF basis, units, winding i UV; `targetTransformStatus=UNRESOLVED_M3`.
- Zablokowano GLB 2.0 embedded subset, schema v1, project limits, fatal errors, warning/blocking gates, native/WASM API, fixtures A-F, negative matrix i Definition of Done.
- Limity sa jawnie project/WASM guardrails, nie faktami Aurory.
- Geometry policy: warning >5000 triangles, blocking >10000.
- Inspection poprawnie zakodowanego GLB zwraca report nawet z blocking gates; fatal jest tylko malformed/unsupported encoding lub safety limit.
- Nie dodano kodu, GLB, obrazow, retail/CEP payloadow ani host paths.

### Aurora First i provenance

- `documentation/PROJECT_RULES.md` - Aurora First, TDD i synthetic proof base.
- `documentation/plan-implementacji-orkiestrator-codex.md` - M2 scope/DoD.
- `documentation/architektura-meshy2aurora-codex.md` - Rust `gltf`, pure core i module boundary.
- `documentation/architektura-web-wasm-codex.md` - bytes-once, versioned JSON i public WASM boundary.
- `documentation/konwersja-meshy-odpowiedz-codex.md` - forward/unit OPEN, UV evidence i product geometry guardrails.

### Current problems

```yaml
current_problems:
  - "M2 implementation and synthetic fixtures A-F do not exist yet."
  - "Target Aurora transform is intentionally UNRESOLVED_M3 and is not an M2 blocker."
bugs: []
```

### Nastepny krok

TDD: najpierw fixture A-C i native report/IR, potem D-E inventory, F gates/limits, public WASM adapter, full negative matrix i independent review. Nie implementowac target transform ani M3 scope w M2.

## M2-20260712-01 - checkpoint implementacyjny A-C

Status: IN_PROGRESS
Owner: Codex orchestrator + M2 implementation/review subagents
Stage: M2

### Zakres checkpointu

Zaimplementowano source-preserving GLB 2.0 inspect/ingest dla slice A-C oraz publiczna granice WASM. Ten checkpoint nie zamyka M2: materialy/obrazy, skin, animacje, fixtures D-F, pelna macierz negatywna i finalny review pozostaja otwarte.

### Zaimplementowany wynik A-C

- `m2a-core` udostepnia wersjonowane `inspect_glb` i `ingest_glb`, budujace deterministyczny `GlbInspectionReport` oraz `AuroraAssetIR` w `GLTF_SOURCE`.
- Fixtures A-C dowodza minimalnego indexed triangle, zachowania osi/winding oraz zachowania UV bez target swapu i bez `1-v`.
- Publiczne adaptery WASM `inspectGlb` i `ingestGlb` deleguja do tego samego core, zwracaja stabilny JSON/report lub error envelope i nie modyfikuja input bytes.
- Macierz testowa GLB ma 11 testow; ostatni zweryfikowany native workspace gate ma 79 testow.
- Publiczna macierz WASM ma 6 testow obejmujacych istniejacy MDL adapter oraz nowe GLB inspect/ingest success/error/determinism checks.
- Finalny post-fix checkpoint przeszedl `cargo fmt`, clippy `-D warnings`, 79 native workspace tests, WASM target build, 6 Node/WASM tests oraz `git diff --check`.
- Findings security/correctness dla A-C zostaly naprawione: limity i skumulowany internal decoded-memory budget sa sprawdzane przed alokacja, a external URI, sparse accessor i compression/required-extension przypadki maja stabilna klasyfikacje.
- Finalny niezalezny re-review zakresu A-C po poprawkach nie wykazal findings.
- Utworzono docs-only `documentation/docker-build-test-suplement-codex.md`; nie dodano jeszcze `Dockerfile` ani `.dockerignore`, a Docker nie jest runtime produktu ani substytutem proofu NWN EE na hoscie Windows.

### Stan bramek

```yaml
native_workspace_tests:
  result: PASS
  count: 79
glb_tests:
  result: PASS
  count: 11
wasm_node_tests:
  final_post_fix_result: PASS
  count: 6
quality_gates:
  cargo_fmt: PASS
  cargo_clippy_D_warnings: PASS
  wasm_target_build: PASS
  git_diff_check: PASS
independent_review_A_C:
  result: "NO FINDINGS"
stage_status: IN_PROGRESS
```

Zielone bramki dotycza checkpointu A-C po finalnej poprawce internal memory-budget. Nie zamykaja one M2, poniewaz D-F i finalny full-scope review nadal nie istnieja.

### Changed files checkpointu

- `Cargo.lock`
- `crates/m2a-core/Cargo.toml`
- `crates/m2a-core/src/lib.rs`
- `crates/m2a-core/src/glb/mod.rs`
- `crates/m2a-core/tests/fixtures/build_synthetic_glb.rs`
- `crates/m2a-core/tests/glb.rs`
- `crates/m2a-wasm/src/lib.rs`
- `documentation/m2-aurora-asset-ir-glb-kontrakt-suplement-codex.md`
- `documentation/docker-build-test-suplement-codex.md`
- `documentation/evidence/M2-evidence.md`
- `documentation/orchestrator-state.yaml`

### Current problems

```yaml
current_problems:
  - "D-F remain open: materials/images, skin, animation inventory, fixtures/gates/limits and the full negative matrix."
  - "Target Aurora transform remains intentionally UNRESOLVED_M3 and is not an M2 blocker."
bugs: []
```

### Nastepne akcje

1. Zaimplementowac fixture D oraz material/image inventory bez payloadu w serializowanym JSON.
2. Zaimplementowac fixture E oraz skin/inverse-bind i translation/rotation/scale animation inventory w source basis.
3. Domknac fixture F, limity i pelna fatal/gate/truncation/no-panic negative matrix.
4. Uruchomic finalne native i WASM gates po wszystkich poprawkach.
5. Wykonac niezalezny final M2 review; dopiero green D-F + gates + review pozwalaja rozwazyc `DONE`.
