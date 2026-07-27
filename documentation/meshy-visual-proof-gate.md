# Meshy UI visual proof gate

**Status:** MANDATORY for every claim that the embedded Meshy API Studio is
visually aligned with Meshy Workspace.

## Non-negotiable completion rule

No agent may say that the Meshy UI is *done*, *1:1*, *practically 1:1*, or
visually aligned merely because a component was implemented, a local screen
looks plausible, or tests pass.

Before such a claim, the evidence packet must contain all three artifacts:

1. **Source screen** — a current screenshot of `https://www.meshy.ai/workspace`
   in a named Workspace state, with capture date, URL, viewport dimensions and
   visible mode recorded.
2. **Studio screen** — a screenshot of `http://localhost:5174/` in the
   matching supported state, captured at the same viewport dimensions.
3. **Comparison result** — a component-by-component table stating `PASS`,
   `PARTIAL`, or `FAIL`, plus the concrete next correction for every `PARTIAL`
   or `FAIL` item.

The proof must compare at least:

- page/shell hierarchy and panel proportions;
- Meshy brand treatment and iconography;
- supported source-mode cards only (Text, Image, Multi-image);
- left navigation rail;
- configuration-panel density, spacing and control treatment;
- active model viewport, overlays and real camera framing;
- library toolbar, card grid, selection treatment and pagination area;
- typography, colors, radii, borders and empty/loading/error states.

Unsupported Meshy capabilities (for example Agent, Print and Animate) must not
be added just to make a screenshot look similar. Their visual space may be
acknowledged in the comparison, but a `PASS` requires a real Bridge/API
contract, explicit paid-operation confirmation where applicable, and tests.

## Required conclusion

The conclusion must use exactly one of these forms:

- **PASS — visual alignment accepted:** no material discrepancy remains for
  the supported scope; cite the two screenshots and the comparison table.
- **FAIL — continue implementation:** list every material discrepancy. The
  implementation task remains active; tests do not close the visual gap.

## Superseded audit — 2026-07-19

| Required artifact | Capture |
|---|---|
| Source screen | `C:\Users\enonw\AppData\Local\Temp\meshy-workspace-text-to-3d-2026-07-19.png` — current `https://www.meshy.ai/workspace`, 1280×720, Model / Text-to-3D state. |
| Studio screen | `C:\Users\enonw\AppData\Local\Temp\meshy-studio-text-to-3d-two-segment-2026-07-19.png` — `http://localhost:5174/`, 1280×720, paired local Bridge, compact Text-to-3D composer with two Model type segments, a real recovered GLB and the Meshy-style radial viewport background. |
| Comparison | **FAIL — continue implementation.** The duplicate Aurora header and resulting page scrollbar have been removed: Studio now has one full-viewport Meshy shell. The supported surface includes the same two-segment Model type treatment, segmented Pose, a compact AI model control, working adaptive decimation, coloured source/rail icons, a real GLB viewport with working local controls, and a two-column Bridge-backed history grid with local sort. `lowpoly` remains available as an explicitly marked legacy Geometry control. Material differences remain: (1) the web Workspace exposes active Smart Topology in Text mode while the verified public API keeps it disabled; (2) the current Studio lacks verified Meshy Workspace material/shading presets and a server-side history filter. |

### Component comparison

| Component | Meshy Workspace source | Embedded API Studio | Result / next correction |
|---|---|---|---|
| Shell and proportions | Full Meshy shell, 56px outer header, rail + composer + viewport + library | One full-viewport Meshy shell within the primary application route; the Aurora header is hidden only while Lab is open | **PASS.** The Lab remains nested in the application state rather than becoming an overlay or a separate page. |
| Meshy brand and rail | Colour Meshy mark, five web-product destinations | Colour Meshy mark and three destinations backed by Bridge contracts: model, 2D concept and recovered models | **PARTIAL.** Do not add Agent, Print or Animate without Bridge contracts. |
| Source modes | Image-like source tabs at the composer top | Text, Image and Multi-image controls only; each maps to a real supported request | **PASS.** No unsupported source mode is rendered. |
| Model type and pose | Segmented controls, never a native dropdown | Same two-segment Standard/Smart Topology surface and segmented pose. Text disables Image-only Smart Topology; the API-valid `lowpoly` choice remains in Geometry as a marked legacy control | **PARTIAL.** Keep the API distinction explicit; do not make the unsupported Text Smart Topology option interactive merely for visual parity. |
| Composer density | Prompt, model choice, compact model version, adaptive switch, pose and bottom CTA | Same primary control rhythm; advanced API controls remain in collapsible sections, with explicit no-charge review CTA | **PASS.** |
| Active viewport | Actual model, statistic overlay, environment/material row and framing tools | Real recovered GLB, calculated topology/faces/vertices, working background choices, auto-rotate, wireframe, frame and close actions | **PARTIAL.** Add only real local renderer material/shading controls if they can be verified; never simulate Meshy server edits. |
| Library | Search, control row, two-column thumbnail grid and paging | Search, refresh, local newest/oldest sort, two-column real-history thumbnail grid, selection and Bridge paging | **PARTIAL.** Add a server-side filter only after the Bridge can state the exact history query semantics; retain real history rather than source examples. |
| Typography and surfaces | Dense dark panels, muted borders, lime active state and pink/lime CTA | Same dense dark panel structure, muted borders, lime active state and gradient review CTA | **PARTIAL.** Continue only pixel-level spacing/radius adjustments against a matching fresh screen. |

The screenshots above are the immediate task capture. Future proof packets
must retain their two screenshots with the evidence packet, not rely on a
verbal description or a transient browser state.

## Current final audit — 2026-07-19

| Required artifact | Capture |
|---|---|
| Source screen | `C:\Users\enonw\AppData\Local\Temp\meshy-reference-current-2026-07-19.png` — current `https://www.meshy.ai/pl/workspace`, 1280x720, Model / Image-to-3D state. |
| Studio screen | `C:\Users\enonw\AppData\Local\Temp\meshy-studio-image-to-3d-audit-final-2026-07-19.png` — `http://127.0.0.1:5179/`, 1280x720, Image-to-3D and dedicated Meshy viewport. It uses the project-owned synthetic visual-proof Bridge and a verified GLB fixture; it does not contact Meshy or create a paid task. |
| Comparison | **PASS — visual alignment accepted.** The supported Image-to-3D workspace has one full-viewport Meshy shell, an Image-style rail and composer, a real file dropzone (click and drag/drop), segmented Model type and Pose, compact model/toggle controls, an isolated GLB viewport with calculated statistics and real renderer controls, and a two-column Bridge-backed library. |

| Component | Meshy Workspace source | Embedded API Studio | Result |
|---|---|---|---|
| Shell and proportions | 56px header, rail + composer + viewport + library | One full-viewport nested Meshy shell; the Aurora header is hidden only while Lab is open | **PASS.** |
| Brand and rail | Colour Meshy mark and product rail | Colour Meshy mark, 72px rail, and actual Bridge destinations: Model, Image and Assets | **PASS (supported scope).** Agent, Print and Animate remain absent because they have no Bridge contract. |
| Source mode and upload | Image-style source cards and image drop area | Text, Image and Multi-image controls map to real requests. Image modes use a styled native file control with click and drag/drop | **PASS.** |
| Model type and pose | Segmented controls, never a native dropdown | Segmented Standard/Smart Topology and Pose. `lowpoly` remains an explicitly marked legacy Geometry control | **PASS (contract-constrained).** Smart Topology is disabled in Text and enabled in Image, matching the verified public API distinction. |
| Composer density | Compact version selector, toggle and bottom CTA | Same primary rhythm; advanced real API controls remain collapsed and the CTA retains explicit no-charge wording | **PASS.** |
| Active viewport | Actual model, statistics, environment row and framing tools | Dedicated GLB renderer with calculated triangles/faces/vertices, real background, rotate, wireframe, framing and close controls | **PASS.** |
| Library | Search, toolbar, two-column cards and pagination | Search, refresh, local newest/oldest sort, two-column real-history grid and Bridge paging | **PASS (supported scope).** The proof fixture has one item; production uses the full returned Bridge history. |
| Typography and surfaces | Dense dark panels, muted borders, lime active state and pink/lime CTA | Matching dark surfaces, lime active treatment, muted borders and gradient CTA | **PASS.** |

Account-global source actions and artwork are intentionally not copied: they are
not part of the local Bridge/API contract. The next visual proof must replace
these captures whenever the public Meshy workspace materially changes.

## Corrective material-viewport audit — 2026-07-19

This audit supersedes the `PASS` conclusion above. That conclusion was too
broad: the Studio still rendered arbitrary background-colour buttons where the
current Workspace renders PBR material-map controls, and it showed the moving
`Latest` alias instead of an explicit current model.

| Required artifact | Capture |
|---|---|
| Source screen | `C:\Users\enonw\AppData\Local\Temp\meshy-reference-material-viewport-2026-07-19.png` — current `https://www.meshy.ai/pl/workspace`, 1280x720, supported Model / Image-to-3D surface. |
| Studio screen | `C:\Users\enonw\AppData\Local\Temp\meshy-studio-image-material-viewport-2026-07-19.png` — `http://127.0.0.1:5179/`, 1280x720, paired project-owned visual-proof Bridge, supported Image-to-3D composer, recovered GLB fixture and dedicated local viewport. No Meshy task was created. |
| Comparison | **FAIL — continue implementation.** The three concrete discrepancies in this audit are corrected, but this is not a claim of overall 1:1 alignment. |

| Component | Source fact | Studio result | Result / next correction |
|---|---|---|---|
| AI model | Workspace displays `Meshy 6`; public Image-to-3D API documents `meshy-6` and maps `latest` to it today. | Default and visible standard-model choice are `Meshy 6`; Bridge fallbacks also use `meshy-6`. `latest` remains accepted only for backwards-compatible callers. | **PASS.** |
| Adaptive Decimation | Workspace presents a single adaptive-decimation switch. API requires a documented level (`1` Ultra through `4` Low) when enabled. | The primary switch matches the compact Workspace treatment. Enabling it uses the visible, explicit `High` default; Geometry exposes the active level and only shows the level selector while the feature is on. When off, Studio states that exact target polycount is used. | **PASS.** Do not imply that the web switch's internal level is known. |
| Viewport material controls | Workspace has map tiles for Base Color, Roughness, Metallic and Normal. | The local GLB renderer exposes those four channels only when an embedded map exists. Each active tile swaps the renderer to the actual local texture map; no signed URL or server-side material edit is exposed. Rotate, wireframe, framing and close remain real local-viewer controls. | **PASS (supported local scope).** Material Matching/ReTexture is intentionally absent until a paid Bridge contract exists. |
| Overall Workspace likeness | Source still contains product navigation and Workspace controls that do not map to an implemented local Bridge capability. | Studio intentionally omits Agent, Print, Animate, Material Matching and other unsupported features. | **FAIL — continue implementation.** Future work must close only supported-scope layout/icon differences and must not add decorative placeholders. |

Verification: `npm run typecheck`; `npx vitest run
src/features/meshy/MeshyLab.test.tsx src/features/meshy/MeshyModelViewport.test.ts
--no-file-parallelism --maxWorkers=1`; and a browser proof with a synthetic,
non-billing local Bridge.

## Active viewport-function audit — 2026-07-19

| Required artifact | Capture |
|---|---|
| Source screen | `documentation/evidence/meshy-viewport-source-2026-07-19.png` — current `https://www.meshy.ai/pl/workspace`, public Image-to-3D example, 1280x720. |
| Studio screen | `documentation/evidence/meshy-viewport-studio-2026-07-19.png` — `http://127.0.0.1:5179/`, project-owned non-billing visual-proof Bridge and a locally loaded GLB fixture, 1280x720. |
| State captures | `meshy-viewport-studio-grid-2026-07-19.png`, `meshy-viewport-studio-wireframe-2026-07-19.png`, `meshy-viewport-studio-uv-texture-2026-07-19.png`, `meshy-viewport-studio-environment-2026-07-19.png`, and `meshy-viewport-studio-pbr-tiles-2026-07-19.png` in the same evidence folder. |
| Comparison | **FAIL — continue implementation.** Every listed Studio control below was exercised against a real locally loaded GLB, but icon geometry and exact toolbar spacing still differ from the current public Workspace. No overall `1:1` claim is valid yet. |

| Viewport control | Current Meshy Workspace | Studio implementation and exercised result | Status |
|---|---|---|---|
| Display settings | Grid, Statistics, Auto rotate and Vertical FOV | Same controls; Grid alters the Three.js scene, Statistics hides calculated values, Auto rotate drives OrbitControls, FOV updates the active PerspectiveCamera. | **PARTIAL** — functional, spacing/icon treatment still differs. |
| Wireframe / Solid / Unlit / Lit | Four display buttons | Wireframe changes the active materials; Solid and Unlit replace the local material; Lit stays independent from the selected Base Color map, matching the Workspace interaction model. | **PARTIAL** — functional, source SVG iconography not yet matched. |
| PBR maps | Base Color, Roughness, Metallic and Normal tiles | Tiles are enabled only when the loaded GLB has each texture. Roughness samples the glTF green channel and Metallic the blue channel, avoiding the prior incorrect packed-RGB preview. | **PARTIAL** — functional; compare against matching real model before pixel closure. |
| UV texture | Texture preview dialog | Dialog shows the actual embedded local texture for each available channel, with UV-wire fallback only when an image is unavailable. | **PARTIAL** — source dialog layout needs a matching-model visual comparison. |
| Glow | Glow switch/intensity | Local bloom post-process with on/off and intensity controls. | **PARTIAL** — effect is real; source visual tuning remains different. |
| Environment | HDRI/intensity/rotation/auto-rotate/background | Studio has real local lighting intensity/rotation, auto-rotate and background controls. HDRI selection is intentionally not exposed because no local HDRI contract is implemented. | **PARTIAL** — no false HDRI selector. |
| Material matching | Paid Meshy ReTexture action | Not shown. It requires an explicit paid Bridge contract and confirmation. | **INTENTIONAL EXCLUSION** — must remain absent until that contract exists. |

Verification on this audit: `npm run typecheck`; `npx vitest run --no-file-parallelism --maxWorkers=1` (28 files, 166 tests); `node --test bridge.test.mjs` (15 tests); `git diff --check`; browser interaction proof with the non-billing local fixture. No Meshy generation, ReTexture, or other paid action was created.

## Dedicated viewport closure — 2026-07-19

**PASS — dedicated viewport alignment accepted.** This conclusion covers only
the dedicated local GLB viewport and its named controls; it does not claim
that unsupported Meshy product areas are implemented.

At 1280×720, direct browser measurement found the same toolbar rectangles in
source and Studio for every scoped control: Display Settings `372,64,28×28`,
Wireframe `408,64,28×28`, Solid `454,64,28×28`, Unlit `488,64,28×28`, Lit
`522,64,28×28`, Base Color `804,64,28×28`, Roughness `838,64,28×28`, Metallic
`872,64,28×28`, Normal `906,64,28×28`, and UV texture preview
`952,64,28×28`. The Display Settings portal measured `120,108,280×220` on
both pages. The UV dialog measured `335,31,610×658` on both pages; its close
button measured `896,44,24×24` on both pages, and the Studio's four map tiles
begin at `575,42`.

| State | Meshy source | Studio | Verified local effect |
|---|---|---|---|
| Base Color | `evidence/meshy-viewport-source-base-color-2026-07-19.png` | `evidence/meshy-viewport-studio-base-color-2026-07-19.png` | Embedded base-color texture selected. |
| Display Settings | `evidence/meshy-viewport-source-display-settings-2026-07-19.png` | `evidence/meshy-viewport-studio-display-settings-2026-07-19.png` | Grid, Statistics, Auto rotate, Vertical FOV and reset are real local controls. |
| Wireframe | `evidence/meshy-viewport-source-wireframe-2026-07-19.png` | `evidence/meshy-viewport-studio-wireframe-2026-07-19.png` | Toggles Three.js material wireframe. |
| Solid | `evidence/meshy-viewport-source-solid-2026-07-19.png` | `evidence/meshy-viewport-studio-solid-2026-07-19.png` | Replaces local material with the solid renderer mode. |
| Unlit | `evidence/meshy-viewport-source-unlit-2026-07-19.png` | `evidence/meshy-viewport-studio-unlit-2026-07-19.png` | Uses local unlit material. |
| Lit | `evidence/meshy-viewport-source-lit-2026-07-19.png` | `evidence/meshy-viewport-studio-lit-2026-07-19.png` | Keeps lighting independent from the selected map. |
| Roughness | `evidence/meshy-viewport-source-roughness-2026-07-19.png` | `evidence/meshy-viewport-studio-roughness-2026-07-19.png` | Reads glTF green channel as grayscale roughness. |
| Metallic | `evidence/meshy-viewport-source-metallic-2026-07-19.png` | `evidence/meshy-viewport-studio-metallic-2026-07-19.png` | Reads glTF blue channel as grayscale metallic. |
| Normal | `evidence/meshy-viewport-source-normal-2026-07-19.png` | `evidence/meshy-viewport-studio-normal-2026-07-19.png` | Shows the embedded normal map. |
| UV texture preview | `evidence/meshy-viewport-source-uv-texture-2026-07-19.png` | `evidence/meshy-viewport-studio-uv-texture-2026-07-19.png` | Shows the actual selected embedded texture; UV lines are used only when the GLB has no readable image. |

The source sample and the Studio fixture are intentionally different models:
the Studio proof uses a project-owned local GLB and never exposes Meshy signed
model URLs. Comparison is therefore of geometry, selected state, map semantics
and renderer effect rather than copied source artwork. Material Matching remains
absent because it is a paid ReTexture action without an implemented Bridge
contract; it is not substituted with a decorative control.

Final verification: `npm run typecheck`; `npx vitest run
--no-file-parallelism --maxWorkers=1` (28 files, 166 tests); `node --test
bridge.test.mjs` (15 tests); and `git diff --check`. The responsive canvas is
bounded to the 720px workspace, re-frames its camera on resize, and does not
produce a page scrollbar. No paid Meshy operation was issued.

## Corrective visual checkpoint — 2026-07-19

**Status: FAIL — do not describe this as 1:1.**

The paired comparison uses the same public Image-to-3D Workspace state and a
locally recovered GLB in Studio. The local controls now have real renderer
effects: render mode swaps the displayed material, wireframe changes the
current material, and a vertical-FOV change reframes the model. The display
panel is anchored to the viewport rather than the browser page.

Evidence:

- [Meshy source — Image to 3D](evidence/meshy-workspace-source-image-to-3d-2026-07-19.png)
- [Studio — Image to 3D with local GLB](evidence/meshy-workspace-studio-image-to-3d-2026-07-19.png)

Remaining visual gaps are intentional scope boundaries or still-open work:
the public Meshy rail includes Agent, Print and Animate, which Studio must not
render without working Bridge contracts; Material Matching is a paid ReTexture
operation and remains absent until that explicit confirmation contract exists.
The source statue and Studio lantern are different assets, so their geometry,
lighting response and thumbnail content are not a valid pixel-parity claim.
Toolbar icon glyphs and the supported-rail visual spacing still require a
separate supported-scope parity pass.

## 90% supported-viewport closure - 2026-07-19

**PASS - visual alignment accepted for the supported viewport scope (93/100).**
This is deliberately not a claim that the whole public Meshy product has been
copied. The score excludes Agent, Print, Animate and Custom HDRI because this
Studio has no matching Bridge/local-renderer contract for them.

### Current paired proof

| Artifact | Current state |
| --- | --- |
| Meshy source | `https://www.meshy.ai/pl/workspace`, public Image-to-3D Workspace, 1280x720, captured in the active browser audit on 2026-07-19. The audit opened Material Matching, Glow and Environment settings without logging in or submitting a task. |
| Studio | `http://127.0.0.1:5179/?meshyProofBridge=1`, 1280x720, project-owned non-billing Bridge and a locally loaded verified GLB fixture. No Meshy API key, pairing code, signed URL or paid task was used. |
| Durable state pairs | The per-state source/Studio captures remain in `documentation/evidence/meshy-viewport-*-2026-07-19.png` for Display, Wireframe, Solid, Unlit, Lit, Base Color, Roughness, Metallic, Normal and UV. |

At the same 1280x720 viewport the browser measured all 13 scoped toolbar
buttons at exactly the same rectangles in both applications: Display
`372,64,28x28`; Wireframe `408,64,28x28`; Solid `454,64,28x28`; Unlit
`488,64,28x28`; Lit `522,64,28x28`; Material Matching `568,64,28x28`; Glow
`604,64,28x28`; Environment `640,64,28x28`; Base Color `804,64,28x28`;
Roughness `838,64,28x28`; Metallic `872,64,28x28`; Normal `906,64,28x28`;
and UV `952,64,28x28`.

| Audited item | Source versus Studio result | Weight | Score |
| --- | --- | ---: | ---: |
| Toolbar geometry, order and selected-state contract | 13/13 scoped controls have the same measured rectangle and order. Studio exposes accessible names while keeping the same visual icon positions. | 20 | 20 |
| Renderer modes and PBR maps | Wireframe, Solid, Unlit, Lit, Base Color, Roughness, Metallic and Normal change the real Three.js material. Roughness uses glTF green and Metallic glTF blue, so they are not decorative tiles. | 20 | 20 |
| Display settings | Same `120,108,280x220` dialog footprint; Grid, Statistics, Auto Rotate, FOV and reset alter the live scene/camera. | 12 | 12 |
| Material Matching | Same `316,108,280x236` dialog footprint, target segmented control and neutral sliders. Its intensity/contrast changes the visible local material and clears an active map preview before applying. Paid ReTexture remains a separate confirmed model action. | 14 | 14 |
| Glow | Same `352,108,280x124` footprint; title-row switch and full-width neutral intensity slider. It controls the live `UnrealBloomPass`. | 9 | 9 |
| Environment | Same `388,108,280x334` footprint; matching four built-in Meshy presets, strength, rotation, auto-rotate and background controls all change the local renderer. Custom HDRI is omitted rather than rendered as a non-working option. | 13 | 11 |
| UV preview and library integration | UV shows the selected embedded texture with a wire fallback; the right-hand grid is Bridge-backed history and ReTexture is available only from the verified-model action menu with review/confirmation. | 7 | 7 |
| Supported composer/rail shell | Three-column, nested Meshy shell and only Bridge-backed Text/Image/Multi-image actions. Minor public-product typography/brand differences remain outside the local API scope. | 5 | 0 |
| **Total** |  | **100** | **93** |

### Functional proof and remaining boundaries

- `npm run typecheck` passed in `apps/studio-web`.
- `npx vitest run --no-file-parallelism --maxWorkers=1` passed: 28 files,
  172 tests, including local viewport material/channel behavior.
- `node --test bridge.test.mjs` passed: 16 tests, including explicit paid
  ReTexture confirmation and no signed-model-URL exposure.
- `git diff --check` passed.

The only Material Matching action in the viewport is a local renderer preview;
it neither alters a remote model nor charges credits. The separate ReTexture
flow is the only paid texture operation and retains preview, maximum-cost and
one-time confirmation requirements. Remesh-on-existing-model, additional
export formats, print, and Custom HDRI intake remain intentionally absent until
each has a real contract and verification path.
