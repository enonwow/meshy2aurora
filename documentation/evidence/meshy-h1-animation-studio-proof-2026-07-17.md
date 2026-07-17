# Meshy H1 animation Studio proof — 2026-07-17

Status: `STUDIO CPAUSE1 PASS / NATIVE AURORA-NWN OPEN`.

## Scope

This packet records a local, browser/WebAssembly proof for the owner-provided
Meshy H1 input. It is deliberately not a claim that the generated model already
animates in Aurora Toolset or NWN EE.

## Input and route

- local ignored input: `test-assets/meshy/incoming/h1-humanoid-1500.glb`;
- local ignored appearance table: `local-reference-assets/appearance.2da`;
- route: local file -> Studio Worker/WASM -> own binary MDL writer -> own
  binary readback -> Three preview built only from that readback;
- clip: generated `cpause1`, duration `4.03 s`.

The preview does not load the source GLB at the result stage. It projects the
Rust readback's node tree, meshes, skin `nodeToBoneMap`/`inlineMapping`, raw
inverse binds, weights, bone references and decoded controller tracks.

## Visible evidence

All binary inputs and rendered artifacts remain local and ignored by Git:

`C:\Projects\meshy2aurora\proof-output\meshy-h1-e2e-2026-07-17`

| Artifact | Observation |
|---|---|
| `13-converted-h1-skinned-000ms.png` | H1 is visibly rendered as a skinned binary-MDL readback at `0.00 s`. |
| `14-converted-h1-skinned-500ms.png` | The same H1 remains in frame at `0.50 s` after 15 decoded keyframes; its limbs/pose visibly differ. |
| `15-converted-h1-cpause1-skinning-000-to-500ms.mp4` | 16 captured browser frames from real `cpause1` states `0.00..0.50 s`. |
| `15-converted-h1-cpause1-skinning-000-to-500ms.gif` | Looping review copy of the same 16 states. |
| `16-converted-h1-cpause1-skinning-full-clip.mp4` | Full `0.00..4.03 s` clip sampled from 16 browser states; each later state advances eight decoded keyframes, and the final pose remains visible. |
| `16-converted-h1-cpause1-skinning-full-clip.gif` | Looping review copy of the full-clip samples. |

Image comparison over the render area between the first and final frame found
`15,965` pixels over absolute-difference threshold `20` (`1.893%` of the
`1152 x 732` render area). This is supporting evidence of visible motion, not
a substitute for native runtime acceptance.

The full-clip capture advances from `0.00` to `4.03 s` with `120` exact
`Next keyframe` operations (eight between each of its 16 samples). It does not
reposition the camera or advance time by a synthetic timer.

## Regression gates executed

```text
cargo test -p m2a-core --test profile_a
# 40 passed

npm --prefix apps/studio-web test -- --run \
  src/features/preview/AuroraReadbackViewport.test.tsx \
  src/features/results/projectReadback.test.ts
# 15 passed

npm --prefix apps/studio-web run build
# PASS

npx vitest run --config vitest.worker-integration.config.ts \
  tests/browser/h1-skinning-diagnostic.integration.ts
# local real-H1 PASS; harness is intentionally not committed because its
# source GLB and appearance.2da are local ignored owner inputs.
```

## Fix captured by this packet

The initial result preview used `THREE.Mesh` and therefore discarded all
canonical skin data. It now creates `THREE.SkinnedMesh` only after validating
the decoded inverse-bind tree ordinals against `inlineMapping` and
`nodeToBoneMap`.

The first real-H1 visual capture then exposed a separate retargeting defect:
the H1 Armature container scale and Profile A geometry normalization scale were
both applied to root translation deltas. The H1 mapping now compensates that
composition so animated deltas stay in the target bind-local coordinate space.
The first/last evidence frames show the concrete outcome: the model no longer
leaves the viewport at `0.50 s`.

## Remaining acceptance gates

1. Capture a non-loop terminal-pose proof if a future source clip is intended
   to be one-shot rather than this looping Idle.
2. Produce required movement/gameplay clips (`cwalk`, `crun`, attack, damage,
   death) before claiming a complete animation set. This H1 input currently
   contains one source Idle clip only.
3. With a separately authorized Toolset/game write boundary, install the
   generated HAK/MOD and record a native Aurora/NWN playback proof. Until then
   the native acceptance status remains `OPEN`.

## Native packet prepared, not installed

The reproducible CLI route is now available:

```text
cargo run -p m2a-core --example materialize_m6 -- \
  --meshy-h1-source test-assets/meshy/incoming/h1-humanoid-1500.glb \
  --appearance-2da local-reference-assets/appearance.2da \
  --output-dir <new-empty-output-dir>
```

It refuses to overwrite an existing output directory and writes the source,
MDL, TGA, append-only `appearance.2da`, HAK, MOD and reports together. A fresh
packet was materialized locally at:

`C:\Projects\meshy2aurora\proof-output\meshy-h1-native-packet-2026-07-17-v2`

| Artifact | SHA-256 |
|---|---|
| `generated/m2a_m6p01.hak` | `da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756` |
| `generated/m2a_h1proof.mod` | `52732975d10bb3718a86dc657a9ead71775eb65fbfe3fe2ed1a727845445a4f7` |

The packet retains neutral `generated/source.glb` provenance naming; it does
not claim that the source is an owned fixture. The packet remains within the
canonical workspace. Nothing has been copied to the NWN installation or user
module directories.

## Native installation gate

`tools/install-nwn-h1-runtime-proof.ps1` makes the next operation explicit and
reproducible. Its default `-Mode Plan` is read-only: it verifies the two packet
hashes, checks the user `hak`/`modules` destinations and reports target state.
Only `-Mode Install` attempts a copy; it refuses to overwrite either target and
verifies the destination hash after each copy.

Plan executed on 2026-07-17:

```text
HAK source SHA-256: da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756
HAK target: C:\Users\enonw\Documents\Neverwinter Nights\hak\m2a_m6p01.hak (ABSENT)
MOD source SHA-256: 52732975d10bb3718a86dc657a9ead71775eb65fbfe3fe2ed1a727845445a4f7
MOD target: C:\Users\enonw\Documents\Neverwinter Nights\modules\m2a_h1proof.mod (ABSENT)
```

This is preparation only, not native proof. The remaining permission boundary
is an explicit approval to place those two generated files in the user's NWN
folders and then operate the live Toolset/game for a visible capture.

## Superseded runtime packet

For all future native proof work, the earlier local names in this evidence are
historical only. The one canonical test module and its one companion HAK are
now defined in `documentation/codex-animation-runtime-proof-module.md`:
`m2a_codex_aproof.mod` and `m2a_codex_aproof.hak`. This document's Studio
evidence remains valid, but its old packet must not be installed for a new
Toolset/NWN proof.
