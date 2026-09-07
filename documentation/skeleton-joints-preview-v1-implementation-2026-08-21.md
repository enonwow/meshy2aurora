# Skeleton/Joints Preview V1 — implementacja 2026-08-21

## Wynik

Studio nie używa już `THREE.SkeletonHelper` do przedstawiania binary-MDL
readbacku. Zastąpił go semantyczny podgląd riga, który rozdziela właściwe
połączenia ruchowe od rootów, helperów i attachmentów. Domyślny widok pokazuje
kości oraz jointy riga, ale ukrywa połączenia helperów, więc `impact`,
`headconjure` i `handconjure` nie tworzą mylących długich promieni.

## Kontrakt semantyczny

Każdy węzeł dokładnego readbacku otrzymuje jedną kategorię:

- `MODEL_ROOT` — kontener modelu;
- `RIG_ROOT` — korzeń właściwego riga, np. `Wolf_rootdummy`;
- `SKIN_BONE` — aktywny slot mapy skina;
- `RIGID_PIVOT` — transform sterujący sztywną geometrią lub strukturalnym
  fragmentem hierarchii;
- `ATTACHMENT` — znany punkt przyłączenia, m.in. `impact` i `*conjure`;
- `HELPER` — pozostały pomocniczy węzeł bez roli deformacyjnej;
- `UNKNOWN` — pusty lub nierozpoznany węzeł, jawnie zachowany zamiast
  zgadywania.

Klasyfikacja wynika z readbacku: kolejności drzewa, parenta, obecności mesha,
kontrolerów transformacji oraz aktywnych map skina. Nie jest zakodowana tylko
dla `c_wolf`.

## UI i renderer

`SceneViewport` udostępnia osobne przełączniki:

- `Kości` — linie tylko właściwej hierarchii riga;
- `Jointy` — kolorowe punkty transformacji;
- `Helpery / attachmenty` — jawnie opcjonalne pomocnicze punkty i połączenia;
- `Etykiety` — nazwy węzłów;
- `X-Ray` — wyłączenie depth testu dla warstw riga;
- `Bind / rest` i `Animated` — przełączanie między dokładną pozycją readbacku
  a aktualną klatką animacji.

Kliknięty joint pokazuje nazwę, kategorię, rodzica, numer węzła i głębokość.
Renderer aktualizuje istniejące bufory linii i pozycje markerów w każdej klatce;
nie alokuje nowego bufora geometrii na każdą klatkę.

### Korekta czytelności X-Ray

Po próbie właścicielskiej warstwa X-Ray została włączona w domyślnym zestawie
`Grid + Kości + Jointy + X-Ray`. Powód jest geometryczny: większość jointów i
łączących je linii znajduje się wewnątrz bryły modelu, więc zwykły depth test
zasłania poprawnie narysowany rig. Użytkownik nie musi już wykonywać
dodatkowego kliknięcia, aby zobaczyć kości. Kontrolka pozostaje dostępna pod
jednoznaczną nazwą `Kości przez model (X-Ray)`, aby można było przywrócić
widok respektujący głębokość.

Świeża próba `c_wolf/crun` potwierdziła zaznaczone domyślnie `Kości`, `Jointy`
i `Kości przez model (X-Ray)`, binding 25/25, widoczne linie wewnątrz bryły i
brak błędów lub ostrzeżeń konsoli. Regresja Studio po korekcie: 321 testów
PASS.

Podgląd supermodelu pokazuje również dokładne źródła:

- źródło riga/geometrii;
- źródło oraz nazwę bieżącej animacji;
- liczbę kontrolowanych węzłów związanych z carrierem i listę brakujących nazw.

## Realny przypadek `c_wolf`

Końcowy build został uruchomiony w Studio na lokalnym `nwn_base.key` i BIF.
Widoczny stan aplikacji potwierdził:

- katalog `COMPLETE`, 206 supermodeli, 32 832/32 832 przeskanowanych MDL i 0
  błędów nagłówków;
- wybrany `c_wolf`, representative carrier `c_barghest`;
- `c_barghest`: 29 węzłów transformacji, 25 jointów riga, 24 sztywne pivoty
  oraz 3 helpery/attachmenty;
- clip `c_wolf/crun`: binding 25/25 i aktywne odtwarzanie (`0.60 s`);
- `Kości` i `Jointy` włączone domyślnie, helpery wyłączone;
- `Helpery / attachmenty`, `Etykiety`, `X-Ray` oraz `Bind / rest` zostały
  przełączone w realnej karcie i działały, po czym pozostawiono czysty widok
  `Animated` z odtwarzanym `crun`.

W trakcie próby znaleziono i poprawiono rozjazd kontraktu runtime: frontend
oczekiwał historycznego `TOPOLOGY_BIND...V2`, podczas gdy bieżący WASM poprawnie
publikuje `EXACT_REFERENCE_BIND_AND_WEIGHTED_ANCHORS_V3_WITH_MATERIAL_LEDGER`.

## Walidacja

- `npm run typecheck` — PASS;
- `npm test` — 56 plików, 320 testów PASS;
- `npm run test:worker-integration` — 15 PASS, 2 świadomie pominięte;
- pełny build WASM + TypeScript + Vite — PASS;
- realny lokalny przebieg Studio opisany wyżej — PASS.

## Granica dowodu

Jest to lokalny renderer dokładnego binary-MDL readbacku w Three.js. Nie jest
to dowód wyglądu lub zachowania modelu w Aurora Toolset ani NWN. Human-owned
Toolset/NWN proof pozostaje osobną granicą projektu.
