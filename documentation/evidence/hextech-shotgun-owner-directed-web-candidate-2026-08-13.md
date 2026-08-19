# Hextech Shotgun owner-directed web candidate

Date: 2026-08-13

Scope: Meshy2Aurora web application and offline geometry gates

Branch: `codex/items-agent-remediation`

> **Superseded on 2026-08-17.** The owner rejected the composition recorded in
> the original packet below: it incorrectly depended on retail heavy-crossbow
> BaseItem `6` and placed the assembled longitudinal frame on Aurora `Z` rather
> than `Y`. The final amendment at the end of this file is authoritative for
> the repaired web candidate. The earlier values remain only as failure
> provenance.

## Result

The Studio recognizes one exact candidate from immutable source and reference
identity, applies its directed Bottom/Middle/Top transforms, and submits those
transforms to `ITEM_REFERENCE_MANUAL_FIT_V2`. The candidate passes both
authoritative adjacent-connector gates and preserves the HAND reference frame.

This is a technical web-candidate result. `ownerStatus` remains
`NOT_REVIEWED`; this packet does not claim Aurora Toolset or NWN visual proof.

## Immutable identity

| Input | Identity |
|---|---|
| Agent-only comparison reference | `documentation/concepts/firearm-hextech-shotgun-v1/hextech-shotgun-concept.png` · SHA-256 `cfa31ccea74b53b1e0c55182ec3e1ed4a2072041b433009a448b7717509bd8f9` |
| Bottom GLB | `sample-3d/tlc-hextech-shotgun-parts-v1/bottom.glb` · SHA-256 `69c78999590b248bf9c642516ffa595d33774ead3436166963b27dfaa71ad48d` |
| Middle GLB | `sample-3d/tlc-hextech-shotgun-parts-v1/middle.glb` · SHA-256 `8fafe6a55dd77107a67f29c7519f3b6edc390b310f918a89131b003517720147` |
| Top GLB | `sample-3d/tlc-hextech-shotgun-parts-v1/top.glb` · SHA-256 `6ce1281a4ed8a239bf0d6fc9388fe8a977a2811750d40eab4320e13b642c77bf` |
| Reference set | `wbwxh_b_014/wbwxh_m_014/wbwxh_t_014` |
| Output identity | BaseItem `113` · `hextech_shotgun` · ItemClass `WHxSh` · ModelType `2` |

The exact ignored retail references are available read-only under
`C:\Projects\meshy2aurora\local-reference-assets\item\hextech-shotgun-wbwxh-014`:

| File | SHA-256 | Bytes |
|---|---|---:|
| `wbwxh_b_014.mdl` | `66a9c08a9af50181442086cade525a185d9047447f62451ec92952f6f56ca3ec` | `4,812` |
| `wbwxh_m_014.mdl` | `29274c2ccb02e72cd706c6948d0ba0ff0ecc2225dfc4be7126c6b93265c24524` | `16,268` |
| `wbwxh_t_014.mdl` | `996ec5b3878fabfb451c4b6d2e13291beca47144d34bd138833b432d9fe46c0a` | `18,962` |

These files were copied from the earlier temporary extraction only after
before/after SHA-256 equality was confirmed. They remain Git-ignored reference
inputs and are not product, fixture, HAK or MOD payloads.

The concept identity above records documentation provenance only. It is not a
Studio input, product asset or directed-composition identity field.

## Exact directed transforms

| Part | Translation | Quaternion XYZW | Internal uniform scale | Target-space scale |
|---|---|---|---:|---|
| Bottom / ModelPart1 | `[-0.00431, 0.12917034, 0.15773459]` | `[0.5, -0.5, 0.5, 0.5]` | `0.15796308` | `[1, 1, 1]` |
| Middle / ModelPart2 | `[-0.00431, -0.00852164987, -0.18716540565]` | `[-0.5, -0.5, -0.5, 0.5]` | `0.21066014` | `[1, 1, 1]` |
| Top / ModelPart3 | `[-0.00431, 0.008945521, -0.49844033]` | `[-0.5, -0.5, -0.5, 0.5]` | `0.13161969` | `[1, 1, 1]` |

The internal GLB scale is intentionally distinct from the target-space scale.
The latter remains the owner's exact `[1,1,1]` contract. Substituting absolute
`uniformScale=1` fails the authoritative surface/connector gate and is covered
by the real-corpus regression.

## Authoritative geometry result

- validation algorithm: `ITEM_REFERENCE_MANUAL_FIT_V2`;
- tolerance: `0.005`;
- Bottom → Middle axial overlap: approximately `0.0051`;
- Middle → Top axial overlap: approximately `0.0137`;
- adjacent connections: `2/2 OVERLAPPING`;
- non-adjacent Bottom/Top separation: preserved;
- attachment route: `HAND` preserved;
- fit status: `PASSED`.

## Regression coverage

- `itemAuthoringRecipeV2.test.ts` binds the exact source hashes, reference ID,
  protected Bottom/Top transforms and Middle-only correction scope;
- `ItemWorkflow.test.tsx` proves that the exact candidate performs the baseline
  request followed by manual-fit validation without rendering the agent-only
  concept reference or marking the candidate owner-accepted;
- `crates/m2a-core/tests/item.rs` validates these exact transforms against the
  real canonical GLBs and exact retail MDL frames under the env-gated corpus.

## Remaining owner decision

The final visual decision belongs to the owner. The Studio explicitly shows
`Visual owner acceptance is still pending` in the candidate status. Agent-run
Toolset/NWN proof remains forbidden by the project decision dated 2026-07-24.
No new MOD, HAK, resref or model iteration was created for this web candidate.

On 2026-08-13 automated control of the local in-app browser was rejected by the
browser URL policy. No screenshot or visual acceptance is inferred from that
lane failure. The application can still be opened for the owner's direct review.

## Owner scope correction 2026-08-17

The owner clarified that the supplied concept art was intended only for the
agent's implementation comparison, not for display or packaging in Studio.
The product import, UI comparison card, CSS, runtime contract field and
concept-bound tests were removed. The PNG remains only under `documentation`
as implementation-review provenance. Previously captured screenshots of the
incorrect concept-bearing UI were removed from the active evidence set.

Post-correction verification loaded the exact candidate again in Studio. The
live UI retained `2/2 connected`, `HAND preserved`, `Fit validated` and the
pending owner decision, while both the concept image count and concept-label
count were zero. The production Vite build emitted no
`hextech-shotgun-concept` asset.

## Build-boundary repair 2026-08-17

The first live package attempt exposed
`ITEM-FIT-PROVENANCE-MISMATCH: ModelPart2 source, sourceNode or transform
changed after fit`. The source and node identities were unchanged. The cause
was numeric representation at the JavaScript/WASM boundary: the directed
editor values are JavaScript numbers, while `ItemPartTransformV1` validates,
hashes and serializes `f32`. ModelPart2 contained additional decimal digits,
so its authored JavaScript tuple and the authoritative returned `f32` tuple
were equivalent for geometry but not identical under the Worker's strict JSON
provenance comparison.

Studio now rebinds every editor part to the exact transform returned by the
successful fit report before enabling the package build. The directed-candidate
recognizer treats only `f32`-identical numeric representations as equivalent;
the Worker still requires exact JSON equality between the build request and
the validated fit snapshot. A regression test simulates the WASM `f32`
round-trip and requires the emitted ModelPart2 build request to contain the
validated representation.

The next attempt exposed a separate presentation-only issue:
`ITEM-ICON-PRESENTATION-FIT-INVALID`. The real source corpus proves that the
deterministic icon layout has a valid Aurora YZX frame and positive axial
overlap for both ordered part pairs, while the second pair has a physical
surface gap and therefore reports `MANUAL_REQUIRED`. Surface continuity is a
world-model requirement, not an inventory-icon requirement. The icon binding
now accepts that state only when all three source hashes and nodes match, the
frame is exactly YZX with proper handedness, both axial overlaps remain inside
their required ranges, and all transform hashes are valid. The emitted icon
still has to pass the independent shared-canvas silhouette, clipping and
Bottom/Middle/Top order conformance gate.

After both boundary repairs, the same source/reference candidate completed a
fresh live offline build in Studio:

- status: `OFFLINE_ITEM_PACKAGE_PASSED`;
- UTI: BaseItem `113`, ModelType `2`, numeric part readback `PASS`;
- concrete model resources: `12` MDLs, all append-conformance `PASS`;
- icon resources: `12` TGA layers and `4` native composites, conformance
  `PASS`;
- triangle count: `28,620 / 300,000`;
- persisted world seam gate: both adjacent pairs overlap, Bottom/Top remains
  separated;
- candidate HAK: `m2aihak35.hak`, SHA-256
  `72fe7a79b0e894547cc4624ff9df05d9b23f184fc1f80faec18b8f061cf66ef0`;
- candidate MOD: `m2aimod35.mod`, SHA-256
  `8aafcd1e61420e164b1f31ad0476c19cdfb35077ce7dfdab390bd2af6ffb62ed`.

These artifacts remain in the live Studio review result. They have not been
owner-accepted, downloaded, installed or byte-verified in the native NWN user
directories. Accordingly the candidate remains `modelVisibility=not_tested`,
`proofCompleteness=missing` and not ready for owner proof. No Toolset or NWN
session was started.

## Owner rejection and standalone manual repair 2026-08-17

The owner explicitly rejected the preceding candidate and clarified four
requirements:

1. Studio must create a new BaseItem rather than clone or use an existing base.
2. Bottom, Middle and Top must be manually assembled from the exact canonical
   GLBs according to the agent-only visual reference.
3. The author must perform the direction and rotation correction explicitly;
   an automatic fit is not the source of truth.
4. The concept image must not be an application input, display, package asset
   or acceptance criterion.

The confirmed implementation defect was a frame mismatch. The earlier
candidate's fit report used `targetAxialAxis=2` (Aurora `Z`), while Studio's
diagnostic text incorrectly claimed axial `Y`. A manually authored rigid
`+90°` rotation around Aurora `X` maps the complete ordered chain from `-Z` to
`+Y` without changing either connector overlap. The repaired order is Bottom
(stock) → Middle (receiver) → Top (barrels), with ascending centers on `Y`.

### Repaired standalone BaseItem contract

- physical append index and output identity: BaseItem `113`, label
  `hextech_shotgun`, ItemClass `WHxSh`;
- schema: standalone request V3;
- definition source: `EXPLICIT_COLUMN_ASSIGNMENTS`;
- no donor BaseItem is resolved, read, cloned or referenced by the product
  flow;
- explicit cells include ModelType `2`, `2x4` inventory footprint, item model
  and icon defaults, equipable slots, ranges and weapon behavior columns;
- the application no longer requests retail `WBwXh` Bottom/Middle/Top MDLs for
  this authoring route.

### Repaired manual transforms

| Part | Translation | Quaternion XYZW | Authored Euler | Internal scale |
|---|---|---|---|---:|
| Bottom / ModelPart1 | `[-0.00431, -0.15773459, 0.13717034]` | `[0.70710678, -0.70710678, 0, 0]` | `[180, 0, 90]` | `0.15796308` |
| Middle / ModelPart2 | `[-0.00431, 0.18716541, -0.00852165]` | `[0, 0, -0.70710678, 0.70710678]` | `[0, 0, -90]` | `0.21066014` |
| Top / ModelPart3 | `[-0.00431, 0.49844033, 0.008945521]` | `[0, 0, -0.70710678, 0.70710678]` | `[0, 0, -90]` | `0.13161969` |

The application consumes only this numeric author-authored profile and the
three exact source hashes. Its identity is
`hextech-shotgun-manual-assembly-v3`; the concept filename and concept hash are
absent from the runtime contract.

### Repaired gate results

- target frame: axial `Y`, width `Z`, depth `X`;
- Bottom → Middle overlap: approximately `0.0051`;
- Middle → Top overlap: approximately `0.0137`;
- adjacent connector status: `2/2 OVERLAPPING`;
- exact GLB source hashes: unchanged from the immutable identity table above;
- standalone append regression uses a synthetic 113-row table containing no
  `heavycrossbow` or `WBwXh` text and passes;
- core Item suite: `52 passed`;
- Studio suite: `268 passed`, `1 skipped`;
- TypeScript typecheck: passed;
- production WASM/Vite build: passed.

Studio now blocks the custom build unless the exact source hashes, authored
profile identity and all three validated transforms still match this manual
contract. That is a technical gate for the authored numeric result, not an
image-comparison criterion. Visual owner acceptance remains pending, and no
Toolset or NWN session was started.

### Live Studio readback

The repaired production build was then opened in the in-app browser. A
113-row input table containing only neutral filler rows was loaded, followed by
the three exact canonical GLBs from `sample-3d`. Studio read back:

- `BaseItem 113 · hextech_shotgun · BOTTOM_MIDDLE_TOP`;
- `Manually assembled candidate loaded`;
- `ITEM_REFERENCE_MANUAL_FIT_V2` and `FULL FRAME PASSED`;
- frame `depth X · axial Y · width Z`;
- Bottom → Middle `+Y overlap 0.0051`;
- Middle → Top `+Y overlap 0.0137`;
- `2/2 connected`, `HAND preserved` and `Fit validated`;
- stock, receiver and twin barrels rendered as one ordered composition.

No retail donor MDL and no concept image was supplied to Studio. The live tab
was left on this exact Prepare Item result for owner inspection. Build Package
was deliberately not invoked: this amendment changes the web authoring
contract but does not claim or allocate a new Toolset/NWN proof iteration.

### Visible in-page proof panel

At the owner's request, the Prepare Item page now renders the candidate-bound
technical proof directly above the composition viewport. It exposes the
standalone V3 identity, `No donor BaseItem`, authoring evidence, YZX frame,
ordered assembly, all three complete GLB SHA-256 values, validated T/Q/S
transforms, both connector overlaps, and the exact profile and fit hashes.
The panel appears only when the exact custom identity, source hashes, authored
profile and transforms all match; otherwise it is absent and the build remains
blocked. It contains no concept image and makes no Toolset/NWN visibility
claim. Live browser readback showed `VERIFIED`, `+Y 0.0051` and `+Y 0.0137`.

## Visible weapon proof correction 2026-08-17

The owner correctly rejected the preceding in-page evidence as insufficient:
the technical panel was visible, but the actual weapon appeared only lower in
the page and the Item Properties camera presented Aurora axial `+Y` as
screen-up. That produced a narrow vertical silhouette even though the authored
Bottom/stock → Middle/receiver → Top/barrels chain was present.

The Studio proof presentation now keeps the exact emitted model transforms and
the Aurora YZX frame unchanged, but rolls the broadside camera so the authored
axial direction reads left-to-right. The exact standalone candidate therefore
opens with a visible horizontal weapon viewport before the detailed hash panel:

- label: `Visible standalone weapon`;
- identity: `BaseItem 113 · authored broadside assembly`;
- accessible viewport identity:
  `Assembled standalone BaseItem 113 Hextech Shotgun`;
- camera presentation: `HORIZONTAL_BROADSIDE`;
- reference envelopes hidden by default for this standalone proof;
- no concept image, filename, hash or bytes imported into the application.

The owner-supplied concept was inspected only outside Studio to check the
left-to-right stock, receiver and barrel order. This correction is a camera and
page-order change, not a new model, MOD, HAK, resref or emitted-transform
iteration. Visual owner acceptance remains pending.

### Persistent local proof URL

The transient Item workflow depended on browser-selected `File` objects, so a
new tab at the same root URL could not reproduce the proof. The local Vite
server now exposes one read-only, allowlisted proof route that streams only the
three canonical GLBs from `sample-3d/tlc-hextech-shotgun-parts-v1`. The page at
`/proof/hextech-shotgun.html` hashes every payload in the browser, rejects any
identity mismatch, applies the exact authored transforms from the product
contract and renders the complete horizontal broadside assembly. It shows
BaseItem `113`, `WHxSh`, ModelType `2`, `No donor BaseItem`, `VERIFIED`, all
three full SHA-256 values and T/Q/S readback. No owner reference image is
served by this route.

Fresh live readback from the persistent URL showed the full, unobstructed
stock → receiver → twin-barrel silhouette and all three expected source hashes.
Unlike the transient file-picker state, this URL can be reopened directly
while the local development server is running.

### Owner broadside refinement

After reviewing the persistent broadside proof, the owner requested that only
the Bottom/stock sit slightly lower relative to the Middle/receiver. The
minimal authored delta is Bottom translation `Z 0.12917034 → 0.13717034`
(`+0.008`); Bottom axial `Y`, rotation, scale and pivot remain unchanged, and
Middle/Top are byte-for-byte unchanged. The authored Bottom slot bounds and
attachment-zone maximum were shifted by the same `+0.008` in width `Z`.

Fresh browser readback showed the adjusted complete silhouette and the exact
Bottom transform. The env-gated real-corpus test over all three canonical GLBs
and the exact retail reference frames passed, including both adjacent
connections. No MOD, HAK, resref or source payload was created or changed.
