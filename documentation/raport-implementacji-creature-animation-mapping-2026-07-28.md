# Raport implementacji: Creature Animation Mapping

Data: 2026-07-28
Branch/worktree: `animation` /
`C:\Projects\meshy2aurora\.worktrees\animation`

## Wynik

Etapy E0–E11 planu zostały zaimplementowane. Aplikacja ma rzeczywisty krok
`ANIMATION_MAPPING` w pipeline creature, wersjonowany dokument decyzji,
Base 42, fallbacki wymagające review, animacje custom, source/readback preview,
kanoniczną walidację core, addytywną lane V4 oraz zgodność
authoring → report/manifest → binary readback → Review.

Nie utworzono nowego MOD-a, HAK-a, modelu, resrefu ani wiersza 2DA. Nie
uruchamiano i nie przejmowano Aurora Toolset ani NWN. E11 zamknięto przez
ponowne użycie exact candidate r46: authored V4 odtworzył w pamięci bajtowo
identyczne MDL, TGA, `appearance.2da`, HAK i MOD, a odczyt z instalacji
właściciela ponownie potwierdził identyczne hashe. Nie utworzono r47.

## Zaimplementowany przepływ

1. Studio inwentaryzuje klipy GLB i tworzy deterministyczne propozycje.
2. Użytkownik rozstrzyga mapowanie 42 slotów, fallbacki i definicje custom.
3. Dokument V1 jest zapisywany z `sourceRevision` i `authoringRevision`.
4. UI oraz core walidują ten sam kontrakt; build jest dostępny wyłącznie dla
   statusu `READY` z fingerprintem zwróconym przez core/WASM. Lokalne
   `READY` pokazuje stan `VALIDATING WITH CORE` i nie odblokowuje builda.
5. Worker przekazuje dokument do WASM i
   `build_meshy_h1_model_package_v4`.
6. Core materializuje wyłącznie jawnie wybrane źródła; nie istnieje cichy
   fallback do idle.
7. Report i manifest zapisują fingerprint, provenance, Base 42, custom oraz
   conformance.
8. Studio porównuje dokument z binarnym readbackiem. Rozbieżność blokuje
   Review/Download, a zmiana dokumentu oznacza poprzedni build jako stale.
   Fingerprint report/manifest musi być identyczny z zamrożonym inputem.

## Najważniejsze kontrakty

- katalog: `contracts/creature-animation-catalog-v1.json`;
- authoring: `DIRECT_CREATURE_S_L_BASE_42_AUTHORING_V1`;
- worker lane: `H1_SKINNED_FULL_42_AUTHORED`;
- WASM: `validateCreatureAnimationAuthoringV1`,
  `resolveCreatureAnimationMappingV1`,
  `buildMeshyH1ModelPackageV4`;
- core: `validate_creature_animation_authoring_v1`,
  `resolve_creature_animation_mapping_v1`,
  `materialize_authored_direct_creature_clips_v1`,
  `evaluate_authored_animation_conformance_v1`;
- candidate-freezing boundary:
  `build_meshy_h1_model_package_v4_with_identity`.

Base 42 pozostaje osobnym kontraktem kompletności. Dodatkowe klipy custom są
addytywne. Dla `LOOPING_PHASED` nazwy wynikowe to `<name>_s`, `<name>` i
`<name>_e`; obecność custom nie osłabia ani nie blokuje behawioralnego gate’u
Base 42.

Kontrakt przewiduje `INHERITED_SUPERMODEL`, ale repo nie zawiera obecnie
wersjonowanego, kompatybilnego providera dla lane V4. Taka decyzja nie otrzyma
fałszywego `READY`: UI i core zwracają blocker
`M2A-ANIMATION-SUPERMODEL-PROVIDER-UNAVAILABLE`. Surowe ścieżki nadal są
odrzucane osobnym kodem. Obsługiwane, buildowalne źródła tej wersji to lokalne
klipy GLB, generator proceduralny i klipy custom z bieżącego GLB.

## Dowody offline

Na tym samym stanie drzewa przeszły:

- `npm test`: 48 plików, 243 testy;
- `npm run typecheck`;
- `npm run build`, łącznie z release buildem WASM;
- pełny `npm run test:worker-integration`: 3 pliki, 10/10 testów, w tym
  `animation-mapping-worker.integration.ts`;
- `cargo fmt --all --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace`;
- `cargo test -p m2a-core --test creature_animation_mapping`: 7/7;
- regresja behawioralna `Base 42 + custom`: 1/1;
- wybrane regresje V2 i V3: 2/2;
- natywna granica authored WASM: 1/1.
- exact lokalny audyt V4 → r46: 1/1; MDL, TGA, 2DA, HAK i MOD byte-identical.

Dedykowany test przeglądarkowy wykonuje realny przepływ
UI contract → Worker → zbudowany WASM → core → report/manifest → binary
readback. Sprawdza 42 bazowe klipy oraz dodatkowy custom `wave`.

## Canonical dane testowe w worktree

Pełne `cargo test --workspace` przechodzi bez kopiowania Git-ignored payloadów.
Wspólny, testowy resolver rozpoznaje zatwierdzony układ
`<canonical>\.worktrees\<branch>` i odczytuje `sample-3d`, `proof-output` oraz
`local-reference-assets` bezpośrednio z
`C:\Projects\meshy2aurora`.

Konfiguracja browser integration używa analogicznego aliasu do canonical
repository i udostępnia te pliki bezpośrednio przez Vite. Nie kopiuje GLB ani
2DA do `.generated`.

Nie utworzono drugiego asset rootu, junctionu ani kopii modelu. Produkcyjny
CLI diagnostyczny nadal fail-closed odrzuca worktree. Wyjątek istnieje tylko
pod `cfg(test)` dla dokładnie zagnieżdżonego układu canonical worktree, dzięki
czemu testy no-clobber i publishera wykonują się na rzeczywistych canonical
danych.

## Exact handoff E11

Authoring V4 dla dokładnego źródła i identity r46 otrzymał status core `READY`
oraz fingerprint:

```text
9e09be1e1307f96ae9a6b7e91a5cf6b33e6105d2ff3b1981df4ffe06d618affe
```

Odtworzone artefakty były bajtowo identyczne z immutable r46:

```text
m2a_h2r46.mod  b206f5507ce14406d62716f9e75c029044e9090f8ead1cdebfc2574e887b959c
m2a_h2r46.hak  027a9fc32fee9f75447c63f7308def35a1d098018d68d4d788d39c1174c7f98a
m2a_h2p46.mdl  d788f07137c7bf713f18654a14f0ce0559e46315cbb2558ea6dcb6fb7fc8d739
```

Te same hashe MOD i HAK zostały ponownie odczytane z natywnej instalacji
właściciela. Ponieważ nie zmienił się ani jeden bajt runtime, kandydat został
ponownie użyty; model iteration gate nie dopuszczał i nie wymagał r47.

Kompletny handoff zaczynający się od nazwy MOD:
`documentation/evidence/creature-animation-mapping-v4-r46-ready-for-owner-proof-2026-07-28.md`.
Maszynowy zapis zgodności:
`documentation/evidence/creature-animation-mapping-v4-r46-compatibility-2026-07-28.json`.

Wcześniejszy wynik właściciela pozostaje osobny od przygotowania agenta:
Toolset ma `modelVisibility=visible`, `proofCompleteness=missing`, a NWN
`modelVisibility=visible`, `proofCompleteness=verified`. Agent nie deklaruje
tego wyniku na podstawie preview lub readback.
