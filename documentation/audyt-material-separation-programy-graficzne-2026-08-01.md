# Audyt Material Separation w programach 3D

Data: 2026-08-01

Status: `AUDIT_COMPLETE / IMPLEMENTATION_NOT_STARTED`

Zakres: sposób rozdzielania jednego modelu na obszary materiałowe w Blenderze,
Maya, 3ds Max, Houdini, Substance 3D Painter i Unreal Engine oraz wnioski dla
local-first pipeline Meshy2Aurora. Audyt nie uruchamia nowej generacji Meshy,
nie tworzy nowego modelu ani proof candidate.

## 1. Werdykt

Branżowy wzorzec nie polega domyślnie na fizycznym cięciu modelu na osobne
obiekty. Najczęściej:

1. użytkownik lub algorytm tworzy grupę ścian/trójkątów;
2. grupa otrzymuje `Material ID`, material slot albo material binding;
3. renderer zapisuje grupy jako osobne sekcje/submeshe/draw calls;
4. narzędzie teksturujące tworzy osobny zestaw tekstur dla każdego materiału.

Fizyczne `Separate by Material` jest operacją dodatkową. Dla Meshy2Aurora
powinniśmy zachować jeden logiczny Placeable i przypisywać Material ID do grup
trójkątów bez przesuwania, usuwania, upraszczania lub ponownego generowania
geometrii.

Najlepsza nazwa trybu produktu: **Material Separation**. Najtrafniejsza nazwa
operacji wewnętrznej: **Material ID Assignment**.

## 2. Wspólny model pojęciowy

| Pojęcie | Znaczenie |
|---|---|
| Material slot | pozycja materiału dostępna dla obiektu/mesha |
| Material ID / material index | identyfikator zapisany na ścianie lub trójkącie |
| Face/primitive group | nazwany zbiór ścian używany do selekcji i operacji |
| Geometry subset / submesh / section | grupa trójkątów renderowana jednym materiałem |
| Texture set | komplet map generowany dla jednego materiału |
| Loose part / connected component | topologicznie rozłączny fragment geometrii |
| Material separation | workflow tworzenia grup materiałowych i przypisania ID |

Material ID nie jest tym samym co fizyczny osobny mesh. Jeden mesh może mieć
wiele ID i wiele materiałów.

## 3. Macierz programów

| Program | Sposób selekcji | Reprezentacja wyniku | Automatyzacja | Najważniejsza lekcja |
|---|---|---|---|---|
| Blender | wybrane faces, materiał, loose parts | material slots i material index per face; opcjonalnie osobne obiekty | manualna + topologiczna | rozdzielić przypisanie materiału od fizycznego `Separate` |
| 3ds Max | polygon/element selection | liczbowy Material ID + Multi/Sub-Object material | manualna, modyfikatory i skrypty | Material ID jest stabilnym kontraktem między geometrią i materiałem |
| Maya | obiekt albo polygon faces; także paint selection | shading groups przypisane do komponentów mesha | manualna i paint-assisted | kliknięcie/malowanie jest dobrym narzędziem korekty |
| Houdini | primitive groups, patterns, atrybuty, VEX | material path/attribute per primitive lub geometry subset | bardzo wysoka, proceduralna | grupa jest pierwszorzędnym, wersjonowalnym wejściem operacji |
| Substance 3D Painter | konsumuje Material IDs z importu | osobny Texture Set i layer stack per Material ID | automatyczne utworzenie texture sets, brak źródłowej segmentacji | rozdział musi powstać przed teksturowaniem |
| Unreal Engine | triangle selection i PolyGroups | wiele przypisań materiałów na triangulowanym meshu | manualna + generowanie/malowanie PolyGroups | podgląd, Accept/Cancel i Undo są częścią bezpiecznej operacji |

## 4. Fakty z programów

### 4.1. Blender

`EXTERNAL_TOOL_FACT`

Blender pozwala przypisać materiał ze slotu do zaznaczonych faces. Osobna
operacja `Mesh > Separate` może następnie rozdzielić geometrię według:

- aktualnego zaznaczenia;
- materiału;
- rozłącznych części (`By Loose Parts`).

Wniosek: materiał może być przypisany per face bez tworzenia nowego obiektu;
fizyczne rozdzielenie jest opcjonalnym etapem.

Źródło: [Blender Manual — Separate](https://docs.blender.org/manual/en/2.83/modeling/meshes/editing/mesh/separate.html).

### 4.2. Autodesk 3ds Max

`EXTERNAL_TOOL_FACT`

Editable Poly zapisuje liczbowy Material ID na wybranych polygonach. Materiał
`Multi/Sub-Object` mapuje te identyfikatory na submateriały. 3ds Max pozwala też
ponownie zaznaczyć polygony według ID lub nazwy submateriału. Autodesk podaje
przykład jednego samochodu z osobnymi materiałami lakieru, chromu i szyb.

To jest najbliższy odpowiednik funkcji potrzebnej w Meshy2Aurora.

Źródła:

- [Autodesk — Material ID](https://help.autodesk.com/cloudhelp/2024/ENU/3DSMax-Reference/files/GUID-D8EDE0E1-9694-4844-B58B-A8CB0EBD473B.htm);
- [Autodesk — Editable Poly Material IDs](https://help.autodesk.com/cloudhelp/2024/ENU/3DSMax-Modifiers/files/GUID-FF7D7633-03AD-4427-821A-65F8AC484CDD.htm);
- [Autodesk — Multi/Sub-Object Material](https://help.autodesk.com/cloudhelp/2022/ENU/3DSMax-Lighting-Shading/files/GUID-D968CDD9-4C5D-489D-A311-ED7486FCD4AA.htm).

### 4.3. Autodesk Maya

`EXTERNAL_TOOL_FACT`

Maya pozwala przypisać materiał do całego obiektu albo do wybranych polygon
faces. `Paint Assign Shader` używa narzędzia malowania selekcji, aby przypisywać
shader klikniętym lub malowanym obszarom powierzchni. Outliner pokazuje wiele
materiałów przypisanych do faces jednego obiektu.

Wniosek: dla trudnych granic potrzebny jest szybki mechanizm ręcznej korekty,
nie tylko jednorazowy automatyczny klasyfikator.

Źródło: [Autodesk Maya — Outliner material commands](https://help.autodesk.com/cloudhelp/2023/ENU/Maya-Basics/files/GUID-77994E64-9574-4E95-9540-B3CA881D96CD.htm).

### 4.4. Houdini

`EXTERNAL_TOOL_FACT`

Houdini traktuje grupy jako nazwane zbiory points albo faces. Material SOP
przypisuje materiał do grupy primitives i zapisuje ścieżkę materiału jako
atrybut. W Solaris `Assign Material` może programowo wyliczać binding per
element i automatycznie tworzyć geometry subsets dla elementów zwracających tę
samą ścieżkę materiału.

Wniosek: najlepszy kontrakt automatyzacji to `grupa -> materiał`, a nie
bezpośrednia mutacja obrazu tekstury.

Źródła:

- [SideFX — Groups](https://www.sidefx.com/docs/houdini/model/groups.html);
- [SideFX — Material SOP](https://www.sidefx.com/docs/houdini/nodes/sop/material.html);
- [SideFX — Assign Material](https://www.sidefx.com/docs/houdini/nodes/lop/assignmaterial.html).

### 4.5. Substance 3D Painter

`EXTERNAL_TOOL_FACT`

Painter nie rozwiązuje źródłowego problemu Material Separation. Przy imporcie
automatycznie tworzy osobny Texture Set dla każdego Material ID znalezionego w
modelu. Każdy texture set ma własny layer stack i ustawienia. Adobe oczekuje
unikalnych UV w obrębie material ID, z wyjątkiem logicznych overlapów, np.
geometrii lustrzanej.

Wniosek: Material IDs powinny powstać w naszym authoringu przed uruchomieniem
per-material texture override. Obecny model z jednym materiałem da Painterowi
tylko jeden texture set.

Źródła:

- [Adobe — Texture Set](https://experienceleague.adobe.com/en/docs/substance-3d-painter/using/interface/texture-set/texture-set);
- [Adobe — Project Creation](https://experienceleague.adobe.com/en/docs/substance-3d-painter/using/getting-started/project-creation);
- [Adobe — Texture Set List](https://experienceleague.adobe.com/en/docs/substance-3d-painter/using/interface/texture-set/texture-set-list).

### 4.6. Unreal Engine Modeling Mode

`EXTERNAL_TOOL_FACT`

Unreal działa na triangulowanej geometrii. PolyGroups są dowolnymi grupami
trójkątów, które można generować albo malować. Modeling Mode obejmuje wiele
przypisań materiałów, UV i texture baking. Operacje mają podgląd oraz jawne
`Accept`/`Cancel`; w trakcie działania dostępne jest Undo/Redo.

Wniosek: wynik automatyczny powinien najpierw być sugestią w podglądzie, a
dopiero po `Apply` wejść do wersjonowanego authoringu.

Źródło: [Epic — Modeling Mode](https://dev.epicgames.com/documentation/unreal-engine/modeling-mode-in-unreal-engine?lang=en-US).

## 5. Stan automatycznej segmentacji materiałowej

`RESEARCH_FACT`

W popularnych DCC dominują selekcja, grupy i ręczne przypisanie. Pełna
semantyczna segmentacja materiałów z jednego nieuporządkowanego modelu nadal
jest aktywnym tematem badawczym.

`SAMa` tworzy spójne między widokami maski materiałowe po kliknięciu materiału
i projektuje wyniki z wielu renderów na reprezentację 3D. Autorzy wskazują, że
rozkład assetu na materiały jest powszechny, ale nadal mocno manualny. Metoda
jest interaktywna, lecz wymaga wytrenowanego modelu oraz renderów wielowidokowych.

Źródło: [SAMa — Material-aware 3D Selection and Segmentation](https://arxiv.org/abs/2411.19322).

`Material Magic Wand` zakłada wstępny podział na części, często przez connected
components. Użytkownik klika jedną część, a system wyszukuje inne części, które
prawdopodobnie powinny mieć ten sam materiał. Próg podobieństwa działa jak
tolerance w klasycznej różdżce. Metoda dobrze odpowiada statkowi z wieloma
deskami, linami i podporami, ale sama nie rozcina ciągłej, zespawanej powierzchni.

Źródło: [Material Magic Wand](https://arxiv.org/abs/2603.17370).

## 6. Stan Meshy2Aurora

### 6.1. Istniejące fundamenty

`PROJECT_FACT`

- Core liczy connected components i ma deterministyczny split komponentów.
- Placeable Authoring obsługuje stabilne element IDs, wybór raycastem,
  Outliner, multi-select, isolate, hide, lock oraz pełne Undo/Redo.
- Autor może grupować rozłączne komponenty bez zmiany source GLB.
- Texture Authoring obsługuje niezależne `SOURCE/OVERRIDE` dla każdego
  istniejącego material slotu oraz exact TGA/HAK binding.
- Profile A i binary writer potrafią zachować wiele material slots i dzielić
  duże strumienie bez usuwania trójkątów.

Źródła projektowe:

- `documentation/implementacja-edytora-elementow-placeable-2026-07-26.md`;
- `documentation/audyt-edycji-tekstur-placeable-plan-implementacji-2026-08-01.md`.

### 6.2. Brakująca warstwa

`PROJECT_FACT`

- element authoring nie przechowuje przypisania element/component -> Material ID;
- nie można utworzyć nowego material slotu z grupy zaznaczonych komponentów;
- nie ma trybu kolorowego `Material ID overlay`;
- nie ma `Select by material`, `Assign`, `Unassign` ani `Apply/Cancel` dla sesji
  separation;
- source z jednym materialem nadal trafia do texture authoringu jako jeden slot;
- aplikacja celowo nie ma selekcji pojedynczych polygonów, więc welded surface
  bez rozłącznych części wymaga późniejszego rozszerzenia.

Wniosek: nie potrzebujemy nowego systemu tekstur ani drugiego edytora
geometrii. Potrzebujemy warstwy przypisania materiału pomiędzy istniejącym
authoringiem elementów a istniejącym resolverem tekstur.

## 7. Rekomendowany wzorzec dla produktu

`IMPLEMENTATION_RECOMMENDATION`

### 7.1. Tryb `Material Separation`

1. `Analyze parts` używa istniejących connected components.
2. Viewport pokazuje każdy kandydat kolorem ID, bez zmiany tekstury źródłowej.
3. Kliknięcie wybiera komponent; Shift dodaje, Alt odejmuje.
4. `Assign material` tworzy lub wybiera slot, np. `Wood`, `Canvas`, `Rope`,
   `Metal`.
5. `Select by material`, `Isolate` i `Unassigned only` umożliwiają kontrolę.
6. Zmiany pozostają w preview do `Apply`; `Cancel` odrzuca sesję.
7. Po Apply aktualizowany jest wersjonowany dokument authoringu i obecny panel
   tekstur pokazuje nowo utworzone sloty.

### 7.2. Automatyzacja warstwowa

Rekomendowana kolejność, od najbardziej deterministycznej:

1. **Connected components** — już dostępne, bez zgadywania.
2. **Select similar** — podobieństwo rozmiaru, proporcji, położenia, normalnych,
   source base color i UV coverage; użytkownik steruje tolerancją.
3. **Region grow** — dla zespawanych powierzchni, ograniczony przez kąt
   normalnych, krzywiznę, UV seams i różnicę koloru.
4. **Semantic labels** — przyszły opcjonalny model wielowidokowy, nigdy
   automatyczny nieodwracalny zapis.

Pierwsza wersja powinna być component-first i user-guided. Nie powinna obiecywać
bezbłędnego rozpoznania `wood/canvas/rope/metal` bez potwierdzenia użytkownika.

### 7.3. Reprezentacja wyniku

```text
source triangle
  -> stable source primitive/component identity
  -> authored Material ID
  -> deterministic material bucket / target primitive
  -> existing material slot binding
  -> existing SOURCE/OVERRIDE texture resolver
  -> MDL + TGA + HAK
```

Material Separation nie zmienia pozycji, indeksowanej powierzchni, UV,
normalnych, tangentów, triangle count ani PWK. Zmienia wyłącznie grupowanie
trójkątów do material buckets. Fizyczne utworzenie osobnych obiektów nie jest
potrzebne.

## 8. Ryzyka i ograniczenia

- Więcej materiałów oznacza więcej render sections/draw calls i potencjalnie
  więcej TGA w HAK; aplikacja musi raportować koszt przed Apply.
- Jeden source atlas może mieć UV islands różnych materiałów na tych samych
  pikselach. Osobne wynikowe tekstury per slot pozwalają to rozdzielić, lecz
  zwiększają rozmiar paczki.
- Connected components rozwiązują rozłączne deski, liny i podpory, ale nie
  rozdzielą kadłuba i pokładu, jeśli są topologicznie zespawane.
- Cienkie żagle i liny są podatne na błędy selekcji z jednego widoku; podgląd
  musi pozwalać obrócić model i kontrolować wynik z kilku stron.
- Podział materiałów nie naprawia złej geometrii ani złych UV.
- Automatyczne nazwy materiałów są sugestią UI; deterministyczna tożsamość
  opiera się na stabilnym ID i source hash, nie na nazwie `Wood`.

## 9. Konkluzja

Wzorzec do skopiowania nie pochodzi z jednego programu. Najlepsza kombinacja to:

- 3ds Max: Material ID jako kontrakt per polygon;
- Blender: osobne `By Loose Parts` i `By Material`;
- Houdini: grupy jako dane wejściowe proceduralnej operacji;
- Maya: paint-assisted korekta selekcji;
- Substance Painter: osobny Texture Set per material;
- Unreal: kolorowe PolyGroups, preview i jawne Apply/Cancel.

Dla Meshy2Aurora oznacza to tryb **Material Separation**, który buduje na już
istniejącym split connected components i kończy się Material ID Assignment.
To daje wiele niezależnych tekstur z jednego płatnego modelu Meshy, bez
dodatkowych generacji części.
