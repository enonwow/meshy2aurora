# Borzoi: bind registration findings and prevention, 2026-09-07

Status: **current dog remains a visually rejected draft**. This is an offline diagnosis, not a model repair or native NWN proof.

## Evidence and owner's observation

The owner observed that the dog appears to need pulling forwards after watching the original c_wolf and the dog at the same cwalk phase. This is a valuable registration hypothesis; a uniform whole-mesh translation has not been established as the cause or a sufficient fix.

Comparison: artifacts/creatures/borzoi-neverblender-draft-20260907/wolf-comparison/wolf-vs-borzoi-walk.mp4. Both subjects use the same reference motion, equal camera scale and synchronized samples. Maximum deformation-matrix discrepancy was 0.0000090674. This verifies motion transfer only.

Exact dog: artifacts/creatures/borzoi-neverblender-draft-20260907/borzoi-cwolf-draft.blend, SHA256 406927e23f85f4db4bb23e12f090892078e37b57728605e5803966ea006d8c2d.
Reference c_wolf.mdl: C:/Projects/the last city/assets/creatures/borzoi/reference/c_wolf.mdl, SHA256 168984607f739350c96fdc896b705011e9b726d8567034014eb1fbb2cebf389b.

## What the new inspection establishes

The scene was evaluated in the actual armature REST state, not cwalk frame zero. Three orthographic overlays use joint origins and joint-to-joint segments; arbitrary Blender bone tails are not anatomical evidence. Cyan is the reference joint chain; orange is a sampled source-surface centre. Model and weights were not edited. The final dog hash remains identical.

Artifacts are under artifacts/creatures/borzoi-neverblender-draft-20260907/bind-alignment-audit: bind-side.png, bind-front.png, bind-top.png, bind-sections.json, registration-review.json and registration-report.json.

Sixteen lower-limb section probes at Z=0.145, 0.245, 0.35 and 0.46 were measured directly on the undeformed source surface, without using weights. Forward is +Y in this Blender scene. Required longitudinal offsets range from -0.005606 to +0.024303 scene units. Their mean is +0.003396. At Z=0.46 the left-front probe wants +0.024303 but the right-front probe wants -0.003316. At Z=0.245 both front probes instead want a small backwards correction.

These are **geometric proxies**, not independently authored joint centres. Fur, toes, asymmetric surface and sampling affect them. They neither establish whole-body translation nor measure the correct shoulder, hip, hock or head anatomy. In particular, this is not evidence against a local forward reshaping of the shoulder/chest. The body and joint annotations still have to be authored and reviewed.

## Failure of the previous process

1. Exact rig transforms, normalized weights and successful export were allowed to advance work without a reviewed anatomical registration of the source mesh.
2. Partial lower-leg corrections were mistaken for sufficient whole-body fit. The old proposal for topology work must be preceded by this registration review; topology is not yet established as the only remaining cause.
3. Surface centres, points inferred from target joints and weight assignments cannot independently certify that the source anatomy fits those joints.
4. The reference/model synchronized comparison was produced too late. It should be an early diagnostic before repeated weight optimisation.
5. A global average can hide opposite regional errors. Moving the entire mesh cannot be accepted solely because the average error improves.

## Implemented independent audit command

Own tool: tools/audit-creature-registration.mjs. It is a standalone command, **not yet connected to Studio's generation controls**. It does not modify a mesh, choose anatomical landmarks, create weights or export NWN assets. It is generic: names, regions, tolerances and forward axis are data, with no wolf-specific logic.

Usage:

    node tools/audit-creature-registration.mjs review.json exact-source-file exact-reference-file report.json

The JSON review contains schemaVersion:1, sourceSha256, referenceSha256, forwardAxis (unit vector), anatomyCoverageReviewed, requiredPointIds and points. Each point has id, region, source, target, tolerance, sourceKind and reviewed. Source/target are in a common world bind coordinate frame. sourceKind is MESH_ANNOTATION, GEOMETRIC_PROXY or RIG_DERIVED. Coverage and tolerances must be reviewed for the particular morphology; there is no universal anatomical tolerance.

The command checks actual input-file hashes, required coverage, annotation provenance and individual local errors. It reports a least-squares translation and residuals by point and region, but never applies that translation. Only reviewed independent MESH_ANNOTATION points inside their tolerances can authorize weight authoring. Missing landmarks, unreviewed coverage, geometric proxies, target-derived landmarks and stale source/reference identity block this authorization. Output never certifies animation quality or native compatibility. Exit 0 means registration prerequisites satisfied, 2 means a written blocking report, 1 means invalid input or I/O error. Existing output is never overwritten.

For the present dog the command correctly returns BLOCKED, canAuthorWeights:false: independent anatomical landmarks and reviewed coverage are absent. Some geometric probes also exceed the provisional 0.02 diagnostic tolerance. This provisional threshold is not an approved anatomical acceptance limit. The 20 missing anatomical points (head, neck, left/right shoulder, elbow, wrist, hip, knee, hock, paws and tail ends) are explicitly listed. The proxy measurements cannot turn this into a pass.

Validation: node --test tools/audit-creature-registration.test.mjs — 9 tests passed. Cases cover uniform backwards translation, opposite regional offsets hidden by an average, missing annotations, zero-error target-derived and proxy points, stale model hash, unreviewed coverage, invalid values and the separation between registration and motion review. The real dog command returned the expected blocking exit 2. The Blender diagnostic exited 0 and the three overlays were visually inspected. No application, runtime, source mesh or frozen MOD/HAK was altered.

## Required integration before automatic generation can rely on this

1. Studio must collect and store independent source-mesh anatomical landmarks (prefer surface triangle/barycentric anchors with a reviewed internal joint offset), and reference target joints. Do not synthesize source annotations by copying target coordinates or inferring them from current weights.
2. Bind the review to the exact fitted geometry and reference rig. Recompute after translation, scaling, reshaping, remeshing, reference changes or landmark edits. Keep the original source immutable.
3. Before weight generation, require complete profile-defined coverage and a nonblocking registration report. Show individual arrows and regional maxima in side/front/top views; retain explicit missing/unknown states. Do not silently fit the entire model from a mean alone.
4. After weighting, compare reference and owned mesh in synchronized walk, run and idle, plus extreme joint bends and ground contact, from multiple views. Record source geometry, rig, weights, clip and phase identities in the review.
5. Treat registration, deformation quality, export fidelity and native NWN validation as separate results. A failure or unknown result in an earlier prerequisite cannot be promoted by a later technical PASS.

The next modelling experiment should test a precisely scoped local or global registration hypothesis with before/after landmarks and synchronized motion. This turn makes no claim that moving the dog forwards has repaired it.
