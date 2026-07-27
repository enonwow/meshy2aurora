# M0 r28 — single-r27-HAK controlled iteration — 2026-07-21

## Wynik

Zmaterializowano najmniejszą kontrolowaną iterację usuwającą nierozstrzygnięty
wybór zasobu między trzema aktywnymi HAK-ami. Nowy MOD `m2a_m0r28` zawiera
dokładnie jeden wpis `Mod_HakList`: istniejący `m2a_m0r27`. Nie utworzono,
nie skopiowano i nie zainstalowano HAK-a r28.

Zmiana identity `m2a_m0r25/m2a_m0a25 -> m2a_m0r28/m2a_m0a28` jest wymaganą
przez iteration gate nową, niezmienną nazwą kontenera. Jedyną zmienną
runtime-resource względem exact r27 jest:

`[m2a_m0r27, m2a_m0r26, m2a_m0r21] -> [m2a_m0r27]`.

MDL, TGA i `appearance.2da` nie zostały zregenerowane ani skopiowane. R28
odwołuje się do byte-identical, już zainstalowanego HAK-a r27.

## Admission gate

Źródłowy exact r27:

- MOD `m2a_m0r25.mod`, SHA-256
  `4a4f98ce14c41940be215ae69739a397294b61f24db2b973a1d9aa8de2a4c257`;
- ordered HAK:
  - `m2a_m0r27` — `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd`;
  - `m2a_m0r26` — `6f805873420b5279b45dadc26d913fa7692374df9e0582c23b42c8a4b42d6bb0`;
  - `m2a_m0r21` — `25b753f6f1b16ed74615bf0243d4d91a245d5428de8963b49ad2b4d4ebd7c1d6`;
- Toolset: `modelVisibility=visible`, `proofCompleteness=verified`;
- NWN: `modelVisibility=not_visible`, `proofCompleteness=failed`;
- trwały packet:
  `C:\Projects\meshy2aurora\proof-output\m0-r27-animation-type5-20260721\live\runtime\aur-s07-runtime-packet.json`,
  SHA-256 `ae9d94af029ad164322eaa874cfd8a2e7ace56afc6a39e50c27dda95ff0664f3`;
- runtime capture:
  `C:\Projects\meshy2aurora\proof-output\m0-r27-animation-type5-20260721\live\runtime\r27-nwn-runtime.png`,
  SHA-256 `45540e33c08c35c768df896b95292b5c87cfee2a645382f391070132f2e95524`.

Przed materializacją proof owner zamknął sesje; niezależny readback wykazał
`nwmain=0` i `nwtoolset=0`. Następnie przeszedł canonical guard dla
`C:\Projects\meshy2aurora`.

## Diagnoza i kolejność hipotez

1. **H1 — multi-HAK resource winner ambiguity (najwyższy priorytet).** Każdy
   z r27, r26 i r21 zawiera te same klucze `appearance/2017`,
   `m2a_m0p01/2002`, `m2a_m0t01/3`. Profile wiązały hashe kontenerów, lecz
   packet runtime nie zawiera engine trace wybranego payloadu. R28 usuwa tę
   jedną zmienną przez pojedynczy aktywny HAK, bez zmiany payloadu.
2. **H2 — runtime local-state/animation activation.** Jeżeli single-r27 HAK
   nadal da świeże `not_visible`, następną hipotezą jest semantyka aktywnego
   stanu klienta NWN dla istniejących siedmiu local animations. Nie została
   włączona do r28 i nie wolno dodawać controllerów, aliasów ani geometrii
   state bez nowego evidence-bound A/B.
3. **H3 — stale cache/override dla stałych resrefów.** Świeża sesja i jeden
   aktywny HAK są pierwszym testem. Zmiana resrefów modelu/tekstury nie jest
   częścią r28, bo zmieniłaby kilka artefaktów naraz i zamaskowała H1.
4. **H4 — 2DA/geometria/materiał/bounds/root/classification.** To hipotezy
   niskiego priorytetu: row 848 i zasoby są hash-bound, Toolset pokazał model,
   a wcześniejszy audit nie znalazł offline podstaw do ich zmiany.

R28 jest więc testem H1, a nie deklaracją, że H1 została już potwierdzona jako
ostateczna przyczyna renderera.

## Exact r28 lineage

| Element | Wartość |
|---|---|
| MOD | `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\generated\m2a_m0r28.mod` |
| MOD SHA-256 / bytes | `08ac62b9b3decd33322657a25996557dcdf96e3db0a2d9f7f292f8b72be4cce2` / `13173` |
| Module / Area | `m2a_m0r28` / `m2a_m0a28` |
| Ordered HAK | wyłącznie `m2a_m0r27` |
| HAK SHA-256 | `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd` |
| MDL `m2a_m0p01` | `bf7864f0ed541ed93c4075d167f4a2392eeb6ef37b4cdf444cbb74efd5731578`, `153032` B |
| TGA `m2a_m0t01` | `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`, `12582956` B |
| `appearance.2da` | `04f7933cd6cb67ae4d065a18c6bed9e18b8688bb97bd4cae5f9072e1d2792acb`, row 848 |
| Entry | `[10,10,0]`, facing `[0,+1]` |
| Fixture | `m0_fixture`, `nw_dwarfmerc001`, Appearance 848, `[10,14.5,0]`, orientation `[1,0]`, scale 1 |
| Native tree name | `Meshy M0 binary vertical-slice fixture` |

Profile i kontrakt:

- binary bootstrap:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r28-single-r27-binary-bootstrap-v1.json`,
  SHA-256 `0d2311b4929e5768249e9e20e5b327efd14e6f12cb73c6785380ef5cfb1b43e3`;
- no-Save Gate B:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r28-single-r27-toolset-proof-v1.json`,
  SHA-256 `146142d9db03c6aab1b2cbf93205eea25cdaa302458732040792eec307b046e7`;
- machine-readable lineage:
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\m0-r28-single-r27-lineage-contract-v1.json`,
  SHA-256 `c5c16f42e3035e41b69db0cae68e323ebf4bffb5dee65ca8bc55630fb5f4732e`.

Runtime profile nie został zmaterializowany. Wymaga świeżego, rzeczywistego
packetu Gate B r28 i jego SHA-256; wpisanie fikcyjnego hasha byłoby złamaniem
kontraktu.

Powyższe dwa zdania opisują stan przed live proofem. Zostały zastąpione
rzeczywistymi artefaktami Gate B i Gate C opisanymi niżej; pozostają w dokumencie
wyłącznie jako historyczny zapis kolejności materializacji.

## Weryfikacja offline

Uruchomiono:

```powershell
cargo test -p m2a-core --test m0_r28_single_r27_lineage --quiet
node tools\m0-r28-single-r27-lineage.contract.test.mjs
```

Wynik: oba testy PASS. Test Rust rekonstruuje MOD i wykonuje własny semantic
readback IFO/GIT: jeden HAK r27, exact entry/facing, fixture, Appearance,
position/orientation i native name. Test Node ponownie hashuje MOD, HAK,
profile, source packet/capture i porównuje MDL/TGA/2DA z r27 materialization
manifestem; sprawdza też brak HAK-a r28.

Centralne read-only dry-runy, wykonane z
`C:\Projects\aurora-web` jako shared operator tooling:

- `binary_bootstrap_profile_schema_valid`;
- `binary_bootstrap_structural_readback_valid`;
- `binary_native_geometry_plan_valid`;
- wszystkie deklarują `startsToolset=false`, `startsNwn=false`,
  `usesGlobalInput=false`.

## Proof handoff

Powtórzenie read-only Gate A:

```powershell
Set-Location C:\Projects\aurora-web
$binaryProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r28-single-r27-binary-bootstrap-v1.json'
$toolsetOut = 'C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\toolset'

node backend\scripts\validate-aurora-toolset-binary-module-bootstrap.mjs --mode dry-run --profile $binaryProfile
node backend\scripts\validate-aurora-toolset-binary-module-bootstrap.mjs --mode preflight --profile $binaryProfile
node backend\scripts\aurora-toolset-binary-module-native-geometry.mjs dry-run --profile $binaryProfile --outDir $toolsetOut
```

Live handoff ma użyć aktualnych `aurora-toolset-operate`,
`aurora-toolset-author` i `aurora-toolset-prove`, wskazując:

```powershell
$toolsetProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r28-single-r27-toolset-proof-v1.json'
```

Sekwencja jest no-Save: exact MOD/Area -> exact fixture
`Meshy M0 binary vertical-slice fixture` occurrence 0 -> readback -> świeży
zwalidowany `TScrollBox` -> profile-bound Gate B packet. Nie wolno wykonywać
`geometry-observe`, ponieważ historyczna native-geometry kontynuacja prowadzi
do Save, a r28 Gate B ma `noSave=true`. Brak publicznego all-in-one runnera
nie zezwala na lokalny adapter; proof owner używa wyłącznie wspólnych skillów,
kanonicznego operatora i zweryfikowanych atomów.

Po zaakceptowanym Gate B należy wyliczyć SHA jego świeżego packetu, dopiero
wtedy utworzyć runtime profile i uruchomić NWN na tej samej lineage. R27
packet, MOD, HAK-i i obrazy pozostają historycznie niezmienne.

## Live Gate B — Toolset

Gate B został wykonany w jednej proof-owned sesji Toolsetu dla exact
`m2a_m0r28.mod`, Area `m2a_m0a28` i jedynej instancji
`Meshy M0 binary vertical-slice fixture` occurrence 0. Selection/readback oraz
świeży zwalidowany obraz `TScrollBox` domknęły identity bez Save, Build, repacku,
`geometry-observe` ani zmiany MOD/HAK.

- verdict: `modelVisibility=visible`, `proofCompleteness=verified`;
- packet:
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\toolset\r28-gate-b-packet.json`,
  SHA-256 `4084246f96b2425d4fba766b4ffeb60a6288f4862d56624bf95ae0a6d4f6d584`;
- capture:
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\toolset\r28-after-selection.png`,
  SHA-256 `ecca0f8f37c2bd991039e07b1a6e2c395f2f44c1c9e07127794b0214a6a27231`;
- capture metadata:
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\toolset\r28-after-selection.json`,
  SHA-256 `f0b00ba9882dce5831cf692846bcdc66ce3887a54a18c64e2d10c0a78e1dd93a`.

## Live Gate C — NWN exact r28

Runtime został uruchomiony dokładnie raz przez publiczną centralną trasę
AUR-S07/Test Module, z request-bound handoffem i świeżym `nwmain`. Profil i
immutable binding sidecar:

- `C:\Projects\meshy2aurora\proof-profiles\m0-r28-single-r27-runtime-v1.json`,
  SHA-256 `3f8d2d56e907822a80315bbd3a1dc05aea6a28981110d760cf81663208cf65f8`;
- `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\m0-r28-runtime-v2-binding-sidecar-v1.json`,
  SHA-256 `69cf69ba5cac02e8a4d1c974abe1909a30c6310b6925666bbbf60b0a60b2f723`.

Fresh engine log zawiera exact linię
`[Tue Jul 21 19:14:14] Loading Module: m2a_m0r28`. Capture pokazuje osadzoną
kamerę third-person za graczem oraz otwarty, niezasłonięty teren dokładnie przed
nim. Binding umieszcza gracza w `[10,10,0]`, facing `[0,+1]`, a jedyny fixture
w `[10,14.5,0]`: lateral 0, dokładnie 4,5 m z przodu, scale 1, bounds modelu
około `[1.0873,0.6390,1.8930]` m. Exact model nie jest widoczny w kadrze, który
pozwala na werdykt. Wynik Gate C jest zatem świeżym candidate-bound failure:
`modelVisibility=not_visible`, `proofCompleteness=failed`.

- runtime packet:
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\runtime\aur-s07-runtime-packet.json`,
  SHA-256 `79150527f58ddd158975036d5472d947964b8b7db9723059b0fd74f5bf21e04f`;
- runtime capture:
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\runtime\r28-nwn-runtime.png`,
  SHA-256 `0fc1eafe82190934b2e445a04dd86e024e44f400788ab783420c9bbb37cf6689`;
- capture metadata:
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\runtime\r28-nwn-runtime.capture.json`,
  SHA-256 `92b702fa7ea815f67aee9f54090440544fbf453329198eef42a8a9df73e12b14`;
- launch request / handoff / observation:
  SHA-256 odpowiednio
  `32dee591986c5beed38555dbf51b9359bdeed9ea91d46ef6532a87347b0b35ee`,
  `1b608efa6310e7238d9d7f173b06bb2cc2abdac3894442724a0cecfb015739ac`,
  `3018626050e9f905ff14962318f826acf85bf9728b645deab0496ccaa3fe6c29`;
- engine log snapshot:
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\runtime\engine-load-log.txt`,
  SHA-256 `b701ee5cb0c4afc332a428a6e4ed610d341730e68964dc61945c423f730d9a06`;
- validator record:
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\runtime\aur-s07-runtime-validation.json`,
  SHA-256 `f86f7cf4d5d3ccf08edf1c3a0e462be247d9ecca889f5f3c116f68f35f24172f`.

Centralny validator potwierdził identity, hashes, fixture i wymagane artefakty;
jego jedyną failure jest oczekiwane `runtime_model_blank_or_placeholder`, czyli
ten sam trwały wynik `not_visible/failed`. Izolacja ordered HAK do wyłącznie
`m2a_m0r27` nie przywróciła modelu w runtime, więc H1 nie wystarcza jako pełna
diagnoza przyczyny.

## Czyste zamknięcie i rollback evidence

Po durable proof zamknięto najpierw exact `nwmain`, następnie proof-owned
Toolset, wyłącznie targeted/no-global-input trasą. Nie wystąpił Save prompt;
nie wykonano Save, Build, repacku, mutacji modułu/HAK ani force-kill. Stan
końcowy: `nwmain=0`, `nwtoolset=0`.

Evidence zamknięcia:
`C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\runtime\r28-close-rollback-evidence.json`,
SHA-256 `993066836de73483ca5af656f23404a2abd655974c7ba491c0398089e3bfafba`.
Post-close rehash potwierdził byte-identical source/installed MOD SHA-256
`08ac62b9b3decd33322657a25996557dcdf96e3db0a2d9f7f292f8b72be4cce2`
oraz niezmieniony installed HAK SHA-256
`8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd`.
