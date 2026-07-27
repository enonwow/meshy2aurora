# M0 r29 — identity streams on all seven type-5 states — 2026-07-21

## Wynik

Zmaterializowano kontrolowaną iterację `r29` po trwałym, candidate-bound
failure exact r28. Jedyna zmiana produktu znajduje się w
`crates/m2a-core/src/model_pipeline.rs`, w
`static_direct_creature_runtime_clips()`: wszystkie siedem istniejących clipów
type `5` emituje teraz istniejącą parę identity controllerów roota:

- translation type `8`, times `[0,1]`, values `[[0,0,0],[0,0,0]]`;
- rotation type `20`, times `[0,1]`, values
  `[[0,0,0,1],[0,0,0,1]]`.

`cpause1` jest semantycznie niezmieniony. Sześć pozostałych stanów dostało
dokładnie dwanaście nowych streamów. Nie dodano clipów, eventów, controllerów
na child mesh placeholderach ani innego kodu runtime.

## Admission gate

R28 jest trwałym źródłem failure i jednocześnie obala multi-HAK ambiguity:

| Dowód r28 | SHA-256 | Wynik |
|---|---|---|
| runtime packet `live/runtime/aur-s07-runtime-packet.json` | `79150527f58ddd158975036d5472d947964b8b7db9723059b0fd74f5bf21e04f` | `modelVisibility=not_visible`, `proofCompleteness=failed` |
| runtime capture `live/runtime/r28-nwn-runtime.png` | `0fc1eafe82190934b2e445a04dd86e024e44f400788ab783420c9bbb37cf6689` | exact fixture nie jest widoczny |
| engine log `live/runtime/engine-load-log.txt` | `b701ee5cb0c4afc332a428a6e4ed610d341730e68964dc61945c423f730d9a06` | exact `m2a_m0r28` loaded |
| cleanup `live/runtime/r28-close-rollback-evidence.json` | `993066836de73483ca5af656f23404a2abd655974c7ba491c0398089e3bfafba` | `nwmain=0`, `nwtoolset=0` |

Wszystkie ścieżki powyżej są pod
`C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721`.
R28 używał dokładnie `[m2a_m0r27]`, więc r29 nie zmienia resolvera po raz
drugi. Testuje wyłącznie brak root state streams na sześciu stanach.

## Exact delta i zachowane invarianty

Model r28/r29 ma te same clip names, każdy type `5`, ten sam animroot, length,
transition, events, dwuwęzłowe animation node trees oraz child mesh placeholder
`contentFlags=0x21` z zerową geometrią. Bazowa semantyka modelu ma po obu
stronach ten sam layout-independent digest SHA-256:

`f4172130a98ff6b3b4f3c9e67e9b915d0b9da72f7dff6293eabed51e369e7b6c`.

MDL zmienił się z `153032` B na `153608` B, czyli dokładnie o `576` B.
TGA i `appearance.2da` są byte-identical względem r27/r28. Fixture pozostaje:
`m0_fixture`, native tree text `Meshy M0 binary vertical-slice fixture`,
`nw_dwarfmerc001`, Appearance `848`, `[10,14.5,0]`, orientation `[1,0]`,
scale `1`. Entry pozostaje `[10,10,0]`, facing `[0,+1]`; Area nadal ma `2x2`.

Nowe identity resrefy lineage to MOD `m2a_m0r29`, Area `m2a_m0a29` oraz HAK
`m2a_m0r29`. Model i tekstura celowo zachowują `m2a_m0p01` i `m2a_m0t01`, bo
ich zmiana rozszerzyłaby A/B poza zatwierdzoną politykę clipów.

## Exact r29 artifacts

| Element | Absolute path | Bytes | SHA-256 |
|---|---|---:|---|
| MOD | `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\generated\m2a_m0r29.mod` | 13173 | `c28af8aff97be895d80c9ac01d5aa5ec909ec6b2a96f8e6ffd0317ffb16931d4` |
| HAK | `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\generated\m2a_m0r29.hak` | 13125014 | `e8c007faace65255cca2e76845db435eadf19c06c1455dedada133a95ddaf9a3` |
| MDL | `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\generated\m2a_m0p01.mdl` | 153608 | `a69fc655d5975b866215f17b142089fa29c0c1f7ea8286db8c0f5136f9e890e7` |
| TGA | `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\generated\m2a_m0t01.tga` | 12582956 | `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b` |
| `appearance.2da` | `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\generated\appearance.2da` | 388194 | `04f7933cd6cb67ae4d065a18c6bed9e18b8688bb97bd4cae5f9072e1d2792acb` |

Ordered HAK list MOD-a to dokładnie `[m2a_m0r29]`. Nowe native targets były
nieobecne przed stagingiem. Zainstalowano byte-identical kopie i ponownie
zweryfikowano hashe:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_m0r29.mod` —
  `c28af8aff97be895d80c9ac01d5aa5ec909ec6b2a96f8e6ffd0317ffb16931d4`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m0r29.hak` —
  `e8c007faace65255cca2e76845db435eadf19c06c1455dedada133a95ddaf9a3`.

Nie nadpisano, nie skopiowano i nie zmieniono żadnego artefaktu r27/r28.

## Profile i lineage

- binary bootstrap:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r29-all-state-identity-binary-bootstrap-v1.json`,
  SHA-256 `f49b5990cd6cb8b6a625bf0a7cd5f2f4b905c77fe5a3312018a15320cb471050`;
- no-Save Gate B:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r29-all-state-identity-toolset-proof-v1.json`,
  SHA-256 `0278403896c5ddc03eea0ceaa121b5c4b28012f1b6519b1da08cbd93c75ade85`;
- machine-readable iteration record:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\m0-r29-all-state-identity-lineage-contract-v1.json`,
  SHA-256 `bde4965dbf99436d61684c90ce3ecd373a681b42dc986d6cac0a7f304a54dee0`.

Runtime profile nie został zmaterializowany. Wymaga świeżego zaakceptowanego
Gate B packetu exact r29 i jego SHA-256.

## TDD i walidacja offline

Test najpierw był RED dla poprzedniej polityki: sześć stanów miało zero
controllerów. Po minimalnej zmianie oba testy są GREEN:

```powershell
cargo test -p m2a-core --test model_pipeline static_meshy_m0_materializes_in_a_self_contained_valid_vertical_slice --quiet
cargo test -p m2a-core --test model_pipeline exact_r29_adds_only_six_identity_root_stream_pairs_to_the_r28_model --quiet
node tools\m0-r29-all-state-identity-lineage.contract.test.mjs
```

Pierwszy test sprawdza siedem exact nazw, type `5`, 14 controllerów, dokładnie
12 streamów poza `cpause1`, brak eventów/child controllerów, placeholdery i
zamrożony base digest. Drugi wiąże exact r28/r29 MDL hashe oraz potwierdza, że
jedyna semantyczna różnica to sześć par root streams. Node contract ponownie
hashuje admission evidence, source/installed r29, profile i trzy zasoby HAK-a;
ma cztery negatywne mutacje fail-closed.

Centralne shared-tooling runy zakończyły się PASS:

- `binary_bootstrap_profile_schema_valid`;
- `binary_bootstrap_structural_readback_valid`;
- `binary_native_geometry_plan_valid`;
- każdy deklaruje `startsToolset=false`, `startsNwn=false`,
  `usesGlobalInput=false`.

## Proof handoff

Powtórzenie read-only Gate A:

```powershell
Set-Location C:\Projects\aurora-web
$binaryProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r29-all-state-identity-binary-bootstrap-v1.json'
$toolsetOut = 'C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\toolset'

node backend\scripts\validate-aurora-toolset-binary-module-bootstrap.mjs --mode dry-run --profile $binaryProfile
node backend\scripts\validate-aurora-toolset-binary-module-bootstrap.mjs --mode preflight --profile $binaryProfile
node backend\scripts\aurora-toolset-binary-module-native-geometry.mjs dry-run --profile $binaryProfile --outDir $toolsetOut
```

Gate B ma użyć shared `aurora-toolset-operate` + `aurora-toolset-prove` i exact
profilu:

```powershell
$toolsetProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r29-all-state-identity-toolset-proof-v1.json'
```

Sekwencja: exact installed/hash-verified MOD/HAK -> Area `m2a_m0a29` -> exact
fixture occurrence `0` -> selection/readback -> świeży zwalidowany
`TScrollBox` -> profile-bound Gate B packet. Profil ma `noSave=true`; nie wolno
uruchamiać `geometry-observe` ani historycznego
`run-aurora-toolset-module-proof.mjs`, ponieważ te trasy obejmują Save lub
tworzenie innego modułu. Nie wolno tworzyć lokalnego runnera.

Po accepted Gate B należy związać jego packet/capture SHA, dopiero potem
zmaterializować exact r29 AUR-S07 runtime profile i wykonać NWN na tej samej
lineage. W tym cyklu Toolset i NWN nie zostały uruchomione, nie wykonano Save.

## Live Gate B — Toolset

Powyższe ostatnie zdanie opisuje stan handoffu przed live proofem. Po
niezależnym `AUDIT PASS` wykonano exact no-Save Gate B r29 przez shared
`aurora-toolset-operate`, `aurora-toolset-author` i `aurora-toolset-prove`.

Centralny startup zachował fail-closed wynik
`unexpected_module_window_loaded`, ponieważ MRU wczytał `m2a_m0r28.mod`.
Manifest pozostał niezmieniony. Po świeżym readbacku jednego proof-owned PID,
czystego frame bez `*`, zera modali i `nwmain=0`, audit-admitted kompozycja
zweryfikowanych atomów wykonała w tej samej sesji exact File -> Open
`m2a_m0r29.mod`. Post-switch readback potwierdził exact frame, niegłówny
`DISPLAY1`, brak dirty state i byte-identical MOD.

W Area `m2a_m0a29` ustawiono Objects mode. Native tree dał dokładnie jeden
match `Meshy M0 binary vertical-slice fixture`, occurrence `0`, depth `3`;
niezależny read-only caret zwrócił ten sam handle i tekst. Świeży zwalidowany
`TScrollBox` pokazuje małą, pionową niebiesko-szarą geometrię exact fixture
wewnątrz zielonego outline'u na teksturowanym terenie. Werdykt:
`modelVisibility=visible`, `proofCompleteness=verified`.

- Gate B packet:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\toolset\r29-gate-b-packet.json`,
  SHA-256 `e64389db6d74133228d8bbee61bdba4c13e0f93e811852ab2d24c23e27fb1f53`;
- capture:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\toolset\r29-after-selection.png`,
  SHA-256 `aeb49f6cb4335f1e91a1f33aa06675b94ec829e006cbb2d905fa0cf19b0fc01b`;
- capture metadata:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\toolset\r29-after-selection.json`,
  SHA-256 `acab0706eac2539ba771b599b6a8298a34d29e4cdce4c95b78c5b5472187de22`;
- same-session continuation:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\toolset\r29-clean-session-switch-continuation.json`,
  SHA-256 `b05f6d6f0e92a365f2b6bea5a77ca252445dfbb51683ce52021cc5fb0b73aec2`.

Nie wykonano Save, Build, repacku, `geometry-observe`, ruchu kamery ani
globalnego inputu. Toolset PID `3720` pozostał otwarty i responsywny pod
proof ownership; NWN nie został uruchomiony.

## Live Gate C — NWN runtime

Po zaakceptowanym Gate B zmaterializowano immutable runtime profile i binding
sidecar:

- `C:\Projects\meshy2aurora\proof-profiles\m0-r29-all-state-identity-runtime-v1.json`,
  SHA-256 `926af303858c86ecb6f923b862527d25034c2b1aef34725e79f47575b6e88814`;
- `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\m0-r29-runtime-v2-binding-sidecar-v1.json`,
  SHA-256 `ab10e85e4f8c0e496d39d1998e4077fd027172bde752bf596a5336f836a79ff4`.

Centralny AUR-S07 dry-run przeszedł dla exact MOD
`c28af8aff97be895d80c9ac01d5aa5ec909ec6b2a96f8e6ffd0317ffb16931d4`,
Area `m2a_m0a29`, entry `[10,10,0]`, ordered HAK `[m2a_m0r29]` i exact fixture.
Po arm request `4a77ef27-77ea-445a-93cc-f58fbe3f4a60` zweryfikowany atom wysłał
dokładnie jeden `Test Module`, menu command ID `121`. Świeży `nwmain` PID
`15780` wystartował `2026-07-21T18:22:00.2632566Z`, a świeży log zawiera:

`[Tue Jul 21 20:22:09] Loading Module: m2a_m0r29`

Request-bound handoff, centralna obserwacja i jedyny centralny capture zachowały
ten sam profile/module/Area/entry lineage. Świeży kadr third-person pokazuje
otwarty, niezasłonięty teren dokładnie przed graczem. Entry jest `[10,10,0]`
z facing `[0,+1]`; jedyny exact fixture jest `[10,14.5,0]`, czyli lateral `0`
i `4.5 m` dokładnie z przodu. Model scale `1` ma wymiary około
`1.0873 x 0.6390 x 1.8930 m`. W tym verdict-permitting kadrze modelu nie ma.

Werdykt exact r29:

- `modelVisibility=not_visible`;
- `proofCompleteness=failed`.

Dowody:

- runtime capture:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\runtime\r29-nwn-runtime.png`,
  SHA-256 `5836a5e43dd9d29c57522cbbd7a7d1f87808f476e5d4da2dda64d0456634e10d`;
- capture metadata:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\runtime\r29-nwn-runtime.capture.json`,
  SHA-256 `dcc3e41a7983c93ebf6f93f1fc1f8a51b39bb41a2b87cb8a9fb75fab413e3511`;
- AUR-S07 packet:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\runtime\aur-s07-runtime-packet.json`,
  SHA-256 `4539b88024fb1b8977cc2a2f92554f6a78cdf0234570bbc01423bc55488b6fb6`;
- validation record:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\runtime\aur-s07-runtime-validation.json`,
  SHA-256 `55235d38b08650693139d848919aeadc15ff382093e6a01601ca502bf8751085`;
- engine-log snapshot:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\runtime\engine-load-log.txt`,
  SHA-256 `a7bac7974eb4307c6b892e9f802e6181b736b2b81a5973872c649d844a9b54c3`.

Centralny validator zwrócił oczekiwane `status=failed` z
`runtime_model_blank_or_placeholder`, ponieważ jego positive gate wymaga
`modelVisibility=visible`. Nie jest to brak capture, identity, logu ani
spatial bindingu; trwały packet klasyfikuje fresh candidate-bound visual
failure. Dodanie identity translation/rotation streamów do wszystkich siedmiu
stanów type-5 nie uczyniło fixture widocznym w NWN. Nie utworzono ani nie
zatwierdzono r30. Nie wykonano Save, Build, repacku, ruchu postaci/kamery ani
globalnego inputu.

## Clean close i rollback evidence

Po trwałym runtime packet/validation zamknięto najpierw exact `nwmain` PID
`15780`, następnie Toolset PID `3720`. NWN otrzymał targetowany `WM_CLOSE`, a
procesowo-lokalny quit confirmation został potwierdzony zweryfikowaną sekwencją
`WM_MOUSEMOVE -> WM_LBUTTONDOWN -> 80 ms -> WM_LBUTTONUP` na exact HWND i
świeżo potwierdzonej geometrii klienta `1904x1041`. Nie użyto globalnego inputu
ani force termination. Toolset był czysty: ukryty exact frame r29 nie miał `*`,
nie było widocznego modala ani Save promptu. Centralny closer z
`--expectedPid 3720` zamknął exact `TApplication` bez Save.

Finalnie `nwmain=0`, `nwtoolset=0`. Source/installed MOD pozostały
byte-identical z SHA-256
`c28af8aff97be895d80c9ac01d5aa5ec909ec6b2a96f8e6ffd0317ffb16931d4`,
a source/installed ordered HAK `[m2a_m0r29]` pozostał byte-identical z SHA-256
`e8c007faace65255cca2e76845db435eadf19c06c1455dedada133a95ddaf9a3`.

Rollback evidence:
`C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\runtime\r29-close-rollback-evidence.json`,
SHA-256 `7bb44a4d0b1458b9d600ff81ebdc9ccae1e31a4854ff6a908d19f496c8e10925`.
