# P-REF: retailowe party itemu `WSwLs`

Data: 2026-07-29
Packet timestamp UTC: `2026-07-29T10:24:34Z`
Status: `STRUCTURAL_PACKET_PASS`
Źródło: `base_nwn`, odczyt in-place przez KEY/BIF
Payloady zewnętrzne w Git: `NONE`

## 1. Cel

Pakiet wiąże trzy dokładne modele wskazane przez retailowe:

```text
UTI nw_wswmls002
BaseItem=1
ItemClass=WSwLs
ModelType=2
ModelPart1/2/3=23/63/23
```

z raportem własnego readera `m2a-core::mdl`.

Pakiet nie twierdzi, że trzy modele dowodzą uniwersalnej obsługi wszystkich
itemów. Jest dokładnym witnessem profilu Bottom/Middle/Top i miejsca
transformacji partów.

## 2. Execution identity

```yaml
reader:
  name: m2a-core::mdl
  version: 0.1.0
  reportSchemaVersion: 1
execution:
  commandLabel: m2a-core-inspect-reference-range
  timestampUtc: 2026-07-29T10:24:34Z
source:
  class: base_nwn
  key: nwn_base.key
  container: models_02.bif
  access: read_in_place
storage:
  committedPayloads: []
  extractedPayloads: []
```

Logiczna komenda reprodukcji dla każdego wpisu:

```text
cargo run -q -p m2a-core --example inspect_mdl_range -- <NWN-EE>/data/models_02.bif <offset> <length>
```

Ścieżka hosta nie jest częścią identity packetu.

## 3. Exact input fingerprints

| Packet ID | Resref | KEY index | Offset | Length | SHA-256 |
|---|---|---:|---:|---:|---|
| `P-REF-ITEM-WSWLS-B-023` | `wswls_b_023` | 106720 | 77400701 | 4056 | `092b063692f5a427fe514931b1e950dec2fcfa8f58e4f78b406a12fc72a73459` |
| `P-REF-ITEM-WSWLS-M-063` | `wswls_m_063` | 106769 | 77663124 | 5828 | `ccb9ab1a92e25bcf9293f6fc30f566bc0e14e59d9f21625266d4ac7da3aa5ae0` |
| `P-REF-ITEM-WSWLS-T-023` | `wswls_t_023` | 106786 | 77769376 | 5456 | `1582ebe4d3d037ef262e9d691d5c5bbc65d447593f5f19aff07c61ccab9d91dc` |

Resource type w KEY i BIF dla wszystkich trzech: `2002`.

## 4. Own-reader summary

### 4.1 Bottom

```yaml
packetId: P-REF-ITEM-WSWLS-B-023
modelName: WSwLs_b_023
binaryMdlId: 0
byteLength: 4056
coreRange: { start: 12, length: 1872, end: 1884 }
rawRange: { start: 1884, length: 2172, end: 4056 }
nodes: { declared: 2, parsed: 2, meshes: 1 }
mesh:
  name: g_WSwLs_b_023
  position: [0.000183583, -0.14571, 0.000191967]
  orientation: [0, 0, 0, 1]
  vertices: 56
  triangles: 26
  boundsMin: [-0.0259236, -0.0568282, -0.0674395]
  boundsMax: [0.0259236, 0.229862, 0.0674396]
  texture0: W_metal_tex
animations: 0
unsupportedFamilies: []
diagnostics: []
```

### 4.2 Middle

```yaml
packetId: P-REF-ITEM-WSWLS-M-063
modelName: WSwLs_m_063
binaryMdlId: 0
byteLength: 5828
coreRange: { start: 12, length: 2576, end: 2588 }
rawRange: { start: 2588, length: 3240, end: 5828 }
nodes: { declared: 2, parsed: 2, meshes: 1 }
mesh:
  name: g_WSwLs_m_063
  position: [0.000394173, -0.0206093, 0.000168152]
  orientation: [0, 0, 0, 1]
  vertices: 82
  triangles: 48
  boundsMin: [-0.0262558, 0, -0.0974612]
  boundsMax: [0.025551, 0.17765, 0.0972681]
  texture0: W_metal_tex
animations: 0
unsupportedFamilies: []
diagnostics: []
```

### 4.3 Top

```yaml
packetId: P-REF-ITEM-WSWLS-T-023
modelName: WSwLs_t_023
binaryMdlId: 0
byteLength: 5456
coreRange: { start: 12, length: 2384, end: 2396 }
rawRange: { start: 2396, length: 3060, end: 5456 }
nodes: { declared: 2, parsed: 2, meshes: 1 }
mesh:
  name: g_WSwLs_t_023
  position: [0.0000303633, 0.262925, 0.0018867]
  orientation: [0, 0, 0, 1]
  vertices: 78
  triangles: 42
  boundsMin: [-0.0102512, -0.134606, -0.0479554]
  boundsMax: [0.010243, 0.668461, 0.0444966]
  texture0: W_metal_tex
animations: 0
unsupportedFamilies: []
diagnostics: []
```

## 5. Capability matrix

| Capability | Bottom | Middle | Top |
|---|---|---|---|
| Header | PASS | PASS | PASS |
| CoreRanges | PASS | PASS | PASS |
| NodeTree | PASS | PASS | PASS |
| Mesh | PASS | PASS | PASS |
| Skin | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT |
| Controllers | PASS | PASS | PASS |
| Animations | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT |
| Events | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT |
| UnsupportedNodeFamily | NOT_PRESENT | NOT_PRESENT | NOT_PRESENT |

## 6. Invariant results

Wszystkie poniższe invarianty mają status `PASS` dla każdego z trzech
packetów:

| Invariant | Expected | Actual |
|---|---|---|
| `binary-mdl-id-is-zero` | `0` | `0` |
| `payload-byte-length-exact` | fingerprint length | reader byteLength |
| `core-byte-length-exact` | header-declared range | reader-validated |
| `raw-byte-length-exact` | header-declared range | reader-validated |
| `file-header-core-raw-cover-payload` | exact chain to EOF | exact chain to EOF |
| `declared-node-count-equals-parsed` | `2` | `2` |
| `mesh-node-count-exact` | `1` | `1` |
| `part-position-controller-present` | decoded vec3 | decoded vec3 |
| `part-orientation-controller-present` | decoded quaternion | `[0,0,0,1]` |
| `reader-diagnostic-count-zero` | `0` | `0` |

## 7. Semantic result

```yaml
expectedModelClass: modular_item_part
observed:
  independentBinaryMdlPerSlot: PASS
  rootPlusTrimesh: PASS
  authoredPartTransformInMdl: PASS
  sameMaterialFamilyAcrossSet: PASS
  animationFreeStaticPart: PASS
verdict: PASS
```

Przestrzenne zakresy Y po zastosowaniu kontrolera child mesh:

```yaml
bottom: [-0.2025382, 0.084152]
middle: [-0.0206093, 0.1570407]
top: [0.128319, 0.931386]
bottomMiddleOverlap: PASS
middleTopOverlap: PASS
```

To potwierdza, że part placement nie pochodzi z UTI. Jest zapisany w node
controllers każdego MDL.

## 8. Preview applicability

```yaml
ownPreviewCapture:
  status: NOT_AVAILABLE_FOR_IN_PLACE_BIF_RANGE
  reason: >
    Aktualny publiczny audit command zwraca BinaryMdl InspectionReport, ale
    nie ma opublikowanego wejścia preview, które bez ekstrakcji retailowego
    payloadu montuje zakres KEY/BIF jako asset wizualny.
  claimBoundary: >
    Packet dowodzi structural reader compatibility i transform controllers.
    Nie rości wizualnego pixel-parity własnego preview, Toolsetu ani NWN.
```

Nie użyto screenshotu Toolsetu jako substytutu own-reader proof.

## 9. Icon namespace corroboration

Nie są to modele uruchomione przez reader i nie tworzą dodatkowych P-REF MDL.
Scan KEY potwierdził jednak osobny namespace:

| Resref | TGA type 3 | DDS type 2033 | MDL type 2002 |
|---|---:|---:|---:|
| `iwswls_b_023` | 16428 B | 5540 B | absent |
| `iwswls_m_063` | 16428 B | 5540 B | absent |
| `iwswls_t_023` | 16428 B | 5540 B | absent |

Wniosek: `i` jest prefiksem warstwy ikony, nie alternatywnego MDL.

## 10. Provenance statement

- retailowe zasoby były czytane wyłącznie in-place;
- nie zapisano ich zakresów do plików;
- do repo trafiają tylko locatory logiczne, hashe, raport summary, invarianty
  i komendy;
- packet nie zastępuje synthetic writer/readback ani owner-run Toolset/NWN
  proof własnego wygenerowanego itemu.
