# `m2aborzmod4.mod`

Toolset module name: `Meshy2Aurora Borzoi c_wolf Demo V4`\
Exact Area name: `Meshy2Aurora Borzoi Test Area V4`

Date: 2026-08-20

Status: `ready_for_owner_proof`. The exact MOD and HAK are installed in the
native NWN user directories and verified byte-for-byte. The project owner owns
the final Toolset/NWN visual verdict; no agent-run visual-success claim is made.

## What V4 fixes

The owner-visible V3 result proved that the model and inherited combat behavior
loaded, but the long-haired surface formed spikes, sheets and severe head/neck
deformation. V4 implements the explicitly authorized quality-remediation delta:

- correct Aurora handedness: negative X is the creature's left side;
- source-derived neutral root and quadruped pivots;
- all 1,836 disconnected source surfaces receive one coherent dominant-bone
  assignment, eliminating cross-bone stretching inside every surface;
- four under-covered upper-leg regions receive deterministic, spatially nearest
  component assignments;
- every one of the 16 leg bones owns at least 661 primary vertices (maximum
  4,883), replacing the old non-zero-only gate with a material coverage gate;
- an offline oracle resolves real clips directly from retail `c_wolf` without
  copying them into the generated model and blocks V4 on edge stretch,
  compression, area collapse, area expansion or non-moving clips.

The generated model retains a 30-node compatible hierarchy, has zero local
animation clips and inherits 42 clips from `c_wolf`.

## Offline deformation result

Seven representative inherited clips were sampled at five times each:
`cpause1`, `cwalk`, `crun`, `ca1slashl`, `ca1slashr`, `cdamagel` and
`ckdbckdie`.

| Metric | V3 baseline | V4 |
|---|---:|---:|
| Edge samples | 31,500,000 | 31,500,000 |
| Stretch over 2x | 539,614 | 0 |
| Compression under 0.5x | 108,885 | 0 |
| Triangle samples | 10,500,000 | 10,500,000 |
| Area collapse under 0.2x | 12,749 | 0 |
| Area expansion over 5x | 217,048 | 0 |

All seven clips move the skin. `worldNormalOppositionCount` remains
informational: a correctly rigid triangle rotated by an inherited attack or
death animation can face more than 90 degrees away from its bind normal, so
that value is not a topology-flip verdict.

## Exact owner handoff

- Test module file: `m2aborzmod4.mod`
- Ordered HAK: `m2aborzhak4`
- Creature blueprint: `m2aborzutc4`
- Appearance row: `848`
- Model resref: `m2aborzcre4`
- Texture resref: `m2aborztex4`
- Fixture: `(10.0, 14.5, 0.0)`, facing `(0.0, -1.0)` toward the player entry point

## Immutable artifact identity

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2aborzmod4.mod` | 15,205 | `6a7b321c963d2425b371cc7f5132aea6e40c410cf6f9d88a83b3dc8d078fdfc6` |
| `m2aborzhak4.hak` | 76,222,468 | `fb772efc667cfadf4729c318fd7d7e83454ce64fc6f9f300e6a854df7beb607b` |
| `m2aborzcre4.mdl` | 25,502,368 | `fa369a7259565efb073de13d5fa517acdc4d50776d4cc9769c0454f4ba493a69` |
| `m2aborztex4.tga` | 50,331,692 | `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1` |

Canonical packet: `proof-output/borzoi-c-wolf-demo-v4-20260820`.

Native installation:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aborzmod4.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aborzhak4.hak`

Both destination hashes equal their canonical sources.

## Owner test criteria

In NWN, verify the exact V4 module and judge these visible properties:

1. the dog stands on the ground and faces the player;
2. head, neck, torso and coat retain a dog-like silhouette without spikes or
   sheet-like stretching;
3. all four legs, including upper segments and paws, participate during idle,
   walk/run and combat;
4. no large component detaches conspicuously from the body during motion;
5. inherited `c_wolf` movement and attack behavior remains active.

Until the owner reports that result, the NWN axis remains
`modelVisibility=not_tested`, `proofCompleteness=missing`,
`qualityVerdict=not_tested`.
