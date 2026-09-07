# Implementacja naprawy nakładania szkieletu supermodelu — 2026-08-26

## Wynik

Zaimplementowano ogólną, fail-closed trasę analizy struktury, dopasowania
jointów, skinningu, authoringu i oceny deformacji dla dowolnie wybranego
supermodelu. Trasa nie zawiera produkcyjnych wyjątków po nazwie `c_wolf`, psa
ani innej rodziny.

Dokładny negatywny oracle borzoja V10 jest obecnie poprawnie blokowany przed
eksportem:

- `status=APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY`;
- `motionCompatible=false`;
- `motionQualityStatus=BLOCKED`.

Nie utworzono MOD/HAK ani nowego modelowego `rNN`. Nie uruchamiano Aurora
Toolset ani NWN. Offline gate wykazał, że ten target rig nie jest jeszcze
bezpiecznym kandydatem do wizualnego proofu właściciela.

Plan źródłowy:
`documentation/reference-supermodel-skeleton-application-repair-plan-2026-08-26.md`.

Audyt przyczyny:
`documentation/evidence/reference-supermodel-dog-deformation-root-cause-audit-2026-08-26.md`.

## Zaimplementowany zakres

### Struktura i anatomia

- `ReferenceSupermodelStructuralProfileV1` wyprowadza role, centralny łańcuch,
  kończyny, terminale kontaktu i appendage z exact motion contract; nie z
  resrefu lub gatunku.
- `TargetSurfaceAnatomyV1` raportuje komponenty, płaszczyznę symetrii,
  authoritative surface, oś medialną i kontakty z podłożem.
- Analiza anatomii używa całej authoritative component envelope, zamiast
  traktować jeden największy fragment wielokomponentowej siatki jako całego
  Creature.
- Joint fitter zachowuje exact nazwy, parenty i part numbers carrierów oraz
  wystawia confidence i provenance dopasowania.

### Skinning

- Usunięto z aktywnej trasy globalne, stałe wygładzanie 512-iteracyjne.
- Wagi powstają lokalnie względem segmentów i topologii carrierów, z małym,
  ograniczonym wygładzaniem grafu.
- Zaimplementowano separację lewej/prawej strony, lokalny repair granic gałęzi,
  jawny audit dodatnich wpływów całych trójkątów, projekcję małych komponentów
  oraz identyczne wagi dla atomowych grup seamów.
- Brak wymaganej lokalnej coverage jest blokujący; generator nie rezerwuje
  przypadkowej wyspy powierzchni tylko w celu uzyskania kompletności jointów.
- Wycofano eksperymentalne wymuszanie jednego wspólnego carriera na junctionach.
  Szeroka wersja pogorszyła deformację, a wersja triangle-vote utworzyła 327
  niedozwolonych trójkątów łączących gałęzie. Produkcyjny gate nie został
  osłabiony.

### Deformacja i diagnostyka

- Każdy wymagany klip i render component otrzymuje niepustą próbkę geometrii.
- Próbkowanie czasu zawiera obowiązkowe czasy, klucze i adaptacyjne środki
  największych przedziałów; dla V10 oceniono 9 czasów na klip.
- Gate ma absolutne limity dla krawędzi, ekspansji i zapadania pola trójkątów,
  niezależne raporty per klip/per komponent, clip-start continuity oraz
  względny ruch appendage.
- Diagnostyka najgorszego trójkąta zawiera klip, czas, segment, indeksy
  wierzchołków i ich dominujące wpływy.
- Dodano batch evaluator cache'ujący próbki deformacji. Test potwierdza jego
  dokładną zgodność z powtarzanym single-sample oracle.

### Authoring i granice produktu

- `ReferenceSupermodelRigAuthoringDocumentV2` obsługuje landmark overrides,
  joint overrides, component bindings, region weight constraints i sparse
  weight overrides.
- Dokument jest związany hashami z source, exact chain, kontraktem, bazowym
  rigiem i analizą anatomii; nieprawidłowy lub stale dokument jest odrzucany.
- Studio udostępnia edycję wszystkich powyższych typów korekt bez zmiany nazw,
  hierarchii ani numerów carrierów.
- Core, WASM, Worker i Studio używają tej samej wersji V2 authoringu oraz tej
  samej trasy preview/product. Worker kieruje schema V2 do product V4.
- Produkt nie może ominąć motion-quality gate ani samodzielnie
  przepieczętować zmienionego authoringu.

## Dokładny wynik diagnostyczny V10

Źródło:

- `sample-3d/borzoi-c-wolf-bind-v6-p300k-v1/source.glb`;
- SHA-256:
  `3EFD673F4EC953DE568B2A30F6F14131A62828398F3133BB89855193B5FFFD93`.

Raport:

- `artifacts/diagnostics/borzoi-v10-skeleton-repair-core-v11/base-preview-report.json`;
- SHA-256:
  `FBF7B6D0D964B118BEAB4CF18FBAE241FF85AF0682BB55D3B522D1B90FCE5C9F`.

Diagnostyczny MDL:

- `artifacts/diagnostics/borzoi-v10-skeleton-repair-core-v11/base-preview.mdl`;
- SHA-256:
  `23482541B90A5902619FB22AA93B001CA4CD60BCC28333AD039846DDD9C0D4B7`.

Struktura i skinning:

- 162 359 wierzchołków i 299 783 trójkąty;
- 30 dopasowanych jointów, 4 łańcuchy kontaktu, 1 appendage chain;
- minimum joint-fit confidence: `0.7727951`;
- 1 751 początkowych konfliktów branch-triangle naprawionych przez 1 710
  zmian etykiet;
- `crossSideLeakageVertexCount=0`;
- `crossBranchLeakageVertexCount=0`;
- `crossBranchTriangleCount=0`;
- średnio `2.2675` dodatniego wpływu na wierzchołek;
- średnia entropia `0.6043`, maksimum `1.0952`;
- 932 lokalne grupy musiały zachować sztywny topology label;
- eksperymentalne shared-carrier groups: `0`.

Anatomia:

- 326 komponentów: 115 authoritative i 211 auxiliary;
- 151 299 authoritative vertices;
- wszystkie wykryte ground-contact landmarks mają
  `Z=0.000006020069122314453`;
- `familyOrResrefRuleUsed=false`.

Motion-quality:

- 42/42 wymagane klipy ocenione;
- 18 696 449 próbek trójkątów;
- 56 089 347 próbek krawędzi;
- 160 792 niedozwolone ekspansje pola;
- 568 niedozwolonych zapaści pola;
- 1 318 125 twardych odchyleń długości krawędzi;
- 7 naruszeń clip-start anchor continuity;
- appendage relative-motion coverage jest obecne i nie ma naruszenia
  bezwładnego ogona;
- każdy oceniony klip ma lokalne naruszenia deformacji, więc wyniku nie można
  naprawić uśrednieniem globalnym.

Najgorszy zarejestrowany przypadek to `ckdbckdie`, czas `1.0`, segment
`m2a_seg_6`, wierzchołki `733, 729, 2755`: jeden trójkąt łączy lokalnie
sztywne wpływy `Wolf_ribcage`, `Wolf_neck` i `Wolf_Rfrontupperleg`, osiągając
area ratio `21337.5371`. To jest konkretna granica junctionu wymagająca
korekty wag/regionu, a nie błąd braku animacji supermodelu.

## Weryfikacja

Przeszły:

- pełne `cargo test -p m2a-core` — bez błędów; testy zależne od brakujących
  lokalnych fixture pozostały jawnie `ignored`;
- 5/5 jednostkowych testów lokalnego skinningu;
- 9/9 testów generic supermodel;
- 15/15 testów motion;
- 5/5 testów authoringu;
- 3/3 testów produktu;
- cached deformation evaluator equivalence;
- `cargo check -p m2a-wasm`;
- `npm run build:wasm` dla web;
- Studio TypeScript typecheck;
- 7 plików / 21 testów Studio supermodels;
- 3 pliki / 15 testów Worker integration, 2 jawnie pominięte;
- Node/WASM export boundary: `M5/M7 Node boundary PASS` na właściwym buildzie
  `--target nodejs`.

Pierwsza próba Node boundary na paczce `--target web` była niewłaściwą komendą
testową i zakończyła się brakiem inicjalizacji instancji WASM. Powtórzenie na
docelowej paczce Node przeszło; nie jest to regresja produktu.

## Co pozostaje przed nowym demo

Implementacja infrastruktury naprawczej i blokad jest zakończona, ale sama
automatyczna kalibracja V10 nie spełnia kryteriów jakości. Następny bezpieczny
krok to authoring V2 dla wykazanych junctionów/regionów albo użycie źródła z
topologią powierzchni lepiej zgodną z wybranym structural profile. Po każdej
korekcie trzeba ponownie uzyskać wszystkie zielone metryki offline.

Nie są jeszcze zamknięte pełne kryteria całego planu:

- wielorodzinny pozytywny korpus regresyjny z etapów 1 i 7;
- Studio heatmapa wag oraz graficzne zaznaczanie regionu (dane i operacje V2
  istnieją, ale obecny edytor używa jawnych indeksów/liczb);
- pozytywny fitted/authored kandydat z `motionQualityStatus=PASS`;
- MOD/HAK i `ready_for_owner_proof`;
- końcowy wizualny werdykt właściciela w NWN.

Do chwili spełnienia tych punktów generator ma nadal blokować produkt. Nie
wolno obniżać limitów, przywracać triangle-vote shared carriera ani tworzyć
nowego demo z raportem `BLOCKED`.
