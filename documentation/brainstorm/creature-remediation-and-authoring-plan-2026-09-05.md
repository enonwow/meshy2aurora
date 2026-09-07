# Creature remediation and authoring plan

Date: 2026-09-05. Status: **PROPOSED DELIVERY PLAN — no product implementation performed**.

## 1. Outcome and scope

Make Meshy2Aurora Studio a reliable, repeatable workshop for creating Creature:

`source -> saved project -> animation or supermodel selection -> material/fit/event corrections -> final binary readback preview -> validated export -> owner verification`.

The first useful release should let an author finish a supported Creature, close Studio, reopen the project, and continue without reconstructing settings or writing JSON. The next Creature should reuse a compatible recipe while retaining its own source identity and validation. Preserve the existing Rust conversion core; consolidate its product paths instead of rewriting it.

This plan concerns `C:\Projects\meshy2aurora`, the project analyzed in the linked task, although the requesting task is attached to `aurora-web`. Durable notes therefore belong in this repository's `documentation` tree. Aurora Web remains a read-only reference, not a dependency or test oracle.

The three workflows must be explicit in the UI:

| Workflow | First-release boundary |
| --- | --- |
| Owned rig and source animations | Import rigged GLB, select/map clips, explicitly fill missing supported states, author events. |
| New surface using an existing supermodel | Select exact dependencies; preserve reference carrier hierarchy and bind transforms; fit the owned surface and edit its weights. |
| Static rigid Creature | Explicitly static output, with honest material/facing capabilities; not presented as an animated Creature. |

Creating a reusable **owned supermodel** is a later, separate workflow. Applying an existing supermodel does not implement it. A full sculpting, skeleton-construction and keyframe editor is outside the first release; owned rigged GLB and authored animation inputs are sufficient to start.

## 2. Evidence and pre-flight

### Source register

| ID | Source | Use in this plan |
| --- | --- | --- |
| A1 | [General audit, 2026-09-05](../audyt-projektu-2026-09-05.md) | Nine bridge, Studio, materials, identity, CI and Docker findings; recorded quality results. |
| A2 | [Creature/animation/supermodel audit, 2026-09-05](../audyt-creature-animacje-supermodele-2026-09-05.md) | Ten detailed findings, existing capabilities, product gaps and bounded reproductions. |
| I1 | [Creature remediation, 2026-08-17](../CREATURE_AUDIT_IMPLEMENTATION_2026-08-17.md) | Existing revision/epoch guards and profile capability handling to extend. |
| I2 | [Creature stages 1/4/5/6/7, 2026-08-19](../creature-stages-1-4-5-6-7-implementation-2026-08-19.md) | Existing equipment, UTC, envelope, MotionPack, material and performance contracts; not all exposed by Studio. |
| I3 | [Rig authoring, 2026-08-25](../reference-supermodel-rig-authoring-v1-implementation-2026-08-25.md) | Existing sealed authoring and numeric editing; historical transform-editing scope must be reconciled with current immutable bind. |
| P1 | [Immutable supermodel repair, 2026-09-02](../evidence/borzoi-v10-immutable-supermodel-repair-offline-pass-2026-09-02.md) | Current documented invariant: unchanged reference bind, owned-surface registration and generated weights. Offline success with a native HAK collision; not native visual success. |
| P2 | [New weapon animations V2 plan](../plan-implementacji-animacji-nowych-broni-v2-2026-08-27.md) | Reuse this separate program for advanced weapon layering; do not duplicate it here. |
| C1 | [Web/WASM architecture](../architektura-web-wasm-codex.md) | Local-first UI -> Worker -> WASM -> Core, binary output and own readback. |
| C2 | [Knowledge-readiness matrix](../macierz-gotowosci-wiedzy-codex.md) | Starting index for semantic research; dated claims require exact-source revalidation. |

The linked task `01a070c5-26ca-7a42-924a-cd6a1763b590` is titled **Przeprowadź audyt meshy2aurora**. Its retrieved main turn is marked interrupted and contains no final response. Two child analyses have final findings. A1 and A2 exist on disk and contain the fuller durable results; this plan uses those documents rather than treating the main turn's absence of a final response as an absence of findings.

Current read-only inspection: HEAD `7289f2b0c8d385b2058912fb4fa6c0eb7e1990ba`, branch `build-week-submission`, 417 porcelain entries before this document. This is a changing working-tree baseline, not a reproducible committed release. The previous general audit recorded 410 entries. Preserve all unrelated changes; implementation starts on a dedicated registered branch/worktree after selecting the actual required changes, including untracked dependencies.

Rechecked in current source: missing material fields in the M0/reference requests; incomplete reference identity; material-editor remount reset; supermodel reapply without authored rig; final readback animation path; exact-42 clip selection; ASCII `mesh: None`; all-node/carrier-count comparison; one-primitive admission; artifact-only persistence; constant positive tangent handedness; bridge nonce reservation after an `await` and cancellation checked only before polling. Other reproductions and historical runtime outcomes are attributed to A1/A2, not rerun here.

**Correction to carry forward:** A2 supersedes A1's preliminary ASCII diagnosis. The dummy-only test fixture is inadequate, but merely adding geometry to that fixture does not fix a parser that discards geometry and events.

Pre-flight applied:

- Read requesting project's canonical rules, applicable reference map, Meshy2Aurora root/documentation rules and canonical-workspace policy. The canonical workspace guard passed before documentation writes.
- This task is planning and source review. Session persistence, identity and bridge concurrency require no new Aurora semantics. Inheritance, animation events/transitions, bind/deformation, material interpretation and new anatomical exporters require the bounded primary-evidence gates below before behavioral implementation.
- Contract-first applies here to **Studio/Worker/WASM/Core**, and to Studio/bridge where used. Do not introduce an Aurora Web backend/API into the local-first product.
- Tests and diagnostic fields are specified before implementation. No full decompilation, product tests, model generation, paid provider calls or live proof were run for this plan.
- Reviewed the external-GPL policy. No xoreos code was consulted or used as implementation authority. A historical document mentioning xoreos cannot by itself close a runtime gap.
- No installed skill specifically improves a Markdown source-grounded delivery plan. Use native file review for this task; browser implementation/proof later must use the applicable Playwright skill. Agent-run Toolset/NWN proof is outside this plan's execution and remains human-owned under Meshy2Aurora rules.

## 3. Delivery order

All tasks below are **planned**, not completed. Priority is relative to the affected feature's release. Effort is relative: S = focused change; M = several cooperating modules; L = split into smaller slices before coding. These are not calendar commitments.

| Stage | Tasks | User-visible result | Exit gate |
| --- | --- | --- | --- |
| 0 — trustworthy baseline | CR-00, CR-01 | Reproducible build inputs; one confirmed provider run and reliable local cancel. | Checkout/build-input defects are repaired and bridge tests pass; remaining semantic failures have assigned slices. The full suite closes after those slices. |
| 1 — preserve author work | CR-02, CR-03 | Save, reopen and reanalyze without losing edits; distinct output identities. | Reopened recipe has equal canonical content; changed dependencies invalidate only affected outputs. |
| 2 — trustworthy export | CR-04, CR-05, CR-06 | Supported materials reach final bytes; valid references are admitted; inherited motion is visible in final Review. | Real Worker/WASM end-to-end tests and exact binary readback pass. |
| 3 — animation authoring | CR-07, CR-08 | Map ordinary clip names, choose completion policy and place action events visually. | Partial/full/extra-clip cases and weapon-family selection pass with explicit provenance. |
| 4 — fit and motion repair | CR-09, CR-10 | Find a bad region/frame and fix its surface or weights without entering vertex indices. | Known negative cases remain blocked; multi-part source and motion coverage gates pass. |
| 5 — faster next Creature | CR-11 | Reusable recipes, contextual presets, incremental work and a guided workflow. | Measured reduction in repeated work with no reuse of stale rigs or validation. |
| 6 — extensibility and release | CR-12, CR-13 | Owned supermodel libraries, then advanced anatomy/weapon extensions; recorded release coverage. | Each enabled capability has its own offline and owner-verification status. |

**Recommended first implementation slice:** CR-00 baseline and CR-02a lifecycle preservation, then CR-03 identity and CR-04 capability honesty. CR-01 must precede use or expansion of integrated Meshy generation. Local GLB authoring need not wait for new generation features. CR-05/06 are mandatory before presenting the reference route as fully reviewable. CR-12 does not block releasing a useful workbench for already supported routes.

## 4. Actionable backlog

### CR-00 — reproducible baseline and ownership (P1, M)

Origin: A1 findings 8/9 and quality results. Dependencies: none.

- Select and record the working-tree inputs needed for this program; do not build a plan around HEAD alone or stage unrelated item/placeable/VFX work. Use a dedicated `codex/` branch for implementation.
- Convert unconditional local-corpus reads into an explicit corpus tier with documented required inputs and explicit skip/block reporting. Keep meaningful synthetic fixtures in default CI. Do not make failures disappear by blanket ignoring tests.
- Repair the two stale surface-anatomy provenance assertions only after checking the preceding geometric assertions and intended provenance semantics. Track ASCII separately in CR-05.
- Include every required workspace member, including `m2a-win32-noreplace`, in Docker build inputs. Verify the declared targets without a bind mount hiding missing files.
- Close current Clippy failures with scoped repairs; do not weaken warning policy globally. Establish bounded local test/proof wrappers where absent before launching resource-heavy validation.
- Reconcile stale documentation: older instructions for moving reference pivots versus immutable bind; root AGENTS' mandatory absent-target/hash-verified MOD/HAK preparation versus the older PROJECT_RULES paragraph forbidding installation. Use the current root owner decision when preparing a future candidate; do not edit rules to bypass a collision.

Acceptance: clean Windows/Linux checkout runs the default synthetic suite; optional corpus availability is separately reported; relevant Docker targets resolve the full workspace; full Rust quality failures are explained and closed before release. Record exact command, revision, input availability and result.

### CR-01 — generation request and cancel safety (P1, M)

Origin: A1 findings 1/2. Boundary: `tools/meshy-local-bridge/index.mjs` and Studio bridge adapter. Dependencies: CR-00 baseline.

- Reserve a confirmation atomically before the first asynchronous balance/provider operation. Bind it to a request fingerprint and one run/result; define duplicate and incompatible-reuse responses.
- Keep cancellation monotonic. Check after awaited operations and immediately before every new provider stage. A late successful poll cannot resume a cancelled local run.
- Define recovery of a request with an unknown provider outcome. Retrying an ambiguous submission must reconcile its known task identity, not silently create another billed task. Do not promise cancellation/refund of work already submitted.

Tests first: use a fake provider and controlled promise barriers for two concurrent confirmations and cancel-during-poll, plus image/retexture paths. Expect at most one local run, no later-stage POST after cancel, and no transition from cancelled to ready. No paid call is needed to reproduce or close these local defects.

### CR-02 — one durable Creature project (P1, L; deliver 02a before 02b)

Origin: A1 finding 3; A2 finding 7 and save/resume gap. Boundaries: `App.tsx`, `app/studioSession.ts`, material editor, supermodel library, new project storage adapter. Dependencies: CR-00.

**02a: preserve edits within the session.** Give one project state owner responsibility for applied material documents, texture blobs/references, rig overrides, source orientation and authoring revisions. Mounting an editor must not write an empty bootstrap over an existing project. Reanalysis of the same exact source/reference retains compatible edits. Reset is explicit and undoable. Source/chain changes mark old edits incompatible rather than silently applying them elsewhere. Extend the existing authoringRevision/buildEpoch guards.

**02b: save and reopen.** Define a versioned project document and local storage adapter. Autosave the recipe and offer explicit project-file export/import. Restore owned texture overrides within a bounded blob store or request a hash-matching relink; a filename alone is insufficient. External reference resources remain local selected dependencies, not silently copied into portable projects. Display missing files and the precise relink action. Migrate supported versions transactionally; reject unknown versions without overwriting the last good save.

Tests first: Inspect -> Apply -> Build -> Inspect; failed build -> Inspect; apply rig -> reanalyze same reference; refresh/reopen; missing blob; wrong-hash relink; quota failure; older schema; late worker result after an edit. Acceptance: canonical recipe equality after round-trip; all supported edits survive; no stale build becomes current; a successful save is reported only after durable storage commits.

### CR-03 — complete, layered build identity (P1 for safe packaging, M)

Origin: A1 finding 7. Boundaries: `creatureArtifactIdentity.ts`, App identity construction, Worker and package manifests. Dependencies: CR-02 project contract.

- Hash canonical typed inputs, not raw JSON formatting. Include all output-affecting source/texture hashes, route, orientation, material profile/recipe, exact ordered dependency chain, sealed rig/weights, animation/event plan, donor selection, repair options and generator/schema versions at the appropriate dependency layer.
- Separate geometry/animation identity, resource-package identity and demo/UTC identity by their real inputs. HP-only edits should reuse unchanged MDL; material, appearance table or envelope changes may still invalidate the resource package. Do not assume every gameplay field has the same dependency footprint.
- Make preview, cache, final build and recovered artifacts report the same recipe revision/digest. Keep display labels and timestamps outside content identity.
- Detect namespace collisions and report exact existing/new hashes. This design applies to newly admitted builds; it is **not** permission to rename or regenerate an existing frozen candidate to bypass the model-iteration gate.

Tests first: identical inputs -> identical bytes/names; changing only the chain, rig, texture bytes, event plan or generator version changes the relevant identity; JSON key order does not; a demo-only stat change leaves the model hash unchanged; differing content at a frozen native target remains blocked.

### CR-04 — materials and facing survive every supported route (P1, L)

Origin: A1 findings 4/5/6. Boundaries: Worker request types, WASM APIs, `model_pipeline.rs`, shared material/UV modules, GLB IR and writer. Dependencies: CR-03.

- First publish a capability matrix keyed by typed route/profile, enforced in UI, Worker and Core. Unsupported applied edits must block with an explanation; never silently discard them. Preserve their project draft for a compatible route.
- Carry sourceForward, material separation, texture authoring and actual texture payloads through M0 and reference export. Reuse the existing material compiler and shared conversion stages. Keep experimental lanes explicitly constrained.
- Initially reject channels needing UV streams that the active path cannot preserve. Then implement full supported UV streams through import -> IR -> segmentation -> writer -> semantic readback under an audited format contract. A report may say preserved only when final bytes prove it.
- Consolidate tangent generation and calculate handedness from tangent/bitangent orientation. Handle degenerate UV and mirrored seams explicitly; preserve all remapped per-vertex attributes when splitting vertices or streams.

Tests first: distinct UV0/UV1; normal map on UV1; ordinary/mirrored UV islands; opposite box-projection faces; two authored materials and a texture override through each enabled route; a non-default facing input. Validate final binary/TGA/MTR/TXI readback and hashes, not only sent props. Run placeable/item regressions for shared-module changes. Visual runtime interpretation remains a separate owner gate.

### CR-05 — honest reference admission and appearance donor (P1, L)

Origin: A2 findings 2/3/10; A1 ASCII failure. Boundaries: `reference_supermodel_generic.rs`, motion contract, product donor selection and catalog capability report. Dependencies: CR-00, CR-03.

- Separate render-node inventory used for bounds from animated carrier inventory. Resolve identities/transforms explicitly rather than pairing all nodes with carriers by array position. Preserve exact carrier identity and immutable bind checks.
- In the first slice, mark the current ASCII path as structural analysis only where geometry/event support is absent; disable Apply there with actionable diagnostics. Do not erase the failing test or relabel unsupported application as success.
- A later ASCII-application slice must preserve required mesh/skin/controllers/events or return explicit unsupported diagnostics, with owned paired ASCII/binary fixtures. ASCII is reference input/debug; final game output stays binary MDL.
- Record the chosen appearance donor row and source hash explicitly. Prefer an explicit author choice; an automatic selection needs a documented deterministic priority independent of unrelated physical row ordering. Surface inherited locomotion/size values for review.

Tests first: root + animated joint + separate render-only mesh; corresponding skin case; genuine missing/changed carrier remains rejected; ASCII geometry and event preservation or explicit non-applicability; permuted unrelated 2DA rows do not change the selected donor's semantics. Before supporting additional source variants, close the exact node/controller/transform evidence gaps with the project's own reader and primary Aurora evidence.

### CR-06 — final Review uses exported geometry and exact inheritance (P1, M)

Origin: A2 finding 1. Boundaries: `AuroraReadbackViewport`, `SupermodelPreviewViewport`, exact-chain loader and result projection. Dependencies: CR-03, CR-05.

- Feed final MDL readback plus the hash-verified ordered chain into one shared resolver used by the library and Review. Show each clip's origin: local, inherited or generated. Preserve local override precedence only under the audited inheritance contract.
- Play the final exported geometry/weights. Show source/applied/final views with synchronized camera and clip time for comparison. Correct the misleading GLB-empty message when the actual problem is missing inherited dependencies.
- Reopen stored artifacts with their recipe and dependency manifest; missing or mismatched parents require relinking. Never fall back to an unrelated resource with the same name.

Tests first: a child with zero local and known inherited clips; local override of one inherited slot; multi-level chain; missing/hash-changed parent; changed recipe while a preview is loading. Capture the real browser's settled final readback view and retain the exact artifact. Browser playback is offline product evidence, not proof of NWN behavior.

### CR-07 — visual animation selection and mapping (P1, L)

Origin: A2 findings 4/5. Boundaries: source controls, MotionPack, `profile_a.rs`, direct/procedural pipeline. Dependencies: CR-02, CR-06.

- Replace clip-count routing with an explicit animation plan: select source clips, map them to supported NWN slots, exclude extras, choose weapon family, and choose which missing states may be generated.
- Reuse source-bound MotionPack validation behind a UI. Distinguish source-rig playback from retargeting from a different rig; the latter requires a separate compatibility contract and evidence, not silent reuse of the old donor path.
- Select the requested family and clips before validating uniqueness of final output slots. Duplicate slots within the selected family remain errors; disjoint unselected families cannot contaminate the export mapping.
- Suggestions are author-editable and record their origin. Do not infer that a sole clip is idle. A required state marked absent remains absent/blocked until explicitly mapped or supported completion is selected. Forty-two states belong to the relevant profile, not every possible Creature anatomy.

Tests first: Idle/Walk/Run/Attack/Death; one Walk clip; full 42; 42 + extra; excluded clips; duplicate source names; same output slot in two disjoint weapon families; conflicting slots within the selected family; unsupported quadruped-to-humanoid route. Acceptance: the author can resolve supported cases without editing JSON, and final clip inventory equals the resolved plan.

### CR-08 — event timeline and sequence review (P1, L)

Origin: A2 finding 6 and scenario gap. Boundaries: shared animation plan/event compiler, player UI and semantic readback. Dependencies: CR-07.

- Place a shared event-authoring stage after clip selection/completion. Retain source/generated/authored provenance, provide an editable timeline and support moving/adding/removing events with undo.
- Give each supported animation profile separate clip-completeness and event-completeness results. Automatic event timing is a proposed starting point. Do not invent native event semantics for an unresearched profile.
- Provide an offline sequence inspector for idle -> movement -> attack -> damage -> death, with events and boundary poses visible. Only advertise native transition parity after confirming its timing/blending/layering semantics from primary evidence.

Tests first: authored hit time round-trips at the declared serialization tolerance; event order and valid time range; clip trimming/rescaling updates or invalidates dependent event times deterministically; generated partial set accepts authored overrides; full source set reports missing required events. Native verification checks actual action synchronization on the exact candidate.

### CR-09 — motion-quality coverage that explains failures (P2 validation gap, L)

Origin: A2 findings 8/9. Boundaries: `reference_supermodel_motion.rs`, shared motion diagnostics and runtime envelope. Dependencies: CR-06/07; event overlays integrate with CR-08.

- Include significant keys and transform extrema; adaptively refine suspicious intervals. Define a bounded work budget and report covered/skipped intervals. Budget exhaustion means incomplete validation, not PASS.
- Test the full root trajectory, not only start/end displacement. Add applicable contact/sliding/cycle-speed checks across supported source, procedural and inherited routes. Bind WALKDIST/RUNDIST comparisons to the selected donor/envelope and researched engine interpretation.
- Link failures to clip, exact time, joint, component and vertices. Clicking a diagnostic seeks the player and selects the affected region. Preserve existing negative reference cases; never raise thresholds to make one named asset pass.

Tests first: short collapse impulse between the previous nine samples; 0 -> 2 m -> 0 root excursion; valid in-place cycle; sliding contact; appendage motion; clips with no usable samples; budget exhausted. Benchmark the actual sampler and deformation workload at several geometry sizes. These are measured product diagnostics, not a claim to have sampled all possible continuous-time poses.

### CR-10 — visual surface/weight repair and multi-part input (P2 usability, L)

Origin: A2 authoring gaps and single-primitive restriction. Boundaries: rig editor, existing material face-selection tools, source IR, segmentation and skinning. Dependencies: CR-02, CR-05, CR-09.

- Deliver surface picking/brush or region selection, influence heatmap, normalize/lock/smooth weights, landmarks, component attachment and bounded undo/redo. Reuse existing selection/overlay infrastructure; do not require typed vertex indices for ordinary corrections.
- For inherited motion, keep reference bind and carrier structure immutable. Controls edit source registration/surface/weights and validated semantic constraints. Owned-rig creation is a separate mode; do not revive superseded reference-pivot editing from older plans.
- Extend static reference application to multiple primitives/materials with stable source primitive/vertex lineage through deterministic stream partitioning. Do not merge away UV seams/materials or discard source skin/animations; an already rigged source must use an explicitly compatible route.
- Preview only affected parts and resample affected dependencies; run the full required motion gate before export.

Tests first: body + eyes + teeth/accessory with multiple materials; picking survives final segmentation through provenance mapping; sparse weight edit and undo; locked influence; invalid weights; 300,000 accepted and 300,001 blocked; stream partitioning retains all geometry and deformation metadata. Browser evidence must show correction and final binary readback at the failing frame.

### CR-11 — reusable recipes and faster preparation (P2 product accelerator, M/L)

Dependencies: CR-02 through CR-10 for the capabilities each recipe exposes.

- A guided workflow exposes only supported choices and names the next corrective action. Add compatible recipe templates for source orientation, material setup, clip mapping, event defaults, envelope and quality presets. Start with validated structural profiles, not resref/species switches in runtime code.
- Add a reference chooser with searchable structural capabilities, available states, dependency availability and why a candidate is incompatible. A score is a suggestion; it cannot override admission or promise arbitrary source/reference compatibility.
- Provide Duplicate as Draft and a semantic settings diff. Rebind a template to the new source explicitly; never reuse old vertex indices, sealed rigs or a previous model's proof on another mesh. Variants of a frozen proof candidate still obey the iteration gate.
- Expose existing runtimeEnvelope/performancePreset settings and relevant equipment options through coherent advanced sections. Presets are warning/quality targets; preserve the shared 300,000 product limit and independent writer-stream limit.
- Cache deterministic stages by complete dependency digest. Release scene resources when leaving the project. Keep evictable asset cache within 2 GB and separate it from durable project documents; eviction must not silently delete unsaved author work.
- Add a serial local preparation queue only after per-item cancel/recovery and project identity work. Provider generation requires the existing explicit confirmation per authorized operation; batch preparation does not silently submit paid jobs.

Acceptance: reopen a project and prepare a second compatible owned source using its template without reentering unchanged settings; changed source invalidates fit and quality results; material-only edits reuse unrelated source analysis; gameplay-only edits reuse unchanged model output. Instrument time-to-review and unnecessary recomputations before claiming a speedup.

### CR-12 — owned supermodels and later extensions (P2 strategic, L)

Dependencies: CR-03, CR-06 through CR-10; bounded research gate before implementation.

- First support importing an owned skeleton/animation library, declaring provided states/events, validating names/hierarchy/bind, and exporting a versioned owned supermodel plus a child using it. Keep dependency/license provenance explicit and avoid copying reference payloads into generated libraries.
- Show state origin and child overrides; reject dependency cycles, incompatible bind and ambiguous controller binding. Reuse the common resolver and binary writer. A second child demonstrates library reuse without asset-specific code.
- Only then add separately scoped anatomical exporters, advanced source-motion retargeting, native transition composition and weapon layers. Connect to the existing P2 weapon V2 program, with a dedicated carrier where that contract requires one. Do not treat MotionPack quadruped intake as a completed quadruped exporter.
- In-Studio bone creation/reparenting/keyframe authoring is a future design decision, not a prerequisite for the import-based library slice.

Acceptance: own parent + two independently validated children; local override and inherited event cases; missing dependency/cycle/conflicting carrier negatives; deterministic packaging and binary readback; owner verdict for each exact native candidate admitted for proof.

### CR-13 — release evidence and candidate handoff (P1 release gate, M)

Dependencies: the tasks behind each capability being released. Do not postpone all visual evaluation until CR-12; use this handoff discipline at every model-affecting milestone.

- Maintain a capability matrix for static, owned animated and inherited-reference routes, classic/EE materials and supported anatomy profiles. Unsupported combinations remain visible as unavailable, not silently rerouted.
- Build a representative owned/synthetic corpus: humanoids with different proportions, a supported quadruped reference pair, appendage, multi-part/multi-material source, mirrored UV, high geometry, missing events/dependencies and known bad deformation. These are coverage axes, not a promise that one passing model proves a whole family.
- Preserve all frozen lineages and candidate-bound historical outcomes. Before any new candidate, verify admission under the current iteration gate. Synthetic in-memory regressions are not a reason to materialize another real MOD/HAK lineage.
- For an admitted MOD/HAK candidate, prepare exact hashes and the required non-overwriting, byte-verified installation under current root AGENTS. A collision remains blocked and is reported; no automatic renaming. This is preparation, not permission to operate Toolset/NWN.
- Handoff starts with exact `.mod` filename, Toolset module name and Area; then object/appearance identity, placement, ordered HAKs, hashes and expected checks. Record owner's Toolset and NWN outcomes separately with `modelVisibility` and `proofCompleteness`.

Acceptance: offline validation, final browser view, candidate preparation and native visual verification have distinct statuses. An offline PASS or restored artifact is never presented as new native success. The September 2 HAK collision remains a historical blocker until its exact current state is safely rechecked and resolved under owner instructions.

## 5. Contract and diagnostic work before implementation

The following are **proposed contract scopes**, not declarations that matching exported types already exist. Reuse/version the existing domain contracts after checking their actual shape; avoid a duplicate parallel protocol.

| Contract scope | Minimum content |
| --- | --- |
| Creature project document | Schema version, project/revision, route/profile, source descriptors and hashes, selected dependencies/order, material/texture documents, mapping/events, surface/weight edits, envelope/equipment, algorithm versions, last compatible build digest. |
| Capability report | Route/profile, supported feature, supported input constraints, unavailable reason, corrective action, evidence revision. |
| Resolved build recipe | Immutable normalized inputs, complete content digests and dependency graph, reproducibility policy, affected output identities. |
| Animation plan | Source-hash/clip identity, selected/excluded clips, target slots, family/scope, completion policy, provenance and events. |
| Reference context | Exact ordered resource hashes, carrier contract, render inventory, immutable bind identity, donor selection and clip-provider map. |
| Validation report | Separate structure/material/bind/skinning/motion/event/export statuses, units/tolerances, sampling coverage, budget exhaustion, evidence and human-readable corrective actions. |
| Worker/bridge operation | Request ID, project revision/digest, progress, cancellation identity, typed result/error, capability/version compatibility; bridge confirmation binding and ambiguous-outcome recovery. |

Each contract change must define maximum JSON bytes, blob bytes, resource/primitive/vertex/influence/clip/key/event counts, transfer ownership, streaming/chunking or explicit bounded non-streaming, timeout/cancel behavior and schema migration. First reuse measured current parser/Worker limits; benchmark missing limits before assigning final numeric values. Do not invent unmeasured engine limits. Transfer large binary payloads as buffers instead of serializing them repeatedly into JSON.

Keep Core responsible for conversion/validation, WASM as a thin adapter, Worker for orchestration, and React for interaction. Extract shared recipe/resolver/preview functions in bounded slices from App; do not pair the fixes with a wholesale architecture rewrite.

Diagnostic records should include request/build ID, project revision, recipe/source/chain hash, route/profile, failing stage/code, clip/time, node/component/vertex where applicable, expected/actual value, metric units, duration and corrective action. Persistent project-level diagnostic reports are part of the product; internal interactive debugging remains development-only. Collect aggregate timings locally; remote telemetry is not required for these functions.

## 6. Aurora-first research gates

| Scope | Required evidence before behavioral change | Initial disposition |
| --- | --- | --- |
| Project persistence, identity, nonce/cancel | Local contracts and deterministic negative tests. | No unknown Aurora semantic; implement without a full engine audit. |
| Reference render nodes, carriers, ASCII variants, donor data | Own-reader reports on owned fixtures; exact primary evidence for transform/controller classification and supported fields. | A2 demonstrates defects; closes reproduction, not every source variant. |
| Inheritance, events, transitions, weapon layers | Exact source/version/function citations or authoritative independently licensed specifications; child/local override/event witnesses and owner acceptance cases. | Existing research is a starting point; audit missing semantics, not names alone. |
| Extra UV streams and material normal basis | Independently confirmed writer/layout/material contract, paired fixtures and semantic readback; native visual comparison. | Mathematical tangent defect is known; native interpretation still requires corroboration. |
| Immutable bind, retargeting, new anatomy/exporter | Exact chain and bind invariants, deformation evidence and bounded corpus; explicit distinction between inherited-bind and independently owned-rig modes. | Preserve current immutable behavior; additional modes remain blocked until researched. |

Research records must separate facts from Aurora decompilation, retail/resource evidence, current local-code observations, implementation inferences and hypotheses. Record input hashes and limitations. Do not use xoreos to close an engine-behavior gap, copy external code/payloads, or introduce per-asset behavior exceptions.

## 7. Verification and speed targets

For every implementation slice: **contract -> failing regression -> diagnostics -> minimal implementation -> focused verification -> review**. Use the actual repository test organization and package scripts. Limit local Vitest/Jest to two workers and 1 GB heaps, own long-running process trees, and serialize browser capture under the shared lock. Where Meshy2Aurora scripts currently lack enforced bounds, CR-00 must supply them rather than bypassing scripts with direct test binaries. Mutation testing is not part of this plan.

Required test layers:

1. Core unit and regression tests for each reproduced defect.
2. Parser/writer round-trip and golden/semantic tests for changed serialization; no automatic acceptance of changed frozen candidate hashes.
3. Actual Worker/WASM boundary tests for recipe-to-output behavior; React mocks alone cannot prove material or animation preservation.
4. Browser scenarios for edit/save/reopen, map/events, fit correction and final Review with fresh settled state and saved exact captures.
5. Focused benchmarks for sampling, multi-part partitioning, texture work, cache reuse and main-thread responsiveness.
6. One full required Rust/Studio/build gate at handoff, plus the relevant clean-checkout/Docker gates. Optional local corpus availability and skipped cases are reported independently. Native model behavior closes only through candidate-bound owner results.

Record a baseline on the same workstation, input hashes, browser/build versions and cache conditions. Measure at least five runs for representative small/medium/near-budget owned inputs and keep individual timings. Proposed product targets, to be calibrated after CR-00:

- Zero lost accepted edits in the save/reopen and step-navigation acceptance corpus.
- Zero ignored authored inputs in any advertised route.
- Zero repeated provider submissions in the deterministic duplicate/cancel tests.
- At least 50% lower median hands-on preparation time for the second compatible Creature using a saved recipe, compared with the measured current manual workflow. Measure provider generation wait and owner verification separately.
- p95 ordinary control response at or below 150 ms on the declared reference workstation while heavy work runs in the Worker; measure input-to-visible-feedback rather than total conversion time.
- No full GLB parse/rig solve for a change whose dependency graph affects only events/materials/UTC respectively; final validation still covers the changed outputs and required dependencies.
- Every advertised final clip has a resolvable provider and explicit event/coverage status. Required work skipped by a budget cannot produce a complete verdict.

These are acceptance targets, not measured results from this planning task. Do not promise universal fitting or a fixed total generation time before baseline data and native evidence exist.

## 8. Coverage of the original findings

| Audit finding | Planned owner |
| --- | --- |
| A1-1 duplicate provider runs; A1-2 cancel resumes | CR-01 |
| A1-3 material/texture state reset | CR-02 |
| A1-4 missing Creature materials/facing; A1-5 lost UV; A1-6 tangent sign | CR-04 |
| A1-7 incomplete reference identity | CR-03 |
| A1-8 local-only default tests; A1-9 Docker crate; red Rust quality | CR-00, with ASCII semantic work in CR-05 |
| A2-1 inherited final Review | CR-06 |
| A2-2 render-node/carrier mismatch; A2-3 ASCII loss; A2-10 donor order | CR-05 |
| A2-4 clip routing; A2-5 weapon-family slot collision | CR-07 |
| A2-6 events | CR-08 |
| A2-7 rig reapply reset | CR-02 |
| A2-8 temporal sampling gap; A2-9 root/contact validation | CR-09 |
| Product gaps: project saving, visual repair, multi-part, reuse, own libraries | CR-02, CR-10, CR-11, CR-12 |
| Final capability and native evidence | CR-13 |

## 9. Planning handoff

Delivered: source-grounded backlog, priorities/dependencies, first implementation slice, proposed contract scopes, research gates, negative tests, quality gates, speed metrics and mapping of all 19 audit findings.

Validation performed for this task: canonical workspace guard and current source/document inspection; all nine local source links resolve, all fourteen CR task sections are present, the plan has no trailing whitespace, and the README passes `git diff --check`. Audit test numbers in A1/A2 remain their dated observations. No product tests or native sessions were rerun, and no model candidate was created or changed. This document adds a plan; it does not mark any remediation, release or model as complete.
