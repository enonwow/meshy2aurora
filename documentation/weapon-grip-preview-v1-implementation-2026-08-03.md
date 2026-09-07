# Weapon Grip Preview V1 — implementation status

Date: 2026-08-03\
Status: implemented and verified offline\
Scope: Creature product pipeline and Converted Model viewport in Studio

## Outcome

Studio can now display an explicit calibration sword at the exact `rhand` or
`lhand` hook reconstructed from the generated binary MDL. The preview follows
the hand during animation playback and exposes the hook parent, local position,
local orientation, selected hand, and optional XYZ hook axes.

The rendered sword is deliberately labelled `CALIBRATION PROXY`. It is a
project-owned procedural shape used to inspect grip position and rotation. It
is not a copied NWN retail weapon and must not be interpreted as exact evidence
of a particular stock item model.

## Implemented contract

- preview modes: `Off`, `Right hand`, `Left hand`;
- default mode: right hand when an exact `rhand` hook exists, otherwise left
  hand, otherwise off;
- the proxy is parented directly under the reconstructed binary-readback hook;
- the sword uses the canonical item longitudinal axis and inherits every hand
  transform produced by animation playback;
- optional hook axes make position and orientation errors visible;
- missing or duplicate requested hooks fail closed;
- the Core product report retains `weaponAnchorAuthoring`, including both
  authored hook matrices, node identities, parent identities, calibration and
  the proof that the hooks have zero skinned vertices;
- Studio parses that report strictly and compares the authored matrix against
  the matrix reconstructed from the generated binary MDL;
- a mismatched node, parent, weight count or matrix is rejected instead of
  showing an unaudited preview.

## Evidence chain

```text
Core authors rhand/lhand
  -> binary MDL writer
  -> Core weaponAnchorAuthoring report
  -> WASM reportJson
  -> strict Studio canonical projector
  -> binary-MDL readback hierarchy/controllers
  -> authoring/readback parity gate
  -> procedural calibration sword in Converted Model
```

This means the visible preview is based on output data, not on the original GLB
joint guessed independently by the UI.

## Verification completed

- Studio TypeScript typecheck: passed;
- `AuroraReadbackViewport` and canonical-result tests: 37/37 passed;
- exact proxy parenting, hand-motion inheritance, Off/Right/Left switching,
  missing/duplicate hook rejection and matrix mismatch rejection are covered;
- Core product integration test on the local canonical Meshy humanoid: passed;
- Core weapon-anchor unit tests: 5/5 passed;
- complete native `m2a-wasm` library suite: 39/39 passed;
- production WASM build: passed;
- local browser smoke test: Studio loaded at `127.0.0.1:5175` with no browser
  console errors or warnings.

## Exact production-browser validation amendment — 2026-08-03

The production Studio build was run on
`sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1/source-death-continuous.glb`
(297,190 source triangles) with the canonical `appearance.2da`. The generated
binary-MDL readback reported:

- hook `rhand`, parent `RightHand`;
- local position `(-0.00897, 0.01797, 0.08099)`;
- local orientation `(-0.69269, 0.44461, 0.44461, 0.35330)`;
- the calibration sword grip at the hand hook in the neutral pose and the
  sword following the hand during `ca1slashl` playback;
- no authoring/readback parity warning.

A read-only retail corpus check of `wswss_b_011`, `wswss_m_011` and
`wswss_t_011` confirms that the stock short-sword is assembled along local
`+Y`: the blade part extends from about `Y=0.093` to `Y=0.604` after its node
translation, while the pommel-side geometry extends into negative `Y`.
Therefore the preview proxy keeps the same `+Y` blade convention; it must not
apply an arbitrary 180-degree visual flip. A pose in which that correctly
attached blade still follows an unarmed wrist angle is an animation/motion-pack
limitation, not an attachment-position correction.

The parity validator intentionally does not compare the Core `anchorNodeId`
with binary `NodeReport.number`. The former identifies the node in canonical
IR, while the binary writer assigns a separate depth-first part number. Hook
name, parent name, zero weighted vertices and the complete local matrix remain
strictly compared, so an actual attachment mismatch still fails closed.

## Deliberate limitations

- V1 previews grip placement with a calibration proxy, not the exact inventory
  item selected in a Toolset UTC;
- V1 does not import or redistribute retail NWN item geometry;
- the preview proves attachment transform and animation following, but it does
  not prove that a particular UTI is equipped in a module;
- final Aurora Toolset/NWN visual proof remains owner-owned and was not run by
  the implementation agent;
- this change does not create a new MOD, HAK, model resref or proof candidate.

An exact item-preview feature can be added later through an explicit,
provenance-safe item-resource input. It must remain separate from this
calibration contract so a proxy is never presented as a retail weapon.

## Outward-blade V4 correction amendment — 2026-08-03

The production V3 browser proof established that the handle intersected the
right hand but the blade pointed back toward/across the forearm. The earlier
structural parity result was correct; the Meshy H1 hook basis itself still had
the wrong longitudinal direction for native item geometry.

V4 changes the actual Core-authored hook transform, not just Studio rendering:

- calibration is now `MESHY_H1_NATIVE_ITEM_HOOK_OUTWARD_BLADE_V4`;
- the existing bind-normalized local hook basis receives a deterministic
  180-degree rotation around hook-local Z;
- local position remains `(-0.00897, 0.01797, 0.08099)`;
- the exact production binary-MDL readback reports local orientation
  `(-0.44461, -0.69269, -0.35330, 0.44461)`;
- the stock item/proxy blade still extends along its own local `+Y`; V4 corrects
  the hook coordinate frame that receives it;
- authoring/readback parity remains strict and passed without a warning;
- source and converted triangle counts both remain `297,190`.

The production web packet is preserved at
`artifacts/diagnostics/weapon-grip-web-proof-v4/proof.md`. Its clean and
axes-enabled captures show the blade extending outward from `rhand` rather than
through the forearm. This closes attachment position and orientation in the
binary-readback web lane. It does not implement a closed-fist weapon animation,
equip a specific UTI, or replace the owner-owned Aurora Toolset/NWN proof.

## Skin-weighted palm-center V5 amendment — 2026-08-03

The final sentence of the V4 amendment was too strong for position accuracy.
V4 fixed orientation, but a subsequent exact inverse-bind audit showed its
fixed-ratio pivot at approximately the 5th percentile of the hand's
longitudinal surface and outside the 95th percentile on local Z. It was still a
wrist-derived estimate copied from one retail rig, not a measurement of the
selected Meshy model's palm.

V5 replaces that position policy systemically:

- hand vertices require at least `0.5` summed influence to the semantic hand;
- spatial welding removes seam-duplication bias;
- the coordinate-wise median of at least eight unique bind-space samples is
  the pivot;
- the blade frame follows the measured wrist-to-palm direction and remains a
  proper Z-up rotation;
- fixtures without enough hand surface retain the audited V4 fallback instead
  of producing an unaudited value;
- the production `+Z` source-forward readback reports right position
  `(0.01083, 0.00014, 0.09678)` and left position
  `(-0.00937, 0.00126, 0.10334)` after Aurora basis conversion; other explicit
  source-forward profiles correctly produce different local matrices for the
  same measured surface point.

The production binary-readback packet is
`artifacts/diagnostics/weapon-grip-web-proof-v5/proof.md`. V5 closes the generic
hook's geometry-relative palm placement. Exact visual identity with a selected
stock item still requires that item's actual geometry/pivot in the preview, and
natural finger closure still requires a weapon-aware hand pose.

## Owner Toolset correction to V5 — 2026-08-03

The previous sentence claiming that V5 closes generic hook placement must not
be read as closing stock-item orientation. Owner proof of exact
`m2aweapdemo5.mod` showed the model and stock sword, but the sword was visibly
twisted incorrectly. The candidate therefore has `modelVisibility=visible`
and grip `proofCompleteness=failed`.

Confirmed code-level gaps:

- the Studio object is a labelled procedural `CALIBRATION_PROXY`, not the
  assembled `nw_wswss001` geometry and pivot;
- authoring/readback parity only proves that Studio and binary MDL contain the
  same hook matrix; it does not prove that this matrix is correct for a retail
  item;
- V5 constrains weapon local `+Y` using wrist-to-palm direction, but resolves
  roll with a world-up seed instead of a measured palm plane or native item
  hook basis;
- the geometry-supported V5 branch constructs a new frame and does not apply
  the explicit V4 correction matrix, so the packet statement “V4 retained” was
  inaccurate.

No arbitrary 90/180-degree patch is authorized by this observation alone. The
safe next implementation step is an Aurora-first calibration against the full
native hand-hook and assembled weapon basis, followed by an exact-item or
explicitly basis-equivalent preview. Until then the Studio view is diagnostic,
not pixel-perfect stock-weapon proof.

## V6 native-basis remediation — 2026-08-03

V6 preserves the geometry-measured V5 palm center but replaces the V5
world-up-derived rotation with the complete audited native item basis already
used by the V4 fallback. Position and rotation are therefore no longer derived
by two conflicting frame policies. The geometry-supported and fallback paths
share one rigid orientation contract.

Studio now labels its overlay `NWN SHORTSWORD BASIS PROXY` and displays
`Not exact item geometry or Toolset proof.` The preview remains strict binary
MDL readback, but no longer suggests that procedural proxy geometry is an
exact rendering of the selected NWN inventory item.

Implementation and offline gates are recorded in
`documentation/evidence/creature-weapon-grip-v6-offline-remediation-2026-08-03.md`.
No V6 proof artifact was materialized by this implementation turn.
