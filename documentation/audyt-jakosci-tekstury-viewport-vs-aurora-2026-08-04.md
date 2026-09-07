# Audyt jakości tekstury: Studio viewport kontra Aurora Toolset

Data: 2026-08-04

Zakres: statek `tlc-ship-under-construction-s1-p150k-v1`, bez tworzenia nowej iteracji modelu.

## Wniosek

Słabszy wygląd statku w Aurora Toolset nie wynika z utraty połowy tekstury,
redukcji modelu ani automatycznego zmniejszenia TGA do 1024 px. Dla bezpośredniego
demonstratora Meshy `m2a_tlcs1` geometria, UV i tekstura przechodzą przez pipeline
z zachowaniem danych.

Najważniejsza rozbieżność jest po stronie podglądu Studio: bieżący ekran separacji
materiałów nie pokazuje wyniku Aurora. Pokazuje źródłowy GLB renderowany przez
Three.js, a w domyślnym trybie edycji nakłada na niego w 90% kryjące kolory
diagnostyczne Material ID. Jednocześnie Three.js używa jasnego nowoczesnego
oświetlenia i materiału PBR. Aurora renderuje wynikowy MDL/TGA przez własny,
klasyczny model oświetlenia. Porównywane obrazy nie są więc parytetowe, mimo że
pochodzą od tej samej geometrii źródłowej.

## Dokładna linia danych

- źródło: `sample-3d/tlc-ship-under-construction-s1-p150k-v1/source.glb`
- SHA-256 źródła: `61baad674f9509e9e7f1718aa415dc12411b099ffedaea6484f68ceb3bf10278`
- bezpośredni MOD: `m2a_tlcs1_mod.mod`
- HAK: `m2a_tlcs1_hak.hak`
- MDL: `m2a_tlcs1_mdl.mdl`
- TGA: `m2a_tlcs1_tex.tga`
- SHA-256 TGA: `92297351fa02b6731a7a340039ab41b3744424acfb7d195454d7843d74f209b8`
- Toolset: `The Last City - Ship Under Construction Demo`
- Area: `The Last City Shipyard Construction Demo`

Aktualny adres Studio dodaje do tego samego GLB późniejszą receptę
`recipe-detailed-v1/material-separation-v2.json`. Jest to inny stan materiałowy
niż bezpośredni eksport `m2a_tlcs1`.

## Ustalenia

### 1. Tekstura nie została zredukowana ani istotnie zdegradowana

Źródłowy GLB zawiera jeden JPEG 2048×2048. Wynikowy zasób to nieskompresowany
TGA RGB 2048×2048. Porównanie pikseli po dekodowaniu dało średnią bezwzględną
różnicę tylko:

- R: `0.268 / 255`
- G: `0.213 / 255`
- B: `0.349 / 255`

Orientacja bez odbicia pionowego była najbliższym i poprawnym wariantem. Tak mała
różnica pochodzi z odmiennych dekoderów/konwersji barw i nie tłumaczy widocznego
spadku jakości w Toolset.

### 2. Geometria i kanały potrzebne do teksturowania są obecne

- źródło: 152 574 trójkąty, 176 201 wierzchołków;
- wynik: 152 574 trójkąty w siedmiu strumieniach MDL;
- agresywne czyszczenie: wyłączone;
- usunięte trójkąty: 0;
- każdy z 176 222 wynikowych wierzchołków strumieni ma normalną i `uv0`;
- niewielki wzrost liczby wierzchołków wynika z deterministycznego podziału na
  strumienie mieszczące limit indeksów MDL.

Nie ma dowodu na zgubienie UV lub normalnych w tej linii.

### 3. Podgląd materiałów nie ładuje wynikowych tekstur

`MaterialSeparationDemo` pobiera:

- źródłowy GLB;
- receptę separacji;
- metadane `texture-authoring.json`.

Nie pobiera bajtów wskazanych plików PNG/TGA dla dwunastu materiałów. W
`MaterialSeparationEditor` mapa `textureFiles` startuje pusta, a viewportem jest
`SourceViewport`. Domyślny stan edytora to `selectionMode = FACE` i
`overlayEnabled = true`. Dla przypisanych ścian `SourceViewport` tworzy osobną
geometrię z `MeshBasicMaterial`, kolorem `previewColor` i opacity `0.9`.

To oznacza, że widoczne jaśniejsze drewno, liny, metal i żagle są przede wszystkim
diagnostycznym kodowaniem Material ID. Nie są odczytem tekstur, które zobaczy
Aurora. Sam komponent opisuje się jako `Original local GLB — viewport only, never
proof of Aurora output`.

### 4. Renderer Studio wzmacnia czytelność

Źródłowy GLB ma materiał:

- `baseColorFactor = [1, 1, 1, 1]`;
- `metallicFactor = 0`;
- `roughnessFactor = 0.8`;
- `doubleSided = true`;
- sampler: linear + trilinear mipmap.

Studio renderuje go jako glTF/PBR przez `GLTFLoader` i `MeshStandardMaterial`.
`SceneViewport` ustawia wyjście sRGB oraz bardzo jasne światło hemisferyczne o
intensywności `2` i kierunkowe o intensywności `2`. Dodatkowy overlay Material ID
jest nielitowany (`MeshBasicMaterial`), więc zachowuje nasycenie niezależnie od
oświetlenia sceny.

Aurora nie odtwarza modelu PBR glTF. MDL korzysta z klasycznej tekstury diffuse i
oświetlenia Toolset. Parametr roughness z glTF nie ma równoważnej semantyki w tym
eksporcie. Różnica światła i modelu materiału obniża lokalny kontrast drewna w
Toolset, nawet gdy wejściowe RGB jest prawie identyczne. Parytet gamma/przestrzeni
barw nie został jeszcze skalibrowany i pozostaje osobnym punktem do pomiaru.

### 5. Mipmapy i rozdzielczość nie są obecnie głównym podejrzanym

Dekompilacja lokalnego retail Toolset pokazuje:

- `FUN_00a6e364`: redukcja wymiarów następuje dopiero po przekroczeniu limitu
  tekstury zwróconego przez OpenGL;
- `FUN_00a9cf30`: domyślne flagi mipmap i filter są włączone;
- `FUN_00a6ee64` i `FUN_00a6edc8`: Toolset ustawia parametry filtrowania i ma
  ścieżkę anizotropii.

Nie znaleziono ustawienia w `nwtoolset.ini`, które dla tego użytkownika jawnie
obniżałoby jakość tekstur. Brak pliku TXI nie oznacza tutaj automatycznie braku
mipmap lub filtracji.

TGA 2048×2048 pozostaje poza największym rozmiarem TGA znalezionym w lokalnym
korpusie retail (1024×1024; DDS występuje do 2048×2048), ale loader nie wykazuje
automatycznej redukcji tego konkretnego rozmiaru. Jest to ryzyko kompatybilności
do osobnego testu, nie potwierdzona przyczyna obecnej różnicy.

### 6. Ograniczenie samego assetu

Cały statek o wymiarach około 15.17×8.75×6.29 m korzysta z jednego atlasu 2K i
wielu małych wysp UV. Tekstura ma stosunkowo niski kontrast: średnie RGB to około
`100/75/53`, a odchylenia standardowe około `20/18/16`. Stary renderer łatwiej
spłaszcza takie drewno niż jasny podgląd Studio.

To ograniczenie obniża docelowy sufit jakości, ale samo nie wyjaśnia różnicy
między rendererami. Oba renderery otrzymują tę samą informację bazową.

## Klasyfikacja przyczyn

| Priorytet | Ustalenie | Status |
|---|---|---|
| P0 | Studio pokazuje Source GLB i Material ID overlay, a nie wynik MDL/TGA | potwierdzone |
| P0 | Porównanie dotyczy różnych stanów materiałowych: szczegółowa recepta w Studio kontra bezpośredni `m2a_tlcs1` w Aurora | potwierdzone dla adresu Studio; screenshot Aurora nie zawiera belki tytułu |
| P1 | Three.js i Aurora używają innego oświetlenia i modelu materiału | potwierdzone; parytet gamma niezmierzony |
| P1 | Jeden mało kontrastowy atlas 2K ogranicza czytelność dużego assetu | potwierdzone |
| P2 | TGA 2K jest poza lokalnie potwierdzonym profilem retail | ryzyko, nie przyczyna |
| Odrzucone | pipeline zmniejszył TGA do 1024 | odrzucone |
| Odrzucone | agresywne czyszczenie usunęło geometrię | odrzucone |
| Odrzucone | tekstura została mocno skompresowana albo odwrócona | odrzucone |

## Co należy poprawić w pipeline i aplikacji

1. Dodać trzy jawnie rozdzielone widoki: `Source GLB`, `Material ID` i
   `Aurora Export`.
2. Po wygenerowaniu wyniku domyślnie otwierać `Aurora Export`, nie Source.
3. `Aurora Export` musi czytać dokładny binarny MDL po readback, jego prawdziwe
   `uv0` oraz dokładne wynikowe TGA/DDS. Nie może używać `previewColor`.
4. Rozszerzyć `AuroraReadbackViewport`: obecnie odtwarza geometrię, ale tworzy
   jednobarwny `MeshStandardMaterial` bez tekstury, więc również nie jest
   podglądem jakości tekstury.
5. Dodać profil renderowania zbliżony do Aurory: klasyczny diffuse/ambient,
   kontrolowane światło, brak PBR oraz jawny tryb gamma. Najpierw trzeba
   skalibrować go na małym fixture z wzornikiem kolorów w Toolset.
6. W raporcie wyniku zapisywać i pokazywać: hash źródłowego obrazu, hash TGA,
   wymiary, średnią/różnicę pikseli, liczbę UV i status parytetu tekstury.
7. Dodać porównanie A/B przy tej samej kamerze i skali: Source, Export Preview,
   Toolset. Zdjęcie Toolset musi zawierać nazwę dokładnego modułu/Area.
8. Dopiero po uruchomieniu parytetowego podglądu oceniać kolejne bake'i,
   separację materiałów, DDS/TXI lub zwiększanie kontrastu. Obecny podgląd nie
   pozwala rzetelnie zatwierdzić tych zmian.

## Kryterium poprawnego rozwiązania

Problem będzie rozwiązany po stronie aplikacji dopiero wtedy, gdy użytkownik
zobaczy w Studio dokładny wygenerowany MDL z dokładnymi wygenerowanymi
teksturami, bez overlayu diagnostycznego, a kontrolowany fixture wykaże zgodność
kierunku UV, przybliżonej jasności i filtrowania z Toolset. Source GLB i Material
ID mogą pozostać, ale muszą być jednoznacznie opisane jako podglądy pomocnicze,
nie jako wygląd wynikowy.

## Źródła lokalne

- `proof-output/tlc-ship-under-construction-placeable-v1-20260731/ready-for-owner-proof.json`
- `proof-output/tlc-ship-under-construction-placeable-v1-20260731/generated/placeable-report.json`
- `proof-output/tlc-ship-under-construction-placeable-v1-20260731/generated/geometry-readback.json`
- `apps/studio-web/src/features/material-separation/MaterialSeparationDemo.tsx`
- `apps/studio-web/src/features/material-separation/MaterialSeparationEditor.tsx`
- `apps/studio-web/src/features/material-separation/stateV2.ts`
- `apps/studio-web/src/features/preview/SourceViewport.tsx`
- `apps/studio-web/src/features/preview/SceneViewport.tsx`
- `apps/studio-web/src/features/preview/AuroraReadbackViewport.tsx`
- `documentation/konwersja-meshy-odpowiedz-codex.md`
- `C:/Projects/New Folder/export/decompiled_all.c` (Aurora First, tylko odczyt)
