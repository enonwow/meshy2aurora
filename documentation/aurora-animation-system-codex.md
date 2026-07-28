# aurora-animation-system-codex.md

Status 2026-07-09: AKTYWNA REFERENCJA FORMATU/SYSTEMU, ale proofy i sciezki z `aurora-web` sa reference-only. Implementacja `meshy2aurora` nie moze importowac ani odpalac `aurora-web`.
Status 2026-07-28: UZUPELNIENIE TERMINOLOGII I KONTRAKTU 42 BAZOWYCH SLOTOW `S/L`.
Data: 2026-07-08  
Status: AKTYWNA REFERENCJA; kierunek produktu i korekta lokalnego binary sa w `animacje-kontrakt-profil-a-codex.md`.

## Zakres

Ten dokument opisuje animacje creature potrzebne do decyzji implementacyjnej `meshy2aurora`: nazwy klipow, `supermodel` chain, `setanimationscale`, `transtime`, `event`, petle i minimalny zestaw klipow dla pierwszego `direct creature`.

## Decyzja 2026-07-28: 42 bazowe animacje creature `S/L`

Status: DECYZJA PRODUKTOWA wlasciciela oraz POTWIERDZONE lokalnym retail
Aurora/NWN i own binary readbackiem.

Dla docelowego, kompatybilnego z rodzina `c_Horror` profilu `direct creature`
o `MODELTYPE=S` albo `MODELTYPE=L` obowiazuje katalog **42 standardowych
bazowych slotow animacji creature**. Jest to namespace klipow z prefiksem `c`,
zawierajacy m.in. `cpause1`, `cwalk`, `crun`, ataki, reakcje na obrazenia,
knockdown, smierc, casting oraz pojawienie i znikniecie.

Liczba `42` nie oznacza liczby wszystkich stanow calego silnika Aurora.
Modele `MODELTYPE=P/F` korzystaja z innego, wiekszego zestawu klipow.
Nie oznacza tez, ze kazdy istniejacy retailowy model `S/L` materializuje
lokalnie dokladnie 42 klipy. Jest to bazowy kontrakt pokrycia naszego profilu;
konkretny model moze czesc slotow pominac, nadpisac albo odziedziczyc.
Standardowe sloty `S/L` moga byc:

- zapisane lokalnie w docelowym MDL;
- odziedziczone z kompatybilnego `supermodel`;
- wygenerowane przez `meshy2aurora` z materialu nalezacego do uzytkownika.

Model dziedziczacy moze dlatego miec `0` lokalnych animacji, a nadal
udostepniac efektywny standardowy zestaw przez lancuch supermodelu.

### Obowiazujace rozroznienie pojec

| Pojecie | Znaczenie w produkcie |
| --- | --- |
| Stan/akcja silnika Aurora | Semantyczny stan lub akcja, np. pause, walk, run, attack, damage albo death. |
| Bazowy slot `S/L` | Nazwa klipu realizujacego stan dla Simple/Limited creature, np. `cpause1`, `cwalk`, `crun`. Standardowy namespace tego profilu ma 42 pozycje. |
| `ANIMATION_*` | Publiczny selektor API NWScript dla skryptowalnych animacji. Nie jest pelnym katalogiem automatycznych stanow ruchu i walki silnika. |
| Loop/FNF/Start/End | Typ odtwarzania lub faza animacji, a nie stan Aurory. |
| Animacja uzytkownika | Dodatkowy klip i jego jawne mapowanie. Nie zmienia znaczenia ani liczby 42 bazowych slotow. |

Przykladowe mapowanie zalezne od `MODELTYPE`:

| Stan Aurory | `MODELTYPE=P/F` | `MODELTYPE=S/L` | Selektor NWScript |
| --- | --- | --- | --- |
| pause | `pause1` | `cpause1` | `ANIMATION_LOOPING_PAUSE = 0` |
| walk | `walk` | `cwalk` | stan ruchu sterowany przez silnik |
| run | `run` | `crun` | stan ruchu sterowany przez silnik |
| taunt | `taunt` | `ctaunt` | `ANIMATION_FIREFORGET_TAUNT = 108` |

`FIREFORGET` w nazwie ostatniego selektora opisuje sposob odtworzenia
one-shot. Nie stanowi osobnego stanu ani zrodla katalogu stanow.

### Zrodlo standardu i provenance

Nie istnieje jeden projektowy ani retailowy `animations.2da`, ktory bylby
pelna definicja tego kontraktu. Zrodla sa rozdzielone:

1. retailowy `nwmain.exe` zawiera hardkodowana tabele stanow i wariantow nazw
   klipow; lokalnie potwierdzono m.in. `pause1`, `cpause1`, `cwalk`, `crun`
   oraz `customXlp/start/end`;
2. retailowy `nwscript.nss`, odczytany in-place przez
   `nwn_base.key -> data/base_scripts.bif`, definiuje publiczne selektory
   `ANIMATION_*`;
3. `appearance.2da` i jego `MODELTYPE` wybieraja rodzine Full, Simple albo
   Limited;
4. exact namespace 42 slotow `S/L` potwierdza own binary reader na trzech
   niezaleznych rodzinach: retail `c_horror`, external reference binary
   `c_Direwolf` oraz CEP R3 `c_phod_horror_b`.

Katalog 42 nazw jest faktem o publicznym kontrakcie formatu i silnika.
Nie wolno kopiowac z modeli referencyjnych keyframes, event timingow,
szkieletow ani payloadow. Wygenerowane animacje musza pochodzic z danych
uzytkownika albo z materialu stworzonego przez `meshy2aurora`.

Dowody repozytorium:

- [runtime_witness_conformance.rs](../crates/m2a-core/tests/runtime_witness_conformance.rs)
  odczytuje exact payloady, sprawdza ich SHA-256 i porownuje zbiory nazw;
- [direct_creature_animation.rs](../crates/m2a-core/src/direct_creature_animation.rs)
  przechowuje wersjonowany katalog `FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1`;
- [m0-runtime-conformance-state-projection-fix-2026-07-21.md](evidence/m0-runtime-conformance-state-projection-fix-2026-07-21.md)
  zapisuje wlasciwosci trzech rodzin runtime;
- [M1B-evidence.md](evidence/M1B-evidence.md) zapisuje exact CEP resource
  ranges, hashe i liczby animacji.

### Kontrakt UI i pipeline

Gracz wybiera w UI semantyczny stan/akcje Aurory, a nie surowa sciezke do
referencyjnego modelu i nie przypadkowa nazwe klipu Meshy. Pipeline wykonuje:

```text
stan Aurory
-> rodzina MODELTYPE
-> bazowy slot P/F albo S/L
-> lokalny klip, kompatybilny supermodel albo klip uzytkownika
-> walidacja obecnosci, rigu, eventow i binary readback
```

UI ma pokazywac osobno:

- 42 bazowe sloty profilu `S/L` i stan ich pokrycia;
- dodatkowe animacje uzytkownika;
- zrodlo realizacji slotu: local, inherited albo generated;
- typ odtwarzania jako metadane, nie jako nazwe stanu.

Plan wykonawczy i odznaczana checklista implementacji tego kontraktu:
[plan-implementacji-creature-animation-mapping-ui-2026-07-28.md](plan-implementacji-creature-animation-mapping-ui-2026-07-28.md).

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

## Standardowe 42 bazowe sloty `S/L` potwierdzone na rodzinie horror

Status: POTWIERDZONE przez own binary reader na retail `c_horror`,
external reference binary `c_Direwolf` i CEP R3 `c_phod_horror_b`.
`c_kocrachn` ma `0` lokalnych animacji i dziedziczy je przez `c_Horror`;
nie jest zrodlem 42 lokalnych payloadow.

```yaml
c_kocrachn_supermodel_chain:
  model: "c_kocrachn"
  supermodel: "c_Horror"
  animation_scale: 0.72
  own_animation_count_in_local_binary: 0
  c_Horror_found_in_114_scanned_local_haks: false
  reference_only_resolved_clip_count: 42
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

Lista jest standardowym katalogiem 42 bazowych slotow `S/L`, ale nie jest
zrodlem keyframes ani event timingow. Nie wolno utozsamiac jej z liczba
wszystkich stanow silnika Aurora. Produkcyjny profil A jest self-contained;
`c_kocrachn/c_Horror` pozostaje materialem read-only do obserwacji struktury.

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
    status: "pozniejszy compatibility profile; POTWIERDZONE w aurora-web tylko jako reference-only"
    description: "model ma wlasna siatke/szkielet zgodny z referencja i korzysta z supermodelu"
    required: "zgodne nazwy kosci z supermodelem"
  self_contained_direct_mode:
    status: "wymagany kierunek produktowego proofu; runtime proof otwarty"
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
    - "loader smoke moze testowac zero eventow jako jawna hipoteze"
    - "gameplay acceptance wymaga osobnego proofu eventow hit/footstep/sound"
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
    path: "mesh plus own/user-provided compatible skeleton"
    conversion:
      - "use read-only Aurora models only to understand naming and structure"
      - "fit mesh to an own or user-provided skeleton"
      - "emit own/generated clips using compatible state names"
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
