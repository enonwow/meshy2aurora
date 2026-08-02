# Void Crystal Knight `vckcleave2` — owner proof handoff

MOD: `vckcleave2.mod`  
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

The owner reported the exact `vckcleave1` animation as visible but visually
failed. The durable failure and admission record is
`documentation/evidence/void-crystal-knight-vckcleave1-owner-failure-and-vckcleave2-admission-2026-07-31.md`.
This packet is the admitted replacement iteration and changes only
`m2a_voidcleave` plus its native attack routes.

## Where the demo is

Generated packet:

`C:\Projects\meshy2aurora\.worktrees\animation\artifacts\void-crystal-knight-void-cleave-demo-v2-2026-07-31`

Installed owner-test files:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\vckcleave2.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\vckcleave2.hak`

Both native destinations were absent before copying. The installed files were
then hashed and matched the generated packet byte for byte.

## Exact identity

| Resource | Size | SHA-256 |
|---|---:|---|
| `vckcleave2.mod` | 15,161 B | `48f663de89de889b7057d86ff97cc16d615a9af5c19537a0b96bf9e47f086b51` |
| `vckcleave2.hak` | 22,044,102 B | `540b2a349c6d972011cacac72b5b3172c3efd2949472765c6570c973ae3986fb` |
| `vckcleave2.mdl` | 2,559,392 B | `e2d5c2c3c26e1c32d87ffbf839c14c3b810f068fd74b81fcdac5e6484d7be720` |
| `vckcleave2.tga` | 12,582,956 B | `5101483b93b6eb4e0aa39cc705dcbed9de1a70c1661f85eacffcb58793025e8a` |
| generated `appearance.2da` | 6,901,498 B | `3c4f7d3e4e6b9c665af7fb3a7df2a56d3decaa842b8f4812b5024b3f508911e4` |

Runtime bindings:

- module, Area, creature, model, texture and HAK resref: `vckcleave2`;
- Appearance row: `15101`;
- creature runtime profile: `ActiveMonsterBaseline`;
- creature faction: hostile;
- production idle `cpause1`: preserved.

## What the application authored

The animation was created, previewed and materialized through the
Meshy2Aurora application pipeline:

1. Animation Studio clones the exact source clip `cpause1`.
2. It samples the source pose at `0.54 s`.
3. The editor creates a compact boxing guard for both arms.
4. A two-link arm solve authors the load, straight right impact and recoil.
5. Hips, three spine joints, head, both legs and both feet add the weight
   transfer and counter-rotation.
6. The one-second motion returns to the same guard without an idle override.
7. V5 routes the authored clip to `ca1slashl`, `ca1slashr` and `ca1stab`,
   and also preserves it as custom output `m2a_voidcleave`.

Authored clip identity:

- id: `vck-authored-void-cleave-v2`;
- name: `m2a_voidcleave`;
- editor status after `Save to Custom`: `VALID`;
- source kind: `SOURCE_CLIP_COPY`;
- source clip: `cpause1`;
- source clip fingerprint:
  `2ce47124e3c1825fdbff7a3c321f1d4d74c604de6e90a8bf8bc90491a90c8502`;
- playback: `ONE_SHOT`;
- duration: `1.0 s`;
- transition: `0.1 s`;
- phase times: `0.00`, `0.18`, `0.34`, `0.50`, `0.66`, `0.82`, `1.00 s`;
- keyframes: `162`;
- changing controllers: `19`;
- motion SHA-256:
  `dcd6aaf3e29110aaf40079180342f23b63f61328d1f47c38aad6c2e406d2044d`.

`ONE_SHOT` means one complete attack ends after one second. NWN starts another
attack only when combat requests another swing; the clip is not expected to
loop like idle.

## Application preview evidence

The exact preset was recreated in the Studio UI and inspected headlessly
without taking over the owner's desktop:

- guard:
  `output/playwright/void-crystal-cleave-v2/21-guard-front.png`;
- load:
  `output/playwright/void-crystal-cleave-v2/23-load-front.png`;
- impact:
  `output/playwright/void-crystal-cleave-v2/22-impact-front.png`;
- recoil:
  `output/playwright/void-crystal-cleave-v2/24-recoil-front.png`;
- continuous playback capture:
  `output/playwright/void-crystal-cleave-v2/boxing-cycle.webm`;
- live playback samples:
  `output/playwright/void-crystal-cleave-v2/live-00.png` through
  `live-04.png`.

The preview confirmed continuous playhead movement from guard through impact
and back to guard. This is application-side evidence only; it does not replace
the owner's Aurora/NWN visual verdict.

## Offline verification

- canonical source GLB SHA-256:
  `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`;
- input/output geometry: `19,704 -> 19,704` triangles;
- detached accessory stabilization: 4 components / 341 vertices;
- all eight generated-file manifest bindings: size and SHA-256 match;
- authored V5 binary readback: `MATCH`;
- `m2a_voidcleave` and all three native attack variants: present and moving;
- all four attack outputs share the expected motion SHA-256;
- production idle preserved: yes;
- `cargo fmt --all -- --check`: pass;
- `cargo clippy --workspace --all-targets -- -D warnings`: pass;
- full `cargo test --workspace`: pass;
- both exact Void Crystal Knight V5 tests: pass;
- Studio typecheck: pass;
- Studio tests: 389 passed, 3 environment-skipped;
- Worker/WASM browser tests: 23 passed, 2 environment-skipped;
- production Studio/WASM build and bundle budgets: pass.

## Owner proof

1. Open `vckcleave2.mod`.
2. Confirm the module name is
   `Meshy2Aurora procedural humanoid proof`.
3. Open `Meshy2Aurora M0 binary vertical-slice area`.
4. Use Test Module and enter combat with the hostile creature.
5. Judge a complete combat swing, not the Toolset idle viewport.
6. Confirm whether the exact `vckcleave2` model is visible and whether the
   guard, straight right impact, recoil and return are visually correct.

Until the owner reports that result, this candidate remains
`modelVisibility=not_tested`, `proofCompleteness=missing` and
`animationPlaybackProof=missing`.
