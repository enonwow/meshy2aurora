# Raport implementacji Animation Studio

- Data: 2026-07-28
- Branch: `animation`
- Zakres: F1-F11 z planu `plan-implementacji-animation-studio-2026-07-28.md`
- Status: `F1-F10 COMPLETE / F11 BLOCKED_BY_MODEL_ITERATION_GATE`

## Wynik

Animation Studio działa jako sub-mode istniejącego kroku
`3 Animation Mapping`. Użytkownik może utworzyć klip z pozy, wykonać
edytowalną kopię klipu źródłowego, edytować translację i rotację kości,
keyframe'y, eventy, trim i retime, korzystać z undo/redo oraz IndexedDB,
zapisać klip do Custom i przypisać poprawny Custom do Base 42.

Build V5 jest addytywny względem V4. Przyjmuje exact source GLB, mapping V2 i
dokument Studio V1, materializuje authored motion, emituje eventy, wykonuje
własny binary MDL readback, zapisuje fingerprint i blokuje wynik przy
mismatchu. Źródłowy GLB pozostaje niezmieniony.

## Zaimplementowane warstwy

- strict Rust/TypeScript contracts, stable serializer, SHA-256 fingerprint i
  deterministyczna migracja V1 -> V2;
- kanoniczne operacje clip/track/keyframe/event oraz wspólny golden TS/Rust;
- limity produktu niezależne od granicy formatu writera;
- API Rust/WASM, Worker, transferables, supersede/cancel i lane
  `H1_SKINNED_FULL_42_EDITED`;
- IndexedDB, recovery, autosave, undo/redo i revision binding;
- biblioteka Source/Edited/Generated, statusy Draft/Valid/Invalid i exact
  source-inventory resolution dla migrowanych Custom;
- viewport THREE, drzewo kości, inspector numeryczny, dope sheet, transport,
  timeline i operacje klawiaturowe;
- event editor, trim i retime;
- Custom picker, stable-ID assignment do Base 42 i powrót do exact authored
  clip;
- V5 manifest, Review, source-unchanged evidence oraz readback reconciliation.

## Istotne problemy domknięte podczas finalnego gate'u

1. UI dopuszczało krótkie okno, w którym `Continue to Build` było aktywne przed
   nadejściem kanonicznego fingerprintu. Selector i reducer korzystają teraz z
   tej samej fail-closed bramy.
2. Runtime normalizuje root modelu do resrefu, a profil proceduralny może
   wcześniej dodać osobny controllerless root. Readback rozpoznaje teraz
   dokładny przypadek: zmienia nazwę w mapie Studio tylko wtedy, gdy runtime
   root jest tym samym node ID. Testy obejmują oba warianty.
3. Finite wartości zapisane notacją wykładniczą mogły dać różne
   reprezentacje fingerprintu Rust/TypeScript. Wspólny limit produktu blokuje
   ten zakres przed serializacją i buildem.

## Finalne gate'y

| Gate | Wynik |
|---|---|
| `cargo fmt --all --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS; wszystkie testy nieignorowane, 0 failures |
| ignored `r46_v4_compatibility_audit` | PASS 1/1 |
| `npm run typecheck` | PASS |
| `npm test` | PASS: 59 plików + 1 skipped; 317 testów + 1 skipped |
| `npm run build` | PASS |
| `npm run test:worker-integration` | PASS: 6 plików, 16/16 testów |
| browser IndexedDB persistence | PASS: 1/1 |
| V5 real Worker/WASM/readback/repeat | PASS: 2/2 |

Jawny audyt r46/V4 zachował:

- authoring fingerprint:
  `9e09be1e1307f96ae9a6b7e91a5cf6b33e6105d2ff3b1981df4ffe06d618affe`;
- model:
  `d788f07137c7bf713f18654a14f0ce0559e46315cbb2558ea6dcb6fb7fc8d739`;
- HAK:
  `027a9fc32fee9f75447c63f7308def35a1d098018d68d4d788d39c1174c7f98a`;
- MOD:
  `b206f5507ce14406d62716f9e75c029044e9090f8ead1cdebfc2574e887b959c`.

## Pomiar wydajności

Pomiar na kanonicznym
`sample-3d/h1-humanoid-1500/source.glb`:

| Pole | Wynik |
|---|---:|
| kości | 24 |
| authored tracks | 48 |
| authored keyframes | 48 |
| dokument | 9 633 B |
| 100 projekcji do THREE | 13 ms |
| średnia projekcja | 0,13 ms |
| 50 000 aktualizacji gestu | 3,6 ms |

Limity produktu: 64 authored clips, 10 000 keyframe'ów na klip, 100 000
łącznie, 2 000 markerów DOM, 86 400 s długości i bezwzględna wartość tracku
1 000 000. Limity nie używają `u16` writera jako limitu UX.

## F11 i human-owned proof

F11 pozostaje zablokowane. Obowiązujący dokładny r46 ma właścicielski wynik
`visible`; brak świeżego `modelVisibility=not_visible` nie dopuszcza nowej
iteracji, resrefu, HAK-a ani MOD-a. Agent nie uruchamiał Toolsetu/NWN, nie
utworzył kandydata V5 i nie deklaruje `ready_for_owner_proof`.

Warunek wznowienia: bezpośrednia decyzja właściciela dopuszczająca jednego
dokładnego kandydata V5 albo świeży, exact candidate-bound wynik
`modelVisibility=not_visible`, który spełni model iteration gate.

Pełny zapis blokady:
[`evidence/animation-studio-v5-owner-proof-gate-2026-07-28.md`](evidence/animation-studio-v5-owner-proof-gate-2026-07-28.md).
