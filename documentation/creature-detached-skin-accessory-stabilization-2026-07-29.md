# Stabilizacja odłączonych skinned accessories Creature

Data: 2026-07-29
Aktualizacja kontraktu: 2026-07-30
Status: `IMPLEMENTED / EXACT_SOURCE_REPLAY_PASS / OWNER_VISUAL_FIX_CONFIRMED`

## Potwierdzona przyczyna

Dla `void-crystal-knight-h1-v1-demo1` właściciel potwierdził model widoczny,
ale proof niekompletny z powodu `detached_skin_accessory_deformation`.
Po przestrzennym weldzie źródło ma pięć komponentów: ciało i cztery kryształy.
Dolne kryształy miały około 50% wpływu ramion i mieszankę tułowia/nóg, a górne
85–88% wpływu ramion. Podczas animacji mieszane wagi obracały i rozciągały
sztywne części.

Logical hashe `POSITION/JOINTS_0/WEIGHTS_0/INDICES` były identyczne w rigged
GLB, animation GLB i merged `source.glb`. Merge, segmentacja i writer nie
wprowadziły błędu. `doubleSided` i `emissive` pozostają osobnym ograniczeniem
materiałowym i nie wyjaśniają czarnych „skrzydeł”.

## Kontrakt V2

Canonical API:

- `SkinAccessoryStabilizationOptionsV2`;
- `SkinAccessoryStabilizationReportV2`;
- `audit_and_stabilize_skin_accessories_v2`.

Nazwy V1 pozostają jedynie jako deprecated alias/wrapper kompatybilności.

```json
{
  "schemaVersion": 1,
  "textureArtifactCleanup": false,
  "skinAccessoryStabilization": {
    "schemaVersion": 2,
    "mode": "SELECT_BONE",
    "componentBoneOverrides": [
      { "segmentIndex": 0, "componentIndex": 2, "boneName": "Spine02" },
      { "segmentIndex": 0, "componentIndex": 4, "boneName": "Spine" }
    ]
  }
}
```

W `SELECT_BONE` globalne `selectedBoneName` jest opcjonalnym fallbackiem.
Jawne mapowanie komponentu ma pierwszeństwo. Duplikat mapowania, pusta,
nieistniejąca lub niejednoznaczna nazwa kości jest błędem fail-closed.

## Algorytm

1. Tolerancja weld: `max(diagonal * 1e-6, 1e-6)`.
2. Pozycje są łączone przez komórki przestrzenne i 27 sąsiadów; seamy UV i
   normalnych nie tworzą fałszywych wysp.
3. Największy komponent po liczbie trójkątów, wierzchołków i stabilnym indeksie
   jest głównym ciałem i nigdy nie jest automatycznie stabilizowany.
4. Komponent do 128 wierzchołków jest mierzony dokładnie. Większy używa
   deterministycznej próbki obejmującej ekstrema pozycji, maksymalne wpływy
   aktywnych kości i farthest-feature selection pozycji+wag.
5. Do 256 czasów kluczy na klip jest mierzonych dokładnie; większa liczba jest
   próbkowana deterministycznie i raportowana.
6. Metryki obejmują stosunek/błąd odległości par i zgodność osi względem bind
   pose, z normalizacją jednolitej skali pozy.
7. `AUTO` rozpatruje tylko odłączony komponent mający co najmniej 2 trójkąty,
   najwyżej 25% segmentu, niejednolite wagi i zmierzoną niesztywną deformację.
8. Kość Auto jest wybierana spośród stabilnych kości tułowia według odległości
   bind-space, nie według najsilniejszego wpływu kończyny.

Naprawa zmienia wyłącznie wagi zatwierdzonego komponentu na jednolite `1.0` do
wybranej kości. `source.glb`, pozycje, indeksy, liczba trójkątów, normalne,
tangenty, UV, materiały, hierarchia i animacje pozostają bez zmian.

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
lowerPair: Spine02
upperPair: Spine
```

Maksymalne rozciągnięcie czterech akcesoriów spadło do około `1.0`, a linia
geometrii pozostała `19 704 -> 19 704 -> 19 704`.

## Testy regresyjne

Pokryte są:

- seamy i brak raw-index false positives;
- ochrona głównego ciała;
- brak zmian dla poprawnego sztywnego accessory;
- brak Auto-fix dla samych mieszanych wag bez zmierzonej deformacji;
- Auto, Keep, global Select i Select per komponent;
- brak duplikatów override;
- deterministyczność;
- outlier wag w dużym komponencie;
- zachowanie geometrii, UV, materiału, hierarchii i animacji.

## Wynik właścicielski

Właściciel potwierdził, że `vckdemo2.mod` jest „ładnie poprawiony”. To jest
pozytywny wynik dla naprawy `detached_skin_accessory_deformation`. Dokument nie
fabrykuje oddzielnych wyników Toolset/NWN ponad dokładny zakres przekazanego
potwierdzenia.
