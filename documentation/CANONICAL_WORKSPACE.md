# Canonical workspace invariant

<!-- WORKSPACE-INVARIANT: Canonical repo C:\Projects\meshy2aurora. Never use C:\Users\enonw\Documents\meshy2aurora for project work. -->

Status: `MANDATORY / HARD STOP`

## One project location

The only canonical repository, implementation target, documentation root and
place for project-local temporary work is:

`C:\Projects\meshy2aurora`

The following path is explicitly forbidden:

`C:\Users\enonw\Documents\meshy2aurora`

The forbidden path was created by Codex automation. The owner did not request,
select or authorize it. It must never be treated as a checkout, staging area,
scratch directory, cache, mirror, migration source or fallback when the
canonical repository is outside the current sandbox.

## Required behavior

Before the first write, every root agent, subagent and local automation must:

1. resolve the repository root;
2. confirm it is exactly `C:\Projects\meshy2aurora`;
3. stop without writing when the check fails;
4. resume only in a task whose workspace root is the canonical repository.

It is forbidden to prepare changes in another similarly named folder and copy
them later. A missing permission to the canonical repository is a workspace
configuration problem, not permission to create a second worktree.

Read-only diagnosis may identify the mismatch, but it must not create project
files, Git objects, tests, build outputs or notes outside the canonical repo.

## Documentation-wide enforcement

`documentation/AGENTS.md` applies recursively to every file and subdirectory
under `documentation`. This enforces the invariant for all current and future
documents without rewriting historical snapshots or `*-cloud.md` files.

The visible source of truth is this document together with `PROJECT_RULES.md`,
root `AGENTS.md`, `documentation/AGENTS.md` and `orchestrator-state.yaml`.

## Consolidation record

On 2026-07-15 the non-canonical folder was audited before removal. Its
`s1-staging` payload was an obsolete predecessor of code already present in the
canonical repository; the canonical implementation was newer and remained the
sole source of truth. No stale file was allowed to overwrite canonical code.
The superseding history includes commits `b026ba7`, `5d6c952` and `f8801a2`.
All 16 staging files were classified `safe-to-delete`; none required migration.
All contents of the forbidden root were removed. The misconfigured active task
held an operating-system handle to the now-empty directory entry; that entry
must be deleted as soon as the task releases it and must never be recreated.
