# Animacje - kontrakt profilu A

Data: 2026-07-10 | Status: AKTYWNY KIERUNEK, RUNTIME PROOF OTWARTY

## 1. Najwazniejsza korekta

Lokalny binary `c_kocrachn` nie zawiera 42 wlasnych animacji.

```yaml
c_kocrachn_binary:
  resource: "cep3_core1.hak / c_kocrachn / type 2002 / id 724"
  model_name: "c_kocrachn"
  supermodel: "c_Horror"
  animation_scale: 0.72
  own_animation_count: 0
  own_animation_array_offset: 232
```

`c_Horror` nie wystepuje jako zasob typu 2002 w 114 przeskanowanych lokalnych HAK. Lista 42 klipow widziana w starszym proofie `aurora-web` opisuje rozwiazany chain w osobnym projekcie, a nie payload `c_kocrachn` ani samodzielny proof `meshy2aurora`.

Dwa lokalne, pokrewne zasoby `c_phod_horror_b` i `c_phod_horror_p` maja po 42 animacje o tym samym namespace klipow. Potwierdzaja strukture i nazewnictwo jako read-only observation, ale nie sa dowodem, ze ich keyframes sa kontraktem `c_Horror`.

## 2. Decyzja produktowa

Finalny proof `meshy2aurora` ma byc self-contained:

- generowany MDL zawiera wlasny szkielet i wlasne/generowane klipy;
- proof nie kopiuje keyframes, szkieletu ani payloadu z retail/CEP;
- `c_kocrachn`, `c_Horror` i pokrewne modele sluza do poznania struktury, nazw i zachowania engine'u;
- tryb dziedziczenia po zewnetrznym supermodelu moze powstac pozniej jako jawny compatibility profile, ale nie jest podstawowym proofem produktu.

## 3. Dwie trasy implementacji

```yaml
routes:
  A_self_contained:
    priority: "required for product proof"
    input: "own or user-provided rig and animation clips"
    output: "model with own animation headers, node tracks and optional events"
    external_runtime_animation_dependency: false
  B_supermodel_compatibility:
    priority: "later compatibility profile"
    input: "model skeleton compatible by node names with a user-selected installed supermodel"
    output: "supermodel name + animationScale, possibly zero own animations"
    gate: "separate provenance and NWN EE proof"
```

## 4. Minimalne poziomy akceptacji

Nie nazywamy jednej petli idle "pelna obsluga animacji".

```yaml
acceptance_levels:
  loader_smoke:
    clips: ["cpause1"]
    purpose: "binary layout, animroot, controllers and visible motion"
  movement_smoke:
    clips: ["cpause1", "cwalk", "crun"]
    purpose: "looping movement states"
  profile_A_gameplay_candidate:
    clips: ["cpause1", "cwalk", "crun", "ca1slashl", "cdamagel", "cdead"]
    status: "candidate names; exact state routing must be proved in game"
  full_selected_profile:
    purpose: "every state selected for the product has an explicit clip/fallback decision and proof"
```

Nazwy szesciu klipow sa kierunkiem kompatybilnosci, nie licencja na kopiowanie danych animacji.

## 5. Potwierdzony binary contract

Animation header ma `0xc4` bajty i zawiera geometry header, `length`, `transition_time`, `animroot[64]` oraz array eventow. Event ma `0x24` bajty: czas i `name[32]`. Model header wskazuje array offsetow animation headers.

Kazda animation geometry ma wlasny node tree/controller data. Track moze byc zastosowany tylko do istniejacej nazwy noda; walidator musi raportowac track bez targetu.

`animationScale` jest polem modelu istotnym dla pozycyjnych trackow dziedziczonych/retargetowanych. Nie wolno go traktowac jako ogolnej skali siatki.

## 6. Eventy i petle

W pokrewnych lokalnych 42-clip modelach zaobserwowano event names `hit`, `cast`, `snd_footstep` i `snd_hitground`. To dowodzi, ze eventy sa realna czescia payloadu, ale nie dowodzi, ktore sa obowiazkowe dla wygenerowanego creature.

```yaml
event_policy:
  loader_smoke: "events optional; zero events is allowed only as a test hypothesis"
  gameplay: "hit/footstep/sound timing requires explicit state proof"
  validation:
    - "event time is finite and within 0..length"
    - "events are emitted in deterministic time order"
    - "unknown event names are preserved but reported"
loop_policy:
  current_status: "engine loop semantics not yet fully mapped"
  rule: "do not invent a binary loop flag; prove behavior per state/clip in NWN EE"
```

## 7. TDD i zamkniecie GB-005

```yaml
tests:
  binary:
    - "zero, one and multiple animations round-trip"
    - "name, length, transition, animroot and events round-trip"
    - "animation node/controller pointers stay inside core"
  skeleton:
    - "every animated node exists exactly once"
    - "every weighted bone exists"
    - "root and animroot are explicit"
  behavior:
    - "loader smoke shows measurable cpause1 motion"
    - "movement proof distinguishes cwalk and crun"
    - "one-shot proof reaches terminal pose without being mistaken for a loop"
    - "event proof records observed hit/footstep behavior or a named blocker"
  provenance:
    - "no external animation payload is committed or copied"
```

GB-005 ma status `DIRECTION_DEFINED_RUNTIME_OPEN`. Implementacja M4A zaczyna sie od self-contained `loader_smoke`, a nie od zalozenia, ze `c_Horror` bedzie dostepnym proof dependency.
