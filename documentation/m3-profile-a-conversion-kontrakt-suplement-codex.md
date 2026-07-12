# M3 - kontrakt konwersji creature Profile A

Data: 2026-07-12
Status: `PROFILE_A_LOCKED_M3`; implementacja `IN_PROGRESS`; proof Toolset/game pozostaje `OPEN_M6`

## 1. Cel i autorytet

Ten suplement jest implementowalnym kontraktem M3 i zastepuje starsze kierunki,
jezeli sugeruja kopiowanie szkieletu, wag, animacji albo payloadu
`c_kocrachn`, `c_Horror`, retail lub CEP. Zasoby te sa tylko read-only
obserwacjami invariantow. Obowiazuja `PROJECT_RULES.md`, source-preserving M2,
Aurora First i self-contained profil z `animacje-kontrakt-profil-a-codex.md`.

M3 przyjmuje caly `GlbIngestResult`, aby IR, raport, gates i hash wejscia nie
mogly sie rozjechac. M3 nie zmienia semantyki `AuroraAssetIr`: tworzy oddzielny,
writer-ready `AuroraCreatureIrV1` w target space.

## 2. Granica etapu

### REQUIRED

- odrzucenie konwersji, gdy M2 zawiera choc jeden blocking gate;
- basis, scale, alignment, UV V-flip, winding, normals i tangents;
- jawny `CreatureRigProfileV1` o dozwolonym provenance;
- deterministyczne przypisanie segmentu i bucket `(segmentId, materialSlot)`;
- nearest-surface i barycentryczny transfer wag;
- merge, stable top-4 i normalizacja wag;
- osobne target-space segmenty `SKIN` i opcjonalne `RIGID`;
- raport kazdej transformacji, duplikacji vertexa i naprawy wag;
- deterministyczny Rust/WASM JSON bez sciezek hosta i payloadow reference.

### DEFERRED

- binary MDL/MDX writer, sentinel lanes i smoothing groups: M4;
- emisja/mapowanie klipow: M4A;
- compatibility profile z zewnetrznym supermodelem: osobny pozniejszy profil;
- TGA, bake i atlas: M5;
- Toolset/game, engine facing i wizualny proof UV: M6;
- realny corpus Meshy: M7;
- decymacja/remesh, manual rig, auto-fit pozy i humanoidalny Meshy rig;
- eksperyment pojedynczego globalnego skin node.

## 3. Clean-room provenance

```rust
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RigProvenanceKindV1 {
    Synthetic,
    Owned,
    UserProvided,
    ReferenceOnly,
    Unknown,
}

#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RigProvenanceV1 {
    pub kind: RigProvenanceKindV1,
    pub export_allowed: bool,
    pub attestations: RigProvenanceAttestationsV1,
}

#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RigProvenanceAttestationsV1 {
    pub controlled_construction: bool,
    pub no_reference_payload_copied: bool,
    pub rights_confirmed: bool,
}
```

Eksport jest dozwolony tylko dla:

```text
kind in {SYNTHETIC, OWNED, USER_PROVIDED}
&& exportAllowed == true
&& all attestations == true
```

Poprawnie zdeserializowany `REFERENCE_ONLY`, `UNKNOWN`, `exportAllowed=false`
albo falszywa attestation zwraca `Ok` z blocking gate
`M3A-PROFILE-PROVENANCE-FORBIDDEN` i `creature:null`. Nie jest fatalem.
Nieznany enum, nieznane pole JSON, brak pola albo malformed JSON jest fatal
`M3A-PROFILE-JSON-INVALID`; nie wolno mapowac nieznanej wartosci na `UNKNOWN`.

Attestations sa jawna deklaracja kontrolowanego procesu, nie kryptograficznym
dowodem pochodzenia ani licencji. Evidence musi dodatkowo wykazac controlled
construction synthetic fixture i repo scan bez zewnetrznych payloadow/sciezek.
Content hash wykrywa zmiane tresci, nie dowodzi prawa do eksportu.

Publiczny helper:

```rust
pub fn canonical_profile_sha256(
    profile: &CreatureRigProfileV1,
) -> Result<String, ProfileAConversionFatalError>;
```

Helper serializuje typowany struct po sprawdzeniu finite values: compact JSON,
camelCase, stala kolejnosc pol wynikajaca z definicji struct, bez map/objectow o
dynamicznych kluczach i z pominietym `contentSha256`. Kazdy `-0.0` jest najpierw
normalizowany do `+0.0`; finite `f32` ma najkrotsza reprezentacje zachowujaca
round-trip. Hash to lowercase hex SHA-256 dokladnych bajtow UTF-8. Ten sam helper
jest uzywany przez native i WASM; nie wolno hashowac arbitralnego input JSON.

`canonical_creature_sha256(&AuroraCreatureIrV1)` uzywa tych samych zasad bez
pomijania pol i dostarcza `report.creatureSha256`. Helper przed serializacja
sprawdza finite dla wszystkich matrices, positions, normals, tangents, UV i
weights; NaN lub infinity daje `M3A-NONFINITE-FLOAT`, nigdy JSON `null` ani hash.

Profil ani output nie moga zawierac sciezki hosta, bajtow zewnetrznego assetu,
obrazow, keyframes ani nazw udajacych provenance.

`c_kocrachn` i `c_vampire_f` nie sa rig profile ani wejściem produktu. Ich
metryki w sekcji 12 sa wyłącznie read-only observations.

## 4. Publiczne API

```rust
pub fn convert_profile_a(
    source: &GlbIngestResult,
    rig: &CreatureRigProfileV1,
    options: &ProfileAOptionsV1,
) -> Result<ProfileAConversionOutcomeV1, ProfileAConversionFatalError>;
```

Wejscie niespelniajace policy/gates zwraca poprawny outcome, nie fatal:

```yaml
ProfileAConversionOutcomeV1:
  schemaVersion: 1
  sourceSha256: lowercase_hex_64
  report: ProfileAConversionReportV1
  creature: AuroraCreatureIrV1 | null
```

`creature: null` wystepuje przy co najmniej jednym blocking gate. Fatal jest
zarezerwowany dla nieprawidlowego kontraktu API/profilu, overflow, przekroczenia
limitu pracy albo non-finite danych, ktorych nie mozna bezpiecznie raportowac.

### 4.1 Source contract validation przed gates

Przed odczytem lub propagacja jakiegokolwiek gate M2, M3 niezaleznie waliduje:

```yaml
requiredSourceContract:
  ingestSchemaVersion: 1
  irSchemaVersion: 1
  reportSchemaVersion: 1
  format:
    irSourceFormat: GLB_2_0
    reportFormat: GLB_2_0
  identity:
    irSourceSha256: "equals report.input.sha256; lowercase hex 64"
    irSourceByteLength: "equals report.input.byteLength"
  coordinateSpace:
    equality: "ir.coordinateSpace equals report.coordinatePolicy field-for-field"
    storedSpace: GLTF_SOURCE
    up: POSITIVE_Y
    forwardConvention: POSITIVE_Z
    handedness: RIGHT_HANDED
    units: METERS_DECLARED_BY_MESHY
    positionsPolicy: PRESERVED
    uvPolicy: PRESERVED
    windingPolicy: PRESERVED
    targetTransformStatus: UNRESOLVED_M3
```

Kazda niezgodnosc jest fatal `M3A-SOURCE-CONTRACT-MISMATCH`; nie wolno jej
zdegradowac do gate. Kontrola wiąze logicznie IR z raportem, ale nie jest pelna
kryptograficzna binding proof, poniewaz API core M3 nie otrzymuje oryginalnych
bajtow GLB. M3 nie ufa samym licznikom/gates M2: przed alokacja i konwersja
samodzielnie sprawdza scene/node/mesh/primitive/material refs, counts, indeksy,
finite geometry, topology i zgodnosc attribute lengths.

Poprawny wynik M2 moze zawierac source defect zachowany w IR i odpowiadajacy mu
blocking gate, na przyklad `OTHER` topology, niepelny trojkat, index OOB albo
attribute-count mismatch. Taki wynik nie jest fatal M3. Kolejnosc jest
obowiazkowa:

1. M3 waliduje required source envelope/identity/coordinate contract;
2. w przebiegu O(N), przed alokacja output/scene instances, samodzielnie
   klasyfikuje source defects;
3. kazdy wykryty defekt musi miec matching M2 gate o severity `BLOCKING`, exact
   code i kanonicznym primitive path; gate twierdzacy defekt nieobecny w IR,
   brak gate, zla severity albo zly path daje fatal
   `M3A-SOURCE-CONTRACT-MISMATCH`;
4. gdy istnieje dowolny poprawnie skorelowany M2 blocking gate, M3 natychmiast
   zwraca `Ok`, `creature:null` i `M3A-SOURCE-BLOCKED`, bez transformacji ani
   output/work allocation;
5. dopiero source bez blocking gates przechodzi conversion-ready strict
   validation.

Co najmniej pary `M2A-GLB-PRIMITIVE-MODE-UNSUPPORTED`,
`M2A-GLB-INCOMPLETE-TRIANGLES`, `M2A-GLB-INDEX-OOB`,
`M2A-GLB-ATTRIBUTE-COUNT-MISMATCH`, `M2A-GLB-POSITION-MISSING`,
`M2A-GLB-UV0-MISSING` i `M2A-GLB-DEGENERATE-TRIANGLES` maja test positive
correlation oraz missing/wrong-severity/wrong-path negative correlation.
Non-finite IR pozostaje fatal, poniewaz poprawny ingest M2 nie materializuje go
w `GlbIngestResult`.

Adapter WASM:

```text
convert_profile_a_glb_json(glb_bytes, rig_profile_json, options_json)
  -> deterministic JSON outcome/error envelope
```

WASM wywoluje ten sam core. Nie ma osobnej implementacji transformacji.

Pelny kontrakt opcji (wszystkie pola wymagane, unknown fields denied):

```yaml
ProfileAOptionsV1:
  schemaVersion: 1
  sourceScenePolicy: DEFAULT_SCENE_ONLY
  sourceRigPolicy: REJECT_PRESENT
  sourceAnimationPolicy: REJECT_PRESENT
  normalPolicy: REQUIRE_SOURCE
  basisPolicy: GLTF_TO_AURORA_XZY
  uvPolicy: FLIP_V_ONCE
  windingPolicy: REVERSE_ONCE
  alignmentPolicy: BOTTOM_CENTER_TO_PROFILE_ANCHOR
  materialPolicy: SINGLE_SOURCE_SLOT
  weightMergeEpsilon: 0.0
  weightSumTolerance: 0.00001
  boundsToleranceFactor: 0.00001
  limits: ProfileALimitsV1
```

Enumy i stale liczbowe sa zablokowane w schemaVersion 1. Inna poprawna
syntaktycznie wartosc jest fatal `M3A-OPTIONS-INVALID`, a nie alternatywnym
profilem. `ProfileALimitsV1` musi spelniac invariants i hard maxima z sekcji 8.

Pelny raport ma staly typ i kolejnosc pol:

```yaml
ProfileAConversionReportV1:
  schemaVersion: 1
  source: { sha256, byteLength, defaultSceneId }
  rig:
    profileId: string
    contentSha256: lowercase_hex_64
    provenanceKind: SYNTHETIC | OWNED | USER_PROVIDED | REFERENCE_ONLY | UNKNOWN
    exportAllowed: bool
    attestations:
      controlledConstruction: bool
      noReferencePayloadCopied: bool
      rightsConfirmed: bool
    allAttestationsSatisfied: bool
  policies:
    basisStatus: PROFILE_A_LOCKED_M3
    basisEvidence: REFERENCE_ONLY_IMPLEMENTATION_INFERENCE
    assetForwardMapping: GLTF_POSITIVE_Z_TO_AURORA_POSITIVE_Y
    orientationParity: NEGATIVE_FOR_POSITIVE_SOURCE_AND_RIG_PARITY
    uvEvidence: REFERENCE_ONLY_IMPLEMENTATION_INFERENCE
    uvMapping: GLTF_V_TO_ONE_MINUS_V
    engineFacingProof: OPEN_M6
    uvRuntimeProof: OPEN_M6
    sourceScenePolicy: DEFAULT_SCENE_ONLY
    alignmentPolicy: BOTTOM_CENTER_TO_PROFILE_ANCHOR
  sourceSelection:
    reachableNodeCount: u64
    reachableMeshInstanceCount: u64
    ignoredNodeCount: u64
    ignoredMeshCount: u64
    duplicatedMeshInstanceCount: u64
  transform:
    basisMatrix: [f32; 16]
    determinant: f32
    sourceWorldBounds: { min, max }
    afterBasisBounds: { min, max }
    targetBounds: { min, max }
    scale: f32
    sourceBottomCenter: [f32; 3]
    alignmentAnchor: [f32; 3]
    translation: [f32; 3]
  materials:
    uniqueUsedCount: u64
    maxUniqueMaterials: u64
    bindings: [MaterialSourceBindingV1]
  geometry:
    sourceTriangleCount: u64
    outputTriangleCount: u64
    sourceVertexInstanceCount: u64
    outputVertexCount: u64
    duplicatedVertexCount: u64
  segments:
    - { segmentId, materialSlot, deformation, triangleCount, vertexCount }
  weights:
    skinnedVertexCount: u64
    rigidVertexCount: u64
    mergedDuplicateInfluenceCount: u64
    droppedZeroInfluenceCount: u64
    droppedAfterTopFourCount: u64
    normalizedVertexCount: u64
    maxInfluencesBefore: u64
    maxInfluencesAfter: u64
  work:
    distanceEvaluations: u64
    maxDistanceEvaluations: u64
    workBytesPeak: u64
  gates: [ProfileAGateV1]
  diagnostics: [ProfileADiagnosticV1]
  conversionEligible: bool
  creatureSha256: lowercase_hex_64 | null
```

Gates maja `{code,severity,path,expected,actual,message}`, diagnostics maja
`{schemaVersion,code,severity,path,message}`. Wszystkie liczniki sa checked u64,
a konwersja do host `usize` odbywa sie dopiero po sprawdzeniu zakresu.

## 5. Typy wejscia i wyjscia

Minimalny kontrakt profilu:

```yaml
CreatureRigProfileV1:
  schemaVersion: 1
  profileId: string
  contentSha256: lowercase_hex_64
  provenance:
    kind: SYNTHETIC | OWNED | USER_PROVIDED | REFERENCE_ONLY | UNKNOWN
    exportAllowed: bool
    attestations:
      controlledConstruction: bool
      noReferencePayloadCopied: bool
      rightsConfirmed: bool
  targetBounds: { min: [f32; 3], max: [f32; 3] }
  alignmentAnchor: [f32; 3]
  nodes:
    - { id: u32, name: string, parentId: u32|null, bindLocalMatrix: [f32; 16] }
  segments:
    - id: u32
      name: string
      deformation: SKIN | RIGID
      parentNodeId: u32
      surfacePositions: [[f32; 3]]
      surfaceIndices: [u32]
      allowedBoneNodeIds: [u32]
      referenceWeights:
        - [{ boneNodeId: u32, value: f32 }]
```

`bindLocalMatrix` jest column-major transformem noda wzgledem parenta. Bind-world
jest liczony deterministycznie od jednego root. `surfacePositions` sa w local
space `parentNodeId`; do porownan distance sa przeliczane bind-world dokladnie
raz. Kolejnosc `nodes` i `segments` jest czescia canonical hash; implementacja
nie sortuje profilu w ukryciu.

Target IR nie uzywa glTF-owego znaczenia `JOINTS_0`:

```yaml
AuroraCreatureIrV1:
  schemaVersion: 1
  profileId: string
  sourceSha256: lowercase_hex_64
  basisStatus: PROFILE_A_LOCKED_M3
  engineFacingProof: OPEN_M6
  uvRuntimeProof: OPEN_M6
  nodes:
    - { id: u32, name: string, parentId: u32|null, bindLocalMatrix: [f32; 16] }
  materialSourceBindings:
    - { slot: u32, sourceMaterialId: u32|null, sourceMaterialName: string|null }
  segments:
    - segmentId: u32
      materialSlot: u32
      deformation: SKIN | RIGID
      parentNodeId: u32
      positions: [[f32; 3]]
      normals: [[f32; 3]]
      tangents: [[f32; 4]] | null
      uv0: [[f32; 2]]
      indices: [u32]
      weights:
        - boneNodeIds: [u32|null, u32|null, u32|null, u32|null]
          values: [f32, f32, f32, f32]
          influenceCount: 0..4
```

Output `nodes` jest dokladnie kopia semantyczna rig node hierarchy po walidacji;
nie zawiera source GLB hierarchy. Kazdy output segment wskazuje istniejacy
`materialSlot`. `sourceMaterialId:null` jest legalnym, odrebnym binding key;
opcjonalna nazwa pochodzi z M2 materialu i nie steruje identity. Unikalne
uzyte keys sa liczone tylko z reachable instances default scene i tworza zbior
`None | Some(sourceMaterialId)`; `None` liczy sie jako jedna
wartosc. Slots sa przydzielane deterministycznie: `None` pierwsze, potem
`Some(id)` rosnaco. Dla guardrailu M3 wynik ma zawsze slot `0` albo brak outputu.
`sourceMaterialName` jest zachowane tylko, gdy jest logicznym label <=128 bez
separatorow sciezki, drive prefix ani URI; inaczej output ma `null` i stabilny
diagnostic. Nazwa nigdy nie moze przeniesc host path do reportu/outputu.

Dla `SKIN` kazdy vertex ma `influenceCount` 1..4 i cztery jawne lanes z null dla
pustych. Dla `RIGID` `weights: []` na segmencie i `influenceCount=0`; sztywne
przywiazanie wynika wyłącznie z `parentNodeId` segmentu. M4 mapuje puste skin
lanes na potwierdzony binary representation. M3 nie koduje `0xffff` i nie
wpisuje fikcyjnego bone id.

## 5.1 Default scene i instancje

M3 przetwarza wyłącznie `AuroraAssetIr.defaultSceneId`. Brak default scene,
nieistniejacy scene id, pusty default scene albo niejednoznaczny reachable parent
daje blocking gate. Wszystkie nodes/meshes nieosiagalne z rootow default scene sa
ignorowane i policzone w raporcie; nigdy nie sa konwertowane tylko dlatego, ze
istnieja w tablicy M2.

Traversal jest iteracyjny (bez rekursji/ryzyka stack overflow) i zachowuje
pre-order po `rootNodeIds`, a potem `childIds`, w kolejnosci M2.
Kazdy reachable node z `meshId` jest osobna mesh instance. Gdy dwa nodes wskazuja
ten sam mesh, geometria jest bake'owana i duplikowana dwa razy; wspolny mesh id
nie deduplikuje instancji. Jeden source primitive moze wiec dac wiele output
triangles/vertices. Iloczyny instances x primitives/vertices/indices sa checked
u64 i porownane z output/work budgets przed `try_reserve` oraz przed pierwszym
`push`. Source node hierarchy nie przechodzi do outputu.

## 6. Zablokowana transformacja Profile A

Basis i UV sa `REFERENCE_ONLY_IMPLEMENTATION_INFERENCE`, nie faktem z runtime.
Kotwice sa zapisane w `konwersja-meshy-odpowiedz-codex.md`: Q1, mapowanie
Aurora-MDL -> glTF `[x,y,z] -> [x,z,y]` (linie wskazanego v13 `3476-3478`) oraz
Q6, MDL tverts -> glTF `[u,v] -> [u,1-v]` (linie `1320-1328` i `2245-2250`).
`aurora-web` pozostaje reference-only. Odwrotnosc obserwowanego basis jest
inwolucja wybrana dla implementacji:

```text
P(x, y, z) = (x, z, y)
det(P) = -1
assetForwardMapping = GLTF_POSITIVE_Z_TO_AURORA_POSITIVE_Y
orientationParity = NEGATIVE
```

To zamyka deterministyczna implementacje M3, ale nie nazywa `Aurora +Y`
dowiedzionym frontem engine'u ani V-flip wizualnie zaakceptowanym przez gre.
Oba proofy pozostaja `OPEN_M6`. Raport zawsze zawiera:

```yaml
basisStatus: PROFILE_A_LOCKED_M3
assetForwardMapping: GLTF_POSITIVE_Z_TO_AURORA_POSITIVE_Y
orientationParity: NEGATIVE
engineFacingProof: OPEN_M6
uvRuntimeProof: OPEN_M6
```

Kolejnosc przestrzeni jest zablokowana. Dla kazdej reachable mesh instance:

1. policz `sourceNodeWorld` z local transforms default scene;
2. bake primitive local -> source world dokladnie raz; shared mesh w wielu
   nodes jest duplikowany per instance;
3. zastosuj globalny `G = translation * uniformScale * P`;
4. przypisz triangle do rig segmentu w wspolnym target world;
5. przelicz geometrie przez `inverse(rigParentBindWorld)` do local space
   segmentu;
6. output nodes to dokladnie rig nodes; source hierarchy jest odrzucona.

Nie wolno zastosowac source node transform ponownie w segmencie ani zapisac
global-target positions jako segment-local. Dla punktow:

```text
sourceWorldPosition = sourceNodeWorld * sourceLocalPosition
targetWorldPosition = G * sourceWorldPosition
segmentLocalPosition = inverse(rigParentBindWorld) * targetWorldPosition

sourceWorldNormal = normalize(inverseTranspose(sourceNodeWorld3) * sourceLocalNormal)
targetWorldNormal = normalize(inverseTranspose(G3) * sourceWorldNormal)
segmentLocalNormal = normalize(transpose(rigParentBindWorld3) * targetWorldNormal)

sourceWorldTangent = normalize(sourceNodeWorld3 * sourceLocalTangent.xyz)
targetWorldTangent = normalize(G3 * sourceWorldTangent)
segmentLocalTangent = normalize(inverse(rigParentBindWorld3) * targetWorldTangent)
tangentW = sourceW * sign(det(segmentLocalFromSourceLocal3))

segmentLocalFromSourceLocal = inverse(rigParentBindWorld) * G * sourceNodeWorld
triangle [a,b,c] -> [a,c,b] iff det(segmentLocalFromSourceLocal3) < 0
uv [u,v] -> [u,1-v]
```

Wszystkie macierze uzyte do inverse musza byc finite i odwracalne. Normal lub
tangent o zerowej/non-finite dlugosci blokuje rezultat. Brak normals jest
blocking gate; M3 nie zawiera ukrytego generatora normals. Tangents sa wymagane
tylko, gdy source primitive je posiada; wtedy count musi odpowiadac positions.
Wszystkie primitives trafiajace do jednego output bucket `(segmentId,
materialSlot)` musza miec jednolite pokrycie tangentow: albo kazdy ma tangenty,
albo zaden. Mieszane pokrycie daje blocking gate
`M3A-TANGENT-COVERAGE-MIXED`; M3 nie generuje tangentow, nie wpisuje zer i nie
traci poprawnych danych przez usuniecie tangentow z calego bucketu.
Przy dodatniej parity source node i rig parent winding jest odwracany dokladnie
raz ze wzgledu na globalne `det(P)=-1`. Dla reflected source/rig transform
decyduje znak pelnego composite, aby transformed face normal pozostala zgodna.
UV jest flipowane dokladnie raz.

## 7. Scale i alignment

W target space wysokosc oznacza os Z:

```text
sourceHeight = max(P(sourceWorldBounds)).z - min(P(sourceWorldBounds)).z
targetHeight = rig.targetBounds.max.z - rig.targetBounds.min.z
scale = targetHeight / sourceHeight
```

Obie wysokosci musza byc dodatnie i finite. Nie zakladamy `1 MDL unit = 1 m`
i nie uzywamy `animationScale` jako skali geometrii.

`targetBounds.min/max` musza byc finite i `min < max` osobno na X, Y i Z.
`alignmentAnchor` musi byc finite i lezec wewnatrz zamknietego targetBounds.
Rig bind-world surface bounds (kazdy surface vertex przeliczony z segment local
przez parent bind-world) musza lezec w targetBounds z tolerancja per os:

```text
surfaceTolerance(axis) = boundsToleranceFactor * max(1, targetExtent(axis))
```

Naruszenie target envelope, invalid anchor albo invalid bounds jest profile
fatal `M3A-PROFILE-BOUNDS-INVALID`, nie gate source assetu.

Jedyna polityka M3:

```text
alignment = BOTTOM_CENTER_TO_PROFILE_ANCHOR
bottomCenter(bounds) = [(min.x+max.x)/2, (min.y+max.y)/2, min.z]
translation = rig.alignmentAnchor - bottomCenter(scaledBasisBounds)
```

Brak jawnego alignment lub inna wartosc jest bledem kontraktu. Raport zapisuje
source/after-basis/target bounds, scale, anchor, bottom center i translation.
Output bbox height musi zgadzac sie z target height w tolerancji absolutnej
`1e-5 * max(1,targetHeight)`; szersza wizualna tolerancja `+-20%` dotyczy M6,
nie arytmetyki M3.

## 8. Budzety Profile A

`maxUniqueMaterials=1` jest project guardrail pierwszego profilu, nie
twierdzeniem o limicie Aurory. Dwa materialy daja blocking gate. M3 zachowuje
material id i nie wykonuje bake/atlasu.

```yaml
ProfileALimitsV1:
  maxRigNodes: 4096
  maxSegments: 256
  maxReferenceVertices: 1000000
  maxReferenceTriangles: 1000000
  maxOutputVertices: 1000000
  maxOutputIndices: 3000000
  maxDistanceEvaluations: 3000000
  maxWorkBytes: 268435456
  maxDiagnostics: 2048
  maxUniqueMaterials: 1
  triangleWarningAbove: 5000
  triangleBlockingAbove: 10000
```

Invariants opcji/limitow:

- kazde pole count/bytes/evaluations/diagnostics jest `>0`;
- `triangleWarningAbove == 5000`, `triangleBlockingAbove == 10000` i
  `triangleWarningAbove <= triangleBlockingAbove` dla schemaVersion 1;
- `maxUniqueMaterials == 1` jest exact contract, nie wartoscia podnoszona przez
  custom options;
- custom `maxRigNodes/maxSegments/maxReferenceVertices/maxReferenceTriangles`
  nie przekracza odpowiedniej wartosci tabeli powyzej;
- custom output/work budgets
  `maxOutputVertices/maxOutputIndices/maxDistanceEvaluations/maxWorkBytes` nie
  przekraczaja hard maxima tabeli; moga byc tylko dodatnio obnizone;
- `maxDiagnostics <= 2048`.

Naruszenie jest fatal `M3A-OPTIONS-INVALID` przed konwersja. To zapobiega
podniesieniu budzetu output/work ponad compiled hard maxima przez JSON.

Kazde `count * stride`, suma, duplikacja i rezerwacja jest checked przed
alokacja. Limity sa kumulatywne dla calego assetu/profilu, nie per segment.
`maxDistanceEvaluations`, nie liczba high-level queries, jest autorytatywnym
budzetem nearest-surface. Implementacja moze pozniej dodac akceleracje, ale M3
v1 ma zachowac exact exhaustive wynik i exact accounting z sekcji 9.

## 9. Rig i segmentacja

Profil jest poprawny tylko, gdy:

- ids node i segmentow sa unikalne; nazwy niepuste i logiczne;
- istnieje dokladnie jeden root, parent refs istnieja, hierarchia jest acykliczna;
- bind-local i wyliczone bind-world matrices sa finite i odwracalne;
- target bounds/envelope, anchor i rig surface bounds spelniaja sekcje 7;
- kazdy segment wskazuje istniejacy parent;
- `surfaceIndices.len % 3 == 0`, indeksy sa in-range, powierzchnia niepusta;
- trójkaty reference surface sa niezerowe;
- `referenceWeights.len == surfacePositions.len` dla `SKIN`;
- allowed bones sa unikalne, istnieja i naleza do hierarchii segmentu;
- `RIGID` ma parent i nie ma skin reference weights;
- wszystkie wagi sa finite, nieujemne i wskazuja allowed bone.

Przypisanie jest deterministyczne i wszystkie distance comparisons odbywaja
sie w target world. Reference segment surface jest przeliczona do target world
przez `rigParentBindWorld`; emitted geometry dopiero po wyborze segmentu trafia
do jego local space.

Exact accounting `distanceEvaluations`:

```text
assignment evaluations =
  sum for each target triangle T:
    sum for each candidate segment S:
      3 * surfaceTriangleCount(S)

weight-transfer evaluations =
  sum for each emitted SKIN vertex V:
    surfaceTriangleCount(selectedSegment(V))

total = assignment evaluations + weight-transfer evaluations
```

Przed kazdym pojedynczym vertex-to-triangle distance evaluation licznik jest
checked-incrementowany. Jezeli kolejna wartosc przekroczylaby
`maxDistanceEvaluations`, funkcja konczy sie fatal `M3A-LIMIT-EXCEEDED` przed
obliczeniem dystansu. Nie wolno inkrementowac raz per segment/query ani dopiero
po pracy. RIGID emitted vertex nie wykonuje weight-transfer evaluation.

Algorytm:

1. Dla kazdego target triangle i kazdego segmentu sprawdz kazdy jego surface
   triangle. Dla kazdego z 3 target vertices wybierz minimalny squared distance,
   a wynik segmentu jest suma tych trzech minimow.
2. Wybierz najmniejszy wynik; exact tie rozstrzyga rosnacy `segmentId`.
3. Zbuduj deterministic material bindings; bucket wyjsciowy ma klucz
   `(segmentId, materialSlot)`.
4. Vertex uzyty przez rozne buckety jest jawnie duplikowany; mapping source ->
   output zapisuje raport.
5. Dla kazdego output `SKIN` vertex sprawdz wszystkie triangles powierzchni
   wybranego segmentu i wybierz nearest; tie rozstrzyga najnizszy surface
   triangle index.
6. Uzyj closest point barycentric coordinates i interpoluj wagi trzech
   reference vertices.
7. Odrzuc wplywy spoza `allowedBoneNodeIds` jako blocking gate; nie przenos ich
   automatycznie na parent.
8. Zmerguj te same bone ids, usun wartosci `<=0`, sortuj
   `(weight DESC, boneId ASC)`, zachowaj pierwsze cztery.
9. Suma przed normalizacja musi byc dodatnia i finite. Po normalizacji suma
   musi wynosic `1.0 +- 1e-5`.

Dla `RIGID` nie ma transferu wag: segment ma `weights=[]`, a raport liczy jego
vertices jako rigid. Brak poprawnego wplywu SKIN, nieistniejaca/niedozwolona
kosc, ujemna/NaN waga albo niejednoznaczna hierarchia blokuje creature.

## 10. Stable errors i gates

Fatal codes:

```text
M3A-PROFILE-JSON-INVALID
M3A-PROFILE-SCHEMA-UNSUPPORTED
M3A-PROFILE-HASH-MISMATCH
M3A-SOURCE-CONTRACT-MISMATCH
M3A-OPTIONS-INVALID
M3A-PROFILE-BOUNDS-INVALID
M3A-PROFILE-HIERARCHY-INVALID
M3A-PROFILE-SEGMENT-INVALID
M3A-NONFINITE-FLOAT
M3A-INTEGER-OVERFLOW
M3A-LIMIT-EXCEEDED
M3A-INTERNAL-CONTRACT
```

Report gate codes:

```text
M3A-SOURCE-BLOCKED
M3A-DEFAULT-SCENE-REQUIRED
M3A-DEFAULT-SCENE-HIERARCHY-INVALID
M3A-PROFILE-PROVENANCE-FORBIDDEN
M3A-SOURCE-RIG-DEFERRED
M3A-SOURCE-ANIMATION-DEFERRED
M3A-NORMALS-REQUIRED
M3A-TANGENT-COVERAGE-MIXED
M3A-MATERIAL-LIMIT
M3A-TRIANGLE-BUDGET-WARNING
M3A-TRIANGLE-BUDGET-BLOCKING
M3A-ZERO-HEIGHT
M3A-SEGMENT-ASSIGNMENT-FAILED
M3A-WEIGHT-BONE-FORBIDDEN
M3A-WEIGHT-SUM-INVALID
M3A-OUTPUT-BOUNDS-MISMATCH
```

Nonblocking diagnostic `M3A-SOURCE-MATERIAL-NAME-OMITTED` zapisuje odrzucenie
path-like/non-logical source material name bez kopiowania tej nazwy do message.

Severity domain jest zamknieta:

```text
fatal error: FATAL
report gate: WARNING | BLOCKING
report diagnostic: INFO | WARNING
```

`M3A-TRIANGLE-BUDGET-WARNING` jest `WARNING`; pozostale gate codes z listy sa
`BLOCKING`. Fatal nigdy nie jest wpisywany jako gate/diagnostic. Diagnostic nie
moze miec `BLOCKING` ani zmieniac `conversionEligible`.

Kazdy envelope ma `schemaVersion`, `code`, severity z powyzszej domeny,
logiczny `path` i message bez sciezek hosta. Deterministyczna kolejnosc gates:

1. phase `PROVENANCE`;
2. phase `M2_SOURCE_GATES`;
3. phase `DEFAULT_SCENE`;
4. phase `GEOMETRY_MATERIAL`;
5. phase `SEGMENT_WEIGHT`;
6. phase `OUTPUT`.

W fazie sort key to `(path UTF-8 bytewise, code UTF-8 bytewise, severityRank)`,
gdzie `WARNING=0`, `BLOCKING=1`; duplikaty exact code/path sa scalane. Kolejnosc
diagnostics to phase `SOURCE`, `MATERIAL`, `TRANSFORM`, `OUTPUT`, potem ten sam
`(path,code,severityRank)` z `INFO=0`, `WARNING=1`. Fatal validation source/API
wykonuje sie przed budowa obu list.

Source rig albo source animation w pierwszym nonhumanoid Profile A jest
blocking, a nie silent discard. Ten profil wymaga jawnego rig profile; obsluga
mapowania source rig/animations nalezy do pozniejszych etapow.

## 11. TDD, fixtures i gates

Minimalna macierz testowa:

- axis: bazowe X/Y/Z, asymetryczny marker przod/gora i `det=-1`;
- default scene only, unreachable mesh ignored, shared mesh duplicated per instance;
- nested MATRIX/TRS source-local -> world bake exactly once; output nodes == rig nodes;
- winding: dot transformed face normal z transformed source normal jest dodatni;
- normal/tangent through source-world, global and inverse rig-parent spaces;
- winding/tangent parity dla dodatnich i reflected source/rig transforms;
- UV: cztery opisane rogi i double-flip round-trip;
- MATRIX/TRS conjugation, nested hierarchy i world bounds;
- scale 1x, 2x, zero-height, non-finite target, bottom-center alignment;
- granice 5000/5001/10000/10001 triangles;
- material `null` PASS/binding slot 0, one id PASS, 2 unique keys BLOCKING;
- segment exact tie i tie-break po id;
- duplikacja vertexa na granicy segment/material;
- barycentric weights, duplicate bones, ponad 4 wplywy, zero sum, invalid bone;
- `SKIN` i `RIGID`;
- forbidden provenance/hash/exportAllowed;
- malformed/unknown JSON fatal vs valid REFERENCE_ONLY/UNKNOWN blocking;
- canonical profile hash: field order, omitted hash, shortest f32, `-0 -> 0`;
- target envelope/anchor/surface-bounds valid i invalid;
- exact distanceEvaluations boundary i limit+1 przed evaluation;
- source rig/animations obecne: blocking bez silent loss;
- brak normals: blocking;
- exact limits i limit+1, overflow, truncation/no-panic;
- native/WASM byte-identical deterministic JSON outcome;
- source IR, source report i source bytes niezmienione;
- serializacja bez host path/external payload.

Required commands przed `DONE`:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p m2a-wasm --target wasm32-unknown-unknown
wasm-pack test --node crates/m2a-wasm
git diff --check
```

## 12. Canonical read-only observations

Te metryki nie sa kopiowane do profilu i nie sa engine limits:

| witness | obserwacja own reader / canonical packet | rola M3 |
|---|---|---|
| `c_kocrachn` R1 | declared base nodes `66`, reachable base nodes `38`; 3 `extended64` skin nodes; map/q/t counts `38,38,38`; zero-weight lanes moga miec `0xffff` | struktura/segmentacja read-only; nie rig source |
| `c_kocrachn` current own-reader observation | `31` mesh nodes, `2009` vertices, `1458` triangles; 3 `extended64` skin nodes, 303 weighted vertices, max 2 influences | metryki z kanonicznego payloadu read-only; nie fixture/oracle ani zrodlo eksportowanego rigu |
| `c_vampire_f` R5 | reachable base nodes `28`; 2 `legacy17` skin nodes; map/q/t/constants `28,28`; nonzero bind pose observed | dowod, ze `17` jest width, nie capacity; nie rig source |

Poprawna aktywna interpretacja to count `38` dla `c_kocrachn` i `28` dla
`c_vampire_f`. Starsze twierdzenia `legacy17 => max 17`, `parser max 128` jako
limit engine albo mieszanie `c_drider` (`1238` triangles/`38` meshes) z
`c_kocrachn` sa zastapione tym zapisem.

## 13. Definition of Done M3

M3 moze przejsc na `DONE` dopiero, gdy:

- wszystkie decyzje tego suplementu maja implementacje i synthetic proof;
- raport nazywa basis/UV/winding/normal/tangent/scale/alignment/segment/weights;
- invalid geometry/skin daje exact stable gate/error;
- native i WASM sa deterministyczne i nie mutuja source;
- nie ma zewnetrznych assetow, szkieletow, animacji ani sciezek;
- wszystkie commands z sekcji 11 przechodza;
- `documentation/evidence/M3-evidence.md` zawiera exact liczniki;
- niezalezny final review konczy sie `NO FINDINGS`.

Realny Meshy GLB nie blokuje synthetic M3; jest wymaganiem M7. Runtime nie
blokuje implementacji M3, ale `engineFacingProof` i `uvRuntimeProof` pozostaja
jawnie `OPEN_M6` do wizualnego dowodu w NWN EE.
