# engine-mdl-pytania-cloud.md
Data: 2026-07-08 | Status: OTWARTE PO KOREKCIE 2026-07-09 | Priorytet: BLOKUJĄCE dla binary MDL/MDX writer i zawartości HAK
Kontekst: audyt-stanu-2026-07-08-cloud.md, sekcja 2.

## Q1: Binary MDL jako format docelowy
Korekta Mateusza 2026-07-09: nie badamy ASCII MDL jako runtime shortcut. Aktywny pipeline to `model z Meshy -> nasz parser/konwerter -> natywny MDL + 2DA + HAK dla Aurory/NWN`. ASCII MDL może istnieć co najwyżej jako debug dump/golden snapshot, nie jako docelowy format proofu.

Rozstrzygnij pod binary writer: jaki minimalny binarny MDL musi wyemitować `meshy2aurora` dla direct creature z geometrią, skin weights i animacjami? Potwierdź pola nagłówka, node tree, controllery, mesh/skin arrays, animacje i offsety/pointery potrzebne writerowi. Format odpowiedzi: YAML z polami `required_for_writer`, `can_defer`, `unknown`.

## Q2: MDX — osobny zasób czy embedded
Binary c_kocrachn ma MDX embedded/pointed przez `p_start_mdx`/`size_mdx` w nagłówku, ale stare runbooki wymieniały osobny zasób MDX (2003). Rozstrzygnij dla natywnego outputu `meshy2aurora`: kiedy HAK ma zawierać tylko zasób MDL (2002) z danymi MDX w payloadzie, a kiedy osobny zasób MDX (2003)? Popraw expected_resources dla M3.

## Q3: Podgląd nwnexplorer a bind pose
Czy podgląd modelu w nwnexplorer pokazuje bind pose (geometrię spoczynkową), czy odtwarza animację (np. cpause1)? Jeśli animację — wskaż sposób na screenshot czystej bind pose (opcja w nwnexplorer / toolset / inny sposób). Wpływa na jakość referencji `sample-2d/_reference/`.
