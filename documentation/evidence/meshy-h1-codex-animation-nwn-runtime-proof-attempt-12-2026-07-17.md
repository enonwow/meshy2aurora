# Attempt 12 — first Aurora viewport audit (failed)

Date: 2026-07-17

## What this attempt proved

The v5 direct writer produced a module that both native consumers can parse:

- Toolset opened `m2a_codex_aproof.mod` and its tree showed
  `Codex H1 animation proof area` plus
  `Codex Meshy H1 animation proof creature` under `Creatures`.
- NWN logged `Loading Module: m2a_codex_aproof` and entered the area rather
  than remaining on `Unloading Area...`.

Artifacts:

- `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-12/toolset-v5-area-state.png`
- `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-12/toolset-v5-area-after-settle.png`
- `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-12/nwn-v5-full-ifo-are.png`

## Failed gate

Both the Toolset viewport and NWN scene were visually empty. Aurora's tree
readback is therefore structural evidence only; it is not a Toolset visual
proof and it does not prove the model or animation loaded.

The failure exposed a process error in earlier attempts: the generated area
was sent to NWN before a Toolset viewport gate had been performed. The
runbook now requires that gate before `Build -> Test Module`.

## Aurora-first finding

`documentation/m6-typed-resource-manifests-codex.md` marks the schema of
ARE43/tile10 as ready, but records the generated proof preset
`Tileset + Tile_ID + minimal environment` as `NOT_READY`. Earlier attempts
invented a `ttr01` / tile 139 environment without closing that source-backed
profile. That is the next narrow implementation gap; it must be resolved from
Aurora/Toolset readback and then recreated by the own writer, not copied as a
retail payload.

The engine also logs `Unable to load faction table` and invalid class IDs. It
is a separate creature runtime gap; it must not be conflated with the blank
area viewport.

## Reset

After a full five-minute attempt without a visible H1, Toolset and NWN were
closed at `17:28:24`. The next attempt begins from zero and may not dispatch
Test Module until the new Toolset viewport gate passes.
