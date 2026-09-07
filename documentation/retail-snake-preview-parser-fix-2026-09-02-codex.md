# Retail snake preview parser fix — 2026-09-02

## Objaw i zakres

Widok istniejących modeli `c_cobra01`, `c_cobra02` i `c_cobra03` nie mógł
zbudować pełnego readbacku z lokalnego `nwn_base.key`/`data/xp3.bif`.
`c_cobra01` zatrzymywał się na `required uv0 pointer is null`, a warianty 02/03
dodatkowo na walidacji nieużywanych wpisów `skin.inlineMapping`.

## Dowody

- **Fakt z retail/resource/binary:** `c_cobra01` ma 51 węzłów. Brak UV0
  występuje na niewidocznych siatkach pomocniczych z `texture_count=0` i
  `render=0`; renderowane segmenty kobry mają UV0 i teksturę.
- **Fakt z retail/resource/binary:** w `cobra_mid` modeli `c_cobra02/03`
  aktywnych jest 14 kolejnych slotów kości (0..13). Profil `legacy17` przechowuje
  za nimi trzy nieużywane, niezerowane wartości i16: `27371, -16617, -10256`.
- **Fakt z dekompilacji Aurory:** tekstowe pola `tverts` są odczytywane
  warunkowo; brak strumienia nie jest globalnie wymagany dla każdej siatki.
- **Odrzucona hipoteza:** nie jest to uszkodzenie trzech renderowanych siatek
  ani brak animacji. Każdy zasób deklaruje 28 lokalnych animacji.

## Rozwiązanie

- **Wniosek implementacyjny:** parser wymaga UV0 tylko dla nie-AABB siatki z
  wierzchołkami i co najmniej jedną teksturą. Teksturowany model bez UV0 nadal
  kończy się `M2A-MDL-POINTER-OOB`.
- **Wniosek implementacyjny:** webowy projector zachowuje pełny zakres i16
  surowego `inlineMapping`; semantyczne użycie nadal ogranicza się do aktywnych
  slotów wyznaczonych przez `nodeToBoneMap`.

## Weryfikacja

- `cargo test -p m2a-core --test mdl`: 43/43 PASS.
- `vitest run src/features/results/projectReadback.test.ts`: 15/15 PASS.
- Readback w Studio: `c_cobra01`, `c_cobra02`, `c_cobra03` — każdy 51 węzłów,
  28 lokalnych animacji i pełny podgląd modelu, kości oraz jointów.

Pozostałe ryzyko: modele te mają `supermodel_name=NULL`; są samodzielnymi
modelami bazowymi z lokalnymi animacjami, a nie trzema potomkami wspólnego
supermodelu węża.
