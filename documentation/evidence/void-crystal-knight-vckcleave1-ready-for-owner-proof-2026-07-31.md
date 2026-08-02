# Void Crystal Knight `vckcleave1` — owner proof handoff

MOD: `vckcleave1.mod`  
Module name in Toolset: `Meshy2Aurora procedural humanoid proof`  
Area: `Meshy2Aurora M0 binary vertical-slice area`

Date: 2026-07-31  
Branch/worktree: `animation` /
`C:\Projects\meshy2aurora\.worktrees\animation`

## Status

- `modelVisibility=not_tested`
- `proofCompleteness=missing`
- `animationPlaybackProof=missing`
- agent-run Toolset/NWN proof: not performed
- exact MOD/HAK installation: verified
- offline Animation Studio V5 readback: `MATCH`
- handoff state: `ready_for_owner_proof`

The owner explicitly authorized one new demo iteration exclusively for
`m2a_voidcleave`. This packet is that iteration. It does not modify or replace
the previously verified `vckattack1` lineage.

## Where the demo is

Generated packet:

`C:\Projects\meshy2aurora\.worktrees\animation\artifacts\void-crystal-knight-void-cleave-demo-v1-2026-07-31`

Installed owner-test files:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\vckcleave1.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\vckcleave1.hak`

Both native destinations were absent before copying. The installed files were
then hashed and matched the generated packet byte for byte.

## Exact identity

| Resource | Size | SHA-256 |
|---|---:|---|
| `vckcleave1.mod` | 15,161 B | `f81c87cfaa6be86dc0bba9f873184c0a17ba41916d0dc75221d4f83ff5bb513f` |
| `vckcleave1.hak` | 22,040,198 B | `cf5429f2cda36fdf9e39e38564212564a6894cc9203ad741b88b97a5e22bf84e` |
| `vckcleave1.mdl` | 2,555,488 B | `33178eec422ee5cd457c87011ae8b67aaa254e86f900fe93aeae5c2acad782ad` |
| `vckcleave1.tga` | 12,582,956 B | `5101483b93b6eb4e0aa39cc705dcbed9de1a70c1661f85eacffcb58793025e8a` |
| generated `appearance.2da` | 6,901,498 B | `7341b8cf12f07968eecec36ba189cb76954c07e663bddab10d709b7db9180f20` |

Runtime bindings:

- module, Area, creature, model, texture and HAK resref: `vckcleave1`;
- Appearance row: `15101`;
- creature runtime profile: `ActiveMonsterBaseline`;
- creature faction: hostile;
- production idle `cpause1`: preserved.

## What the application authored

The MOD contains the exact application pipeline result, not a manually guessed
MDL animation:

1. Animation Studio clones the exact source clip `cpause1`.
2. It samples that clip at `0.54 s`.
3. It creates the one-second `m2a_voidcleave` motion from the sampled pose.
4. It authors six phases at `0.00`, `0.16`, `0.38`, `0.54`, `0.72` and
   `1.00 s`.
5. V5 routes that authored clip to `ca1slashl`, `ca1slashr` and `ca1stab`
   while preserving the production idle.

Authored clip identity:

- id: `vck-authored-void-cleave-v1`;
- name: `m2a_voidcleave`;
- source kind: `SOURCE_CLIP_COPY`;
- source clip: `cpause1`;
- source clip fingerprint:
  `2ce47124e3c1825fdbff7a3c321f1d4d74c604de6e90a8bf8bc90491a90c8502`;
- playback: `ONE_SHOT`;
- duration: `1.0 s`;
- transition: `0.1 s`;
- keyframes: `113`;
- changing controllers: `13`;
- motion SHA-256:
  `ead005d8037048d812d9b0096bf7922693e211beadcb08b73d5155c91220628f`.

`ONE_SHOT` means one complete attack ends after one second. NWN starts another
attack only when the combat state requests another swing; the clip is not
supposed to loop continuously like idle.

## Offline verification

- canonical source GLB SHA-256:
  `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`;
- input/output geometry: `19,704 -> 19,704` triangles;
- detached accessory stabilization: 4 components / 341 vertices;
- all eight generated-file manifest bindings: size and SHA-256 match;
- authored V5 binary readback: `MATCH`;
- `m2a_voidcleave` and all three native attack variants: present and moving;
- all native attack variants share the expected motion SHA-256;
- production idle preserved: yes;
- focused Animation Studio tests: 19/19 pass;
- Animation Studio V5 active tests: 8/8 pass, 2 exact-asset tests ignored by
  the default suite;
- both exact Void Crystal Knight tests: pass when run explicitly;
- `cargo fmt --check`: pass;
- `cargo clippy --workspace --all-targets -- -D warnings`: pass;
- full `cargo test --workspace`: execution exceeded the 120-second command
  budget after compilation; no test failure was reported before timeout.

## Owner proof

1. Open `vckcleave1.mod`.
2. Confirm the module name is
   `Meshy2Aurora procedural humanoid proof`.
3. Open `Meshy2Aurora M0 binary vertical-slice area`.
4. Use Test Module and enter combat with the hostile creature.
5. Judge a complete combat swing, not the Toolset idle viewport.
6. Report whether the exact `vckcleave1` model is visible and whether its
   attack is visually correct.

Until the owner reports that result, the candidate remains
`modelVisibility=not_tested`, `proofCompleteness=missing` and
`animationPlaybackProof=missing`.
