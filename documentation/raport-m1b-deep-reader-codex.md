# Raport M1B - deep binary MDL reader

Data: 2026-07-11 | Autor: Codex | Status: VERIFYING, NIE DONE

## 1. Wynik checkpointu

Synthetic M1B checkpoint jest zielony. Deep reader, pure-bytes P-REF contract i publiczna granica WASM przeszly aktualna macierz native/Node. M1B nie jest `DONE`: canonical own-reader P-REF R1/R3, wybor R4-R6 i evidence GB-001-SKIN wymagaja najpierw wlasnego locatora M1C.

## 2. Zaimplementowane funkcje

- model metadata, jawne core/raw ranges oraz base node tree z declared-vs-reachable budget;
- runtime `geometry_ptr` i `parent_ptr` traktowane jako ignore/inventory; parentage wynika z children traversal;
- controller keys/data z signed layoutem, checked indices i common types position, orientation, scale, self-illumination oraz alpha;
- wszystkie model animation roots, length, transition, animroot, events i controllery animation nodes;
- trimesh common prefix: faces, signed adjacency, vertices, UV0, normals, texture resrefs i walidacja pozostalych raw pointers;
- addytywne/deferred node families z zachowanym common mesh prefix i structured diagnostics;
- skin `legacy17` i `extended64`, jawny classifier boundary, map/bind limits, raw weights/bone refs i zachowanie nierozstrzygnietego `0xffff` bez wymyslania semantyki;
- zakresowe typed core claims, core/raw OOB, overflow, count, cycle, limit i truncation guards;
- deterministyczny JSON report przez Rust i publiczny adapter WASM;
- pure-bytes P-REF: own reader i SHA-256 na tych samych bytes, expected input/capabilities/invariants, safe logical provenance, schema binding i no-payload boundary.

## 3. Weryfikacja

| Command | Actual | Status |
|---|---|---|
| `cargo test --workspace` | 49 native tests: 2 unit + 34 MDL + 13 P-REF; 0 failed | PASS |
| `wasm-pack test --node crates/m2a-wasm` | 4 Node/WASM tests; 0 failed | PASS |
| `cargo fmt --all -- --check` | brak roznic formatowania | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | brak warnings | PASS |
| `cargo build --workspace` | `m2a-core` i `m2a-wasm` zbudowane | PASS |
| `git diff --check` | brak whitespace errors | PASS |

## 4. Review

- Pierwszy deep-reader review: osiem findings dotyczacych controller layout, skin counts/sentinel, addytywnych flags, core overlap, JSON inventory/adjacency i deep WASM coverage; wszystkie maja status `FIXED` po testach regresyjnych.
- Finalny P-REF review: brak findings.
- Finalny deep-reader code/test re-review: brak findings.
- Stare repro potwierdzaja poprawki: invalid common-controller columns i legacy map count `18` sa odrzucane; `0xffff` jest zachowany; supported mesh ma puste `unsupportedFamilies`; adjacency raportuje `[-1, -1, -1]`.

## 5. Provenance i payload boundary

- Syntetyczne builders sa jedynymi payloadami testowymi w repo.
- Nie dodano retail/CEP MDL, MDX, tekstur, animacji, szkieletow ani extracted HAK/BIF payloadow.
- Container-level R1/R2/R3 hashes pozostaja inventory, nie canonical own-reader P-REF.
- P-REF identity nie zapisuje prywatnych host paths i rekurencyjnie odrzuca payload/bytes keys.

## 6. Real-binary correction

Pierwszy read-only real-binary smoke ujawnil, ze runtime `parent_ptr` nie jest serialized core pointerem. Poprawka wyprowadza parent relationship wylacznie z children traversal. Regresja jest zielona i finding ma status `FIXED`; ponowienie na canonical bytes nastapi po M1C.

## 7. Pozostaly warunek M1B

M1B pozostaje `VERIFYING`. Nastepna sekwencja jest obowiazkowa:

1. M1C implementuje wlasny read-only HAK/ERF locator;
2. M1B wraca do `IN_PROGRESS` dla canonical P-REF R1/R3;
3. M1B wybiera R4-R6 i zamyka albo jawnie klasyfikuje GB-001-SKIN;
4. dopiero finalne evidence moze promowac M1B z `VERIFYING` do `DONE`.
