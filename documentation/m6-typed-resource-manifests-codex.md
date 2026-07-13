# M6 typed resource manifests

Data: 2026-07-13

Status: AKTYWNY CONTRACT-LOCK SCHEMATOW; GENERATED PRESET CZESCIOWO `NOT_READY`; RUNTIME `OPEN`

## 1. Zakres i provenance

Ten dokument zamyka uporzadkowane manifesty label/type i nesting potrzebne do
wlasnego writera/readback M6 dla `UTC V3.2`, `IFO V3.2`, `ARE V3.2`,
`GIT V3.2` oraz `GIC V3.2`.

Zrodla sa read-only. Nie kopiujemy do repo payloadow, wartosci gameplay,
komentarzy, nazw, tilesetow, tile IDs, skryptow ani wyposazenia. Hashe ponizej
sa wylacznie evidence tozsamosci packetow, z ktorych odczytano strukture.

Primary Aurora anchors:

- `C:\Projects\New Folder\export\decompiled_all.c:194114-194120` i
  `195134-195152` - UTC `TemplateResRef` oraz `Appearance_Type`;
- `decompiled_all.c:197447-197456` i `197569-197589` - GIT placement i GIC
  `Comment`;
- `decompiled_all.c:221754-221826` i `222030-222035` - rownolegle listy
  GIT/GIC i creature child ID `4`;
- `decompiled_all.c:222704-222728` - `AreaProperties` struct ID `100`;
- `decompiled_all.c:262516-262552`, `264456-264510` - IFO area/HAK nesting,
  area child ID `6`, HAK child ID `8`;
- `decompiled_all.c:255238-255345`, `255391-255420`, `255938-255952` - ARE
  dimensions, `Tile_List` i exact tile field types.

Read-only evidence packets:

```yaml
packets:
  utc67_corpus:
    checked_paths: 4
    unique_payloads: 3
    canonical_order_source: "C:\\Projects\\Claude\\Radoub\\Radoub.IntegrationTests\\TestData\\TestModule\\bandit002.utc"
    packets:
      - source: "C:\\Projects\\Claude\\Radoub\\Radoub.IntegrationTests\\TestData\\TestModule\\bandit002.utc"
        sha256: "687a8342a8ee1ec50b3f9507847690bf2bcc95c7af5f6c671a1336cf5e047a4c"
      - source: "C:\\Projects\\Claude\\Radoub\\Radoub.IntegrationTests\\TestData\\TestModule\\earyldor.utc"
        sha256: "21be9ebeb12d8312e338059c1410b5be4c6fad0bf6891d8982e159041e747198"
      - source: "C:\\Projects\\Claude\\Radoub\\Radoub.IntegrationTests\\TestData\\TestModule\\parleypirate.utc"
        sha256: "0cdc06565cde0592bf8d8b36b13bf0149da3de9d694d7d3173e8efccc9e321e8"
      - source: "C:\\Projects\\Claude\\Radoub\\Parley\\TestingTools\\TestFiles\\parleypirate.utc"
        sha256: "0cdc06565cde0592bf8d8b36b13bf0149da3de9d694d7d3173e8efccc9e321e8"
    note: "The two parleypirate paths are byte-identical copies, so the audit covers four paths but three unique UTC payloads."
  ifo55:
    source: "Radoub IntegrationTests TestModule/module.ifo"
    sha256: "9b15e7aad514f1b5865640a4974072e15c681c084756297110b7948e7fa0f3ad"
  ifo_nonempty_hak_crosscheck:
    source: "local read-only extracted module.ifo"
    sha256: "ba451295e533ac795302c77792fbe55e957b7f298fcdce5251131daf0afcd2c8"
    observed: "58 Mod_HakList children; every child struct ID is 8"
  are43_tile10:
    source: "Radoub IntegrationTests TestModule/area001.are"
    sha256: "a43954232305f51b29a483f994b0bf520a675807f7752c258b387f09e3ecb0db"
  git:
    source: "Radoub IntegrationTests TestModule/area001.git"
    sha256: "72f0c5831a4d053edcb80d478cc098e9b3640b65398fad6c04053d3f69646bc1"
  gic:
    source: "Radoub IntegrationTests TestModule/area001.gic"
    sha256: "67d7e69bc4592b8279e6ab33edbf949d202ff5ab5ec3bc8ec387742b1d51b875"
```

## 2. Wspolny slownik typow

```yaml
gff_field_types:
  BYTE: 0
  WORD: 2
  SHORT: 3
  DWORD: 4
  INT: 5
  FLOAT: 8
  CExoString: 10
  CResRef: 11
  CExoLocString: 12
  VOID: 13
  Struct: 14
  List: 15
```

Kolejnosc pol nie jest wymogiem generic GFF lookup, ale jest zamrozona jako
deterministyczna kolejnosc canonical packetu. Root jest pierwszym structem i ma
ID `0xffffffff`.

## 3. UTC V3.2

### 3.1 Ordered root manifest UTC67

```text
struct ID = 0xffffffff

01 TemplateResRef:CResRef
02 Race:BYTE
03 FirstName:CExoLocString
04 LastName:CExoLocString
05 Appearance_Type:WORD
06 Gender:BYTE
07 Phenotype:INT
08 PortraitId:WORD
09 Description:CExoLocString
10 Tag:CExoString
11 Conversation:CResRef
12 IsPC:BYTE
13 FactionID:WORD
14 Disarmable:BYTE
15 Subrace:CExoString
16 Deity:CExoString
17 Wings_New:DWORD
18 Tail_New:DWORD
19 SoundSetFile:WORD
20 Plot:BYTE
21 IsImmortal:BYTE
22 Interruptable:BYTE
23 Lootable:BYTE
24 NoPermDeath:BYTE
25 BodyBag:BYTE
26 StartingPackage:BYTE
27 DecayTime:DWORD
28 Str:BYTE
29 Dex:BYTE
30 Con:BYTE
31 Int:BYTE
32 Wis:BYTE
33 Cha:BYTE
34 WalkRate:INT
35 NaturalAC:BYTE
36 HitPoints:SHORT
37 CurrentHitPoints:SHORT
38 MaxHitPoints:SHORT
39 refbonus:SHORT
40 willbonus:SHORT
41 fortbonus:SHORT
42 GoodEvil:BYTE
43 LawfulChaotic:BYTE
44 ChallengeRating:FLOAT
45 CRAdjust:INT
46 PerceptionRange:BYTE
47 ScriptHeartbeat:CResRef
48 ScriptOnNotice:CResRef
49 ScriptSpellAt:CResRef
50 ScriptAttacked:CResRef
51 ScriptDamaged:CResRef
52 ScriptDisturbed:CResRef
53 ScriptEndRound:CResRef
54 ScriptDialogue:CResRef
55 ScriptSpawn:CResRef
56 ScriptRested:CResRef
57 ScriptDeath:CResRef
58 ScriptUserDefine:CResRef
59 ScriptOnBlocked:CResRef
60 SkillList:List
61 FeatList:List
62 TemplateList:List
63 SpecAbilityList:List
64 ClassList:List
65 Equip_ItemList:List
66 PaletteID:BYTE
67 Comment:CExoString
```

Ta kolejnosc 67 wspolnych pol jest identyczna po odfiltrowaniu dodatkowych
fields w czterech sprawdzonych sciezkach UTC. Dwie sciezki `parleypirate.utc`
sa byte-identical, wiec corpus ma trzy unikalne payloady.

### 3.2 UTC child structs

```yaml
SkillList:
  observed_count_in_all_four_paths: 28
  child:
    struct_id: 0
    fields:
      - "Rank:BYTE"

FeatList:
  observed_counts: [8, 11, 13]
  child:
    struct_id: 1
    fields:
      - "Feat:WORD"
  proof_values_and_count: NOT_READY

TemplateList:
  observed: "empty in all four paths; three unique payloads"
  child_schema: "not required for an empty list; otherwise NOT_READY"

SpecAbilityList:
  observed: "empty in all four paths; three unique payloads"
  child_schema: "not required for an empty list; otherwise NOT_READY"

ClassList:
  observed_counts: [1, 2]
  simple_non_caster_child:
    struct_id: 2
    fields:
      - "Class:INT"
      - "ClassLevel:SHORT"
  caster_nested_lists: "outside first proof slice; NOT_READY"

Equip_ItemList:
  struct_id_rule: "equipment slot ID"
  observed_struct_ids: [2, 16, 256, 512, 8192, 131072]
  fields:
    - "EquippedRes:CResRef"
  proof_equipment_set: NOT_READY
```

Schema UTC67 i prosty non-caster child schema sa `READY`. Pelny generated
gameplay preset jest `NOT_READY`: `FeatList`, `ClassList` i `Equip_ItemList`
nie maja jednej wspolnej kardynalnosci ani dozwolonych syntetycznych wartosci
zamknietych przez Aurora proof.

## 4. IFO V3.2

### 4.1 Ordered root manifest IFO55

```text
struct ID = 0xffffffff

01 Mod_ID:VOID
02 Mod_MinGameVer:CExoString
03 Mod_Creator_ID:INT
04 Mod_Version:DWORD
05 Expansion_Pack:WORD
06 Mod_Name:CExoLocString
07 Mod_Tag:CExoString
08 Mod_Description:CExoLocString
09 Mod_IsSaveGame:BYTE
10 Mod_CustomTlk:CExoString
11 Mod_Entry_Area:CResRef
12 Mod_Entry_X:FLOAT
13 Mod_Entry_Y:FLOAT
14 Mod_Entry_Z:FLOAT
15 Mod_Entry_Dir_X:FLOAT
16 Mod_Entry_Dir_Y:FLOAT
17 Mod_Expan_List:List
18 Mod_DawnHour:BYTE
19 Mod_DuskHour:BYTE
20 Mod_MinPerHour:BYTE
21 Mod_StartMonth:BYTE
22 Mod_StartDay:BYTE
23 Mod_StartHour:BYTE
24 Mod_StartYear:DWORD
25 Mod_XPScale:BYTE
26 Mod_OnHeartbeat:CResRef
27 Mod_OnModLoad:CResRef
28 Mod_OnModStart:CResRef
29 Mod_OnClientEntr:CResRef
30 Mod_OnClientLeav:CResRef
31 Mod_OnActvtItem:CResRef
32 Mod_OnAcquirItem:CResRef
33 Mod_OnUsrDefined:CResRef
34 Mod_OnUnAqreItem:CResRef
35 Mod_OnPlrDeath:CResRef
36 Mod_OnPlrDying:CResRef
37 Mod_OnPlrEqItm:CResRef
38 Mod_OnPlrLvlUp:CResRef
39 Mod_OnSpawnBtnDn:CResRef
40 Mod_OnPlrRest:CResRef
41 Mod_OnPlrUnEqItm:CResRef
42 Mod_OnCutsnAbort:CResRef
43 Mod_OnPlrChat:CResRef
44 Mod_OnPlrTarget:CResRef
45 Mod_OnPlrGuiEvt:CResRef
46 Mod_OnPlrTileAct:CResRef
47 Mod_OnNuiEvent:CResRef
48 Mod_StartMovie:CResRef
49 Mod_DefaultBic:CResRef
50 Mod_UUID:CExoString
51 Mod_PartyControl:INT
52 Mod_CutSceneList:List
53 Mod_GVar_List:List
54 Mod_Area_list:List
55 Mod_HakList:List
```

### 4.2 IFO child structs

```yaml
Mod_Expan_List: "empty in source packet"
Mod_CutSceneList: "empty in source packet"
Mod_GVar_List: "empty in source packet"

Mod_Area_list:
  child:
    struct_id: 6
    fields:
      - "Area_Name:CResRef"

Mod_HakList:
  child:
    struct_id: 8
    fields:
      - "Mod_Hak:CExoString"
```

IFO55 oraz dzieci area/HAK sa `READY`.

Radoub writer nie jest oracle. Realne packety i dekompilacja rozstrzygaja
rozjazdy: `Mod_ID` jest `VOID`, `Mod_Creator_ID` i `Mod_PartyControl` sa `INT`,
`Mod_HakList` child ma ID `8`.

## 5. ARE V3.2

### 5.1 Ordered root manifest ARE43

```text
struct ID = 0xffffffff

01 ID:INT
02 Creator_ID:INT
03 Version:DWORD
04 Tag:CExoString
05 Name:CExoLocString
06 ResRef:CResRef
07 Comments:CExoString
08 Expansion_List:List
09 Flags:DWORD
10 ModSpotCheck:INT
11 ModListenCheck:INT
12 MoonAmbientColor:DWORD
13 MoonDiffuseColor:DWORD
14 MoonFogAmount:BYTE
15 MoonFogColor:DWORD
16 MoonShadows:BYTE
17 SunAmbientColor:DWORD
18 SunDiffuseColor:DWORD
19 SunFogAmount:BYTE
20 SunFogColor:DWORD
21 SunShadows:BYTE
22 IsNight:BYTE
23 LightingScheme:BYTE
24 ShadowOpacity:BYTE
25 FogClipDist:FLOAT
26 SkyBox:BYTE
27 DayNightCycle:BYTE
28 ChanceRain:INT
29 ChanceSnow:INT
30 ChanceLightning:INT
31 WindPower:INT
32 LoadScreenID:WORD
33 PlayerVsPlayer:BYTE
34 NoRest:BYTE
35 Width:INT
36 Height:INT
37 OnEnter:CResRef
38 OnExit:CResRef
39 OnHeartbeat:CResRef
40 OnUserDefined:CResRef
41 TileBrdrDisabled:BYTE
42 Tileset:CResRef
43 Tile_List:List
```

`Expansion_List` jest pusty w source packet.

### 5.2 Ordered tile manifest ARE tile10

```text
struct ID = 1

01 Tile_ID:INT
02 Tile_Orientation:INT
03 Tile_Height:INT
04 Tile_MainLight1:BYTE
05 Tile_MainLight2:BYTE
06 Tile_SrcLight1:BYTE
07 Tile_SrcLight2:BYTE
08 Tile_AnimLoop1:BYTE
09 Tile_AnimLoop2:BYTE
10 Tile_AnimLoop3:BYTE
```

Inwariant: `Tile_List.count == Width * Height`.

ARE43/tile10 schema jest `READY`. Generated area preset jest `NOT_READY`,
poniewaz nie zamknieto jeszcze wlasnego proofowego `Tileset + Tile_ID` ani
minimalnych wartosci environment. Radoub zapisuje `Tile_AnimLoop*` jako `INT`,
ale realny packet i Aurora potwierdzaja `BYTE`.

## 6. GIT V3.2

### 6.1 Ordered minimal root

```text
struct ID = 0xffffffff

01 AreaProperties:Struct -> child ID 100
02 Creature List:List
03 Door List:List
04 Encounter List:List
05 List:List
06 SoundList:List
07 StoreList:List
08 TriggerList:List
09 WaypointList:List
10 Placeable List:List
```

Pierwszy generated proof candidate ma jeden `Creature List` child ID `4` i
osiem pozostalych list pustych. GFF empty-list representation nie wymaga child
schema dla pustych list.

### 6.2 Ordered AreaProperties

```text
struct ID = 100

01 AmbientSndDay:INT
02 AmbientSndNight:INT
03 AmbientSndDayVol:INT
04 AmbientSndNitVol:INT
05 EnvAudio:INT
06 MusicBattle:INT
07 MusicDay:INT
08 MusicNight:INT
09 MusicDelay:INT
```

### 6.3 Ordered creature instance manifest GIT70

```text
struct ID = 4

01 XPosition:FLOAT
02 YPosition:FLOAT
03 ZPosition:FLOAT
04 XOrientation:FLOAT
05 YOrientation:FLOAT
06 TemplateResRef:CResRef
07 Race:BYTE
08 FirstName:CExoLocString
09 LastName:CExoLocString
10 Appearance_Type:WORD
11 Gender:BYTE
12 Phenotype:INT
13 PortraitId:WORD
14 Description:CExoLocString
15 Tag:CExoString
16 Conversation:CResRef
17 IsPC:BYTE
18 FactionID:WORD
19 Disarmable:BYTE
20 Subrace:CExoString
21 Deity:CExoString
22 Wings_New:DWORD
23 Tail_New:DWORD
24 SoundSetFile:WORD
25 Plot:BYTE
26 IsImmortal:BYTE
27 Interruptable:BYTE
28 Lootable:BYTE
29 NoPermDeath:BYTE
30 BodyBag:BYTE
31 StartingPackage:BYTE
32 DecayTime:DWORD
33 Str:BYTE
34 Dex:BYTE
35 Con:BYTE
36 Int:BYTE
37 Wis:BYTE
38 Cha:BYTE
39 WalkRate:INT
40 NaturalAC:BYTE
41 HitPoints:SHORT
42 CurrentHitPoints:SHORT
43 MaxHitPoints:SHORT
44 refbonus:SHORT
45 willbonus:SHORT
46 fortbonus:SHORT
47 GoodEvil:BYTE
48 LawfulChaotic:BYTE
49 ChallengeRating:FLOAT
50 CRAdjust:INT
51 PerceptionRange:BYTE
52 ScriptHeartbeat:CResRef
53 ScriptOnNotice:CResRef
54 ScriptSpellAt:CResRef
55 ScriptAttacked:CResRef
56 ScriptDamaged:CResRef
57 ScriptDisturbed:CResRef
58 ScriptEndRound:CResRef
59 ScriptDialogue:CResRef
60 ScriptSpawn:CResRef
61 ScriptRested:CResRef
62 ScriptDeath:CResRef
63 ScriptUserDefine:CResRef
64 ScriptOnBlocked:CResRef
65 SkillList:List
66 FeatList:List
67 TemplateList:List
68 SpecAbilityList:List
69 ClassList:List
70 Equip_ItemList:List
```

Child schemas sa takie same jak w UTC sekcja 3.2.

Dokladna roznica blueprint/instance:

```yaml
utc_blueprint_only:
  - "PaletteID:BYTE"
  - "Comment:CExoString"
git_instance_only:
  - "XPosition:FLOAT"
  - "YPosition:FLOAT"
  - "ZPosition:FLOAT"
  - "XOrientation:FLOAT"
  - "YOrientation:FLOAT"
shared_fields: 65
shared_type_differences: 0
```

Schema GIT root/AreaProperties/GIT70 jest `READY`. Generated creature gameplay
preset pozostaje `NOT_READY` z tych samych powodow listowych co UTC. Nie wolno
zakladac, ze sparse GIT z samym `TemplateResRef` odtworzy UTC dynamicznie.

## 7. GIC V3.2

GIC jest resource type `2046` i uzywa tego samego area resref co ARE/GIT.

### 7.1 Ordered root lists

```text
struct ID = 0xffffffff

01 Creature List:List
02 Door List:List
03 Encounter List:List
04 List:List
05 SoundList:List
06 StoreList:List
07 TriggerList:List
08 WaypointList:List
09 Placeable List:List
```

Kazdy element odpowiada elementowi GIT w tej samej liscie i pod tym samym
indeksem. Zachowuje odpowiadajacy GIT struct ID i ma dokladnie jedno pole:

```text
struct ID = corresponding GIT child ID
01 Comment:CExoString
```

Potwierdzone mappingi source packetu:

```yaml
gic_child_ids:
  Creature List: 4
  Door List: 8
  List: 0
  StoreList: 11
  Placeable List: 9
```

Pierwszy generated candidate ma jeden `Creature List` comment child ID `4` i
osiem pustych list. Schema GIC i regule alignment uznajemy za `READY`.

Czy GIC jest bezwzglednie wymagany przez game pozostaje `OPEN_RUNTIME`.
Aurora Toolset zapisuje go rownolegle, dlatego pierwszy Toolset-safe MOD ma go
zawierac.

## 8. Cross-resource invariants

```yaml
appearance:
  utc_template_resref: "equals UTC resource resref"
  git_template_resref: "equals UTC TemplateResRef"
  utc_appearance_type: "equals appended physical appearance.2da index"
  git_appearance_type: "equals UTC Appearance_Type"

area:
  are_resref: "equals ARE resource resref"
  git_resref: "same resref as ARE, resource type 2023"
  gic_resref: "same resref as ARE/GIT, resource type 2046"
  ifo_mod_entry_area: "equals ARE/GIT/GIC resref"
  ifo_area_name: "equals ARE/GIT/GIC resref"

module_resource_types:
  ARE: 2012
  IFO: 2014
  GIT: 2023
  UTC: 2027
  GIC: 2046
```

## 9. Readiness lock

```yaml
schema:
  gff_field_manifests: READY
  utc67: READY
  utc_non_caster_child_structs: READY
  ifo55_and_children: READY
  are43_tile10: READY
  git_root_area_properties_creature70: READY
  gic_lists_comment_alignment: READY

generated_preset:
  utc_gameplay_values_and_variable_lists: NOT_READY
  are_tileset_tile_environment_values: NOT_READY
  git_gameplay_values_and_variable_lists: NOT_READY

runtime:
  sparse_utc_toolset_acceptance: OPEN
  sparse_git_dynamic_utc_resolution: OPEN
  gic_absolute_game_requirement: OPEN
  toolset_module_open: OPEN
  game_appearance_model_texture_resolution: OPEN
```

`READY` w tym dokumencie znaczy gotowosc schematu do TDD own writer/reader i
semantic readback. Nie znaczy Toolset/game acceptance. `NOT_READY` blokuje
zamrozenie wartosci generated fixture, ale nie blokuje implementacji typed
schema. `OPEN` wymaga live runtime proof w M6.
