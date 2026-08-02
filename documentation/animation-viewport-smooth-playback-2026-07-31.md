# Płynny playback viewportu Animation Studio — 2026-07-31

Status: `IMPLEMENTED / OFFLINE + LIVE WEB VERIFIED`

## Zakres i klasyfikacja

Zmiana dotyczy wyłącznie webowego Animation Studio i podglądu Three.js. Nie
zmienia animacji zapisywanej do binary MDL, mapowania stanów Aurory ani
zamrożonych MOD/HAK. Jest to wniosek implementacyjny z iteracyjnego użycia
aplikacji podczas authoringu `m2a_voidcleave`.

## Objaw i dowody przed poprawką

- Workspace wywoływał `setPlayheadSeconds()` w każdej klatce
  `requestAnimationFrame`.
- Każdy tick renderował ponownie viewport, dope sheet, bibliotekę i inspektor.
- Kontrolowany viewport zatrzymywał mixer Three.js i wykonywał `seek()` z
  efektu Reacta dla każdej próbki czasu.
- Dope sheet ponownie sortował wszystkie markery względem playheada.
- Zmiana obiektu authored clip zmieniała `buildRoot`, ponownie odczytywała i
  parsowała cały GLB.
- Dotychczasowy browser performance gate mierzył parse/build/projection, ale
  nie rzeczywisty playback viewportu.

Model Void Crystal Knight ma 24 834 wierzchołki i 19 704 trójkąty. Ten koszt
nie był pierwotną przyczyną klatkowania.

## Zaimplementowane rozwiązanie

- [x] Runtime Three.js jest jedynym zegarem ciągłego playbacku.
- [x] Workspace nie posiada własnej pętli `requestAnimationFrame`.
- [x] React otrzymuje próbki czasu co około 80 ms zamiast jednej aktualizacji
  na każdą klatkę renderera.
- [x] Zakończenie klipu zwraca `playing=false`, a ponowne Play restartuje akcję
  od początku.
- [x] Seek podczas pauzy pozostaje dokładny; próbki Reacta nie cofają mixera
  podczas odtwarzania.
- [x] Ukryty panel playbacku nie wykonuje redundantnego `setAnimationUi`.
- [x] Linia playheada używa kompozytowego `transform` z liniowym przejściem
  między próbkami UI.
- [x] Markery timeline są sortowane raz po zmianie klipu; dla dużych klipów
  wybór ograniczonego okna jest liniowy względem limitu 2 000 markerów i
  zmienia kotwicę najwyżej dwa razy na sekundę podczas Play.
- [x] Tracki, keyframe markers i event track są memoizowane.
- [x] GLB jest parsowany tylko po zmianie pliku źródłowego. Zmiana authored
  clip wymienia `AnimationClip` w istniejącym mixerze bez zastępowania sceny,
  geometrii, materiałów, tekstur, kamery i kontrolek.
- [x] Dodano browser gate na dokładnym
  `sample-3d/void-crystal-knight-h1-v1/source.glb`.

## Kontrakty regresyjne

Nowe testy wymagają, aby:

1. Play był delegowany przez workspace do viewport runtime;
2. czas Reacta zmieniał się wyłącznie po callbacku próbkowanym z runtime;
3. runtime wymieniał authored clips bez wymiany model root;
4. `SourceViewport` zachował tę samą funkcję loadera GLB między rewizjami
   klipu;
5. dokładny Void Crystal Knight odtwarzał `m2a_voidcleave` bez renderów rodzica
   na każdą klatkę i bez ponownego `File.arrayBuffer()`;
6. timeline zachował limit 2 000 markerów i zawsze utrzymał zaznaczone klucze.

## Weryfikacja

- `npm test`: PASS — 393 passed, 3 skipped.
- `npm run typecheck`: PASS.
- exact browser performance test: PASS — 2/2 w pliku, rzeczywisty VCK GLB,
  rzeczywisty canvas/WebGL i authored clip; p95 frame interval przechodzi
  `<=25 ms` na profilu desktopowym i `<=50 ms` przy CPU throttle 4×, a udział
  klatek dłuższych niż 50 ms na profilu desktopowym pozostaje poniżej 5%.
- production Vite build: PASS.
- bundle budget: PASS — main JS 459 925 / 460 000 B; wszystkie pozostałe
  limity PASS.
- live Studio flow na `m2a_voidcleave`: czas postępował monotonicznie w
  próbkach około 80 ms, zakończył się na 1,00 s, ponowne Play rozpoczęło klip
  od początku, playhead miał aktywne `transform 80ms linear`, a konsola nie
  zawierała warningów ani błędów.

## Pozostałe ryzyko

Automatyczny gate potwierdza architekturę zegara, brak React frame-renders i
brak reloadu GLB. Nie jest laboratoryjnym pomiarem p95 GPU na wszystkich
kartach graficznych. Jeśli pojawi się problem na słabszym urządzeniu, kolejny
krok to adaptacyjny pixel ratio renderera, bez przywracania zegara Reacta.

## MP4 proof — 2026-07-31

- [x] Nagrano rzeczywisty, headless przebieg webowego Animation Studio przez
  normalny flow aplikacji i dokładny fixture Void Crystal Knight.
- [x] Kadr zawiera nazwę `m2a_voidcleave`, model w `Edited result`, sterowanie
  Play, bieżący czas i dope sheet.
- [x] Odtworzono cały klip od `0.00` do `1.00 s`; osobne klatki kontrolne
  potwierdzają pozy i playhead dla `0.00`, `0.15`, `0.65` i `1.00 s`.
- [x] MP4 zweryfikowano jako H.264, `1440x900`, `25 FPS`, `yuv420p`,
  `11.36 s`, `1,667,219 B`.

Artefakt:
`output/playwright/animation-viewport-proof-2026-07-31/m2a-voidcleave-animation-viewport-proof.mp4`

SHA-256:
`a0ad6e58704b669077b5c23573cac214b4e03b0e2892ddcd080bf1e928c2932f`

To jest proof płynnego playbacku w webowym viewportcie aplikacji. Nie jest to
proof zachowania modelu lub animacji w Aurora Toolset ani NWN.

## Demo z pipeline'u — 2026-07-31

- [x] Demo powstało przez normalny flow aplikacji: canonical Void Crystal
  Knight fixture → Inspect → Animation Mapping → Create & edit →
  `Void crystal cleave`.
- [x] Aplikacja utworzyła `m2a_voidcleave` o długości `1.00 s` i `162`
  kluczach.
- [x] Nagranie zawiera widoczny kadr Animation Studio z nazwą klipu, a potem
  trzy rzeczywiste wywołania Play na authored clipie w pełnoekranowym
  viewportcie aplikacji.
- [x] Canvas Three.js został przechwycony bezpośrednio przez
  `HTMLCanvasElement.captureStream(60)`; nie użyto zewnętrznej animacji,
  generowania klatek ani motion interpolation.
- [x] Surowy capture nie ma przerw dłuższych niż `34 ms` i nie ma żadnej
  przerwy powyżej `50 ms`.
- [x] Finalne MP4 ma H.264, `1280x720`, stałe `60 FPS`, `320` klatek,
  `5.334 s` i `364,654 B`.

Artefakt:
`output/playwright/animation-viewport-demo-2026-07-31/m2a-voidcleave-pipeline-demo.mp4`

SHA-256:
`342301519c1dfc3150bacfdf3dc3c141aff6332d51d5f668fe88109befeb4bc1`

FFmpeg został użyty wyłącznie do kadrowania, konwersji VP8 → H.264 i
ustawienia stałych timestampów 60 FPS. Nie zmieniał pozy ani ruchu
wygenerowanego przez pipeline Meshy2Aurora.

## Korekta demo V1 i zweryfikowane demo V2 — 2026-07-31

- [ ] Poprzedni artefakt `animation-viewport-demo-2026-07-31` jest odrzucony
  jako dowód animacji. Kontrola klatek wykazała, że capture canvasu zachował
  praktycznie jedną pozę mimo postępu czasu aplikacji.
- [x] Demo V2 ponownie przeszło normalny flow produktu: canonical Void Crystal
  Knight fixture → Inspect → Animation Mapping → Create & edit → `Void crystal
  cleave` → `m2a_voidcleave` (`1.00 s`, `162` klucze).
- [x] Każda klatka ruchu została wyrenderowana przez rzeczywisty playhead i
  viewport aplikacji. Zapisano 120 klatek viewportu oraz 31 unikalnych próbek
  authoringowych z krokiem `1/30 s`.
- [x] Kontrola finalnego MP4 potwierdziła wizualnie różne fazy gardy,
  wyprowadzenia ciosu i powrotu. FFmpeg nie interpolował ani nie generował
  pozy; złożył wyłącznie wyrenderowane klatki aplikacji w przebieg zwolniony i
  przebieg `1x`.
- [x] Finalne MP4 ma H.264, `1280x720`, stałe `60 FPS`, `252` klatki,
  `4.200 s` i `609,460 B`.

Artefakt:
`output/playwright/animation-viewport-demo-v2-2026-07-31/m2a-voidcleave-pipeline-demo-v2.mp4`

SHA-256:
`d4e8cf6d26219cda8a66e9248904055d18fb9bfeff414460e55b475d94c8691e`
