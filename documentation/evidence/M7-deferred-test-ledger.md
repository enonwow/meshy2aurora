# M7-V1/V2 deferred test ledger

Status: `DEFERRED_UNTIL_SHARED_TEST_WAVE`

Ten ledger opisuje testy odroczone zgodnie z aktywnym suplementem
implementation-first. Nie jest evidence PASS i nie pozwala oznaczyc M7 jako
DONE.

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
