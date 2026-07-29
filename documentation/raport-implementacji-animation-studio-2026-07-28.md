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
| `npm test` | PASS: 60 plików + 1 skipped; 324 testy + 1 skipped |
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

## Korekta zgodności wizualnej z mockupem 02

Pierwsza funkcjonalna wersja F5/F6 nie realizowała wystarczająco wiernie
hierarchii wizualnej zaakceptowanego mockupu
`02-animation-studio-within-mapping.png`. Dnia 2026-07-28 wykonano osobny pass
visual parity:

- `Create & edit` pozostaje sub-mode kroku `Animation Mapping`;
- wszystkie standardowe ekrany Studio używają jednego pionowego workflow raila;
- `Create & edit` nie przełącza aplikacji na osobny shell ani layout;
- biblioteka, viewport i inspector tworzą jeden trzykolumnowy workbench;
- dope sheet zajmuje pełną szerokość pod trzema panelami;
- toolbar viewportu obsługuje prawdziwe przełączanie `Source | Edited` oraz
  fullscreen;
- inspector używa kompaktowego wyboru kości, transformacji i akcji keyframe;
- zapis do Custom, undo/redo, autosave oraz status pozostają częścią tego
  samego ekranu;
- układ mieści kompletny stan roboczy w widoku 1600 x 1000 i ma breakpointy
  dla węższych ekranów.

Weryfikacja po korekcie: `npm test` — 319 passed, 1 skipped; `npm run build` —
PASS.

## F12: kopiowanie animacji z innego modelu

Dnia 2026-07-28 dodano opcję `Copy from another model...` wewnątrz menu
`+ New animation`. Nie powstał nowy krok workflow. Lokalny donor GLB jest
inspektowany w pamięci, a Studio pokazuje inventory klipów, czas, liczbę
tracków, liczbę kości oraz skrót SHA-256.

Import jest fail-closed. Output rig bieżącego modelu i dawcy musi mieć te same
ID kości, nazwy, parenty oraz local rest translation/rotation. Różny rig zwraca
`M2A-ANIMATION-IMPORT-RIG-MISMATCH`; UI pokazuje pierwszą różnicę i nie
odblokowuje `Copy to Custom`. Automatyczny retarget różnych szkieletów nie jest
częścią tej fazy.

Zgodny klip jest kopiowany jako self-contained authored tracks z provenance
`IMPORTED_MODEL_COPY`: SHA-256 dawcy, source clip name i exact clip
fingerprint. Donor GLB nie jest przechowywany w projekcie ani potrzebny do
późniejszego builda. Automatyczna nazwa `imp_<clip>` jest ograniczona do 16
znaków.

Weryfikacja:

- TypeScript: 29/29 testów celowanych;
- pełny `npm test`: 60 plików i 324 testy PASS, 1 plik/test skipped;
- Rust/WASM: boundary inventory test PASS;
- `cargo test --workspace`: PASS;
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS;
- `npm run build`: PASS;
- browser compatible fixture: import, Save i status `VALID`;
- browser real H1 24-bone przeciwko fixture 2-bone: `Different rig`, akcja
  kopiowania disabled;
- console errors: 0.
