# M0 r28 runtime proof handoff — 2026-07-21

## Wynik

Po świeżym Gate B `visible/verified` zmaterializowano dwuwarstwowy handoff
runtime dla exact r28. Warstwa wykonawcza jest profilem
`aur-s07-runtime-profile/v1`, który obecny publiczny runner AUR-S07 faktycznie
przyjmuje. Osobny, immutable project-owned sidecar wiąże brakujące w schemacie
v1 tożsamości Gate B, capture, binarnego profilu, lineage i zasobów.

Nie zmieniono `aurora-web`, nie utworzono lokalnego runnera i nie wykonano
żadnej operacji live. Historyczny packet Gate B, lineage contract oraz r27/r28
MOD/HAK/MDL/TGA/2DA pozostają niezmienione.

## Warstwa 1 — central-compatible execution profile

- profil:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r28-single-r27-runtime-v1.json`;
- SHA-256:
  `3f8d2d56e907822a80315bbd3a1dc05aea6a28981110d760cf81663208cf65f8`;
- version/id:
  `aur-s07-runtime-profile/v1` / `m2a-m0-r28-single-r27-runtime`;
- MOD SHA-256:
  `08ac62b9b3decd33322657a25996557dcdf96e3db0a2d9f7f292f8b72be4cce2`;
- Area/entry: `m2a_m0a28` / `[10,10,0]`;
- ordered HAK: wyłącznie `m2a_m0r27`, SHA-256
  `8714f7417ea06abc081a6d387dc2be14ac65e8f322292ae24f8ba9fa60984afd`;
- fixture: `m0_fixture`, `m2a_m0p01`, `nw_dwarfmerc001`, Appearance row 848,
  `[10,14.5,0]`;
- launch: `user_owned_test_module_action`, process `nwmain`;
- `noLocalSubstitute=true`.

Do profilu v1 nie dodano pól spoza istniejącego schematu. Jego
`geometryGate.evidence` wskazuje zachowany lineage contract r28; jego hash i
pełna semantyka są dodatkowo zamknięte w sidecarze.

## Warstwa 2 — v2-style identity sidecar

- sidecar:
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\m0-r28-runtime-v2-binding-sidecar-v1.json`;
- SHA-256:
  `69cf69ba5cac02e8a4d1c974abe1909a30c6310b6925666bbbf60b0a60b2f723`;
- version/id:
  `m2a-aur-s07-runtime-v2-binding-sidecar/v1` /
  `m2a-m0-r28-single-r27-runtime-v2-binding`.

Sidecar wiąże:

- execution profile v1 i jego exact SHA;
- source binary profile
  `C:\Projects\meshy2aurora\proof-profiles\m0-r28-single-r27-binary-bootstrap-v1.json`,
  SHA-256 `0d2311b4929e5768249e9e20e5b327efd14e6f12cb73c6785380ef5cfb1b43e3`;
- zachowany lineage contract, SHA-256
  `c5c16f42e3035e41b69db0cae68e323ebf4bffb5dee65ca8bc55630fb5f4732e`;
- finalny Gate B packet
  `C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\toolset\r28-gate-b-packet.json`,
  SHA-256 `4084246f96b2425d4fba766b4ffeb60a6288f4862d56624bf95ae0a6d4f6d584`;
- fresh validated `TScrollBox` PNG, SHA-256
  `ecca0f8f37c2bd991039e07b1a6e2c395f2f44c1c9e07127794b0214a6a27231`;
- capture JSON, SHA-256
  `f0b00ba9882dce5831cf692846bcdc66ce3887a54a18c64e2d10c0a78e1dd93a`;
- Gate B verdict `modelVisibility=visible`,
  `proofCompleteness=verified`;
- exact source i installed MOD path, Area, entry, jeden ordered r27 HAK,
  native fixture name, occurrence-independent source identity, pozycję,
  orientation i scale;
- MDL `m2a_m0p01` SHA-256
  `bf7864f0ed541ed93c4075d167f4a2392eeb6ef37b4cdf444cbb74efd5731578`;
- TGA `m2a_m0t01` SHA-256
  `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`;
- `appearance.2da` row 848 SHA-256
  `04f7933cd6cb67ae4d065a18c6bed9e18b8688bb97bd4cae5f9072e1d2792acb`.

Sidecar nie jest substytutem centralnego profilu ani runnerem. Jest trwałym
kontraktem tożsamości, który proof owner sprawdza razem z profilem v1 przed
przyjęciem runtime packetu.

## Weryfikacja offline

Uruchomiono po canonical guardzie:

```powershell
Set-Location C:\Projects\meshy2aurora
node tools\m0-r28-runtime-profile.contract.test.mjs
```

Wynik: `m0_r28_runtime_profile_and_binding_contract_valid`, PASS. Test hashuje
wszystkie istniejące bound artifacts, porównuje Gate B packet/capture,
source binary profile, lineage, exact MOD, pojedynczy HAK, fixture i hashe
MDL/TGA/2DA. Pięć negatywnych przypadków odrzuca dodatkowy HAK, zmieniony hash
Gate B, MOD, Appearance row oraz MDL.

Publiczny centralny dry-run:

```powershell
Set-Location C:\Projects\aurora-web
node backend\scripts\aur-s07-runtime-execution.mjs dry-run `
  --profile 'C:\Projects\meshy2aurora\proof-profiles\m0-r28-single-r27-runtime-v1.json' `
  --outDir 'C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\runtime'
```

Wynik: `ok=true`, `command=dry-run`, exact MOD SHA/Area/entry, wymagane
`runtime-observation`, `runtime-capture`, `engine-load-log`,
`startsNwn=false`, `usesGlobalInput=false`.

## Exact proof command handoff

Poniższa sekwencja jest przeznaczona dla proof ownera. W tym kroku nie została
uruchomiona poza pierwszym, read-only `dry-run`.

```powershell
Set-Location C:\Projects\aurora-web

$runtimeProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r28-single-r27-runtime-v1.json'
$runtimeBinding = 'C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\m0-r28-runtime-v2-binding-sidecar-v1.json'
$runtimeOut = 'C:\Projects\meshy2aurora\proof-output\m0-r28-single-r27-hak-20260721\live\runtime'

node backend\scripts\aur-s07-runtime-execution.mjs dry-run `
  --profile $runtimeProfile `
  --outDir $runtimeOut

node backend\scripts\aur-s07-runtime-execution.mjs arm `
  --profile $runtimeProfile `
  --outDir $runtimeOut
```

Następnie proof owner wykonuje dokładnie jeden `Test Module` dla już otwartego,
exact r28 przez shared `aurora-toolset-operate` + `aurora-toolset-prove`.
Powstały handoff musi mieć version `aur-s07-runtime-handoff/v1`, requestId z
`aur-s07-runtime-launch-request.json`, profileId
`m2a-m0-r28-single-r27-runtime`, exact MOD SHA, Area `m2a_m0a28`, entry
`[10,10,0]`, rzeczywisty `nwmain` PID i fresh `Loading Module: m2a_m0r28`.

```powershell
node backend\scripts\aur-s07-runtime-execution.mjs observe `
  --profile $runtimeProfile `
  --outDir $runtimeOut `
  --handoff "$runtimeOut\runtime-handoff.json"
```

Capture oraz niezależny visual verdict powstają wyłącznie przez shared
operate/prove. Finalny packet v1 musi zawierać exact fixtures z profilu oraz
hash-bound `runtime-observation`, prawdziwy runtime PNG i świeży wycinek logu.

```powershell
node backend\scripts\aur-s07-runtime-execution.mjs validate `
  --profile $runtimeProfile `
  --packet "$runtimeOut\aur-s07-runtime-packet.json"

Set-Location C:\Projects\meshy2aurora
node tools\m0-r28-runtime-profile.contract.test.mjs
```

Final acceptance wymaga równocześnie centralnego `status=verified`, zgodności
runtime packetu z exact profilem v1 oraz niezmienionego SHA sidecaru. Sam
centralny v1 nie zastępuje sidecarowego bindingu Gate B i zasobów.
