# Fogbound Ghostly Visage V1 — ready for owner proof

- Test module file: `m2aghostdemo1.mod`
- Module name in Toolset: `Meshy2Aurora Fogbound Ghostly Visage V1`
- Exact Area name: `Meshy2Aurora Fogbound VFX Test V1`

Status: `ready_for_owner_proof`.

## What this candidate contains

The demo uses the last generated Creature, Fogbound Claw Guard, from the
canonical source:

- asset: `sample-3d/tlc-fogbound-claw-guard-h1-p300k-v1`;
- source: `source-death-continuous.glb`;
- source SHA-256:
  `ae32c5028ece19dad24c121f9993767a7fdd87616401a94a3fb71f42427364af`;
- source geometry: 297,190 triangles;
- source animation clips: 9;
- emitted native Creature animation profile: 42 animations, including the
  preserved source combat motions `ca1slashl` and `ca1slashr`.

All MDL, texture, `appearance.2da`, HAK and MOD assets were produced offline by
the current Meshy2Aurora pipeline. The output passed the Creature Basis V3
gate: `GLTF_POSITIVE_Z_TO_AURORA_POSITIVE_Y`, determinant `+1`. Engine-facing
remains an owner visual verdict.

The Creature uses appearance row `15100`, `ModelType L`, model resref
`m2aghostcre1`, texture resref `m2aghosttex1`, and UTC resref
`m2aghostutc1`. It is placed at `(10.0, 14.5, 0.0)`, facing `(0.0, -1.0)`,
directly in front of the player start `(10.0, 10.0, 0.0)` which faces
`(0.0, +1.0)`.

## Exact VFX binding

The demo applies the base-game constant
`VFX_DUR_GHOSTLY_VISAGE_NO_SOUND = 478` permanently to `OBJECT_SELF` from the
Creature `ScriptSpawn` event. Row 478 in `visualeffects.2da` resolves through
`progfx.2da` row 402 to `AlphaLightBlue`. It intentionally has no repeating
Ghostly Visage sound.

The module embeds both exact script resources under resref `m2aghostsp1`:

- NSS SHA-256:
  `80e11e6a90511c2aff65362296968f30c8f5b621ff96bfd393a1a7714dda4578`;
- NCS SHA-256:
  `5a99ae5ed86472c7bbd222a496368758849a65dccfc63c10c01b85f73fa29cca`.

The NCS was compiled with `nwnsc 1.1.5`, strict compatibility enabled and
optimization enabled. Disassembly readback contains literal effect ID `478`,
`EffectVisualEffect`, duration type `2` (`DURATION_TYPE_PERMANENT`), object
constant `0` (`OBJECT_SELF`), and `ApplyEffectToObject`.

The MOD readback independently requires:

- the exact NSS and NCS resources;
- `ScriptSpawn=m2aghostsp1` in the placed GIT Creature;
- `ScriptSpawn=m2aghostsp1` in its module-local UTC;
- one passive monster fixture with appearance row `15100`;
- no module-local UTI or held-item dependency.

## Frozen artifacts and native installation

- Project MOD:
  `C:\Projects\meshy2aurora\proof-output\creature-ghostly-visage-v1\m2aghostdemo1.mod`
  — 15,194 bytes — SHA-256
  `fe8d19b35445d1d08d79a75e0558ed7a2457aa4ed1469935f8c2f69bb9b8d8f6`.
- Project HAK:
  `C:\Projects\meshy2aurora\proof-output\creature-ghostly-visage-v1\m2aghosthak1.hak`
  — 44,113,346 bytes — SHA-256
  `45f2ae742a5076bb12711a8b1e6d86d38b9fcbe081e8f2eb41fa277022e72adb`.
- Installed MOD:
  `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aghostdemo1.mod`.
- Installed HAK:
  `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aghosthak1.hak`.

Both native destinations were absent before copy. Their post-copy SHA-256
values are byte-identical to the project artifacts above. No existing native
file was overwritten or replaced.

The complete offline packet, including `materialization.json`, reports,
standalone NSS/NCS, MDL, texture and `appearance.2da`, is in
`proof-output/creature-ghostly-visage-v1`.

## Owner proof criteria

1. Open `m2aghostdemo1.mod`; confirm the module and Area names listed at the
   top of this handoff.
2. Start the Area. The Fogbound Claw Guard should be directly ahead of the
   player and should face the player rather than move sideways or backwards.
3. Confirm the complete Creature is rendered and covered by the light-blue,
   semi-transparent Ghostly Visage treatment.
4. Confirm the effect remains present while the Creature idles and that no
   repeating Ghostly Visage sound plays.
5. Report the Toolset and NWN results separately as `visible`, `not_visible`,
   or `not_tested`; a screenshot is useful but the owner visual verdict is the
   completion authority.

No Aurora Toolset or NWN session was started, adopted, controlled or captured
during candidate preparation.

## Offline verification

- targeted red/green Ghostly Visage module test: PASS;
- full `m2a-core` library suite: 119 passed, 0 failed, 3 ignored because they
  require separate local runtime/corpus fixtures;
- Rust formatting check: PASS;
- exact pipeline materialization and MDL/2DA/MOD semantic readback: PASS;
- native MOD/HAK byte-identity verification: PASS.
