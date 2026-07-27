# M0 Toolset recovery incident — 2026-07-19

Status: `SUPERSEDED AS OWNER-WAIT BLOCKER; VIEWPORT ADAPTER GAP REMAINS`.

## Scope

This record covers the live Toolset lane for the recovered Meshy M0 proof only.
It does not claim a Toolset viewport result or an NWN runtime result.

## Confirmed state

- Installed proof target: `m2a_m0_proof.mod`, with the module-frame title
  `BioWare Aurora Neverwinter Nights Toolset v89.8193.37-17 - m2a_m0_proof.mod`.
- The one responding Toolset process is PID `38596` on the non-primary
  `\\\\.\\DISPLAY1`; no `nwmain.exe` process was observed.
- Before any Area action, Toolset presented a visible top-level modal titled
  `Confirmation` with this text:

  ```text
  The toolset did not close properly last time it was used.
  A module was found, would you like to try to recover it?
  Pressing 'No' will destroy the backup.
  ```

- The only available choices are `Tak`, `Nie`, and `Anuluj`.

## Result of the original attempt

The Toolset viewport, `Build -> Test Module`, and NWN lanes are all `missing`.
No modal button was invoked. The current live session was preserved.

## Corrected interpretation

The modal correctly pauses the current live mutation. It does not block the
whole goal and does not require the owner to answer by default. The exact text
states that `No` destroys the backup, so recovery policy 1.0.1 selects the
non-destructive `Tak` action through the freshly resolved modal control.

## Subsequent recovery and module state

The owner explicitly directed `Nie` for this disposable M0 backup and then
directed the operator to open the correct module. The action was targeted to
the freshly resolved `&Nie` button. The only responding Toolset process then
loaded `m2a_m0_proof.mod`; its frame title was independently read back as
`BioWare Aurora Neverwinter Nights Toolset v89.8193.37-17 - m2a_m0_proof.mod`.
No `nwmain.exe` process was started.

The exact Area tree entry `Meshy2Aurora M0 static runtime proof area` was also
read back under `Areas`. This is module/Area discovery only: no visible
`TfrmViewerArea` or viewport capture exists yet.

## Current blocker: missing project-owned adapter

### Superseded policy correction

This blocker classification is superseded. The shared Aurora operating skills,
their canonical runner, and verified native atoms are central operator tooling;
they are not a Meshy2Aurora product dependency covered by the `reference-only`
boundary. No project-owned adapter is required or permitted as a replacement.
The next live lane must use the shared standard from its last verified state.

The approved project adapter `tools/m2a-aurora-proof.mjs` exposes live actions
only for `open` and `complete-open-dialog`; its own CLI contract explicitly
states that viewport and Test Module actions are not live adapters. A search of
the canonical project found no approved project-owned counterpart for the
reference-only Aurora actions `view-aurora-toolset-area-tree-item-no-global-input`,
viewport capture, or Build/Test Module.

Project rule 2.2 forbids executing those `aurora-web` scripts and forbids
improvising a replacement live UI workflow. Therefore:

- Toolset module load: `verified`.
- Exact Area tree identity: `verified`.
- Visible `TfrmViewerArea` and Toolset viewport: `missing`.
- Build/Test Module: `missing`.
- NWN runtime proof: `missing`.

## Resume condition

The owner must either identify the approved Meshy2Aurora adapter that performs
the Area viewer, viewport capture, and Test Module stages, or explicitly
authorize creation and qualification of narrow project-owned adapters for
those stages. Until then, launching NWN would violate the mandatory viewport
gate.

This record does not authorize closing Toolset, modifying `nwtoolset.ini`, or
replacing the installed M0 HAK/MOD.
