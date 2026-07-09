# konwersja-meshy-pytania-cloud.md

Status 2026-07-09: ZAMKNIETE z korekta D7-D8. Odpowiedzi/pomiary z `aurora-web` sa reference-only; aktywne rozstrzygniecia maja trafic do standalone test gates.
Data: 2026-07-08 | Status: ZAMKNIĘTE (odpowiedź: konwersja-meshy-odpowiedz-codex.md; Q1/Q2 = NIE WIEM → rozstrzygną fixtury s3, Q7 wariant single-skin = osobny eksperyment) | Priorytet: WYSOKIE

## Q1: Oś forward modelu Aurora
W którą stronę "patrzy" creature w przestrzeni MDL (po Z-up): +Y, -Y, +X? Kotwica z dekompilacji lub z transformacji orientacji w aurora-web (bearing/yaw creature). Potrzebne do rotacji importu z glTF (+Z forward).

## Q2: Jednostki Aurory
Czy 1 jednostka MDL = 1 metr w świecie gry? Kotwica (np. PERSPACE/HEIGHT w 2da, rozmiary tile 10x10 jednostek?). Potrzebne do skalowania z metrów meshy.

## Q3: Budżet geometrii creature
Realne liczby trójkątów/wierzchołków retail creature (zmierz 2–3 modele z mirrora: c_kocrachn + większy potwór) oraz czy aurora-web/derived v13 ma praktyczny limit. Rekomendowany target decymacji dla m2a_*.

## Q4: Smoothing groups i normalne
Jak dokładnie ASCII MDL koduje smoothing group w faces i jak engine liczy normalne? Czy konwerter v13 czyta smoothing groups, czy używa normalnych z innego źródła? Co emitować dla siatki meshy (jedna grupa vs rozbicie po kątach)?

## Q5: Tekstury — limity
Maksymalny/typowy rozmiar tekstury TGA w retail (kotwica z zasobów) i czy aurora-web ma limit. Czy TGA musi być bez kompresji RLE? Bity alpha?

## Q6: Orientacja UV
Czy tverts w MDL mają V rosnące w górę czy w dół względem glTF (czy potrzebny flip V przy przepisywaniu TEXCOORD_0 → tverts)? Kotwica z konwertera v13 (jak czyta tverts → UV w GLB).

## Q7: Segmentacja skin nodes
c_kocrachn ma wiele skin nodes z ograniczonymi listami kości (influencingBoneNames). Czy engine/format wymaga takiego podziału (np. limit kości per skin node), czy pojedynczy skin node z pełną listą kości też jest legalny? Kotwica. Decyduje o s6.
