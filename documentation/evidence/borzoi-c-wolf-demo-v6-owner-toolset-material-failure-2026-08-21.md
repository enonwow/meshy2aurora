# Borzoi c_wolf demo V6 — owner Toolset material failure

Date: 2026-08-21\
Status: `OWNER_MATERIAL_QUALITY_FAILURE_RECORDED / V7_DIRECTLY_AUTHORIZED`

## Exact V6 candidate

- MOD: `m2aborzmod6.mod`, SHA-256
  `5330fa8da1a8570ae0342a66c719715ec52fbd9f1bdde6cd0a8bc5fc9a134e79`;
- module name: `Meshy2Aurora Borzoi c_wolf Demo V6`;
- Area: `Meshy2Aurora Borzoi Test Area V6`;
- HAK: `m2aborzhak6.hak`, SHA-256
  `90f06145306de60b2d4decd7167606a7f1d0ee0e2f331bdb713ea94fd91cb604`;
- MDL: `m2aborzcre6.mdl`, SHA-256
  `eba9c0ab085ad3f2243be0b1c419d76cc71d31a31c4056c62b16454b69a18203`;
- diffuse TGA: `m2aborztex6.tga`, SHA-256
  `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1`;
- Appearance row: `848`;
- placement: `(10.0, 14.5, 0.0)`.

## Owner evidence and verdict

The owner supplied a fresh Toolset screenshot of exact module
`m2aborzmod6.mod` showing the V6 Borzoi rendered in the expected Area. The
model is visible and its inherited `c_wolf` animations were separately
reported as correct. The owner rejected material quality because dotted and
short broken lines interrupt the coat and expose the terrain through the
render surface.

- durable screenshot:
  `documentation/evidence/borzoi-c-wolf-demo-v6-owner-toolset-material-failure-2026-08-21.png`;
- screenshot bytes: `2588073`;
- screenshot SHA-256:
  `ab540e84edf1ab8d9d0c463ed1445792d671a75a9fb801533d1b17005e237cb5`.

Toolset axes:

- `modelVisibility = visible`;
- `proofCompleteness = verified` for the reported material defect;
- `qualityVerdict = failed`.

## Diagnosed cause

Facts:

1. The owner-selected Meshy GLB declares `doubleSided=true`.
2. V5 material compilation preserved that source semantic as
   `mtr.twosided`.
3. The V6 `ClassicDiffuseMotion` route bypassed the shared material compiler
   and structurally packaged only MDL, diffuse TGA and `appearance.2da`.
4. V6 therefore removed the required two-sided material semantic together
   with the unsafe V5 normal/specular/tangent stack.
5. Backface culling removes reverse-facing triangles in the thin and
   fragmented fur geometry. The terrain visible in their place produces the
   reported dotted/broken-line artifact.

Rejected causes for this exact symptom: animation, corrupt TGA payload, UV
seam and texture filtering. The V5 client crash remains a separate runtime
problem; this record does not claim that `twosided` caused that crash.

## Direct owner authorization and minimal V7 delta

After the audit, the owner directly requested generation of a corrected demo
without the broken lines. This authorizes exactly one V7 candidate despite V6
remaining model-visible.

V7 must preserve V6 source GLB, fitted V5 rig, weights, geometry, hierarchy,
supermodel contract, motion oracle, diffuse TGA content, Area layout and
placement. The only functional material delta is:

- restore source `doubleSided=true` as a minimal canonical MTR containing
  `twosided 1`;
- bind that MTR in the MDL material slot;
- keep render hint non-normal-mapped;
- keep tangent stream count at zero;
- package no normal map, specular map or TXI resource;
- fail closed when the source requires two-sided rendering but the MTR or MDL
  material binding is absent or differs after readback.

V6 remains immutable and must not be overwritten or repackaged.
