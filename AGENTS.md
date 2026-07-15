# Meshy2Aurora agent rules

## Canonical workspace — HARD STOP

The only canonical repository and writable project workspace is:

`C:\Projects\meshy2aurora`

The following path is forbidden for every project operation:

`C:\Users\enonw\Documents\meshy2aurora`

Never create, edit, stage, copy, migrate, test, build, cache, or store temporary
project work there. It is not a fallback workspace, scratch directory, staging
area, clone, mirror, or migration source. The owner never authorized that path.

Before any implementation or documentation write, run:

`powershell -NoProfile -ExecutionPolicy Bypass -File assert-canonical-workspace.ps1`

The check must resolve the repository root.
If it is not exactly `C:\Projects\meshy2aurora`, stop. Do not work around the
problem by writing elsewhere and do not repeatedly request permissions for
out-of-workspace writes. Reopen or resume the task with the canonical repository
as its workspace root.

All durable project documentation belongs in
`C:\Projects\meshy2aurora\documentation`. Read
`documentation/PROJECT_RULES.md` and
`documentation/CANONICAL_WORKSPACE.md` before changing the project.

Aurora, NWN, Toolset, game installations, user configuration and
`nwtoolset.ini` remain read-only unless the owner gives a separate, explicit
instruction that changes that boundary.
