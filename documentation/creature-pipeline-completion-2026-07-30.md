# Domknięcie pipeline'u Creature — 2026-07-30

Status: `IMPLEMENTED / OFFLINE_GATES_PASS / OWNER_RUNTIME_PROOF_HUMAN_OWNED`

## Zakres

Główna trasa produktu jest jedna:

`Meshy API -> Local Bridge -> merged source.glb -> Studio -> Worker/WASM -> m2a-core -> MDL/TGA/2DA/HAK -> opcjonalny demo MOD`

Agent nie uruchamia Aurora Toolset ani NWN. Zgodnie z `AGENTS.md` końcowy test
wizualny dokładnego kandydata wykonuje właściciel.

## Jeden wspólny limit 300 000 trójkątów

Creature, Placeable i pozostałe trasy render-model korzystają z
`AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000`. Dokładnie 300 000 jest dozwolone,
a 300 001 blokowane. Próg ostrzeżenia jest wyprowadzony z tej samej stałej i
wynosi 150 000.

Profile A, profil zgodnościowy P300K, UI Meshy Lab i Local Bridge nie utrzymują
już własnych progów produktu. Rust wyprowadza P300K bezpośrednio z
`AURORA_MODEL_TRIANGLE_BUDGET_V1`, a Bridge ma jeden lustrzany kontrakt
`AURORA_MODEL_TRIANGLE_BUDGET_V1`, z którego wyprowadza limit rigu i cel
awaryjnego remeshu 295 000.

Niezależna granica binary MDL wynosi 65 535 indeksów na jeden stream
triangle-list, czyli 21 845 trójkątów. Writer dzieli większą geometrię na
deterministyczne segmenty bez usuwania trójkątów i bez zmiany materiału, UV,
tangentów, wag, hierarchii lub animacji.

Liczba poligonów podana w Meshy Lab jest celem API, a nie obietnicą dokładnej
liczby wyjściowej. Bridge mierzy pobrany GLB. Jeżeli wynik przekracza 300 000,
automatycznie wykonuje remesh do bezpiecznego celu nie większego niż 295 000,
ponownie mierzy wynik i dopiero wtedy uruchamia rig oraz animacje. Task REMESH,
koszt, hash wejścia i hash wyniku pozostają w provenance.

## Animacje Meshy

Studio obsługuje od 1 do 10 mapowań:

```json
{ "actionId": 198, "clipName": "ca1slashl" }
```

Kontrakt wymaga unikalnych nieujemnych action IDs, unikalnych nazw z namespace
NWN, dokładnie jednego `cpause1` i najwyżej dziesięciu akcji.

Bridge pobiera osobny raw GLB każdej akcji i scala je na jednym rig task.
Przed merge oraz po merge porównuje logiczne dane:

- `POSITION`;
- `JOINTS_0`;
- `WEIGHTS_0`;
- indeksy;
- morph targets i metadata primitive;
- skins oraz inverse-bind matrices.

Sparse accessors są odrzucane. Merge nie może po cichu zmienić siatki ani
skinningu. Kanoniczny artefakt ma `artifactKind=MERGED_ANIMATION_GLTF`; raw
action GLB są audytowalnymi wejściami.

Pełny 42-klipowy profil H1 — z opcjonalnym `animation-events.json` lub bez —
nie uruchamia już historycznej ścieżki ze stałymi resrefami. Przechodzi przez
collision-free granicę full-native V4 z tożsamością produktu i demo, wspólnym
limitem 300K, dokładną sanitacją degeneratów, opcją naprawy tekstury oraz
stabilizacją accessories. Ponieważ ten profil zachowuje dowolne nazwy kości
źródłowego riga i nie wymaga semantycznego `Hips`, MOD pozostaje częścią jego
audytowalnego, atomowego pakietu. Sidecar zastępuje jedynie tabelę eventów; nie
zmienia geometrii ani keyframe'ów.

## Stabilizacja odłączonych elementów

Canonical wire/API to `SkinAccessoryStabilizationOptionsV2`,
`SkinAccessoryStabilizationReportV2` oraz
`audit_and_stabilize_skin_accessories_v2`. Alias V1 istnieje wyłącznie dla
kompatybilności kodu wywołującego i jest oznaczony jako deprecated.

Audyt:

- liczy komponenty po przestrzennym weldzie pozycji, a nie po raw indeksach;
- nigdy nie modyfikuje głównego ciała;
- nie uznaje samych mieszanych wag za dowód awarii;
- mierzy orientację i rozciągnięcie względem bind pose na finalnych klipach;
- dla komponentu do 128 wierzchołków mierzy wszystkie wierzchołki;
- dla większego komponentu używa deterministycznej próbki zachowującej ekstrema
  przestrzenne i zróżnicowanie wag;
- mierzy wszystkie czasy kluczy do 256 na klip, a większy zbiór próbuje
  deterministycznie i raportuje truncation.

Tryby:

- `AUTO` — stabilizuje tylko mały, odłączony komponent z potwierdzoną
  niesztywną deformacją i wybiera stabilną kość tułowia;
- `KEEP_SOURCE_WEIGHTS` — wykonuje audyt bez zmian wag;
- `SELECT_BONE` — przyjmuje opcjonalną globalną kość oraz jawne mapowania
  `segment:component=BoneName`.

Raport V2 podaje próbkę wierzchołków/czasów, powód ryzyka, kość, liczbę
zmienionych wierzchołków i metryki przed/po. Pozycje, indeksy, UV, normalne,
tangenty, materiały, hierarchia i animacje pozostają niezmienione.

## Materiał i tekstura

Bezpieczny profil Creature świadomie obsługuje dokładnie jeden użyty materiał
źródłowy i jeden klasyczny diffuse TGA. Więcej materiałów daje blocking gate
`M3A-MATERIAL-LIMIT`; pipeline nie wybiera po cichu pierwszego.

`baseColorFactor` jest mnożony w przestrzeni liniowej, a RGB ponownie kodowane
do sRGB. Semantyka alfa jest zgodna z glTF:

- dla `alphaMode=OPAQUE` alfa tekstury i alfa `baseColorFactor` są ignorowane,
  więc wyjściowy TGA jest RGB z identycznymi wartościami RGB i wymiarami;
- dla trybów innych niż OPAQUE kanał alfa może zostać zachowany, ale tryb,
  cutoff i brak pełnego odwzorowania Aurora są jawnie raportowane jako
  niewspierane w bezpiecznym profilu.

Raport wymienia mapowane pola oraz niewspierane metallic/roughness, normal,
emissive i `doubleSided`. Pipeline nie tworzy MTR/TXI ani nie mapuje tych pól
na ryzykowny biały specular bez osobnego kontraktu i proofu runtime.

Opcjonalne `Repair texture artifacts` jest filtrem ochrony krawędzi dla
rzeczywistych odstających pikseli/alpha holes. Nie zastępuje napraw geometrii,
normalnych, UV ani skinningu.

## Tożsamość artefaktów

Resrefy produktu i demo są deterministycznie wyprowadzane z:

- SHA-256 source GLB;
- SHA-256 wejściowego `appearance.2da`;
- SHA-256 dokładnych bajtów opcjonalnego `animation-events.json`;
- profilu i opcji naprawy tekstury;
- trybu stabilizacji;
- znormalizowanych mapowań kości per komponent.

Ziarno tożsamości ma schema V3, a token ma 70 bitów kodowanych base32.
Zmiana sidecaru eventów nie może więc odziedziczyć resrefów starego
MOD/HAK/modelu. Po asynchronicznym obliczeniu SHA-256
Studio ponownie sprawdza revision oraz dokładne obiekty File, więc spóźniony
hash nie może uruchomić builda dla starego wejścia.

Serialized product report/summary/manifest używają schema V3. Canonical typy
Rust również mają sufiks V3; aliasy V2 są tylko deprecated compatibility API.

## Demo z aplikacji

Proceduralny produkt pozostaje immutable i nie zawiera MOD. Osobna granica
buduje demo z dokładnie tego produktu. Pełny profil H1 zachowujący dowolny rig
korzysta z opisanego wyżej atomowego pakietu V4, ale stosuje tę samą
collision-free identity oraz raport demo V2. Raport demo zawiera:

- dokładną nazwę pliku `.mod`;
- `moduleResref` oraz nazwę modułu widoczną w Toolsecie;
- `areaResref` oraz dokładną nazwę Area;
- UTC, HAK, appearance row, długość i SHA-256 MOD;
- `semanticReadbackStatus=PASS`.

Review pokazuje kolejno `.mod`, nazwę modułu i Area, zgodnie z handoffem
wymaganym przez `AGENTS.md`.

## Granica weryfikacji

Testy offline mogą potwierdzić deterministyczność, zachowanie geometrii,
skinningu, animacji, materiału, struktur MOD/HAK i dokładne hashe. Nie są
końcowym proofem renderowania. Status `ready_for_owner_proof` wolno nadać tylko
po zbudowaniu jednej dopuszczonej iteracji, instalacji exact MOD/HAK do
natywnych katalogów z kontrolą hashy i przekazaniu jej właścicielowi. Ten
dokument nie tworzy nowej iteracji modelu i nie zastępuje właścicielskiego
wyniku Aurora/NWN.

## Końcowe bramki offline

Stan z 2026-07-30:

- canonical workspace oraz canonical Meshy asset layout: `PASS`;
- `cargo fmt --all -- --check`: `PASS`;
- `cargo clippy --workspace --all-targets -- -D warnings`: `PASS`;
- pełne `cargo test -p m2a-core`: `PASS`; testy wymagające zewnętrznych
  witnessów pozostają jawnie `ignored`;
- `cargo test -p m2a-wasm`: `33/33 PASS`;
- exact lokalny korpus Creature w release: `5/5 PASS`, w tym 20K, 100K,
  owner-verified V5 około 296K, Stoneback około 296K i Void Crystal Knight;
- Studio typecheck: `PASS`;
- Studio unit/component: `229/229 PASS`;
- produkcyjny build Studio z ponownie zbudowanym WASM: `PASS`;
- Worker z rzeczywistym WASM: `9 PASS`, `2` jawnie pominięte przypadki
  środowiskowe;
- Local Bridge i merge animacji: `26/26 PASS`;
- `git diff --check`: `PASS`.

Nie wykonano płatnego wywołania Meshy API, nie utworzono nowej iteracji
MOD/HAK i nie uruchomiono Aurora Toolset ani NWN. Są to świadome granice
zakresu, a nie niezaliczone bramki implementacji.
