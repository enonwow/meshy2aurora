# Attempt 10 — native NWN runtime proof (failed)

Date: 2026-07-17

## Scope

The canonical single test module was opened in Aurora Toolset and started via
the verified **Build -> Test Module** route.  This is one fresh Toolset/NWN
attempt; it is not a continuation of attempt 9.

## Native evidence captured

- Toolset loaded `m2a_codex_aproof.mod` and Build/Test Module started NWN.
- NWN engine log contains `Loading Module: m2a_codex_aproof` at `17:03:15`.
- NWN client log contains the same module-load record.
- Captures:
  - `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-10/nwn-after-module-load.png`
  - `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-10/nwn-stable-check.png`
- The image remained on `Unloading Area...`; H1 was not visible.
- NWN produced `C:\Users\enonw\Documents\Neverwinter Nights\crashreport\nwmain-crash-1784300597.nwcrash.txt`
  at `17:03:17`.  Therefore the saved loading images are not a usable runtime
  proof and this attempt is **FAILED**, not merely incomplete.

## Aurora-first audit performed after the failure

Read-only audit of `C:\Projects\aurora-web` found an exact generated-ARE
contract violation:

- `backend/docs/aurora-reverse/are-git-validation-spec-r63.md` defines
  `len(ARE.Tile_List) == ARE.Width * ARE.Height` as
  `ARE_TILE_CARDINALITY_MISMATCH`, non-degradable.
- `backend/src/modules/areas/adapters/outbound/blob/aurora-gff-area-editing-writer.ts`
  writes each tile with `Tile_ID`, `Tile_Orientation`, and `Tile_Height`.
- The generated proof ARE had `Width=1`, `Height=1`, but an empty
  `Tile_List`.

This is a confirmed defect in the proof-module generator, and it is sufficient
to explain why an area cannot become a valid playable entry area.  The log also
contains pre-existing `Invalid Class` warnings and an empty `MODULE.ifo` field
label warning.  Those are recorded as separate follow-up diagnostics; they are
not being guessed at as the cause of this failed area load.

## Change required before a fresh attempt

1. Add exactly one valid `Tile_List` record to the generated `1 x 1` ARE.
2. Emit the confirmed tile fields with correct types.
3. Add a readback test asserting tile cardinality and field types.
4. Regenerate the same canonical module, run the local test, then repeat the
   entire Toolset -> Test Module -> NWN sequence from zero.

No native H1 visibility, animation screenshot, or MP4 is claimed by this
attempt.
