# Documentation workspace rule

This rule applies to every file and subdirectory under `documentation`.

The only location where Meshy2Aurora documentation may be created, edited,
generated or staged is:

`C:\Projects\meshy2aurora\documentation`

Never use or recreate:

`C:\Users\enonw\Documents\meshy2aurora`

That path was created by Codex automation without owner authorization. It is
forbidden as a repository, workspace, staging area, scratch directory, backup,
worktree, cache, migration source, migration target or documentation target.

Before changing any document, run the repository workspace guard. A mismatch
requires HARD STOP without creating a file elsewhere. Read `PROJECT_RULES.md`
and `CANONICAL_WORKSPACE.md` for the full invariant.
