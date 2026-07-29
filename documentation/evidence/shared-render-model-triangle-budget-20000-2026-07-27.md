# Wspolny budzet render-model: 20 000 triangles

> **SUPERSEDED 2026-07-29:** ten dokument jest historycznym zapisem decyzji z
> 2026-07-27. Aktualny limit produktu wynosi 300 000 trojkatow i jest opisany w
> [shared-render-model-triangle-budget-300000-2026-07-29.md](shared-render-model-triangle-budget-300000-2026-07-29.md).

Data: `2026-07-27`

Status: `IMPLEMENTED / TESTED OFFLINE`

## Decyzja

Creature i Placeable nie maja juz osobnych product guardrails geometrii.
Jedynym numerycznym source of truth jest:

```rust
AURORA_MODEL_TRIANGLE_BUDGET_V1 = 20_000
```

Warning jest wyliczany z tej samej stalej:

```rust
AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1 =
    AURORA_MODEL_TRIANGLE_BUDGET_V1 / 2; // 10_000
```

Semantyka granic:

| Triangle count | Wynik |
| ---: | --- |
| `0..=10 000` | bez geometry-budget warning |
| `10 001..=20 000` | warning, conversion eligible |
| `>20 000` | blocking `M2A-GLB-GEOMETRY-OVER-BUDGET` |

Dokladnie `20 000` jest akceptowane. `20 001` jest blokowane.

## Dlaczego poprzedni stan byl bledny

Przed ta decyzja istnialy dwa skompilowane profile:

- Creature: warning `>5 000`, blocking `>10 000`;
- Placeable: warning `>10 000`, blocking `>21 845`.

Byl to rozjazd polityki produktu, a nie wymog silnika. Placeable mial osobne
nadpisania w GLB admission i Profile A, mimo ze Creature i Placeable korzystaja
ze wspolnego `AuroraModelIrV1` i writera MDL.

## Implementacja

Wspolna stala znajduje sie w:

`crates/m2a-core/src/model_limits.rs`

Korzystaja z niej:

- `GlbLimits::default()`;
- `ProfileALimitsV1::default()`;
- Creature przez domyslny Profile A;
- Placeable przez domyslny budzet Profile A i GLB;
- trasy Tile, ktore korzystaja z tego samego static render-model admission.

Usunieto osobne stale
`PROFILE_A_PLACEABLE_TRIANGLE_WARNING_ABOVE_V1` i
`PROFILE_A_PLACEABLE_TRIANGLE_BLOCKING_ABOVE_V1`.

## Osobna granica formatu MDL

Writer nadal ma niezalezna granice jednego triangle-list mesh streamu:

```text
65 535 index entries / 3 = 21 845 triangles
```

To ograniczenie reprezentacji indeksow `u16`, nie budzet produktu. Wspolny
limit `20 000` pozostawia `1 845` triangles zapasu przed ta granica. Nie wolno
uzywac `21 845` jako alternatywnego limitu Placeable ani Creature.

## Testy kontraktowe

Testy wymagaja:

- warning na `10 001`;
- akceptacji exact `20 000`;
- blokady `20 001`;
- identycznych wartosci w Creature Profile A, Placeable Profile A oraz GLB;
- zachowania technicznej granicy writera `21 845` jako osobnego invariant.

Historyczne, zamrozone proof packets zachowuja capture-time profile i hashe.
Ta decyzja obowiazuje nowe konwersje; nie przepakowuje starych lineage.
