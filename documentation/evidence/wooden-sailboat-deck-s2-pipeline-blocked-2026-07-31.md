# Wooden sailboat S2 — pipeline intake blocker

Date: 2026-07-31

## Scope

The owner supplied a new, independent Meshy GLB and requested an unchanged
end-to-end Placeable conversion followed by a demo module. The exact source was
registered at:

`C:\Projects\meshy2aurora\sample-3d\wooden-sailboat-deck-s2-textured-v1\source.glb`

No geometry, texture, UV, material, hierarchy or scale edits were made.

## Immutable source identity

- original filename: `Meshy_AI_Wooden_Sailboat_Deck__0731170742_texture.glb`
- byte length: `98546920`
- SHA-256: `5085db9399eb7d30ddcb6c65b51748f4cabc5ed10cdf7ba912cdf2ef62e1dabe`
- generator: `meshy-scene`
- meshes / primitives: `1 / 1`
- materials / embedded textures: `1 / 4`
- indexed triangle count: `1971350`
- accessor dimensions: `1.9014360309 x 1.3827780485 x 0.8555000126`

The canonical copy was hash-verified byte-identical with the owner-provided
download before the product gate was executed.

## Product gate result

The current browser-product WASM build executed `inspectGlbJson` against the
canonical bytes and returned:

```json
{
  "schemaVersion": 1,
  "code": "M2A-GLB-INPUT-LIMIT-EXCEEDED",
  "message": "input length 98546920 exceeds 67108864"
}
```

This is a deterministic intake failure, not a Toolset/NWN visual verdict. The
source also independently exceeds `AURORA_MODEL_TRIANGLE_BUDGET_V1 = 300000`
by `1671350` triangles. The project contract blocks totals above that limit and
does not authorize silent decimation or geometry deletion.

## Result and required next input

No MDL, texture conversion, HAK, MOD, native installation or live Toolset/NWN
session was produced or started. A byte-preserving end-to-end demo cannot be
built from this exact payload under the active product limits.

The required replacement export is one GLB that is both:

- no larger than `67108864` bytes (64 MiB); and
- no larger than `300000` triangles, preferably approximately `100000` as
  originally selected for this ship test.

Embedded PBR textures may remain, but a 2K texture export is recommended when
needed to keep the complete GLB below the input-size boundary. The next run
must use the replacement as a separately hashed owner-provided source; the
blocked payload remains immutable evidence and must not be overwritten.

## Amendment — owner-approved exact-source exception

Later on 2026-07-31 the owner explicitly instructed the pipeline to bypass the
current intake block for this new model. The exception was bound to SHA-256
`5085db9399eb7d30ddcb6c65b51748f4cabc5ed10cdf7ba912cdf2ef62e1dabe` and
did not change the default 64 MiB or 300000-triangle product limits.

The exception exposed that `1751244` of the `1971350` declared source
triangles were degenerate. Standard static-model sanitation removed those
invalid faces without manual modelling and produced `220106` Aurora-safe
triangles. The completed candidate is recorded separately in
`wooden-sailboat-deck-placeable-v1-ready-for-owner-proof-2026-07-31.md`.
