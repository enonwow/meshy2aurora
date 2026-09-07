# Generyczne dopasowanie ruchu supermodelu — wynik r22–r24 (2026-09-01)

## Zakres i invariant

Zmiany dotyczą wspólnej trasy `reference_supermodel_motion`, a nie profilu
`c_wolf`. Algorytm wybiera support na podstawie hierarchii parent–child,
structural roles, fitted carrier topology i zmierzonych krawędzi. Nie zawiera
warunku po resrefie, nazwie kości, rasie ani gatunku. `c_wolf` jest wyłącznie
realnym, read-only przypadkiem walidacyjnym.

Dokładne wejście wszystkich przebiegów:

- `sample-3d/borzoi-c-wolf-bind-v6-p300k-v1/source.glb`;
- SHA-256 `3EFD673F4EC953DE568B2A30F6F14131A62828398F3133BB89855193B5FFFD93`;
- ta sama linia kandydata V10; nie utworzono V11, MOD ani HAK;
- nie uruchamiano Aurora Toolset ani NWN.

## Potwierdzone fakty

1. Wstępny skinning przechodzi audit gradientu wag z `0` naruszeń.
2. Pełna macierz joint×clip pozostaje `907/907`.
3. Względny ruch appendage/ogona pozostaje `38/38`, bez naruszeń.
4. Wszystkie przebiegi mają `0` seam violations.
5. Blokada pochodzi z lokalnych skrajnych deformacji krawędzi podczas ruchu,
   a nie z braku jointów, braku animacji albo zerwanego ogona.
6. Najgorsze krawędzie mają gładkie, trzyjointowe supporty jednego lineage;
   nie są brakującą kością ani trójkątem łączącym rodzeństwo w hierarchii.

## Zaimplementowana zmiana

- diagnostyka raportuje dokładną najgorszą krawędź collapse/expansion: klip,
  czas, segment, indeksy, endpointy bind/sample, ratio i wpływy jointów;
- katastroficzna faza naprawia jeden najwyższy peak na transakcję i po każdej
  zmianie ponownie mierzy pełny motion/joint/seam oracle;
- wprowadzono dwa generyczne targety: `ProjectedCarrierEdge` oraz
  `FullCarrierLineage`;
- `FullCarrierLineage` zachowuje średnią wag obu endpointów i zmniejsza lokalny
  gradient bez usuwania jointu z supportu;
- line search zmniejsza blend przy regresji innej osi, a przy płaskim wyniku
  zwiększa go do zamrożonego maksimum, zanim przejdzie do następnego targetu;
- wszystkie zmiany są transakcyjne; regresja powoduje rollback;
- corrected-bind ma ten sam fail-safe budżet 48 rund co istniejąca generyczna
  faza targeted. Zmiana 32→48 została dopuszczona dopiero po r23, który
  zakończył budżet w połowie policy sweep przy nadal poprawiającym się minimum.

## Wyniki

### r22 — target policy search

- raport:
  `artifacts/diagnostics/borzoi-v10-target-policy-search-r22/base-preview-report.json`;
- SHA-256 `937FE0AF4C4C9BC59C8F71CEA817BF5540BBFB1E19CC27E4ADC9185D73EBA4B7`;
- status `DIAGNOSTIC_BLOCKED`;
- najgorszy edge collapse ratio `0.0360935777`;
- znormalizowana severity collapse `1.73161`;
- hard edges `52 438`, collapsed triangles `1 049`, expanded triangles `306`.

### r23 — dwukierunkowy strength search, 32 targeted rounds

- raport:
  `artifacts/diagnostics/borzoi-v10-bidirectional-strength-search-r23/base-preview-report.json`;
- SHA-256 `48390CA176728D13020B8861FAABB023376838C63A3416060FBA5100573D7C45`;
- status `DIAGNOSTIC_BLOCKED`;
- najgorszy edge collapse ratio `0.0369407460`;
- znormalizowana severity collapse `1.6918987`;
- hard edges `52 648`, collapsed triangles `1 073`, expanded triangles `306`.

### r24 — ukończony 48-round fail-safe

- raport:
  `artifacts/diagnostics/borzoi-v10-complete-strength-search-r24/base-preview-report.json`;
- SHA-256 `03214FF9AAC8D7EEE5CD26D63282A8C62B7FF24164B1081DBC487CAEEC886D93`;
- status `DIAGNOSTIC_BLOCKED`;
- najgorszy edge collapse ratio `0.0380955003` przy wymaganym `>= 0.0625`;
- znormalizowana severity collapse `1.6406137`;
- hard edges `52 639`, collapsed triangles `1 146`, expanded triangles `306`;
- końcowy peak: `m2a_seg_7`, wierzchołki `3164–3149`, klip `cdamagel`,
  czas `0.4 s`;
- najgorsza expansion pozostała `15.0464239` na `m2a_seg_15:4134–4132` w
  `ckdbckdie` przy `0.616667 s`.

## Odrzucone hipotezy

- Sam target zachowujący średnią całego lineage nie wystarcza: r22 był
  metrycznie identyczny z wcześniejszym najlepszym wynikiem.
- Porządkowanie równych peaków nie rozwiązało blockera: r21 i r20 były
  identyczne.
- Samo zwiększanie liczby rund nie jest rozwiązaniem. r24 poprawił globalne
  minimum, ale nadal jest daleko od progu i zwiększył liczbę collapsed
  triangles. Dalsze podnoszenie limitu bez nowego solvera jest zabronione.

## Weryfikacja kodu

- `cargo test -p m2a-core reference_supermodel_motion --lib`:
  `34 passed, 0 failed`;
- pełne `cargo test -p m2a-core --lib`:
  `195 passed, 2 failed, 3 ignored`;
- dwa niezielone testy są w
  `reference_supermodel_surface_anatomy` i nie dotykają zmienianego modułu
  motion. Przy brudnym worktree ten przebieg nie dowodzi, czy są wcześniejszą
  awarią czy niezależną regresją; nie zostały zmienione w ramach tego zadania.

## Aktualny wniosek i następny krok

Implementacja pozostaje generyczna dla dowolnie wybranego supermodelu, ale nie
jest jeszcze pozytywnie potwierdzona na wielorodzinnym realnym corpusie i nie
ma prawa otrzymać statusu gotowego produktu.

Następna naprawa ma zastąpić iteracyjne uśrednianie endpointów lokalnym
solverem korzystającym z faktycznie zmierzonych transformacji najgorszej
krawędzi w jej klipie/czasie. Solver ma szukać wag w dozwolonym carrier lineage
i optymalizować minimalny edge ratio, zachowując pełny joint/appendage/seam
oracle. Nie wolno dalej zwiększać liczby rund ani poluzować progu `0.0625`.
