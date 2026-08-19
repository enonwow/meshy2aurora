# Hextech Shotgun owner concept

Status: documentation-only comparison reference for implementation review.

## Asset identity

- file: `hextech-shotgun-concept.png`
- SHA-256: `cfa31ccea74b53b1e0c55182ec3e1ed4a2072041b433009a448b7717509bd8f9`
- byte length: `1,180,762`
- provenance: concept art supplied by the project owner in the Item authoring task

Owner clarification 2026-08-17: this image is for agent-side visual comparison
only. Studio must not import, render or bundle it, and the runtime composition
contract must not depend on its bytes or hash.

The concept is a visual reference, not an Aurora runtime payload. It must not
be copied into a HAK/MOD or treated as a substitute for the canonical Meshy
sources under `sample-3d/tlc-hextech-shotgun-parts-v1`.

## Hextech Shell concept

- file: `hextech-shell-concept-v1.png`
- SHA-256: `4af93ef0b2b07d85bb0fc5950e5da06c310afaf4f7bb3673cc2037301867d3e1`
- byte length: `2,035,109`
- provenance: Codex ImageGen concept selected by the project owner on
  2026-08-17 as the lower-detail projectile reference
- Meshy source asset: `sample-3d/tlc-hextech-shell-s1-p1500-v1/source.glb`

This concept remains documentation-only. Its approved use is as the exact
image-to-3D source reference recorded by the Meshy asset manifest; Studio and
the eventual NWN package must use the generated model and converted runtime
resources, not render or bundle this PNG as product UI content.

## Canonical source relocation amendment — 2026-08-19

The local `source.glb` recorded by the original generation and proof packets
was relocated from the registered `items-agent-remediation` worktree to the
single canonical source root:

`C:\Projects\meshy2aurora\sample-3d\tlc-hextech-shell-s1-p1500-v1\source.glb`

The relocation was byte-identical: 9,432,832 bytes, SHA-256
`f4f8c41d445036b1364a16f779b17b21baa4f49caff356d77f6d8b8e1d9727ec`.
The immutable generation and candidate packets retain their capture-time paths
and remain authoritative for historical provenance.
