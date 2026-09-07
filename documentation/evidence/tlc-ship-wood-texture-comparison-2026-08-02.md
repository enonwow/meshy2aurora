# Porównanie drewna — TLC ship vs wooden-sailboat-deck

Status: offline_reference_comparison_complete

## Referencja

- asset: `wooden-sailboat-deck-s2-textured-v1`;
- source SHA-256:
  `5085db9399eb7d30ddcb6c65b51748f4cabc5ed10cdf7ba912cdf2ef62e1dabe`;
- 1,971,346 poprawnych trójkątów po odrzuceniu czterech faktycznie
  zdegenerowanych źródłowych faces;
- cztery źródłowe mapy PBR;
- do właściwego porównania drewna wyłączono w Blenderze emission, normal,
  metallic i metallic-roughness, pozostawiając wyłącznie base color z
  roughness 0.8.

Pierwszy rozważany `concept-medieval-ship-s1-p100k-v1` został odrzucony jako
referencja, ponieważ jego GLB jest przestrzennym concept sheetem z wieloma
rozrzuconymi widokami, a nie jednym porównywalnym statkiem.

## Kandydat

- asset: `tlc-ship-under-construction-s1-p150k-v1`;
- source SHA-256:
  `61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278`;
- 152,574 trójkąty;
- aktywny Material Separation: Wood/Rope/Sail/Cloth/Metal;
- geometry cleanup: false;
- source UV0 preserved: true;
- aktywne drewno: source-atlas v3, ciemna ciepła krzywa plus lokalny high-pass
  przywracający wyłącznie istniejące w atlasie szczeliny i zużycie.

## Wynik wizualny

Tablica porównawcza:

`C:\Projects\meshy2aurora\artifacts\material-separation\ship-wood-comparison\reference-vs-target-v7.png`

SHA-256:
`f2559ff886e186416f9008de35ff082f236bbdb82ec641de4e54f6da281639f4`

| Pomiar renderu | Referencja base-color-only | TLC v6 | TLC v7 |
|---|---:|---:|---:|
| średnia luminancja | 53.79 | 60.01 | 65.31 |
| odchylenie luminancji | 24.37 | 16.10 | 26.45 |
| luminancja P10 | 20.91 | 43.73 | 32.12 |
| luminancja P90 | 86.60 | 75.93 | 100.18 |
| lokalny kontrast | 4.55 | 3.68 | 8.54 |

Metryki są wskaźnikowe, nie absolutne: referencję renderuje Blender Eevee, a
TLC diagnostyczny rasterizer pipeline'u. Oba rendery używają poprawnej
konwencji glTF UV, a referencja ma wyłączone dodatkowe PBR, więc porównanie
izoluje przede wszystkim czytelność base color.

## Wniosek

Słabość TLC v6 nie wynikała już z błędnego koloru średniego, lecz z za małej
rozpiętości tonalnej i zanikających lokalnych szczelin. Referencja czytelnie
oddziela każdą deskę przez ciemne przerwy, zróżnicowany odcień i brud na
krawędziach. V7 odzyskuje ten typ czytelności bez zmiany geometrii, UV i liczby
materiałów. Nie kopiuje tekstury referencji i nie generuje nowych desek; wzmacnia
wyłącznie istniejący detal źródłowego atlasu TLC.

Pozostała różnica jakości pochodzi głównie z samego assetu: referencja ma niemal
2 mln trójkątów i pełny zestaw PBR, a TLC 152,574 trójkąty i jedną źródłową
bitmapę base color. Material Separation może poprawić czytelność materiałów,
ale nie zastąpi brakującego detalu geometrii ani map PBR.
