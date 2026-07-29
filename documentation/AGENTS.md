# Documentation workspace rule

This rule applies to every file and subdirectory under `documentation`.

Meshy2Aurora documentation may be created, edited, generated or staged only in
the active approved worktree's `documentation` directory. The primary location
is:

`C:\Projects\meshy2aurora\documentation`

Registered linked worktrees are also approved when their resolved roots remain
inside `C:\Projects\meshy2aurora` and their Git common directory is exactly
`C:\Projects\meshy2aurora\.git`. No documentation worktree outside the
canonical root and no path using different Git metadata is authorized.

Never use or recreate:

`C:\Users\enonw\Documents\meshy2aurora`

That path was created by Codex automation without owner authorization. It is
forbidden as a repository, workspace, staging area, scratch directory, backup,
worktree, cache, migration source, migration target or documentation target.

Before changing any document, run the repository workspace guard. A mismatch
requires HARD STOP without creating a file elsewhere. Read `PROJECT_RULES.md`
and `CANONICAL_WORKSPACE.md` for the full invariant.
