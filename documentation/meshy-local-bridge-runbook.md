# Meshy Local Bridge — runbook

Status: implementation guide for the optional local companion introduced with Meshy Lab.

## Purpose and boundary

The Bridge is a local process, not a product backend. It binds only to
`127.0.0.1`, owns `MESHY_API_KEY`, and exposes the narrow `/v1/*` contract used
by Studio. React, the Web Worker, WASM, browser storage, proof manifests and
repository files never receive the API key or Meshy signed URLs.

The implementation is at `tools/meshy-local-bridge/index.mjs`; its security
and protocol tests are `tools/meshy-local-bridge/bridge.test.mjs`.

The current fail-closed contract is protocol `2` with build capability
`M2A_MESHY_BRIDGE_2026_07_31_V2`. Studio verifies the complete capability set,
including named multi-animation merge and run recovery, before it permits a
paid operation. A running protocol-1 process must be restarted; source files
changing on disk do not hot-reload the Node Bridge.

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

In Docker Compose the Bridge listens on `0.0.0.0` only inside its isolated
container, because a container loopback listener cannot receive published-port
traffic. This requires `MESHY_BRIDGE_ALLOW_CONTAINER_BIND=1`; the Compose port
mapping remains fixed to `127.0.0.1:43119`, and the exact-origin/session checks
remain enforced by the Bridge.

### Restart from Studio

The Docker Compose Bridge advertises a `Restart local Bridge` control on the
Studio pairing screen. It is available only when the Bridge is started with
`MESHY_BRIDGE_RESTARTABLE=1` under Compose's `restart: unless-stopped`
supervisor. The exact-origin `POST /v1/bridge/restart` route causes only that
Bridge process to exit; Compose recreates it and emits a fresh one-use pairing
code in the local container terminal/log. The UI never receives or displays
that code. Restarting clears local Bridge sessions and does not cancel a Meshy
task that is already running. READY runs are recovered from the local state
journal described below.

### Durable local run recovery

Set `MESHY_BRIDGE_STATE_DIRECTORY` to a Bridge-owned local directory. Docker
Compose uses `/state` backed by the named `meshy_bridge_state` volume. Before a
run becomes `READY`, the Bridge writes its redacted metadata, merged artifact
and raw action GLBs separately. It never writes the API key or signed Meshy
URLs. On restart it loads only complete entries whose recorded byte length and
SHA-256 still match every stored payload; corrupt or incomplete entries remain
unavailable.

The authenticated `GET /v1/runs` endpoint returns the safe run inventory.
Studio remembers the exact active run ID in session storage and, after a
reload, recovers that run or the newest available READY run. Built MOD/HAK and
related downloads are stored separately in browser IndexedDB and are exposed
again only after exact length and SHA-256 verification.

### Same-origin automatic pairing in Docker Compose

Compose also sets `MESHY_BRIDGE_AUTOMATIC_PAIRING=1`. On the exact configured
Studio origin only, the explicit **Connect local bridge** action can mint a
short-lived local session without sending a pairing code or `MESHY_API_KEY` to
the browser. This is intended for the owner-operated `localhost` Compose
Studio. A Bridge started directly without this explicit capability retains the
manual one-use pairing-code flow.

For a static deployment, `MESHY_BRIDGE_ALLOWED_ORIGIN` must be the exact
deployed Studio origin. Wildcards and LAN binding are intentionally unsupported.
`VITE_MESHY_LAB=1` must be present when Vite starts/builds; the optional Lab is
hidden by default when the feature is not explicitly enabled.

## Supported proof profiles

| Profile | Pipeline |
| --- | --- |
| H1 humanoid animated | Text/Image-to-3D -> rigging -> od 1 do 10 akcji Animation API -> jeden scalony GLB |
| N1 quadruped | Text-to-3D preview -> refine -> GLB |
| S1 static prop | Text-to-3D preview -> refine -> GLB |

Only H1 can use rigging/animation. Studio sends `animationActions`: from one
to ten unique pairs `{actionId, clipName}`. Action IDs must be unique, clip
names must be unique members of the exact 42-state NWN namespace, and exactly
one mapping must target `cpause1`. The Bridge creates every action on the same
exact rig, downloads and validates every GLB, calculates its SHA-256 and
exposes the canonical artifact only after the whole run reaches `READY`. The
maximum credit estimate is calculated from the actual number of requested
actions, not from a fixed single-animation assumption. H1 confirmation also
reserves the worst-case five-credit automatic-remesh recovery before any paid
task is created. For one action the maximum is therefore 43 credits
(`30 generation + 5 recovery reserve + 5 rig + 3 animation`); unused recovery
reserve is not spent.

The Bridge invokes `merge-animation-glbs.mjs` automatically. It combines all
same-rig action GLBs into one canonical `/artifact`, gives each clip its
requested NWN name and records `artifactKind=MERGED_ANIMATION_GLTF` plus every
raw action identity in provenance. Studio imports that one merged file into
Source; the user must not manually choose the first action GLB. The merge fails
closed when node, skin, POSITION, JOINTS, WEIGHTS or index topology differs. A
second inspection of the final merged GLB must return the exact requested clip
count, order and names. Source then requires the imported bytes to match the
run-bound SHA-256 and animation provenance before Build can start. A signed
download failure is a transport-lane failure, not permission to create
another model or rig:
`resume-animation-lineage.mjs` first binds to the exact recorded model and rig
task IDs, reuses already completed animation tasks, and creates only missing
actions within the remaining owner-approved credit cap.

### Geometry targets

`AURORA_PROOF` is the default Meshy Lab choice. The shared whole render-model
limit for creature, placeable, tile and other render routes is `300,000`
triangles. Meshy `target_polycount` is a
target, not an exact guarantee, so intake always measures the downloaded GLB.
When a result is above the product limit,
`tools/meshy-skinned-triangle-budget.mjs` may reduce the same source lineage
below the global limit while preserving skin, joint weights and animation
clips. An over-limit payload must not be silently admitted.

Binary MDL still allows at most `65,535` index entries (`21,845` triangles) in
one mesh stream. The application partitions larger render meshes
deterministically; this is not decimation and does not remove geometry.

## Operational limits

- A browser session expires after 15 minutes. The direct-process pairing code
  is one-use; restart the Bridge to obtain a fresh code before pairing again.
  The explicit Docker Compose automatic-pairing capability mints a new
  short-lived session for the exact configured Studio origin instead.
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
$env:MESHY_REAL_E2E_OUTPUT_PATH = ".\\sample-3d\\<asset-id>\\source.glb"
node tools/meshy-local-bridge/real-e2e.mjs
```

For an exact single PNG/JPEG Image-to-3D input, set
`MESHY_REAL_E2E_IMAGE_PATH` instead of `MESHY_REAL_E2E_PROMPT`. The runner
hashes and forwards those exact image bytes as one data URI. It disables image
enhancement, baked-light removal and Meshy auto-size so that the source image is
not preprocessed and downstream product authoring remains responsible for game
scale:

```powershell
$env:MESHY_REAL_E2E_IMAGE_PATH = "C:\path\to\reference.png"
$env:MESHY_REAL_E2E_TARGET_POLYCOUNT = "100000"
node tools/meshy-local-bridge/real-e2e.mjs
```

Run it separately once for approved H1, N1 and S1 prompts. Its output is a
redacted JSON proof summary with the measured triangle count; it never persists
the API key or signed URL. The GLB is not persisted by default. Any persisted
source belongs only under `sample-3d/<asset-id>/` and requires a complete
`manifest.yaml`; `test-assets/meshy` is forbidden. For H1, the runner sets the
documented explicit humanoid preflight.

For an owner-approved image-to-3D H1 run with several animations, use
`real-image-multi-animation-e2e.mjs`. It requires an existing canonical asset
directory, refuses to overwrite any payload or provenance file, enforces the
owner credit ceiling before the paid request, and accepts one to ten
`{actionId,fileStem,clipName}` entries through
`MESHY_REAL_E2E_ANIMATIONS`. The runner downloads every raw action for
provenance, then writes the automatically merged canonical `source.glb` in the
same declared `sample-3d/<asset-id>/` directory. Do not manually substitute a
single action GLB or create a second source library.

The runner enables Meshy moderation by default. If Meshy rejects an
owner-approved fictional horror concept before creating a model with
`TASK_REJECTED`, the same exact source may be retried with
`MESHY_REAL_E2E_MODERATION=0`. The override accepts only `0` or `1` and is
recorded in durable provenance. It is not a geometry/model iteration and must
not be used to bypass a rejection involving real-person abuse or otherwise
disallowed source material.

## References

- [Meshy errors and browser CORS restriction](https://docs.meshy.ai/en/api/errors)
- [Text-to-3D v2](https://docs.meshy.ai/en/api/text-to-3d)
- [Rigging API](https://docs.meshy.ai/en/api/rigging)
- [Animation API and action IDs](https://docs.meshy.ai/en/api/animation-library)
- [Balance API](https://docs.meshy.ai/en/api/balance)
