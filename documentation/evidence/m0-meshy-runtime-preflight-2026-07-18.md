# M0 Meshy runtime preflight (2026-07-18)

Status: `PACKET MATERIALIZED AND INSTALLED / LIVE TOOLSET+NWN PROOF MISSING`.

## Scope

This record continues the M0 Meshy source evidence. It records the exact
recovery, package, install, and native-runtime gate state. It does not claim a
Toolset or NWN result.

## Recovered owner asset

The completed Meshy `REFINE` artifact was recovered through the loopback-only
local Bridge. No new Meshy task was created. The selected GLB has:

- size: `8,581,684` B;
- SHA-256: `aac32ee6197457653b9b247a8f360230cfa0709cb2b2989d85aa155e1699ece1`;
- source stage: `REFINE`;
- profile constraints: static, unskinned source with no source animation.

## Generated packet and own readback

`materialize_m6 --meshy-m0-static-source` created the isolated packet
`proof-output/m0-meshy-golem-20260718` using the owner-provided base
`appearance.2da`. The generated files are:

| File | SHA-256 |
| --- | --- |
| `m2a_m0p01.mdl` | `971a15b0990fb0f308be929e23fc3fb70c8c7342c0c39246613687f6c6c84e66` |
| `m2a_m0t01.tga` | `079bfedbb952ae3f3bbdd43f1bb244eb6247ca92a25d0acad2f3273f9aace59b` |
| `m2a_m0_proof.hak` | `2eb0c336c9ba3de7a40e10b653f5e55a43d88ae5b18d8745a80006fac3623294` |
| `m2a_m0_proof.mod` | `8b3e4f1358d4f788c67d76c4c10a59db138dd8dd47617d74edd3692d7c474c5e` |

The package readback reports two appended `appearance.2da` rows: external
tortoise control `15100` and generated M0 `15101`. M0 is one `RIGID` segment
with seven clean-room, motionless direct-creature lifecycle clips. No tortoise
model or texture payload is in the generated HAK.

## Installation preflight and result

`tools/install-nwn-m0-runtime-proof.ps1` first ran in `Plan` mode, then copied
only absent generated files with a no-overwrite guard. The external reference
HAK was already installed and matched its source exactly:

- `znd_tortoise.hak`: `3f85946f1e86e20b4d9c6fae311f214587da4009080b64376711f630b0327021`;
- installed generated HAK: `m2a_m0_proof.hak`;
- installed generated module: `m2a_m0_proof.mod`.

The installer does not alter the game installation, `nwtoolset.ini`, or any
existing runtime asset.

## Native gate and current audited state

The first nonstandard Toolset attempt is retained only as negative evidence:
it opened the wrong module once and then returned to an unloaded frame. It
never produced a Toolset viewport for `m2a_m0_proof`, did not start NWN, and
must not be retried. It is `failed` for the module-open lane, not evidence for
M0.

After PID-first recovery on 2026-07-18, the current state is clean:

- no running `nwtoolset.exe` or `nwmain.exe` process;
- no visible Toolset/NWN top-level window or modal;
- `\\.\DISPLAY1` exists and is `primary=false` (`1920x1080` at `x=-1920`);
- `tools/install-nwn-m0-runtime-proof.ps1 -Mode Plan` verified all three
  installed inputs without writing: `znd_tortoise.hak`, `m2a_m0_proof.hak`,
  and `m2a_m0_proof.mod`.

### Superseded policy correction

The earlier requirement for a Meshy2Aurora-local approved adapter is
superseded. The shared Aurora operating skills, their canonical runner, and
verified native atoms are central operator tooling and remain usable even when
`aurora-web` product code is reference-only. A local replacement runner or UI
adapter must not be created.

The shared Aurora standard requires an existing, approved Meshy2Aurora
adapter before a live `Open Module` action. `C:\Projects\aurora-web` supplies
the reference fast-module-proof runner and the verified module-open precedent,
but project rules make that repository reference-only. The local
`tools/m2a-aurora-proof.mjs` is an unapproved, untracked runner from the
failed attempt and is explicitly excluded from the live route.

Therefore the live proof remains `missing`: there is no approved project-owned
adapter for `Open Module -> Area viewport proof -> Test Module` at this point.
The precise resume condition is an owner-approved Meshy2Aurora adapter that
implements the shared `aurora-toolset-fast-module-proof-runner-standard.md`
contract, or an explicit owner designation of an already approved local
equivalent. Only then may one fresh single-session attempt open the exact M0
module, confirm `m2a_m0proof_area` plus both fixtures, and proceed to NWN
load-log and capture gates.
