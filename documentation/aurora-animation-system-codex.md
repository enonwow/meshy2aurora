# aurora-animation-system-codex.md

Status 2026-07-09: AKTYWNA REFERENCJA FORMATU/SYSTEMU, ale proofy i sciezki z `aurora-web` sa reference-only. Implementacja `meshy2aurora` nie moze importowac ani odpalac `aurora-web`.
Data: 2026-07-08  
Status: POTWIERDZONE czesciowo; HIPOTEZA dla minimalnego zestawu gameplayowego; NIE WIEM dla pelnej semantyki eventow retail

## Zakres

Ten dokument opisuje animacje creature potrzebne do decyzji implementacyjnej `meshy2aurora`: nazwy klipow, `supermodel` chain, `setanimationscale`, `transtime`, `event`, petle i minimalny zestaw klipow dla pierwszego `direct creature`.

## Zrodla

```yaml
primary_sources:
  decompiled_aurora:
    path: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
    anchors:
      top_level_animation_keywords: "614810-614811, 881757-881758, 881821"
      animation_fields: "615434-615442, 886456-886494"
      controller_key_strings: "614904-614923, 882865-882972"
  aurora_web_backend:
    converter: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
    anchors:
      source_animation_parser: "1668-1760"
      source_animation_node_keys: "1770-1818"
      binary_animation_events: "859-915"
  aurora_web_frontend:
    loader: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    anchors:
      supermodel_clip_builder: "4193-4238"
      animation_scale: "4206-4216, 4427"
      source_hierarchy_transform: "5324-5358"
  runtime_proof:
    summary: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-source-skin-modelspace-v194\\summary.json"
    state: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-source-skin-modelspace-v194\\c_kocrachn-cpause1-state.json"
```

## Mechanika wyboru klipu

Status: POTWIERDZONE w `aurora-web`; HIPOTEZA dla pelnej zgodnosci gry bez dodatkowych retail testow.

`aurora-web` buduje klipy z modelu/supermodelu po nazwach nodow. `buildAuroraSupermodelAnimationClips` zbiera targety w modelu docelowym, targety w supermodelu, wyznacza root animacji i skaluje retarget pozycji przez `animationScale`.

```yaml
animation_resolution:
  target_model:
    source: "sourceHierarchy / scene object names"
    required: "nazwy kosci/nodow musza byc zgodne z animowanymi trackami"
  supermodel:
    field: "setsupermodel <model> <supermodel>"
    aurora_web_metadata: "extras.aurora.supermodel"
    status: POTWIERDZONE
  animation_scale:
    field: "setanimationscale <model> <float>"
    aurora_web_metadata: "extras.aurora.animationScale"
    usage: "skaluje pozycje retargetowanych trackow"
  animroot:
    field: "animroot <node_name>"
    role: "root animacji; musi wskazywac istniejacy node dla stabilnego retargetu"
```

## Skladnia bloku animacji

Status: POTWIERDZONE.

```yaml
newanim:
  start: "newanim <clip_name> <target_model>"
  scalar_fields:
    length: "seconds"
    transtime: "seconds; blend/transition time"
    animroot: "node name"
  event:
    syntax: "event <time_seconds> <event_name>"
    parser_behavior: "eventy sortowane po czasie w dekompilacji i aurora-web metadata"
  node_keys:
    - "positionkey <count>"
    - "orientationkey <count>"
    - "scalekey <count>"
    - "alphakey <count>"
    - "selfillumcolorkey <count>"
  end: "doneanim <clip_name>"
```

## Lista klipow potwierdzona na wzorcu `c_kocrachn`

Status: POTWIERDZONE z runtime proof `c_kocrachn` / `c_Horror`.

```yaml
c_kocrachn_supermodel_chain:
  model: "c_kocrachn"
  supermodel: "c_Horror"
  animation_scale: 0.7200000286102295
  available_clip_count: 42
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
```

To jest najlepszy lokalny wzorzec dla pierwszego MVP, bo decyzja projektowa D4 wskazuje `c_kocrachn/c_horror`.

## Minimalny zestaw dla grywalnego direct creature

Status: HIPOTEZA wdrozeniowa, oparta na potwierdzonej liscie `c_kocrachn` i typowych stanach creature. Wymaga testu w NWN/Toolset.

```yaml
minimum_direct_creature_clips:
  idle:
    - cpause1
  movement:
    - cwalk
    - crun
  combat_basic:
    - ca1slashl
  damage:
    - cdamagel
  death:
    - cdead
recommended_next_clips:
  facing_turns:
    - chturnl
    - chturnr
  combat_more:
    - ca1slashr
    - ca1stab
    - creadyr
    - creadyl
  knockdown:
    - ckdbck
    - ckdbckdie
    - cguptokdb
    - cgustandb
```

Implementacyjnie pierwszy etap moze miec dwa tryby:

```yaml
implementation_modes:
  reference_supermodel_mode:
    status: POTWIERDZONE w aurora-web dla odtwarzania
    description: "model ma wlasna siatke/szkielet zgodny z referencja i korzysta z supermodelu"
    required: "zgodne nazwy kosci z supermodelem"
  self_contained_direct_mode:
    status: HIPOTEZA do proofu w grze
    description: "MDL zawiera wlasne newanim dla minimalnych klipow"
    required: "kazdy wymagany clip jako newanim targetujacy model"
```

## Eventy animacji

Status: POTWIERDZONE, ze parser obsluguje eventy; NIE WIEM, ktore nazwy sa obowiazkowe dla hitow/krokow w retail runtime.

Dekompilacja i `aurora-web` potwierdzaja strukture `event <time> <name>`. Nie znalazlem lokalnie zamknietej tabeli nazw eventow typu hit/footstep/sound dla creature. Z tego powodu nie wolno generowac nazw eventow "na wyczucie" jako fakt.

```yaml
event_policy:
  loading_requirement:
    status: HIPOTEZA
    value: "eventy nie sa wymagane do samego zaladowania i odtworzenia klipu"
  gameplay_requirement:
    status: NIE WIEM
    value: "nazwy eventow dla trafienia, krokow i dzwiekow wymagaja retail/decomp testu"
  emitter_rule_for_mvp:
    - "emituj animacje bez eventow dla proofu wizualnego"
    - "dodaj eventy dopiero po znalezieniu retail anchorow w modelach/supermodelach"
```

## Petle i one-shot

Status: HIPOTEZA; brak lokalnej kotwicy jednoznacznie mapujacej flagi petli.

W samym bloku `newanim` potwierdzone sa `length`, `transtime`, `animroot`, `event` i klucze. Petlowanie wyglada na zalezne od nazwy/stanu animacji engine'u, a nie od pola `loop` potwierdzonego w tej rundzie. Dlatego w specu TDD nie zakladac pola `loop`; testowac zachowanie po nazwie klipu (`cpause1`, `cwalk`, `crun` jako naturalnie petlowane).

## Wnioski dla Meshy

Status: POTWIERDZONE dla ograniczenia Meshy z dokumentu Cloud; HIPOTEZA dla retargetu.

```yaml
meshy_pipeline_animation_decision:
  humanoid:
    source: "meshy-api-cloud.md"
    path: "pelny Meshy rig + Meshy animation clips"
    conversion:
      - "map Meshy bones to Aurora-compatible names"
      - "emit newanim blocks"
      - "validate in meshy2aurora animation validator and binary readback parser"
  monster_or_quadruped:
    source: "meshy-api-cloud.md"
    path: "mesh only + Aurora reference skeleton"
    conversion:
      - "fit mesh to reference skeleton"
      - "reuse/retarget c_kocrachn/c_Horror style clips"
      - "do not rely on Meshy auto-rig"
```

## Testy TDD dla animacji

Status: REKOMENDACJA.

```yaml
tests:
  parser_contract:
    - "given generated binary/debug model with cpause1/cwalk/crun, meshy2aurora readback returns animation names"
    - "given animation with animroot, meshy2aurora readback preserves animRoot"
    - "given event rows, meshy2aurora validator sorts or reports event ordering"
  skeleton_contract:
    - "all animation nodeNames exist in sourceHierarchy"
    - "all weighted bones exist in sourceHierarchy"
  runtime_contract:
    - "generated HAK/module proof opens in NWN EE Toolset/gra"
    - "manual proof confirms minimum_direct_creature_clips are present/usable when animation export is enabled"
```
