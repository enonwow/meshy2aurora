# Audyt kursow Claude AI for Blender dla `meshy2aurora`

Data: 2026-07-13 | Autor: Codex | Status: informacyjny, future backlog only

## Wniosek

Pakiet kursow moze podniesc wartosc kolejnych funkcji `meshy2aurora`, ale nie
jest zrodlem kontraktu Aurora ani nie odblokowuje aktywnego milestone'u. W
momencie audytu projekt mial strukturalnie gotowy reader/writer MDL, Profile A
i mapping animacji, a aktywny zakres obejmowal natywne TGA,
preserve-and-append `appearance.2da` oraz HAK. Pierwszy gate pozostaje
niezmienny:

```text
GLB -> own converter -> binary MDL + 2DA + HAK -> NWN EE Toolset/game proof
```

Kursy sa materialem do nauki authoringu w Blenderze i do projektowania
oryginalnych workflow. Nie sa dowodem runtime Aurora, licencja na ich project
files ani powodem do kopiowania ich payloadow do repo.

## Mapa kursow do backlogu

| Temat kursu | Przyszla wartosc | Docelowy obszar | Granica |
|---|---|---|---|
| PBR Material Authoring; Blender Scene Building and Rendering | Wysoka | F5 Material Variants, F6 Alpha and Emissive Inspector, przyszly controlled texture-bake contract | M5 emituje obecnie tylko TGA z gotowych RGB/RGBA; bez PBR bake, TXI i bez dowodu runtime przed M6. |
| Generate Asset Libraries | Wysoka | F5 variants, manifest zasobow i przyszly batch import | Biblioteka ma skladac sie z wlasnych GLB/TGA/recipe; nie z kursowych project files. |
| AI Box Modeling; Primitive Modeling; Hard-Surface; Boolean/Non-Destructive | Srednia | F3 Model Health, F7 Geometry Inspector, upstream Blender authoring | To poprawia jakosc wejscia GLB, ale Studio nie staje sie modelerem ani nie wykonuje automatycznej decymacji. |
| Rigging and Animation with Blender in Claude Code | Bardzo wysoka | F8 Skeleton and Weight Inspector, F9 Animation Inspector, przyszle preflight GLB | M4A jest strukturalnie gotowe, ale mapping i zachowanie animacji musza przejsc Toolset/game proof w M6. |
| Environment Modeling with Blender using Claude Code | Wysoka po M6 | nowy, osobny profil static-prop/environment oraz batch conversion | Nie wolno traktowac go jako rozszerzenia creature Profile A bez osobnego kontraktu, fixture i NWN proofu. |
| Low-Poly Stylization; Digital Sculpting; Portraits; Vehicle Modeling | Srednia | standardy authoringu i test corpus wejsc GLB | Pomaga tworzyc reprezentatywne, oryginalne assety M7, nie zmienia formatow MDL/2DA/HAK. |
| Advanced Lighting | Niska/srednia | F4 Reference Compare i ewentualny bake contract | Oswietlenie viewportu nie jest proofem renderingu NWN; wartosc jest glownie w kontrolowanych referencjach tekstur. |

## Najbardziej obiecujace przyszle funkcje

1. **Blender preflight companion**: oryginalny skrypt/add-on eksportujacy GLB
   wraz z raportem osi, skali, UV, trojkatow, materialow, wag i klipow.
   Nie pisze MDL i nie omija walidatora `meshy2aurora`.
2. **Static-prop/environment profile**: oddzielny od creature Profile A
   converter dla nierigowanych propsow i fragmentow otoczenia, poprzedzony
   proofem na generated HAK/module.
3. **Recipe-based asset library**: deklaratywne, wersjonowane recipes dla
   wariantow materialowych i batch conversion, z unikalnymi resrefami oraz
   manifestem HAK.
4. **Animation preflight**: mapa `source clip -> Aurora animation`, kontrola
   bind pose/wag i raport odrzucajacy niezgodne klipy przed writerem.

## Kolejnosc

Nie rozpoczynac tych funkcji w M5. Po M6 najbardziej uzasadniona kolejnosc to:

```text
F1-F4 diagnostics -> F8/F9 rig and animation inspection -> Blender preflight
companion -> static-prop/environment profile -> recipes and batch library
```

Kursy warto przerabiac juz teraz po stronie authoringu, zaczynajac od:
`AI Box Modeling`, `PBR Material Authoring`, `Generate Asset Libraries`,
`Rigging and Animation` oraz `Environment Modeling`. Kazdy wniosek techniczny
z kursu wymaga nastepnie wlasnego testu/fixture i proofu w NWN EE.
