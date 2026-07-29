# Stabilizacja odłączonych skinned accessories Creature

Data: 2026-07-29
Status: `IMPLEMENTED / EXACT_SOURCE_REPLAY_PASS / OWNER_VISUAL_FIX_CONFIRMED`

## Wynik

Pipeline Creature ma wersjonowany, deterministyczny etap
`SkinAccessoryStabilizationV1`. Etap wykrywa odłączone części siatki po
przestrzennym sklejeniu pozycji, mierzy spójność wag oraz deformację na
źródłowych klipach, a zatwierdzony sztywny accessory może przepiąć do jednej
stabilnej kości.

Nie jest to filtr pikseli, wygładzanie tekstury ani usuwanie geometrii. Naprawa
zmienia wyłącznie `JOINTS_0/WEIGHTS_0` zatwierdzonego komponentu. Pozycje,
indeksy, liczba trójkątów, normalne, tangenty, UV, materiały, hierarchia i
animacje pozostają bez zmian. `source.glb` pozostaje bajtowo niezmieniony.

## Potwierdzona przyczyna `void-crystal-knight-h1-v1-demo1`

Właścicielski screenshot exact kandydata `vckdemo1.mod` / `vckhak1.hak` /
`vcknight_m1` potwierdził:

```yaml
modelVisibility: visible
proofCompleteness: failed
failure: detached_skin_accessory_deformation
screenshotSha256: c631239994d395b72fe0622b6eef2154142d684b4bf8a6a5f0f2ee646f5c8850
```

Po przestrzennym weldzie źródło ma pięć komponentów:

- główne ciało;
- dwa dolne kryształy z około 50% wpływu lewego/prawego ramienia i mieszanką
  tułowia/nóg;
- dwa górne kryształy z 85–88% wpływu ramion.

Takie wagi obracają i rozciągają sztywne kryształy podczas animacji. Logical
hash `POSITION/JOINTS_0/WEIGHTS_0/INDICES` jest identyczny w rigged GLB,
animation GLB i połączonym `source.glb`; łączenie animacji nie wprowadziło
błędu. Segmentacja i binary writer zachowały wejściowe wagi, więc nie były
źródłem artefaktu.

`doubleSided` i `emissive` materiału wejściowego pozostają osobnym problemem
zgodności materiału. Nie wyjaśniają czarnych „skrzydeł” i nie zastępują naprawy
skinningu.

## Algorytm

1. Tolerancja weld jest wyprowadzana deterministycznie ze skali całego modelu:
   `max(diagonal * 1e-6, 1e-6)`.
2. Wierzchołki są łączone przestrzennie przez siatkę komórek i 27 sąsiednich
   komórek. Dzięki temu seamy UV/normalnych nie tworzą tysięcy fałszywych wysp.
3. Komponent główny to największy komponent liczony trójkątami, potem
   wierzchołkami i stabilnym indeksem. Nigdy nie jest automatycznie
   stabilizowany.
4. Dla każdego komponentu raportowane są: udział dominującej kości, liczba
   aktywnych kości, jednorodna waga sztywna oraz metryki deformacji dla
   maksymalnie 128 czasów na klip i 64 deterministycznie wybranych
   wierzchołków.
5. Metryki obejmują maksymalny stosunek i błąd odległości par względem bind
   pose oraz minimalną zgodność osi. Jednorodna skala pozy jest normalizowana,
   aby nie uznać stałego skalu armatury Meshy za rozciągnięcie accessory.
6. Auto rozpatruje wyłącznie mały odłączony komponent (co najmniej dwa
   trójkąty, najwyżej 25% segmentu), który nie ma już jednolitej sztywnej wagi
   i ma audytowalny powód ryzyka.
7. Kość Auto jest wybierana spośród stabilnych kości tułowia (`spine`,
   `chest`, `torso`, `hips`, `pelvis`) według odległości bind-space, a nie
   według najsilniejszego obecnego wpływu. Zapobiega to ponownemu wyborowi
   ramienia.

## Opcje Studio

Studio, Worker, WASM i core używają jednego kontraktu:

```json
{
  "schemaVersion": 1,
  "textureArtifactCleanup": false,
  "skinAccessoryStabilization": {
    "schemaVersion": 1,
    "mode": "AUTO"
  }
}
```

Dostępne tryby:

- `Auto` — audytuje i stabilizuje ryzykowne odłączone accessories;
- `Keep source weights` — wykonuje pełny audyt i raport, ale nie zmienia wag;
- `Select bone` — wymaga istniejącej, jednoznacznej nazwy kości i stosuje ją
  do wszystkich zatwierdzonych ryzykownych accessories.

Brak lub niejednoznaczność jawnie wybranej kości jest błędem fail-closed.

## Exact replay Void Crystal Knight

Źródło:

```text
sample-3d/void-crystal-knight-h1-v1/source.glb
SHA-256 d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7
19 704 triangles
```

Wynik Auto:

```yaml
componentCount: 5
detachedComponentCount: 4
riskyComponentCount: 4
stabilizedComponentCount: 4
changedVertexCount: 341
geometryTrianglesBefore: 19704
geometryTrianglesAfter: 19704
lowerPositiveX:
  bone: Spine02
  changedVertices: 94
  maxPairDistanceRatio: 12.815454 -> 1.0
  maxPairDistanceError: 11.815454 -> 0.0000000147
lowerNegativeX:
  bone: Spine02
  changedVertices: 87
  maxPairDistanceRatio: 13.860403 -> 1.0
  maxPairDistanceError: 12.860403 -> 0.0000000146
upperNegativeX:
  bone: Spine
  changedVertices: 77
  maxPairDistanceRatio: 2.3176343 -> 1.0
  maxPairDistanceError: 1.3176343 -> 0.0000000236
upperPositiveX:
  bone: Spine
  changedVertices: 83
  maxPairDistanceRatio: 2.1471014 -> 1.0
  maxPairDistanceError: 1.1471013 -> 0.0000000253
```

Każdy pomiar objął trzy klipy i 176 próbek pozy. Dobór kości zgadza się z
oczekiwaniem diagnozy: dolna para → `Spine02`, górna para → `Spine`.

## Testy regresyjne

Testy syntetyczne obejmują:

- sklejenie seamów i brak fałszywych wysp raw-index;
- bezwzględną ochronę komponentu głównego;
- brak zmian dla poprawnego sztywnego accessory;
- poprawę metryk mieszanego accessory;
- tryby Keep i Select bone;
- deterministyczność;
- zachowanie geometrii, UV, materiału, hierarchii i animacji.

Ignored exact-corpus test odtwarza lokalny kanoniczny model i sprawdza pięć
komponentów, cztery naprawy, dokładne kości oraz pełną linię 19 704 trójkątów.

## Demo2

Właściciel jawnie dopuścił nową iterację po wyniku `visible/failed` z artefaktem
deformacji. Dokładny `vckdemo2.mod` został zbudowany przez Product V3 + Demo V2
i zachowuje `19 704 -> 19 704 -> 19 704` trójkątów. MOD i HAK są zainstalowane
oraz zweryfikowane bajtowo; szczegóły:
[handoff demo2](evidence/void-crystal-knight-h1-v1-demo2-product-v3-ready-for-owner-proof-2026-07-29.md).

## Wynik właścicielski

2026-07-30 właściciel potwierdził, że model jest „ładnie poprawiony”, a następnie
zlecił publikację brancha do merge. Jest to pozytywny wynik dla naprawy
`detached_skin_accessory_deformation`: czarne skrzydła i wydłużone odłamki nie
blokują już kandydata. Właściciel nie przypisał tej wypowiedzi osobno do linii
Toolset i NWN, dlatego dokumentacja nie fabrykuje niezależnych wyników per-lane.
