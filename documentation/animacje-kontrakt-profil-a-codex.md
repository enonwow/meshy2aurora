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
    clips: ["cpause1", "cwalk", "crun", "ca1slashl", "cdamagel", "ckdbckdie", "cdead"]
    status: "candidate names; exact state routing must be proved in game"
  full_selected_profile:
    purpose: "every state selected for the product has an explicit clip/fallback decision and proof"
```

Nazwy klipow sa kierunkiem kompatybilnosci, nie licencja na kopiowanie danych animacji.

### 4.1. Static-rigid visibility control nie jest animowanym creature

Trzeba rozdzielac dwie niezalezne bramki:

```yaml
static_rigid_visibility_control:
  proves:
    - "resolver MOD/HAK/appearance.2da znajduje exact model"
    - "base rigid mesh moze zostac narysowany jako creature"
  does_not_prove:
    - "ruch w stanie cwalk lub crun"
    - "atak, damage, death albo transition"
    - "deformacje SkinMesh lub ruch segmentow"
animated_creature_candidate:
  requires_one_of:
    - "spojny SkinMesh: joint hierarchy + JOINTS/WEIGHTS + bind data + motion tracks"
    - "spojna rigid-segment hierarchy + motion tracks"
  also_requires:
    - "jawne lub kompatybilnie odziedziczone stany gameplay"
    - "widoczny ruch potwierdzony w runtime"
```

Sam brak SkinMesh nie dyskwalifikuje kazdego creature: legalny rigid creature
moze poruszac segmentami parentowanymi do animowanych nodow. Model bez obu
mechanizmow — bez skina/wag oraz bez przegubowej hierarchii z motion tracks —
nie jest jednak kandydatem animowanego creature.

Emisja nazwanych naglowkow animacji z identity transform nie tworzy ruchu.
Takie stany sa dozwolone tylko w statycznym controlu kontenera. Moga pomoc
sprawdzic, czy engine rozwiazuje i rysuje base mesh, ale nie zaliczaja nawet
`loader_smoke`, ktory wymaga widocznego motion.

Konkretna kwalifikacja N1 z `2026-07-27`:

- source ma jeden node bez przegubowej hierarchii;
- source ma `0` skinow;
- source primitive ma `0` lanes `JOINTS` i `0` lanes `WEIGHTS`;
- source ma `0` animacji;
- output ma `7` stanow type `5`, ale sa to `static-identity`, a nie motion
  odzyskany lub wygenerowany dla quadrupeda.

Wniosek: N1 moze zaliczyc tylko proof widocznosci nieruchomego rigid modelu.
Nie moze otrzymac statusu dzialajacego animowanego creature, nawet jezeli jego
base mesh bedzie widoczny w Toolsecie i NWN.

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
    - "ckdbckdie one-shot reaches a terminal pose before the cdead state"
    - "event proof records observed hit/footstep behavior or a named blocker"
  provenance:
    - "no external animation payload is committed or copied"
```

GB-005 ma status `DIRECTION_DEFINED_RUNTIME_OPEN`. Implementacja M4A zaczyna sie od self-contained `loader_smoke`, a nie od zalozenia, ze `c_Horror` bedzie dostepnym proof dependency.

## 8. Implementacja full selected profile -- 2026-07-25

`FullNative42ExplicitV1` jest obecnie wersjonowanym, opt-in profilem Core,
WASM i Studio. Wymaga exact 42-name namespace potwierdzonego przez own reader
na `c_Direwolf`, `c_horror` i CEP R3. Kazdy output clip musi pochodzic z
osobnego, jawnego source animation mappingu; profil nie uzupelnia brakow przez
klonowanie `cpause1`.

Poza samym namespace builder V2 uruchamia dwa fail-closed oracle:

- behavior oracle wymaga kontrolerow we wszystkich 42 klipach, ruchu we
  wszystkich stanach aktywnych zgodnych pomiedzy trzema rodzinami native,
  roznych semantyk `cpause1`/`cwalk`/`crun`/atak/damage/death oraz terminalnej
  pozy ruchomego `ckdbck`; dodatkowo wymaga ciaglosci granic
  `ckdbck -> ckdbckdie -> cdead`;
- SkinMesh oracle probkuje `cwalk`, `crun`, `ca1slashl`, `cdamagel` i
  `ckdbck` po own binary readbacku i wymaga rzeczywistej zmiany ksztaltu
  wazonej siatki. Sam rigid transform root nie przechodzi.

Wazna korekta semantyczna: `cdead` nie jest uniwersalnym one-shotem smierci.
Retail `c_Direwolf` trzyma tam statyczna poze, a CEP R3 ma zmienne kontrolery.
Analogicznie `ccastoutlp`, `cgetmidlp`, `ckdbckps` i `ckdbckdie` sa
family-variable. Kontrakt wymaga dla nich jawnego payloadu kontrolerow, ale nie
narzuca globalnie ruchu ani bezruchu. Widoczny upadek jest sprawdzany w
`ckdbck`, a dwa kolejne stany musza zaczynac sie od pozy koncowej poprzednika.

### MESHY_DEAD_CONTINUOUS_DEATH_FAMILY_V2 — korekta 2026-07-28

Pierwszy owner runtime test Powrotnika sugerowal, ze jawny ruch Meshy `Dead`
przypisany do `cdead` zostal poprzedzony osobnym proceduralnym ruchem
`ckdbckdie`. V2 zastapilo `ckdbckdie` nieruchomym bridge'em, lecz drugi owner
runtime test wykazal dokladnie to samo zachowanie. V3 przenioslo Meshy `Dead`
do `ckdbckdie`, ale trzeci owner test nadal wykazal odskok.

Exact binary audit V3 potwierdzil pomijana wczesniej granice:

- V1, V2 i V3 zachowaly ten sam ruchomy `ckdbck` z osobnego Meshy
  `Knock_Down`;
- koniec tego `ckdbck` i poczatek Meshy `Dead` w V3 roznily sie o
  `2.837246 m` na `Hips`, `77.989°` na rotacji `Hips` i do `84.386°` na
  kosciach;
- exact retail `c_squirrel` ma na tej samej granicy `0 m` oraz maksymalnie
  `0.051°`.

Wlasciciel obejrzal offline preview wszystkich `90` klatek exact Meshy action
`8 Dead` i potwierdzil, ze caly klip `3.0 s` jest oczekiwanym, pojedynczym
ruchem zgonu. Klipu nie nalezy przycinac.

Profil proceduralny stosuje teraz jednoznaczna regule:

1. caly Meshy `Dead` bez zmian czasu, eventow i klatek zastępuje `ckdbck`;
2. osobny Meshy `Knock_Down` nie jest doklejany przed nim w death-family;
3. `ckdbckps`, `ckdbckdie` i `cdead` sa holdami `1/30 s` z ostatniej pozy
   Meshy `Dead`;
4. behavior oracle dopuszcza dla tych holdow ruch albo bezruch, ale zawsze
   wymaga ciaglosci pozycji do `0.001 m` i rotacji do `1°`;
5. nieciaglosc `ckdbck -> ckdbckdie` albo `ckdbckdie -> cdead` blokuje
   materializacje;
6. SkinMesh oracle probkuje `ckdbck` jako wlasciciela widocznego upadku.

Kontrakt runtime brzmi: `ckdbck` odtwarza caly Meshy `Dead` dokladnie raz,
po czym silnik przechodzi przez ciagle terminalne stany bez drugiego upadku.

Granice:

- legacy V1 nadal ma 7-state idle-fallback, aby nie zmieniac zamrozonych
  lineage i ich hashy;
- full V2 nie deklaruje, ze przejscie oracle dowodzi artystycznej jakosci ruchu
  ani state routingu zamknietego engine'u;
- syntetyczna fixture ma 42 niezalezne source payloady, trzy jawne stany
  family-variable oraz ruch child-bone powodujacy nierigid deformacje;
- finalne zamkniecie nadal wymaga runtime proofu callbackow eventow,
  petli/state routingu oraz hash-bound owner proof w NWN.

## 9. Caller-owned event authoring V1 -- 2026-07-25

Decyzja o eventach jest zamknieta strukturalnie, ale nie runtime'owo. Own
reader potwierdza dokladny wspolny floor trzech niezaleznych rodzin:
retail `c_Direwolf`, retail `c_horror` i CEP R3 `c_phod_horror_b`.
Wspolna czesc zawiera dokladnie 23 pary `(clip, event)`:

| Clip | Event |
|---|---|
| `ca1slashl` | `hit` |
| `ca1slashr` | `hit` |
| `ca1stab` | `hit` |
| `ca1stab` | `snd_footstep` |
| `ccastout` | `cast` |
| `ccloseh` | `hit` |
| `cclosel` | `hit` |
| `ccturnr` | `snd_footstep` |
| `ccwalkb` | `snd_footstep` |
| `ccwalkf` | `snd_footstep` |
| `ccwalkl` | `snd_footstep` |
| `ccwalkr` | `snd_footstep` |
| `cdamagel` | `snd_footstep` |
| `cdamager` | `snd_footstep` |
| `cdamages` | `snd_footstep` |
| `cdodgelr` | `snd_footstep` |
| `cdodges` | `snd_footstep` |
| `ckdbck` | `snd_hitground` |
| `creach` | `hit` |
| `creach` | `snd_footstep` |
| `crun` | `snd_footstep` |
| `ctaunt` | `snd_footstep` |
| `cwalk` | `snd_footstep` |

Ten floor jest profilem pokrycia, nie zrodlem timingow. Produkt nie kopiuje
czasow, keyframes ani innych payloadow witnessow. Uzytkownik podaje wlasny
strict JSON V1:

```json
{
  "schemaVersion": 1,
  "clips": [
    {
      "clipName": "cwalk",
      "events": [
        { "timeSeconds": 0.25, "name": "snd_footstep" }
      ]
    }
  ]
}
```

`DirectCreatureEventAuthoringV1` wymaga unikalnych nazw klipow po ASCII
case-fold. Nazwa eventu musi byc niepusta, ASCII, bez NUL i miec najwyzej
31 bajtow. Czas musi byc skonczony i nalezec do `0..clip.length`. Eventy
nieznane sa zachowywane, ale raportowane.

Core V3 naklada caller-owned tabele po materializacji exact 42 stanow, zapisuje
MDL, ponownie czyta jego dokladne bajty i dopiero z binary readbacku wymaga
`23/23`. Raport zawiera:

- `animationEventConformance`;
- kanoniczny `byteLength` i SHA-256 sparsowanego sidecara w
  `animationEventAuthoringCanonical`.

WASM udostepnia `buildMeshyH1ModelPackageV3`. Worker ma osobny lane
`H1_SKINNED_FULL_42_EVENTS`, a Studio przekazuje wybrany
`animation-events.json` bez zmiany. Sidecar z placeable albo source bez exact
42 stanow jest odrzucany fail-closed. Legacy V1 i V2 zachowuja dotychczasowe
bajty i nie dostaja niejawnych eventow.

Zamkniete offline:

- strict schema, nazwy i zakres czasu;
- deterministyczny zapis i own binary readback;
- dokladny floor 23 par;
- brakujaca para, malformed JSON i niepelny profil sa bledami;
- Core, WASM, Worker i Studio przenosza ten sam caller-owned payload.

Nadal otwarte runtime:

- czy NWN wywoluje `hit`, `cast` i sound callback w oczekiwanym momencie;
- engine loop i state routing;
- artystyczna/gameplayowa jakosc caller-owned timingow.

Dlatego status GB-005 pozostaje `DIRECTION_DEFINED_RUNTIME_OPEN`: implementacja
event payloadu jest kompletna offline, lecz callback semantics wymagaja
hash-bound owner proof w NWN.
