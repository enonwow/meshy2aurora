# Applied supermodel preview c_wolf V1 — implementation record

Date: 2026-08-21

## Outcome

Studio can now build a source-fitted diagnostic preview after applying the exact
binary `c_wolf` supermodel to the currently selected creature GLB. The resulting
owned binary MDL is read back by `m2a-core` and shown in the existing Three.js
supermodel compositor. Inherited clips, the target skin, bones and joints are
therefore inspected together before any export decision.

The library still inventories and previews every resolved supermodel. Automatic
application is deliberately limited to exact `c_wolf`; other supermodel families
need their own source-rig profile before they can enter this route.

## Product behavior

1. Select and inspect a creature GLB in Studio.
2. Open the supermodel library and select exact binary `c_wolf`.
3. Use `Zastosuj c_wolf do nowego modelu`.
4. Studio sends the exact source GLB, selected `c_wolf` bytes, source-forward
   axis and a SHA-256 binding to the Worker/WASM boundary.
5. WASM builds the source-fitted rig V5, exact reference motion contract and an
   owned binary MDL whose `supermodelName` is `c_wolf`.
6. The `Na nowym modelu` viewport composes the generated target with the
   inherited `c_wolf` clips. Bones, joints and X-ray remain available in that
   viewport.

The strict export route remains fail-closed. A separate diagnostic-preview
entry point may return a model after a sampled motion-quality failure, but its
report is required to preserve `motionCompatible=false`, `BLOCKED` quality and
`DIAGNOSTIC_PREVIEW_ONLY_MOTION_QUALITY_BLOCKED`. Studio displays a warning and
does not present that result as export-ready.

## Real API evidence: canonical Borzoi plus retail c_wolf

The release-profile boundary test read `c_wolf` from the locally selected
read-only `nwn_base.key`/BIF lineage and used the canonical source at
`sample-3d/borzoi-c-wolf-n1-p60k-v1/source.glb`.

- source size: `6,847,588` bytes
- source SHA-256: `44b5d3c63387ae7587b4de8dc9e4a6866948cd034cd1a1e3a0c0ce69af3e6678`
- retail `c_wolf` size: `340,812` bytes
- retail `c_wolf` SHA-256: `a17b3613c356399d996707d26c4c6d87aa659fe5935c620c7e72de4443c7d726`
- generated preview MDL SHA-256: `5bd0f9526795f0aba7ee3ac97f4223646e351bb4c7c7fcea58f19c13632fcbb9`
- generated preview MDL size: `7,690,800` bytes
- readback supermodel: `c_wolf`
- local animations: `0`
- inherited animations: `42`
- required sampled clips: `8`
- bind-pose compatible: `true`
- skin-bind compatible: `true`
- motion compatible: `false`
- motion-quality status: `BLOCKED`
- blocking seam samples: `961,926`, allowed `519,265`

This is positive evidence that the applied-preview API builds and reads back the
exact result and exposes the inherited animation set. It is also positive
evidence that the quality gate is not bypassed or hidden: the current canonical
Borzoi result remains diagnostic-only.

The proving command was:

```powershell
$env:M2A_REFERENCE_NWN_KEY='C:\Program Files (x86)\Steam\steamapps\common\Neverwinter Nights\data\nwn_base.key'
$env:M2A_CWOLF_PREVIEW_SOURCE_GLB='C:\Projects\meshy2aurora\sample-3d\borzoi-c-wolf-n1-p60k-v1\source.glb'
cargo test --release -p m2a-wasm c_wolf_applied_preview_builds_from_real_selected_key_and_source_when_configured --lib -- --nocapture
```

Result: `1 passed; 0 failed`.

The public JavaScript/WASM export was then exercised independently through
`crates/m2a-wasm/tests/c_wolf_applied_preview_boundary.cjs`. Its returned API
summary was:

```json
{"api":"buildCWolfAppliedPreviewV1","status":"APPLIED_PREVIEW_BLOCKED_MOTION_QUALITY","supermodelResref":"c_wolf","inheritedAnimationCount":42,"bindPoseCompatible":true,"skinBindCompatible":true,"motionCompatible":false,"motionQualityStatus":"BLOCKED","modelSha256":"5bd0f9526795f0aba7ee3ac97f4223646e351bb4c7c7fcea58f19c13632fcbb9","modelByteLength":7690800}
```

## Verification

- `npm test -- --run`: `57` files and `325` tests passed.
- `cargo test -p m2a-wasm --lib`: `51` tests passed.
- `npm run build`: WASM, TypeScript and Vite production build passed.
- `npm run typecheck`: passed.
- public Node/WASM `buildCWolfAppliedPreviewV1` boundary: passed.
- canonical workspace guard: passed.

## Completion boundary and limitations

Implemented and verified offline:

- exact `c_wolf` application request/response contract;
- real KEY/BIF resource binding and SHA-256 verification;
- source-fitted binary MDL build and canonical readback;
- inherited clip compositor on the generated target;
- bones, joints and X-ray in the applied-result viewport;
- honest ready-versus-diagnostic quality verdict in API and UI;
- invalidation when the selected source or source-forward axis changes.

Not claimed:

- Toolset or NWN runtime visual success;
- export readiness of the current canonical Borzoi preview;
- automatic rigging/application for supermodels other than exact `c_wolf`;
- correction of the previously owner-observed Borzoi seam/tail quality defect.

Under the human-owned final-proof decision, no Toolset/NWN process was started
or controlled for this implementation. The offline viewport is an inspection
aid, not an Aurora runtime verdict.
