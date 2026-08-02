# Void Crystal Knight — Animation Studio visual QA

Date: 2026-07-30

## Exact source

- Canonical model: `sample-3d/void-crystal-knight-h1-v1/source.glb`
- Source SHA-256:
  `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`
- Base table SHA-256:
  `3dc8505bb5848044659cf44f9fe60e3f7c401db772496c4571ec0fff62a761d6`
- Studio fixture route:
  `?animationMode=edit&visualQaFixture=vck-authored-sword`

The fixture is development-only and opt-in. It verifies both exact files by
SHA-256, then feeds them through the regular Source, Inspect, Animation
Mapping, Create/Edit and Custom workflows.

## Browser-authored flow

- [x] Loaded the exact model and base table in the running Studio application.
- [x] Completed source inspection: 24,834 vertices, 19,704 triangles, 24 bones,
  and three source clips.
- [x] Created `Humanoid sword slash` from the `+ New animation` menu.
- [x] Viewed the authored result at start, wind-up `0.38 s`, impact `0.54 s`,
  follow-through and return.
- [x] Used the real Play control; the shared playhead advanced from `0.00 s`
  through the one-second clip.
- [x] Saved the clip as `m2a_sword_slash` with status `VALID`.
- [x] Confirmed one item in the Custom library.
- [x] Assigned that exact Custom item to Aurora base slot `ca1slashl`.
- [x] Confirmed `Base slots using Custom: 1`.

## Defects found only by visual authoring

1. The original default output name exceeded Aurora's 16-character boundary.
   It is now `m2a_sword_slash` (15 characters).

## Visual iteration 2 — Void crystal cleave

The earlier clip remained technically valid but did not read as a decisive
attack. A second in-app visual pass therefore replaced the subtle torso twist
with a model-specific one-shot preset exposed as:

`+ New animation` → `Void crystal cleave`

The generated output name is `m2a_voidcleave`. The clip still starts from an
exact sampled `cpause1` source pose and keeps `SOURCE_CLIP_COPY` lineage.

The authored motion now has six explicit phases:

1. neutral recovery-compatible start at `0.00 s`;
2. early anticipation at `0.16 s`;
3. coiled wind-up at `0.38 s`;
4. fast impact at `0.54 s`;
5. low follow-through at `0.72 s`;
6. return to the sampled source pose at `1.00 s`.

The impact interval stays intentionally short at `0.16 s`. The weapon-side arm
crosses more than 120 degrees between wind-up and impact. Hips translation adds
0.30 units of forward travel and 0.09 units of vertical compression so the
attack uses the whole body rather than only rotating the shoulders.

Headless Studio captures produced without occupying the owner's active window:

- `output/playwright/void-crystal-cleave/01-wind-up.png`
- `output/playwright/void-crystal-cleave/02-impact.png`
- `output/playwright/void-crystal-cleave/03-follow-through.png`
- `output/playwright/void-crystal-cleave/page@502b4bddf41047d5a435c05c4834588b.webm`

The captures show three visibly distinct silhouettes and continuous playback
through the normal Studio dope-sheet Play control. They are application visual
QA, not Aurora Toolset or NWN proof.

No MOD, HAK, resref, Appearance row, or immutable proof candidate was
regenerated. The existing model-iteration gate remains intact.

### End-to-end Studio result

- [x] `m2a_voidcleave` saved through `Save to Custom`.
- [x] Studio reported `Valid`.
- [x] The saved one-shot contains 113 keys and 6 timeline moments.
- [x] The Custom picker preserved `SOURCE_CLIP_COPY` provenance and the exact
  source fingerprint.
- [x] The clip was assignable to `ca1slashl`.
- [x] Mapping readback reported `Custom library: 1` and
  `Base slots using Custom: 1`.

The authored animation changes the rig only. The exact current source GLB does
not show separate sword geometry in the Studio viewport, so this pass proves
the body motion and Custom/Aurora mapping, not synthesis of a missing weapon
mesh.

### Verification

- `npm test -- --run`: PASS, 69 files passed / 2 skipped,
  389 tests passed / 3 skipped.
- `npm run typecheck`: PASS.
- focused animation-editor tests: PASS, 38/38.
- `cargo fmt --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test -p m2a-core --test animation_studio --test animation_studio_v5`:
  PASS, 18 + 8 tests; 2 exact local tests remain opt-in.
- exact Void Crystal Knight authored-attack regression: PASS.
- Studio production build and bundle budget: PASS.
2. The viewport framed the pre-animation bind pose and did not refit after a
   fullscreen resize. It now frames the displayed pose after seek and on
   resize.
3. The output-rig quaternion values were applied directly to source-glTF
   joints. This mixed Aurora XZY and glTF coordinate bases and visibly
   collapsed the character. Preview projection now applies the Profile A H1
   basis conjugation and source-relative translation delta.
4. The first attack used the output bind pose and excessive cumulative torso
   and arm rotations. The Studio preset now samples the real `cpause1` source
   pose, retains source-clip lineage, and uses visually reduced six-phase
   rotations.

## Captures

- [Impact pose at 0.54 s](../../artifacts/void-crystal-knight-animation-studio-visual-qa-2026-07-30/m2a-sword-slash-impact-0p54.png)
- [Saved VALID Custom clip](../../artifacts/void-crystal-knight-animation-studio-visual-qa-2026-07-30/m2a-sword-slash-valid-custom.png)

These captures are Studio visual-QA evidence. They do not replace the
human-owned Aurora Toolset/NWN proof boundary.
