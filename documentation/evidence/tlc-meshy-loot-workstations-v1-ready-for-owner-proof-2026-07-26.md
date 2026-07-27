# TLC Meshy Loot Workstations V1 — `owner_proof_failed`

Data: 2026-07-26

## Dokładny handoff

- testowy MOD: `m2a_tlcw1_mod.mod`
- nazwa modułu w Toolset: `Meshy2Aurora TLC Loot Workstations V1`
- Area: `Meshy2Aurora TLC Loot Workstations`
- Area resref: `m2a_tlcw1_ar`
- uporządkowany HAK: `m2a_tlcw1_hak.hak`
- status: `owner_proof_failed`
- `modelVisibility = visible`
- `proofCompleteness = verified`
- `collisionRuntimeVerdict = not_tested`

Agent nie uruchamiał Aurora Toolset ani NWN. Właściciel wykonał test NWN i
odrzucił V1: wszystkie trzy urządzenia są za małe, a strumień lawy reaktora nie
wpada prawidłowo do dolnej misy. Kolizja nadal nie została oceniona.

Trwały wynik właściciela:

`C:\Projects\meshy2aurora\documentation\evidence\tlc-meshy-loot-workstations-v1-owner-nwn-result-2026-07-26.json`

## Rezultat

Przygotowano trzy nowe modele przez Meshy Multi-Image-to-3D, sprowadzono każdy
do dokładnie 20 000 trójkątów, a następnie przeprowadzono przez wspólny pipeline
statycznego modelu:

`GLB -> wspólny parser/IR -> binarny MDL -> TGA -> PWK -> placeables.2da -> UTP -> GIT/GIC/ITP -> HAK/MOD`

Pipeline placeabla i tile używa tego samego parsera i IR co pozostałe modele.
Różni się profilem dopuszczenia: statyczny model otrzymuje limit Aurory 21 845
trójkątów na pojedynczy mesh. Webowy krok `INSPECT_SOURCE` został poprawiony,
aby dla targetu `PLACEABLE` i `TILE` stosował ten sam profil statyczny, zamiast
błędnie używać limitu creature 10 000.

## Modele i pochodzenie

### 1. Dismantling Station

- Meshy task: `019f9dba-23f4-7f53-aad6-7fd42121acfc`
- wejście Meshy: 20 337 trójkątów
- finalny GLB: 20 000 trójkątów
- GLB SHA-256:
  `86388b96b6c5c010153de279a8e0708a4f93488228d9e5633e6ffee1f98a3561`
- model resref: `m2a_tlcw1_dis`
- blueprint resref: `m2a_tlcw1_du`
- Appearance row: `16500`
- MDL SHA-256:
  `926032290b37d807cd6b02c25eb2283268f47bca982a75524e4e20ccd94d7075`
- PWK SHA-256:
  `e932c88c51ed1ab9458c9cb42823ab20fb92aff8549b04f77ff9358fb265bbe8`
- shadow adjacency: 58 960 powiązanych krawędzi, 1 040 krawędzi brzegowych

### 2. Purification Station

- Meshy task: `019f9dcb-f81b-78e8-83d5-99e5da50be51`
- wejście Meshy: 20 151 trójkątów
- finalny GLB: 20 000 trójkątów
- GLB SHA-256:
  `540eaa7cf3bce1583ff4d34013162ee399ee5ef30fd845eace8d50c83ca2316e`
- model resref: `m2a_tlcw1_pur`
- blueprint resref: `m2a_tlcw1_pu`
- Appearance row: `16501`
- MDL SHA-256:
  `2d556e10050d81b018abc80c59f0336564c55a3a16af876b59ee304d3381deed`
- PWK SHA-256:
  `40cb59d148f6bcffadf68c0a588f661608cea9876ac2747ed9e60ebeb0671f14`
- shadow adjacency: 51 654 powiązane krawędzie, 8 346 krawędzi brzegowych

### 3. Upgrading Reactor

- Meshy task: `019f9dcc-0f49-70a2-97f7-75962403ea27`
- wejście Meshy: 20 397 trójkątów
- finalny GLB: 20 000 trójkątów
- GLB SHA-256:
  `cf2f6a7b43c6ccaf09ebb2d560a62bed0245ec4734b8c8df5d9b9e4dd0776ebd`
- model resref: `m2a_tlcw1_upg`
- blueprint resref: `m2a_tlcw1_uu`
- Appearance row: `16502`
- MDL SHA-256:
  `efb968f110f30bf6ab485a917c2aee009019e7fc0316c69da16a36905a81497c`
- PWK SHA-256:
  `7110955e139e655de97541cc1ca2d0ea4084e675f9abbf83f1638118af779124`
- shadow adjacency: 51 710 powiązanych krawędzi, 8 290 krawędzi brzegowych

Reaktor ma otwartą kulistą komorę, boczne łańcuchy połączone z konstrukcją nóg,
widoczne stopione wnętrze, strumień lawy i dużą dolną misę.

## Geometria, cienie i kolizja

- [x] Trzy GLB mają dokładnie 20 000 trójkątów.
- [x] Każdy wynik ma jeden mesh i 60 000 indeksów.
- [x] Każdy wynik mieści się poniżej limitu 21 845 o 1 845 trójkątów.
- [x] Niebezpieczne dla Aurory trójkąty zostały usunięte przed uzupełnieniem
  siatki do dokładnego targetu.
- [x] Zachowano gabaryty, materiały i osadzone obrazy Meshy.
- [x] Binarny MDL przeszedł pełny readback.
- [x] Adjacency cieni zostało obliczone przez spawanie pozycji przy zachowaniu
  seamów UV i twardych normalnych.
- [x] Każdy model ma PWK o tym samym resrefie co MDL.
- [x] Każdy PWK przeszedł odczyt semantyczny: 4 wierzchołki, 2 ściany,
  `surface_id = 7`.
- [ ] Wygląd cieni w Aurora Toolset potwierdzony przez właściciela.
- [ ] Wygląd cieni w NWN potwierdzony przez właściciela.
- [ ] Blokowanie gracza przez wszystkie trzy PWK potwierdzone w NWN.

## Pakiet NWN

- [x] `placeables.2da` rozszerzone o wiersze `16500..16502`.
- [x] Trzy UTP mają prawidłowe Appearance i resrefy.
- [x] GIT/GIC zawierają dokładnie trzy instancje.
- [x] Custom palette ITP zawiera dokładnie trzy wpisy.
- [x] HAK zawiera `placeables.2da`, trzy MDL, trzy PWK i trzy TGA.
- [x] MOD zawiera Area, GIT/GIC, IFO/FAC, ITP i trzy UTP.
- [x] Po zbudowaniu HAK i MOD zostały ponownie sparsowane i sprawdzone.

Kanoniczne artefakty:

- `C:\Projects\meshy2aurora\proof-output\tlc-meshy-loot-workstations-v1-20260726\generated\m2a_tlcw1_mod.mod`
- `C:\Projects\meshy2aurora\proof-output\tlc-meshy-loot-workstations-v1-20260726\generated\m2a_tlcw1_hak.hak`

Hashe:

- MOD:
  `6fda8b551dd1799f0eee066b06892dc06eeda3feac3f6298be8bcefa19f3cc79`
- HAK:
  `251d6c0888124b7fdaddffececfed40e893bc3e27fd39216ba40154fa8ddfe36`

## Instalacja natywna

- [x] Cel MOD nie istniał przed kopiowaniem.
- [x] Cel HAK nie istniał przed kopiowaniem.
- [x] MOD skopiowano do:
  `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_tlcw1_mod.mod`
- [x] HAK skopiowano do:
  `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_tlcw1_hak.hak`
- [x] Hash natywnego MOD jest identyczny z kanonicznym źródłem.
- [x] Hash natywnego HAK jest identyczny z kanonicznym źródłem.

## Test właściciela

- [ ] Otworzyć `m2a_tlcw1_mod.mod`.
- [ ] Potwierdzić nazwę modułu `Meshy2Aurora TLC Loot Workstations V1`.
- [ ] Otworzyć Area `Meshy2Aurora TLC Loot Workstations`.
- [ ] Potwierdzić widoczność wszystkich trzech nazwanych placeabli.
- [ ] Włączyć cienie i sprawdzić brak gęstych pasów oraz odłączonych wysp cienia.
- [ ] Uruchomić ten sam moduł z tym samym HAK w NWN.
- [ ] Podejść do każdego urządzenia z kilku stron i potwierdzić kolizję.
- [ ] Zapisać niezależnie `modelVisibility` i `proofCompleteness` dla Toolset/NWN.

Pełny maszynowy handoff:

`C:\Projects\meshy2aurora\proof-output\tlc-meshy-loot-workstations-v1-20260726\ready-for-owner-proof.json`
