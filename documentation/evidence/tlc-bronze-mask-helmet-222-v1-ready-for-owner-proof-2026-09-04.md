# m2bronpal222.mod — hełm 222 gotowy do testu właściciela

Moduł w Toolsecie: Meshy2Aurora Bronze Mask Palette 222

Obszar: Bronze Mask Item Palette 222

Data: 2026-09-04

## Co otworzyć

- dokładny plik MOD: `m2bronpal222.mod`
- SHA-256 MOD:
  `0126dfbae0b3459c975b53daccd070d616e27ca974ffa261ef35758167611f4c`
- moduł wyświetlany w Toolsecie:
  `Meshy2Aurora Bronze Mask Palette 222`
- dokładny obszar:
  `Bronze Mask Item Palette 222`
- paleta:
  `Custom → Armor → Helmets → Brązowa Maska 222`
- Appearance: `helm_222`
- uporządkowane HAK-i:
  1. `m2bronh222v1`
  2. `lc_2da`

Moduł jest palette-only. Nie zawiera UTC, stworzenia ani umieszczonego
stworzenia.

## Dokładna linia zasobów

- HAK: `m2bronh222v1.hak`
  - SHA-256:
    `129d399e61ffcf0abac5bb17c58b82ea1e5f2edbba073cf4f88950b0f781b381`
- MDL: `helm_222.mdl`
  - SHA-256:
    `d6c722d089ae7a1e514b2755ff991c4468d4624e3b2f6ae9c49d112d0b4784d7`
  - 36 365 trójkątów
  - 2 deterministyczne strumienie binarnego MDL
- tekstura modelu: `helm_222.plt`, typ zasobu 6
  - SHA-256:
    `3d17f7fd181f9d681293c2b586758d51e7d969a2a4d70c78ac1acbd92a83b5e1`
  - `Metal 1`: 1 713 393 piksele
  - `Cloth 1`: 2 480 911 pikseli
- ikona: `ihelm_222.plt`, typ zasobu 6, 64×64
  - SHA-256:
    `131461a30a41aa2e45363471aa2b4658210af333cf0db105a779b0da23b86852`
  - 1932 piksele przezroczyste
  - pełna przezroczysta ramka
- UTI: `m2bronmask222.uti`
  - SHA-256:
    `3fe40159e4df12120dfe236f9db51b16b1359378addfbce4e2e30787296a6c9a`

## Źródło i jawne oczyszczenie

- oryginał właściciela: `bronze mask 3d model.glb`
  - SHA-256:
    `17ea0a4c777352885d81d311c7c6996bad2d3106e62c2f11d990bc3eecc429b5`
  - 36 367 trójkątów
- kanoniczny wariant kompilacyjny:
  `sample-3d/tlc-bronze-mask-tripo-20260904-v1/source-clean-v1.glb`
  - SHA-256:
    `1f032f9388fc7c94522669938a35855d4c2037d7321fd112730ed644e9c419d3`
  - usunięto wyłącznie 2 zeropowierzchniowe trójkąty
  - zachowano UV, teksturę 2048×2048 i zamierzoną sylwetkę

Pozostają jawne ostrzeżenia źródła: po spawaniu pozycji model ma jedną
powierzchnię, 1239 krawędzi otwartych i 103 krawędzie non-manifold. Nie są one
blokadą statycznego renderu, ale wymagają oceny wizualnej w Toolsecie i NWN.

## Skala

- donor: `bdhd_items:helm_035`
- zajętość szerokości donora: 96%
- jednostajna skala: `0.34307569`
- niezależne skalowanie osi: nie zastosowano
- granice po dopasowaniu:
  - min `[-0.12119886, -0.13096201, -0.12338276]`
  - max `[0.12119886, 0.13756841, 0.21952541]`

## Instalacja natywna

Cele nie istniały przed kopiowaniem. Po instalacji potwierdzono identyczny
SHA-256 źródła i celu:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2bronpal222.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2bronh222v1.hak`

`lc_2da.hak` pozostał bez zmian. Agent nie uruchamiał ani nie kontrolował
Toolsetu lub NWN.

## Co sprawdzić

1. Czy `Brązowa Maska 222` jest widoczna w palecie hełmów.
2. Czy model leżący w obszarze ma teksturę i daje się zaznaczać/przesuwać.
3. Czy `Metal 1` zmienia brązowe panele.
4. Czy `Cloth 1` zmienia czarny kaptur i wykończenia.
5. Czy ikona ekwipunku przedstawia maskę zamiast czarnego kwadratu.
6. Czy po założeniu rozmiar i położenie odpowiadają `helm_035`.
7. Czy otwory na oczy, perforacje i tył kaptura nie migoczą ani nie znikają.

## Granica dowodu

- GLB ingest: PASS
- source cleanup: PASS, dokładnie 2 usunięte trójkąty
- MDL semantic readback: PASS
- PLT modelu i ikony: PASS
- UTI readback: PASS
- HAK/MOD readback: PASS
- creature count: 0
- placed creature count: 0
- `modelVisibility=not_tested`
- `proofCompleteness=missing`

Finalny proof wizualny w Toolsecie i NWN wykonuje właściciel.
