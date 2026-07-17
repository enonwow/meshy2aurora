# Meshy H1 Codex animation native NWN runtime proof — attempt 7

Status: `FAILED` (attempt only; native-proof loop remains active).

Window: `2026-07-17T16:24:17.4402076+02:00` to
`2026-07-17T16:27:41.5296808+02:00`. Complete native packet: `false`.

## Corrected Open Module adapter

This fresh Toolset PID `30564` used the canonical installed pair without any
copy or overwrite. The current native dialog route followed the checked
Aurora-web adapter: it set the visible `TdlgModuleSelect` `TEdit` to
`m2a_codex_aproof` with `WM_SETTEXT`, posted `BM_CLICK` to its visible `Open`
button, then polled the `TfrmFrame` title. No global input, INI/MRU mutation,
second MOD/HAK or NWN launch occurred.

The Toolset did extract the archive into its own user-directory `temp0` flow:
`module.ifo`, `m2a_caproof_area.are/.gic/.git`, and `m2a_caproof_h1.utc` all
received the same fresh timestamp. Thus the selector did select the canonical
archive. Nevertheless, the frame never gained `m2a_codex_aproof.mod`, the
Toolset tree contained zero items, and no area viewer existed. Supporting
capture: `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-7/toolset-extracted-no-context.png`.

## Read-only binary and aurora-web audit

The extracted `module.ifo` has both a root `Mod_Hak` and
`Mod_HakList[0].Mod_Hak` as GFF type `11` (`CResRef`). Its extracted GIC root
is empty; its GIT root has only sparse `Creature List`. The checked
Aurora-web reader
`backend/src/modules/runtime-settings/application/runtime-settings.application.service.ts`
`extractModuleIfoHakList()` reads `Mod_HakList` entries through its string
decoder, and the checked Open adapter
`backend/scripts/open-aurora-toolset-module-from-dialog-no-global-input.mjs`
requires exactly the title/area readback that is absent here.

This is therefore not an Open-dialog failure: Toolset accepts/extracts the
archive but cannot construct its module context from this serialized schema.
The next change is material: replace only the existing
`m2a_codex_aproof.mod` with the already-generated schema-correct canonical
MOD (same resref; the HAK is unchanged), then run a fresh five-minute attempt.
The Toolset was closed after this audit; NWN was not started.
