# Borzoi V10 + `c_direwolf` — wynik nanoszenia offline

Data: 2026-09-02\
Status: `DIAGNOSTIC_BLOCKED`

## Zakres

Na kanoniczny model źródłowy został nałożony wybrany supermodel
`c_direwolf` przez publiczną granicę
`buildReferenceSupermodelAppliedPreviewV4`. Włączono jawne eksperymentalne
obejście limitu naprawy granic gałęzi skinningu. Nie wykonano eksportu ani
materializacji MOD/HAK.

## Tożsamość wejść i wyniku

- source GLB:
  `sample-3d/borzoi-c-wolf-bind-v6-p300k-v1/source.glb`
- source SHA-256:
  `3efd673f4ec953de568b2a30f6f14131a62828398f3133bb89855193b5fffd93`
- wybrany reference MDL:
  `local-reference-assets/neverwintervault-2026-07-18/extracted/nwn2-creature-conversion/models02/HellHound/c_direwolf.mdl`
- reference SHA-256:
  `121b63cd51ff46c3633c740951752110bebdeab7db139719ff6ff7db077f0fd9`
- wygenerowany preview MDL SHA-256:
  `2f8eb7ae458a39879aa847695a26e8986c4048e035b4e140e7ee11210190a6bc`
- katalog diagnostyki:
  `artifacts/diagnostics/borzoi-v10-immutable-c-direwolf-bind-v1`

## Fakty z walidacji

- exact structure: `PASS` — 30 carrierów;
- surface anatomy: `READY`;
- immutable joint fit: `READY`;
- skinning: `READY`;
- bind pose: `PASS`;
- aktywne kości ważone: 25/25;
- odziedziczone animacje: 42;
- lokalne animacje w wygenerowanym MDL: 0;
- joint × clip: 884/884;
- seam violations: 0;
- paw contact violations: 0;
- paw side violations: 0;
- naprawa granic gałęzi: 12 046 wierzchołków przy limicie 8 117;
- obejście limitu 5% zostało użyte i zapisane jako ostrzeżenie
  `EXPERIMENTAL_BRANCH_BOUNDARY_REPAIR_LIMIT_BYPASSED:12046:8117`;
- motion quality: `BLOCKED`;
- hard-limit edge samples: 107 094;
- triangle-area collapse: 5 413 przy dopuszczalnych 5 107;
- clip-start anchor jumps: 4;
- per-component deformation coverage: niezaliczone dla 42/42 klipów.

Centralny admission zatrzymał wynik kodami
`BLOCKED_MOTION_INCOMPATIBLE` i
`BLOCKED_DIAGNOSTIC_RUNTIME_READINESS`.

## Wniosek implementacyjny

Mechanizm wyboru dowolnego supermodelu działa: pipeline zachował dokładną
topologię, jointy i bind `c_direwolf`, przypisał pełne pokrycie kości oraz
odziedziczył 42 klipy. Checkbox zgodnie z kontraktem ominął wyłącznie limit
naprawy granic gałęzi; nie ominął późniejszej kontroli jakości ruchu.

Ten dokładny wynik nie może wejść do eksportu lub demo NWN. Kolejny krok musi
dotyczyć przyczyny deformacji tego mesha pod ruchem `c_direwolf`, a nie
poluzowania centralnego admission.
