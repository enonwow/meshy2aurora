# Borzoi c_wolf V8 — bezpośrednia autoryzacja właściciela — 2026-08-21

Status: `ONE_EXACT_ITERATION_AUTHORIZED`

Właściciel po wdrożeniu wspólnego exact-bind pipeline polecił:

> naloz do testu ponownie na nasz model supermodel cwolf

Polecenie bezpośrednio zastępuje oczekiwanie na dalszy proof V7 i autoryzuje
jedną nową materializację testową V8. Agent nie interpretuje tego jako zgody na
dalsze V9 ani na samodzielne sterowanie Toolsetem/NWN.

## Zamrożony predecessor

- MOD: `m2aborzmod7.mod`, SHA-256
  `7be4c692cafcff70c349b6b4a0310f6450519b38f2a508943b15b50c678d90a2`;
- HAK: `m2aborzhak7.hak`, SHA-256
  `c2e17bd7fa072b4375e8be95a7f995f0674d53a6847e067aa56ac3bea51e05e7ff`;
- MDL: `m2aborzcre7.mdl`, SHA-256
  `6659dc523fc5b310a6a12a7359c1234eab57fc262648d370a895da44a71ef359`;
- `modelVisibility=visible`;
- wynik jakości zgłoszony przez właściciela: linie/artefakty materiału oraz brak
  oczekiwanego ruchu ogona mimo dziedziczenia animacji.

## Początkowo zakładana delta V8

- zachować source GLB SHA-256
  `f96be83949dcf06b3942d2c11ec746eb7aa45ded1cbfeb5e8525190c50475dda`;
- zachować 300 000 trójkątów, recipe rig/weights V5 i minimalny two-sided MTR
  z V7;
- zbudować kontrakt i direct carrier z exact neutral local matrices z
  immutable inspekcji retail `c_wolf`;
- wymagać `referenceBindVerified=true`, zerowego
  `referenceBindMaxAbsError` w tolerancji oraz kompletnego ważonego
  `tail_tip`;
- nowe resrefy: `m2aborzcre8`, `m2aborztex8`, `m2aborzhak8`,
  `m2aborzmod8`;
- visual proof wykonuje właściciel.

## Amendament po blokadzie offline

Pierwsza materializacja nie utworzyła żadnego pliku: exact retail bind z
zachowanymi wagami V5 przekroczył bramki krawędzi, trójkątów i szwów. Tak samo
został odrzucony exact carrier z korekcyjnymi dziećmi. Były to lane/build
failures tej samej V8, nie kolejne iteracje.

Zamrożona, udana V8 zachowuje dokładnie zweryfikowane nazwy, hierarchię,
inventory klipów/kanałów i supermodel `c_wolf`, ale pozostawia neutralne pivoty
anatomiczne targetu. Strategia ma identyfikator
`VERIFIED_RETARGETED_TARGET_BIND_V4`. Wagi i rig source V5 pozostają zachowane,
a bramka V4 dodatkowo wymaga ważonego `tail_tip`. Uzasadnienie i hashe znajdują
się w
`documentation/evidence/borzoi-c-wolf-demo-v8-ready-for-owner-proof-2026-08-21.md`.
