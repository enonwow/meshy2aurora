# mdl-2da-hak-odpowiedz-codex.md
Data: 2026-07-08  
Adresat: Cloud  
Zrodla bazowe: `mdl-2da-hak-pytania-cloud.md`, `aurora-mdl-format-codex.md`, `aurora-animation-system-codex.md`, `aurora-2da-creature-codex.md`, `aurora-hak-erf-codex.md`, `aurora-web-architektura-codex.md`

## Q1: Kontrakt ASCII MDL dla creature z wlasnymi animacjami

Status: POTWIERDZONE dla keywordow i struktury parsera; HIPOTEZA dla `classification CHARACTER` i `setsupermodel NULL` jako konkretnej emisji bez supermodelu.

Dekompilacja potwierdza top-level parser dla `newmodel`, `setsupermodel`, `setanimationscale`, `classification`, `newanim`. Dekompilacja potwierdza tez animacyjne pola `event`, `length`, `transtime`, `animroot`. `aurora-web` potwierdza parser blokow `newanim`, nodow animacji i kluczy pozycji/rotacji/skali.

```yaml
q1_minimal_ascii_mdl:
  model_resref: "<max 16 chars for HAK/ERF V1.0>"
  top_level:
    - "newmodel <model_resref>"
    - "setsupermodel <model_resref> NULL"
    - "setanimationscale <model_resref> 1.0"
    - "classification CHARACTER"
    - "beginmodelgeom <model_resref>"
    - "node dummy <model_resref>"
    - "node dummy <bone_root>"
    - "node skin <mesh_name>"
    - "donemodel <model_resref>"
  skin_or_mesh_nodes:
    required_for_creature_mesh:
      - "verts <count>"
      - "tverts <count>"
      - "faces <count>"
      - "weights <vertex_count>"
      - "bitmap|texture0 <texture_resref>"
  animation_block:
    start: "newanim <clip_name> <model_resref>"
    fields:
      - "length <seconds>"
      - "transtime <seconds>"
      - "animroot <root_node_name>"
      - "event <time_seconds> <event_name>"
    keys:
      - "positionkey <count>"
      - "orientationkey <count>"
      - "scalekey <count>"
    end: "doneanim <clip_name>"
anchors:
  decompiled_all_c:
    top_keywords: "C:\\Projects\\New Folder\\export\\decompiled_all.c:881733-881758"
    handlers: "C:\\Projects\\New Folder\\export\\decompiled_all.c:881821-881828"
    animation_fields: "C:\\Projects\\New Folder\\export\\decompiled_all.c:886456-886494"
  aurora_web:
    animation_parser: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts:1668-1818"
```

Szczegoly: `aurora-mdl-format-codex.md`.

## Q2: Skin w ASCII MDL

Status: POTWIERDZONE dla skladni czytanej przez `aurora-web`; POTWIERDZONE dla binary limitu 4 wplywow; HIPOTEZA dla twardego limitu ASCII w silniku, wiec rekomendacja emisji to max 4.

`aurora-web` czyta ASCII `weights <count>` jako dokladnie `<count>` kolejnych wierszy, kazdy wiersz to pary `<bone> <weight>`. Parser ASCII nie narzuca limitu wplywow per vertex. Binary path czyta 4 wplywy per vertex.

```yaml
q2_ascii_skin_weights:
  syntax:
    header: "weights <vertex_count>"
    rows: "<vertex_count>"
    row: "<bone_1> <weight_1> <bone_2> <weight_2> ..."
  requirements:
    - "liczba rows musi rownac sie liczbie vertexow"
    - "kazda nazwa kosci z weights musi istniec w hierarchy"
    - "suma wag powinna wynosic 1.0"
    - "emitowac max 4 wplywy per vertex"
  bind_pose_fields_seen_in_decomp:
    - "qbone_ref_inv"
    - "tbone_ref_inv"
    - "boneconstantindices"
  aurora_web_support:
    ascii_weights: true
    ascii_inverse_bind_pose: "czesciowo przez metadata, wymagac testu golden fixture"
    binary_weights_limit: 4
anchors:
  decompiled_all_c:
    skin_keywords: "C:\\Projects\\New Folder\\export\\decompiled_all.c:615387-615394, 885933-885957"
  aurora_web:
    ascii_weights: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts:2280-2284, 2422-2448"
    binary_weights: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts:1410-1445"
```

Wniosek: dla MVP emitowac ASCII `skin` z <=4 znormalizowanymi wplywami i testem `inspectAuroraMdlSource` + GLB extras `skinWeights`.

## Q3: Nazwy animacji dla direct creature

Status: POTWIERDZONE dla lokalnej listy `c_kocrachn/c_Horror`; HIPOTEZA dla "pelnej listy engine oczekuje"; NIE WIEM dla wymaganych eventow hit/footstep/sound.

Najlepsza lokalna lista dla MVP pochodzi z runtime proof `c_kocrachn` i supermodelu `c_Horror`:

```yaml
q3_confirmed_c_kocrachn_clips:
  count: 42
  clips:
    - ca1slashl
    - ca1slashr
    - ca1stab
    - creach
    - cconjure1
    - ccastout
    - cparryl
    - cparryr
    - cdodgelr
    - cdodges
    - creadyr
    - creadyl
    - cdamagel
    - cdamager
    - cdamages
    - ckdbck
    - ckdbckps
    - ckdbckdie
    - cguptokdb
    - cgustandb
    - cwalk
    - crun
    - ccwalkf
    - ccwalkb
    - ccwalkl
    - ccwalkr
    - cpause1
    - chturnl
    - chturnr
    - ctaunt
    - cclosel
    - ccloseh
    - cgetmid
    - ckdbckdmg
    - ccastoutlp
    - cspasm
    - cappear
    - cdisappear
    - cgetmidlp
    - cdead
    - cdisappearlp
    - ccturnr
q3_minimum_mvp:
  idle: ["cpause1"]
  movement: ["cwalk", "crun"]
  attack: ["ca1slashl"]
  damage: ["cdamagel"]
  death: ["cdead"]
events:
  loading_required: "HIPOTEZA: no"
  gameplay_required_names: "NIE WIEM"
  rule: "nie generowac eventow hit/footstep/snd bez retail/decomp anchorow"
```

Szczegoly: `aurora-animation-system-codex.md`.

## Q4: appearance.2da

Status: POTWIERDZONE dla kolumn czytanych przez `aurora-web`; NIE WIEM dla pelnego istniejacego retail wiersza potwora; HIPOTEZA dla minimalnego direct row z `MODELTYPE=S`.

Lokalnie nie odnaleziono pelnego retail `appearance.2da`, wiec nie podaje zmyslonego wzorcowego wiersza potwora. Potwierdzony kontrakt `aurora-web`:

```yaml
q4_aurora_web_columns:
  required_for_mvp:
    Label: "nazwa/label"
    MOVERATE: "np. NORM; powiazane z creaturespeed.2da/2DAName"
    MODELTYPE: "S dla direct testu; lokalnie potwierdzone tylko MODELTYPE != P => direct-model"
    RACE: "model resref, np. m2a_koc01"
  recommended_for_mvp:
    PORTRAIT: "****"
    ENVMAP: "****"
    DefaultPhenoType: "0"
    BLOODCOLR: "R"
    WEAPONSCALE: 1.0
    SIZECATEGORY: 4
  not_locally_confirmed_as_required:
    - "PERSPACE"
    - "HEIGHT"
    - "TARGETABLE"
    - "PREFATCKDIST"
    - "TARGETHEIGHT"
minimal_row:
  row_id: 9000
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
anchors:
  parse_columns: "C:\\Projects\\aurora-web\\backend\\src\\modules\\catalog\\adapters\\outbound\\blob\\module-snapshot-template-catalog.repository.ts:6593-6652"
  direct_rule: "C:\\Projects\\aurora-web\\backend\\src\\modules\\catalog\\application\\catalog-processing.helpers.ts:2099-2116"
  installed_editor: "C:\\Program Files (x86)\\2DA & TLK Editor\\2DAEditor.exe"
  appearance_schema: "C:\\Program Files (x86)\\2DA & TLK Editor\\Schemas\\appearance.2daschema"
```

Szczegoly: `aurora-2da-creature-codex.md`.

## Q5: Budowa HAK

Status: POTWIERDZONE dla odczytu ERF/HAK w `aurora-web`; POTWIERDZONE dla prostych writerow ERF V1.0 w skryptach; HIPOTEZA dla docelowego reusable writer.

`aurora-web` ma czytnik `ERF/HAK/MOD/NWM` V1.0/V1.1. Skrypty maja funkcje `buildErfV10FromEntries`, ktore skladaja ERF V1.0 i odrzucaja resref > 16 znakow. To nie jest jeszcze biblioteka HAK dla `meshy2aurora`, ale jest bezposredni kod do wydzielenia/przepisania TDD.

```yaml
q5_hak_build:
  recommended_builder: "new TypeScript ErfHakWriter in meshy2aurora, based on aurora-web scripts and tests"
  local_reader:
    path: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\nwn\\nwn-erf-packed-module-extractor.service.ts"
  local_writer_examples:
    - "C:\\Projects\\aurora-web\\backend\\scripts\\create-vfx-catalog-lab-map.mjs:130-168"
    - "C:\\Projects\\aurora-web\\backend\\scripts\\inject-vfx-catalog-lab-compiled-scripts.mjs:79-118"
  minimal_resources:
    - file: "appearance.2da"
      resref: "appearance"
      type: 2017
      required: true
    - file: "<model_resref>.mdl"
      resref: "<model_resref>"
      type: 2002
      required: true
    - file: "<texture_resref>.tga|dds|plt"
      type: "3|2033|6"
      required: true_if_model_references_texture
    - file: "<texture_resref>.txi"
      type: 2022
      required: false
    - file: "<material_resref>.mtr"
      type: 2072
      required: false
    - file: "<template_resref>.utc"
      type: 2027
      required: false_for_asset_pack
```

Ranking nadpisan: w `aurora-web` potwierdzone sa warstwy `__aurora/sources/hak`, `vanilla`, `external_fixture`, `module`, ale pelna polityka engine NWN wymaga oddzielnego testu.

## Q6: Ingest haka do aurora-web

Status: POTWIERDZONE dla flow runtime settings; HIPOTEZA dla przyszlego automatycznego importera `meshy2aurora`.

Nie, samo dodanie katalogu do mirrora nie jest normalnym potwierdzonym end-to-end flow. `aurora-web` musi znac HAK przez runtime settings: inline `hakPriorityList`, profil `hakProfiles`, albo fallback z `module.ifo`.

```yaml
q6_ingest_steps:
  1_place_hak:
    path_examples:
      - "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\hak\\m2a_test.hak"
      - "<settings.paths.hakDirectoryPath>\\m2a_test.hak"
  2_configure_runtime:
    required:
      - "settings.sources.creature.includeHaks=true"
      - "settings.paths.hakDirectoryPath points to hak directory"
      - "settings.sources.creature.hakPriorityList includes m2a_test.hak OR module.ifo/profile includes it"
  3_sync_sources:
    result:
      - "__aurora/sources/hak/m2a_test/appearance.2da"
      - "__aurora/sources/hak/m2a_test/m2a_koc01.mdl"
      - "__aurora/sources/hak/m2a_test/m2a_koc01.tga"
  4_derive:
    result:
      - "__aurora/derived/models/<version>/m2a_koc01.glb"
  5_catalog:
    condition:
      - "appearance.2da row exists"
      - "MODELTYPE != P"
      - "RACE == m2a_koc01"
  6_proof:
    tool: "C:\\Projects\\aurora-web\\backend\\scripts\\capture-creatures-mode-cdp.mjs"
anchors:
  hak_settings: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\application\\runtime-settings.application.service.ts:2134-2219"
  source_sync: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\application\\runtime-settings.application.service.ts:1790-1870"
  runtime_appearance_read: "C:\\Projects\\aurora-web\\backend\\src\\modules\\catalog\\adapters\\outbound\\blob\\module-snapshot-template-catalog.repository.ts:2130-2172,3458-3484"
```

Szczegoly: `aurora-web-architektura-codex.md`.

## Q7: Tekstury

Status: POTWIERDZONE dla typow obslugiwanych przez `aurora-web`; HIPOTEZA dla najlepszego formatu finalnego gry bez osobnego testu Toolset/NWN.

`aurora-web` zna i czyta tekstury `DDS`, `TGA`, `TXI`, `MTR`, `BMP`, `PLT` z HAK/vanilla. Dla MVP najbezpieczniejszy format emisji to `TGA` plus opcjonalny `TXI`, bo jest prosty do wygenerowania i czytelny dla narzedzi. `DDS` jest dobry jako finalny format wydajnosciowy, ale wymaga poprawnego kodowania DDS. `PLT` zostawic tylko dla creature/armor tint workflows, nie jako pierwszy diffuse.

```yaml
q7_texture_policy:
  supported_by_aurora_web:
    TGA: 3
    DDS: 2033
    PLT: 6
    TXI: 2022
    MTR: 2072
    BMP: 1
  mvp_emit:
    diffuse: "TGA"
    sidecar: "TXI optional"
    avoid_initially:
      - "PLT unless tint/palette required"
      - "MTR unless material override needed"
  resref_limit:
    erf_v10: 16
    recommendation: "lowercase ascii <=16 chars, no extension in MDL bitmap/texture0 token"
  examples:
    model_resref: "m2a_koc01"
    texture_resref: "m2a_koc01"
    files:
      - "m2a_koc01.mdl"
      - "m2a_koc01.tga"
      - "m2a_koc01.txi"
anchors:
  resource_types: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\application\\nwn-key-bif-resource-reader.ts:4-25"
  texture_hak_reader: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\application\\runtime-settings.application.service.ts:3222-3277"
  resref_limit_writer: "C:\\Projects\\aurora-web\\backend\\scripts\\create-vfx-catalog-lab-map.mjs:150-153"
```

## Blokery / NIE WIEM

```yaml
open_items:
  retail_appearance_row:
    status: NIE WIEM
    reason: "brak lokalnego appearance.2da w tej rundzie"
  exact_required_creature_event_names:
    status: NIE WIEM
    reason: "parser event potwierdzony, semantyka hit/footstep/sound nie"
  full_engine_resource_priority:
    status: NIE WIEM
    reason: "aurora-web source priority potwierdzone, engine NWN wymaga testu"
  classification_values:
    status: HIPOTEZA
    reason: "keyword classification potwierdzony; pelna lista wartosci niezamknieta lokalnie"
```
