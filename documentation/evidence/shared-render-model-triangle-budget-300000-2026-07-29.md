# Wspólny budżet render-model: 300 000 trójkątów

Status: `IMPLEMENTED_OFFLINE_VERIFIED`

Data decyzji właściciela: `2026-07-29`

Ta decyzja zastępuje produktowy limit 20 000 z `2026-07-27`. Starsze pakiety i
raporty P20K/P100K/P300K pozostają prawdziwymi zapisami swoich historycznych
lineage, ale nie definiują już bieżącej granicy produktu.

## Kontrakt

```text
AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000
AURORA_MODEL_TRIANGLE_WARNING_ABOVE_V1 = 150_000
```

Ten sam budżet całego modelu obowiązuje Creature, Placeable, Tile i każdą
pozostałą trasę render-model:

- `0..=150 000`: accepted;
- `150 001..=300 000`: warning, nadal conversion eligible;
- `300 001+`: blocking;
- dokładnie `300 000`: accepted.

Meshy `target_polycount` pozostaje celem generacji, nie obietnicą dokładnego
wyniku. Pipeline zawsze mierzy faktyczną liczbę trójkątów.

## Granica binary MDL

Limit produktu nie zmienia ograniczenia pojedynczego strumienia binary MDL:

```text
65 535 index entries = 21 845 triangle-list faces per mesh stream
```

To granica jednego mesha, nie całego modelu. Wspólny segmenter dzieli wyłącznie
strumienie przekraczające tę granicę. Zachowuje:

- wszystkie trójkąty i ich kolejność;
- materiał i parent node;
- Skin/Rigid, wagi oraz joint IDs;
- pozycje, normalne, tangenty i UV;
- `castShadow` i opcjonalne surface IDs.

Segment już bezpieczny dla writera pozostaje byte-for-byte równy w common IR.
Segmentacja nie jest decymacją i nie może usuwać mikrotrójkątów.

## Miejsca egzekwowania

- GLB ingest: `GlbLimits::default()`;
- Profile A: `ProfileALimitsV1::default()`;
- gotowy common IR: `validate_model_triangle_budget_v1`;
- zapis Creature, Placeable i Tile:
  `segment_model_for_binary_mdl_v1` przed writerem.

Historyczne profile `EXPERIMENTAL_P100K` i `EXPERIMENTAL_P300K` pozostają tylko
do deterministycznego replay wcześniejszych lineage. Domyślny profil Studio to
`PRODUCT_300K`.

## Weryfikacja offline

Zielone testy graniczne potwierdzają:

- `300 000` accepted oraz `300 001` rejected;
- pełny model 300K dzielony na strumienie z najwyżej 65 535 indeksami;
- brak utraty trójkątów i metadata przy podziale;
- Tile z `21 846` faces zapisany jako dwa render meshe, następnie odczytany z
  binary MDL z sumą dokładnie `21 846`;
- Placeable pobiera te same progi GLB/Profile A i blokuje bezpośredni IR ponad
  budżetem.

Pełny release replay ścieżki `PRODUCT_300K` na kanonicznym źródle
`tlc-stoneback-brute-h1-p300k-v1` również jest zielony:

- wejście po sanitacji exact: `296 276` trójkątów;
- wyjście zapisane przez pipeline: `296 276` trójkątów;
- utrata geometrii: `0`;
- profil powierzchni i sanitacja używają tej samej reguły
  `ExactFiniteNonCollinear`, dzięki czemu zachowane zostają bezpośrednie wagi
  skina Meshy i pipeline nie uruchamia zbędnej projekcji odległościowej.

Granica dowodu pozostaje `offline_verified`. Ta zmiana nie tworzy nowego
kandydata MOD/HAK i nie stanowi nowego visual proof w Toolsecie lub NWN.
