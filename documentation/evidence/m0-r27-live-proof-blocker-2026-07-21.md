# M0 r27 — live proof blocker packet

Data: 2026-07-21  
Snapshot: `2026-07-21T16:35:34.7796004+02:00`  
Status: `LIVE_GATE_BLOCKED / MODEL NOT TESTED`

## Zamrożona tożsamość kandydata

- MOD `m2a_m0r25.mod`: `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257`;
- MDL `m2a_m0p01`: `bf7864f0ed541ed93c4075d167f4a2392eeb6ef37b4cdf444cbb74efd5731578`;
- HAK `m2a_m0r27`: `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd`;
- ordered HAK: `m2a_m0r27`, `m2a_m0r26`, `m2a_m0r21`;
- Area `m2a_m0a25`;
- fixture `m2a_m0p01`, Appearance `848`, pozycja `(10,14.5,0)`.

Caller-owned profil:
`C:\Projects\meshy2aurora\proof-profiles\m0-r27-on-r25-binary-bootstrap-v1.json`

SHA-256 profilu:
`759f9a95dad7b7d8de6d2ca494800bc8d9c65c25a9dcab3c7651df07c9c4aa6c`

## Fingerprint procesu i sesji — snapshot offline przed próbą live

- `nwtoolset.exe`: `0` procesów;
- `nwmain.exe`: `0` procesów;
- publiczny `aurora-toolset-session.mjs status`: `state=null`, `processes=[]`;
- domyślny ledger
  `C:\Projects\aurora-web\.codex-tmp\area-toolset-oracle-20260710\session-state.json`
  nie istnieje;
- katalog caller-owned
  `C:\Projects\meshy2aurora\proof-output\m0-r27-animation-type5-20260721\live`
  jest pusty;
- read-only startup admission odczytał
  `MRU0=...\modules\m2a_now01.mod`; jest to `mruConflict=true`, nie aktywna
  sesja i nie zgoda na zmianę INI/MRU.

Nie zamknięto, nie zabito, nie uruchomiono ani nie przełączono żadnego procesu.

## Pierwsza próba publicznego `open` — 2026-07-21T15:19:21Z

Po decyzji właściciela, że akceptacją wizualną są świeże obrazy z widocznym
modelem w Toolsecie i NWN, wykonano wyłącznie legalne fazy:

`dry-run -> explicit-open-startup-admission-dry-run -> preflight -> open`.

Nie wykonano `geometry-observe`, Area navigation, wyboru obiektu, capture,
`Save`, Test Module ani NWN.

Wynik `open`:

- `blockerCode=unexpected_module_window_loaded`;
- Welcome zostało obsłużone przez publiczny kontrakt v3, z readbackiem
  `Open an existing Module=checked` i `Start normally=unchecked`;
- zanim pojawił się kontrolowany dialog wyboru modułu, Toolset załadował MRU
  `m2a_now01.mod`;
- oczekiwany `m2a_m0r25.mod` nie został otwarty;
- publiczna trasa przerwała fail-closed bez drugiego `Open Module`;
- pozostawiono dokładnie jeden responsywny `nwtoolset.exe`, PID `51964`,
  z ramką `... - m2a_now01.mod` na monitorze proof;
- `nwmain.exe=0`;
- pełny packet znajduje się w
  `proof-output\m0-r27-animation-type5-20260721\live\toolset\binary-native-geometry-manifest.json`.

Procesu nie zamknięto ani nie przełączono, ponieważ nie ma autoryzacji
zamknięcia. To jest awaria startup/MRU lane, nie obserwacja modelu:
`modelVisibility=not_tested`, `proofCompleteness=missing`.

Najkrótsze wznowienie zgodne z nową decyzją właściciela wymaga ręcznego,
user-owned otwarcia exact `m2a_m0r25.mod`, Area `m2a_m0a25` i wyboru
`Dwarf Mercenary`, bez Save. Następnie centralny read-only hook ma wykonać
świeży capture zwalidowanego `TScrollBox`; właściciel ocenia obraz. Po wyniku
`visible` ten sam lineage przechodzi do user-owned Test Module i świeżego
obrazu NWN.

## Zielone bramki offline

1. `m0_r27_binary_bootstrap_profile_contract_valid`;
2. `binary_bootstrap_profile_schema_valid`;
3. `binary_bootstrap_structural_readback_valid` — istniejący exact MOD,
   Area `2x2`, entry `(10,10,0)`, jeden exact fixture i ordered multi-HAK
   `[r27,r26,r21]`;
4. `binary_native_geometry_plan_valid`;
5. `binary_native_explicit_open_startup_admitted` dla kontraktu startup v3;
6. AUR-S07 `dry-run` zwrócił `ok=true` dla profilu
   `m2a-m0-r27-on-r25-runtime`, ale nie zamyka bramki akceptacji opisanej
   poniżej.

## Korekta authority shared operator tooling

Wniosek, że brak jednego all-in-one backend CLI oznacza brak legalnej trasy
proofu, został wycofany. Obowiązkową centralną warstwą wykonawczą są skille
`aurora-toolset-operate`, `aurora-toolset-author` i `aurora-toolset-prove`,
kanoniczny runner oraz zweryfikowane natywne atomy. Exact caller-owned profil
r27 może być przez tę warstwę obsłużony. Osobny kompozytor i mocniejszy packet
validator pozostają usprawnieniami automatyzacji, nie warunkiem samej próby
wizualnej.

Właściciel ustalił, że akceptacją są dwa świeże, jednoznaczne obrazy z
widocznym modelem: Toolset `TScrollBox` oraz NWN runtime, związane z exact r27.
Poniższe wcześniejsze sekcje opisują luki hardeningu i nie są już blanket
blockerem wykonania.

Aktualny live blocker to wyłącznie zachowany fingerprint:
`PID 51964 / m2a_now01.mod / moduleLoadOwner already claimed`. W tej sesji
`File > Open` jest zabronione. Potrzebna jest user-owned zmiana stanu przez
zamknięcie tej sesji i ręczne ustanowienie świeżej sesji exact
`m2a_m0r25.mod`; dopiero potem shared operator kontynuuje Area, selection
readback i capture.

## Historyczna luka hardeningu Gate B — nie jest już blanket blockerem

Kod:
`CENTRAL_EXACT_R27_OBJECT_TSCROLLBOX_ROUTE_MISSING`

Potwierdzone fakty:

- publiczny binary-bootstrap profile legalnie opisuje istniejący MOD i trzy
  uporządkowane HAK-i; nie wymaga nowego destination;
- publiczna kontynuacja `aurora-toolset-binary-module-native-geometry.mjs`
  ma fazy `preflight -> open -> geometry-observe`;
- `geometry-observe` wykonuje natywny Save i następnie wiąże nowy post-save
  hash MOD-u; uruchomienie tej fazy naruszyłoby bieżące zamrożenie exact MOD SHA
  bez osobnej, jawnej decyzji o przejściu hasha;
- ta trasa nie wykonuje exact object selection/readback i nie tworzy świeżego
  proofu `TScrollBox` ani packet validatora widoczności;
- `aurora-toolset-viewport-proof.mjs` potrafi wykonać read-only capture
  zwalidowanego `TScrollBox`, ale sam nie wiąże exact fixture przez natywny
  selection/readback;
- `validate-aurora-m0-toolset-packet.mjs` pozostaje związany z historycznym
  profilem `m0-static-rigid-v5`.

Nie ma więc zatwierdzonej publicznej kompozycji:

`exact r27 profile -> exact object selection/readback -> fresh TScrollBox -> exact packet validator`.

## Historyczna luka hardeningu Gate C — nie jest już blanket blockerem

Kod:
`CENTRAL_AUR_S07_ACCEPTANCE_HARDENING_MISSING`

Potwierdzone fakty z aktualnego centralnego kodu:

- `dry-run` sprawdza kształt profilu, ale profil nie wiąże ścieżek
  zainstalowanego MOD-u i HAK-ów ani nie hashuje ich przy wykonaniu;
- wskazany geometry gate nie jest odczytywany, hashowany i semantycznie
  weryfikowany przez `aur-s07-runtime-execution.mjs`;
- validator sprawdza hashe dostarczonych plików, lecz nie potwierdza, że runtime
  capture jest rzeczywistym świeżym PNG z właściwego procesu/okna;
- nie sprawdza treści ani freshness engine logu, w tym wymaganego
  `Loading Module: m2a_m0r25`;
- akceptuje `modelVisibility.status` z dostarczonego packetu zamiast wymagać
  werdyktu zapisanego po wizualnej inspekcji.

Dlatego zielony AUR-S07 `dry-run` nie uprawnia do claimu runtime
`proofCompleteness=verified`.

## Historyczny warunek pełnego hardeningu — nie jest warunkiem próby wizualnej

Live można wznowić dopiero, gdy centralny operator dostarczy i przetestuje:

1. publiczny exact-r27 Gate B profile/route/validator, który wiąże bieżący
   saved-verified MOD i ordered HAK bez ponownego native Save albo zawiera
   osobno autoryzowane, hash-bound przejście post-save;
2. natywny exact object selection/readback dla fixture oraz świeży capture
   zwalidowanego `TScrollBox`; `Focus on Object` i kadrowanie pozostają
   opcjonalne zgodnie z regułą projektu;
3. negatywne testy odrzucające inny MOD/hash, HAK order/hash, Area, fixture,
   Appearance, position, capture target i stale/unbound capture;
4. hardened centralny AUR-S07 wiążący zainstalowane artefakty, geometry gate,
   one-time handoff, świeży pojedynczy `nwmain`, realny PNG oraz świeży engine
   log z właściwym modułem.

Po spełnieniu tych warunków należy ponownie odczytać fingerprint procesów i
hash profilu, a następnie wykonać wyłącznie publiczne fazy canonical route.
Sesje mają pozostać otwarte; brak autoryzacji ich zamknięcia.

## Status dwóch osi

| Środowisko | `modelVisibility` | `proofCompleteness` |
| --- | --- | --- |
| Aurora Toolset, exact r27 | `not_tested` | `missing` |
| NWN, exact r27 | `not_tested` | `missing` |

## Aktualny warunek wznowienia live

1. Właściciel ręcznie zamyka responsywną sesję `m2a_now01.mod`; jeżeli pojawi
   się prompt zapisu lub inny nieoczekiwany modal, pozostawia go bez odpowiedzi
   i przekazuje jego treść.
2. Właściciel uruchamia jedną świeżą sesję Toolsetu, wybiera exact
   `m2a_m0r25.mod`, Area `m2a_m0a25` / `Meshy2Aurora M0 binary vertical-slice
   area` oraz `Dwarf Mercenary`, bez Save.
3. Shared operator wykonuje readback tożsamości i świeży zwalidowany
   `TScrollBox`; właściciel zatwierdza obraz.
4. Jeżeli model jest widoczny, ten sam lineage przechodzi do AUR-S07,
   user-owned Test Module i świeżego obrazu NWN.

Ten blocker jest lane/profile/validator blockerem. Nie jest wizualnym wynikiem
`not_visible` i nie dopuszcza r28 ani żadnej zmiany linii artefaktów.
