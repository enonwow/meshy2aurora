# aurora-hak-erf-codex.md
Data: 2026-07-08  
Status: AKTYWNA REFERENCJA. Produkcyjny kontrakt Rust writerow i proofu jest w `hak-2da-gff-crosswalk-codex.md`.

## Zakres

Dokument opisuje format ERF/HAK, lokalne typy zasobow, istniejacy kod odczytu/zapisu i minimalny sklad HAK dla custom creature.

## Zrodla

```yaml
primary_sources:
  decompiled_aurora:
    path: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
    anchors:
      hak_signature: "8477, 12637, 122308, 200965"
      erf_signature: "15430, 260457"
      restype_strings: "16141, 16150, 21816, 21822, 273869, 390269, 390417, 392043, 392191"
  aurora_web_erf_reader:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\nwn\\nwn-erf-packed-module-extractor.service.ts"
    anchors:
      constants_and_signatures: "9-16"
      resource_extensions: "16-58"
      extract_container: "70-163"
      read_resource_index: "172-243"
      read_resource_from_index: "245-271"
      read_resource: "273+"
  aurora_web_resource_types:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\application\\nwn-key-bif-resource-reader.ts"
    anchors:
      resource_type_constants: "4-25"
  aurora_web_erf_writer_examples:
    - path: "C:\\Projects\\aurora-web\\backend\\scripts\\create-vfx-catalog-lab-map.mjs"
      anchors: "130-168"
    - path: "C:\\Projects\\aurora-web\\backend\\scripts\\inject-vfx-catalog-lab-compiled-scripts.mjs"
      anchors: "79-118"
internet_supplement:
  - "https://nwn.fandom.com/wiki/ERF"
  - "https://forums.beamdog.com/discussion/67106/no-erf-folder"
  - "https://github.com/xoreos/xoreos-docs/blob/master/specs/bioware/ERF_Format.pdf"
  - "https://nwntools.sourceforge.net/"
```

## Sygnatury i wersje

Status: POTWIERDZONE.

`aurora-web` traktuje jako poprawne kontenery:

```yaml
valid_erf_signatures:
  - "ERF "
  - "HAK "
  - "MOD "
  - "NWM "
supported_versions:
  - "V1.0"
  - "V1.1"
header_size_bytes: 160
key_entry_size:
  V1_0: 24
  V1_1: 40
resref_length:
  V1_0: 16
  V1_1: 32
resource_entry_size_bytes: 8
```

Internet potwierdza, ze HAK jest praktycznie kontenerem ERF z inna sygnatura/rozszerzeniem, ale w tym projekcie traktujemy to jako uzupelnienie, nie jako primary source.

## Layout ERF V1.0

Status: POTWIERDZONE przez czytnik i writer-przyklady.

```yaml
erf_v10_layout:
  header:
    size: 160
    signature: "bytes 0..3"
    version: "bytes 4..7"
    entry_count_offset: 16
    localized_string_offset_offset: 20
    key_list_offset_offset: 24
    resource_list_offset_offset: 28
  key_list:
    entry_size: 24
    fields:
      resref: "16 bytes ascii, nul padded"
      resource_id: "uint32"
      resource_type: "uint16"
      unused: "uint16"
  resource_list:
    entry_size: 8
    fields:
      file_offset: "uint32"
      file_size: "uint32"
  payload:
    compression: "none in local writer/readers"
```

Prosty writer `buildErfV10FromEntries` w `aurora-web`:

```yaml
writer_example:
  status: POTWIERDZONE
  behavior:
    - "kopiuje oryginalny header"
    - "ustawia entry count i offsety"
    - "pisze key table"
    - "pisze resource table"
    - "dokleja payloady"
    - "blokuje resref > 16 znakow"
  limitation:
    - "to funkcja w skryptach, nie wydzielony reusable pakiet"
    - "zaklada ERF V1.0 i oryginalny header"
```

## Typy zasobow przydatne dla creature

Status: POTWIERDZONE w lokalnym kodzie.

```yaml
resource_types:
  BMP: 1
  TGA: 3
  WAV: 4
  PLT: 6
  BMU: 8
  MP3: 11
  MDL: 2002
  SET: 2013
  WOK: 2016
  TWO_DA: 2017
  TXI: 2022
  UTI: 2025
  UTC: 2027
  ITP: 2030
  DDS: 2033
  UTP: 2044
  UTM: 2051
  DWK: 2052
  PWK: 2053
  MTR: 2072
extra_seen_in_erf_reader:
  NSS: 2009
  NCS: 2010
  ARE: 2012
  IFO: 2014
  BIC: 2015
  GIT: 2023
  DLG: 2029
  UTD: 2042
  GIC: 2046
  SSF: 2060
```

## Minimalny HAK dla direct creature

Status: HIPOTEZA wdrozeniowa oparta na potwierdzonych readerach i `appearance.2da`.

```yaml
minimal_creature_hak:
  signature: "HAK "
  version: "V1.0"
  resources:
    - resref: "appearance"
      extension: "2da"
      type: 2017
      required: true
      note: "zawiera wiersz z RACE=<model_resref>, MODELTYPE=S"
    - resref: "<model_resref>"
      extension: "mdl"
      type: 2002
      required: true
      note: "native binary MDL direct creature; profil A ma appended volatile/MDX w tym samym type-2002 payload"
    - resref: "<texture_resref>"
      extension: "tga|dds|plt"
      type: "3|2033|6"
      required: true
      note: "co najmniej diffuse texture, jesli MDL ma bitmap/texture0"
    - resref: "<texture_resref>"
      extension: "txi"
      type: 2022
      required: false
      note: "opcje tekstury"
    - resref: "<material_resref>"
      extension: "mtr"
      type: 2072
      required: false
      note: "material EE; aurora-web ma czesciowe wsparcie material metadata"
    - resref: "<creature_template>"
      extension: "utc"
      type: 2027
      required: false_for_asset_pack
      note: "potrzebne, jesli HAK ma dostarczac gotowy blueprint/template"
```

Kontrakt nazewniczy dla MVP:

```yaml
resref_rules:
  erf_v10_max_chars: 16
  charset: "ascii lower-case recommended"
  model_example: "m2a_koc01"
  files:
    mdl: "m2a_koc01.mdl"
    texture: "m2a_koc01.tga"
```

## Ranking / priorytet zrodel

Status: POTWIERDZONE dla lokalnego `aurora-web`; NIE WIEM dla pelnego runtime NWN bez dodatkowego testu.

`aurora-web` rozroznia warstwy source:

```yaml
aurora_web_source_layers:
  hak:
    object_key_contains: "/__aurora/sources/hak/"
    layer: "hak"
  vanilla:
    object_key_contains: "/__aurora/sources/vanilla/"
    layer: "vanilla"
  external_fixture:
    object_key_contains: "/__aurora/sources/"
    layer: "external_fixture"
  module:
    fallback: true
```

Dla placeables istnieje osobna funkcja `resolvePlaceablesTwoDaSourcePriority`, ktora uwzglednia skonfigurowany porzadek HAK. Dla ogolnych 2DA `resolveTwoDaSourcePriority` w kodzie daje `vanilla=0`, `hak=1`, `external_fixture=2`, `module=3`, ale interpretacja "0 = wyzej/nizej" zalezy od miejsca sortowania. Nie przenosic tego bez testu na policyjke engine'u.

Internetowy Beamdog forum i NWNWiki potwierdzaja ogolnie, ze HAK/ERF sa kontenerami zasobow, ale ranking runtime engine powinien byc potwierdzony oddzielnie w Toolset/NWN.

## Istniejacy kod w ekosystemie

Status: POTWIERDZONE.

```yaml
local_erf_hak_code:
  aurora_web_reader:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\nwn\\nwn-erf-packed-module-extractor.service.ts"
    reusable: true
    supports:
      - "ERF/HAK/MOD/NWM read"
      - "V1.0/V1.1"
      - "resource index"
      - "single resource read by resref/type"
  aurora_web_writer_examples:
    path:
      - "C:\\Projects\\aurora-web\\backend\\scripts\\create-vfx-catalog-lab-map.mjs"
      - "C:\\Projects\\aurora-web\\backend\\scripts\\inject-vfx-catalog-lab-compiled-scripts.mjs"
    reusable: "copy/refactor needed"
    supports:
      - "ERF V1.0 build from entries"
  nwn_last_city_reader:
    path: "C:\\Projects\\nwn-last-city\\Areas Generator 3d\\Web Model Viewer\\Common\\erf_reader.py"
    reusable: "reference only unless Python path is chosen"
  external_candidate:
    name: "xoreos-tools erf"
    status: "internet candidate, not confirmed installed locally"
```

## Rekomendacja dla `meshy2aurora`

Status: REKOMENDACJA.

Nie uzywac recznego skladania HAK w UI bez testow binarnych. Wydzielic maly `ErfHakWriter` w rdzeniu Rust zgodnie z aktywna architektura web/WASM i testami deterministycznego readbacku:

```yaml
tdd_plan:
  writer_unit_tests:
    - "writes HAK V1.0 signature and 160-byte header"
    - "rejects resref longer than 16 chars"
    - "writes key list/resource list offsets correctly"
    - "own Rust reader reads back appearance.2da"
    - "own Rust reader reads back model .mdl and texture"
  integration_tests:
    - "meshy2aurora HAK reader reads back generated appearance.2da/model/texture"
    - "NWN EE Toolset/gra manual proof opens generated HAK/module"
    - "optional independent consumer check is supplementary only, never the final oracle"
```
