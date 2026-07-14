# M7-V1--V4 test ledger

Status: `CODE_WAVE_PASS_REVIEW_CLEAN_M7_V5_DEFERRED`

Ten ledger opisuje testy odroczone zgodnie z aktywnym suplementem
implementation-first. Status PASS dotyczy wspolnej fali testow kodu; nie jest
real Meshy acceptance evidence i nie pozwala oznaczyc M7 jako DONE.

Shared wave 2026-07-14:

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace`: PASS, 319 testow.
- M7: 13 testow integracyjnych oraz 3 prywatne testy source-binding/readback/replay.
- Manifest JSON ma przetestowane camelCase, strict unknown-field rejection,
  deferred intake/batch i deterministyczne packety.
- Niezalezne finalne rereview po poprawkach: P1=0, P2=0.
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
- Bezposrednie regresje obejmuja undeclared i case-insensitive duplicate payload,
  brak wymaganego klipu, rigged source w obu rolach statycznych oraz unsafe path
  z drive, backslash, `.` i `..`.

## M7-V4W publiczny most WASM

- Completion sub-slice ma status `PASS_REVIEW_CLEAN`.
- Publiczne API `validateM7CorpusManifestV1Json`,
  `inspectM7CorpusIntakeV1Json` i `buildM7CorpusBatchV1` deleguja do
  kanonicznych API `m2a-core`.
- Jeden blob payloadow ma strict, wersjonowane deskryptory, checked ranges,
  exact coverage oraz stabilne bledy gap/overlap/OOB/overflow/cardinality.
- Native M7 adapter: 8/8 testow PASS; real generated Node/WASM boundary: PASS.
- READY fixture przechodzi przez publiczny WASM: 1 humanoid materialized i
  2 jawne deferred routes. Native i Node potwierdzaja ten sam frozen hash
  pelnego batch JSON `ee04ebfcdbb3e1265913de8f88d3c05f9277d18c7d0c75bdbcecc8139046c808`.
- Determinizm, packet order/identities, immutable input, no-base64, deferred
  flow i rzeczywisty wasm32 range overflow sa bezposrednio sprawdzone.
- Dwa niezalezne finalne review: P1=0, P2=0.
- Fixture jest repo-owned synthetic test input; nie jest Meshy acceptance
  evidence i nie otwiera M7-V5.

## Pozniejsze granice

- Publiczny WASM adapter i native/WASM parity sa domkniete przez `M7-V4W`.
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
- Materializowany route humanoid powstaje tylko przez prywatny wrapper, ktorego
  constructor sam uruchamia canonical `build_m6_model_package_v1`; nie ma
  callbacka, alternatywnego konwertera ani publicznych pol do podmiany artifactu.
- Source bytes sa zwiazane z typed ingest/conversion evidence, summary,
  manifestem i ich canonical JSON; mutacja `source_glb` uniewaznia artifact.
- Humanoidowego M6 artifactu nie mozna przypisac do non-humanoid ani static.
- Non-humanoid reference-supermodel i static placeable/item maja jawne route
  `DEFERRED_M7_V5` oraz stabilne diagnostyki; ich canonical pipeline nie jest
  falszowany w first pass.
- Canonical HAK bytes, writer report i `PackageManifestV1` musza miec ten sam
  hash, dlugosc, zasoby, offsety i resource ids.
- Own `ErfArchive` readback potwierdza dokladnie wszystkie manifest resources.
- Canonical writer replay z payloadow odczytanych z realnego HAK musi odtworzyc
  identyczne HAK bytes, writer report i package manifest.
- Conversion report musi byc canonical JSON zgodnym z typed reportem i ma
  wlasna byte identity.
- Bezposrednie regresje odrzucaja unknown/duplicate canonical artifact, wspolna
  mutacje resource metadata po HAK readback oraz wspolna mutacje writer report
  i package manifest podczas exact replay.

## M7-V4 per-profile proof packets

- Zawsze powstaja dokladnie trzy packety w stabilnej kolejnosci rol.
- Kazdy packet ma jawny canonical route, source identity, output inventory,
  package manifest, diagnostyki i status bramek.
- Packet bez modelu ma `INPUT_DEFERRED`, puste canonical outputs i
  `m7DoneClaimAllowed=false`.
- Gotowy humanoid canonical package ma `CANONICAL_PACKAGE_MATERIALIZED`
  dopiero po own HAK readback i exact writer replay.
- Hash packet JSON jest zapisany w batch report i jest deterministyczny.
- Batch z trzema canonical packets nadal ma `m7DoneClaimAllowed=false`;
  M7-V5, real E2E i external acceptance pozostaja odroczone.
