# Zakres produktu MVP

Data decyzji: 2026-07-29
Dotyczy: etap E5 planu aplikacji

## Widoczne targety MVP

- `Creature` — pełny przepływ Source → Inspect → Animation Mapping → Build →
  Review → Download.
- `Placeable` — pełny przepływ Source → Inspect → Build → Review → Download.
  Każdy build otrzymuje caller-owned project ID i rewizję. Core wyprowadza z
  nich jeden 16-znakowy namespace dla resrefów i nazw `.mod`/`.hak`, a raport
  oraz manifest pobrania zachowują dokładną identity i SHA.

Oba targety używają jednego budżetu produktu:

`AURORA_MODEL_TRIANGLE_BUDGET_V1 = 20_000`

Warning zaczyna się powyżej 10 000, dokładnie 20 000 jest dozwolone, a
20 001 jest blokowane.

## Tile

`Tile` nie wchodzi do pierwszego MVP. Pozostaje eksperymentalnym targetem
udostępnianym wyłącznie przez `VITE_TILE_TARGET=1`.

- bez flagi nie jest widoczny;
- z flagą jest jawnie oznaczony jako `Experimental`;
- istniejący WOK/SET/MDL pipeline i testy pozostają utrzymywane;
- owner proof Tile nie jest warunkiem wydania MVP Creature/Placeable;
- promocja do MVP wymaga osobnej decyzji, pełnego Review oraz human-owned
  owner proof.

## Meshy Lab

`Meshy Lab` jest osobnym, opcjonalnym narzędziem developersko-proofowym, a nie
częścią podstawowej dystrybucji MVP.

- pozostaje ukryty bez `VITE_MESHY_LAB=1`;
- po włączeniu UI oznacza go jako `Developer tool`;
- lokalny Bridge jest granicą zaufania;
- klucz Meshy API, pairing code i podpisane URL-e nie trafiają do projektu,
  Workera WASM ani pobieranego manifestu;
- do Source wraca wyłącznie jawnie zatwierdzony GLB i zredagowane provenance;
- obowiązuje ten sam budżet 20 000 trójkątów.

## Granica dowodowa

Browser E2E i binary readback są dowodem offline. Nie oznaczają widoczności w
Aurora Toolset/NWN. Przed wynikiem właściciela manifest zachowuje:

- `modelVisibility=not_tested`;
- `proofCompleteness=missing`;
- `ownerProof.status=PENDING_OWNER`.
