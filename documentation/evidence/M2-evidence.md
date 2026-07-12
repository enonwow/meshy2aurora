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

## M2-20260712-02 - contract correction for slice E

Status: IN_PROGRESS
Owner: Codex orchestrator + M2 contract/review subagents
Stage: M2 E (PENDING)

### Wynik korekty kontraktu

- Ustalono, ze brak `skin.inverseBindMatrices` jest poprawny i daje puste `inverseBindMatrices` w IR.
- Obecny inverse-bind accessor musi byc non-normalized `FLOAT/MAT4`, miec `count >= joints.len()` i zachowuje plaskie `[f32;16]` w glTF column-major.
- `IrAnimationSampler` zachowuje `outputAccessorType`; `targetPath` nalezy do `IrAnimationChannel`, bez dublowania znaczenia w samplerze.
- M2 E wymaga `LINEAR`, `STEP` i `CUBICSPLINE`; brak source interpolation canonicalizuje sie do glTF default `LINEAR`.
- `keyframeCount` jest suma input count samplerow, a duration animacji maksimum ich ostatnich czasow.
- Kanal `WEIGHTS` pozostaje deferred i daje exact `M2A-GLB-ANIMATION-WEIGHTS-DEFERRED` `BLOCKING`.
- Dodano project/WASM guardraile: `maxAnimationSamplers=100000`, `maxAnimationChannels=100000`, `maxDecodedSkinAnimationBytes=67108864`.
- Rozszerzono exact validation/negative matrix o IBM, TRS output layout/count, interpolation, refs, aggregate limits, decoded budget i checked overflow.

### Stan

Korekta jest docs-only. Nie zmienia kodu ani statusu etapu: M2 pozostaje `IN_PROGRESS`, a slice E pozostaje `PENDING` do czasu implementacji, testow i niezaleznego review.

## M2-20260712-03 - implementation A-F closed, final verification open

Status: VERIFYING
Owner: Codex orchestrator + M2 implementation/review subagents
Stage: M2

### Wynik implementacji

- Zamknieto pelny source-preserving ingest A-F: geometry, osie/winding/UV, materialy i embedded image identity, samplery, skin/IBM oraz animacje `LINEAR`, `STEP` i `CUBICSPLINE` pozostaja w glTF source basis.
- Slice D i E przeszly niezalezne review bez findings; historyczny wpis `E PENDING` powyzej opisuje stan korekty kontraktu, nie stan biezacy.
- Slice F obejmuje pelna macierz fatal/gate/limit/truncation/no-panic i ma `28/28 PASS` w `crates/m2a-core/tests/glb.rs`.
- Dodano i zweryfikowano blocking gates `M2A-GLB-INCOMPLETE-TRIANGLES` oraz `M2A-GLB-DEGENERATE-TRIANGLES`; osobno przechodza warning gates dla geometrii, brakujacych normals/base-color texture, optional extension, skin influence count i nierozstrzygnietego target transform.
- Finalna poprawka testowego helpera `POSITION` prawidlowo materializuje wariant missing-position; nie zmienia produkcyjnego kontraktu ani nie oslabia gate'a.
- Publiczne native/WASM API pozostaja deterministyczne, nie modyfikuja source bytes i nie serializuja BIN/image payloadu, sciezek hosta ani zewnetrznych assetow.

### Bramy jakosci

```yaml
glb_fixture_and_negative_tests:
  result: PASS
  count: 28
  expected: 28
native_workspace_tests:
  result: PASS
  count: 96
wasm_node_tests:
  result: PASS
  count: 12
quality_gates:
  cargo_fmt_all_check: PASS
  cargo_clippy_workspace_all_targets_D_warnings: PASS
  cargo_test_workspace: PASS
  wasm32_unknown_unknown_build: PASS
  wasm_pack_test_node: PASS
  git_diff_check: PASS
independent_reviews:
  slice_D: "NO FINDINGS"
  slice_E: "NO FINDINGS"
  final_full_scope: "NO FINDINGS"
stage_status: VERIFYING
```

### Docker build/test checkpoint

- `docker build --pull --target quality -t m2a-quality:local .` - PASS.
- `docker build --pull --no-cache --target quality -t m2a-quality:clean .` - PASS.
- Finalny zweryfikowany obraz ma tag `m2a-quality:final` i digest `sha256:9f84561c7271968bfb8de9997d97e33360e1765e217620897c404af236f6b620`.
- `docker image inspect` size: `1067266351` bytes; Docker CLI virtual size: `4.43GB`; build context: `212.69kB`; czas finalnego buildu: `137.5s`.
- SHA256 `crates/m2a-core/src/glb/mod.rs` jest identyczny na hoscie i w obrazie: `029f9c41319dda5b32a6bd33ae19cba9dedda8b1e2d5e7a50540190a4a11e2fa` - PASS.
- Przejrzane context, history i image content nie zawieraja retail/CEP HAK/MDL/MDX/KEY/BIF, sekretow ani absolutnych sciezek hosta.
- Obraz jest wylacznie build/test targetem `quality`: bez serwera, runtime'u produktu i bez substytutu proofu NWN EE na hoscie Windows.

### Current problems i nastepna akcja

```yaml
current_problems:
  - "Final documentation consistency scan is still pending before M2 can be marked DONE."
bugs: []
next_action: "Run the final documentation consistency scan; all implementation, quality, Docker and independent review gates already PASS."
```

## M2-20260712-04 - final review and stage closure

Status: DONE
Owner: Codex orchestrator + M2 implementation/review subagents
Stage: M2

### Finalny wynik

- Niezalezny finalny review calego post-fix zakresu A-F: `NO FINDINGS`.
- Finalny skan spojnosci kontraktu, evidence, Docker supplement, orchestrator state i aktywnego planu: `NO FINDINGS`.
- Source-preserving GLB ingest, publiczne native/WASM API, fatal/gate/limit/truncation/no-panic matrix i wszystkie synthetic fixtures A-F sa zamkniete.
- M2 celowo nie rozstrzyga Aurora target axis/scale/UV/winding. `UNRESOLVED_M3` jest teraz jawnym work itemem M3 i nie moze zostac zastapiony zgadywaniem.

```yaml
final_gates:
  glb_fixture_and_negative_tests: "28/28 PASS"
  native_workspace_tests: "96 PASS"
  wasm_node_tests: "12 PASS"
  cargo_fmt_all_check: PASS
  cargo_clippy_workspace_all_targets_D_warnings: PASS
  cargo_test_workspace: PASS
  wasm32_unknown_unknown_build: PASS
  wasm_pack_test_node: PASS
  git_diff_check: PASS
  independent_review_D: "NO FINDINGS"
  independent_review_E: "NO FINDINGS"
  independent_review_final_full_scope: "NO FINDINGS"
  final_documentation_consistency_scan: "NO FINDINGS"
docker_final:
  standard_quality_build: PASS
  no_cache_quality_build: PASS
  tag: "m2a-quality:final"
  digest: "sha256:9f84561c7271968bfb8de9997d97e33360e1765e217620897c404af236f6b620"
  inspect_size_bytes: 1067266351
  build_context: "212.69kB"
  build_elapsed: "137.5s"
  host_image_glb_rs_sha256: "029f9c41319dda5b32a6bd33ae19cba9dedda8b1e2d5e7a50540190a4a11e2fa"
  content_assets_secrets_paths_audit: PASS
current_problems: []
bugs: []
stage_status: DONE
handoff: "M3-20260712-01"
```
