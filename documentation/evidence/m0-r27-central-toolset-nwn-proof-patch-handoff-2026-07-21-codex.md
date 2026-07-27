# M0 r27: centralny handoff patcha Toolset i NWN proof

Data: 2026-07-21  
Status: `OFFLINE HANDOFF CORRECTED / SHARED CENTRAL EXECUTION LAYER AVAILABLE / LIVE PROOF PAUSED`  
Watek zrodlowy: `019f850c-3cf3-7cb1-8853-cbdb4cfa8b05`

## Cel i granice tej notatki

Dokument utrwala zaakceptowany, implementowalny handoff dla dwoch opcjonalnych
centralnych zmian automation/validation hardening. Nie sa one warunkiem
uruchomienia exact kandydata M0 r27 ani uzyskania akceptacji wizualnej
wlasciciela opartej na swiezym obrazie Toolsetu i swiezym obrazie NWN:

- patch A: publiczna kontynuacja `no-Save` dla caller-owned binary profile v1,
  od dokladnego otwarcia modulu przez wybor/readback fixture do swiezego
  capture `TScrollBox` i profile-bound validatora;
- patch B: utwardzenie AUR-S07 tak, aby weryfikowal zainstalowany MOD i ordered
  HAK-i, semantyke geometry gate, rzeczywisty i swiezy PNG zwiazany z procesem,
  swieza linie `Loading Module`, a werdykt wizualny przyjmowal dopiero jako
  osobna inspekcje zwiazana z niezmiennym capture.

## Korekta zakresu: legalna warstwa wykonawcza juz istnieje

Wczesniejszego blanketowego wniosku `no legal route` nie nalezy uzywac ani
interpretowac jako blokady wykonania r27. Byl bledem syntezy: brak jednego
all-in-one publicznego skryptu nie jest brakiem legalnej sciezki operacyjnej.

Legalna, centralna execution layer dla exact r27 to aktualne wspolne skille i
ich canonical runner / verified native atoms:

- `aurora-toolset-operate` prowadzi wlasnosc, inspekcje, wznowienie i otwarcie
  jednej sesji Aurora Toolset;
- `aurora-toolset-author` jest sciezka dla exact module/Area/HAK/fixture
  contextu, gdy taki native etap jest potrzebny;
- `aurora-toolset-prove` prowadzi candidate-bound Toolset oraz NWN proof,
  capture i walidacje dowodow.

Zgodnie z project rules sa to mandatory shared operator tooling. Wyjatek
reference-only pozwala ich uzywac bezposrednio z canonical runnerem i
zweryfikowanymi atomami; zakazane pozostaja Meshy2Aurora-local runner, UI
adapter, launcher NWN i wrapper obchodzacy centralny standard. W szczegolnosci
caller-owned binary profile r27 pozostaje legalnym wejsciem, a `no-Save` oraz
niezmiennosc lineage pozostaja obowiazkowe.

Patch A i patch B ponizej pozostaja wartosciowe jako opcjonalne ulepszenia
automatyzacji, niezaleznosci validatora i hardeningu packetow. Przy aktualnym
kryterium wlasciciela (swiezy obraz z widocznym modelem w Toolsecie oraz NWN)
nie sa bramka poprzedzajaca legalne wykonanie proofu przez shared execution
layer.

Ta tura byla wylacznie offline/read-only wzgledem `C:\Projects\aurora-web`,
Aurora Toolset, NWN, r27 oraz `proof-output\...\live`. Nie uruchamiano Toolsetu
ani NWN, nie wykonywano `Save`, nie zamykano procesow, nie utworzono r28 i nie
zmieniono ani nie kopiowano MOD-u, HAK-a, MDL-a, 2DA, tekstury lub fixture.

Przed zapisem dokumentu wykonano obowiazkowy guard:

```text
canonical-workspace-ok: C:\Projects\meshy2aurora
```

## Dokladny kandydat r27

Caller-owned profil istnieje pod:

`C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-binary-bootstrap-v1.json`

Jego SHA-256:

`759f9a95dad7b7d8de6d2ca494800bc8d9c65c25a9dcab3c7651df07c9c4aa6c`

Profil i wykonany centralny structural preflight wiaza:

| Element | Dokladna wartosc |
| --- | --- |
| MOD | `m2a_m0r25.mod` |
| MOD SHA-256 | `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257` |
| Area | `m2a_m0a25`, `2x2`, display name `Meshy2Aurora M0 binary vertical-slice area` |
| Entry | `[10,10,0]` |
| Fixture ID | `m0_fixture` |
| Template | `nw_dwarfmerc001` |
| Toolset tree text | `Dwarf Mercenary`, occurrence `0` |
| Appearance | `848` |
| Fixture position | `[10,14.5,0]` |
| HAK 1 | `m2a_m0r27`, `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd` |
| HAK 2 | `m2a_m0r26`, `6f805873420b5279b45dadc26d913fa7692374df9e0582c23b42c8a4b42d6bb0` |
| HAK 3 | `m2a_m0r21`, `25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6` |

Aktualny project-owned geometry gate:

`C:\Projects\meshy2aurora\proof-output\m0-r27-animation-type5-20260721\native-geometry-gate.json`

ma SHA-256:

`f10e9bc7df3a8b132d37e26eb920fb2c19dd4214f431a8614b1c6975719881be`

## Niezaleznie zweryfikowany stan obecnej trasy

Z biezacych, a nie historycznych plikow w `C:\Projects\aurora-web` wykonano
read-only:

```powershell
node backend\scripts\validate-aurora-toolset-binary-module-bootstrap.mjs `
  --mode dry-run `
  --profile C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-binary-bootstrap-v1.json

node backend\scripts\validate-aurora-toolset-binary-module-bootstrap.mjs `
  --mode preflight `
  --profile C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-binary-bootstrap-v1.json

node backend\scripts\aurora-toolset-binary-module-native-geometry.mjs dry-run `
  --profile C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-binary-bootstrap-v1.json `
  --outDir C:\Projects\meshy2aurora\proof-output\m0-r27-animation-type5-20260721\live
```

Wyniki:

- `binary_bootstrap_profile_schema_valid`;
- `binary_bootstrap_structural_readback_valid`;
- `binary_native_geometry_plan_valid`.

Wniosek jest waski i potwierdzony kodem oraz preflightem:

1. `aurora-toolset-binary-module-bootstrap-profile/v1` legalnie opisuje
   istniejacy exact r27. Centralny validator nie wymaga nowego MOD-u ani
   absent-destination MOD-u. Wymaga istniejacego pliku o dokladnym basename,
   hashu i zgodnym packed readbacku.
2. Native-geometry `preflight` odrzuca istniejacy manifest w swoim `outDir` i
   pre-existing Toolset, ale nie odrzuca modulu dlatego, ze juz istnieje.
3. `geometry-observe` nie jest odpowiednim automated etapem immutable-r27
   no-Save, poniewaz
   `geometry-observe` zawsze wywoluje
   `save-aurora-toolset-module-no-global-input.mjs`, po czym oblicza post-save
   hash i tworzy `post-native-save-binary-bootstrap-profile.json`. Native Save
   moze przepakowac MOD i zmienic hash `4a4f98...`.

   Jest to ograniczenie tej automatycznej komendy, nie brak legalnej execution
   layer dla r27. Exact selection/readback i fresh `TScrollBox` realizuje shared
   `operate`/`author`/`prove` przez canonical runner i verified native atoms.
4. Obecna trasa native-geometry nie wybiera i nie odczytuje dokladnego fixture
   dla proofu oraz nie wykonuje `TScrollBox` capture. Geometry profile wskazuje
   capture tylko jako pozniejszy krok i sam nie rości visual/runtime proofu.
5. Bezposrednie wywolanie leaf
   `aurora-toolset-viewport-proof.mjs` nie jest publiczna kontynuacja binary
   profile i nie dostarcza profile-bound exact-object packetu. Nie wolno go
   samodzielnie podstawic zamiast canonical route.

Kod ponizej opisuje tylko luke opcjonalnej automatyzacji:

`CENTRAL_BINARY_R27_TSCROLLBOX_ROUTE_MISSING`

## Opcjonalny patch A: centralna automatyzacja Toolset proof bez Save

### Plan plik po pliku w `C:\Projects\aurora-web`

| Plik | Zmiana |
| --- | --- |
| `backend/scripts/aurora-toolset-binary-module-toolset-proof.mjs` | Nowy publiczny runner z komendami `dry-run`, `preflight`, `open`, `capture`, `record-visibility`, `status`. Ma konsumowac osobny proof profile, uzywac native-geometry tylko do `preflight` i `open`, nigdy do `geometry-observe`, i pozostawiac proces otwarty. |
| `backend/scripts/validate-aurora-toolset-binary-module-toolset-proof.mjs` | Nowy niezalezny validator finalnego packetu. Ma ponownie hashowac profile i artefakty oraz sprawdzac wszystkie wiazania i brak mutacji. |
| `backend/scripts/lib/aurora-proof-png.mjs` | Nowy wspolny parser dowodowych PNG: sygnatura, `IHDR`, dodatnie wymiary, liczba bajtow, SHA-256 i `mtime`. Ma byc wspolny dla patcha A i B. |
| `backend/docs/aurora-reverse/proof-profiles/binary-module-toolset-proof-profile.example.json` | Nowy przyklad `aurora-toolset-binary-module-toolset-proof-profile/v1`. |
| `backend/docs/aurora-reverse/aurora-toolset-binary-module-bootstrap-standard.md` | Dodac nazwana publiczna kontynuacje no-Save, pre/postconditions, zakaz `geometry-observe` dla immutable candidate i brak automatycznego close. |
| `backend/scripts/test-aurora-toolset-binary-module-toolset-proof-contract.mjs` | Test publicznego runnera i braku sciezki Save. |
| `backend/scripts/test-validate-aurora-toolset-binary-module-toolset-proof-contract.mjs` | Test validatora, w tym wszystkie negatywne przypadki identity/capture/freshness. |
| `backend/scripts/test-aurora-toolset-binary-module-toolset-proof-consumer-cwd.mjs` | Test uruchomienia centralnego entrypointu z consumer CWD bez lokalnego runnera lub adaptera. |

### Minimalny proof profile v1

```json
{
  "version": "aurora-toolset-binary-module-toolset-proof-profile/v1",
  "id": "m2a-m0-r27-on-r25-toolset-proof-v1",
  "binaryProfile": {
    "path": "C:\\Projects\\meshy2aurora\\proof-profiles\\m0-r27-on-r25-binary-bootstrap-v1.json",
    "sha256": "759f9a95dad7b7d8de6d2ca494800bc8d9c65c25a9dcab3c7651df07c9c4aa6c"
  },
  "fixtureSelection": {
    "fixtureId": "m0_fixture",
    "treeObjectText": "Dwarf Mercenary",
    "occurrence": 0
  },
  "capture": {
    "targetClass": "TScrollBox",
    "allowUniformScene": false
  },
  "noSave": true,
  "noLocalToolsetAdapter": true
}
```

### Aktualny deklaratywny profil r27

Deklaratywna warstwa Gate B zostala juz legalnie dodana w Meshy2Aurora jako:

`C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-toolset-proof-v1.json`

Jej SHA-256 to:

`731db6efbd8004060ea130a760b5755d0518adda0938bde6f21091fc054c213e`

Niezaleznie uruchomiony test:

```powershell
node C:\Projects\meshy2aurora\tools\m0-r27-toolset-proof-profile.contract.test.mjs
```

zwraca `m0_r27_toolset_proof_profile_contract_valid` i potwierdza source
binary profile, `m0_fixture` / `Dwarf Mercenary` / occurrence `0`, capture
`TScrollBox`, `allowUniformScene=false`, `noSave=true` oraz
`noLocalToolsetAdapter=true` bez uruchamiania Toolsetu lub NWN.

Ten profil nie jest nowa iteracja modelu, poniewaz nie zmienia payloadu ani
lineage r27. Sam profil deklaratywny nie jest proofem, lecz moze byc legalnie
wykonany przez shared execution layer. Opisany tu centralny runner i validator
sa opcjonalnym hardeningiem automatyzacji; runtime profile v2 pozostaje
artefaktem tylko wariantu z patchem B, a nie warunkiem owner image acceptance.

### Preconditions patcha A

- exact binary profile istnieje, ma deklarowany hash i przechodzi centralny
  structural `preflight`;
- MOD i wszystkie HAK-i istnieja pod zadeklarowanymi sciezkami i maja dokladne
  hashe oraz ordered HAK readback;
- nowy proof `outDir` nie istnieje lub jest jawnie wymaganym pustym celem;
- przed swiezym `open` nie istnieje `nwtoolset` ani `nwmain`;
- otwarcie jest wlasnoscia centralnego explicit-open admission v3;
- nie istnieje nieobsluzony modal, drugi module-load owner lub inny proces;
- fixture selection jest zadeklarowane przed capture;
- `Focus on Object` i kadrowanie sa opcjonalne dla Meshy2Aurora i nie moga byc
  bramka akceptacji.

### Dokladna kompozycja istniejacych atomow/precedensow

Runner powinien komponowac:

1. `validate-aurora-toolset-binary-module-bootstrap.mjs --mode preflight`;
2. `aurora-toolset-binary-module-native-geometry.mjs preflight` oraz `open`
   w podporzadkowanym katalogu sesji, ale nigdy `geometry-observe`;
3. `run-toolset-area-oracle-queue.mjs --capture false` dla exact Area;
4. `set-aurora-toolset-selection-mode-no-global-input.mjs --mode objects`;
5. `inspect-aurora-toolset-area-tree-object-context-no-global-input.mjs`
   z `--selectOnly true`, exact `objectText` i `occurrence`;
6. `probe-aurora-toolset-area-tree-selection-no-global-input.mjs` jako
   niezalezny caret/tree readback;
7. `aurora-toolset-viewport-proof.mjs capture` dla fresh before i
   after-selection `TScrollBox`;
8. eksport `analyzeAuroraToolsetSelectionDelta` z
   `analyze-aurora-toolset-selection-anchor.mjs` dla niezaleznego outline
   delta;
9. `inspect-aurora-toolset-visible-windows-no-global-input.mjs` przed i po
   krytycznych fazach;
10. wzorzec pristine `outDir`, `captured_pending_visual_inspection` i osobnego
    `record-visibility` z `aurora-toolset-vertical-slice.mjs`, linie logiki
    `bootstrap-focus-capture` / `bootstrap-focus-record-visibility`.

Nie wolno tworzyc Meshy2Aurora-local wrappera tych atomow.

### Wymagany packet i postconditions

Packet `aurora-toolset-binary-module-toolset-proof-packet/v1` ma zawierac co
najmniej:

- path i SHA-256 proof profile oraz source binary profile;
- exact module resref/path/hash przed i po capture;
- exact ordered HAK list z path/hash przed i po capture;
- Area resref, display name i queue readback;
- source fixture `id/templateResRef/appearanceType/position`;
- deklarowany tree selector i jego exact selection result;
- `Objects` toolbar checked-state readback;
- niezalezny caret/tree readback tego samego node;
- before i after-selection `TScrollBox` result, PNG path/hash/bytes/dimensions,
  target class, module/Area, PID i capture time;
- selection-outline delta;
- safety: `noSave=true`, brak HAK/module edit, brak global inputu, brak close;
- po capture domyslnie
  `modelVisibility=not_tested`, `proofCompleteness=missing`;
- dopiero po osobnej inspekcji `visualInspection` zwiazana z hashem capture i
  wynik `visible|not_visible`, `proofCompleteness=verified`.

Jezeli capture nie pozwala na wizualny werdykt, packet pozostaje
`not_tested/missing`. Nie jest to model failure i nie dopuszcza r28.

### Negatywne testy patcha A

Testy musza odrzucic:

- inny hash/path source proof profile;
- inny MOD hash, basename lub packed readback;
- zmieniona kolejnosc, resref, path lub hash ktoregokolwiek HAK-a;
- inna Area, display name, entry lub niepelna Area;
- brak fixture ID, inny template, Appearance lub pozycje;
- pusty tree text, ujemne occurrence, niejednoznaczny node bez occurrence;
- stale lub istniejace artifact destination;
- pre-existing Toolset/NWN przed fresh `open`;
- native manifest bez statusu `module_ready`, inny profile/hash/PID;
- niepotwierdzony tryb `Objects`;
- wrong/ambiguous tree target lub caret mismatch;
- brak niezaleznego selection outline delta;
- capture result innego MOD-u, Area, PID-u lub targetu niz `TScrollBox`;
- plik tekstowy z rozszerzeniem `.png`, zla sygnature, uszkodzony/truncated PNG,
  brak/zerowe wymiary lub stale `mtime`;
- capture starszy niz run/session/selection;
- zmiane MOD-u lub HAK-a po `open` albo capture;
- `usesGlobalInput=true`, Save, Build, HAK edit lub automatyczny close;
- supplied `visible` bez osobnego `record-visibility`;
- visual inspection zwiazana z innym capture hashem;
- `modelVisibility=not_visible` wyprowadzone z nieczytelnego lub brakujacego
  capture.

Test source contract ma dodatkowo wymagac, aby runner nie wywolywal atomu Save
ani komendy `geometry-observe`.

## Opcjonalny patch B: utwardzenie AUR-S07

### Obecna luka

Aktualny `aur-s07-runtime-execution.mjs` sprawdza ksztalt profilu i
`geometryGate.status=verified`, ale:

- profil nie musi wskazywac zainstalowanych sciezek MOD/HAK, a runner ich nie
  hashuje;
- geometry gate nie jest odczytywany, hashowany ani sprawdzany semantycznie;
- request nie jest jednorazowo zwiazany z handoffem i swiezym procesem;
- packet moze sam zadeklarowac `modelVisibility.status=visible`;
- runtime capture i log sa sprawdzane tylko przez existence/hash;
- biezacy contract test zapisuje zwykly tekst `visible model pixels` jako
  `runtime.png` oraz `module loaded` jako log i otrzymuje `verified`.

To jest validator/proof-lane failure, nie visual model failure r27.

### Plan plik po pliku w `C:\Projects\aurora-web`

| Plik | Zmiana |
| --- | --- |
| `backend/scripts/aur-s07-runtime-execution.mjs` | Wprowadzic `aur-s07-runtime-profile/v2`; v1 odrzucac jako unhardened dla `arm/observe/capture/final acceptance`. Dodac fazy `capture`, `record-visibility` i finalny `validate` bez caller-supplied visual verdict. |
| `backend/scripts/lib/aur-s07-runtime-proof-contract.mjs` | Nowe czyste funkcje walidujace profile, binary binding, geometry gate, Toolset proof, request/handoff/process, capture/log freshness i finalny proof. |
| `backend/scripts/capture-nwn-runtime-window-no-global-input.mjs` | Nowy centralny read-only atom NWN. Ma wymagac exact obserwowanego PID/StartTime, jednego widocznego owned window, zwalidowanego monitora i fizycznych pikseli; ma zwracac metadata i prawdziwy PNG bez inputu/launch/close. |
| `backend/scripts/lib/aurora-proof-png.mjs` | Wspolny z patchem A parser sygnatury/IHDR/hash/freshness. |
| `backend/docs/aurora-reverse/aur-s07-runtime-execution-standard.md` | Zmienic lifecycle i wyraznie opisac osobna inspekcje wizualna oraz fail-closed v1. |
| `backend/docs/aurora-reverse/proof-profiles/aur-s07-runtime-profile-v2.example.json` | Nowy przyklad v2. |
| `backend/scripts/test-aur-s07-runtime-execution-contract.mjs` | Zastapic tekstowe fixture prawdziwym PNG i swiezym dopisanym logiem; dodac pelna macierz negatywna. |
| `backend/scripts/test-capture-nwn-runtime-window-contract.mjs` | Test metadata/PID/window/PNG/no-global-input nowego capture atomu. |
| `backend/scripts/test-aur-s07-runtime-execution-consumer-cwd.mjs` | Test publicznego centralnego entrypointu z consumer CWD. |

### Minimalny profil AUR-S07 v2

Profil v2 powinien dziedziczyc exact installed module/HAK/Area/entry/fixture z
source binary profile zamiast duplikowac je w slabszej formie:

```json
{
  "version": "aur-s07-runtime-profile/v2",
  "id": "m2a-m0-r27-on-r25-runtime-v2",
  "sourceBinaryProfile": {
    "path": "C:\\Projects\\meshy2aurora\\proof-profiles\\m0-r27-on-r25-binary-bootstrap-v1.json",
    "sha256": "759f9a95dad7b7d8de6d2ca494800bc8d9c65c25a9dcab3c7651df07c9c4aa6c"
  },
  "geometryGate": {
    "evidence": "C:\\Projects\\meshy2aurora\\proof-output\\m0-r27-animation-type5-20260721\\native-geometry-gate.json",
    "sha256": "f10e9bc7df3a8b132d37e26eb920fb2c19dd4214f431a8614b1c6975719881be",
    "version": "m2a-m0-r27-native-geometry-gate/v1",
    "status": "verified"
  },
  "toolsetProof": {
    "path": "<toolset-proof.json from patch A>",
    "sha256": "<exact hash>"
  },
  "fixtureTargets": [
    { "fixtureId": "m0_fixture", "modelResRef": "m2a_m0p01" }
  ],
  "engineLog": {
    "path": "C:\\Users\\enonw\\Documents\\Neverwinter Nights\\logs\\nwclientLog1.txt",
    "loadingModuleResref": "m2a_m0r25"
  },
  "launch": {
    "mode": "user_owned_test_module_action",
    "processName": "nwmain"
  },
  "noLocalSubstitute": true
}
```

### Geometry gate hash i semantyka

Nie wolno akceptowac samego `geometryGate.status=verified`. Runner ma:

1. odczytac evidence z dokladnie przypietej sciezki;
2. sprawdzic SHA-256, version i status;
3. porownac module resref/path/hash z source binary profile;
4. porownac Area, entryPoint i ordered HAK resref/hash;
5. porownac fixture ID, template, Appearance i pozycje, normalizujac jedynie
   nazwe `appearanceType` kontra `appearanceRow`;
6. wymagac prawdziwych readback fields deklarowanych przez dany schema, np.
   `orderedHakListVerified=true` i `fixtureBindingVerified=true`;
7. osobno zwalidowac fresh Toolset proof patcha A, exact ten sam binary profile,
   `modelVisibility=visible`, `proofCompleteness=verified`, zanim `arm` zostanie
   dopuszczony do NWN.

### Utwardzony lifecycle AUR-S07

1. `dry-run` ponownie hashuje source binary profile, uruchamia jego centralny
   structural `preflight`, sprawdza installed MOD/ordered HAK, geometry gate i
   Toolset proof. Nie startuje Toolsetu ani NWN.
2. `arm` odrzuca kazdy pre-existing `nwmain`, wymaga exact responsywnego
   Toolset/module owner, zapisuje jednorazowy `requestId` oraz baseline
   przypietego engine logu: bytes, hash i mtime.
3. User wykonuje jedno `Test Module`. To pozostaje jawna user-owned granica.
4. `observe --handoff` wymaga tego samego `requestId`, profile/module/Area/entry
   i dokladnie jednego nowego responsywnego `nwmain`, ktorego StartTime jest po
   `request.createdAt` i nie nalezy do baseline.
5. `capture` nie przyjmuje dowolnego caller packetu. Ponownie weryfikuje
   obserwowany PID/StartTime, wywoluje centralny read-only NWN capture atom,
   waliduje prawdziwy PNG i analizuje tylko bajty logu dopisane po `arm`.
6. Swiezy segment logu musi zawierac dokladna linie
   `Loading Module: m2a_m0r25`. Linia obecna tylko przed `arm` nie jest
   dowodem. Poczatkowa wersja powinna fail-closed odrzucac nieudokumentowana
   rotacje lub truncation zamiast zgadywac.
7. Centralne evidence po capture ma status
   `captured_pending_visual_inspection`, `modelVisibility=not_tested`,
   `proofCompleteness=missing`.
8. `record-visibility` jest osobna faza po faktycznej inspekcji niezmiennego
   PNG. Przyjmuje tylko `visible|not_visible`, wiaze opis i czas z capture hash
   i ustawia `proofCompleteness=verified`.
9. `validate` ponownie sprawdza caly lancuch. Pole `modelVisibility` z dowolnego
   caller-supplied JSON nie moze samo utworzyc werdyktu.

### Negatywne testy patcha B

Testy musza odrzucic:

- profil AUR-S07 v1 dla live/final acceptance;
- source binary profile hash/path drift;
- zainstalowany MOD o innym hashu/basename;
- brak, zmiane lub inna kolejnosc zainstalowanych HAK-ow;
- geometry gate missing, hash/version/status mismatch;
- gate z innym module, Area, entry, fixture lub ordered HAK list;
- Toolset proof missing, innego source profile, `not_visible`, `not_tested` lub
  `proofCompleteness=missing`;
- `arm` przy istniejacym `nwmain` albo drugi `arm` w tym samym runie;
- handoff bez requestId lub z innym request/profile/module/Area/entry;
- zero, dwa lub wiecej `nwmain`, proces stary, niereponsywny albo PID reuse;
- replay juz skonsumowanego requestu;
- caller-supplied runtime packet jako zrodlo visual verdict;
- zwykly tekst nazwany `.png`, zla sygnature, uszkodzony/truncated PNG,
  niedodatnie wymiary, pusty lub stale capture;
- capture metadata z innym PID/StartTime/HWND, nieowned/zasloniete okno lub
  `usesGlobalInput=true`;
- log pod inna sciezka niz profil;
- prawidlowa linie `Loading Module` wystepujaca tylko w baseline sprzed `arm`;
- truncation/rotation bez jawnego kontraktu;
- swiezo dopisana linie dla innego modulu lub stale mtime logu;
- `record-visibility` przed centralnym capture;
- visual verdict zwiazany z innym capture hashem;
- `not_tested` polaczone z `proofCompleteness=verified`;
- `not_visible` wyprowadzone z brakujacego lub nieczytelnego obrazu.

## Command plan opcjonalnego hardeningu po osobnej autoryzacji centralnego patcha

Ponizsze komendy sa planem tylko po wdrozeniu opcjonalnych patchy. Nowe
entrypointy jeszcze nie istnieja i nie wolno ich zastepowac lokalnymi
skryptami. Nie sa one warunkiem legalnego wykonania r27 przez shared execution
layer ani akceptacji obrazowej wlasciciela.

### 1. Centralne testy regresyjne

```powershell
Set-Location C:\Projects\aurora-web

node backend\scripts\test-validate-aurora-toolset-binary-module-bootstrap-contract.mjs
node backend\scripts\test-validate-aurora-toolset-binary-module-bootstrap-consumer-cwd.mjs
node backend\scripts\test-aurora-toolset-binary-module-native-geometry-contract.mjs

node backend\scripts\test-aurora-toolset-binary-module-toolset-proof-contract.mjs
node backend\scripts\test-validate-aurora-toolset-binary-module-toolset-proof-contract.mjs
node backend\scripts\test-aurora-toolset-binary-module-toolset-proof-consumer-cwd.mjs

node backend\scripts\test-capture-nwn-runtime-window-contract.mjs
node backend\scripts\test-aur-s07-runtime-execution-contract.mjs
node backend\scripts\test-aur-s07-runtime-execution-consumer-cwd.mjs
```

### 2. Exact r27 Toolset proof

```powershell
Set-Location C:\Projects\aurora-web

$toolsetProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-toolset-proof-v1.json'
$toolsetOut = 'C:\Projects\meshy2aurora\proof-output\m0-r27-animation-type5-20260721\live\toolset'

node backend\scripts\aurora-toolset-binary-module-toolset-proof.mjs dry-run --profile $toolsetProfile --outDir $toolsetOut
node backend\scripts\aurora-toolset-binary-module-toolset-proof.mjs preflight --profile $toolsetProfile --outDir $toolsetOut
node backend\scripts\aurora-toolset-binary-module-toolset-proof.mjs open --profile $toolsetProfile --outDir $toolsetOut
node backend\scripts\aurora-toolset-binary-module-toolset-proof.mjs capture --profile $toolsetProfile --outDir $toolsetOut
```

Nastepnie nalezy obejrzec exact after-selection `TScrollBox`. Tylko gdy kadr
pozwala na werdykt:

```powershell
node backend\scripts\aurora-toolset-binary-module-toolset-proof.mjs record-visibility `
  --profile $toolsetProfile `
  --outDir $toolsetOut `
  --modelVisibility visible `
  --inspection '<dokladny opis obserwacji>'

node backend\scripts\validate-aurora-toolset-binary-module-toolset-proof.mjs `
  --profile $toolsetProfile `
  --packet "$toolsetOut\toolset-proof.json"
```

Jezeli obraz jest nieczytelny, nie uruchamiac `record-visibility not_visible`.
Pozostawic `not_tested/missing` i dokonczyc ten sam kandydat.

### 3. Exact r27 NWN proof po `Toolset visible/verified`

```powershell
Set-Location C:\Projects\aurora-web

$runtimeProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-runtime-v2.json'
$runtimeOut = 'C:\Projects\meshy2aurora\proof-output\m0-r27-animation-type5-20260721\live\nwn'

node backend\scripts\aur-s07-runtime-execution.mjs dry-run --profile $runtimeProfile --outDir $runtimeOut
node backend\scripts\aur-s07-runtime-execution.mjs arm --profile $runtimeProfile --outDir $runtimeOut
```

Po jednym user-owned `Test Module` i przygotowaniu handoffu z exact requestId:

```powershell
node backend\scripts\aur-s07-runtime-execution.mjs observe `
  --profile $runtimeProfile `
  --outDir $runtimeOut `
  --handoff '<runtime-handoff.json>'

node backend\scripts\aur-s07-runtime-execution.mjs capture `
  --profile $runtimeProfile `
  --outDir $runtimeOut
```

Po inspekcji centralnego runtime PNG:

```powershell
node backend\scripts\aur-s07-runtime-execution.mjs record-visibility `
  --profile $runtimeProfile `
  --outDir $runtimeOut `
  --modelVisibility visible `
  --inspection '<dokladny opis obserwacji>'

node backend\scripts\aur-s07-runtime-execution.mjs validate `
  --profile $runtimeProfile `
  --outDir $runtimeOut
```

Nie wolno automatycznie zamykac Toolsetu ani NWN. Close wymaga osobnej,
jawnej autoryzacji wlasciciela.

## Granica prawna i projektowa

### Legalne w `C:\Projects\meshy2aurora`

- caller-owned deklaratywny binary profile;
- po zatwierdzeniu centralnych schematow: Toolset proof profile v1 i AUR-S07
  runtime profile v2;
- handoff user-owned Test Module;
- profile-owned evidence i immutable proof packet w `proof-output`;
- dokumentacja, inspekcyjny opis werdyktu oraz test wlasnego profilu;
- odczyt centralnego operatora jako mandatory shared operator tooling.

### Bezwzglednie central-only w `C:\Projects\aurora-web`

- publiczny Toolset/NWN runner i orchestration;
- native UI atomy;
- selection/capture/process/log adaptery;
- parser/validator PNG i proof packetow;
- centralne schematy, standardy, przyklady i contract tests;
- zmiany do `aurora-toolset-operate`, `aurora-toolset-prove` lub canonical route.

Meshy2Aurora nie moze dodac lokalnego Toolset runnera, proof runnera, UI
adaptera, launchera NWN, kopii centralnego skryptu ani wrappera obchodzacego
brakujaca publiczna continuation. Wyjatek shared operator tooling pozwala
uzywac centralnych runnerow bez robienia z `aurora-web` zaleznosci produktu.

## Change-control i bardzo brudny worktree `aurora-web`

W chwili audytu `C:\Projects\aurora-web` mial bardzo brudny working tree.
Co szczegolnie istotne, nastepujace aktualne pliki operatora byly oznaczone
przez `git status --short` jako untracked (`??`):

- `backend/docs/aurora-reverse/aur-s07-runtime-execution-standard.md`;
- `backend/scripts/aur-s07-runtime-execution.mjs`;
- `backend/scripts/aurora-toolset-binary-module-native-geometry.mjs`;
- `backend/scripts/test-aur-s07-runtime-execution-contract.mjs`;
- `backend/scripts/validate-aurora-toolset-binary-module-bootstrap.mjs`.

Dlatego implementer centralnego patcha musi:

1. potraktowac aktualne bajty w tym worktree jako source of truth;
2. przed zmiana zapisac waski `git status` i przejrzec wszystkie nakladajace sie
   zmiany;
3. patchowac in-place bez `reset`, `checkout`, odtwarzania plikow z historii
   lub nadpisywania cudzej pracy;
4. dodac test negatywny przed minimalna implementacja;
5. nie patchowac shared route podczas aktywnego Toolset Build/Save;
6. uzyskac osobna autoryzacje wlasciciela na centralna zmiane i osobna na live
   Toolset/NWN execution.

Ten dokument jest handoffem wiedzy i planem change-control. Nie jest zgoda na
modyfikacje `aurora-web`, nie jest dowodem wizualnym i nie zmienia statusu
kandydata.

## Aktualny stan proofu i warunek wznowienia

- Toolset r27: `modelVisibility=not_tested`, `proofCompleteness=missing`;
- NWN r27: `modelVisibility=not_tested`, `proofCompleteness=missing`.

### Zachowany aktualny live fingerprint

- Publiczny etap `open` zatrzymal sie fail-closed.
- Zachowany stan sesji to responsywny PID `51964` z zaladowanym, dla tej
  krytycznej sciezki niewlasciwym `m2a_now01.mod`.
- W tej odnodze nie bylo `Save`, otwarcia/weryfikacji Area, capture
  `TScrollBox` ani uruchomienia NWN.
- Nie wolno akceptowac tego modulu jako r27 ani powtarzac slepo publicznego
  `open`. Wymagane jest reczne wznowienie exact-session przez shared canonical
  operator (nie ad-hoc local wrapper), po osobnej autoryzacji live; pozostawic
  PID i jego stan zachowane do czasu takiej inspekcji/wznowienia.

Warunek wznowienia krytycznej sciezki:

1. osobno autoryzowane live manual exact-session resume z aktualnego
   fingerprintu, prowadzone przez `aurora-toolset-operate`, a gdy potrzebne
   przez `aurora-toolset-author` i `aurora-toolset-prove`;
2. exact r27 no-Save: native selection/readback `m0_fixture`, swiezy
   kandydat-bound `TScrollBox` oraz inspekcja obrazu; widoczny model zamyka
   wizualna akceptacje Toolsetu wedlug kryterium wlasciciela;
3. ten sam exact MOD/ordered HAK lineage w NWN, swiezy obraz runtime i
   inspekcja obrazu; widoczny model zamyka wizualna akceptacje NWN;
4. patch A/B mozna osobno autoryzowac pozniej jako automation/validation
   hardening, lecz nie sa one precondition powyzszych dwoch akceptacji;
5. nowa iteracja dopiero po swiezym candidate-bound `not_visible/verified` w
   Toolsecie albo NWN, z diagnoza i minimalna deklarowana delta.

Nie wolno uruchamiac `geometry-observe`, wykonywac Save, tworzyc r28, kopiowac
MOD/HAK ani zmieniac payloadu r27 w celu obejscia brakujacego proofu.
