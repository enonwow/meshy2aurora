# Raport M1B - deep binary MDL reader

Data: 2026-07-11 | Autor: Codex | Status: DONE

## 1. Wynik checkpointu

Implementation i canonical corpus gate sa zielone. Own locator/reader zbudowal szesc packetow R1, R3a, R3b, R4 `c_nulltail`, R5 `c_vampire_f` i R6 `c_eye` z exact identity, hashami, ranges, capabilities i role invariants. Niezalezny final re-review nie znalazl findings; M1B ma status `DONE`.

## 2. Zaimplementowane funkcje

- model metadata, jawne core/raw ranges oraz base node tree z declared-vs-reachable budget;
- runtime `geometry_ptr` i `parent_ptr` traktowane jako ignore/inventory; parentage wynika z children traversal;
- controller keys/data z signed layoutem, checked indices i common types position, orientation, scale, self-illumination oraz alpha;
- wszystkie model animation roots, length, transition, animroot, events i controllery animation nodes;
- trimesh common prefix: faces, signed adjacency, vertices, UV0, normals, texture resrefs i walidacja pozostalych raw pointers;
- addytywne/deferred node families z zachowanym common mesh prefix i structured diagnostics;
- skin `legacy17` i `extended64`, classifier przez boundary `0x2d4/0x330` bez falszywego capacity limitu 17/64, checked map/bind arrays, raw weights/bone refs i canonical zachowanie `0xffff` w zero-weight lanes;
- zakresowe typed core claims, core/raw OOB, overflow, count, cycle, limit i truncation guards;
- deterministyczny JSON report przez Rust i publiczny adapter WASM;
- pure-bytes P-REF: own reader i SHA-256 na tych samych bytes, expected input/capabilities/invariants, safe logical provenance, schema binding i no-payload boundary.

## 3. Weryfikacja

| Command | Actual | Status |
|---|---|---|
| `cargo test --workspace` | 68 native tests: 2 unit + 18 ERF + 1 env integration clean-skip + 34 MDL + 13 P-REF; 0 failed | PASS |
| canonical real-env integration | 1 test; 6 packets R1/R3a/R3b/R4/R5/R6; 0 failed | PASS |
| `wasm-pack test --node crates/m2a-wasm` | 4 Node/WASM tests; 0 failed | PASS |
| `cargo fmt --all -- --check` | brak roznic formatowania | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | brak warnings | PASS |
| `cargo build --workspace` | `m2a-core` i `m2a-wasm` zbudowane | PASS |
| `git diff --check` | brak whitespace errors | PASS |

## 4. Review

- Wczesniejszy deep-reader/P-REF checkpoint zakonczyl review bez pozostalych findings.
- Final-review P1 dotyczace R0 capabilities, exact skin boundary/sentinel, R6 diagnostic i R5 nonzero bind pose maja status `FIXED`; niezalezny clean re-review nie znalazl findings.
- Regresje potwierdzaja, ze legacy17 map/bind count `28` jest walidowany przez arrays/ranges, extended64 count `38` przechodzi, a `0xffff` jest zachowany w zero-weight lanes.

## 5. Provenance i payload boundary

- Syntetyczne builders sa jedynymi payloadami testowymi w repo.
- Nie dodano retail/CEP MDL, MDX, tekstur, animacji, szkieletow ani extracted HAK/BIF payloadow.
- R1/R3a/R3b/R4/R5/R6 sa canonical own-locator/own-reader P-REF; R2 KEY/BIF pozostaje opcjonalny.
- P-REF identity nie zapisuje prywatnych host paths i rekurencyjnie odrzuca payload/bytes keys.

## 6. Real-binary correction

Pierwszy read-only real-binary smoke ujawnil, ze runtime `parent_ptr` nie jest serialized core pointerem. Poprawka wyprowadza parent relationship wylacznie z children traversal. Regresja i canonical R1/R3 handback sa zielone; finding ma status `FIXED`.

Full scan ujawnil drugi real-binary bug: `c_vampire_f` ma legacy17 header boundary, ale map/bind count `28`. Usunieto falszywy capacity limit wynikajacy z szerokosci headera; synthetic count28 i canonical R5 przechodza. `M1B-BUG-002` ma status `FIXED`, a final re-review nie znalazl findings.

## 7. Finalny status i handoff

M1B ma status `DONE`: 68 native tests, 4 WASM tests, szesc canonical packetow, pelne gates, privacy/payload boundary i niezalezny no-findings review sa zapisane w evidence. Jedyny aktywny etap zostal przekazany do M2 (`M2-20260711-01`) dla AuroraAssetIR i synthetic GLB axis/UV fixtures.

`GB-001-SKIN` jest strukturalnie `CLOSED`: 17/64 to header boundary, nie capacity, a canonical `0xffff` zostal zaobserwowany w zero-weight lanes. Globalny GB-001 writer/readback/runtime pozostaje `DIRECTION_DEFINED_EVIDENCE_OPEN` dla M4. Full-scan named gaps sa kontekstem inventory i nie blokuja DoD wybranych rol R4/R5/R6.
