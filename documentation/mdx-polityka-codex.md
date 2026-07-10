# Polityka MDX

Data: 2026-07-10 | Status: AKTYWNY KONTRAKT PROFILU A

## 1. Decyzja

Dla pierwszego profilu direct creature generujemy jeden zasob HAK:

```yaml
resource:
  type: 2002
  extension: "mdl"
  payload: "12-byte binary MDL header + core + appended volatile/MDX"
separate_type_2003_resource: false
```

Nazwa `MDX` oznacza tutaj volatile/raw block wskazany przez header binary MDL, a nie drugi plik wymagany przez HAK profilu A.

## 2. Lokalny dowod binarny

`C:\Users\enonw\Documents\Neverwinter Nights\hak\cep3_core1.hak` jest tylko read-only reference:

```yaml
container:
  signature: "HAK V1.0"
  entries: 6402
  type_2002_mdl_entries: 3517
  type_2003_entries: 0
c_kocrachn:
  resource_id: 724
  resource_size: 163192
  header_zero: 0
  core_size_raw_data_offset: 76048
  volatile_size_raw_data_size: 87132
  exact_check: "12 + 76048 + 87132 = 163192"
```

To rozstrzyga polozenie dla profilu A, ale nie dowodzi, ze kazda historyczna rodzina modelu NWN nigdy nie uzywa osobnego zasobu.

## 3. Semantyka offsetow

```text
file_header_start = 0
core_start        = 12
volatile_start    = 12 + raw_data_offset
file_end          = volatile_start + raw_data_size
```

- core offsets sa liczone wzgledem `core_start`;
- raw/volatile offsets sa liczone wzgledem `volatile_start`;
- `raw_data_offset` jest rozmiarem core, a nie absolutnym file offsetem;
- payload profilu A nie ma trailing bytes poza zadeklarowanym volatile blockiem.

## 4. Zawartosc profilu A

Volatile przechowuje duze tablice mesha, m.in. vertices, UV, normals, indices oraz - zalezne od profilu - tangents, handedness, colors, skin weights i bone references. Faces, controllers, node tree, animation headers i eventy pozostaja w core.

Writer nie moze przechowywac surowych pointerow procesu ani kopiowac runtime routine addresses z referencyjnego pliku. Kazdy offset jest wyliczany od nowa z deterministycznego layout planu.

## 5. Walidacja

```yaml
mdx_gates:
  structural:
    - "zero == 0"
    - "12 + raw_data_offset <= file_size"
    - "12 + raw_data_offset + raw_data_size == file_size"
    - "every core pointer is inside core"
    - "every raw pointer and array is inside volatile"
  semantic:
    - "vertex/index/UV/normal counts agree with the IR"
    - "skin arrays agree with vertex count and supported influence count"
  deterministic:
    - "same IR produces same SHA-256"
  runtime:
    - "generated HAK loads in Toolset and game"
```

## 6. Reopen rule

Polityke wolno otworzyc ponownie tylko dla nazwanego nowego profilu, gdy lokalny binary/resource evidence pokazuje inny uklad. Nie zmieniamy profilu A na podstawie komentarza w repozytorium, internetowego przykladu albo zachowania `aurora-web`.
