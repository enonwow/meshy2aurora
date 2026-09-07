# Borzoi: Blender + Neverblender trial, 2026-09-07

Status: **DRAFT — not a correct finished NWN creature.** The user-visible lifted-paw deformation remains. Technical readback is not visual acceptance or native proof.

## Input and environment

- Canonical source: sample-3d/borzoi-tripo-dc22ecb0/source.glb, SHA256 dc22ecb0561fc10773e1a156b0d126225a15cdfb67cd7c0999a965a4f1d59689.
- Reference: C:/Projects/the last city/assets/creatures/borzoi/reference/c_wolf.mdl, existing cleanmodels ASCII reference, SHA256 168984607f739350c96fdc896b705011e9b726d8567034014eb1fbb2cebf389b. Imported with Neverblender including all 42 reference animations.
- Existing Blender 4.0.2 exited with 0xC0000005 even for --version. Blender 5.1 lacks APIs used by this Neverblender release.
- Working isolated runtime: proof-output/third-party-tools/blender-3.6.23-local/blender-3.6.23-windows-x64/blender.exe. Downloaded from the official Blender archive; archive SHA256 e3296eba7eab32c2e5182459ec7614af32224eee2bd32c9d0a08ffd751c54f3b was verified.
- Neverblender remains in its existing external-tool installation. No addon code was copied into the product or modified. All child processes used windowsHide:true, redirected output, and observed exit codes.

## Findings

1. The Tripo reconstruction did not preserve the exact rig placement from the concept. Front lower-leg centers were approximately 0.07–0.09 from the center plane versus c_wolf joints around 0.115–0.13. Rear lower legs also needed longitudinal shifts, up to approximately 0.074 scene units in sampled sections.
2. Direct pseudo-bone-to-armature conversion introduced different rest axes for non-identity transforms, notably the tail. The trial instead evaluates imported MDL world matrices and derives pose matrices relative to the true bind matrices. Blender pose matrix error was below 0.000010 across the sampled walk, run and idle frames.
3. The original NWN wolf already contains strong carpal and paw rotations. It uses rigid mesh pieces. A continuous Tripo surface with blended weights behaves differently around those joints. Neither renaming bones nor successful export establishes deformation quality.
4. Anatomical weight fields, rigid cuts, joint caps, reconstructed lower shafts and internal support geometry were tried and visually inspected. They introduced seams, visible supports or unnatural surfaces and are NOT the delivered variant.
5. The best appearance-preserving trial uses Blender bone heat on a watertight 44,038-vertex proxy and transfers those weights to the detailed surface. Four normalized influences are retained. Seventy-four vertices outside supported transfer received weights from the nearest already-weighted surface; there are no unweighted vertices. The proxy is removed from the delivered scene.
6. This improves the torso/fur behavior but **does not fully fix the raised feet**. Flattening/curling and the under-thigh surface still need a deliberate topology/shape correction. Do not present this trial as a successful finished model or repeat numerical weight optimization and declare the problem solved.

## Delivered draft

Folder: artifacts/creatures/borzoi-neverblender-draft-20260907

- borzoi-cwolf-draft.blend: editable owned mesh and rig; local reference actions cwalk, crun, cpause1.
- borzoi-cwolf-walk.glb: owned skinned mesh with only the cwalk preview clip.
- c_brznbdraft.mdl: actual Neverblender ASCII export, setsupermodel c_brznbdraft c_wolf. No copied retail meshes or animation clips in the MDL.
- brznb_body.tga: actual 4096×4096 32-bit TGA, derived from the user's original diffuse.
- cwalk.mp4 and crun.mp4: five cycles rendered in Blender, not NWN footage.
- mdl-readback-walk.png and readback-report.json: reimported MDL checked against the Blender source.

Roundtrip checks: 25,830 vertices, 20,534 triangles, 29 carrier nodes plus model root and owned skin; reference-node error about 0.0000114 from ASCII precision. Five walk pose readbacks had maximum surface error below 0.000263 scene units. These only establish export fidelity.

## Next work

Inspect and remodel the carpal/metacarpal and rear ankle/paw junctions in the actual extreme c_wolf poses. Preserve the original dog appearance and compare the complete cycle, including both sides and front view. The smooth deformation skin must retain thickness at those bends; the rejected visible rigid collars and internal balls are not an acceptable substitute. Only after visual acceptance should the candidate enter native game qualification.

The old frozen tlc_brz_260906 MOD/HAK, its native installation, source manifest and existing application session were left untouched. No Aurora or NWN test was started. No native correctness or owner acceptance is claimed.

Own authoring scripts and rejected diagnostics are retained at artifacts/diagnostics/borzoi-neverblender-20260907. The sequence for the delivered variant is inspect36.py -> repair-v2.py -> heat.py -> export.py -> readback.py. Later experimental segmentation scripts do not contribute to the delivered model.

## Follow-up after the synchronized wolf comparison

The owner raised a forward-registration hypothesis. See [bind registration findings and prevention](borzoi-bind-registration-lessons-2026-09-07.md). Independent source anatomy must now be reviewed before assuming topology alone is the remaining cause. The new offline diagnostic does not certify a global translation as the fix; the model is unchanged and remains a draft.
