# M0 r29 runtime proof handoff — 2026-07-21

## Wynik

Po świeżym Gate B `visible/verified` zmaterializowano dwuwarstwowy handoff
runtime dla exact r29. Warstwa wykonawcza jest profilem
`aur-s07-runtime-profile/v1`, który przyjmuje istniejący publiczny runner
AUR-S07. Oddzielny immutable project-owned sidecar wiąże exact Gate B,
lineage, fixture i zasoby, których nie ma w schemacie v1.

Nie zmieniono `aurora-web`, nie utworzono lokalnego runnera i nie wykonano
żadnej operacji live. Packet Gate B, lineage contract, MOD, HAK, MDL, TGA,
`appearance.2da` oraz otwarta sesja Toolset pozostają niezmienione.

## Warstwa 1 — central-compatible execution profile

- profil:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r29-all-state-identity-runtime-v1.json`;
- SHA-256:
  `926af303858c86ecb6f923b862527d25034c2b1aef34725e79f47575b6e88814`;
- version/id: `aur-s07-runtime-profile/v1` /
  `m2a-m0-r29-all-state-identity-runtime`;
- MOD SHA-256:
  `c28af8aff97be895d80c9ac01d5aa5ec909ec6b2a96f8e6ffd0317ffb16931d4`;
- Area/entry: `m2a_m0a29` / `[10,10,0]`;
- ordered HAK: wyłącznie `m2a_m0r29`, SHA-256
  `e8c007faace65255cca2e76845db435eadf19c06c1455dedada133a95ddaf9a3`;
- fixture: `m0_fixture`, `m2a_m0p01`, `nw_dwarfmerc001`, Appearance row 848,
  `[10,14.5,0]`;
- launch: `user_owned_test_module_action`, process `nwmain`;
- `noLocalSubstitute=true`.

Profil v1 zawiera wyłącznie pola istniejącego centralnego kontraktu. Jego
`geometryGate.evidence` wskazuje zachowany lineage contract r29; rozszerzony
binding znajduje się wyłącznie w sidecarze.

## Warstwa 2 — v2-style identity sidecar

- sidecar:
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\m0-r29-runtime-v2-binding-sidecar-v1.json`;
- SHA-256:
  `ab10e85e4f8c0e496d39d1998e4077fd027172bde752bf596a5336f836a79ff4`;
- version/id: `m2a-aur-s07-runtime-v2-binding-sidecar/v1` /
  `m2a-m0-r29-all-state-identity-runtime-v2-binding`.

Sidecar wiąże:

- execution profile v1 i jego exact SHA;
- source binary profile SHA-256
  `f49b5990cd6cb8b6a625bf0a7cd5f2f4b905c77fe5a3312018a15320cb471050`;
- zachowany lineage contract SHA-256
  `bde4965dbf99436d61684c90ce3ecd373a681b42dc986d6cac0a7f304a54dee0`;
- finalny Gate B packet
  `C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\toolset\r29-gate-b-packet.json`,
  SHA-256 `e64389db6d74133228d8bbee61bdba4c13e0f93e811852ab2d24c23e27fb1f53`;
- fresh validated `TScrollBox` PNG SHA-256
  `aeb49f6cb4335f1e91a1f33aa06675b94ec829e006cbb2d905fa0cf19b0fc01b`;
- capture JSON SHA-256
  `acab0706eac2539ba771b599b6a8298a34d29e4cdce4c95b78c5b5472187de22`;
- Gate B verdict `modelVisibility=visible`,
  `proofCompleteness=verified`;
- exact source i installed MOD path, Area, entry, ordered HAK, native fixture
  name, pozycję, orientation i scale;
- MDL `m2a_m0p01` SHA-256
  `a69fc655d5975b866215f17b142089fa29c0c1f7ea8286db8c0f5136f9e890e7`;
- niezmieniony TGA `m2a_m0t01` SHA-256
  `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`;
- niezmieniony `appearance.2da` row 848 SHA-256
  `04f7933cd6cb67ae4d065a18c6bed9e18b8688bb97bd4cae5f9072e1d2792acb`.

Sidecar nie zastępuje centralnego profilu ani runnera. Jest trwałym kontraktem
tożsamości, który proof owner sprawdza wraz z profilem v1 przed finalnym
przyjęciem runtime packetu.

## Weryfikacja offline

Lokalny kontrakt:

```powershell
Set-Location C:\Projects\meshy2aurora
node tools\m0-r29-runtime-profile.contract.test.mjs
```

Wynik: `m0_r29_runtime_profile_and_binding_contract_valid`, PASS. Test hashuje
wszystkie bound artifacts, w tym source i installed MOD/HAK, Gate B packet,
PNG, capture JSON, profile, lineage oraz MDL/TGA/2DA. Osiem negatywnych
przypadków fail-closed obejmuje dodatkowy HAK, zmienione SHA Gate B, capture,
MOD i MDL, złą Area, Appearance row oraz odwrócony verdict.

Publiczny centralny dry-run:

```powershell
Set-Location C:\Projects\aurora-web
node backend\scripts\aur-s07-runtime-execution.mjs dry-run `
  --profile 'C:\Projects\meshy2aurora\proof-profiles\m0-r29-all-state-identity-runtime-v1.json' `
  --outDir 'C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\runtime'
```

Wynik: `ok=true`, `command=dry-run`, exact MOD SHA/Area/entry, wymagane
`runtime-observation`, `runtime-capture`, `engine-load-log`,
`startsNwn=false`, `usesGlobalInput=false`.

## Exact proof command handoff

Poniższa sekwencja jest przeznaczona dla proof ownera. W tej materializacji
wykonano wyłącznie pierwsze, offline `dry-run`.

```powershell
Set-Location C:\Projects\aurora-web

$runtimeProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r29-all-state-identity-runtime-v1.json'
$runtimeBinding = 'C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\m0-r29-runtime-v2-binding-sidecar-v1.json'
$runtimeOut = 'C:\Projects\meshy2aurora\proof-output\m0-r29-all-state-identity-20260721\live\runtime'

node backend\scripts\aur-s07-runtime-execution.mjs dry-run `
  --profile $runtimeProfile `
  --outDir $runtimeOut

node backend\scripts\aur-s07-runtime-execution.mjs arm `
  --profile $runtimeProfile `
  --outDir $runtimeOut
```

Proof owner wykonuje dokładnie jeden `Test Module` dla otwartego exact r29
przez shared `aurora-toolset-operate` i `aurora-toolset-prove`. Handoff musi
mieć version `aur-s07-runtime-handoff/v1`, requestId z
`aur-s07-runtime-launch-request.json`, profileId
`m2a-m0-r29-all-state-identity-runtime`, exact MOD SHA, Area `m2a_m0a29`,
entry `[10,10,0]`, rzeczywisty PID `nwmain` i fresh
`Loading Module: m2a_m0r29`.

```powershell
node backend\scripts\aur-s07-runtime-execution.mjs observe `
  --profile $runtimeProfile `
  --outDir $runtimeOut `
  --handoff "$runtimeOut\runtime-handoff.json"

node backend\scripts\aur-s07-runtime-execution.mjs validate `
  --profile $runtimeProfile `
  --packet "$runtimeOut\aur-s07-runtime-packet.json"

Set-Location C:\Projects\meshy2aurora
node tools\m0-r29-runtime-profile.contract.test.mjs
```

Final acceptance wymaga równocześnie centralnego `status=verified`, zgodności
runtime packetu z exact profilem v1 oraz niezmienionych SHA profilu i sidecaru.
