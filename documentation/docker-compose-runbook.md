# Docker Compose — Studio local development

Status: `ACTIVE / LOCAL DEVELOPMENT AND QUALITY GATES`

## Scope

Compose starts only the local Studio development server. The browser still
performs the conversion locally through the existing Worker/WASM path; no
backend, source upload store or Aurora/NWN container is introduced.

The configuration never mounts or writes to Aurora, NWN EE, Toolset, CEP,
`nwtoolset.ini`, game installations or user game folders. Native proof remains
on the authorized Windows host path.

## Required host software

- Docker Desktop with Docker Compose v2;
- the canonical workspace at `C:\Projects\meshy2aurora`.

The image pins Rust `1.96.1`, Node `24.15.0`, npm `11.12.1`, the WASM target
and `wasm-pack 0.15.0`. It installs the Studio lockfile with `npm ci`.

## Operator commands

Run from the repository root:

```powershell
# First start (builds the development image), then open http://localhost:5174.
.\tools\docker-studio.ps1 Start

# Stop or restart the running Studio without removing source files or caches.
.\tools\docker-studio.ps1 Stop
.\tools\docker-studio.ps1 Restart

# Inspect state, recent logs, or rebuild the development image.
.\tools\docker-studio.ps1 Status
.\tools\docker-studio.ps1 Logs
.\tools\docker-studio.ps1 Build

# After a Rust/WASM source change, rebuild the mounted web-WASM package.
.\tools\docker-studio.ps1 RebuildWasm
```

`studio` has `restart: unless-stopped`; editing repository files is reflected
through the bind mount. Polling is enabled by default for dependable file
watching on Docker Desktop. Set `M2A_DOCKER_POLLING=false` only if native file
events are reliable on the host. Set `M2A_STUDIO_PORT` to choose a different
host port. The helper reports an occupied port before starting; for example:

```powershell
$env:M2A_STUDIO_PORT = '5180'
.\tools\docker-studio.ps1 Start
```

The port is bound only to `127.0.0.1`, never to the LAN.

The first start builds the web-WASM package. Subsequent restarts reuse it and
are fast; changes under `crates/m2a-*` require `RebuildWasm`, while TypeScript
and CSS changes use Vite's normal hot reload.

Meshy Lab is enabled in the local Compose profile by default so its paired
loopback workflow and recovery screen can be exercised. It still needs a
separately started local Bridge and an owner-provided API key before any Meshy
request is possible. Set `M2A_MESHY_LAB=0` before `Start` to hide it.

To view existing Meshy tasks, set the key only in the current PowerShell
session, then start the optional Bridge profile and retrieve its one-time
pairing code from the local logs:

```powershell
$env:MESHY_API_KEY = "owner-provided-key"
.\tools\docker-studio.ps1 StartMeshyBridge
.\tools\docker-studio.ps1 MeshyBridgeLogs
```

`StartMeshyBridge` refuses to start when the current terminal has no
`MESHY_API_KEY`; this prevents a reachable-but-unusable Bridge.

Enter that code in Meshy Lab. The **Browse existing Meshy work** control then
lists Text-to-3D tasks and may recover an already-finished refined GLB. This
does not create a paid task. The Bridge is mapped only to `127.0.0.1:43119`,
and the API key remains only in that container's process environment. It binds
to `0.0.0.0` only inside the container so Docker can forward that loopback-only
host port; direct host execution still binds only to `127.0.0.1`.

To reset only generated dependency/build caches — never project files — stop
the containers and remove Compose volumes explicitly:

```powershell
docker compose down --volumes
.\tools\docker-studio.ps1 Start
```

## Quality profile

The `quality` profile rebuilds the existing Docker quality target. Its build
performs Rust formatting, Clippy, workspace tests, WASM build and Node/WASM
adapter tests without access to the network at container runtime.

```powershell
.\tools\docker-studio.ps1 Verify
```

This is intentionally a strict gate. It currently reports any failing project
test or lint rather than hiding it behind Compose. It is not a Toolset/NWN
proof and does not authorize changes outside the canonical repository.
