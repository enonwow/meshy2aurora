# `m2aimod35d.mod` — V14 ready for owner proof

Date: 2026-08-27

## Exact handoff

- test MOD: `m2aimod35d.mod`
- module name: `Meshy2Aurora Hextech Ranged Demo V14`
- Area: `Hextech Ranged Demo V14`
- Area resref: `m2aia35d`
- MOD SHA-256: `ce81eb3a2baf8e84bc851ec820f75e0ab0200eb634576c8ff21f6e7a59e1e52e`
- HAK: `m2aihak35d.hak`
- HAK SHA-256: `473a9b5238a290c041192037ec23c3c032a9f18591a512c8eff8cfb8c0098082`

The exact generated files are frozen under the registered canonical worktree:

`C:\Projects\meshy2aurora\.worktrees\items-agent-remediation-final\proof-output\tlc-hextech-ranged-demo-v14-20260827\generated`

They were installed to the native NWN user directories only after both target
paths were confirmed absent. Source and installed SHA-256 hashes are identical:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aimod35d.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aihak35d.hak`

The earlier `m2aimod35c.mod` / `m2aihak35c.hak` V13 files were not modified.

## Demo contents confirmed offline

- ground weapon: `m2aitm35d` at `[10.8, 11.6, 0]`;
- ground ammunition: `m2ahxammo` at `[9.2, 11.6, 0]`;
- target: `m2arngtarget` at `[10, 13.3, 0]`;
- target is Plot with `1000 / 1000` HP and the fixture's immobile target contract;
- module semantic readback: `PASS`;
- item package: `OFFLINE_ITEM_PACKAGE_PASSED`;
- runtime carrier override: `pfh0.mdl`, SHA-256
  `42c5f0fadb3068c22c47851e40ed518b2dc627c0e90046b29b6b79b90253eb8d`;
- expected runtime states: `pause1`, `xbowrdy`, `xbowshot`, animation root
  `rootdummy`;
- Studio tests: `358 passed`, `1 skipped`, `0 failed`;
- TypeScript typecheck: `PASSED`.

## Honest completion boundary

The agent did not start or control Aurora Toolset or NWN. V14 therefore has
`modelVisibility=not_tested` and `proofCompleteness=missing` until the owner
opens this exact module and reports the visual/runtime result. Offline success
does not assert that the character animation, weapon hold, projectile or ammo
consumption is visually correct in NWN.
