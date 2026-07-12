# M4A evidence - animation writer

Data: 2026-07-13

```yaml
stage: M4A
attempt_id: M4A-20260712-01
status: READER_FIXED_M4A1_WRITER_NEXT
done: false
implementation_started: true
runtime_proof: OPEN_M6
canonical_payload_committed: false
```

## 1. Aktualny wynik

Aurora First contract-lock zostal zakonczony. Zamrozono layout AnimationHeader,
model animation ArrayDef, osobne animation root trees, controller key/data,
timing, interpolation, transition i events. Nie ma jeszcze implementacji M4A1
ani M4A2, dlatego dokument nie raportuje `DONE`.

Autorytatywny suplement:

- `documentation/m4a-animation-writer-kontrakt-suplement-codex.md`.

Historyczny M1B controller row zostal skorygowany z signed `i16`/calego byte
columns na `u16` oraz packed low/high nibble. Implementacyjny reader fix jest
pierwszym krokiem M4A1.

## 2. Read-only evidence sources

```yaml
decompilation:
  source_class: local_read_only_decompiled_aurora
  committed_payload: false
canonical:
  source_class: named_hak_in_place
  container_identity: cep3_core1
  resources:
    - id: R3a
      resref: c_phod_horror_b
      type: 2002
      sha256: 62ab1f512f709f9acd0fe0c5deb9bc65691277c848799d261086bc3d63b28f2a
    - id: R3b
      resref: c_phod_horror_p
      type: 2002
      sha256: 09e43ee9493d2fe2bbf9cbeb44f24dcb999e5f38e651bdc79eefdd5e1f19722f
  extraction_to_repo: false
  external_code_or_payload_copy: false
```

Probe byl read-only i in-place. Nie utworzyl fixture, subrange ani kopii HAK.

## 3. Decomp exact observations

| Capability | Observation | Anchors | Status |
|---|---|---|---|
| AnimationHeader | `0xc4`; geometry, length, transition, animroot, events | `864534-864541`, `871536-871545` | PASS |
| model animation array | `+0x78/+0x7c/+0x80`, entries `u32` core offsets | `862902-862955`, `871551-871572` | PASS |
| own root tree | separate tree from `animation+0x48` | `863021-863047` | PASS |
| target identity | root paired explicitly; children recursively by name | `889100-889146` | PASS |
| clip lookup | animation GeometryHeader name, then supermodel/default | `844151-844216` | PASS |
| controller arrays | keys `0x0c`, data `f32[]` | `843831-843852`, `863740-863779` | PASS |
| controller packing | rows/time/data `u16`; packed columns/flags | `882458-882566`, `882814-882826` | PASS |
| common types | position8/3, orientation20/4, scale36/1 | `882852-882975` | PASS |
| interpolation | endpoint clamp; linear; high nibble `0x10` Bezier | `844653-844897` | PASS |
| animationScale | type8 output is multiplied by current animationScale | `844795-844810` | PASS |
| quaternion | disk XYZW; shortest-path slerp; no explicit normalize | `844904-844990`, `1020552-1020628` | PASS |
| transition | seconds used as blend-in/out divisor; zero immediate | `843901-843947`, `846138-846155`, `846199-846211` | PASS |
| event stride/timing | `0x24`; forward/reverse/wrap windows | `844407-844458`, `886456-886582` | PASS |
| animroot consumer | binary ownership known; direct runtime consumer not found | n/a | OPEN_M6 |
| event name callbacks | time dispatch known; gameplay/audio mapping not proven | n/a | OPEN_M6 |

## 4. Canonical in-place R3 observations

```yaml
per_resource:
  payload_length: 846064
  core_length: 788416
  raw_length: 57636
  animation_array: { pointer: 232, used: 42, allocated: 42 }
  own_animation_count: 42
  event_count: 41
  node_count_per_animation: 27
  controller_count: 966
  controller_types: { position_8: 43, orientation_20: 923 }
  event_names:
    snd_footstep: 33
    hit: 6
    cast: 1
    snd_hitground: 1
```

Additional exact checks:

- 42 header offsets sa strict increasing, pierwszy `0x190`, ostatni `0xb1aec`;
- wszystkie animation trees zachowuja base node number/name/topology;
- base ma dwa nody `head` o numerach 14/15, co potwierdza hierarchical,
  nie global-name identity;
- per clip flags: 21 x `0x21`, 6 x `0x01`;
- wszystkie 882 animation mesh placeholders maja vertex/texture `0/0` i
  pusty faces ArrayDef;
- packed columns sa tylko `3/4`; key byte `+0x0b` jest zawsze zero;
- wszystkie 966 times arrays sa finite, strict, start `0`, end `length`;
- event arrays maja `used=allocated`, times sa ordered i in `0..length`;
- animation block nie posiada wlasnego raw geometry blocku.

## 5. Contract decisions locked

```yaml
M4A1:
  input: separate_versioned_MdlAnimationSetV1
  animroot_granularity: per_clip
  writer_paths: [TRANSLATION, ROTATION]
  interpolation: LINEAR_ONLY
  controller_indices: u16
  packed_columns: low_nibble
  packed_interpolation_flags: high_nibble
  quaternion_disk_order: XYZW
  quaternion_policy: finite_unit_and_sign_canonical
  controller_times: finite_strict_in_0_length
  event_order: stable_sort_by_time
  zero_events: allowed_structurally
  loop_bit: none
  animation_scale: 1.0
  animation_tree: complete_rig_only_dummy_mirror
  skin_and_mesh_segment_nodes: omitted
  opaque_runtime_fields: zero
  animroot: explicit_owned_input
  loader_smoke_motion_threshold: "translation >= 0.01 output unit or orientation >= 1 degree"
M4A2:
  required_before_stage_done: true
  api: convert_profile_a_with_animations_v1
  default_m3_policy_unchanged: true
  stage_private_policy: AllowMappedForM4A2_after_full_validation
  source_ir_or_report_rewrite: forbidden
  source: owned_or_user_provided_GLB_animation
  target: output_rig_local_Aurora_space
  paths: [translation, rotation]
  interpolation: LINEAR_ONLY
```

## 6. Stable blocker inventory

```yaml
structural_implementation_blockers: []
closed_contract_findings:
  - "M4A2 executable API locked without bypassing or rewriting the default M3 gates"
  - "animroot moved to each clip"
  - "empty set bypasses animation-only sibling validation"
  - "duplicate track and invalid event-time diagnostics locked"
implementation_work_required:
  - MdlAnimationSetV1 and M4A1 writer/readback
  - GLB animation to output-rig mapper M4A2
  - synthetic happy/negative/mutation matrix
  - native/WASM and frozen empty-path gates
open_m6:
  - M4A-DECOMP-ANIMROOT-CONSUMER
  - M4A-DECOMP-EVENT-NAME-SEMANTICS
  - M4A-RUNTIME-STATE-ROUTING
  - M4A-RUNTIME-OPAQUE-ZERO
  - M4A-RUNTIME-ANIM-TREE-PROFILE
```

## 7. Frozen M4 regression gate

M4 empty-animation output musi pozostac:

```yaml
payload_length: 1188
core_length: 1072
raw_length: 104
sha256: e100130d1dfbd18657413cdb7a701396d466cee081683591fc9836bf0c11b4b2
```

Ten hash jest gate'em regresji, nie dowodem runtime.

## 8. Required evidence before next status change

Po checkpointcie M4A1.1 nadal nie ma wynikow ponizszych gate'ow:

- M4A1 zero/one/multiple animation round-trip;
- stable fatal matrix i truncation-no-panic;
- exact pointer/core/raw/EOF report;
- frozen zero-animation SHA;
- M4A2 owned GLB mapper fixtures;
- native/public WASM byte identity;
- fmt, clippy, workspace, wasm32 i Docker quality;
- independent P1/P2 review.

Dlatego M4A pozostaje `IN_PROGRESS` ze statusem checkpointu
`READER_FIXED_M4A1_WRITER_NEXT`, nie `VERIFYING`, `DONE` ani `DONE_RUNTIME`.

## 9. Checkpoint M4A1.1 - reader u16 i packed flags

Data: 2026-07-13 | Status: READER_FIXED_M4A1_WRITER_NEXT

Reader nie interpretuje juz high-bit rows/time/data jako signed `i16`.
Controller packed byte jest rozdzielany na low-nibble columns i high-nibble
interpolation flags. Linear data jest dekodowane, a `0x10` pozostaje znanym,
deferred Bezier inventory z kontrola pelnego time range oraz minimalnego
data-index bound. Zero columns jest zawsze fatal. Unknown high bits sa
odrzucane stabilnym `M2A-MDL-CONTROLLER-LAYOUT-INVALID`.

```yaml
native:
  m2a_core_tests: 151
  workspace_tests: 153
  workspace_breakdown: "151 core + 2 additional workspace"
  mdl_tests: 40
wasm:
  node_tests: 14
canonical:
  r3_p_ref_runs: "1/1 PASS"
  animation_controllers: 966
  type_8: 43
  type_20: 923
  packed_3: 43
  packed_4: 923
  nonzero_interpolation_flags: 0
  decoded_false: 0
docker_no_cache:
  seconds: 124.4
  tag: m2a-quality:m4a-reader
  digest: sha256:2c8e9c44c349cea030ed919ce705d8c51c5563da0735f08b9d650779989d57bb
  size_bytes: 1195609328
final_review:
  p1: 0
  p2: 0
```

Canonical R3 zostal odczytany in-place przez own HAK locator/reader; payload
nie zostal wyekstrahowany ani zapisany. Publiczny WASM raport zamraza
`packedByte`, `interpolationFlags` i `decoded`. Existing M4 bind writer nadal
emituje linear packed `3/4`, a frozen zero-animation regresja pozostaje PASS.

Nastepny krok: M4A1.2 `MdlAnimationSetV1` writer/readback. Ten checkpoint nie
zamyka calego M4A ani zadnego runtime proof.
