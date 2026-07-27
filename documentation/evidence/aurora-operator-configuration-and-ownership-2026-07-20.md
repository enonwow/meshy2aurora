# Aurora Toolset: configuration boundary and owned-process authority

**Status:** `MANDATORY` for the active M0 Toolset/NWN proof work.

## Non-negotiable configuration boundary

`nwtoolset.ini`, its MRU entries, saved window placement, and every other
Toolset user setting are read-only.  They must not be changed, proposed as a
workaround, or used as a precondition for the M0 proof route.

This is an explicit requirement of the current shared
`aurora-toolset-operate` skill (suite `1.0.7`): **"Do not change
`nwtoolset.ini`, MRU, stored window position, or user settings."**  It is
also reinforced by the repository `AGENTS.md` boundary for Toolset/NWN user
configuration.

Fact record: no `nwtoolset.ini`, MRU, or other Toolset user configuration was
changed in this work.  A helper route that rejects a nonmatching MRU is a
limitation of that helper route; it is not an Aurora requirement and does not
create authority to edit configuration.

## Standing authority for the owned Toolset session

The owner has granted standing, task-scoped authority to restart the Aurora
Toolset session that this task opened, without another confirmation, when the
normal workflow requires it.

The authority is deliberately narrow:

- Before close/restart, read back exactly one `nwtoolset.exe` PID, process
  responsiveness, frame/module identity, dirty state, and every top-level
  modal.
- The recorded PID and the freshly verified frame must identify the Toolset
  session opened and owned by this task.  A different, missing, or ambiguous
  PID is read-only; it is never an inferred right to close a process.
- Use only the public canonical close/restart route and a normal close
  (`WM_CLOSE`), never force termination, global input, or a second Toolset
  process.
- If the verified active module is dirty, use the standard saved-close flow.
  The known, module-owned save confirmation may be answered through its
  freshly resolved localized affirmative button only after its ownership is
  verified.  An unknown or destructive prompt remains read-only.
- After a close, verify the target process exited.  After a restart, record
  the new PID and re-establish the single-session/module-owner invariants
  before any mutation.

This authority is solely about the verified process lifecycle.  It does not
weaken the configuration boundary above, authorize opening an arbitrary module,
or turn an MRU workaround into a valid route.

## Evidence discipline

Every restart/close record must include the exact PID before and after,
module/frame identity, modal state, chosen public route, and save/readback
outcome.  A successful build or a process restart is not visual proof: model
visibility still requires a fresh `TScrollBox`/viewport capture bound to the
saved module hash, ordered HAK hash, area, fixture, and appearance row.  NWN
proof remains a separate runtime capture.

## Relation to the current M0 packet

The active M0 record is
[`m0-v10-binary-runtime-preflight-2026-07-20.md`](m0-v10-binary-runtime-preflight-2026-07-20.md).
Its section *Correction: configuration boundary and owned-process restart
authority* carries the same facts for the prior v10 route.  This document is
the concise durable rule for all subsequent M0 Toolset work.
