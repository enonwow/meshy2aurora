# aurora-2da-creature-codex.md
Data: 2026-07-08 | Aktualizacja: 2026-07-11
Status: AKTYWNA REFERENCJA; lokalny `appearance.2da` POTWIERDZONY 2026-07-10. Produkcyjny writer contract jest w `hak-2da-gff-crosswalk-codex.md`.

## Zakres

Dokument opisuje `appearance.2da` i powiazane 2DA potrzebne do nowego `direct creature`: minimalny wiersz, kolumny czytane przez `aurora-web`, powiazanie z modelem MDL i ograniczenia.

## Zrodla

```yaml
primary_sources:
  aurora_web_catalog:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\catalog\\adapters\\outbound\\blob\\module-snapshot-template-catalog.repository.ts"
    anchors:
      runtime_hak_appearance: "2130-2172, 3458-3484"
      runtime_hak_parts: "2174-2250, 3486-3548"
      parse_appearance_rows: "6593-6652"
      source_priority: "6508-6580"
  aurora_web_processing:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\catalog\\application\\catalog-processing.helpers.ts"
    anchors:
      direct_vs_part_model: "2099-2116, 2119-2135"
  aurora_web_tests:
    path: "C:\\Projects\\aurora-web\\backend\\test\\unit\\module-snapshot-template-catalog.repository.spec.ts"
    anchors:
      appearance_fixture_header: "498, 1741, 1855"
  decompiled_aurora:
    path: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
    anchors:
      creature_appearance_gff: "194114, 194535, 195146, 195211"
internet_supplement:
  - "https://nwn.fandom.com/wiki/Appearance.2da"
  - "https://nwn.fandom.com/wiki/.2da"
```

## Lokalny stan zrodel `appearance.2da`

Status: POTWIERDZONE.

Odnaleziono dwa lokalne read-only warianty `appearance` typu 2017:

```yaml
base_nwn_retail:
  key: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\data\\nwn_base.key"
  key_entry: 26168
  res_id: "0x00E00004"
  bif: "data\\base_2da.bif"
  bif_index: 14
  bif_variable_resource_index: 4
  bytes: 6901169
  sha256: "815c0b3bce0895e9f17d4b92cb02a6d34366267b5a4b9081dece0f4eee7d7a1a"
  columns: 35
  rows: 15100
  row_range: [0, 15099]
local_hak_variant:
  hak: "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\lc_2da.hak"
  bytes: 7655336
  sha256: "ca0b80b74e068d8ebbd94df6005b5971e50eca5c8662fca10a40688ea2c033a2"
  columns: 35
  rows: 15219
  row_range: [0, 15218]
```

Oba payloady pozostaja poza repo. Pierwszy jest canonicalnym lokalnym punktem odniesienia retail; drugi dowodzi, ze writer musi obslugiwac jawnie wybrana base table, a nie zakladac jednego stalego last-row index.

Wiersz 15216 wariantu HAK potwierdza lokalny direct-creature pattern: `MODELTYPE=S`, `RACE=c_squirrel`, `MOVERATE=VSLOW`, `TARGETABLE=1`. Payload pozostaje poza repo i nie jest fixture/proof base.

Jednoczesnie istnieje zainstalowany edytor/schemat:

```yaml
local_2da_tooling:
  editor: "C:\\Program Files (x86)\\2DA & TLK Editor\\2DAEditor.exe"
  appearance_schema: "C:\\Program Files (x86)\\2DA & TLK Editor\\Schemas\\appearance.2daschema"
  status: POTWIERDZONE
```

## Kolumny czytane przez `aurora-web`

Status: POTWIERDZONE.

`parseCreatureAppearanceTwoDaRows` czyta ponizsze kolumny. To jest minimalny kontrakt dla obecnego `aurora-web`, nie pelna specyfikacja retail `appearance.2da`.

```yaml
aurora_web_appearance_columns:
  label:
    input_columns: ["Label", "Name"]
    output_field: "label"
  moverate:
    input_columns: ["MOVERATE"]
    output_field: "moveRateName"
    related_table: "creaturespeed.2da"
  modeltype:
    input_columns: ["MODELTYPE"]
    output_field: "modelType"
    direct_model_rule: "MODELTYPE != P -> direct-model in aurora-web"
  race:
    input_columns: ["RACE"]
    output_field: "raceModel"
    direct_model_rule: "resref modelu MDL dla non-P/direct"
  portrait:
    input_columns: ["PORTRAIT"]
    output_field: "portrait2da"
  envmap:
    input_columns: ["ENVMAP"]
    output_field: "envMap"
  default_phenotype:
    input_columns: ["DefaultPhenoType", "DefaultPhenotype", "DefaultPhenotypeID", "DefaultPheno"]
    output_field: "defaultPhenotype"
  blood_color:
    input_columns: ["BLOODCOLR", "BloodColor", "Blood"]
    output_field: "bloodColor"
  weapon_scale:
    input_columns: ["WEAPONSCALE", "weapon_scale"]
    output_field: "weaponScale"
  size_category:
    input_columns: ["SIZECATEGORY", "CreatureSize"]
    output_field: "sizeCategory"
```

## Direct creature vs part-based creature

Status: POTWIERDZONE w `aurora-web`.

`buildCreatureRenderSourceModelCandidates` bierze `appearanceProfile.raceModel` i ustawia:

```yaml
source_model_candidate_rule:
  if_MODELTYPE_is_P:
    confidence: "assembly-prefix"
    meaning: "part-based phenotype/body-part assembly"
  if_MODELTYPE_is_not_P:
    confidence: "direct-model"
    meaning: "RACE jest bezposrednim resrefem modelu"
```

Dla `meshy2aurora` MVP interesuje nas `direct-model`, czyli `MODELTYPE` inne niz `P`. Cloud pyta o `MODELTYPE=S`; lokalny kod potwierdza tylko regule `!= P`, ale `S` jest dobrym kandydatem do testu.

## Minimalny wiersz dla nowego direct creature

Status: KIERUNEK WDROZENIOWY; wartosci gameplay pozostaja do runtime proofu.

```yaml
minimal_direct_creature_appearance_row:
  table: "appearance.2da"
  header:
    - "Label"
    - "MOVERATE"
    - "MODELTYPE"
    - "RACE"
    - "PORTRAIT"
    - "ENVMAP"
    - "DefaultPhenoType"
    - "BLOODCOLR"
    - "WEAPONSCALE"
    - "SIZECATEGORY"
  example_row:
    row_id: "append_index derived from selected base table"
    Label: "M2A_Kocrachn_Test"
    MOVERATE: "NORM"
    MODELTYPE: "S"
    RACE: "m2a_koc01"
    PORTRAIT: "****"
    ENVMAP: "****"
    DefaultPhenoType: "0"
    BLOODCOLR: "R"
    WEAPONSCALE: 1.0
    SIZECATEGORY: 4
  constraints:
    RACE_resref_max_chars_for_erf_v10: 16
    RACE_must_match_mdl_newmodel: true
    model_file: "m2a_koc01.mdl"
```

Wiersz testowy `aurora-web` ma podobny naglowek:

```text
Label MOVERATE MODELTYPE RACE PORTRAIT ENVMAP DefaultPhenoType BLOODCOLR WEAPONSCALE SIZECATEGORY
17 Guard_Human WALK P Human po_hm_guard default_env 2 R 0.65 4
```

Ten fixture jest POTWIERDZONY jako test parsera, ale nie jest direct creature.

## Kolumny z pytania Cloud spoza lokalnego kontraktu

Status: NIE WIEM lokalnie.

Cloud pyta o m.in. `PERSPACE`, `HEIGHT`, `TARGETABLE`. Wszystkie ponizsze kolumny sa teraz potwierdzone jako obecne w lokalnym `appearance.2da`; ich dokladna semantyka gameplay nadal wymaga dekompilacji/proofu.

```yaml
columns_confirmed_in_local_table_but_semantics_not_fully_proved:
  - "PERSPACE"
  - "HEIGHT"
  - "TARGETABLE"
  - "PREFATCKDIST"
  - "TARGETHEIGHT"
  - "ABORTONPARRY"
  - "RACIALTYPE"
  - "HASLEGS"
  - "HASARMS"
internet_status:
  source: "https://nwn.fandom.com/wiki/Appearance.2da"
  usage: "uzupelnienie do pozniejszego retail-compatible 2DA"
```

Nie wolno ich w specu konwertera oznaczyc jako potwierdzone bez lokalnego pliku retail albo decomp anchorow.

## Powiazane 2DA

Status: POTWIERDZONE dla `aurora-web` tam, gdzie podano anchor.

```yaml
related_2da:
  creaturespeed_2da:
    status: POTWIERDZONE
    local_contract: "kolumna 2DAName mapuje MOVERATE"
    anchor: "module-snapshot-template-catalog.repository.ts:6668+"
  capart_2da:
    status: POTWIERDZONE
    usage: "part-based MODELTYPE=P"
    runtime_hak_reader: "readRuntimeCreaturePartTwoDaSourceFromHakPaths"
  wingmodel_2da:
    status: POTWIERDZONE
    usage: "accessory model source"
    runtime_hak_reader: "readRuntimeCreatureAccessoryModelTwoDaSourceFromHakPaths"
  tailmodel_2da:
    status: POTWIERDZONE
    usage: "accessory model source"
    runtime_hak_reader: "readRuntimeCreatureAccessoryModelTwoDaSourceFromHakPaths"
  racialtypes_2da:
    status: POTWIERDZONE w parserze pomocniczym wedlug lokalnych wynikow rg
    usage: "Appearance column dla race->appearance powiazan"
  portraits_2da:
    status: HIPOTEZA dla MVP
    usage: "PORTRAIT moze wskazywac portret; dla MVP uzyc ****"
  soundset_2da:
    status: HIPOTEZA dla MVP
    usage: "dzwieki creature; nie blokuje pierwszego visual proofu"
```

## Appearance_Type: zakres wartosci i wybor wiersza

Status: POTWIERDZONE dla szerokosci pola w `.utc`; NIE WIEM dla zarezerwowanego bezpiecznego pasma custom rows w retailowym `appearance.2da`.

```yaml
appearance_type:
  utc_storage:
    type: "uint16 / ushort"
    representable_range: [0, 65535]
    aurora_first_anchor:
      read: "C:\\Projects\\New Folder\\export\\decompiled_all.c:194114-194115"
      write: "C:\\Projects\\New Folder\\export\\decompiled_all.c:195146"
  row_selection:
    required:
      - "new row is appended after the last physical row"
      - "new physical row id is within 0..65535"
      - "all existing columns and rows are preserved"
      - "generated UTC Appearance_Type equals that row id"
    not_confirmed:
      - "a universal safe custom range such as 9000+"
      - "the maximum retail table row count accepted by every NWN EE target"
  fixture_9000:
    status: HIPOTEZA
    meaning: "test example only; not an engine rule"
```

Konwerter musi odczytac jawnie wybrana base table, zachowac wszystkie kolumny i wiersze, a nowy wiersz dopisac na koncu. Oficjalna specyfikacja 2DA zabrania wstawiania wierszy pomiedzy istniejace oraz fizycznego usuwania wierszy. Nie wybieramy "wolnej dziury" po `****`. Nowy fizyczny index musi byc `<= 65535`, a UTC `Appearance_Type` musi byc mu rowny. `9000` pozostaje tylko historycznym przykladem fixture.

## Integracja z HAK

Status: POTWIERDZONE.

`aurora-web` czyta `appearance.2da` z HAK przez `readNwnErfResource(hakPath, "appearance", TWO_DA)`. Source key dla runtime HAK ma forme:

```yaml
runtime_source_key:
  pattern: "runtime-source/hak/<hak_name>/appearance.2da"
  builder: "buildRuntimeHakCreatureTwoDaSourceKey"
```

Source layer po sync zapisuje zasoby HAK pod:

```yaml
source_layer_pattern:
  appearance: "__aurora/sources/hak/<hakName>/appearance.2da"
  models: "__aurora/sources/hak/<hakName>/<resref>.mdl"
  textures: "__aurora/sources/hak/<hakName>/<resref>.<dds|tga|plt|txi|mtr|bmp>"
```

## Testy TDD

Status: REKOMENDACJA.

```yaml
tests:
  parse_appearance:
    - "2DA row with MODELTYPE=S and RACE=m2a_koc01 yields sourceModelCandidate confidence direct-model"
    - "2DA row with MODELTYPE=P yields confidence assembly-prefix"
    - "missing optional PORTRAIT/ENVMAP as **** does not block row parse"
  ingest:
    - "HAK with appearance.2da is read before vanilla fallback"
    - "sourceKey includes runtime-source/hak/<hak_name>/appearance.2da"
  end_to_end:
    - "UTC Appearance_Type points to new row id"
    - "catalog render package resolves direct model resref from appearance.2da.RACE"
  writer:
    - "new row is appended after the last physical row, never inserted into a **** hole"
    - "tabs are rejected and quoted labels with spaces round-trip"
    - "append above uint16 range is rejected"
```
