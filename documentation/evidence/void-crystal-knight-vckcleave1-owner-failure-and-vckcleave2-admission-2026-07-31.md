# `vckcleave1` owner failure and `vckcleave2` admission

Date: 2026-07-31  
Owner-tested MOD: `vckcleave1.mod`  
Owner-tested HAK: `vckcleave1.hak`  
Clip under test: `m2a_voidcleave`

## Exact failed lineage

| Resource | SHA-256 |
|---|---|
| `vckcleave1.mod` | `f81c87cfaa6be86dc0bba9f873184c0a17ba41916d0dc75221d4f83ff5bb513f` |
| `vckcleave1.hak` | `cf5429f2cda36fdf9e39e38564212564a6894cc9203ad741b88b97a5e22bf84e` |
| `vckcleave1.mdl` | `33178eec422ee5cd457c87011ae8b67aaa254e86f900fe93aeae5c2acad782ad` |

Immutable packet:

`C:\Projects\meshy2aurora\.worktrees\animation\artifacts\void-crystal-knight-void-cleave-demo-v1-2026-07-31`

The files installed for the owner test matched this packet byte for byte.

## Owner result

After testing the exact installed candidate, the owner reported:

> troche lepiej ale dalej fatalnie

This is a candidate-bound visual failure of the animation quality. The model
remained visible and the attack played, so it is not a model-visibility or
playback-lane failure.

- `modelVisibility=visible`
- `proofCompleteness=verified`
- `animationPlaybackProof=failed`
- failure class: `VISIBLE_ANIMATION_QUALITY_FAILURE`

No screenshot was supplied. The owner runtime report in the active task is the
authoritative visual verdict for this exact installed lineage.

## Diagnosed cause

The six-pose procedural motion was invented from large independent Euler
offsets. It has no credible boxing guard, foot pivot or leg drive, folds the
torso at impact, rotates both arms through oversized arcs and does not recoil
cleanly to a defensive pose.

Reference comparison uses the CMU Graphics Lab boxing captures:

- Subject 14, trial 02, `40.8-41.8 s`: guard, weight shift, straight punch,
  recoil and guard;
- Subject 15, trial 13, `33.0-34.2 s`: rear-hand drive with the off-hand kept
  near the head.

Local reference files remain temporary research inputs and are not promoted to
the canonical Meshy source-model library.

## Minimal admitted delta

One new iteration is admitted only for `m2a_voidcleave`:

- replace the motion phase table with a mocap-informed boxing strike;
- add coordinated hips, spine, legs and feet;
- keep one striking arm and one guarding arm;
- use a fast contact and recoil instead of an oversized follow-through;
- preserve the exact source GLB, geometry, texture, accessory stabilization,
  production idle, mapping contract and all non-attack animations;
- materialize a fresh `vckcleave2` lineage only after headless Studio preview
  review.
