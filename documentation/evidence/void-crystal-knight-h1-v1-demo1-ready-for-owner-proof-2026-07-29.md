# vckdemo1.mod — Void Crystal Knight demo handoff

**Toolset module name:** `Meshy2Aurora procedural humanoid proof`
**Exact Area name:** `Meshy2Aurora M0 binary vertical-slice area`

Status: `ready_for_owner_proof`

The demo was produced by the Meshy2Aurora pipeline from the canonical
`sample-3d/void-crystal-knight-h1-v1/source.glb`. Offline materialization,
semantic readback, animation conformance, and exact native installation are
complete. Aurora Toolset and NWN were not started; visual proof belongs to the
owner.

## Exact candidate identity

- Candidate packet: `proof-output/void-crystal-knight-h1-v1-demo1`
- Module filename/resref: `vckdemo1.mod` / `vckdemo1`
- Area name/resref: `Meshy2Aurora M0 binary vertical-slice area` / `vckarea1`
- Creature resref: `vckutc1`
- HAK filename/resref: `vckhak1.hak` / `vckhak1`
- Appearance row: `15100`
- Model resref: `vcknight_m1`
- Texture resref: `vcknight_t1`
- Creature placement: `X=10.0`, `Y=14.5`, `Z=0.0`
- Creature orientation: `X=0.0`, `Y=-1.0`
- Ordered HAK readback: `vckhak1`

## Immutable artifacts

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `generated/vckdemo1.mod` | 15,142 | `7a77f51ed2bd711e8cef7397f01c6fea339dcdf35d6d52759e7254a41696cef1` |
| `generated/vckhak1.hak` | 22,037,667 | `b5224c9f9038f329833e5cbd0cae88ce890dbfdcf50ace34071f98374604d7a9` |
| `generated/vcknight_m1.mdl` | 2,553,136 | `213283447da0a5d09a1b4030878e0f81509173b835bbca0c84b9d71d4a01cfde` |
| `generated/vcknight_t1.tga` | 12,582,956 | `5101483b93b6eb4e0aa39cc705dcbed9de1a70c1661f85eacffcb58793025e8a` |
| `generated/appearance.2da` | 6,901,319 | `c209b6d81480f39a3b2c6389c4c58d74e933e072e03e9c0ecaee6219bca92e6c` |
| canonical `source.glb` | 9,951,624 | `d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7` |

The exact MOD and HAK were installed and verified byte-for-byte at:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\vckdemo1.mod`
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\vckhak1.hak`

## Pipeline result

- Source inspection: 1 mesh, 24,834 vertices, 19,704 triangles, 1 skin,
  24 joints, 3 source animations.
- Full native profile: `FULL_NATIVE42_PROCEDURAL_HUMANOID_V1`.
- Animation namespace: 42/42 complete.
- Preserved Meshy clips: `cpause1`, `cwalk`, `crun`.
- Procedurally authored missing Aurora states: 39.
- Gameplay event pairs: 23/23 complete.
- Walk/run distinction: pass.
- Required animated-skin conformance: pass.
- Behavior candidate eligibility: pass.
- Binary MDL readback: 42 used/allocated animation pointers, 26 nodes,
  one skin node.
- MOD semantic readback: pass, six expected resources present.

Offline verification:

- Candidate example test: 1 passed, 0 failed.
- `model_pipeline`: 28 passed, 1 environment-gated test ignored, 0 failed.
- Studio source viewport loaded the canonical GLB and played `crun`.

Studio capture:

- `artifacts/demo-runs/void-crystal-knight-h1-v1-demo1/studio-demo.png`
- SHA-256:
  `239c567d7268a64d76aea9b311782ff719c99b4688be49b969bc0c1a941f3a13`

## Owner visual proof

1. Open `vckdemo1.mod` in Aurora Toolset.
2. Confirm module name `Meshy2Aurora procedural humanoid proof`.
3. Open the exact Area `Meshy2Aurora M0 binary vertical-slice area`.
4. Confirm creature `vckutc1` uses Appearance row `15100` and is visible at
   the placement above.
5. Test the same exact module/HAK lineage in NWN before requesting any new
   model iteration.

Current proof axes:

| Lane | modelVisibility | proofCompleteness |
|---|---|---|
| Aurora Toolset | `not_tested` | `missing` |
| NWN runtime | `not_tested` | `missing` |

Do not create a new candidate unless this exact lineage receives a fresh,
candidate-bound `modelVisibility=not_visible` result in Toolset or NWN.
