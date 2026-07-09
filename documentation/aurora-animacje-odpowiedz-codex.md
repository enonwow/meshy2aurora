# aurora-animacje-odpowiedz-codex.md

Status 2026-07-09: REFERENCE-ONLY dla proofow `aurora-web`. Fakty o nazwach animacji/klipach moga byc uzywane jako material porownawczy, ale proof `meshy2aurora` musi przejsc przez NWN EE i wlasny HAK/modul.
Data: 2026-07-08 | Odpowiada na: aurora-animacje-pytania-cloud.md

## Q1: Wymagany szkielet creature
Status: POTWIERDZONE (zrodlo: `C:\Projects\aurora-web\frontend\public\aurora-evidence\creatures\cdp-proof-2026-07-08-source-skin-modelspace-v194\c_kocrachn-cpause1-state.json`, `C:\Projects\aurora-web\frontend\public\aurora-evidence\creatures\cdp-proof-2026-07-08-mephdrow-bowshot-xbow-map-v197\x2_mephdrow011-bowshot-state.json`, `C:\Projects\aurora-web\frontend\src\modules\layout\adapters\three\workspaceObjectMeshLayer.ts`)

Nie ma jednego globalnego szkieletu dla wszystkich creature. Wymagane sa nazwy nodow zgodne z modelem i jego `supermodel chain`. Dla MVP `meshy2aurora` nie powinien wymyslac nowych kosci, tylko mapowac/retargetowac na istniejace nazwy NWN/Aurora.

```yaml
rule:
  required_strategy: "retarget_to_existing_aurora_supermodel_chain"
  not_valid_for_mvp: "arbitrary_meshy_bone_names_without_mapping"
confirmed_examples:
  direct_creature_c_kocrachn:
    source_model: "c_kocrachn"
    supermodel: "c_Horror"
    proof_state: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-source-skin-modelspace-v194\\c_kocrachn-cpause1-state.json"
    available_clip_count: 42
    selected_clip: "cpause1"
    selected_target_count: 22
    representative_nodes:
      - "c_kocrachn"
      - "rootdummy"
      - "pelvis"
      - "torso"
      - "rbicep"
      - "Rforearm1"
      - "Rforearm2"
      - "Rclaw"
      - "Lbicep"
      - "Lforearm"
      - "Lforearm2"
      - "Lclaw"
      - "neck2"
      - "neck1"
      - "head"
      - "Rthigh"
      - "Rcalf1"
      - "Rcalf2"
      - "Rfoot"
      - "lthigh"
      - "Lcalf1"
      - "Lcalf2"
      - "Lfoot"
  humanoid_part_creature_x2_mephdrow011:
    source_model: "pme0 / part model group"
    proof_state: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-mephdrow-bowshot-xbow-map-v197\\x2_mephdrow011-bowshot-state.json"
    available_clip_count: 202
    selected_clip: "bowshot"
    selected_target_count: 46
    attachment_targets:
      head: "head_g"
      right_foot: "rfoot_g"
      left_foot: "lfoot_g"
      neck: "neck_g"
      right_hand: "rhand_g"
      left_hand: "lhand_g"
      chest: "torso_g"
      cloak: "torso_g"
      ammunition: "rootdummy"
```

## Q2: Strategia animacji
Status: HIPOTEZA (uzasadnienie: decyzja dla `meshy2aurora`; mechanizm retargetu jest potwierdzony w aurora-web)

Dla creature MVP strategia powinna byc: retarget Meshy mesh/rig na istniejace animacje NWN/Aurora, nie przenoszenie animacji Meshy 1:1. Uzasadnienie: aurora-web juz buduje animacje z direct modelu i supermodel chain, a runtime creature oczekuje nazw clipow/targetow zgodnych z NWN.

```yaml
recommended_strategy:
  creature_mvp: "retarget_meshy_to_nwn_aurora_supermodel"
  mesh_animation_1_to_1: "not_recommended_for_mvp"
confirmed_runtime_sources:
  supermodel_clip_builder:
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    function: "buildAuroraSupermodelAnimationClips"
    lines: "4192-4275"
  creature_supermodel_loading:
    file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\layout\\adapters\\three\\workspaceObjectMeshLayer.ts"
    lines:
      - "17668-17683"
      - "18880-18916"
  decomp_animation_commands:
    file: "C:\\Projects\\New Folder\\export\\decompiled_all.c"
    commands:
      newanim: 881821
      setsupermodel: 881827
      setanimationscale: 881828
      animroot: "886491-886494"
      event: "886456-886462"
```

## Q3: Lista wymaganych animacji
Status: POTWIERDZONE (zrodlo: runtime proofy creature); HIPOTEZA dla minimalnej listy MVP

Potwierdzone listy sa per-supermodel. Minimalna lista MVP ponizej jest rekomendacja do testow, nie pelny kontrakt Aurory.

```yaml
confirmed_runtime_clip_sets:
  c_kocrachn:
    source: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-source-skin-modelspace-v194\\c_kocrachn-cpause1-state.json"
    count: 42
    clips:
      - "ca1slashl"
      - "ca1slashr"
      - "ca1stab"
      - "creach"
      - "cconjure1"
      - "ccastout"
      - "cparryl"
      - "cparryr"
      - "cdodgelr"
      - "cdodges"
      - "creadyr"
      - "creadyl"
      - "cdamagel"
      - "cdamager"
      - "cdamages"
      - "ckdbck"
      - "ckdbckps"
      - "ckdbckdie"
      - "cguptokdb"
      - "cgustandb"
      - "cwalk"
      - "crun"
      - "ccwalkf"
      - "ccwalkb"
      - "ccwalkl"
      - "ccwalkr"
      - "cpause1"
      - "chturnl"
      - "chturnr"
      - "ctaunt"
      - "cclosel"
      - "ccloseh"
      - "cgetmid"
      - "ckdbckdmg"
      - "ccastoutlp"
      - "cspasm"
      - "cappear"
      - "cdisappear"
      - "cgetmidlp"
      - "cdead"
      - "cdisappearlp"
      - "ccturnr"
  x2_mephdrow011:
    source: "C:\\Projects\\aurora-web\\frontend\\public\\aurora-evidence\\creatures\\cdp-proof-2026-07-08-mephdrow-bowshot-xbow-map-v197\\x2_mephdrow011-bowshot-state.json"
    count: 202
    representative_core:
      - "walk"
      - "run"
      - "pause1"
      - "1hreadyr"
      - "1hreadyl"
      - "1hslashl"
      - "1hslashr"
      - "1hstab"
      - "2hreadyr"
      - "2hreadyl"
      - "2hslashl"
      - "2hslashr"
      - "2hstab"
      - "bowrdy"
      - "bowshot"
      - "xbowrdy"
      - "xbowshot"
      - "castout"
      - "castself"
      - "castup"
      - "castarea"
      - "damagel"
      - "damager"
      - "deadfnt"
      - "deadbck"
      - "appear"
      - "disappear"
mvp_required_tests:
  direct_creature:
    - "pause_or_cp1: cpause1/pause1"
    - "walk: cwalk/walk"
    - "run: crun/run"
    - "attack: ca1slashl or 1hslashl"
    - "damage: cdamagel or damagel"
    - "death: cdead or deadfnt/deadbck"
  humanoid_with_weapon:
    - "pause1"
    - "walk"
    - "run"
    - "1hslashl"
    - "bowshot_or_xbowshot"
events:
  parser_confirmed_in_decompilation: true
  exact_required_event_markers_for_mvp: "NIE_WIEM"
```

## Q4: Limity techniczne
Status: POTWIERDZONE dla czesci runtime/konwertera; NIE WIEM dla twardego max bones i FPS animacji

Konwerter binary MDL czyta do 4 wplywow na wierzcholek. Runtime buduje tracki Three.js: pozycja jako `VectorKeyframeTrack`, rotacja jako `QuaternionKeyframeTrack`, skala jako `VectorKeyframeTrack`. Nie znalazlem twardego limitu liczby kosci ani stalego FPS animacji; clipy sa oparte o czasy kluczy i dlugosc animacji.

```yaml
confirmed:
  influences_per_vertex:
    value: 4
    source_file: "C:\\Projects\\aurora-web\\backend\\src\\modules\\runtime-settings\\adapters\\outbound\\derived\\aurora-mdl-ascii-to-glb.converter.ts"
    source_lines:
      weights_byte_length: 1418
      influence_loop: "1436-1441"
  track_format:
    position: "THREE.VectorKeyframeTrack"
    rotation: "THREE.QuaternionKeyframeTrack"
    scale: "THREE.VectorKeyframeTrack"
    source_file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    source_lines: "5383-5412"
  loader_limits:
    model_load_concurrency: 4
    model_load_starts_per_frame: 2
    bulk_model_load_starts_per_frame: 8
    model_load_timeout_ms: 120000
    source_file: "C:\\Projects\\aurora-web\\frontend\\src\\modules\\placeables\\adapters\\three\\placeableThreeAssetLoader.ts"
    source_lines: "24-29"
unknown:
  max_bones: "NIE_WIEM"
  animation_fps: "NIE_WIEM"
  hard_vertex_count_limit_for_aurora_web: "NIE_WIEM"
```

## Q5: Audyt animacji
Status: POTWIERDZONE (zrodlo: `C:\Projects\meshy2aurora\documentation\aurora-models-animations-audit-2026-07-08.md`)

Plik audytu zostal zapisany w wymaganym folderze:

```yaml
audit_file:
  path: "C:\\Projects\\meshy2aurora\\documentation\\aurora-models-animations-audit-2026-07-08.md"
  status: "written"
```
