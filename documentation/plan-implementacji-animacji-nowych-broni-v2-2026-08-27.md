# Plan implementacji animacji nowych broni V2

Data: 2026-08-27

Status: `PLAN GOTOWY DO REALIZACJI / BRAK NOWEGO KANDYDATA`

Dokument bazowy:
`documentation/audyt-warstw-animacji-broni-aurora-2026-08-27.md`.

## Cel

Dodać do pipeline opcjonalny, wielokrotnego użytku system konfiguracji animacji
nowej broni, który:

- pozwala wybrać model postaci/carrier, rodzinę zachowania broni i profil pozy;
- zachowuje podstawowe `pause1`, `walk` i `run` postaci;
- składa bazową lokomocję z warstwami broni ograniczonymi przez `animroot`;
- rozróżnia warstwy częściowe od pełnocielesnych akcji `ready` i `shot`;
- pokazuje w aplikacji dokładny wynik eksportu, a nie osobną jego imitację;
- nie modyfikuje globalnie `pfh0` ani innego współdzielonego modelu postaci;
- jest rozszerzalny o następne rodziny broni bez dopisywania wyjątków do
  pipeline przedmiotów.

## Najważniejsza granica silnika

UTI nowej broni nie może samodzielnie wybrać dowolnej nowej animacji postaci.
`baseitems.2da.WeaponWield` kieruje broń do istniejącej rodziny zachowania
silnika. Dlatego produkt musi obsługiwać dwa jawnie rozdzielone tryby:

| Tryb | Wynik | Zakres |
| --- | --- | --- |
| `NATIVE_WEAPON_FAMILY` | nowy model itemu korzysta z istniejących animacji Aurora | nie generuje modelu postaci ani lokalnych klipów |
| `CUSTOM_LAYERED_CARRIER` | wybrany, dedykowany carrier otrzymuje własne warstwy i akcje | wymaga własnego resrefu i jawnego powiązania z appearance/phenotype |

Tryb globalnego przesłaniania współdzielonego `pfh0`, `pmh0` lub innego
retailowego carriera nie należy do V2 i ma być blokowany.

## Docelowy przepływ

| Krok | Dane wejściowe | Wynik |
| --- | --- | --- |
| 1. Item | model broni, BaseItem, `WeaponWield` | natywna rodzina zachowania |
| 2. Binding | wybrany carrier, rodzina i profil pozy | wersjonowany `WeaponAnimationBindingV2` |
| 3. Resolver | własny reader + read-only supermodel chain | lista baz, warstw, akcji i ich zakresów |
| 4. Composer | clean-room profile pozy + strukturalne invarianty retail | własne klipy częściowe/pełnocielesne |
| 5. Validator | binding, hierarchia i controller coverage | PASS albo nazwany błąd fail-closed |
| 6. Writer | zwalidowany IR | binary MDL typu 5 i raport semantic readback |
| 7. Preview | exact binary readback + rozwiązany supermodel chain | ta sama kompozycja stanu co w eksporcie |
| 8. Package | item, opcjonalny dedykowany carrier, 2DA/HAK/MOD | hash-bound handoff do owner proof |

## Kontrakt danych

### `WeaponAnimationProfileV2`

Profil jest wpisem w rejestrze, nie warunkiem `if` dla konkretnej strzelby.
Zawiera:

- stabilne `profileId` i `familyId`;
- zgodne wartości `WeaponWield` i wymagane cechy BaseItem;
- pięć logicznych stanów: `IDLE`, `WALK`, `RUN`, `READY`, `SHOT`;
- dla każdego stanu bazę, opcjonalną warstwę, nazwę slotu runtime, rodzaj
  zakresu i oczekiwany `animroot`;
- dozwolone eventy oraz wymagania przejścia;
- wersjonowany clean-room pose profile bez keyframe'ów skopiowanych z retailu;
- wymagania względem hooka, osi broni i punktów chwytu.

Minimalne profile pierwszej implementacji:

1. `BOW_NATIVE_V1` — natywny `WeaponWield=5`, bez custom carriera.
2. `CROSSBOW_NATIVE_V1` — natywny `WeaponWield=6`, bez custom carriera.
3. `FIREARM_CROSSBOW_LAYERED_V2` — własna pozycja strzelby na audytowanej
   rodzinie routingu kuszy, wyłącznie z dedykowanym carrierem.

Rejestr ma być rozszerzalny o `ONE_HANDED`, `TWO_HANDED`, `SHIELD` i kolejne
rodziny dopiero po dodaniu exact retail witness oraz testu zakresu ich stanów.
Nieznana albo nieudowodniona rodzina jest blokowana, nie zgadywana.

### `WeaponAnimationBindingV2`

Binding zapisuje wybór użytkownika i tożsamość źródeł:

- `mode`;
- `profileId`;
- `sourceCarrierResref` i SHA-256;
- pełny, uporządkowany supermodel lineage z hashami;
- `outputCarrierResref` tylko dla trybu custom;
- wybrany model itemu, model set i hook path;
- hash profilu pozy i transformacji broni;
- rozwiązany plan stanów;
- politykę zakresu: `INHERITED_BASE`, `PARTIAL_SUBTREE` albo
  `FULL_BODY_ACTION`;
- wynik walidacji i hash całego bindingu.

Zmiana modelu postaci, modelu broni, hooka, profilu albo transformacji unieważnia
binding, preview i dotychczasowy build.

### Rozdzielenie od amunicji

`wielderClip` nie jest cechą amunicji. Należy go usunąć z
`ItemRangedAmmunitionDraftV1` i przenieść do `WeaponAnimationBindingV2`.
Amunicja nadal opisuje kanał `ARROW/BOLT/BULLET`, projectile, obrażenia i
zasoby przedmiotu. Rodzina animacji wynika atomowo z BaseItem, `WeaponWield` i
profilu animacji, a nie z modelu pocisku.

## Plan implementacji

### Etap 0 — zamknięcie regresji V14 testem

Najpierw testy, bez generowania nowego MOD/HAK:

- dodać minimalną syntetyczną fixture odpowiadającą błędowi V14: `pause1`,
  `xbowrdy`, `xbowshot`, `animroot=rootdummy`, tylko 10 ścieżek góry ciała;
- wymagać odrzucenia jej kodem
  `WEAPON-ANIMATION-FULL-BODY-COVERAGE-INCOMPLETE`;
- utrzymać env-gated exact retail witnesses dla `pfh0`, `a_fa`, `a_ba` i
  `a_ba_non_combat`;
- rozszerzyć witness o `walk`, `run`, `walk_bowl`, `run_bowl`, `xbowr`,
  `xbowrdy`, `xbowshot`, `animroot` oraz zbiory kontrolowanych węzłów.

Zakończenie: test błędu V14 jest czerwony przed implementacją i zielony po
dodaniu walidatora.

### Etap 1 — domena i rejestr profili w Rust

Utworzyć w core moduł `weapon_animation` zawierający:

- `WeaponAnimationModeV2`;
- `WeaponAnimationFamilyV2`;
- `WeaponAnimationStateKindV2`;
- `WeaponAnimationScopeV2`;
- `WeaponAnimationProfileV2`;
- `WeaponAnimationBindingV2`;
- deterministyczny rejestr profili i funkcję rozwiązującą profil względem
  `baseitems.2da`.

Core, a nie React, jest właścicielem zgodności `BaseItem + WeaponWield +
profileId`. TypeScript otrzymuje projekcję raportu z WASM.

Zakończenie: wybór niezgodnego profilu jest blokowany nazwanym błędem; dwa
identyczne wejścia dają byte-identical JSON bindingu i identyczny SHA-256.

### Etap 2 — resolver supermodeli i stanów

Rozszerzyć własny reader o raport rozwiązanej hierarchii animacji dla jednego
wybranego carriera:

- wykrywanie źródła każdego stanu w uporządkowanym supermodel chain;
- normalizację nazw ASCII case-insensitive;
- raport `animroot`, długości, transition, events i zbioru controller targets;
- klasyfikację controller targets względem poddrzewa `animroot`;
- osobne oznaczenie baz, warstw i akcji;
- wykrywanie lokalnego shadowing stanu odziedziczonego.

Retail i CEP pozostają read-only. Do artefaktów projektu nie kopiujemy ich
keyframe'ów, szkieletów ani payloadów.

Zakończenie: exact witness i syntetyczna fixture dają ten sam semantyczny plan
stanów, a brak wymaganego stanu zatrzymuje tryb custom.

### Etap 3 — clean-room composer warstw i akcji

Zastąpić `buildFirearmRuntimeAnimationExportV1` generycznym builderem V2:

- tryb natywny nie emituje lokalnego modelu postaci;
- `IDLE`, `WALK` i `RUN` zachowują odziedziczone pełnocielesne bazy;
- custom rest/walk/run emituje wyłącznie własne warstwy częściowe;
- `READY` i `SHOT` są własnymi, kompletnymi akcjami pełnocielesnymi albo
  pozostają odziedziczone — nie wolno emitować ich jako `rootdummy` z samą
  górą ciała;
- dane dolnej części własnych pełnocielesnych akcji są generowane clean-room z
  profilu pozy i własnego rig IR, nie kopiowane z retailu;
- transformacja itemu, pozycja dłoni i punkty kontaktu pozostają osobnym
  profilem i nie zmieniają zamrożonej geometrii broni.

Stary `RUNTIME_CLIPS_V1` z mapowaniem `firearm_idle_custom -> pause1` zostaje
usunięty. `pause1` nie jest lokalnym slotem custom firearm.

Zakończenie: wygenerowany plan nie zawiera częściowego `pause1`; numeryczne
porównanie klatek `base` i `base + overlay` daje identyczne macierze dla
korzenia, miednicy i wszystkich kości nóg.

### Etap 4 — walidatory core i writer gates

Dodać przed zapisem binary MDL następujące bramki:

- `animroot` istnieje w hierarchii;
- każdy controller target leży w poddrzewie zadeklarowanego `animroot`;
- `PARTIAL_SUBTREE` nie zawiera zabronionych węzłów dolnej części ciała;
- `FULL_BODY_ACTION` spełnia wymagane pokrycie strukturalne profilu;
- `INHERITED_BASE` nie może być lokalnie emitowany;
- lokalna nazwa klipu nie przesłania bazowego stanu poza jawną, dozwoloną
  akcją profilu;
- wyjściowy model custom nie może mieć resrefu wspólnego retailowego carriera;
- każdy stan ma osobny oczekiwany `animationRoot`; usunąć pojedyncze
  `expectedAnimationRoot` obowiązujące dziś dla wszystkich klipów;
- usunąć ogólne `allowRuntimeOverride: true` jako przepustkę bezpieczeństwa.
  Zastąpić je zwalidowanym, hash-bound bindingiem V2.

Zakończenie: writer nie jest w stanie wyprodukować kontraktu V14 nawet po
ominięciu UI.

### Etap 5 — dedykowany carrier i zakres package

Dla `CUSTOM_LAYERED_CARRIER`:

- użytkownik wybiera model postaci z istniejącego katalogu;
- pipeline wyznacza nowy, wolny resref output carriera bez nadpisywania źródła;
- raport zapisuje source carrier, supermodel chain, output carrier i wszystkie
  hashe;
- package zawiera jawne powiązanie z dedykowanym appearance/phenotype lub
  proof NPC;
- package nie deklaruje, że sama UTI aktywuje animację postaci;
- kolizja resrefu jest fail-closed, bez overwrite i bez cichego przydzielenia
  innego kandydata proof.

Zakończenie: HAK nie zawiera `pfh0.mdl` ani innego retailowego carriera pod tą
samą nazwą; raport określa dokładnie, która postać używa profilu.

### Etap 6 — opcja w Item Workflow

Dodać opcjonalną sekcję `Animacje postaci dla broni`:

1. przełącznik `Użyj natywnej rodziny` / `Własny profil warstwowy`;
2. wybór modelu postaci/carriera;
3. wybór zgodnej rodziny animacji;
4. wybór reusable pose profile;
5. wybór model setu itemu i hooka;
6. tabela pięciu stanów pokazująca bazę, warstwę/akcję, `animroot`, scope i
   źródło;
7. jawne ostrzeżenie, gdy zmiana dotyczy dedykowanego appearance zamiast samej
   UTI;
8. statusy `VALID`, `INVALID`, `READBACK_PENDING`, `OWNER_PROOF_REQUIRED`.

Wybór ma być częścią sesji/recipe projektu, a nie lokalnym stanem samego
viewportu. Zmiana opcji unieważnia build i wymaga ponownego semantic readback.

Zakończenie: użytkownik może wybrać inny carrier i inny zgodny profil bez
zmiany kodu albo używania strony proof.

### Etap 7 — preview z exact export readback

Przebudować `ItemRuntimePosePreview` na trzy jasno opisane źródła:

- `NATIVE REFERENCE` — tylko surowy, read-only stan referencyjny;
- `AUTHORING` — regulacja pozy przed eksportem, bez deklaracji parity;
- `EXACT EXPORT READBACK` — domyślny widok akceptacyjny po buildzie.

Exact view rozwiązuje ten sam binary carrier i supermodel chain co package oraz
odtwarza:

- `IDLE`: baza + rest overlay;
- `WALK`: baza + walk overlay;
- `RUN`: baza + run overlay;
- `READY`: pełna akcja;
- `SHOT`: pełna akcja wraz z eventami.

UI pokazuje SHA-256 modelu, bindingu i package oraz oznacza, które kości pochodzą
z bazy, warstwy i akcji. Nie wolno przedstawiać `AUTHORING` jako proofu NWN.

Zakończenie: podgląd i artefakt do pobrania korzystają z tego samego bufora MDL
i mają identyczny SHA-256. Test wykrywa każdą próbę zbudowania podglądu z
osobnych, niewyeksportowanych klipów.

### Etap 8 — integracja Item/Ammunition/Worker/WASM

- przenieść routing klipu z `rangedAmmunition` do bindingu animacji;
- rozszerzyć request Workera o `weaponAnimationBindingV2` zamiast luźnego
  `runtimeCharacterAnimation` V1;
- przekazywać do WASM zwalidowany binding, nie dowolne JSON-y modelu i klipów;
- rozszerzyć raport builda o resolved state plan, controller coverage,
  carrier scope, readback hashes i status każdej bramki;
- utrzymać niezależność modelu pocisku, amunicji, modelu itemu i animacji
  postaci;
- zachować istniejącą geometrię, tekstury, ikonę i amunicję strzelby bez zmian.

Zakończenie: wybór amunicji nie zmienia profilu postaci, a zmiana profilu
animacji nie zmienia modelu pocisku ani jego obrażeń.

### Etap 9 — testy, migracja i handoff

Uruchomić kolejno:

- unit tests core dla rejestru, bindingu, zakresów i błędów;
- writer/semantic readback tests;
- env-gated exact retail witness tests;
- WASM boundary tests z błędnymi oraz poprawnymi bindingami;
- React tests wyboru carriera, profilu, hooka i invalidacji builda;
- browser test pięciu stanów exact export readback;
- pełny workspace typecheck i pakiet testów item pipeline.

V1 nie jest migrowany przez zachowanie błędnych danych. Sesja zawierająca
`runtimeCharacterAnimation.schemaVersion=1` otrzymuje status
`MIGRATION_REQUIRED` i wymaga ponownego wyboru trybu/profilu. Zamrożony V14
pozostaje wyłącznie negatywnym świadkiem.

Po przejściu wszystkich bramek można przygotować jeden następny dopuszczony
kandydat. Agent kończy na `ready_for_owner_proof`; Toolset/NWN oraz werdykt
wizualny należą do właściciela.

## Mapa zmian w kodzie

| Obszar | Docelowa zmiana |
| --- | --- |
| `crates/m2a-core/src/weapon_animation.rs` | nowa domena, rejestr, binding, resolver i walidacja |
| `crates/m2a-core/src/mdl/write_binary_mdl.rs` | generic subtree/coverage oraz local-shadow gates |
| `crates/m2a-core/src/mdl/semantic_readback.rs` | raport controller targets i resolved clip scope |
| `crates/m2a-core/tests/runtime_witness_conformance.rs` | exact retailowe kontrakty baz/warstw/akcji |
| `crates/m2a-core/tests/weapon_animation.rs` | syntetyczne testy V14 i poprawnych profili |
| `crates/m2a-wasm/src/lib.rs` | wersjonowane API binding/build/readback V2 |
| `apps/studio-web/src/features/preview/firearmRuntimeAnimation.ts` | usunięcie V1 lub adapter migracyjny bez materializacji |
| `apps/studio-web/src/features/item/rangedAmmunition.ts` | usunięcie `wielderClip` |
| `apps/studio-web/src/features/item/types.ts` | projekcja bindingu i raportu V2 |
| `apps/studio-web/src/features/item/ItemWorkflow.tsx` | opcjonalna konfiguracja animacji i trwały wybór |
| `apps/studio-web/src/features/item/ItemRuntimePosePreview.tsx` | exact export readback oraz pięć stanów |
| `apps/studio-web/src/worker/types.ts` | request V2 bez `allowRuntimeOverride` |
| `apps/studio-web/src/worker/m2a.worker.ts` | materializacja wyłącznie po walidacji core |

## Kryteria zakończenia całej implementacji

Implementacja jest zakończona dopiero, gdy wszystkie punkty są spełnione:

- istnieje opcjonalna, reusable konfiguracja animacji nowej broni w głównym
  Item Workflow;
- model postaci, profil animacji, model itemu i hook są wybieralne i zapisane w
  projekcie;
- zgodność profilu z BaseItem i `WeaponWield` jest egzekwowana w Rust;
- tryb natywny nie generuje żadnego modelu postaci;
- tryb custom generuje wyłącznie dedykowany carrier i nie przesłania `pfh0`;
- `pause1`, `walk` i `run` pozostają odziedziczone;
- dolna część ciała jest numerycznie identyczna przed i po nałożeniu warstw
  `IDLE/WALK/RUN`;
- częściowy klip pod `rootdummy` jest zawsze odrzucany;
- `READY/SHOT` są odziedziczone albo mają kompletne, clean-room ścieżki
  pełnego ciała;
- exact export semantic readback potwierdza nazwy, `animroot`, controller
  coverage, eventy, supermodel i carrier identity;
- preview i pobierany MDL są tym samym hash-bound artefaktem;
- wybór amunicji nie steruje animacją postaci;
- test regresji V14 przechodzi jako oczekiwane odrzucenie;
- wszystkie nowe testy core, WASM, React i browser przechodzą;
- nie powstał drugi source root, nie skopiowano retailowych payloadów i nie
  zmieniono zamrożonej geometrii broni;
- następny pojedynczy kandydat ma kompletny handoff `ready_for_owner_proof`;
- właściciel potwierdza w NWN osobno: idle, walk, run, ready i shot, bez
  deformacji nóg i bez rozjazdu względem exact export preview.

## Kolejność dostarczania

Etapy `0 -> 4` tworzą obowiązkowy bezpieczny fundament. Dopiero potem wolno
podłączyć UI i packaging z etapów `5 -> 8`. Etap 9 zamyka offline implementation
gate i przygotowuje jeden kandydat do owner proof. Nie wolno wcześniej
materializować kolejnego demo wyłącznie po to, aby ręcznie sprawdzić hipotezę.
