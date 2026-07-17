# Meshy Local Bridge — runbook

Status: implementation guide for the optional local companion introduced with Meshy Lab.

## Purpose and boundary

The Bridge is a local process, not a product backend. It binds only to
`127.0.0.1`, owns `MESHY_API_KEY`, and exposes the narrow `/v1/*` contract used
by Studio. React, the Web Worker, WASM, browser storage, proof manifests and
repository files never receive the API key or Meshy signed URLs.

The implementation is at `tools/meshy-local-bridge/index.mjs`; its security
and protocol tests are `tools/meshy-local-bridge/bridge.test.mjs`.

## Start locally

PowerShell example for local Studio development:

```powershell
$env:MESHY_API_KEY = "owner-provided-key"
$env:MESHY_BRIDGE_ALLOWED_ORIGIN = "http://localhost:5173"
$env:VITE_MESHY_LAB = "1"
node tools/meshy-local-bridge/index.mjs
```

The Bridge prints a one-time pairing code. Enter it in Meshy Lab after clicking
**Open Meshy Lab** on Source. Do not add the key, pairing code, task response,
or signed artifact URL to source control, a screenshot, or a public issue.

For a static deployment, `MESHY_BRIDGE_ALLOWED_ORIGIN` must be the exact
deployed Studio origin. Wildcards and LAN binding are intentionally unsupported.
`VITE_MESHY_LAB=1` must be present when Vite starts/builds; the optional Lab is
hidden by default when the feature is not explicitly enabled.

## Supported proof profiles

| Profile | Pipeline |
| --- | --- |
| H1 humanoid animated | Text-to-3D preview -> refine -> rigging -> Animation API action `0` (Idle) -> GLB |
| N1 quadruped | Text-to-3D preview -> refine -> GLB |
| S1 static prop | Text-to-3D preview -> refine -> GLB |

Only H1 can use rigging/animation. The Bridge uses Meshy Text-to-3D v2 with
`target_formats: ["glb"]`; it obtains the final binary itself, checks size,
calculates SHA-256, and exposes it only after status `READY`.

### Geometry targets

`AURORA_PROOF` is the default Meshy Lab choice. It sends
`should_remesh: true` with `target_polycount: 1500`. It is a target, not an
exact Meshy guarantee, so Studio intake must still measure the downloaded GLB
before an asset is approved for an Aurora proof. The other explicit choices
remain `LOWER_DETAIL` (10,000), `BALANCED` (30,000), and `HIGHER_DETAIL`
(60,000); they are not appropriate defaults for the first Aurora proof assets.

## Operational limits

- A browser session expires after 15 minutes. The pairing code is one-use; restart
  the Bridge to obtain a fresh code before pairing again.
- `confirmationNonce` is one-use. The Bridge does not retry a paid create request.
- `Cancel run` stops the local pipeline from beginning further stages. A Meshy
  task that is already running may still consume credits; the UI must not claim
  that Meshy refunds or cancels it.
- Actual credit usage and task status come from Meshy. The UI maximum is a
  safety/review estimate, not a billing guarantee.
- Real E2E is manual and requires an owner-approved API key plus an agreed
  credit cap. It is never a default CI test.

## Required checks before a paid run

1. Run `node --test bridge.test.mjs` in `tools/meshy-local-bridge`.
2. Run `npm test` and `npm run typecheck` in `apps/studio-web`.
3. Confirm the Bridge listens only at `127.0.0.1`, the origin is exact, and the
   balance and maximum cost are visible on the review screen.
4. Confirm the prompt/profile on the explicit review screen before Generate.
5. Preserve only redacted proof: profile/version, task IDs, GLB SHA-256,
   timestamps, Studio intake result and screenshots. Do not preserve credentials
   or signed URLs.

## Manual real E2E gate

The runner is intentionally inert unless all of these are set. It creates one
paid run only when its declared maximum is within the owner cap and the account
balance can cover it:

```powershell
$env:MESHY_REAL_E2E = "1"
$env:MESHY_API_KEY = "owner-provided-key"
$env:MESHY_MAX_CREDITS = "40"
$env:MESHY_REAL_E2E_PROFILE = "S1-static-prop/v1"
$env:MESHY_REAL_E2E_PROMPT = "A weathered stone lantern, isolated game asset"
$env:MESHY_REAL_E2E_GEOMETRY_TARGET = "AURORA_PROOF"
$env:MESHY_REAL_E2E_OUTPUT_PATH = ".\\test-assets\\meshy\\incoming\\s1-static-prop-1500.glb"
node tools/meshy-local-bridge/real-e2e.mjs
```

Run it separately once for approved H1, N1 and S1 prompts. Its output is a
redacted JSON proof summary with the measured triangle count; it never persists
the API key or signed URL. The GLB is not persisted by default. Supplying the
explicit output path above saves it only to the ignored `incoming` test-asset
folder; move it to `active` only after owner review, and never force-add it to
Git. For H1, the runner sets the documented explicit humanoid preflight.

## References

- [Meshy errors and browser CORS restriction](https://docs.meshy.ai/en/api/errors)
- [Text-to-3D v2](https://docs.meshy.ai/en/api/text-to-3d)
- [Rigging API](https://docs.meshy.ai/en/api/rigging)
- [Animation API and action IDs](https://docs.meshy.ai/en/api/animation-library)
- [Balance API](https://docs.meshy.ai/en/api/balance)
