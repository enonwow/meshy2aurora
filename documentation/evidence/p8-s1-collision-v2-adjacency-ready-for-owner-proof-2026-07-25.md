# P8 S1 collision V2 — adjacency

Data: 2026-07-25

Status: `OWNER_RUNTIME_NOT_BLOCKING / ROOT_CAUSE_CONFIRMED /
SUPERSEDED_BY_RUNTIME_PWK_AUDIT`

> **Korekta po teście właściciela:** V2 ma poprawne adjacency, ale nadal nie
> blokuje ruchu. Adjacency było rzeczywistą różnicą strukturalną V1, lecz nie
> było root cause kolizji. Aktualny wynik i audyt:
> `p8-s1-collision-v2-owner-runtime-result-2026-07-25.json` oraz
> `p8-s1-collision-v2-aurora-decomp-audit-2026-07-25.md`.
>
> Końcowy audyt exact `nwmain` i `nwserver` potwierdził przyczynę:
> `CNWPlaceableSurfaceMesh::LoadWalkMesh` parsuje PWK jako tekstowe linie.
> V2 jest binary MDL, dlatego loader nie odczytuje żadnych `verts` ani `faces`.

## Exact handoff

1. Testowy MOD: `m2a_s1_c2_mod.mod`
2. Nazwa modułu w Toolset: `Meshy2Aurora S1 Collision V2`
3. Dokładna Area: `Meshy2Aurora M0 binary vertical-slice area`

HAK: `m2a_s1_c2_hak.hak`

Obiekt: `m2a_s1_c2_obj`, appearance row `16500`, placement
`(10.0, 14.5, 0.0)`, bearing `0.0`.

## Podstawa iteracji

Właściciel zgłosił dla dokładnego V1 `m2a_s1_col_mod.mod`, że placeable nie
blokuje ruchu. Jest to zweryfikowany runtime verdict:

- `collisionRuntimeVerdict = not_blocking`;
- `collisionProofCompleteness = verified_by_owner_report`;
- widoczność modelu nie była ponownie oceniana i pozostaje osobną osią.

Dokładny rekord V1:
`p8-s1-collision-v1-owner-runtime-result-2026-07-25.json`.

## Historyczna hipoteza przed testem V2

V1 miał poprawnie spakowany PWK Resource Type `2053`, wspólny resref MDL/PWK,
cztery wierzchołki, dwie trójkątne twarze i `surfaceId = 7`. Znaleziona różnica
w niższej warstwie wspólnego binarnego writera polegała na tym, że wszystkie
trzy pola `NwnMdlFace.asAdjFace` każdej twarzy były zapisywane jako `-1`.
Przed owner testem V2 uznano ją zbyt mocno za przyczynę. Wynik V2 dowodzi, że
poprawka adjacency była niewystarczająca.

Trójkąty V1:

| Face | Vertices | V1 adjacentFaces | Poprawne adjacentFaces |
|---|---|---|---|
| 0 | `[2, 0, 3]` | `[-1, -1, -1]` | `[-1, 1, -1]` |
| 1 | `[1, 3, 0]` | `[-1, -1, -1]` | `[-1, 0, -1]` |

Audytowany compiler referencyjny wylicza adjacency po zbudowaniu faces,
wyszukując drugą twarz współdzielącą oba końce krawędzi. V1 nie łączył więc
wspólnej krawędzi `(0,3)` w binarnej strukturze PWK.

## Minimalna poprawka

- wspólny binary MDL writer wylicza deterministyczne adjacency krawędzi;
- ten sam semantic readback porównuje zapisane adjacency z oczekiwanym;
- krawędź współdzielona przez więcej niż dwie twarze jest odrzucana jako
  niejednoznaczna siatka;
- emisję nowej semantyki aktywuje istniejący wspólny profil
  `PlaceableStaticRigidNativeV1`;
- zamrożone profile creature zachowują historyczne bajty i hashe;
- render MDL, tekstura, `Static = 1`, `Useable = 0`, `surfaceId = 7`, footprint
  oraz placement nie zostały zmienione.

Regresja została najpierw potwierdzona czerwonym testem: V1 zwracał
`[-1,-1,-1]`. Po poprawce ten sam test wymaga i odczytuje
`[-1,1,-1]` / `[-1,0,-1]`.

## Wspólny pipeline

```text
AuroraModelIrV1
  -> PlaceableStaticRigidNativeV1
  -> wspólny binary MDL writer
  -> face plane + surfaceId + adjacentFaces + vertexIndices
  -> wspólny binary MDL semantic readback
  -> PWK type 2053 w tym samym HAK co MDL
```

Nie powstał osobny serializer PWK. Placeable używa tego samego writer/readback,
co pozostałe modele; różni się tylko jawnie wybranym profilem danych.

## Testy offline

| Gate | Wynik |
|---|---|
| czerwony test adjacency przed poprawką | PASS jako regresja: test zawiódł na `[-1,-1,-1]` |
| `cargo test -p m2a-core --test placeable_collision` | PASS: 2/2 |
| `cargo test -p m2a-core --lib --quiet` | PASS: 58 passed, 1 ignored |
| `cargo test -p m2a-core --test h2_r42_visibility_candidate --quiet` | PASS: zamrożony hash creature zachowany |
| `cargo check -p m2a-core --example materialize_s1_collision_placeable` | PASS |
| `cargo fmt --all --check` | PASS |
| V2 PWK binary readback | PASS: vertices `4`, faces `2`, surfaces `[7,7]`, adjacency jak wyżej |

Pełne `cargo test -p m2a-core --quiet` zostało przerwane przez limit procesu po
124 sekundach bez zarejestrowanej awarii poprzedzającej timeout. Dlatego dowód
V2 opiera się na wskazanych, zakończonych gate'ach, a nie na twierdzeniu o
ukończeniu tego długiego wywołania.

## Zamrożone artefakty V2

Katalog:
`C:\Projects\meshy2aurora\proof-output\s1-placeable-collision-v2-adjacency-20260725`

| Artefakt | Bajty | SHA-256 |
|---|---:|---|
| MOD `m2a_s1_c2_mod.mod` | 13 400 | `bdc2c1fc6e1f6f9e0c93d80d3e768fd60e7fe6876b9faeabb9665e83eaaaa7de` |
| HAK `m2a_s1_c2_hak.hak` | 15 741 536 | `25c3f064d6b040cce946adf51092478bc6e239139ccbb4a962249d2e406e8440` |
| render MDL `m2a_s1_c2_ped` | 137 244 | `44036c53394159629e54ae4b01b6f21da5092ae90c84a1ae9a5da376d47e24d6` |
| PWK `m2a_s1_c2_ped`, type `2053` | 1 272 | `4eef6b278925845ebe6ee17646aa0d647a38fbaa56c7e0cce6252950c2e4e6d2` |
| TGA `m2a_s1_c2_tex` | 12 582 956 | `96a45ce0ac3b3eba8e54a56af1aeec425e2c63c441b8a87498f045bb365239dc` |
| `placeables.2da` | 3 019 776 | `7ba24e0829d96ed7651c29c0bd8f2773be9c2cddeb12f09cc40c1b7cbb4413aa` |

Machine-readable handoff:
`C:\Projects\meshy2aurora\proof-output\s1-placeable-collision-v2-adjacency-20260725\ready-for-owner-proof.json`.

## Instalacja natywna

Materializer wymagał nieistniejących celów, zapisał je bez overwrite, a potem
ponownie odczytał i porównał bajty. Dodatkowy niezależny odczyt SHA-256
potwierdził identyczność źródło→cel:

- MOD:
  `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_s1_c2_mod.mod`;
- HAK:
  `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_s1_c2_hak.hak`.

Aurora Toolset ani NWN nie zostały uruchomione przez agenta.

## Wynik checklisty właściciela

- [x] V1 został oznaczony jako `not_blocking`, bez nadpisania historycznych
  artefaktów.
- [x] V2 ma nowy, niekolidujący lineage MOD/HAK/resrefów.
- [x] PWK V2 ma wymagane surface i adjacency.
- [x] MOD/HAK V2 są już w odpowiednich katalogach NWN i są byte-identical z
  kanonicznym outputem.
- [x] Właściciel zgłosił, że kolizja V2 nadal nie działa.
- [x] Wynik zapisano jako `collisionRuntimeVerdict=not_blocking`.
- [x] Adjacency odrzucono jako samodzielną root cause.
- [x] Przeprowadzono audyt exact `nwmain`, `nwserver` i binary PWK V2.
- [x] Potwierdzono root cause: binary PWK trafia do tekstowego loadera
  `CNWPlaceableSurfaceMesh`.
- [ ] Zaimplementować ASCII PWK writer/readback nad wspólnym IR zgodnie z
  audytem V2.

Po wyniku właściciela:

- `collisionCompleteness = pwk_emitted_readback_passed`;
- `collisionRuntimeVerdict = not_blocking`;
- `collisionProofCompleteness = verified_by_owner_report`;
- status = `failed_collision_runtime_gate`;
- następny kandydat = `not_created`.
