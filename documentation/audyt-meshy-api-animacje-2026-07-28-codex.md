# Audyt Meshy API pod katem animacji (2026-07-28)

Status: `AUDIT_COMPLETE / IMPLEMENTATION_GAPS_FOUND / NO_PAID_API_CALL`

Zakres audytu:

- oficjalne, publiczne dokumenty Meshy odczytane 2026-07-28;
- branch `animation`, baza kodu `0a3f80a`;
- Meshy Local Bridge, Meshy Lab i lokalny pipeline GLB -> Aurora;
- rigging, pojedyncze i wieloklipowe pozyskiwanie animacji, koszty,
  recovery, provenance i zgodnosc z profilem animacji Aurory.

Nie uzyto klucza API, nie utworzono zadnego zadania Meshy i nie naliczono
kredytow. Audyt API jest dokumentacyjny; nie jest realnym E2E ani wizualnym
proofem modelu lub animacji w Aurora Toolset/NWN.

## 1. Werdykt

Meshy API jest przydatne jako zrodlo **presetowych klipow ruchu dla jednego
narigowanego humanoida**. Nie jest bezposrednim generatorem kompletnego profilu
42 stanow Aurory:

- rigging API jest oficjalnie ograniczone do teksturowanych, standardowych
  humanoidow o czytelnych konczynach;
- jedno zadanie Animation stosuje jeden `action_id` do jednego zakonczonego
  zadania Rigging i zwraca jeden animowany GLB/FBX;
- API nie mapuje nazw stanowych Aurory, nie dostarcza eventow NWN ani
  machine-readable danych o petli, root motion, czasie trwania lub semantyce
  klipu;
- wiele klipow wymaga wielu zadan Animation na tym samym `rig_task_id`, a potem
  lokalnej walidacji, jawnego mapowania nazw i deterministycznego polaczenia.

Aktualny Bridge potrafi wykonac waski lancuch:

`generation -> rigging -> jedna animation -> jeden zweryfikowany GLB`

To wystarcza do eksperymentu z jednym `Idle`, ale przed platnym wieloklipowym
E2E wymaga poprawek P0. Najwazniejsze z nich to: katalog/allowlista akcji,
prawdziwy preflight humanoida, dynamiczny koszt, recovery taskow Rig/Animation,
wieloklipowy manifest i pelne provenance.

Rekomendacja produktowa: najpierw wykonac kontrolowany slice siedmiu stanow
gameplay, a dopiero po pomiarze rzeczywistych GLB decydowac, czy pozyskiwac
z Meshy wiecej klipow. Nie kupowac w ciemno 42 presetow.

## 2. Klasyfikacja zrodel

W dalszej czesci:

- **Fakt Meshy API** oznacza stan oficjalnej dokumentacji 2026-07-28;
- **Fakt repo** oznacza odczyt kodu brancha `animation`;
- **Wniosek implementacyjny** oznacza decyzje wynikajaca z porownania obu
  kontraktow;
- **Hipoteza do proofu** oznacza zachowanie wymagajace realnego GLB lub
  owner-run proofu, a nie podstawe gotowej implementacji.

Glowne oficjalne zrodla:

- [Rigging API](https://docs.meshy.ai/en/api/rigging)
- [Animation API](https://docs.meshy.ai/en/api/animation)
- [Animation Library Reference](https://docs.meshy.ai/en/api/animation-library)
- [API Pricing](https://docs.meshy.ai/en/api/pricing)
- [Asset Retention](https://docs.meshy.ai/en/api/asset-retention)
- [Errors](https://docs.meshy.ai/en/api/errors)
- [Rate Limits](https://docs.meshy.ai/en/api/rate-limits)
- [API Changelog](https://docs.meshy.ai/en/api/changelog)
- [Webapp Animation guide](https://docs.meshy.ai/en/webapp/guides/animate)

## 3. Potwierdzony kontrakt Rigging API

### 3.1 Wejscie

**Fakt Meshy API:** `POST /openapi/v1/rigging` przyjmuje dokladnie jedno z:

- `input_task_id` zakonczonego zadania Meshy;
- `model_url` wskazujace publiczny URL albo Data URI teksturowanego GLB.

Gdy oba pola sa obecne, `input_task_id` ma pierwszenstwo. Dodatkowe pola:

- `height_meters`: dodatnia liczba, domyslnie `1.7`;
- `texture_image_url`: opcjonalna, UV-unwrapped tekstura bazowa PNG.

Udokumentowane granice:

- dobry przypadek to standardowy humanoid/biped o czytelnych konczynach;
- nie sa wspierane siatki bez tekstury, non-humanoid i nieczytelna anatomia;
- dla `input_task_id` limit riggingu wynosi 300 000 faces;
- przy `model_url` przod postaci musi wskazywac `+Z`, inaczej pose estimation
  moze sie nie udac;
- pose estimation moze zakonczyc sie `422`.

Limit 300 000 jest granica Meshy API. Od decyzji wlasciciela z 2026-07-29
wspolny budzet produktu ma te sama wartosc, ale pozostaje osobnym kontraktem:

`AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300_000`

### 3.2 Odczyt i wynik

Oficjalnie dostepne sa:

| Operacja | Endpoint |
|---|---|
| Create | `POST /openapi/v1/rigging` |
| Get | `GET /openapi/v1/rigging/:id` |
| List | `GET /openapi/v1/rigging?page_num=...&page_size=...` |
| Stream | `GET /openapi/v1/rigging/:id/stream` |
| Delete | `DELETE /openapi/v1/rigging/:id` |

**Fakt Meshy API:** zakonczony task zwraca:

- `rigged_character_glb_url`;
- `rigged_character_fbx_url`;
- opcjonalne `basic_animations`:
  - walking GLB/FBX i armature GLB;
  - running GLB/FBX i armature GLB.

Task object zawiera tez status, progress, `consumed_credits`, kolejke,
timestamps i `expires_at`. Status moze byc jednym z:

`PENDING | IN_PROGRESS | SUCCEEDED | FAILED | CANCELED`

**Wniosek implementacyjny:** rigged GLB oraz opcjonalne walking/running sa
wartosciowymi, osobnymi artefaktami. Nie wolno ich tracic tylko dlatego, ze
kolejny task Animation zwrocil jeden wybrany klip.

## 4. Potwierdzony kontrakt Animation API

### 4.1 Utworzenie taska

`POST /openapi/v1/animations` wymaga:

```json
{
  "rig_task_id": "completed-rig-task-id",
  "action_id": 0
}
```

`rig_task_id` musi wskazywac pomyslnie zakonczone zadanie Rigging.
`action_id` musi istniec w oficjalnej bibliotece.

Opcjonalny `post_process` wspiera:

| `operation_type` | Znaczenie | Dodatkowe pole |
|---|---|---|
| `change_fps` | zmiana FPS | `fps`: `24`, `25`, `30` albo `60` |
| `fbx2usdz` | konwersja do USDZ | brak |
| `extract_armature` | armature output | brak |

Standardowy task zwraca `animation_glb_url` i `animation_fbx_url`. Zaleznie od
post-processingu moze zwrocic tez processed USDZ, armature FBX albo FBX ze
zmienionym FPS.

### 4.2 Lifecycle

Oficjalnie dostepne sa:

| Operacja | Endpoint |
|---|---|
| Create | `POST /openapi/v1/animations` |
| Get | `GET /openapi/v1/animations/:id` |
| List | `GET /openapi/v1/animations?page_num=...&page_size=...` |
| Stream | `GET /openapi/v1/animations/:id/stream` |
| Delete | `DELETE /openapi/v1/animations/:id` |

Delete jest opisane jako nieodwracalne usuniecie taska i danych. Nie nalezy
uzywac go jako automatycznego zamiennika bezpiecznego "cancel".

Task ma ten sam terminalny zbior statusow co Rigging, w tym `CANCELED`, oraz
zwraca `consumed_credits`, `expires_at` i `preceding_tasks`.

## 5. Audyt biblioteki animacji

Oficjalna strona Animation Library zostala 2026-07-28 sparsowana read-only w
pamieci. Normalizowany rekord mial format:

`id|name|category|subcategory`

Wynik:

| Pole | Wartosc |
|---|---:|
| Liczba rekordow | 678 |
| Najmniejsze ID | 0 |
| Najwieksze ID | 696 |
| Liczba brakujacych ID w zakresie | 19 |
| `WalkAndRun` | 176 |
| `BodyMovements` | 158 |
| `DailyActions` | 157 |
| `Fighting` | 154 |
| `Dancing` | 33 |
| SHA-256 znormalizowanego katalogu | `5a512931ddfde95e2f2af4e8f80b9d1671e890a04cc27b7606d6e23c155b416b` |

Brakujace ID:

`332, 373, 374, 380, 383, 400, 418, 423, 424, 443, 454, 469, 600, 602, 603, 614, 633, 634, 655`

**Fakt Meshy API:** `0` to `Idle`. Katalog zawiera rowniez m.in.:

- `4` `Attack`;
- `8` `Dead`;
- `30` `Casual_Walk`;
- `92` `Double_Combo_Attack`;
- `125` `Charged_Spell_Cast`;
- `178` `Hit_Reaction`;
- `187` `Knock_Down`;
- `613` `Casual_Walk_inplace`;
- liczne warianty `_inplace` w wysokim zakresie ID.

**Istotna granica:** w przejrzanej oficjalnej dokumentacji nie ma
udokumentowanego JSON endpointu zwracajacego katalog akcji. "List Animation
Tasks" listuje taski uzytkownika, nie presety. Biblioteka jest referencja HTML z
GIF preview.

Katalog nie publikuje machine-readable:

- czasu trwania;
- FPS;
- loop/one-shot;
- root motion kontra in-place;
- zakresu translacji;
- sygnatury szkieletu;
- listy kanalow;
- kompatybilnosci humanoid/quadruped per akcja.

**Wniosek implementacyjny:** UI nie moze przyjmowac dowolnej liczby jako
`action_id`. Potrzebny jest wersjonowany, kuratorowany snapshot lub allowlista
wybranych akcji z data i hashem katalogu. Kazdy wybrany klip musi potem przejsc
lokalny readback GLB.

## 6. Rozjazd API kontra webapp

Oficjalny przewodnik webapp reklamuje auto-rig humanoidow i quadrupedow oraz
500+ presetow. Oficjalny **Rigging API** nadal mowi o standardowych humanoidach
i jawnie odrzuca non-humanoid.

Werdykt:

- **API:** N1/quadruped pozostaje `not_supported`;
- **webapp:** quadruped jest funkcja interaktywnego produktu Meshy;
- nie wolno przenosic claimu webapp do Local Bridge bez nowego, jawnego
  kontraktu API i testu.

Obecny profil N1 bez auto-riggingu jest zatem poprawnym fail-closed zakresem.

## 7. Koszt

Aktualny cennik API:

- Auto-Rigging: `5` kredytow;
- Animation: `3` kredyty za jeden klip;
- Text to 3D Meshy-6 Preview: `20`;
- Text to 3D Refine 2K/4K: `10`.

Dla `N` jawnie zamowionych klipow:

`koszt animacji po gotowym modelu = 5 + 3*N`

`koszt pelnego Text-to-3D Meshy-6 = 20 + 10 + 5 + 3*N`

| Scenariusz | Kredyty |
|---|---:|
| Obecny H1: generation + rig + 1 Idle | 38 |
| Gotowy model + rig + 1 klip | 8 |
| Generation + rig + 7 klipow gameplay | 56 |
| Gotowy model + rig + 7 klipow | 26 |
| Generation + rig + 42 klipy | 161 |
| Gotowy model + rig + 42 klipy | 131 |

Opcjonalne walking/running zwracane przez Rigging moga zmniejszyc liczbe
dodatkowych taskow Animation, ale dopiero gdy realna odpowiedz zawiera te pola,
a oba GLB przejda identity, skeleton i motion gates. Dokumentacja nazywa
`basic_animations` opcjonalnym; nie mozna zakladac ich obecnosci w kalkulatorze
kosztu.

**Wniosek implementacyjny:** stale `38` jest poprawnym maximum tylko dla
obecnego, jednego klipu H1 przy tej konkretnej trasie. Wieloklipowy review musi
liczyc koszt dynamicznie przed utworzeniem pierwszego taska.

## 8. Audyt aktualnego Meshy Local Bridge

Mapa przejrzanego kodu:

| Plik | Zakres audytu |
|---|---|
| `tools/meshy-local-bridge/index.mjs` | profile H1/N1/S1, walidacja opcji, create/poll/download, historia i lokalny cancel |
| `apps/studio-web/src/features/meshy/MeshyLab.tsx` | preflight H1, pole `action_id`, review i target geometrii |
| `apps/studio-web/src/features/meshy/bridge.ts` | domyslne opcje, typy request/response i provenance |
| `crates/m2a-core/src/direct_creature_animation.rs` | docelowe nazwy i wymagany profil animacji Aurory |
| `documentation/animacje-kontrakt-profil-a-codex.md` | floor 7, pelny profil 42, eventy i bramki readback/proof |

### 8.1 Co jest poprawne

| Obszar | Fakt repo |
|---|---|
| Sekret | Klucz pozostaje w loopback Bridge; Studio nie wywoluje Meshy bezposrednio. |
| Narrow proxy | Bridge wywoluje jawne endpointy, nie arbitralny URL podany przez UI. |
| Platna operacja | Preview, saldo i jednorazowy confirmation nonce istnieja przed create. |
| H1 | Rigging i Animation sa uruchamiane sekwencyjnie. |
| Artefakt | Signed URL nie trafia do UI; Bridge pobiera GLB, sprawdza magic, rozmiar i SHA-256. |
| N1/S1 | Nie wywoluja humanoid-only Rigging/Animation. |

### 8.2 Luki P0

#### P0-1: dowolny `action_id`

**Fakt repo:** `validApiOptions` przyjmuje kazda nieujemna liczbe calkowita.
UI pokazuje surowe pole numeryczne. ID nie jest sprawdzane wzgledem katalogu,
a katalog ma luki.

Skutek: uzytkownik moze wyslac nieistniejace albo usuniete ID i otrzymac
`400 Invalid action ID`. UI nie pokazuje nazwy, kategorii ani semantyki klipu.

Wymagane: selector z kuratorowanej allowlisty, nazwa i preview, hash snapshotu
katalogu oraz fail-closed odrzucenie ID spoza snapshotu.

#### P0-2: preflight H1 nie jest rzeczywistym potwierdzeniem

**Fakt repo:** `MeshyLab.tsx` po wlaczeniu `rigHumanoid` automatycznie wysyla:

```ts
{ standardHumanoid: true, clearLimbs: true, noWeapon: true }
```

Uzytkownik nie potwierdza osobno tych trzech warunkow. `poseMode` moze pozostac
pusty, choc Meshy zaleca A/T pose jako najlepsze wejscie.

Skutek: gate wyglada na safety check, ale obecny UI sam deklaruje jego sukces.

Wymagane:

- trzy jawne, niezalezne potwierdzenia lub wynik lokalnej inspekcji;
- A-pose/T-pose jako domyslne wymaganie dla H1;
- tekstura, standardowy biped i czytelne konczyny pokazane w review;
- brak claimu o pewnym sukcesie pose estimation.

#### P0-3: tylko jeden klip

**Fakt repo:** `MeshyPipelineStage` ma pojedynczy klucz `ANIMATE`, a
`animationActionId` jest pojedyncza liczba. Executor tworzy jeden task i
pobiera tylko `animation_glb_url`.

Skutek: nie da sie zachowac wielu task IDs ani wielu GLB jednego rigu. Nie ma
atomowego, kosztowego review listy klipow.

Wymagane: `AnimationSelectionV1[]`, wiele task IDs powiazanych z jednym
`rig_task_id`, per-clip status/provenance i jawna polityka partial failure.

#### P0-4: wynik rigu i basic animations sa porzucane

Po zakonczonym rigu zmienna `output` zostaje nadpisana wynikiem Animation.
Bridge nie pobiera:

- `rigged_character_glb_url`;
- `basic_animations.walking_glb_url`;
- `basic_animations.running_glb_url`.

Skutek: tracimy bazowy dowod szkieletu i potencjalnie dwa klipy zawarte w
wyniku rigu. Awaria pozniejszego Idle pozostawia platny rig bez artefaktu w
produkcie.

Wymagane: pobrac i zahashowac rigged GLB natychmiast po `SUCCEEDED`; opcjonalne
basic GLB traktowac jako osobne role manifestu.

#### P0-5: brak recovery Rigging/Animation

**Fakt Meshy API:** oba typy taskow maja list/get. Taski utworzone przez API nie
pojawiaja sie w webapp `My Assets`.

**Fakt repo:** historia Bridge listuje tylko
`GET /openapi/v2/text-to-3d`. Runy i task IDs sa trzymane w pamieci procesu.

Skutek: restart Bridge po platnym rigu lub serii animacji moze odciac Studio od
wynikow, mimo ze Meshy pozwala je jeszcze listowac i pobrac.

Wymagane: list/get/recover dla `rig` i `animate`, zapis task ID natychmiast po
create oraz lokalny manifest bez signed URL.

#### P0-6: terminalne statusy i mylace "cancel"

`waitForTask` konczy na `SUCCEEDED` albo `FAILED`. Zdalny `CANCELED` nie jest
traktowany jako terminalny i moze skonczyc sie lokalnym timeoutem.

Lokalny `/cancel` ustawia tylko `run.status = "CANCELED"`. Nie anuluje taska
Meshy, ktory moze dalej pracowac i zuzyc kredyty. Oficjalne API dokumentuje
nieodwracalny DELETE, nie bezpieczny cancel request.

Wymagane:

- obsluzyc `CANCELED` jako terminal z osobnym kodem;
- nazwac obecna akcje `Stop tracking locally`, jesli pozostaje lokalna;
- nie wywolywac automatycznie DELETE;
- pokazywac, ze utworzony task Meshy moze kontynuowac.

#### P0-7: provenance nie wystarcza do odtworzenia animacji

Aktualne provenance zawiera profil, wersje Bridge, SHA-256, byte length i po
jednym task ID per etap. Brakuje:

- `action_id`, nazwy, kategorii i hasha katalogu;
- `rig_task_id` jako wspolnej tozsamosci wielu klipow;
- `height_meters`, source task/model hash i pose;
- rzeczywistego `consumed_credits`;
- `x-api-version`, timestamps i `expires_at`;
- hasha bazowego rigged GLB;
- sygnatury szkieletu;
- clip name/duration/channel/root-motion inventory z naszego GLB readera.

Wymagane: `MeshyAnimationArtifactManifestV1` bez signed URL, z osobna rola dla
rigged base i kazdego klipu.

#### P0-8: domyslny target przekraczal dawny produktowy budzet — zamkniete

**Fakt repo:** domyslne `targetPolycount` i UI `BALANCED` prowadza do 30 000.

**Aktualna decyzja wlasciciela z 2026-07-29:** wspolny budzet render-model
wynosi 300 000 trojkatow, a wartosc 300 000 jest akceptowana.

Zaimplementowane: H1/animation route i Review dopuszczaja najwyzej 300 000.
Limit input task Meshy i limit produktu maja obecnie ta sama liczbe, ale sa
sprawdzane jako dwa niezalezne kontrakty.

### 8.3 Luki P1

| Luka | Ocena |
|---|---|
| Brak riggingu lokalnego GLB przez `model_url` Data URI | Przydatne dla istniejacych modeli, ale wymaga limitu rozmiaru, +Z/preflight i osobnego paid review. |
| Brak `post_process` | Dla obecnego GLB intake poprawne jest pominiecie; nie dodawac FBX/USDZ bez konsumenta. |
| Brak SSE | Polling jest poprawnym MVP, lecz safe GET powinien miec backoff/jitter i obsluge `429`. |
| Brak `x-api-version` | Changelog dokumentuje ten response header; nalezy zapisac go w provenance. |
| Brak katalogu w API | Snapshot dokumentacji musi byc odswiezany jawnie, nigdy podczas platnego create. |
| Brak udokumentowanego idempotency key | Nie wykonywac slepego retry POST po niejednoznacznym timeoutie; najpierw list/recovery i reconciliation taskow. |

## 9. Zgodnosc z profilem animacji Aurory

Meshy `action_id` jest identyfikatorem presetu, a nie nazwa stanu Aurory.
Mapowanie musi byc caller-owned i wersjonowane.

Wstepna lista kandydatow ponizej jest **hipoteza do inspekcji GLB**, nie
zatwierdzonym mapowaniem:

| Stan Aurora | Kandydat Meshy | Ocena |
|---|---|---|
| `cpause1` | `0 Idle` | najsilniejszy kandydat, nadal wymaga duration/loop/root-motion readback |
| `cwalk` | `613 Casual_Walk_inplace` | prawdopodobnie lepszy od root-motion walk, ale trzeba zmierzyc translacje |
| `crun` | wariant `run_fast_*_inplace` | brak semantycznie nazwanego, oczywistego wyboru |
| `ca1slashl` | `4 Attack` lub `92 Double_Combo_Attack` | brak danych o stronie, broni, hit timing i terminalnej pozie |
| `cdamagel` | `178 Hit_Reaction` | brak danych o kierunku i stanie koncowym |
| `ckdbckdie` | `187 Knock_Down`, `189 dying_backwards` albo `190 Knock_Down_1` | wymaga one-shot i terminal-pose gate |
| `cdead` | `8 Dead` albo lokalnie utrzymana finalna poza death transition | `cdead` nie jest uniwersalnym one-shotem; decyzja po readbacku |

Meshy nie dostarcza eventow `hit`, `cast`, `snd_footstep` ani
`snd_hitground` w kontrakcie API. Te eventy pozostaja lokalnym,
caller-owned authoringiem zgodnym z
`animacje-kontrakt-profil-a-codex.md`.

Kazdy klip-kandydat musi przejsc:

1. identyczna sygnature joint names/parents wzgledem rigged base;
2. zgodne inverse bind matrices i skin domain;
3. skonczone, rosnace czasy keyframes;
4. jawny duration, interpolation i channel inventory;
5. pomiar root translation oraz klasyfikacje `in_place | root_motion`;
6. widoczna nierigid deformacje wazonej siatki;
7. jawne mapowanie do jednej nazwy Aurora;
8. lokalne eventy i terminal-pose/loop policy;
9. own binary readback po emisji MDL;
10. owner-run proof runtime na zamrozonym kandydacie, gdy dojdziemy do etapu
    wizualnego.

## 10. Rekomendowana architektura wieloklipowa

```text
exact source model/task
        |
        v
one Rigging task (5 credits)
        |
        +--> rigged base GLB ---------> hash + skeleton signature
        |
        +--> optional walk/run GLB ---> inspect as independent candidates
        |
        v
curated action manifest (N actions, catalog hash)
        |
        v
N Animation tasks on the same rig_task_id (3 credits each)
        |
        v
download each GLB immediately
        |
        v
GLB readback + skeleton equality + motion inventory
        |
        v
explicit Meshy action -> Aurora state mapping
        |
        v
deterministic local multi-clip merge / M2A animation-set input
        |
        v
Aurora binary MDL writer + own readback
```

Zasady:

- jeden rig jest wspolna tozsamoscia lineage;
- kazdy klip ma osobny task ID i hash;
- signed URL nie jest provenance i nie opuszcza Bridge;
- partial success jest zachowywany; jeden nieudany klip nie kasuje poprawnych;
- nowy klip nie wymaga ponownego generowania ani ponownego rigu;
- merge odrzuca rozne szkielety zamiast retargetowac je po cichu;
- nazwy Aurora sa nadawane dopiero po walidowanym, jawnym mapowaniu.

## 11. Plan wdrozenia

### Faza A - kontrakt i testy bez kredytow

1. Dodac tracked, kuratorowany `MeshyAnimationCatalogSnapshotV1` z data, URL,
   normalizowanym SHA-256 i wybranymi akcjami.
2. Zastapic pole numeryczne selektorem akcji.
3. Dodac prawdziwy H1 preflight i aktualny limit 300 000.
4. Wprowadzic `AnimationSelectionV1[]` i dynamiczny kalkulator
   `base + rig + 3*N`.
5. Rozszerzyc fake transport o rigged base, optional basic clips, wiele
   Animation taskow, `CANCELED`, `429` GET i partial failure.
6. Dodac `MeshyAnimationArtifactManifestV1`.
7. Dodac recovery przez list/get dla Rigging i Animation.
8. Uporzadkowac semantyke lokalnego stop/cancel.

### Faza B - bezplatny integration contract

Oficjalny changelog dokumentuje dummy test key:

`msy_dummy_api_key_for_test_mode_12345678`

Osobny, jawny run moze sprawdzic ksztalt request/response bez kredytow. Nie
nalezy zakladac, ze sample output dowodzi jakosci konkretnego modelu, zgodnosci
szkieletu albo mappingu Aurory.

### Faza C - jeden owner-approved real lineage

1. Dokladnie jeden teksturowany humanoid w A/T pose, <= 300 000 trojkatow.
2. Jeden Rigging task; natychmiastowy download base i basic clips.
3. Jeden `Idle` (`action_id=0`).
4. Offline GLB identity/skeleton/motion report.
5. Dopiero po PASS: gameplay-floor kandydaci walk, run, attack, damage,
   death transition i dead.

Nie tworzyc od razu 42 platnych taskow.

### Faza D - Aurora admission

Po zebraniu floor 7:

- jawny mapping do siedmiu stanow;
- caller-owned eventy;
- multi-clip readback;
- Profile A writer/readback;
- owner-run runtime proof zgodnie z aktywna polityka human-owned proof.

Pelne 42 stanow jest oddzielna decyzja kosztowa i semantyczna.

## 12. Definition of Done dla integracji Meshy Animation

- [x] surowy `action_id` nie jest wpisywany recznie;
- [x] snapshot katalogu ma date, URL i hash;
- [x] H1 preflight jest rzeczywistym potwierdzeniem;
- [x] H1 nie przekracza limitu input task Meshy ani produktu 300 000 trojkatow;
- [x] jeden rig moze miec wiele action taskow;
- [x] rigged base i optional basic clips sa zachowane;
- [x] koszt jest liczony dla dokladnej listy klipow;
- [x] kazdy create zapisuje task ID natychmiast;
- [x] restart Bridge pozwala odzyskac Rig/Animation taski;
- [x] `CANCELED` jest terminalne;
- [x] lokalny stop nie udaje zdalnego anulowania;
- [x] provenance zawiera action/catalog/API/skeleton/clip identity;
- [x] wszystkie pobrane GLB maja own readback, a H1 wymaga zgodnego szkieletu;
- [x] mapping Meshy -> Aurora jest jawny i wersjonowany;
- [x] partial failure nie usuwa udanych artefaktow;
- [x] fake transport jest zielony;
- [-] zewnetrzny dummy-key contract pozostaje opt-in i nie byl uruchamiany
  podczas implementacji offline;
- [x] platny E2E wymaga osobnej, jawnej zgody wlasciciela;
- [x] finalny wizualny proof wykonuje wlasciciel.

## 13. Odrzucone wnioski

- "Webapp wspiera quadruped, wiec API tez" - odrzucone; kontrakt API nadal
  ogranicza rigging do humanoidow.
- "678 rekordow oznacza ID 0..677" - odrzucone; ID sa nieciagle i dochodza do
  696.
- "Jedno Animation API zwraca caly pakiet ruchow" - odrzucone; task stosuje
  jeden `action_id`.
- "Idle GLB wystarcza jako pelna obsluga animacji" - odrzucone przez aktywny
  kontrakt profilu A.
- "Cancel w Studio anuluje koszt Meshy" - odrzucone; obecny kod tylko zatrzymuje
  lokalne sledzenie.
- "Ta sama wartosc 300 000 oznacza jeden kontrakt" - odrzucone; provider input
  task i wspolny budzet produktu maja odrebne znaczenie oraz bramki.
- "Nazwa presetu wystarcza do mapowania stanu NWN" - odrzucone; wymagany jest
  readback ruchu, szkieletu, root motion i terminalnej pozy.

## 14. Wynik weryfikacji audytu

- oficjalne strony Rigging, Animation, Library, Pricing, Retention, Errors,
  Rate Limits, Changelog i webapp Animate odczytane 2026-07-28;
- katalog HTML sparsowany read-only: `678` unikalnych rekordow, brak duplikatow
  ID, zakres `0..696`, `19` luk;
- kod Bridge/UI/Core przejrzany na branchu `animation`;
- nie wykonano requestu do `api.meshy.ai`;
- nie pobrano ani nie zapisano zewnetrznego modelu;
- nie uruchamiano Aurora Toolset ani NWN;
- nastepny krok to Faza A, nie platny E2E.
