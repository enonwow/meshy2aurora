# Placeable shadow adjacency — diagnoza i poprawka offline

Data: 2026-07-26

Status: `READY_FOR_OWNER_PROOF / OWNER_RUNTIME_PROOF_NOT_RUN`

## 1. Zakres

Właściciel zgłosił w NWN:EE pasiaste, rozciągnięte cienie trzech placeable z
lineage:

- MOD: `m2a_tlcm3_mod.mod`;
- HAK: `m2a_tlcm3_hak.hak`;
- modele: `m2a_tlcm_rel`, `m2a_tlcm_lamp`, `m2a_tlcm_ward`;
- profil writera: `PlaceableStaticRigidNativeV1`;
- każdy render mesh: `20 000` trójkątów i `shadow=1`.

Najbardziej czytelny objaw wystąpił na `m2a_tlcm_lamp`: cień składał się z
dużej liczby równoległych pasów zamiast jednej spójnej sylwetki.

## 2. Potwierdzona przyczyna

### Fakt z wygenerowanego binary MDL

Poprzedni `checked_face_adjacency()` budował klucz krawędzi wyłącznie z pary
finalnych indeksów renderujących `u32`.

GLB z Meshy zachowuje osobne renderujące wierzchołki dla tej samej pozycji,
gdy różnią się UV albo normalną. Dwie ściany stykające się geometrycznie mogą
więc używać różnych indeksów na wspólnej krawędzi.

Writer wpisywał wówczas `-1` do `adjacentFaces`, mimo że ściany były
geometrycznymi sąsiadami. Przy `shadow=1` takie fałszywe brzegi stają się
krawędziami sylwetki dla shadow volume i tworzą osobne, nakładające się pasy.

### Pomiar exact wygenerowanych modeli

| Model | Faces | Render vertices | Brzegi otwarte według starych indeksów | Brzegi otwarte po exact position weld | Non-manifold edge groups |
|---|---:|---:|---:|---:|---:|
| `m2a_tlcm_lamp` | 20 000 | 26 086 | 24 500 | 31 | 22 |
| `m2a_tlcm_rel` | 20 000 | 25 721 | 24 036 | 40 | 33 |
| `m2a_tlcm_ward` | 20 000 | 24 990 | 23 218 | 49 | 34 |

W `m2a_tlcm_lamp` stary writer oznaczył jako otwarte `40,83%` wszystkich
wystąpień krawędzi. Po połączeniu identycznych pozycji pozostało tylko `31`
rzeczywistych brzegów.

Natywny HAK użyty do obserwacji ma nadal SHA-256:

`0ce9f1097941a928861ee8ee06e44fe174420054c009fd05a16546e5282c55cb`

Jest byte-identical z zamrożonym HAK-em w kanonicznym `proof-output`.

### Fakt z retail

Kontrole `plc_a01` i `plc_b08` pokazują, że detaliczne `adjacentFaces` łączą
ściany przez rozdzielone wierzchołki renderujące.

Przykład `plc_a01`, node `Box18`:

- `26` faces;
- `46` render vertices, lecz `21` unikalnych pozycji;
- topologia po finalnych indeksach sugeruje `46` brzegów;
- retail MDL zapisuje `16` brzegów;
- topologia po pozycjach również daje `16` brzegów.

Hashe lokalnych, read-only kontroli:

- `plc_a01-retail.mdl`:
  `7cdfd63327f23bac22d6cf152c4bbbabc0eb437ad12253b3d2d1a8ea63127634`;
- `plc_b08-retail.mdl`:
  `06470df8d38c720d6b508a19268265f72ffc9d0efc4c3e32e3e82a4d47d1e112`.

### Niezależne potwierdzenie formatu

Lokalny, reference-only `nwn-tools` w `NmcMesh.cpp`:

1. wyznacza face planes i adjacency z pierwotnych indeksów pozycji
   `pTFace->anVerts`;
2. dopiero później buduje finalną listę wierzchołków rozdzielaną przez pozycję,
   normalną, UV, kolor i pozostałe atrybuty.

Nie skopiowano kodu ani payloadu. Fakt o kolejności operacji został
zaimplementowany niezależnie we wspólnym writerze Meshy2Aurora.

Dekompilacja Aurory potwierdza obecność renderera shadow volumes i ścieżki
stencil (`g_nRenderShadowVolumes`, `glStencilOp`, `glStencilFunc`). Powiązanie
pasiastych artefaktów z fałszywymi brzegami zostało dodatkowo zamknięte
bezpośrednim odczytem pól `adjacentFaces` exact wygenerowanych MDL.

## 3. Zaimplementowana delta

Wspólny binary MDL writer:

- nadal zapisuje oryginalne render indices i nie scala UV ani normalnych;
- tworzy klucz adjacency z dwóch dokładnych pozycji geometrycznych;
- kanonizuje `-0.0` i `+0.0` do jednego klucza;
- łączy krawędź tylko wtedy, gdy używają jej dokładnie dwie ściany;
- pozostawia `-1` dla rzeczywiście otwartych krawędzi;
- pozostawia `-1` dla grup non-manifold zamiast wybierać fałszywego sąsiada;
- stosuje tę politykę do `PlaceableStaticRigidNativeV1` i `TileStaticV1`;
- nie zmienia zamrożonych profili creature.

Jest to jedna implementacja we wspólnym writerze, a nie placeable-only parser
lub drugi format modelu.

GLB nie przechowuje oddzielnego, pierwotnego indeksu topologicznego sprzed
rozdzielenia atrybutów. Exact position key jest więc deterministycznym
zamiennikiem dla obecnego ingestu. Jeżeli przyszły importer zachowa jawne
`topologyVertexIds`, writer powinien preferować je nad inferencją z pozycji.

## 4. Testy

Dodano regresje:

- dwie ściany rozdzielone przez różne render vertices, normalne i UV otrzymują
  wzajemne adjacency po wspólnej pozycji;
- trzy ściany używające tej samej geometrycznej krawędzi pozostają jawnie
  non-manifold i nie otrzymują wymyślonego sąsiada.

Uruchomione testy:

- `cargo test -p m2a-core --test mdl_writer`
  — `42 passed`;
- `cargo test -p m2a-core --test placeable_pipeline`
  — `8 passed`, `1 ignored` wymagający wejścia właściciela;
- `cargo test -p m2a-core --test model_pipeline`
  — `26 passed`.

`cargo fmt --all -- --check` nie jest jeszcze zielony z powodu równoległej,
nieukończonej implementacji tile w tym samym worktree. Fragment poprawki cieni
jest zgodny z formatterem; obcych zmian tile nie formatowano ani nie cofano.

## 5. Pozostałe ryzyko i proof

Poprawka usuwa tysiące fałszywych brzegów wynikających z szwów UV. Źródłowe
modele nadal mają po `22–34` grupy non-manifold i po `31–49` rzeczywistych
otwartych brzegów. Mogą one powodować niewielkie lokalne artefakty.

Docelowa polityka jakości powinna dodatkowo oferować zamknięty, uproszczony
shadow proxy:

- visual mesh: `render=1`, `shadow=0`;
- shadow proxy: `render=0`, `shadow=1`.

Shadow proxy nie został dodany w tej delcie, ponieważ wymaga osobnego kontraktu
IR, generatora geometrii i owner runtime proofu.

Po bezpośrednim dopuszczeniu nowej iteracji przez właściciela wygenerowano
jeden świeży lineage `shadow-v2`, bez modyfikowania zamrożonego V1. Exact MOD
`m2a_tlcs2_mod.mod` i HAK `m2a_tlcs2_hak.hak` zostały zainstalowane w
natywnych katalogach NWN po kontroli absent-target i potwierdzeniu
byte-identical SHA-256. Pełny handoff, hashe, pomiary adjacency i checklista
owner proof znajdują się w
`evidence/tlc-meshy-p20k-shadow-adjacency-v2-ready-for-owner-proof-2026-07-26.md`.

Zgodnie z aktywną decyzją projektu końcowy test Toolset/NWN wykonuje
właściciel. Runtime shadow verdict pozostaje `not_tested`.
