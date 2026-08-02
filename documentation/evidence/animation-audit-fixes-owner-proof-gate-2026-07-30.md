# Animation audit fixes — owner proof gate

MOD: `m2c7a58c250ba516.mod`  
Module name in Toolset: `Meshy2Aurora procedural humanoid proof`  
Area: `Meshy2Aurora M0 binary vertical-slice area`

Date: 2026-07-30  
Branch: `animation`

## Status

- `modelVisibility=not_tested`
- `proofCompleteness=missing`
- `animationPlaybackProof=not_tested`
- agent-run Toolset/NWN proof: forbidden
- new audit-fixed MOD/HAK: not materialized

This handoff identifies the exact currently installed candidate that needs the
owner's decision. It does not claim that the audit-fixed three-slot attack
route has been materialized or proven.

## Exact candidate identity

- HAK: `m2c7a58c250ba516.hak`
- MOD SHA-256:
  `97e11b3a635e96e9856bf2151aae77996df5c5dde1ff43633c40d8f87f8af09f`
- HAK SHA-256:
  `e4f46374d2811631acda5c7dcd7ef6c56973bcb10238f108adb87bae6d24a9db`
- MDL SHA-256:
  `e899b52bc138d753877f1233754a5e7ad234715410662b65ab7513839e8103bc`
- Appearance row: `15101`
- creature/model/Area/HAK resref: `m2c7a58c250ba516`
- creature position: `[10.0, 14.5, 0.0]`
- player entry: `[10.0, 10.0, 0.0]`
- packet:
  `artifacts/void-crystal-knight-attack-showcase-demo-2026-07-30`

The packet and native files were already installed and hash-verified by the
historical materialization handoff. This task did not copy, overwrite, rename
or delete any native NWN file.

## What this exact packet contains

This is the pre-audit-fix candidate:

- `vck_cryslash` is routed to `ca1slashl`;
- `vck_showcase` replaces `cpause1` for automatic idle playback;
- source geometry remains 19 704 triangles;
- detached accessory stabilization changes 341 vertex weights;
- binary MDL readback reports changing authored controllers.

The `cpause1` replacement is now rejected as a product demo strategy. The
corrected code keeps the production idle and routes one Custom attack to:

- `ca1slashl`;
- `ca1slashr`;
- `ca1stab`.

No exact MOD/HAK containing that corrected route exists yet.

## Owner verdict requested for the current candidate

Please report the exact candidate-bound result separately:

1. `modelVisibility = visible | not_visible`
2. `proofCompleteness = verified | failed | missing`
3. `animationPlaybackProof = verified | failed | not_tested`
4. whether the visible creature actually changes pose/moves

A visible model with missing motion is an animation-playback failure, not
`modelVisibility=not_visible`. Under the current hard gate, that result alone
does not authorize a new model iteration. A new iteration requires either the
gate's exact admission condition or a separate direct owner decision changing
that boundary.

## Offline implementation now complete

- Custom outputs report `LIBRARY_ONLY` until routed through Base 42.
- Review separates binary MDL readback from owner runtime proof.
- Attack demo routing preserves `cpause1` and covers all generic melee variants.
- Meshy Bridge keeps up to ten verified animation GLBs and passes them to
  Animation Studio as donor choices.
- Animation Studio inspector, key selection and Save validation race are fixed.
- Full offline gates are green.

Implementation report:
[`../audyt-poprawki-rozwiazan-animacji-2026-07-30.md`](../audyt-poprawki-rozwiazan-animacji-2026-07-30.md).
