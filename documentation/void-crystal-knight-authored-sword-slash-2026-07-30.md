# Void Crystal Knight — authored sword slash

Date: 2026-07-30  
Branch/worktree: `animation` /
`C:\Projects\meshy2aurora\.worktrees\animation`

## Result

Meshy2Aurora Animation Studio can now create a real humanoid sword attack from
the application:

`+ New animation` → `Void crystal cleave`

The preset is `HUMANOID_SWORD_SLASH`. It creates a one-second authored motion
clip with six deliberate phases:

1. neutral pose;
2. anticipation;
3. full wind-up;
4. impact;
5. follow-through;
6. return to the exact bind pose.

Unlike the rejected `ROOT_TRANSLATION_PULSE` demo, this clip does not pretend
that root translation is a sword attack. It authors rotation curves on twelve
humanoid bones:

- `Hips`;
- `Spine02`;
- `Spine01`;
- `Spine`;
- `RightShoulder`;
- `RightArm`;
- `RightForeArm`;
- `RightHand`;
- `LeftShoulder`;
- `LeftArm`;
- `LeftForeArm`;
- `Head`.

The right-hand chain drives the sword swing. Torso rotation supplies wind-up
and follow-through, while the left arm counterbalances the strike. Feet and
root translation remain stable, so the animation is in-place and does not
slide the creature across the Area.

## Void Crystal Knight exact replay

Source:
`sample-3d/void-crystal-knight-h1-v1/source.glb`

Source SHA-256:
`d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`

The exact ignored regression builds this local source through Animation Studio
V5 with:

- authored clip ID: `vck-authored-sword-slash-v1`;
- authored output: `vck_cryslash`;
- Custom ID: `vck-custom-sword-slash-v1`;
- playback: `ONE_SHOT`;
- native attack route: `ca1slashl`, `ca1slashr`, `ca1stab`;
- production `cpause1`: preserved.

Observed offline result:

- 19,704 input triangles → 19,704 output triangles;
- 5 spatial components audited;
- 4 detached crystal components stabilized;
- 341 accessory vertices reweighted;
- 12 changing animation controllers in every routed attack output;
- wind-up and impact poses are kinematically distinct;
- the terminal pose equals the initial bind pose;
- canonical binary MDL readback: `MATCH`;
- native `hit` event present in `ca1slashl`, `ca1slashr` and `ca1stab`;
- `cpause1` motion differs from the authored attack.

Regression:

```text
cargo test -p m2a-core --test animation_studio_v5 \
  exact_void_crystal_knight_v5_materializes_a_real_authored_sword_attack \
  -- --ignored --nocapture
```

## Runtime demo

The owner's explicit request for a corrected demo admitted a fresh
animation-only candidate. The application pipeline materialized and installed:

- MOD: `vckattack1.mod`;
- HAK: `vckattack1.hak`;
- model: `vckattack1.mdl`;
- packet:
  `artifacts/void-crystal-knight-authored-sword-attack-demo-v1-2026-07-30`.

The production `cpause1` remains intact. The authored attack is routed to
`ca1slashl`, `ca1slashr` and `ca1stab`, and the generated creature uses the
active-monster combat profile.

The Toolset viewport does not execute combat AI and may therefore show idle.
The attack must be judged in NWN combat, where the engine selects one of the
three routed native attack states. Exact hashes and owner-test instructions are
recorded in:

[`evidence/void-crystal-knight-vckattack1-ready-for-owner-proof-2026-07-30.md`](evidence/void-crystal-knight-vckattack1-ready-for-owner-proof-2026-07-30.md).
