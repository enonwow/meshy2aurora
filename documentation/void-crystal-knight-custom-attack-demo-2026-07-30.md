# Void Crystal Knight — custom attack and self-playing demo

MOD: `m2c7a58c250ba516.mod`  
Toolset module name: `Meshy2Aurora procedural humanoid proof`  
Area: `Meshy2Aurora M0 binary vertical-slice area`

Date: 2026-07-30  
Branch/worktree: `animation` / `C:\Projects\meshy2aurora\.worktrees\animation`

> Amendment 2026-07-30 — audit correction
>
> This immutable packet predates the corrected runtime-demo contract. It
> deliberately replaced `cpause1` with `vck_showcase` to obtain automatic
> playback. That is no longer an accepted product strategy: `cpause1` must
> remain the production idle, while the authored attack is routed to the
> generic melee Base 42 variants `ca1slashl`, `ca1slashr` and `ca1stab`.
> Therefore this packet may be used only for the pending owner verdict on its
> exact installed lineage. It is not evidence that the corrected attack-demo
> route works in Aurora/NWN. No replacement MOD/HAK was generated, because the
> model-iteration gate still records this exact candidate as
> `modelVisibility=not_tested`, `proofCompleteness=missing`.

## Status

The first animation candidate that preserved all source triangles,
`m27ee1666f686a2f`, was reported by the owner as not showing the authored
attack. Its MDL contained changing controllers, but the demo exposed the
attack only through the combat-managed `ca1slashl` state and therefore did not
demonstrate the motion by itself.

The historical candidate attempted to make motion judgeable by using this
demo-only contract:

- `vck_cryslash` remains the real attack assigned to `ca1slashl`;
- a separately authored and kinematically distinct `vck_showcase` clip was
  assigned to `cpause1`;
- the behavior oracle confirms that `cpause1` and `ca1slashl` are distinct;
- the source GLB, geometry, UVs, materials and skinning remain preserved.

Current proof state:

- `modelVisibility = not_tested`
- `proofCompleteness = missing`
- `ready_for_owner_proof = true`
- Aurora Toolset and NWN were not started or controlled by the agent.

## Exact source

- Asset: `sample-3d/void-crystal-knight-h1-v1/source.glb`
- Source SHA-256:
  `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7`
- Source geometry: 24,834 vertices / 19,704 triangles
- Source animation inventory: `cpause1`, `cwalk`, `crun`
- Detached accessory mode: `AUTO`

## Animations authored in Meshy2Aurora Studio

### Real attack

- Authored clip: `vck_cryslash`
- Authored ID:
  `authored-eff6b4bd-d513-4495-8aef-2e2aabf19308`
- Source kind: `PROCEDURAL_TEMPLATE`
- Template: `ROOT_TRANSLATION_PULSE`
- Duration: 1.00 s
- Tracks: 48
- Keyframes: 68
- Aurora Base 42 assignment: `ca1slashl`
- Standalone custom output: `vck_cryslash`
- Binary readback: 48 decoded controllers / 10 changing controllers
- Motion SHA-256:
  `4b4fc7d81d2c22ee54c9642a231494ea1eaa139218346729ab7371186f3f2407`
- Generated gameplay marker: `hit@0.500s`

### Self-playing showcase

- Authored clip: `vck_showcase`
- Authored ID:
  `authored-532cfbd7-2a14-4614-b262-103f15121996`
- Source kind: `PROCEDURAL_TEMPLATE`
- Duration: 1.00 s
- Tracks: 48
- Keyframes: 69
- Aurora Base 42 assignment: `cpause1`
- Standalone custom output: `vck_showcase`
- Binary readback: 48 decoded controllers / 11 changing controllers
- Motion SHA-256:
  `5eda969ad61a2d4299c7eb4b29da67b430dec792616a7ee24bc2ddfea45fd662`

The showcase is a historical demo-only idle override. It is not used as the
gameplay attack state and does not replace `ca1slashl`, but the audit now
rejects any `cpause1` override as an inaccurate runtime demonstration.

## Current offline build evidence

Studio project:
`d744d6c5-3986-45e0-aec8-a952f573b964`, revision 82.

- [x] Source GLB unchanged
- [x] 24,834 source vertices -> 24,834 binary-MDL vertices
- [x] 19,704 source triangles -> 19,704 binary-MDL triangles
- [x] 3 source clips -> 44 binary-MDL clips
- [x] `ca1slashl` and `vck_cryslash` are 1.00 s attack outputs
- [x] `cpause1` and `vck_showcase` are 1.00 s showcase outputs
- [x] Both authored clips are `VALID`
- [x] Canonical animation readback is `MATCH`
- [x] `essentialStatesDistinct = true`
- [x] `behaviorCandidateEligible = true`
- [x] Skin animation conformance is complete
- [x] 5 spatial components audited, 4 detached accessories stabilized,
  341 vertices changed

## Product fixes implemented

- [x] V5 uses exact finite non-collinear geometry sanitation and exact H1
  profile derivation.
- [x] Animation Studio V5 selects exact geometry conversion for the main build
  and source-animation projection.
- [x] Studio readback reconciliation trusts canonical core evidence for
  materialized clip naming, roots and generated events.
- [x] Project recovery removes orphaned authored-Custom references and stale
  Base 42 assignments.
- [x] Duplicating a procedural authored clip preserves its valid
  `PROCEDURAL_TEMPLATE` lineage instead of corrupting it into an incomplete
  `SOURCE_CLIP_COPY`.
- [x] Release WASM is optimized with `wasm-opt -Oz --converge`.
- [x] The raw 4,000,000-byte WASM limit remains unchanged.
- [x] The gzip guard is 1,425,000 bytes; measured release output is
  1,422,230 bytes.

## Verification gates

- [x] Canonical workspace assertion — PASS
- [x] Canonical Meshy asset-layout assertion — PASS
- [x] Duplicate-lineage regression — 12/12 PASS
- [x] Studio typecheck — PASS
- [x] Studio tests — 371 passed, 3 skipped
- [x] Production Studio build — PASS
- [x] Bundle and WASM budgets — PASS
- [x] `git diff --check` — PASS

## Current demo handoff

- MOD: `m2c7a58c250ba516.mod`
- MOD SHA-256:
  `97e11b3a635e96e9856bf2151aae77996df5c5dde1ff43633c40d8f87f8af09f`
- HAK: `m2c7a58c250ba516.hak`
- HAK SHA-256:
  `e4f46374d2811631acda5c7dcd7ef6c56973bcb10238f108adb87bae6d24a9db`
- MDL SHA-256:
  `e899b52bc138d753877f1233754a5e7ad234715410662b65ab7513839e8103bc`
- Model/creature/Area/HAK resref: `m2c7a58c250ba516`
- Appearance row: 15101
- Creature display name: `Meshy procedural humanoid`
- Creature position: `[10.0, 14.5, 0.0]`
- Creature facing: `[0.0, -1.0]`
- Player entry: `[10.0, 10.0, 0.0]`
- Project packet:
  `artifacts/void-crystal-knight-attack-showcase-demo-2026-07-30`
- Native MOD:
  `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2c7a58c250ba516.mod`
- Native HAK:
  `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2c7a58c250ba516.hak`
- Download manifest SHA-256:
  `cc6944866e138b56725344ad8313d3ecc0c1bcf229807884f2e96ab9ffecc774`
- Inspection SHA-256:
  `a9a8e2888f5848ef730b28192909c6c8951c3cb9798e32f1a6db61b569cc89f8`
- Conversion manifest SHA-256:
  `07bdc417db997c88b403a264fc1e60320a3117814961b242ef2964be7f111754`
- Summary SHA-256:
  `099a6ec580f520143575f940c6e498dc735d994e34be72d7621e9ec40993c40b`

All seven files match the hashes exposed by the Studio download surface. The
native MOD and HAK destinations were absent before copy and match their
canonical project sources after copy.

## Rejected lineages — do not use for proof

### `m27ee1666f686a2f`

This candidate preserved 19,704 triangles and contained changing animation
controllers, but the owner reported that the demo did not show the attack.
It relied on combat-managed playback and is superseded by the self-playing
showcase candidate.

- MOD SHA-256:
  `2fcbd4f49beccd6926c29521a1074b331d2c169b5d61049a8ab608fb96c97d4c`
- HAK SHA-256:
  `518f6d55d6882114aec28ad0f3650c90a4704a3ae9c69a97b46637bef513070b`
- Packet: `artifacts/void-crystal-knight-attack-demo-v2-2026-07-30`

### `m2956322c98a83e1`

The first app-generated demo used the legacy sanitizer and removed 26
microtriangles.

- Observed geometry: 24,799 vertices / 19,678 triangles
- MOD SHA-256:
  `532e77da7494e01a2ef7e2001027da7e25db1159e8f889b2beeba09f33def304`
- HAK SHA-256:
  `c7d0e2db55019db04c08b1a14a9a03eaba963604bc1e3e0116dccda67daba313`
- Packet: `artifacts/void-crystal-knight-attack-demo-2026-07-30`

Old native files were not overwritten, renamed or removed.

## Remaining owner-gated stage

- [x] Studio authored a real attack and a distinct self-playing showcase.
- [x] Studio materialized one fresh MOD/HAK demo.
- [x] Exact sources and absent native destinations were resolved and hashed.
- [x] MOD/HAK were installed with no-clobber semantics and verified
  byte-for-byte.
- [x] Handoff leads with the exact MOD filename, module display name and Area.
- [ ] Owner performs the final Aurora Toolset and NWN visual proof.
