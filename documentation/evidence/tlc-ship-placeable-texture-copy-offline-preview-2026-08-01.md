# TLC Ship Placeable — offline texture-copy preview

Data: 2026-08-01

Status: `OFFLINE_PREVIEW_VERIFIED / NOT_A_PROOF_CANDIDATE / OWNER_PROOF_UNCHANGED`

## Zakres

Sprawdzenie produkcyjnego vertical slice'u tekstur Placeable na realnym,
wybranym przez właściciela modelu statku. Przebieg wykonał byte-identical kopię
GLB w pamięci przeglądarki, pozostawił źródło read-only, zastąpił teksturę
jednego material slotu przez Studio -> Worker -> WASM/Core resolver i wykonał
porównawcze zrzuty `Source` oraz `Edited`.

Nie utworzono ani nie zamrożono nowego MOD/HAK/MDL/UTP. Nie uruchamiano Aurora
Toolset ani NWN. Istniejący `tlc-ship-under-construction-s1-p150k-v1` zachowuje
`owner_visual_result: not_tested`; ten przebieg nie jest nową iteracją jego
lineage proof.

## Tożsamość wejścia i kopii

- asset id: `tlc-ship-under-construction-s1-p150k-v1`;
- kanoniczne źródło:
  `sample-3d/tlc-ship-under-construction-s1-p150k-v1/source.glb`;
- source byte length: `11 498 076`;
- source SHA-256 przed i po przebiegu:
  `61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278`;
- nazwa byte-identical kopii przekazanej do Studio:
  `tlc-ship-under-construction-texture-copy.glb`;
- triangles z manifestu: `152 574`;
- material slots: `1`;
- UV: `UV0 present`;
- aggressive geometry cleanup: domyślnie wyłączony.

## Override i wynik resolvera

- wejście override: `painted-teal-wood.png`, fully opaque PNG;
- override byte length: `665 403`;
- override SHA-256:
  `3aaa307235971fe437f910188f1f64d9eaea2afe51f264dd12d7b67bbbac3bc7`;
- resolved authoring SHA-256:
  `4b93b0fb077b39f17eb85d68abdb29af68d6c5674f0a0737b1febd5562510027`;
- resolved TGA resref: `pte244f898`;
- resolved TGA SHA-256:
  `07a0d615d3434a1656929633f4917cb07b21287aa6622b7b903d409a9b8a50c4`;
- Worker response: `PLACEABLE_TEXTURES_RESOLVED`;
- console errors: `0`.

## Artefakty lokalne

Katalog: `output/playwright/ship-texture-copy/`

- `original-source.png` — SHA-256
  `4acb6256d223c74f93aa00335e945fc2da7c1c8a40c93fe327fa84ec755c3073`;
- `edited-teal-wood.png` — SHA-256
  `7f8cd1a196e813d3251de1d78d88fb41de7089a6377f32dfa26eca0664bca8cd`;
- `edited-pipeline-panel.png` — SHA-256
  `724c08c764b5521998e6533f6a30280334323f8ad5bf348397f2a233f930500e`;
- `pipeline-state.json` — maszynowy zapis wejścia, recipe i wyniku resolvera.

Oba kadry używają tej samej geometrii, orientacji i zoomu. `Show PWK` jest
wyłączone, a główny preview pozostaje w trybie `Original`, dzięki czemu różnicą
między kadrami jest wyłącznie texture preview `Source` versus `Edited`.

## Werdykt

Kopia modelu może otrzymać nową, nieprzezroczystą teksturę bez modyfikacji GLB,
UV0 ani geometrii. Resolver wyprodukował deterministyczną tożsamość TGA i
aplikacja pokazała zmianę na realnym modelu. Jest to dowód offline działania
edytora/resolvera, nie wizualny proof Aurory/NWN i nie autoryzuje nowego
proof-candidate lineage.
