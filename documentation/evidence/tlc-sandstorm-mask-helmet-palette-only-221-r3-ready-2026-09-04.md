# m2assmpal221r3.mod — R3 gotowy do testu właściciela

Moduł w Toolsecie: Meshy2Aurora Sandstorm Mask Palette 221 R3

Obszar: Sandstorm Mask Item Palette 221 R3

Data: 2026-09-04

## Co otworzyć

- dokładny plik MOD: m2assmpal221r3.mod
- dokładny SHA-256 MOD:
  9f8af9adf9067a86692a5f36f00a476e6a9e8438768b9fa76053ac34f28afe0a
- moduł wyświetlany w Toolsecie:
  Meshy2Aurora Sandstorm Mask Palette 221 R3
- dokładny obszar:
  Sandstorm Mask Item Palette 221 R3
- paleta:
  Custom → Armor → Helmets → Brązowa Maska Burzy Piaskowej R3
- Appearance:
  helm_221
- uporządkowane HAK-i:
  1. m2assmh221r3
  2. lc_2da

Moduł pozostaje palette-only: nie zawiera UTC ani stworzenia i nie umieszcza
żadnego stworzenia w GIT/GIC.

## Dokładna linia R3

- HAK: m2assmh221r3.hak
  - SHA-256:
    f369cb181dac72595391b42f483a0a54eaa5d9d073d1c131754a0e11d8a434f9
- MDL: helm_221.mdl
  - SHA-256:
    368bb50fa6cde0f3058a1011e1782f520eae810df120cff6f925b4f23fe42325
  - 100,000 trójkątów
  - 5 deterministycznych strumieni
- tekstura modelu: helm_221.plt, typ zasobu 6
  - SHA-256:
    d9505ef2be7290941017487fad46630c65d4fcf13e73925381813995006d232e
- ikona: ihelm_221.plt, typ zasobu 6, 64×64
  - SHA-256:
    0f51f502bc58a0ec18d263ccdba8aa528eb0051ad6d3cd3991bfd24e9ff3d7da
  - 2,403 piksele przezroczyste
  - pełna przezroczysta ramka
- UTI: m2assmaskpal221.uti
  - SHA-256:
    652710f4cc961a68a56339ab265077e811262530f0f0a47eeaa594396513cfdd
- źródło 100k:
  sample-3d/tlc-sandstorm-mask-meshy-0904135727-v1/source-p100k-r3.glb
  - SHA-256:
    b0e791d1606d81dccebd4f425bf60c18f2e8e0f51d22ff64fe8e75629bc437a1

## Poprawki do sprawdzenia

1. Obiekt po przeciągnięciu z palety nie powinien być srebrnym placeholderem.
2. Zmiana Metal 1 powinna zmieniać brązową część maski.
3. Zmiana Cloth 1 powinna zmieniać czarny kaptur i neutralne wykończenia.
4. Slot głowy i ekwipunek powinny pokazywać ikonę maski, nie czarny kwadrat.
5. Hełm powinien być większy i bliższy gabarytom helm_035.
6. Zaznaczanie/przesuwanie obiektu powinno być wyraźnie lżejsze niż przy R2.

Pozostałe cztery kanały koloru są prawidłowymi polami UTI, ale bieżąca
dwumateriałowa maska nie przypisuje do nich pikseli. To zamierzone.

## Skala

- jednostajna skala: 0.18652023
- R2: 0.14936666
- przyrost: około 24.9%
- R3 zajmuje 96% szerokości donora bdhd_items:helm_035
- granice R3:
  - min [-0.12119886, -0.13216108, -0.13546540]
  - max [0.12119886, 0.13876748, 0.21952541]
- niezależne skalowanie osi: nie zastosowano

## Instalacja natywna

Dokładne pliki zostały skopiowane wyłącznie do docelowych katalogów NWN po
sprawdzeniu nieobecności celu, a następnie zweryfikowane SHA-256:

- C:\Users\enonw\Documents\Neverwinter Nights\modules\m2assmpal221r3.mod
- C:\Users\enonw\Documents\Neverwinter Nights\hak\m2assmh221r3.hak

Oba cele były nieobecne przed kopiowaniem; oba są bajtowo identyczne ze
źródłami. lc_2da.hak pozostał bez zmian.

## Walidacja i granica dowodu

- GLB ingest: PASS
- MDL readback: PASS
- PLT modelu: PASS
- PLT ikony: PASS
- UTI readback: PASS
- HAK readback: PASS
- MOD readback: PASS
- pakiet nie zawiera TGA o nazwie helm_221 ani ihelm_221: PASS
- creature count: 0
- placed creature count: 0
- testy ukierunkowane PLT/ikona/pakiet: 13 PASS, 0 FAIL
- pełny m2a-core: 216 PASS, 2 FAIL, 3 ignored; dwa FAIL są zastanym,
  niezwiązanym stanem reference_supermodel_surface_anatomy (oczekiwania
  provenance/nazwy projekcji), a nie ścieżką Item/PLT
- modelVisibility: not_tested dla R3
- proofCompleteness: missing dla R3

Finalny test wizualny Toolset/NWN należy do właściciela. Agent nie uruchamiał,
nie przejmował i nie sterował Toolsetem ani NWN.

Audyt dekompilacji:
documentation/AURORA_DECOMP_HELMET_MODELTYPE1_AUDIT_2026-09-04.md
