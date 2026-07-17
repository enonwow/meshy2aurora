# Meshy H1 Codex animation native NWN runtime proof — attempt 4

Status: `FAILED`.

Window: `2026-07-17T15:24:19.7733414+02:00` to
`2026-07-17T15:29:44+02:00` (five-minute limit). Complete native packet:
`false`.

This was the first timed Toolset adapter loop using corrected top-level window
enumeration. It reached a real Toolset engine error; it did not reach NWN.

## Corrected Toolset activation

The live Toolset PID was `50340`, version `89.8193.37-17`, with a real
`TfrmFrame` on `\\.\DISPLAY1` (`primary=false`, `1920x1032`) and the visible
`TdlgWelcome` modal. The adapter read its actual controls rather than relying
on `Process.MainWindowHandle`:

- `Open an existing Module` was selected;
- the list had 77 entries;
- exactly one was `m2a_codex_aproof`, at index `51`;
- the dialog's own `OK` was used after list selection readback.

This corrects the previous false PID/window blocker. The prior enumerator wrote
its callback results to PowerShell's callback-local scope and therefore lost
the visible `TfrmFrame` and `TdlgWelcome` records.

## Native Toolset error

Toolset then displayed this modal:

```text
Cannot open file "C:\Users\enonw\Documents\Neverwinter Nights\hak\.hak".
Nie można odnaleźć określonego pliku.
```

Raw capture: `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-4/toolset-empty-hak-modal.png`.

The canonical files still have their expected hashes, and own materialization
readback reports `hakResref=m2a_codex_aproof`. The native Toolset instead
resolved an empty HAK resref. Therefore the current GFF/ERF writer contract is
not native-validated: its self-readback is insufficient and must be compared
against a real working `module.ifo` before another Toolset/NWN attempt.

No direct `nwmain` route, second HAK/MOD, global input, INI/MRU mutation, NWN
capture, or animation claim occurred. The next timed loop must first repair
and independently validate the binary module's HAK reference; only then can it
continue through Build → Test Module, the NWN log gate, and `cpause1` capture.

## Mandatory aurora-web post-mortem (read-only)

The precise stopping point was Toolset module-open HAK resolution. The newest
checked local precedent is
`C:\Projects\aurora-web\backend\src\modules\runtime-settings\application\runtime-settings.application.service.ts`,
`extractModuleIfoHakList()` (lines 2617–2660). It reads `Mod_HakList` as a
GFF list and decodes each child `Mod_Hak` with its string reader; a non-empty
normalized name is the entry condition and expected readback. The complementary
native launch precedent is
`backend/docs/aurora-reverse/aurora-toolset-npc-stage3-standard.md`:
after current menu identity/state readback, `Build` position 2 / command 121
is `miTestModule`.

What was wrong here: the generated `Mod_HakList[0]` used a ResRef/type-11
payload and struct id 0, plus a legacy root `Mod_Hak`; the working local MOD
contract uses a string-valued HAK entry. That difference explains the observed
`hak\\.hak`, so repeating the old open sequence would be prohibited.

The one change for the next fresh attempt is to regenerate the **same**
`m2a_codex_aproof.mod` with `Mod_HakList[0]` as struct id 8 containing exactly
one CExoString/type-10 `Mod_Hak=m2a_codex_aproof`, with no root `Mod_Hak`, and
to validate that binary contract before starting Toolset/NWN. The prior
Toolset/NWN processes were then closed; no live session was reused.
