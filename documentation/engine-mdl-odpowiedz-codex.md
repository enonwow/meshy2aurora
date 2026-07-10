# engine-mdl-odpowiedz-codex.md

Data: 2026-07-10 | Odpowiada na: `engine-mdl-pytania-cloud.md`

## Q1: Binary MDL jako format docelowy

Status: POTWIERDZONY KIERUNEK PROFILU A; CZESCIOWO OTWARTE EVIDENCE dla wariantu skin header i runtime acceptance.

Tak: odpowiedz musi wynikac z Aurora First. Docelowy layout binarnego MDL, wymagane offsety i pola nie sa pytaniem do Meshy ani do `aurora-web`. Aktywny inventory i podzial required/deferred/open znajduje sie w `mdl-binary-crosswalk-codex.md`; polityka raw/MDX jest w `mdx-polityka-codex.md`.

```yaml
question_owner: "Aurora/NWN binary format"
authoritative_first_source: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
supplementary_readonly_sources:
  - "C:\\Projects\\Claude\\nwnexplorer\\_NwnLib"
  - "C:\\Projects\\Claude\\xoreos-docs"
not_a_source_of_truth:
  - "Meshy"
  - "C:\\Projects\\aurora-web"
closure_condition:
  - "confirmed minimal writer field and offset inventory: DONE for direction"
  - "GB-001-SKIN 17/64 mapping variant resolved by M1B corpus report"
  - "own reader parses own emitted MDL"
  - "NWN EE Toolset and game proof accepts the generated asset"
```

### Stan Aurora First po audycie 2026-07-10

```yaml
decompilation_search:
  source: "C:\\Projects\\New Folder\\export"
  confirmed_strings:
    - "CResHelper<CResMDL,2002>"
    - "CResMDL"
    - "MdlNode"
    - "MdlNodeSkin"
    - "MdlNodeTriMesh"
    - "MdlNodeAnimMesh"
  direct_writer_field_names_found: false
  p_start_mdx_symbol_found: false
  conclusion: "The export confirms the MDL resource and runtime node families but not a named writer struct. Local binary plus independent parser cross-checks define the profile-A direction; only own readback and runtime proof can close evidence."
```

Plan zamkniecia pozostalej czesci GB-001:

```yaml
writer_contract_closure:
  direction_contract: "documentation/mdl-binary-crosswalk-codex.md"
  step_1: "M1A/M1B own reader records every supported field with provenance and unsupported diagnostics"
  step_2: "resolve GB-001-SKIN by checking both 0x2d4/17 and 64-map variants against local binary boundaries"
  step_3: "write a synthetic profile-A model from own IR; do not copy payloads"
  step_4: "parse it with the own reader and produce a semantic readback diff"
  step_5: "run the generated HAK through NWN EE Toolset and game"
  stop_condition: "if a required field has no Aurora/binary evidence, reject that feature/profile; do not invent its value"
```

## Q2: MDX - osobny zasob czy embedded

Status: POTWIERDZONE dla pierwszego profilu `direct creature` opartego na lokalnym `c_kocrachn`; NIE WIEM dla uniwersalnej polityki wszystkich rodzin modeli.

Lokalny, read-only HAK daje wystarczajacy kontrakt dla pierwszego profilu bez uzycia `aurora-web`:

```yaml
reference_container:
  path: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\cep3_core1.hak"
  signature: "HAK V1.0"
  total_entries: 6402
  mdl_type_2002_entries: 3517
  mdx_type_2003_entries: 0
c_kocrachn:
  key_entries:
    - { resref: "c_kocrachn", resource_type: 2002, resource_id: 724 }
  separate_type_2003_entry: false
  mdl_resource_size: 163192
  file_header:
    size: 12
    p_start_mdx: 76048
    size_mdx: 87132
  exact_size_check: "12 + 76048 + 87132 = 163192"
profile_A_decision:
  expected_hak_resources:
    - { resref: "m2a_<name>", resource_type: 2002, payload: "binary MDL followed by its MDX block" }
  separate_resource_type_2003: false
  writer_rule: "p_start_mdx is relative to the bytes after the 12-byte file header; size_mdx covers the appended block"
  readback_gate: "own reader must prove both ranges are in bounds and consume the exact emitted layout"
  runtime_gate: "NWN EE Toolset/game must still confirm the generated result in M6"
```

Wniosek implementacyjny: GB-002 jest rozstrzygniety dla profilu A i nie musi blokowac startu M4. Nie oznacza to zakazu typu 2003 dla kazdego przyszlego modelu; jesli inna rodzina zasobow dostarczy przeciwne Aurora First evidence, dostanie osobna polityke profilu.

## Q3: Podglad NwnExplorer a bind pose

Status: POTWIERDZONE tylko dla sklonowanej wersji NwnExplorer w `C:\Projects\Claude\nwnexplorer`; nie jest to fakt o loaderze Aurora.

Kolejnosc zrodel dla tego Q byla nastepujaca:

```yaml
aurora_first_check:
  source: "C:\\Projects\\New Folder"
  queries:
    - "nwnexplorer"
    - "bind pose"
    - "rest pose"
    - "animation frame"
  result: "No Aurora anchor describes behavior of the external NwnExplorer viewer."
  conclusion: "Aurora decompilation cannot by itself answer how a third-party viewer renders its window."
supplement_after_aurora_first:
  source: "C:\\Projects\\Claude\\nwnexplorer"
  purpose: "Verify NwnExplorer behavior only; never define the Aurora MDL writer or runtime contract."
```

Podglad nie odtwarza automatycznie `cpause1` ani innego wybranego klipu. Buduje runtime tree z bazowej geometrii `m_pGeometry` i kazde wywolanie renderera dostaje czas `0.0`. Timer tylko wywoluje repaint; nie zmienia czasu animacji ani nie wybiera klipu. Screenshot z tego podgladu przedstawia wiec base/rest pose modelu, a nie klatke odtwarzanej animacji.

```yaml
nwnexplorer_behavior:
  source: "C:\\Projects\\Claude\\nwnexplorer\\nwnexplorer\\ModelWnd.cpp"
  runtime_tree: "CreateNodes(m_sRes, pHeader->m_pGeometry.GetOffset(), NULL)"
  render_time_seconds: 0.0
  automatic_clip_playback: false
  timer_behavior: "repaint only"
  screenshot_reference: "base/rest pose"
  caveat: "This confirms this NwnExplorer source version, not the NWN EE runtime. Final asset proof remains Toolset and game."
```

Kotwice:

- `C:\Projects\Claude\nwnexplorer\nwnexplorer\ModelWnd.cpp:173-189` builds the tree from base `m_pGeometry`.
- `C:\Projects\Claude\nwnexplorer\nwnexplorer\ModelWnd.cpp:573-595` renders every layer at `0.0`.
- `C:\Projects\Claude\nwnexplorer\nwnexplorer\ModelWnd.h:145-151` timer only sends the redraw/context-change message.
