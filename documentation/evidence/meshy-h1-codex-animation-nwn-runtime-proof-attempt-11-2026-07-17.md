# Attempt 11 — native NWN runtime proof (failed after 5-minute reset rule)

Date: 2026-07-17

## Exact run

1. Installed the v4 canonical `m2a_codex_aproof.mod` (SHA-256
   `f3a68c7edee1875b5952700b580ff1663caa713e0bf3a6d72f67c934911a3770`) into
   the same authorized NWN module path.  The HAK was unchanged.
2. Started Toolset without module arguments.  The stale-recovery modal created
   by the preceding forced failure was explicitly rejected; it could not
   contribute state to this attempt.
3. Selected `m2a_codex_aproof` from the live Welcome list (`index=51` of 77)
   on `\\.\DISPLAY1`, then verified the frame title.
4. Read the live menu: `&Build`, position 4, contains enabled command ID `121`
   at position 2.  Dispatched that exact Test Module command without global
   mouse or keyboard input.
5. NWN PID `27776` logged `Loading Module: m2a_codex_aproof` at `17:14:07`.
   It remained responding through `17:19:10`.

## Evidence and result

- Native image:
  `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-11/nwn-v4-after-area-fix.png`
- It visibly remains at `Unloading Area...`; no H1 instance is present.
- No animation MP4 or before/after pair exists, because no valid scene was
  reached.
- The engine log retains `Empty field label while reading ... MODULE.ifo` and
  repeated `Invalid Class` warnings.  These are evidence of malformed or
  incomplete generated resources, not harmless warnings.

## Five-minute Aurora-first audit

The failure was re-audited against the local, read-only Aurora contract before
the mandated clean reset.

1. `documentation/m6-typed-resource-manifests-codex.md` already marks the
   **IFO55** schema and **ARE43/tile10** schema as `READY` from decompilation
   and read-only packet evidence.
2. v4 generated only the entry/HAK subset of IFO55 (10 fields) and only a
   small subset of ARE43.  This is a direct contract gap.  The empty-field
   engine diagnostic is consistent with that mismatch.
3. `aurora-web` reference-only audit independently lists all 43 ARE fields as
   required in `backend/docs/aurora-reverse/are-git-validation-spec-r63.md`.
4. The v4 tile cardinality repair was correct and retained, but it cannot make
   a sparse IFO/ARE runtime-valid.

## What changes in the next attempt

- Implement and own-readback the complete ordered IFO55 and ARE43 manifests;
  retain the confirmed single tile as struct ID `1` with the exact `tile10`
  types.
- Restore only types already frozen by the local Aurora manifest.  Do not
  speculate about synthetic gameplay class values; the existing `Invalid
  Class` diagnostics stay explicitly open until a source-backed class profile
  is established.
- Regenerate the same single module, then restart Toolset and NWN from zero.

The Toolset and NWN processes were closed at `17:19:21` under the agreed
five-minute reset rule.  This file records a failed individual attempt, not a
termination of the proof loop.
