# M2 - AuroraAssetIR i GLB ingest contract supplement

Data: 2026-07-11 | Autor: Codex | Status: AKTYWNY KONTRAKT M2 v1; M2 DONE 2026-07-12

## 1. Cel, autorytet i granica etapu

M2 czyta niezaufane GLB 2.0 do czystego Rust `m2a-core`, buduje deterministyczny `GlbInspectionReport` i source-preserving `AuroraAssetIR`, a nastepnie wystawia ten sam wynik przez cienki adapter WASM. Kolejnosc autorytetu: aktywny plan/state, `PROJECT_RULES.md`, aktywna architektura Rust/WASM, publiczny kontrakt glTF 2.0 i wlasne synthetic fixtures.

M2 nie zgaduje docelowej transformacji Aurory. Po ingest:

```yaml
coordinate_policy:
  stored_space: GLTF_SOURCE
  source_up: POSITIVE_Y
  source_forward_convention: POSITIVE_Z
  source_handedness: RIGHT_HANDED
  source_units: METERS_DECLARED_BY_MESHY
  positions_normals_tangents: PRESERVED
  triangle_winding: PRESERVED
  uv0: PRESERVED_GLTF_VALUES
  target_transform_status: UNRESOLVED_M3
  target_up: POSITIVE_Z_REFERENCE_ANCHOR_ONLY
  target_forward: UNRESOLVED
  target_handedness: UNRESOLVED
  target_units: UNRESOLVED
  owner_of_axis_scale_uv_conversion: M3
```

Swap `[x,y,z] -> [x,z,y]`, UV `v -> 1-v`, bbox scaling, winding correction i normal/tangent transform sa M3-owned. M2 tylko zachowuje source values i raportuje probe results. Nazwa `AuroraAssetIR` oznacza wspolny kontrakt pipeline'u, nie to, ze M2 juz wykonal target transform.

## 2. GLB 2.0 subset v1

REQUIRED/SUPPORTED:

- binary GLB 2.0 z poprawnym headerem i JSON chunk;
- co najwyzej jeden BIN chunk; wszystkie buffer views musza wskazywac w-bounds;
- embedded BIN data; zewnetrzne buffer/image URI sa unsupported encoding w v1;
- scenes, root nodes i acykliczny node graph;
- source node transform zachowany jako `MATRIX` albo `TRS`, bez stratnej dekompozycji;
- dowolna liczba nodes/meshes/primitives w project limits;
- primitive `TRIANGLES`; indexed `UNSIGNED_BYTE/UNSIGNED_SHORT/UNSIGNED_INT` oraz non-indexed triangles materializowane deterministycznie jako sekwencyjne indeksy;
- `POSITION` `VEC3`; standardowe glTF component conversions do `f32` tylko zgodnie z normalized flag;
- opcjonalne `NORMAL VEC3`, `TANGENT VEC4`, `TEXCOORD_0 VEC2`, `JOINTS_0 VEC4`, `WEIGHTS_0 VEC4`;
- source PBR inventory: base color factor/texture, metallic/roughness, normal texture, emissive, alpha mode/cutoff i double-sided;
- embedded image identity i bytes reference tylko dla exact MIME `image/png` albo `image/jpeg`; report zawiera tylko mime/hash/length, nigdy payload;
- skin inventory: skeleton root, ordered joints, optional inverse bind matrices; brak accessor'a inverse bind matrices jest poprawny i daje pusta liste;
- animation inventory/data: name, samplers, channels, target node/path, interpolation, input times i output values w source basis;
- unknown optional extension zachowany w inventory jako diagnostic; unknown required extension jest fatal;
- deterministic ordering zachowuje source index order; stable IDs to zero-based source indices.

DEFERRED/UNSUPPORTED IN V1:

- `.gltf` JSON plus loose resources, FBX i inne formaty;
- sparse accessors;
- Draco, meshopt i inne compressed geometry extensions;
- morph targets/animation weights jako conversion data; ich obecnosc daje blocking gate, nie silent loss;
- cameras, punctual lights i custom extension semantics;
- material bake, texture decode/resize/TGA/TXI;
- decimation/remesh, skeleton retarget, weight transfer i animation-name mapping;
- target Aurora axis/scale/UV/winding transform;
- binary MDL/MDX, 2DA i HAK output.

## 3. Versioned schema v1

Wszystkie JSON nazwy sa `camelCase`; vectors/matrices to finite `f32`; IDs i counts sa nieujemne. Non-finite float jest fatal. Report i IR maja osobne schema versions, oba `1`.

```yaml
AuroraAssetIR:
  schemaVersion: 1
  source:
    format: GLB_2_0
    byteLength: usize
    sha256: lowercase_hex_64
    assetVersion: string
    generator: string_or_null
  coordinateSpace:
    storedSpace: GLTF_SOURCE
    up: POSITIVE_Y
    forwardConvention: POSITIVE_Z
    handedness: RIGHT_HANDED
    units: METERS_DECLARED_BY_MESHY
    positionsPolicy: PRESERVED
    uvPolicy: PRESERVED
    windingPolicy: PRESERVED
    targetTransformStatus: UNRESOLVED_M3
  defaultSceneId: u32_or_null
  scenes: [IrScene]
  nodes: [IrNode]
  meshes: [IrMesh]
  primitives: [IrPrimitive]
  materials: [IrMaterial]
  textures: [IrTexture]
  samplers: [IrSampler]
  images: [IrImageRef]
  skins: [IrSkin]
  animations: [IrAnimation]

IrScene:
  id: u32
  name: string_or_null
  rootNodeIds: [u32]

IrNode:
  id: u32
  name: string_or_null
  childIds: [u32]
  parentIds: [u32]
  transform:
    kind: MATRIX | TRS
    matrix: f32x16_or_null
    translation: f32x3_or_null
    rotation: f32x4_or_null
    scale: f32x3_or_null
  meshId: u32_or_null
  skinId: u32_or_null

IrMesh:
  id: u32
  name: string_or_null
  primitiveIds: [u32]

IrPrimitive:
  id: u32
  sourceMeshId: u32
  sourcePrimitiveIndex: u32
  topology: TRIANGLES | OTHER
  materialId: u32_or_null
  positions: [f32x3]
  normals: [f32x3]_or_empty
  tangents: [f32x4]_or_empty
  uv0: [f32x2]_or_empty
  joints0: [u16x4]_or_empty
  weights0: [f32x4]_or_empty
  indices: [u32]
  boundsMin: f32x3
  boundsMax: f32x3
  sourceWasIndexed: bool

IrMaterial:
  id: u32
  name: string_or_null
  baseColorFactor: f32x4
  baseColorTexture: IrTextureBinding_or_null
  metallicFactor: f32
  roughnessFactor: f32
  metallicRoughnessTexture: IrTextureBinding_or_null
  normalTexture: IrTextureBinding_or_null
  emissiveFactor: f32x3
  emissiveTexture: IrTextureBinding_or_null
  alphaMode: OPAQUE | MASK | BLEND
  alphaCutoff: f32_or_null
  doubleSided: bool

IrTexture:
  id: u32
  sourceImageId: u32
  samplerIndex: u32_or_null

IrSampler:
  id: u32
  name: string_or_null
  magFilter: NEAREST | LINEAR | null
  minFilter: NEAREST | LINEAR | NEAREST_MIPMAP_NEAREST | LINEAR_MIPMAP_NEAREST | NEAREST_MIPMAP_LINEAR | LINEAR_MIPMAP_LINEAR | null
  wrapS: CLAMP_TO_EDGE | MIRRORED_REPEAT | REPEAT
  wrapT: CLAMP_TO_EDGE | MIRRORED_REPEAT | REPEAT

IrTextureBinding:
  textureId: u32
  texCoordSet: u32

IrImageRef:
  id: u32
  name: string_or_null
  mimeType: string
  byteOffset: usize
  byteLength: usize
  sha256: lowercase_hex_64
  payloadEmbeddedInJson: false

IrSkin:
  id: u32
  name: string_or_null
  skeletonRootNodeId: u32_or_null
  jointNodeIds: [u32]
  inverseBindMatrices: [[f32;16]]_or_empty

IrAnimation:
  id: u32
  name: string_or_null
  durationSeconds: f32
  samplers: [IrAnimationSampler]
  channels: [IrAnimationChannel]

IrAnimationSampler:
  id: u32
  interpolation: LINEAR | STEP | CUBICSPLINE
  inputTimesSeconds: [f32]
  outputAccessorType: SCALAR | VEC2 | VEC3 | VEC4 | MAT2 | MAT3 | MAT4
  outputValues: flattened_f32

IrAnimationChannel:
  samplerId: u32
  targetNodeId: u32
  targetPath: TRANSLATION | ROTATION | SCALE | WEIGHTS
```

`inverseBindMatrices` jest puste, gdy `skin.inverseBindMatrices` nie wystepuje. Gdy accessor wystepuje, musi byc non-normalized `FLOAT/MAT4`, a jego `count` musi byc co najmniej liczba `jointNodeIds`. Kazde `f32x16` jest zachowane jako plaska macierz glTF w kolejnosci column-major; `inverseBindMatrices[i]` odpowiada `jointNodeIds[i]`. Nadmiarowe macierze sa poprawne wedlug glTF, ale nie maja mapowania do jointa i nie zwiekszaja `jointReferenceCount`.

`IrSampler` zachowuje jawne filtry source. Brak `magFilter` albo `minFilter` pozostaje `null` i M2 nie zgaduje implementacyjnego filtra renderera. Brak `wrapS` lub `wrapT` jest canonicalizowany zgodnie z glTF do `REPEAT`. Gdy tekstura nie wskazuje samplera, `IrTexture.samplerIndex=null`; M2 nie tworzy sztucznego wpisu `IrSampler`, a downstream stosuje glTF default sampler semantics: oba filtry unspecified i oba wrap modes `REPEAT`.

`IrAnimationSampler` zachowuje typ accessor'a wyjsciowego, a nie znaczenie kanalu. Znaczenie `TRANSLATION | ROTATION | SCALE | WEIGHTS` nalezy wylacznie do `IrAnimationChannel.targetPath`, poniewaz jeden sampler moze byc referencjonowany przez kanal. Wszystkie trzy tryby `LINEAR`, `STEP` i `CUBICSPLINE` sa REQUIRED w M2. Brak source `interpolation` oznacza wymagany przez glTF default `LINEAR`; inna wartosc jest fatal.

`inventory.keyframeCount` jest suma `inputTimesSeconds.len()` wszystkich samplerow, niezaleznie od liczby kanalow odwolujacych sie do samplera. `IrAnimation.durationSeconds` jest maksimum ostatniego czasu wszystkich niepustych samplerow tej animacji; animacja bez samplerow ma `0.0`. Kanal `WEIGHTS` pozostaje deferred i zawsze dodaje `M2A-GLB-ANIMATION-WEIGHTS-DEFERRED` o severity `BLOCKING`; nie jest silently pomijany ani traktowany jak TRS.

`GlbInspectionReport`:

```yaml
schemaVersion: 1
format: GLB_2_0
input: { byteLength, sha256 }
coordinatePolicy: same_summary_as_ir
inventory:
  sceneCount: usize
  nodeCount: usize
  meshCount: usize
  primitiveCount: usize
  materialCount: usize
  textureCount: usize
  samplerCount: usize
  imageCount: usize
  skinCount: usize
  jointReferenceCount: usize
  animationCount: usize
  keyframeCount: usize
statistics:
  vertexCount: usize
  indexCount: usize
  triangleCount: usize
  boundsMin: f32x3_or_null
  boundsMax: f32x3_or_null
  primitivesMissingNormals: usize
  primitivesMissingUv0: usize
  nonTrianglePrimitives: usize
gates: [GlbGate]
diagnostics: [GlbDiagnostic]
```

`GlbGate` ma `{code,severity,path,expected,actual,message}`. `GlbDiagnostic` ma `{schemaVersion:1,code,severity,byteOffset?,jsonPath?,message}`. JSON report/IR nie zawiera image/BIN payloadu, host path ani source filename; in-memory ingest result moze trzymac borrowed/owned binary asset store oddzielnie od serializowanego reportu.

## 4. Project limits v1

To sa guardrails `meshy2aurora`, nie fakty ani limity Aurory:

```yaml
GlbLimitsV1:
  maxInputBytes: 67108864          # 64 MiB
  maxJsonChunkBytes: 16777216      # 16 MiB
  maxNodes: 100000
  maxNodeDepth: 512
  maxMeshes: 100000
  maxPrimitives: 100000
  maxAccessors: 100000
  maxBufferViews: 100000
  maxVertices: 1000000
  maxIndices: 3000000
  maxDecodedGeometryBytes: 268435456 # 256 MiB WASM memory guardrail
  maxImages: 10000
  maxMaterials: 10000
  maxTextures: 10000
  maxSamplers: 10000
  maxSingleImageBytes: 33554432     # 32 MiB
  maxTotalImageBytes: 67108864      # 64 MiB
  maxSkins: 10000
  maxJoints: 100000
  maxAnimations: 10000
  maxAnimationSamplers: 100000
  maxAnimationChannels: 100000
  maxKeyframes: 1000000
  maxDecodedSkinAnimationBytes: 67108864 # 64 MiB WASM memory guardrail
  maxDiagnostics: 2048
  triangleWarningAbove: 5000
  triangleBlockingAbove: 10000
```

Kazde count*stride, offset+length, accessor range, index conversion i allocation estimate jest checked przed alokacja. `maxDecodedSkinAnimationBytes` obejmuje skumulowany koszt zdekodowanych joint IDs, weights, inverse-bind matrices, animation input times i output values w calym dokumencie; wspoldzielony accessor jest liczony raz wedlug rzeczywistej materializacji. Limit jest akceptowany na granicy i odrzucany dopiero po przekroczeniu. Te wartosci sa project/WASM guardrails, nie faktami ani limitami Aurory.

## 5. Fatal errors i nonfatal gates

Fatal oznacza brak zaufanego IR/reportu poza stable error envelope `{schemaVersion:1,code,message,byteOffset?,jsonPath?}`:

- `M2A-GLB-INPUT-EMPTY`;
- `M2A-GLB-INPUT-LIMIT-EXCEEDED`;
- `M2A-GLB-HEADER-INVALID`;
- `M2A-GLB-VERSION-UNSUPPORTED`;
- `M2A-GLB-LENGTH-MISMATCH`;
- `M2A-GLB-CHUNK-INVALID`;
- `M2A-GLB-JSON-INVALID`;
- `M2A-GLB-BIN-MISSING`;
- `M2A-GLB-EXTERNAL-URI-UNSUPPORTED`;
- `M2A-GLB-REQUIRED-EXTENSION-UNSUPPORTED`;
- `M2A-GLB-COMPRESSION-UNSUPPORTED`;
- `M2A-GLB-SPARSE-ACCESSOR-UNSUPPORTED`;
- `M2A-GLB-IMAGE-MIME-UNSUPPORTED`;
- `M2A-GLB-BUFFER-VIEW-OOB`;
- `M2A-GLB-ACCESSOR-OOB`;
- `M2A-GLB-ACCESSOR-LAYOUT-INVALID`;
- `M2A-GLB-NODE-CYCLE`;
- `M2A-GLB-NONFINITE-FLOAT`;
- `M2A-GLB-LIMIT-EXCEEDED`;
- `M2A-GLB-INTEGER-OVERFLOW`.

### 5.1 Exact validation matrix dla skin i animation (slice E)

| Obszar | Warunek wymagany | Wynik naruszenia |
|---|---|---|
| skin joints | `joints` istnieje, jest niepuste, miesci sie w `maxJoints`, a kazdy indeks wskazuje istniejacy node | fatal `M2A-GLB-ACCESSOR-LAYOUT-INVALID` dla pustej/niepoprawnej struktury lub `M2A-GLB-LIMIT-EXCEEDED` dla limitu |
| skin skeleton | brak jest poprawny; obecny indeks wskazuje istniejacy node | fatal `M2A-GLB-ACCESSOR-LAYOUT-INVALID` |
| inverse bind absent | brak `inverseBindMatrices` jest poprawny; IR zawiera `[]` | accepted, bez gate'a |
| inverse bind layout | obecny accessor: `componentType=FLOAT`, `type=MAT4`, `normalized=false`, finite values | fatal `M2A-GLB-ACCESSOR-LAYOUT-INVALID` albo `M2A-GLB-NONFINITE-FLOAT` |
| inverse bind count | obecny accessor ma `count >= joints.len()` | fatal `M2A-GLB-ACCESSOR-LAYOUT-INVALID` |
| animation sampler refs | kazdy channel wskazuje istniejacy sampler; kazdy target node wskazuje istniejacy node | fatal `M2A-GLB-ACCESSOR-LAYOUT-INVALID` |
| animation input | non-normalized `FLOAT/SCALAR`, `count > 0`, wszystkie czasy finite, `>= 0` i strict increasing | fatal `M2A-GLB-ACCESSOR-LAYOUT-INVALID` albo `M2A-GLB-NONFINITE-FLOAT` |
| interpolation | brak pola canonicalizuje sie do `LINEAR`; jawnie dozwolone sa tylko `LINEAR`, `STEP`, `CUBICSPLINE` | fatal `M2A-GLB-ACCESSOR-LAYOUT-INVALID` |
| TRS output type | translation/scale: non-normalized `FLOAT/VEC3`; rotation: non-normalized `FLOAT/VEC4`; wszystkie values finite | fatal `M2A-GLB-ACCESSOR-LAYOUT-INVALID` albo `M2A-GLB-NONFINITE-FLOAT` |
| TRS output count | `LINEAR/STEP`: output `count == input.count`; `CUBICSPLINE`: output `count == checked(input.count * 3)` | fatal `M2A-GLB-ACCESSOR-LAYOUT-INVALID` albo `M2A-GLB-INTEGER-OVERFLOW` |
| weights channel | poprawna referencja jest inventory data, ale conversion semantics sa deferred | report z exact `M2A-GLB-ANIMATION-WEIGHTS-DEFERRED`, `BLOCKING` |
| aggregate counts | laczne samplery, kanaly i keyframes nie przekraczaja odpowiednich limitow | fatal `M2A-GLB-LIMIT-EXCEEDED` przed alokacja |
| decoded budget | checked suma skin/animation decoded bytes nie przekracza `maxDecodedSkinAnimationBytes` | fatal `M2A-GLB-LIMIT-EXCEEDED` przed alokacja |

Walidacja zaleznosci output type/count jest wykonywana per channel, bo `targetPath` nie nalezy do samplera. Jezeli wspoldzielony sampler jest uzyty przez kanaly o niezgodnych wymaganiach, ingest jest fatal zamiast wybierac pierwsza interpretacje. Wszystkie odwolania i mnozenia sa checked przed indeksowaniem lub alokacja.

Poprawnie zakodowany GLB zawsze zwraca report, nawet gdy nie kwalifikuje sie do konwersji. Blocking gates:

- `M2A-GLB-POSITION-MISSING`;
- `M2A-GLB-UV0-MISSING`;
- `M2A-GLB-PRIMITIVE-MODE-UNSUPPORTED`;
- `M2A-GLB-GEOMETRY-OVER-BUDGET` dla triangles > 10000;
- `M2A-GLB-MORPH-TARGETS-DEFERRED`;
- `M2A-GLB-ANIMATION-WEIGHTS-DEFERRED`;
- `M2A-GLB-ATTRIBUTE-COUNT-MISMATCH`;
- `M2A-GLB-INDEX-OOB`;
- `M2A-GLB-INCOMPLETE-TRIANGLES`;
- `M2A-GLB-DEGENERATE-TRIANGLES`.

Warnings:

- `M2A-GLB-GEOMETRY-WARNING` dla triangles > 5000 i <= 10000;
- `M2A-GLB-NORMALS-MISSING`;
- `M2A-GLB-BASECOLOR-TEXTURE-MISSING`;
- `M2A-GLB-OPTIONAL-EXTENSION-IGNORED`;
- `M2A-GLB-SKIN-INFLUENCE-COUNT` gdy source wymaga wiecej niz czterech lanes;
- `M2A-GLB-TARGET-TRANSFORM-UNRESOLVED` zawsze w M2 source-preserving report.

Severity order to `INFO < WARNING < BLOCKING`; packet jest `conversionEligible=true` tylko bez `BLOCKING`. Fatal error nie jest gate'em.

## 6. Native i WASM API

```rust
pub fn inspect_glb(
    input: &[u8],
    limits: &GlbLimits,
) -> Result<GlbInspectionReport, GlbFatalError>;

pub fn ingest_glb(
    input: &[u8],
    limits: &GlbLimits,
) -> Result<GlbIngestResult, GlbFatalError>;

pub struct GlbIngestResult {
    pub schema_version: u32,
    pub ir: AuroraAssetIr,
    pub report: GlbInspectionReport,
}
```

Publiczne WASM v1:

```text
inspect_glb_json(bytes: &[u8]) -> deterministic JSON report or stable error envelope
ingest_glb_json(bytes: &[u8]) -> deterministic JSON {ir,report} or stable error envelope
```

Adapter nie przyjmuje sciezki, nie czyta DOM/filesystem/network, nie duplikuje logiki GLB i przekazuje bytes raz na operacje. Native oraz WASM JSON sa byte-identical dla tych samych bytes/default limits. Source slice nie jest mutowany.

## 7. Synthetic fixture plan A-F - PASS

- **A Minimal**: jeden scene/root/node, jeden indexed triangle POSITION, UV0, normal, material bez image; stable report/IR.
- **B Axis/winding**: asymetryczne osie/strzalka, dodatnie X/Y/Z markery, znany CCW triangle i normals; dowodzi source preservation oraz `UNRESOLVED_M3`, bez target swapu.
- **C UV corners**: quad z czterema rozroznialnymi UV corners; dowodzi, ze M2 zapisuje dokladnie source `[u,v]`, bez `1-v`.
- **D Material/image**: dwa primitives/materials, embedded minimal PNG, basecolor/PBR/sampler links; report zawiera hash/length, nie image bytes.
- **E Skin/animation - PASS**: joints, inverse bind matrices (wariant absent/empty i obecny `FLOAT/MAT4` column-major), JOINTS_0/WEIGHTS_0 oraz translation/rotation/scale channels dla `LINEAR`, `STEP` i `CUBICSPLINE`; inventory i values pozostaja w source basis. Osobny weights channel dowodzi exact `BLOCKING` gate.
- **F Gates**: builder variants missing POSITION, missing UV0, LINES primitive, 5001 i 10001 triangles, morph target oraz mismatched attributes; parse zwraca report z exact warning/blocking gates.

Fixtures sa generowane przez wlasny builder i commitowane jako kod/test data tylko wtedy, gdy sa w pelni synthetic. Real Meshy GLB jest opcjonalnym env-gated smoke i nie blokuje M2.

## 8. Required negative matrix

- empty, every truncated prefix, bad magic/version/declared length;
- duplicate/missing/unaligned chunks, invalid UTF-8/JSON;
- absent BIN, external URI, unknown required extension, compression i sparse accessor;
- buffer view/accessor offset, stride, count, component/type i arithmetic OOB;
- invalid node reference, cycle i depth/count limits;
- invalid image range and per-image/total limits;
- embedded image MIME missing albo inne niz exact `image/png`/`image/jpeg`; material/texture/sampler count limits; sampler enum/default preservation;
- vertex/index/decoded-byte limits; index beyond POSITION count;
- non-finite transforms, positions, normals, UV, weights i animation values;
- animation input nie-`FLOAT/SCALAR`, empty, negative, duplicate/descending/nonfinite time; invalid sampler/channel/node refs; unsupported interpolation; TRS output component/type/count mismatch dla `LINEAR`, `STEP` i `CUBICSPLINE`; sampler/channel/keyframe limits;
- `keyframeCount` przy samplerze wspoldzielonym przez wiele kanalow jest liczony raz; duration bierze max last input time, nie liczbe kanalow ani pierwszy sampler;
- weights channel daje exact `M2A-GLB-ANIMATION-WEIGHTS-DEFERRED` `BLOCKING` bez silent loss;
- skin empty joints, invalid joint/skeleton refs; inverse-bind absent accepted empty; obecny IBM wrong component/type/normalized, count ponizej joints, nonfinite matrix; joint limit;
- `maxDecodedSkinAnimationBytes`: exact boundary accepted, boundary+1 rejected przed alokacja; arithmetic overflow ma `M2A-GLB-INTEGER-OVERFLOW`;
- arbitrary bytes and all fixture truncations never panic;
- input bytes unchanged before/after native and WASM operations;
- deterministic ordering/JSON across repeated native and WASM runs.

## 9. Definition of Done M2

M2 moze przejsc do `DONE` tylko gdy:

1. schema v1, default limits, fatal errors i gates sa publiczne i przetestowane;
2. fixtures A-F sa green i nie zawieraja zewnetrznych payloadow;
3. native `inspect_glb` i `ingest_glb` zwracaja deterministic source-preserving report/IR;
4. axis/UV/winding/unit probes dowodza braku target transform w M2;
5. inventory nodes/primitives/materials/textures/skins/animations jest kompletne dla subsetu v1;
6. missing UV, nontriangle i budget warning/blocking maja exact stable gates;
7. full negative/limit/truncation/no-panic matrix przechodzi;
8. publiczne `inspect_glb_json` i `ingest_glb_json` sa rzeczywiscie wywolane przez `wasm-pack test --node` i daja byte-identical JSON;
9. source bytes pozostaja niezmienione;
10. JSON nie zawiera image/BIN payloadu ani prywatnych host paths;
11. `cargo fmt --all -- --check`, clippy `-D warnings`, `cargo test --workspace`, WASM build, `wasm-pack test --node` i `git diff --check` przechodza;
12. evidence zapisuje exact test counts/commands, changed files, provenance i brak zewnetrznych binaries;
13. niezalezny final review po wszystkich fixes nie ma findings.

M2 `DONE` nie rozstrzyga target Aurora transform. M3 przyjmuje `targetTransformStatus=UNRESOLVED_M3` jako jawna zaleznosc i zamyka axis/scale/UV/winding dla wybranego profilu.

## 10. Stan weryfikacji 2026-07-12

Implementacja kontraktu A-F jest zamknieta funkcjonalnie i pozostaje source-preserving: M2 nie wykonuje target axis/scale/UV/winding transform. Material/image, skin/IBM, animacje, fatal errors, warning/blocking gates, limity, truncation oraz no-panic matrix sa pokryte testami. Slice D i E przeszly niezalezne review bez findings. Fixture/gate slice F ma wynik `28/28 PASS`; dodane sa jawne blocking gates `M2A-GLB-INCOMPLETE-TRIANGLES` oraz `M2A-GLB-DEGENERATE-TRIANGLES`, a warning gates pozostaja osobno weryfikowane. Finalna poprawka helpera `POSITION` usuwa rozjazd testowego wariantu bez pozycji bez oslabiania produkcyjnej walidacji.

```yaml
verification:
  glb_fixture_and_negative_tests: "28/28 PASS"
  native_workspace_tests: "96 PASS"
  wasm_node_tests: "12 PASS"
  cargo_fmt: PASS
  cargo_clippy_D_warnings: PASS
  wasm_target_build: PASS
  git_diff_check: PASS
  independent_review_D: "NO FINDINGS"
  independent_review_E: "NO FINDINGS"
  independent_review_final_full_scope: "NO FINDINGS"
  docker_standard_quality: PASS
  docker_no_cache_quality: PASS
  docker_final_image:
    tag: "m2a-quality:final"
    digest: "sha256:9f84561c7271968bfb8de9997d97e33360e1765e217620897c404af236f6b620"
    inspect_size_bytes: 1067266351
    docker_cli_virtual_size: "4.43GB"
    build_context: "212.69kB"
    build_elapsed: "137.5s"
  docker_source_identity:
    host_glb_rs_sha256: "029f9c41319dda5b32a6bd33ae19cba9dedda8b1e2d5e7a50540190a4a11e2fa"
    image_glb_rs_sha256: "029f9c41319dda5b32a6bd33ae19cba9dedda8b1e2d5e7a50540190a4a11e2fa"
    result: PASS
  docker_content_scan: "PASS - no retail/CEP assets, secrets or absolute host paths"
stage_status: DONE
remaining_gate: null
```

Finalny niezalezny review oraz skan spojnosci dokumentacji zakonczyly sie wynikiem `NO FINDINGS`. Wszystkie punkty Definition of Done M2 maja evidence; M2 ma status `DONE`, a jawnie nierozstrzygniety target axis/scale/UV/winding przechodzi jako obowiazkowy zakres M3, nie jako domysl M2.
