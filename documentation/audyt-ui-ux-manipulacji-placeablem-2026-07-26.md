# Audyt UI/UX manipulacji placeablem

Data: 2026-07-26  
Repozytorium: `C:\Projects\meshy2aurora`  
Zakres: aplikacja `apps/studio-web`, wspólny parser/IR, worker/WASM oraz
pipeline statycznego placeabla.

## 1. Werdykt

> **Stan wdrożenia 2026-07-26:** kompletny edytor elementów z rozszerzonym
> kontraktem authoringu został zaimplementowany. Dokładna, odhaczona macierz
> funkcji, pipeline’u i testów znajduje się w
> [implementacja-edytora-elementow-placeable-2026-07-26.md](implementacja-edytora-elementow-placeable-2026-07-26.md).
> Starsze checkboxy poniżej opisują pierwotny wariant audytu całego modelu,
> włącznie z pomysłami później odrzuconymi przez właściciela (np. sylwetka
> człowieka), i nie są rejestrem nowej implementacji elementowej.

Aplikacja ma większość technicznych elementów potrzebnych do dobrego edytora:

- viewport Three.js;
- `OrbitControls`;
- siatkę, osie, bounds i wireframe;
- odczyt bounds źródłowego GLB;
- wspólny IR i deterministyczny pipeline MDL;
- generowanie oraz odczyt PWK;
- rewizje chroniące przed przyjęciem starego wyniku builda.

Nie ma jednak właściwej warstwy authoringu placeabla. Obecny viewport służy do
oglądania, nie do modyfikowania. Transformacja nie jest częścią stanu projektu,
requestu workera, API WASM ani raportu pipeline'u.

Rekomendowany produkt to nie uproszczony Blender. Powinien to być wąski,
bezpieczny warsztat przygotowania statycznego modelu do Aurory:

1. ustawienie fizycznego rozmiaru;
2. korekta orientacji;
3. ustawienie podstawy i pivotu;
4. kontrola blokującego footprintu PWK;
5. kontrola cieni i ryzyk topologii;
6. identyczne zastosowanie zatwierdzonej transformacji do MDL, PWK i raportu.

Najważniejszą kontrolką nie powinna być surowa `Scale`, lecz
`Docelowa wysokość (m)`. Użytkownik zna oczekiwany rozmiar urządzenia, ale nie
powinien ręcznie obliczać skali `2,2 / 0,6`.

## 2. Źródła audytu

### 2.1. Kod lokalny

- `apps/studio-web/src/App.tsx`
- `apps/studio-web/src/app/studioSession.ts`
- `apps/studio-web/src/features/source/SourceStep.tsx`
- `apps/studio-web/src/features/inspect/InspectStep.tsx`
- `apps/studio-web/src/features/inspect/SourceInspectionPanel.tsx`
- `apps/studio-web/src/features/preview/SceneViewport.tsx`
- `apps/studio-web/src/features/preview/SourceViewport.tsx`
- `apps/studio-web/src/features/meshy/MeshyModelViewport.tsx`
- `apps/studio-web/src/features/review/PlaceableReview.tsx`
- `apps/studio-web/src/worker/types.ts`
- `apps/studio-web/src/worker/m2a.worker.ts`
- `crates/m2a-wasm/src/lib.rs`
- `crates/m2a-core/src/placeable.rs`
- `crates/m2a-core/src/model_ir.rs`
- `tools/meshy-exact-triangle-target.mjs`

### 2.2. Istniejące dowody wizualne

- `documentation/evidence/web-placeable-guide-01-select-placeable.png`
- `documentation/evidence/web-placeable-guide-02-inputs.png`
- `documentation/evidence/web-placeable-guide-03-workflow.png`
- `documentation/evidence/tlc-meshy-loot-workstations-v1-owner-nwn-scale-failure-2026-07-26.png`

### 2.3. Oficjalne wzorce interakcji

- [Three.js TransformControls](https://threejs.org/docs/pages/TransformControls.html):
  tryby `translate`, `rotate`, `scale`, local/world, snapping, ograniczenia osi,
  reset i zdarzenia rozpoczęcia/zakończenia przeciągania.
- [Unity — Positioning GameObjects](https://docs.unity3d.com/ja/current/Manual/PositioningGameObjects.html):
  gizmo plus pola numeryczne, skróty W/E/R, pivot/center oraz local/global.
- [Unity — Move, rotate and scale in increments](https://docs.unity3d.com/ja/6000.0/Manual/SnapIncrements.html):
  jawne wartości kroku i transformacja ze snappingiem.
- [Blender — Numeric Input](https://docs.blender.org/manual/sl/3.6/scene_layout/object/editing/transform/control/numeric_input.html):
  wpisywanie dokładnej wartości oraz ograniczenie operacji do osi.

Wzorce te są użyteczne, ale zakres Meshy2Aurora musi pozostać węższy i
bezpieczniejszy od pełnego edytora sceny.

## 3. Stan obecny

### 3.1. Co już działa dobrze

- [x] GLB jest oglądany lokalnie bez wysyłania pliku.
- [x] Kamera może obracać się wokół modelu.
- [x] Viewport potrafi pokazać grid, axes, bounds i wireframe.
- [x] Inspekcja zna `boundsMin` i `boundsMax`.
- [x] Inspekcja zna liczbę wierzchołków, indeksów i trójkątów.
- [x] Target `PLACEABLE` używa limitu statycznego 21 845 trójkątów.
- [x] Pipeline generuje wspólnie model, teksturę, PWK i zasoby modułu.
- [x] PWK ma offline readback.
- [x] Build jest związany z rewizją sesji i odrzuca stare odpowiedzi.

### 3.2. Obecne ograniczenia

| Priorytet | Luka | Skutek |
|---|---|---|
| P0 | Brak `targetHeightMeters` w API placeabla | Meshy `auto_size` staje się przypadkową skalą świata NWN |
| P0 | Viewport zawsze dopasowuje kamerę do bounds | Model 0,32 m wygląda tak samo duży jak model 2,2 m |
| P0 | Brak referencji skali | Nie widać relacji do gracza, drzwi ani tile'a 10 m |
| P0 | Brak kontraktu transformacji | Preview nie może być deterministycznie powtórzony w MDL/PWK |
| P0 | `AUTHORING_OPTIONS_CHANGED` unieważnia także inspekcję i wraca do Source | Każdy ruch gizma zniszczyłby płynny workflow |
| P0 | `BUILD_PLACEABLE_PACKAGE` nie przyjmuje authoring options | Worker/WASM nie może zbudować zatwierdzonej transformacji |
| P0 | Review placeabla nie ma viewportu wyniku | Nie można porównać Source z zapisanym MDL |
| P0 | Brak wizualizacji PWK | Użytkownik nie wie, gdzie gracz zostanie zablokowany |
| P1 | Hardkodowane identity i placement | Każdy build webowy ma te same nazwy oraz pozycję testową |
| P1 | Brak undo/redo/reset | Błędny drag jest kosztowny i nieprzewidywalny |
| P1 | Brak presetów rozmiaru | Użytkownik musi znać metry Aurory bez pomocy UI |
| P1 | Brak widoków Front/Right/Top/Aurora | Trudno wykryć iluzję poprawnej geometrii z jednego kąta |
| P1 | Brak diagnostyki rozłącznych komponentów | Model z setkami wysp może wyglądać dobrze tylko frontalnie |
| P1 | Zduplikowany wybór targetu na ekranie Source | Dwa kontrolery odpowiadają za tę samą decyzję |
| P2 | Brak edycji footprintu PWK | Pozostaje wyłącznie automatyczny prostokąt |
| P2 | Brak zapisu authoringu do sidecara | Nie da się łatwo odtworzyć ustawień poza bieżącą sesją |

### 3.3. Konkretne obserwacje z kodu

1. `SourceInspection` posiada bounds, ale `SourceInspectionPanel` ich nie
   pokazuje.
2. `SceneViewport.fitCamera` celowo kadruje każdy model do viewportu. Jest to
   dobre dla oglądania detalu, lecz ukrywa fizyczną skalę.
3. `SourceViewport` pokazuje surową scenę GLB. Nie jest to jeszcze kanoniczna
   przestrzeń Aurory `Z-up`.
4. Webowy placeable używa stałego `STUDIO_PLACEABLE_PLACEMENT`.
5. Worker request zawiera `identityJson`, `placementJson` i `paletteId`, ale nie
   zawiera authoring options.
6. `PlaceableReview` pokazuje tabele i readback, ale nie pokazuje zbudowanego
   MDL ani PWK.
7. `MeshyModelViewport` ma bogate narzędzia materiałów i kamery, lecz są one
   ustawieniami podglądu. Nie wolno mylić ich z transformacją zapisywaną w
   wynikowym modelu.
8. Lokalny helper `meshy-exact-triangle-target.mjs` już potrafi skalować
   równomiernie przez `--height`. Potwierdza to wykonalność, ale nie stanowi
   publicznego kontraktu aplikacji.

## 4. Zasady dobrego UX dla Meshy2Aurora

### 4.1. Najpierw intencja, potem macierz

Użytkownik powinien wybierać:

- wysokość `2,2 m`;
- „ustaw na ziemi”;
- „obróć przodem do kamery Aurory”;
- „zablokuj gracza w tym obrysie”.

Nie powinien rozpoczynać od edycji 16 elementów macierzy ani arbitralnych
wartości skali.

### 4.2. Jedna transformacja autorytatywna

Gizmo, pola numeryczne, preview, MDL i PWK muszą korzystać z tego samego
kanonicznego kontraktu. Niedopuszczalne jest:

- ustawienie `object.scale` tylko w Three.js;
- osobne przeliczenie skali w UI i Rust;
- skalowanie MDL bez skalowania PWK;
- zapis transformacji wyłącznie w pamięci komponentu React.

### 4.3. Jednostką są metry

- wszystkie pola authoringu używają metrów i stopni;
- grid ma opisany krok;
- bounds pokazują `szerokość × głębokość × wysokość`;
- UI zawsze pokazuje wysokość źródłową i wynikową;
- surowy mnożnik skali pozostaje wartością informacyjną lub polem Advanced.

### 4.4. Uniform scale jako bezpieczny domyślny tryb

MVP powinien pozwalać wyłącznie na równomierne skalowanie.

Non-uniform scale:

- deformuje proporcje;
- utrudnia przewidywanie normalnych i cieni;
- może popsuć relację model–PWK;
- nie rozwiązuje wad geometrii;
- zwiększa liczbę stanów do walidacji.

Jeżeli kiedyś zostanie dodany, powinien być trybem eksperckim z widocznym
ostrzeżeniem i własnymi testami.

### 4.5. Preview nie jest proofem runtime

Viewport może dokładnie pokazać:

- transformację;
- bounds;
- pivot;
- footprint PWK;
- topologię i adjacency.

Nie może sam zamknąć:

- widoczności w Toolset;
- widoczności w NWN;
- wyglądu cienia w rendererze Aurory;
- blokowania gracza w runtime.

UI musi oznaczać te podglądy jako `offline preview`, nie `Aurora proof`.

## 5. Rekomendowany przepływ użytkownika

Nie należy dodawać szóstego głównego kroku. Dla targetu Placeable obecny krok
`Inspect` powinien stać się `Prepare Placeable`.

```text
Source
  └─ wybór GLB + placeables.2da
       ↓
Prepare Placeable
  ├─ Inspect source
  ├─ Size & orientation
  ├─ Origin & ground
  ├─ Collision footprint
  └─ Validation
       ↓
Build
  └─ wspólny IR → MDL/TGA/PWK/2DA/UTP/GIT/GIC/ITP/HAK/MOD
       ↓
Review Output
  ├─ Source / Authored / Aurora readback
  ├─ PWK overlay
  └─ exact report + download
```

### 5.1. Wejście do Prepare

Po inspekcji GLB aplikacja pokazuje:

- `Source size: 0,55 × 0,48 × 0,60 m`;
- ostrzeżenie `This is unusually small for a freestanding workstation`;
- rekomendowany preset;
- przyciski:
  - `Keep source size`;
  - `Set physical size`.

Nie wolno po cichu skalować modelu tylko dlatego, że wygląda podejrzanie.

### 5.2. Zatwierdzenie

Przycisk przejścia do Build powinien mieć tekst:

`Apply authoring & continue`

Pod nim należy pokazać krótkie podsumowanie:

`2,20 m high · grounded · yaw 180° · auto PWK rectangle`

## 6. Proponowany układ ekranu

Na desktopie warto użyć układu znanego już z Meshy Lab:

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ Prepare Placeable     [Move W] [Rotate E] [Size R] [Undo] [Redo] [Reset] │
├──────┬───────────────────────────────────────────────┬─────────────────────┤
│ View │                                               │ Size & orientation  │
│      │          CANONICAL AURORA VIEWPORT            │                     │
│ Iso  │                                               │ Preset: Workstation │
│ Front│    model + grid + player + bounds + pivot    │ Height: [2.20] m    │
│ Right│                                               │ Width:  2.02 m      │
│ Top  │                                               │ Depth:  1.76 m      │
│ NWN  │                                               │ Scale:  3.6667×     │
│      │                                               │ Yaw:   [180]°       │
│      │                                               │ [Ground at Z=0]     │
├──────┴───────────────────────────────────────────────┴─────────────────────┤
│ Source 0.60 m → Target 2.20 m  |  PWK: ready  |  Validation: 1 warning   │
│ [Back]                               [Apply authoring & continue]          │
└────────────────────────────────────────────────────────────────────────────┘
```

### 6.1. Lewy pasek widoku

- `Isometric`;
- `Front`;
- `Right`;
- `Back`;
- `Top`;
- `NWN camera`;
- `Frame selected`;
- `Actual scale`;
- `Fit model`.

`Actual scale` i `Fit model` muszą być osobnymi funkcjami. Fit jest wygodny,
ale nie może ukrywać relacji do świata.

### 6.2. Górny toolbar

- `Select/Q` — kamera i wybór;
- `Move/W`;
- `Rotate/E`;
- `Size/R`;
- `Undo/Ctrl+Z`;
- `Redo/Ctrl+Shift+Z`;
- `Reset`;
- przełącznik `World / Local`;
- przełącznik snappingu.

`R` w naszym produkcie powinno oznaczać bezpieczne skalowanie równomierne lub
wysokość, a nie trzy niezależne osie.

### 6.3. Prawy inspector

Zakładki:

1. `Transform`;
2. `Collision`;
3. `Diagnostics`;

Nie należy umieszczać wszystkich opcji w jednym długim formularzu.

## 7. Kontrolki Transform

### 7.1. Sekcja Size

Pola:

- `Sizing mode`:
  - `Keep source size`;
  - `Target height`;
  - `Uniform scale` w Advanced.
- `Target height (m)`;
- readonly:
  - source height;
  - output width;
  - output depth;
  - resolved uniform scale.
- `Lock proportions` — w MVP zawsze aktywne.

Presety:

| Preset | Zakres orientacyjny | Zastosowanie |
|---|---:|---|
| Tabletop | 0,3–0,8 m | mały artefakt lub urządzenie stołowe |
| Workbench | 1,0–1,5 m | stół, misa, stanowisko pracy |
| Freestanding device | 1,8–2,5 m | reaktor, piec, automat |
| Monumental | 3,0–5,0 m | pomnik, duża konstrukcja |

Preset wstawia wartość, ale użytkownik nadal zatwierdza wynik.

### 7.2. Sekcja Orientation

Podstawowy tryb:

- `Yaw / Facing` wokół osi Z;
- szybkie przyciski `0°`, `90°`, `180°`, `270°`;
- strzałka `Front` na podłożu.

Advanced:

- `Pitch`;
- `Roll`;
- `Auto-align source up to Aurora Z`;
- `Reset orientation`.

Pitch i roll powinny być ukryte domyślnie, bo zwykły placeable powinien stać
pionowo.

### 7.3. Sekcja Origin

- `Original`;
- `Bottom center` — rekomendowany;
- `Bounds center`;
- `Custom`.

Akcje:

- `Ground at Z = 0`;
- `Center on X/Y`;
- pola dokładnego offsetu X/Y/Z;
- widoczny marker pivotu.

UI musi odróżnić:

- `Model origin offset` — zapiekany w geometrii/MDL;
- `Test Area placement` — pozycja instancji w GIT.

Użycie jednego zestawu pól do obu znaczeń byłoby poważnym błędem UX.

### 7.4. Gizmo

Three.js ma gotowy `TransformControls`, więc nie trzeba budować raycastingu
uchwytów od zera.

Wymagania:

- podczas drag wyłączyć `OrbitControls`;
- na `mouseDown` rozpocząć jedną transakcję undo;
- podczas `objectChange` aktualizować lekki preview;
- na `mouseUp` zatwierdzić jedną zmianę i uruchomić authoritative resolve;
- `Escape` anuluje bieżący drag;
- aktywna oś ma nie tylko kolor, ale również etykietę;
- rozmiar gizma nie może zależeć od skali modelu;
- pola numeryczne i gizmo zawsze pokazują tę samą wartość.

## 8. Referencje skali

P0:

- sylwetka człowieka `1,70 m`;
- opisana siatka co `1 m`;
- drobny grid co `0,1 m`;
- linie wymiarowe width/depth/height;
- tile NWN `10 × 10 m` jako przełączalny obrys.

P1:

- referencja drzwi `2,0 × 1,0 m`;
- kamera izometryczna zbliżona do NWN;
- możliwość pokazania trzech placeabli obok siebie w tym samym świecie.

Referencje są obiektami viewportu. Nie mogą wejść do GLB, MDL, PWK ani HAK.

## 9. Collision UX

PWK placeabla powinien być nazwany w UI:

`Blocking footprint (PWK)`

To precyzyjniejsze niż `3D collider`, ponieważ obecny pipeline generuje
blokujący footprint na płaszczyźnie.

### 9.1. MVP

- tryb `Auto rectangle`;
- readonly bounds footprintu;
- widoczny półprzezroczysty prostokąt na podłożu;
- przełącznik `Show PWK`;
- kolor i hatch, nie sam kolor;
- `padding (m)`;
- `Regenerate from transformed model`;
- readback:
  - 4 vertices;
  - 2 faces;
  - `surface_id = 7`.

### 9.2. Później

- `Convex hull`;
- ręczny wielokąt 2D;
- przesuwanie punktów wyłącznie w rzucie Top;
- kontrola samoprzecięć;
- ograniczenie liczby punktów;
- bezpieczny fallback do auto rectangle.

### 9.3. Zasada parytetu

Footprint pokazany w viewportcie ma pochodzić z tego samego wyniku core, który
zostanie zapisany jako PWK. UI nie powinno odtwarzać algorytmu kolizji
niezależnie w TypeScript.

## 10. Diagnostyka modelu

Panel `Diagnostics` powinien grupować wyniki według wpływu na runtime.

### 10.1. Geometry

- liczba meshów;
- vertices/triangles/indices;
- headroom do limitu 21 845;
- bounds źródłowe i wynikowe;
- liczba rozłącznych komponentów;
- liczba trójkątów usuniętych jako niebezpieczne;
- liczba zdegenerowanych trójkątów;
- brakujące normals/UV.

### 10.2. Feature continuity

Automatyczny test nie zrozumie semantycznie „lawy wpadającej do misy”, ale może
ujawnić ryzyko:

- bardzo duża liczba rozłącznych wysp;
- cienkie, jednostronne płaty;
- małe komponenty oderwane od głównej bryły;
- komponent kończący się nad innym bez kontaktu;
- geometria widoczna tylko z wąskiego kąta.

Wynik powinien brzmieć:

`810 disconnected components — inspect continuity from Front, Right and NWN views`

Nie wolno automatycznie odrzucać wszystkich modeli z wieloma komponentami,
ponieważ łańcuchy i dekoracje mogą być poprawnie rozłączne.

### 10.3. Shadows

Można dodać:

- proxy kierunku światła;
- podgląd rzucanego cienia WebGL;
- boundary-edge overlay;
- liczbę linked/boundary adjacency edges;
- ostrzeżenie o rozłącznych wyspach cienia.

Etykieta:

`Offline shadow diagnostic — not Aurora runtime proof`

## 11. Porównanie Source / Authored / Output

Review placeabla powinien posiadać trzy widoki:

1. `Source GLB`;
2. `Authored preview`;
3. `Aurora MDL readback`.

Tryby porównania:

- tabs;
- split view;
- ghost overlay źródła nad wynikiem;
- ten sam preset kamery we wszystkich widokach.

Podstawowe kryteria:

- source i authored różnią się wyłącznie zatwierdzoną transformacją;
- authored i MDL readback mają zgodne bounds w tolerancji;
- PWK jest pokazany na authored i output;
- UI pokazuje exact authoring hash.

## 12. Proponowany kontrakt danych

### 12.1. Build-affecting contract

```json
{
  "schemaVersion": 1,
  "coordinateSpace": "AURORA_Z_UP_METERS",
  "sizing": {
    "mode": "TARGET_HEIGHT",
    "targetHeightMeters": 2.2,
    "uniformScale": true
  },
  "orientationDegrees": {
    "pitchX": 0,
    "rollY": 0,
    "yawZ": 180
  },
  "origin": {
    "mode": "BOTTOM_CENTER",
    "groundAtZero": true,
    "offsetMeters": [0, 0, 0]
  },
  "collision": {
    "mode": "AUTO_RECTANGLE",
    "paddingMeters": 0
  }
}
```

Nazwa sugerowana:

`StaticPlaceableAuthoringOptionsV1`

### 12.2. Resolved report

Core powinien zwrócić:

```json
{
  "schemaVersion": 1,
  "sourceBounds": {
    "min": [-0.275, -0.240, 0],
    "max": [0.275, 0.240, 0.600]
  },
  "resolvedUniformScale": 3.6666667,
  "resolvedMatrix": [16, "finite", "numbers"],
  "outputBounds": {
    "min": [-1.009, -0.879, 0],
    "max": [1.009, 0.879, 2.200]
  },
  "groundOffsetApplied": 0,
  "pwkFootprint": {
    "vertices": [[-1.009, -0.879], [1.009, -0.879], [1.009, 0.879], [-1.009, 0.879]]
  },
  "warnings": []
}
```

`resolvedMatrix` w prawdziwym schemacie musi być zwykłą tablicą 16 liczb.
Powyższy skrót tylko pokazuje jej rolę.

### 12.3. Display-only preferences

Nie wolno mieszać z build contractem:

- aktywna kamera;
- widoczność gridu;
- intensywność światła preview;
- kolor tła;
- aktywna zakładka;
- rozmiar gizma;
- auto-rotate.

Nazwa sugerowana:

`PlaceableEditorPreferencesV1`

Zmiana tych ustawień nie może zmieniać hasha artefaktów.

## 13. Architektura stanu i wydajność

### 13.1. Problem obecnej rewizji

Aktualne `AUTHORING_OPTIONS_CHANGED` wywołuje pełne `invalidateDownstream`:

- zwiększa rewizję;
- wraca do Source;
- kasuje source inspection;
- kasuje appearance inspection;
- kasuje build/result.

To jest poprawne dla zmiany inputu, ale błędne dla interaktywnego authoringu.

### 13.2. Rekomendowany model

Rozdzielić:

- `inputRevision` — zmienia się po wymianie GLB/2DA;
- `authoringRevision` — zmienia się po zatwierdzeniu transformacji;
- `buildIdentity` — hash inputów plus kanonicznego authoring JSON.

Zmiana transformacji:

- zachowuje source/2DA inspection;
- pozostaje w `Prepare Placeable`;
- unieważnia wyłącznie build/result/download;
- nie wykonuje ponownego parsowania GLB;
- aktualizuje szybki preview natychmiast;
- uruchamia core resolve po commit drag lub debounce.

### 13.3. Draft i commit

Podczas drag:

- `draftTransform` aktualizuje Three.js maksymalnie raz na klatkę;
- nie zwiększa globalnej rewizji na każdy pixel;
- nie uruchamia pełnego builda.

Na `mouseUp`, Enter lub blur:

- walidacja;
- core resolve;
- jeden wpis undo;
- zwiększenie `authoringRevision`;
- wyczyszczenie starego wyniku builda.

## 14. Walidacja

### 14.1. Blocking

- target height nie jest skończoną dodatnią liczbą;
- target height poza zatwierdzonym zakresem produktu;
- macierz zawiera NaN/Infinity;
- wynikowe bounds są puste;
- wynikowa geometria przekracza domenę writerów;
- `groundAtZero` nie może ustalić dolnej granicy;
- PWK jest pusty lub samoprzecinający;
- preview resolve i build resolve mają różny authoring hash.

### 14.2. Warning

- źródłowa wysokość jest podejrzanie mała lub duża;
- model nie dotyka poziomu gruntu;
- model przecina grunt;
- docelowa szerokość/głębokość zbliża się do 10 m;
- setki rozłącznych komponentów;
- znaczny procent boundary edges;
- footprint blokuje dużo pustej przestrzeni;
- Pitch/Roll są różne od zera;
- wybrano oryginalny pivot daleko od bounds.

### 14.3. Informational

- rekomendowany preset;
- wyliczona skala;
- rozmiar na tle człowieka;
- headroom trójkątów;
- informacja, że preview cienia nie jest runtime proof.

## 15. Undo, reset i bezpieczeństwo

- `Undo` i `Redo` dla zatwierdzonych zmian;
- jeden drag = jeden wpis historii;
- edycja pola zatwierdza się Enter/blur;
- Escape przywraca wartość sprzed edycji;
- `Reset section` osobno dla Size, Orientation, Origin i Collision;
- `Reset all` wymaga potwierdzenia, jeżeli istnieją zmiany;
- banner `Unsaved authoring changes` nie jest potrzebny, dopóki stan istnieje w
  sesji, ale jest potrzebny przed zamknięciem/importem nowego GLB;
- Build pokazuje dokładne podsumowanie transformacji przed startem.

## 16. Dostępność i responsywność

- każde pole ma label i jednostkę;
- status nie opiera się wyłącznie na kolorze;
- aktywna oś gizma ma tekstową informację;
- wszystko można wykonać polami numerycznymi bez gizma;
- focus jest widoczny;
- skróty nie działają podczas pisania w input;
- wartości numeryczne nie powinny przypadkowo zmieniać się od scrolla strony;
- klikalne cele mają co najmniej około 40 px;
- na węższym ekranie inspector przechodzi do draweru;
- na urządzeniach bez precyzyjnego pointera MVP może udostępnić tylko pola
  numeryczne i presety.

## 17. Microcopy

Zamiast:

`Scale`

użyć:

`Physical size`

Zamiast:

`Collider`

użyć:

`Blocking footprint (PWK)`

Zamiast:

`Auto-size`

użyć w Meshy Lab:

`Meshy auto-size`

oraz dodać:

`Meshy chooses a normalized source size. Verify the final physical height in Prepare Placeable.`

Zamiast:

`Inspection evidence is ready`

dla placeabla:

`Source is valid. Set and confirm its physical size, orientation and collision footprint.`

## 18. Priorytety wdrożenia

### Etap P0-A — kontrakt i core

- [ ] Utworzyć `StaticPlaceableAuthoringOptionsV1`.
- [ ] Walidować skończone metry, stopnie i tryby enum.
- [ ] Resolve transform wykonywać po przejściu do wspólnego IR.
- [ ] Stosować tę samą transformację do geometrii render i generatora PWK.
- [ ] Wyliczać source/output bounds i resolved uniform scale.
- [ ] Emitować authoring JSON i authoring SHA-256 w raporcie.
- [ ] Zachować stary entrypoint V1 jako bezpieczny default `Keep source size`.
- [ ] Dodać nowy jawny entrypoint API zamiast zmieniać semantykę starego.
- [ ] Dodać testy deterministyczności i parytetu MDL–PWK.

### Etap P0-B — worker/WASM

- [ ] Dodać options JSON do granicy WASM.
- [ ] Dodać `RESOLVE_PLACEABLE_AUTHORING` do workera.
- [ ] Dodać build request związany z authoring hash.
- [ ] Zwracać resolved bounds, matrix, warnings i PWK footprint.
- [ ] Utrzymać transferable buffers dla dużego GLB.
- [ ] Odrzucać nieznane pola i niepoprawne schema version.

### Etap P0-C — stan sesji

- [ ] Dodać placeable authoring state.
- [ ] Rozdzielić input revision od authoring revision.
- [ ] Nie wracać do Source przy zmianie transformacji.
- [ ] Unieważniać tylko build/result/download.
- [ ] Dodać draft/commit.
- [ ] Dodać undo/redo.
- [ ] Powiązać build z hashem source, 2DA i authoring JSON.

### Etap P0-D — numeryczny MVP

- [ ] Pokazać source bounds w metrach.
- [ ] Dodać `Keep source size / Target height`.
- [ ] Dodać pole target height.
- [ ] Dodać readonly width/depth/scale.
- [ ] Dodać Ground at Z=0.
- [ ] Dodać yaw 0/90/180/270.
- [ ] Dodać reset sekcji i reset całości.
- [ ] Pokazać sylwetkę 1,70 m i tile 10×10 m.
- [ ] Pokazać wymiary bez względu na autoframe kamery.
- [ ] Dodać podsumowanie authoringu przed Build.

### Etap P1-A — gizmo

- [ ] Zintegrować `TransformControls`.
- [ ] Dodać W/E/R oraz jawne przyciski.
- [ ] Wyłączać OrbitControls podczas drag.
- [ ] Dodać World/Local.
- [ ] Dodać snapping translation/rotation/scale.
- [ ] Dodać dokładny numeric input.
- [ ] Dodać Escape cancel.
- [ ] Jeden drag zapisywać jako jeden undo.

### Etap P1-B — kamera i porównanie

- [ ] Dodać Front/Right/Back/Top/Isometric/NWN.
- [ ] Rozdzielić Actual scale od Fit model.
- [ ] Dodać Source/Authored/MDL readback.
- [ ] Dodać split view i ghost overlay.
- [ ] Synchronizować kamerę pomiędzy widokami.

### Etap P1-C — collision i diagnostyka

- [ ] Pokazać dokładny auto rectangle PWK.
- [ ] Dodać padding.
- [ ] Pokazać surface ID i readback.
- [ ] Dodać disconnected component count.
- [ ] Dodać thin/detached feature warnings.
- [ ] Dodać adjacency overlay.
- [ ] Dodać offline shadow proxy z jasnym disclaimerem.

### Etap P2 — zaawansowane

- [ ] Convex hull PWK.
- [ ] Edytowalny wielokąt PWK w Top view.
- [ ] Sidecar `.m2a-placeable.json`.
- [ ] Presety użytkownika.
- [ ] Podgląd kilku placeabli w jednym 10×10 m tile.
- [ ] Import/eksport authoring contract bez ponownego uploadu GLB.

## 19. Minimalna wersja, którą warto dostarczyć jako pierwszą

MVP nie potrzebuje jeszcze gizma.

Wystarczy:

1. source/output bounds;
2. target height;
3. yaw;
4. bottom-center + ground at zero;
5. sylwetka człowieka i tile 10 m;
6. preview exact bounds;
7. PWK overlay;
8. identyczny authoring JSON używany w core i buildzie;
9. reset oraz undo dla pól;
10. Review z Source/Authored/MDL.

Taki zakres naprawiłby błąd skali obecnych urządzeń i pozwolił go wykryć przed
wydaniem kredytów na kolejną iterację lub przed zbudowaniem MOD/HAK.

## 20. Kryteria akceptacji

### Funkcjonalne

- [ ] GLB 0,60 m ustawiony na 2,20 m daje MDL o wysokości 2,20 m w readbacku.
- [ ] PWK ma footprint zgodny z przeskalowanymi bounds XY.
- [ ] Zmiana kamery nie zmienia authoring hash.
- [ ] Zmiana target height zmienia authoring hash i unieważnia stary build.
- [ ] Gizmo i pole numeryczne pozostają dwukierunkowo zsynchronizowane.
- [ ] Reset przywraca identyczny hash ustawień domyślnych.
- [ ] Po wymianie GLB stary authoring wynik nie może zostać użyty.
- [ ] Po zmianie transformacji source inspection pozostaje ważna.

### UX

- [ ] Użytkownik rozpoznaje, że model ma 0,32 m bez uruchamiania NWN.
- [ ] Użytkownik ustawia wysokość 2,20 m bez obliczania mnożnika.
- [ ] Użytkownik rozróżnia obrót modelu od obrotu kamery.
- [ ] Użytkownik rozróżnia model origin od Area placement.
- [ ] Użytkownik widzi footprint blokowania przed Build.
- [ ] Wszystkie podstawowe operacje są możliwe bez myszy.
- [ ] Ostrzeżenia opisują skutek i sugerowaną czynność.

### Testy

- [ ] Rust unit: validate authoring options.
- [ ] Rust unit: resolve target height.
- [ ] Rust unit: ground at zero.
- [ ] Rust unit: yaw and bounds.
- [ ] Rust integration: transformed MDL readback.
- [ ] Rust integration: transformed PWK parity.
- [ ] WASM boundary: strict JSON and deterministic report.
- [ ] Worker: stale authoring response rejected.
- [ ] Reducer: transform preserves inspection and invalidates build.
- [ ] React: numeric controls, reset, undo/redo.
- [ ] React: pointer drag commits once.
- [ ] React: keyboard shortcuts ignore focused inputs.
- [ ] Visual QA: 1280×800, 1920×1080 oraz wąski layout.
- [ ] Accessibility: keyboard traversal and non-color status.

## 21. Czego nie dodawać w pierwszej wersji

- edycji pojedynczych wierzchołków;
- sculptingu;
- pełnego systemu materiałów;
- nieograniczonego non-uniform scale;
- automatycznej naprawy semantycznej lawy, łańcuchów lub innych detali;
- obietnicy zgodności cieni WebGL z Aurorą;
- osobnej implementacji macierzy w UI i core;
- automatycznego skalowania bez zgody użytkownika;
- zapisywania ustawień tylko w localStorage bez eksportowalnego kontraktu.

## 22. Rekomendacja końcowa

Najlepsza kolejność to:

1. kontrakt transformacji w core/WASM;
2. numeryczne `Physical size`, `Facing` i `Ground`;
3. referencje skali oraz dokładny PWK overlay;
4. dopiero potem gizmo;
5. na końcu zaawansowana edycja footprintu.

Gizmo bez autorytatywnego kontraktu dałoby atrakcyjny podgląd, ale mogłoby
ponownie wyprodukować MDL/PWK różniące się od tego, co użytkownik widział.
Dlatego API i parytet danych są elementem P0, a sam manipulator jest P1.

## 23. Mockup `Prepare Placeable`

Rekomendacje audytu zostały przełożone na działający, lokalny mockup:

- [README i instrukcja uruchomienia](mockups/placeable-authoring-v1-2026-07-26/README.md);
- [interaktywny ekran](mockups/placeable-authoring-v1-2026-07-26/index.html);
- [widok Transform](mockups/placeable-authoring-v1-2026-07-26/01-prepare-placeable-transform.png);
- [widok Collision](mockups/placeable-authoring-v1-2026-07-26/02-prepare-placeable-collision.png);
- [widok Diagnostics](mockups/placeable-authoring-v1-2026-07-26/03-prepare-placeable-diagnostics.png).

Mockup pokazuje docelowy kontrakt UX, a nie zaimplementowany pipeline:

- [x] fizyczna wysokość 2,20 m i wynikowy uniform scale;
- [x] sylwetka gracza 1,70 m oraz tile referencyjny 10 m;
- [x] rozdzielenie `Actual scale` i `Fit model`;
- [x] presety widoków oraz rozróżnienie kamery i transformacji;
- [x] `bottom center`, `ground at Z=0` i yaw;
- [x] dokładny prostokątny footprint PWK;
- [x] surface ID, readback offline i jawne `NWN runtime: NOT TESTED`;
- [x] diagnostyka rozłącznych komponentów, budżetu geometrii i adjacency;
- [x] ostrzeżenie o zachowaniu ciągłości strumienia lawy;
- [x] końcowe `Apply authoring & continue`, bez powrotu do etapu Source;
- [x] trzy widoki wyrenderowane w Chromium, a konsola pozostaje czysta.

Do wdrożenia produkcyjnego nadal pozostają checkboxy z etapów P0/P1 powyżej.
Mockup nie zmienia statusu żadnej funkcji pipeline'u na ukończoną.
