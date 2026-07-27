# M0 r31 runtime proof handoff — 2026-07-22

## Scope

This record materializes only the declarative runtime handoff for the existing
`m0-r31-hierarchy-only-20260722` lineage. It creates no model iteration, changes
no MOD/HAK/MDL/TGA/2DA payload, and performs no Toolset or NWN action.

## Accepted Gate B identity

- packet: `C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\live\toolset\gate-b-r31-pid6536-packet.json`
  - SHA-256: `68547720a0de2e23c00004198cbedc093d42893587cac74f9593f3fc9568ba89`
- accepted PNG: `C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\live\toolset\gate-b-r31-pid6536-viewport.png`
  - SHA-256: `aeb49f6cb4335f1e91a1f33aa06675b94ec829e006cbb2d905fa0cf19b0fc01b`
- capture JSON: `C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\live\toolset\gate-b-r31-pid6536-viewport.capture.json`
  - SHA-256: `8916d660085853a78fa68ce052e76ec81514fb65d4af12b4a5887346b6d13395`
- accepted axes: `modelVisibility=visible`, `proofCompleteness=verified`
- exact selection: PID `6536`, Area `m2a_m0a31`, occurrence `0`, handle
  `105874560`.

## Runtime handoff artifacts

- central-compatible execution profile:
  `C:\Projects\meshy2aurora\proof-profiles\m0-r31-hierarchy-only-runtime-v1.json`
  - SHA-256: `77c8b0d47a9ccc40fcbcb9114e3c0ebd6314f49a5afedfbbbbca82a7f453dbf3`
- immutable project binding sidecar:
  `C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\m0-r31-runtime-v2-binding-sidecar-v1.json`
  - SHA-256: `9584820100f14c06d23af3316074b256668d423417e29f9e11a4f7a3cf150f17`
- local contract:
  `C:\Projects\meshy2aurora\tools\m0-r31-runtime-profile.contract.test.mjs`

The public v1 profile contains only fields accepted by the shared AUR-S07
runner. The sidecar separately binds the exact MOD
`8575465ef683256a54c101a3f4445a4e22803194c9b899a0e43e3fd069fdeafd`,
Area `m2a_m0a31`, singleton ordered HAK `[m2a_m0r31]` with SHA-256
`ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12`,
fixture row `15100`, MDL
`fcfbe7e329d51aef6ccb3ae87b4bfe7db7c5395cd5e8adf24159e73e556b5ab6`,
TGA `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b`,
and full appearance table
`48d313b75761809e2231c99ad51e17d67632c1ce10e70b87fa8b1ccdfbc474a6`.
It also pins the production V4 source-topology, state-projection, engine-envelope,
raw-MDX, protected-writer and FullRuntimeAppend identities.

## Offline validation

```powershell
Set-Location C:\Projects\meshy2aurora
node tools\m0-r31-runtime-profile.contract.test.mjs
```

Result: PASS, status `m0_r31_runtime_profile_and_binding_contract_valid`;
17 negative identity/provenance mutations fail closed.

```powershell
Set-Location C:\Projects\aurora-web
node backend\scripts\aur-s07-runtime-execution.mjs dry-run `
  --profile 'C:\Projects\meshy2aurora\proof-profiles\m0-r31-hierarchy-only-runtime-v1.json' `
  --outDir 'C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\live\runtime'
```

Result: PASS (`ok=true`, `command=dry-run`, exact MOD/Area/entry,
`startsNwn=false`, `usesGlobalInput=false`). No `arm`, `observe`, `validate`,
Toolset action, or NWN action was executed while creating this handoff.

## Exact proof-owner command handoff

```powershell
Set-Location C:\Projects\aurora-web

$runtimeProfile = 'C:\Projects\meshy2aurora\proof-profiles\m0-r31-hierarchy-only-runtime-v1.json'
$runtimeBinding = 'C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\m0-r31-runtime-v2-binding-sidecar-v1.json'
$runtimeOut = 'C:\Projects\meshy2aurora\proof-output\m0-r31-hierarchy-only-20260722\live\runtime'

node backend\scripts\aur-s07-runtime-execution.mjs dry-run `
  --profile $runtimeProfile `
  --outDir $runtimeOut

node backend\scripts\aur-s07-runtime-execution.mjs arm `
  --profile $runtimeProfile `
  --outDir $runtimeOut
```

After `arm`, the proof owner performs exactly one shared-operator Test Module
action for the already-open exact r31 lineage and writes a version
`aur-s07-runtime-handoff/v1` file at `$runtimeOut\runtime-handoff.json`. It must
bind the request ID from `aur-s07-runtime-launch-request.json`, profile ID
`m2a-m0-r31-hierarchy-only-runtime`, exact MOD SHA, Area `m2a_m0a31`, entry
`[10,10,0]`, the actual `nwmain` PID, and fresh `Loading Module: m2a_m0r31`.

```powershell
node backend\scripts\aur-s07-runtime-execution.mjs observe `
  --profile $runtimeProfile `
  --outDir $runtimeOut `
  --handoff "$runtimeOut\runtime-handoff.json"

node backend\scripts\aur-s07-runtime-execution.mjs validate `
  --profile $runtimeProfile `
  --packet "$runtimeOut\aur-s07-runtime-packet.json"

Set-Location C:\Projects\meshy2aurora
node tools\m0-r31-runtime-profile.contract.test.mjs
```

Runtime acceptance remains unset until the shared central packet, fresh runtime
capture, engine-log window, visual verdict, and clean closure are complete.
