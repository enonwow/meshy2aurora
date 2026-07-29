# Audyt kompletności celu E0–E7

Data: 2026-07-29  
Branch/worktree: `animation` /
`C:\Projects\meshy2aurora\.worktrees\animation`  
Wynik: `E0_E6_COMPLETE_E7_BLOCKED_AT_EXACT_V5_RELEASE_GATE`

## Zakres i metoda

Sprawdzono literalne checkboxy, Definition of Done i kryterium gotowości z:

`documentation/audyt-aplikacji-i-plan-etapowy-2026-07-28.md`.

Zielony test uznano za dowód tylko wtedy, gdy obejmuje dokładny wymagany
przepływ. Historycznego proofu r46 nie przypisano do nowego V5. Technicznych
wersji `0.1.0` z `package.json`/`Cargo.toml` nie uznano za decyzję wydaniową;
repo nie ma taga release.

## Macierz etapów

| Etap | Stan | Dowód | Otwarte wymagania |
| --- | --- | --- | --- |
| E0 | COMPLETE | jeden limit 20k, granice Core/Studio/Bridge, pełne gate’y | brak |
| E1 | COMPLETE | wspólny shell, poprawne kroki, responsive/focus/modals | brak |
| E2 | COMPLETE | project V1, SHA rebind, export/import/recovery IndexedDB | brak |
| E3 | COMPLETE_MVP | Base 42, Custom, authoring, donor import, stable references | retarget, FBX, skin weights i N1 są jawnie P2 poza MVP |
| E4 | COMPLETE | typed Build, Review evidence, exact revision Download | brak |
| E5 | COMPLETE_MVP | Creature/Placeable; Tile i Meshy Lab jawnie eksperymentalne | brak dla MVP |
| E6 | COMPLETE | kontrolery, moduły WASM, CSS split, lazy chunks, budgets, race/a11y | brak |
| E7 | PARTIAL_RELEASE_GATE | owner-asset E2E, manifest, r46 handoff, owner result, user guide | packaged donor V5, exact V5 candidate, final release version |

## E7 — dowody pozytywne

### Realne owner-asset E2E

- H2 Creature:
  `f8cf0af21c8143a62b64c490a81dd2855ad3c3f9865922e3854f84b714dec3a3`;
  42 animacje; dwa buildy dały identyczne SHA sześciu artefaktów.
- H1 donor:
  `3071664994aec7d71f8a6fb8808587161dab9e09816f1a78b8562380e967485f`;
  exact-rig import, 48 tracków, preview, pełny mapping V2 i dwie identyczne
  materializacje JSON.
- Placeable:
  `dad22a5c3490242458cb7a81e50c53e265abf75886f57f6bd8a770938c2f7372`;
  project revision 1; dwa buildy dały identyczne SHA czterech artefaktów.

Oficjalny gate:

```text
npm run test:worker-integration
Worker/WASM 9 files / 23 tests PASS
IndexedDB 2 files / 2 tests PASS
```

Maszynowy zapis:
`documentation/evidence/e7-release-evidence-manifest-2026-07-29.json`.

### Exact istniejący candidate i owner proof

R46 jest immutable, ponownie audytowalny i już zainstalowany byte-identycznie:

```text
MOD m2a_h2r46.mod
SHA-256 b206f5507ce14406d62716f9e75c029044e9090f8ead1cdebfc2574e887b959c

HAK m2a_h2r46.hak
SHA-256 027a9fc32fee9f75447c63f7308def35a1d098018d68d4d788d39c1174c7f98a
```

Wcześniejszy wynik właściciela jest zapisany osobno:

```text
Toolset modelVisibility=visible, proofCompleteness=missing
NWN modelVisibility=visible, proofCompleteness=verified
```

Handoff:
`documentation/evidence/e7-owner-proof-handoff-2026-07-29.md`.

## Niespełnione wymagania

### 1. Packaged Creature z klipem dawcy

Stan:

```text
READY_MATERIALIZED_NO_PACKAGE
modelVisibility=not_tested
proofCompleteness=missing
```

Źródło, projekt rewizji 3, fingerprint Animation Studio, fingerprint mappingu
V2 i SHA materializacji są zapisane. Nie istnieją jednak exact MOD/HAK tego
donor V5, więc minimum E7 nie jest zamknięte jako wynik packaged Creature.

### 2. Exact candidate Animation Studio V5

Nie utworzono MOD, HAK, resrefów ani nowej proof lineage. Aktualny r46 jest
widoczny według wyniku właściciela, więc nie istnieje świeży exact
`modelVisibility=not_visible`, który dopuściłby iterację.

R46 poprzedza obecny project lifecycle i nie ma project revision. Pełnego
wymogu „source SHA + project revision + mapping fingerprint + SHA wszystkich
artefaktów” nie wolno zamknąć przez dopisanie fikcyjnej rewizji. Dopuszczony
V5 musi powstać już z caller-owned project identity.

Wznowienie wymaga jednego z:

1. bezpośredniej decyzji właściciela dopuszczającej utworzenie jednego
   dokładnego kandydata Animation Studio V5; albo
2. świeżego, exact candidate-bound wyniku
   `modelVisibility=not_visible`.

Ogólne „kontynuuj” nie spełnia tego warunku.

### 3. Wersja wydania

Repo nie ma taga wydania. Istniejące `0.1.0` jest techniczną wersją manifestów
pakietów, nie decyzją o wydaniu produktu. Wersja release pozostaje celowo
nienadana do czasu:

- dopuszczonego exact V5;
- wymaganej instalacji exact MOD/HAK;
- wyniku owner proof dla tej lineage;
- końcowego zielonego CI na wersjonowanym, commitowanym stanie.

## Werdykt

Nie wolno oznaczyć całego celu E0–E7 jako complete. Wszystkie istniejące
artefakty są odtwarzalne z manifestu i wszystkie działania niezależne od
owner/model-iteration gate są wykonane. Dalsza implementacja exact V5 bez
jednego z jawnych warunków wznowienia naruszyłaby aktywną regułę projektu.

Agent nie uruchamiał ani nie sterował Aurora Toolset/NWN.
