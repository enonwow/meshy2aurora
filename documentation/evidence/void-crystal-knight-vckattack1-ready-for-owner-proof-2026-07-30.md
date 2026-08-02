# Void Crystal Knight `vckattack1` — owner proof handoff

MOD: `vckattack1.mod`  
Module name in Toolset: `Meshy2Aurora procedural humanoid proof`  
Area: `Meshy2Aurora M0 binary vertical-slice area`

Date: 2026-07-30  
Branch/worktree: `animation` /
`C:\Projects\meshy2aurora\.worktrees\animation`

## Status

- `modelVisibility=visible`
- `proofCompleteness=verified`
- `animationPlaybackProof=verified`
- agent-run Toolset/NWN proof: not performed
- exact MOD/HAK installation: verified
- offline Animation Studio V5 readback: `MATCH`

This is the first materialized candidate containing the corrected authored
sword attack. It supersedes the rejected historical `m2c7a58c250ba516`
showcase for animation testing.

## Where the demo is

Canonical generated packet:

`artifacts/void-crystal-knight-authored-sword-attack-demo-v1-2026-07-30`

Installed owner-test files:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\vckattack1.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\vckattack1.hak`

The native destinations were absent before copying. Post-copy SHA-256 values
match the canonical packet byte for byte.

## Exact identity

| Resource | Size | SHA-256 |
|---|---:|---|
| `vckattack1.mod` | 15,161 B | `e42152c6e714feb887c66cd2713de3215b727fcdfec5f3f96583f551b44b4d77` |
| `vckattack1.hak` | 22,039,878 B | `a97605c6ba3df055a97ced5e4e3e8b7ec14b023dbe3e674c0f59a0ef1cb79414` |
| `vckattack1.mdl` | 2,555,168 B | `ebb173c9af65726972c909034e75391e55661a0b3bdf1f920c5a78cd4f5943d6` |
| `vckattack1.tga` | 12,582,956 B | `5101483b93b6eb4e0aa39cc705dcbed9de1a70c1661f85eacffcb58793025e8a` |
| generated `appearance.2da` | 6,901,498 B | `11b7e1da255d69272386cf4df2827e87bd9052427846940c4e4c3ac883b257cb` |

Runtime bindings:

- module, Area, creature, model, texture and HAK resref: `vckattack1`;
- Appearance row: `15101`;
- player entry: `[10.0, 10.0, 0.0]`;
- creature position: `[10.0, 14.5, 0.0]`;
- creature runtime profile: `ActiveMonsterBaseline`;
- creature faction: hostile;
- standard active-monster AI scripts: preserved.

## What is actually animated

The application-created authored clip is `vck_cryslash`, made with the
`HUMANOID_SWORD_SLASH` Animation Studio preset. It is a one-second one-shot
attack using 12 changing upper-body controllers and returning to bind pose.

The corrected demo maps the same motion to every generic melee variant:

- `ca1slashl` — 12 changing controllers, native `hit` event;
- `ca1slashr` — 12 changing controllers, native `hit` event;
- `ca1stab` — 12 changing controllers, native `hit` event;
- standalone Custom library output: `vck_cryslash`.

All four outputs have motion SHA-256:

`56efa7d5ba2dcad4004ea898d5e337b77f79a52776b1bf872d2e3e3effe0e84d`

`cpause1` remains the original production idle. It is not replaced by the
attack and is not used as an autoplay workaround.

## How playback works

There are two separate playback contexts:

1. Animation Studio has its own Play/Pause transport and playhead. The
   authored clip is projected onto the source GLB skeleton and sampled by the
   preview runtime.
2. The generated MOD uses NWN combat state selection. The Aurora Toolset
   viewport does not run creature combat AI, so seeing idle there is expected.
   Start/Test the module in NWN and enter combat with the hostile creature;
   the engine then selects `ca1slashl`, `ca1slashr` or `ca1stab`. Every one of
   those slots contains the authored slash.

The old `m2c7a58c250ba516` packet instead replaced `cpause1` with
`vck_showcase`. That historical workaround explains the idle/comical motion
reported by the owner and must not be used to judge this candidate.

## Offline gates

- source GLB: exact canonical
  `sample-3d/void-crystal-knight-h1-v1/source.glb`;
- source SHA-256:
  `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`;
- input/output geometry: 19,704 triangles;
- detached accessory stabilization: 4 components / 341 vertices;
- authored V5 binary readback: `MATCH`;
- all three native attack variants routed: yes;
- production idle preserved: yes;
- active-monster MOD semantic readback: pass;
- exact real-asset regression: pass.

## Owner verdict

On 2026-07-30, immediately after testing the exact installed `vckattack1`
candidate, the owner reported:

> teraz faktycznie jest attack

This closes the human-owned visual stage for this exact MOD/HAK lineage:

- the model is visible;
- the expected attack state plays visibly;
- the previous idle/comical showcase defect is not present in this candidate;
- the authored attack pipeline has an owner-confirmed runtime result.

No agent-run Toolset or NWN session was used to make this claim. The visual
verdict is the owner's report; the offline identities and animation readbacks
above bind it to `vckattack1`.
