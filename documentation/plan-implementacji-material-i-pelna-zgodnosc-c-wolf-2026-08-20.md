# Plan implementacji: materiał creature i pełna zgodność z `c_wolf`

Data: 2026-08-20

Dokument realizuje dwa wyniki audytu
`audyt-v3-artefakty-i-kontrakt-supermodelu-c-wolf-2026-08-20.md`:

1. usunięcie regresji materiałowej V3;
2. zastąpienie zgodności topology-only rzeczywistym kontraktem dziedziczonego
   ruchu `c_wolf`.

Plan obejmuje implementację offline, testy produktu i przygotowanie kandydata
dla właściciela. Nie autoryzuje uruchamiania Toolset/NWN ani tworzenia nowego
MOD/HAK/resrefu przed spełnieniem bramki iteracji modelu.

## Wynik docelowy

Pipeline przyjmuje statyczny GLB psa i zwraca creature, które:

- zachowuje wszystkie istotne semantyki materiału NWN:EE, w tym
  `doubleSided`, normal mapę oraz jawną konwersję metallic/roughness;
- pobiera klipy z `c_wolf`, ale nie używa pozy źródłowego borzoja jako rzekomo
  zgodnej pozy `c_wolf`;
- ma osobną warstwę węzłów nośnych dziedziczonych kontrolerów i osobną,
  należącą do produktu warstwę korekt skinningu;
- blokuje eksport, jeżeli potrafi dowieść jedynie zgodności nazw i rodziców;
- przechodzi testy ciągłości powierzchni, pozy łap, kontaktu z ziemią i
  reprezentatywnych klipów.

## Docelowy przepływ

```text
GLB
 ├─ ingest geometrii i pełnego materiału
 ├─ kompilator materiału NWN:EE
 │   └─ TGA + TXI + MTR + material state MDL
 └─ dopasowanie quadruped
     ├─ 30 węzłów nośnych zgodnych z kontraktem c_wolf
     ├─ własne węzły korekcyjne borzoja
     ├─ skin borzoja związany z węzłami korekcyjnymi
     └─ supermodel c_wolf steruje tylko warstwą nośną
```

Obie ścieżki spotykają się dopiero przed zapisem MDL i HAK. Dzięki temu błąd
materiału nie jest maskowany zmianami wag, a błąd deformacji nie jest mylony z
backface cullingiem.

## Etap 0 — zamrożenie regresji przed zmianą kodu

### Zmiany

1. Dodać syntetyczną fixture GLB z jednym materiałem, trzema obrazami,
   `doubleSided=true`, normal mapą i metallic/roughness.
2. Dodać negatywny test odtwarzający błąd V3: tylko diffuse TGA i brak MTR musi
   kończyć się błędem semantyki materiału.
3. Dodać małą syntetyczną fixture supermodelu z dwiema lub trzema kośćmi, w
   której zgodna topologia ma celowo inną pozycję bazową. Dotychczasowa walidacja
   ma ją błędnie akceptować, a nowa ma ją blokować.
4. Zachować dokładne V3 i V4 wyłącznie jako dane diagnostyczne. Nie aktualizować
   ich plików proof ani natywnych instalacji.

### Pliki

- `crates/m2a-core/tests/fixtures/build_synthetic_glb.rs`
- `crates/m2a-core/tests/aurora_material.rs`
- `crates/m2a-core/tests/aurora_material_text.rs`
- `crates/m2a-core/tests/reference_supermodel_retarget.rs`
- nowy test integracyjny regresji borzoja w `crates/m2a-core/tests/`

### Bramka

- Nowe testy muszą najpierw wykazać dwa obecne błędy.
- Fixture nie może zależeć od payloadu retailowego ani lokalnego pliku GLB.

## Punkt 1 — kompletna trasa materiałowa creature

### Etap M1 — wspólny builder materiału

Usunąć z trasy borzoja ręczne:

- `decode_embedded_image_to_tga_v1(..., 0, ...)`;
- pojedyncze `MdlMaterialTextureBindingV1`;
- ręczne dodawanie jednej TGA do HAK-a.

Zastąpić je istniejącym przepływem:

1. `compile_gltf_materials_v1(..., NwnEeMtr)`;
2. `package_aurora_materials_v1(...)`;
3. `ensure_material_tangents_v1(...)`;
4. przekazanie wszystkich `diffuse_binding` do bazowego writera MDL;
5. po zapisie bazowego MDL wywołanie `extend_binary_mdl_with_materials_v1` z
   każdym `package.slots[*].state` oraz readback rozszerzonego MDL;
6. dołączenie wszystkich `package.resources` do HAK-a;
7. `validate_material_resource_semantics_v1` dla każdego TGA, TXI i MTR.

Nie należy implementować drugiego kompilatora specjalnie dla demo. Wspólną
logikę warto wyciągnąć z `model_pipeline.rs` do funkcji używanej zarówno przez
produkt creature, jak i trasę reference-supermodel.

### Etap M2 — raport różnic materiałowych

Dodać `CreatureMaterialSemanticReportV1`, który dla każdego source material i
output slotu zapisuje:

- source material id i output slot;
- base-color texture/factor;
- normal texture i texcoord set;
- metallic/roughness texture oraz operację bake;
- alpha mode/cutoff;
- `doubleSided`;
- wymagane i faktycznie zapakowane resrefy TGA/TXI/MTR;
- dyspozycję każdego kanału: `PRESERVED`, `BAKED`, `DROPPED_WITH_WARNING` albo
  `BLOCKED`.

Status `READY` jest dozwolony tylko wtedy, gdy każdy kanał ma jawną dyspozycję,
a zasoby wynikowe przechodzą readback.

### Etap M3 — dokładny kontrakt dla borzoja

Dla jednego materiału V3 oczekiwany pakiet NWN:EE zawiera:

- diffuse TGA + TXI;
- normal TGA + TXI;
- wypaloną mapę specular/gloss TGA + TXI z wejścia metallic/roughness;
- MTR typu `2072`;
- MTR z `texture0`, `texture1`, `texture2`,
  `renderhint normalandspecmapped` i `twosided 1`.

Resref MDL materiału musi wskazywać dokładnie zapakowany MTR. Wszystkie
zależności muszą być obecne w HAK-u i identyczne po readback.

### Etap M4 — testy materiałowe

Testy jednostkowe:

- `doubleSided=true` → `twosided 1`;
- brak MTR przy `doubleSided=true` → blokada;
- normal map → TGA, TXI `isbumpmap`, tangent data i `texture1`;
- metallic/roughness → deterministyczny bake specular/gloss i `texture2`;
- różne texcoord sety nie mogą być po cichu scalone;
- błędny lub brakujący zasób zależny → blokada.

Test integracyjny:

- package manifest nie może ponownie zawierać tylko modelu, jednej TGA i
  `appearance.2da` dla źródła o trzech obrazach;
- ponowny build tych samych bajtów musi dawać te same SHA-256 wszystkich
  zasobów.

### Kryteria ukończenia punktu 1

- `doubleSided=true` przechodzi source → compiler → MTR → HAK readback bez
  różnicy.
- Wszystkie trzy obrazy są rozliczone w raporcie; żaden nie znika bez jawnej
  dyspozycji.
- MDL zachowuje 300 000 trójkątów, pozycje, normalne, UV i materiały po
  segmentacji/readback.
- Test regresyjny dokładnie odrzuca dawny pakiet V3 bez MTR.
- Pełny `cargo test --workspace` pozostaje zielony.

## Punkt 2 — prawdziwa zgodność ruchu z `c_wolf`

### Etap S1 — naprawa znaczenia API

Zachować odczyt `ReferenceSupermodelContractV1` dla kompatybilności danych, ale
jawnie sklasyfikować go jako `TOPOLOGY_ONLY`.

1. Oznaczyć `retarget_static_mesh_to_reference_supermodel_v1/v2` jako legacy i
   przestać używać ich w produkcyjnej trasie dziedziczonego ruchu.
2. Dodać uczciwie nazwane API topology-only, np.
   `emit_static_mesh_with_supermodel_topology_v1`.
3. Dodać nowe API
   `bind_static_mesh_for_inherited_supermodel_motion_v1`, które wymaga
   `ReferenceSupermodelMotionContractV2`.
4. Raport nowej trasy ma osobne pola:
   - `supermodelClipLookup`;
   - `topologyCompatible`;
   - `bindPoseCompatible`;
   - `skinBindCompatible`;
   - `motionCorrectionApplied`;
   - `motionCompatible`.

Tylko ostatnie pole może być podstawą komunikatu „dziedziczy animacje z
`c_wolf`”.

### Etap S2 — kontrakt ruchu V2

`ReferenceSupermodelMotionContractV2` powinien obejmować:

- resref, classification i `animationScale`;
- ordered node topology;
- wymagane kanały position/orientation/scale per węzeł;
- własny, clean-room neutral kinematic profile z hash-em;
- role anatomiczne: root, torso, neck/head, cztery łańcuchy łap, ogon;
- anchor points i osie stawów;
- wymagane klipy i eventy;
- wersjonowane tolerancje deformacji;
- provenance: read-only reference, bez skopiowanego payloadu.

Profil produktu jest niezależnie napisanym kontraktem semantycznym. Retailowy
`c_wolf` służy wyłącznie jako lokalny, read-only oracle testowy; jego szkielet,
kontrolery i klatki nie są pakowane ani commitowane.

### Etap S3 — test warstwy korekcyjnej

Przed przetwarzaniem borzoja wykonać mały eksperyment na własnej syntetycznej
hierarchii:

1. węzły nośne mają nazwy i rodziców supermodelu i otrzymują jego kontrolery;
2. pod aktywnymi węzłami powstają własne węzły korekcyjne o unikalnych nazwach;
3. dla kości `i` korekta spełnia w pozy neutralnej:
   `worldCarrier(i) * localCorrection(i) = worldTarget(i)`;
4. skin referuje węzły korekcyjne, a nie bezpośrednio węzły nośne;
5. po uruchomieniu kontrolerów warstwa nośna przekazuje ruch, natomiast stała
   korekta zachowuje pozycję i proporcje docelowego riga.

Test ma sprawdzać dokładne położenie w pozy neutralnej oraz znaną rotację i
translację animowaną. Jeżeli writer lub evaluator nie obsługuje tej semantyki,
etap jest fail-closed; nie przechodzimy do ręcznego strojenia wag pełnego psa.

### Etap S4 — builder riga borzoja

Zastąpić `build_c_wolf_compatible_rig_v2/v3/v4` jednym builderem o rozdzielonej
odpowiedzialności:

1. `detect_quadruped_landmarks_v1` — wykrywa cztery łapy, stawy, tułów, szyję,
   głowę i ogon na powierzchni źródła;
2. `build_supermodel_carrier_rig_v1` — tworzy stałą warstwę nośną z profilu
   motion contract, niezależną od boundsów konkretnego psa;
3. `solve_target_correction_nodes_v1` — wylicza własne korekty neutralnej pozy
   borzoja względem warstwy nośnej;
4. `solve_quadruped_skin_weights_v1` — tworzy płynne wagi względem węzłów
   korekcyjnych;
5. `validate_skin_bind_v1` — dowodzi zgodności world bind × inverse bind;
6. `validate_supermodel_motion_compatibility_v1` — próbuje reprezentatywne
   klipy i zwraca status blokujący.

Konstrukcja „każdy rozłączny komponent jako osobny rigid fragment” z V4 nie
wraca do produktu.

### Etap S5 — bramki geometrii i deformacji

Przed skinningiem zbudować dwa grafy:

- zwykłą topologię krawędzi wewnątrz komponentów;
- graf bliskości pomiędzy różnymi komponentami, aby zachować płaty sierści jako
  jedną wizualną powierzchnię.

Minimalne początkowe tolerancje:

- wszystkie pozycje i macierze są skończone;
- `bindWorld * inverseBind` ma maksymalny błąd elementu ≤ `1e-4`;
- zero trójkątów o polu po deformacji < `0.05 ×` pola bazowego;
- zero krawędzi o zmianie długości poza `[0.25, 4.0] ×`;
- co najmniej 99.9% krawędzi pozostaje w `[0.5, 2.0] ×`;
- dla par międzykomponentowych o odległości bazowej
  `d0 ≤ 0.005 × bboxDiagonal`, co najmniej 99.5% spełnia
  `dt ≤ max(2 × d0, 0.01 × bboxDiagonal)`;
- w `cpause1` cztery klastry łap są niepuste, lewe/prawe klastry pozostają po
  właściwych stronach, a błąd kontaktu z ziemią wynosi ≤ 1% wysokości modelu;
- początek klipu nie powoduje skoku żadnego anchor point większego niż 5%
  przekątnej boundsów względem oczekiwanej pozy neutralnej.

Tolerancje należy skalibrować na własnych fixture i bazowym read-only oracle,
a następnie zamrozić przed uruchomieniem borzoja. Nie wolno ich osłabiać tylko
po to, aby przepuścić jeden wynik.

### Etap S6 — zestaw klipów akceptacyjnych

Offline należy próbkować wszystkie keyframes oraz środki przedziałów co
najmniej dla:

- `cpause1`;
- `cwalk` i `crun`;
- `ca1slashl`, `ca1slashr`, `ca1stab`;
- `ckdbck`;
- `cdead`.

Dla każdego klipu raport zawiera bounds, maksymalne przemieszczenie, zakresy
edge/area ratio, spójność grafu międzykomponentowego, kontakt łap i root motion.
Brak wymaganego klipu lub nierozwiązany węzeł jest błędem blokującym.

### Etap S7 — WASM i Studio

Dopiero po zamknięciu Core:

1. wystawić nowy motion contract i raport przez `m2a-wasm`;
2. rozszerzyć typy workera;
3. w Studio pokazywać osobno topology, bind, skin-bind i motion status;
4. nie pokazywać etykiety „compatible/inherited animations”, gdy status jest
   `TOPOLOGY_ONLY`;
5. dodać test worker-WASM i test UI dla blokady niezgodnego bind pose.

### Kryteria ukończenia punktu 2

- Sama zgodność 30 nazw/rodziców nie przechodzi jako `motionCompatible`.
- Syntetyczny test warstwy nośnej i korekcyjnej odtwarza dokładnie neutralną
  pozę oraz oczekiwany ruch.
- Rig borzoja ma osobne węzły nośne i korekcyjne; mesh nie jest ważony do
  źródłowo dopasowanych węzłów udających bezpośrednio `c_wolf`.
- Readback MDL zachowuje węzły, rodziców, bind matrices, inverse binds, wagi i
  `supermodel c_wolf`.
- Wszystkie osiem klipów przechodzi bramki deformacji.
- Dokładny przypadek V4 zostaje odrzucony przez graf międzykomponentowy.
- Dokładny przypadek V3 zostaje odrzucony jako niezgodny bind pose.
- `cargo test --workspace`, build WASM, testy Studio i worker integration są
  zielone.

## Integracja obu punktów

Po osobnym zamknięciu M1–M4 i S1–S7 powstaje jedna wspólna funkcja produktu,
np. `build_reference_supermodel_creature_package_v1`, która w kolejności:

1. weryfikuje źródło i budżet 300 000 trójkątów;
2. kompiluje materiał NWN:EE;
3. buduje carrier/correction rig;
4. przeprowadza offline motion oracle;
5. zapisuje MDL z pełnym material state;
6. pakuje MDL, TGA, TXI, MTR i `appearance.2da`;
7. parsuje cały HAK i porównuje każdy zasób bajtowo;
8. emituje wspólny raport geometry/material/motion.

Przykład `materialize_borzoi_c_wolf_demo_v1.rs` ma zostać cienkim klientem tej
funkcji, a nie drugim produkcyjnym pipeline'em.

## Kolejność wykonania

1. Etap 0 — czerwone testy regresyjne.
2. M1–M4 — kompletna trasa materiałowa.
3. S1–S2 — naprawa kontraktu i nazewnictwa API.
4. S3 — syntetyczny proof warstwy korekcyjnej; twarda bramka dalszych prac.
5. S4–S6 — rig borzoja i offline motion quality.
6. Integracja wspólnego buildera.
7. S7 — WASM/Studio.
8. Pełne testy i deterministyczny podwójny build offline.
9. Po osobnej zgodzie wynikającej z bramki iteracji: zamrożenie jednego nowego
   MOD/HAK, instalacja no-replace i handoff do właściciela.
10. Właściciel wykonuje ocenę Toolset/NWN.

## Globalne kryteria zakończenia

Implementacja jest zakończona po stronie agenta dopiero, gdy:

- dawny błąd materiałowy V3 ma test regresyjny i nie może się powtórzyć;
- dawny topology-only kontrakt nie może zgłosić pełnej zgodności ruchu;
- oba podsystemy są używane przez jeden wspólny builder;
- wszystkie testy Core, WASM i Studio są zielone;
- build jest deterministyczny i nie modyfikuje proofów V3/V4;
- jeden dozwolony kandydat jest zamrożony, zhashowany i zainstalowany no-replace;
- status końcowy brzmi `ready_for_owner_proof`, nie „wizualnie zaliczone”.

Ostateczne zamknięcie wizualne następuje dopiero po potwierdzeniu właściciela,
że ten sam dokładny kandydat nie ma dziur materiałowych, stoi naturalnie na
czterech łapach i przechodzi idle/chód/bieg/atak w Toolset oraz NWN.
