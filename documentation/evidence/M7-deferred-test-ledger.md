# M7-V1/V2 deferred test ledger

Status: `SHARED_TEST_WAVE_PASS`

Ten ledger opisuje testy odroczone zgodnie z aktywnym suplementem
implementation-first. Nie jest evidence PASS i nie pozwala oznaczyc M7 jako
DONE.

Shared wave 2026-07-14:

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS, 298 testow, w tym 3 nowe testy M7.
- Manifest JSON ma przetestowane camelCase, strict unknown-field rejection,
  deferred intake/batch i deterministyczne packety.
- Ten PASS nie otwiera M7-V5 i nie jest real Meshy acceptance evidence.

## M7-V1 corpus contract

- Dokladnie trzy wpisy i po jednej wymaganej roli.
- Stabilne bledy schema, cardinality, role coverage i duplicate identity.
- Strict JSON odrzuca nieznane pola.
- Bezpieczne relatywne sciezki `.glb`; bez drive, backslash, `.` lub `..`.
- Obecne zrodlo wymaga dlugosci, lowercase SHA-256 i atestacji original Meshy.
- Brak descriptorow jest poprawnym `INPUT_DEFERRED`, nigdy synthetic substitute.
- Humanoid wymaga nazw klipow; non-humanoid poprawnego resrefa supermodelu.

## M7-V2 intake

- Brak realnych wejsc zwraca `INPUT_DEFERRED`, `realExecutionReady=false` i
  `m7DoneClaimAllowed=false`.
- Payload spoza manifestu, duplikat case-insensitive i identity mismatch sa invalid.
- Truncated GLB daje stabilna diagnostyke bez panic.
- Humanoid wymaga skin/animations/nazwanych klipow.
- Non-humanoid i static route odrzucaja source skin/animations.
- Trzy poprawne role otwieraja tylko `READY_FOR_M7_V5`; nadal nie pozwalaja
  oglosic M7 DONE.
- Raport jest stabilnie uporzadkowany wedlug roli.

## Pozniejsze granice

- Publiczny WASM adapter i native/WASM parity naleza do pozniejszego slice.
- Syntetyczne unit fixtures nie sa korpusem ani acceptance evidence.
- Trzy oryginalne Meshy GLB, full pipeline i proof packety pozostaja
  `DEFERRED_INPUT_GATE`.

## M7-V3 canonical batch runner

- Runner zawsze zaczyna od `inspect_m7_corpus_intake_v1`.
- Brak descriptorow, payloadow lub canonical outputs materializuje trzy
  uporzadkowane packety `INPUT_DEFERRED`.
- Artifact dla nieznanego sample, duplikat artifactu i artifact dolaczony do
  niegotowego source maja stabilne bledy.
- Source identity artifactu musi byc identyczne z observed intake identity.
- Runner przyjmuje `ModelPackageArtifactV1`; nie ma callbacka ani drugiego
  konwertera.
- Canonical HAK bytes, writer report i `PackageManifestV1` musza miec ten sam
  hash, dlugosc, zasoby, offsety i resource ids.
- Own `ErfArchive` readback potwierdza dokladnie wszystkie manifest resources.
- Conversion report musi byc poprawnym JSON i ma wlasna byte identity.

## M7-V4 per-profile proof packets

- Zawsze powstaja dokladnie trzy packety w stabilnej kolejnosci rol.
- Kazdy packet ma source identity, output inventory, package manifest,
  diagnostyki i status bramek.
- Packet bez modelu ma `INPUT_DEFERRED`, puste canonical outputs i
  `m7DoneClaimAllowed=false`.
- Gotowy canonical package ma `CANONICAL_PACKAGE_MATERIALIZED` dopiero po own
  HAK readback.
- Hash packet JSON jest zapisany w batch report i jest deterministyczny.
- Batch z trzema canonical packets nadal ma `m7DoneClaimAllowed=false`;
  M7-V5, real E2E i external acceptance pozostaja odroczone.
