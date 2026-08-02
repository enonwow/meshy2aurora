# Animation Authoring V2 — stan implementacji i evidence — 2026-07-31

## Wynik

Zaimplementowano działający vertical slice audytu Animation Authoring V2 przez
Core → WASM → Worker → Studio. Zmiana nie tworzy nowej iteracji MOD/HAK i nie
uruchamia Aurora Toolset ani NWN.

## Gotowe

- kanoniczne, rewizyjne i atomowe command batches dla edycji klipów;
- local/world pose sampler oraz pose-parity report;
- Motion Quality Analyzer: static playback, loop seam, root drift, ground
  penetration, foot sliding i motion spikes;
- narzędzia loop blend, Preserve/InPlace/Scale root motion, contact suggestion
  i lock oraz error-bounded key reduction;
- prawdziwy `TransformControls` 3D w viewportcie z blokadą kamery podczas gestu;
- `HeldWeaponAttachmentV1`, ścisła inspekcja rigid GLB, jawny target bone,
  preview przez cały klip, grip metrics oraz IR-level rigid bake bez utraty
  trójkątów;
- semantic humanoid retarget V2 z wersjonowanymi aliasami, required/optional
  semantics, chain checks i fail-closed manual confirmation;
- transakcyjny Core batch transfer `ALL_OR_NOTHING`;
- sequence preview `idle → action → idle` z event offsets i transition-jump
  telemetry;
- warstwy Base/Additive/Override z wagą, maską, mute/solo i deterministycznym
  bake do LINEAR;
- editor curves z Hermite translation, shortest-arc quaternion rotation oraz
  adaptacyjnym resamplingiem do LINEAR;
- UI dla semantic retarget, quality repairs, held equipment, sequence preview,
  curve smoothing i correction layers;
- 30 typowanych requestów Worker/WASM i exhaustive contract gate.

## Częściowe — nie wolno nazywać ukończonym

- weapon bake działa na wspólnym `AuroraModelIrV1`, ale finalny product builder
  nie przyjmuje jeszcze weapon GLB/texture payload i attachment jako jednego
  persisted build input; obecny panel daje Core-inspected preview, nie gotowy
  finalny eksport broni;
- manual semantic overrides istnieją w Core, ale UI nie ma jeszcze edytora
  niejednoznacznych mapowań;
- batch transfer jest atomowy w Core, ale dialog nie ma jeszcze multi-select,
  progress/retry ani exact Fogbound 9/9 browser proof;
- warstwy są bake'owane z UI, lecz stack nie jest jeszcze persisted jako
  osobny dokument edytora;
- sequence preview obsługuje podstawowy idle/action/idle; wspólny phased/event
  timeline oraz walk→stop pozostają do wykonania;
- pose parity nie jest jeszcze dołączone do packaged binary readback i Review
  Download gate;
- motion-diversity MP4 manifest i finalny offline demo packet nie zostały w tej
  zmianie wygenerowane.

## Bramki wykonane

| Bramka | Wynik |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS (aktywne testy; fixture-dependent ignored bez zmiany) |
| Studio typecheck + Worker/WASM contract | PASS, 30/30 request cases, 44 imports |
| Studio full suite | PASS, 420 passed / 3 skipped |
| Worker/WASM browser integration | PASS, 25 passed / 2 skipped |
| Browser persistence | PASS, 2/2 |
| Production Vite build | PASS |
| Bundle budget | PASS po zapisaniu jawnego V2 baseline; main/max chunk bez podniesienia |
| Animation library catalog/immutability | PASS, 7 presets |

Zmierzony production bundle:

- main JS: 421,898 B / limit 460,000 B;
- max JS chunk: 601,685 B / limit 620,000 B;
- total JS: 1,526,605 B / limit 1,530,000 B;
- total CSS: 155,977 B / limit 157,000 B;
- WASM: 3,942,470 B / limit 4,000,000 B;
- WASM gzip: 1,427,107 B / limit 1,430,000 B.

## Bezpieczeństwo i lineage

- source/donor/weapon są czytane jako wejścia; testy nie mutują ich payloadów;
- nie powstał drugi asset root;
- nie utworzono ani nie zainstalowano nowego MOD/HAK;
- nie uruchomiono i nie kontrolowano Toolset/NWN;
- runtime proof pozostaje własnością użytkownika.

## Następny checkpoint

Najbliższy krytyczny checkpoint to persisted held-weapon build input i finalne
połączenie weapon GLB/material/texture z normalnym product builderem. Dopiero po
writer/package readback można oznaczyć F6 jako offline-complete.
