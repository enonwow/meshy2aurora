# H2 r43 — ready for owner proof

Test-module file: `m2a_h2r43.mod`

Toolset module name: `Meshy2Aurora creature comparison`

Area: `m2a_h2a43`

Status: `ready_for_owner_proof`

## Admission and diagnosis

The owner reported a fresh candidate-bound failure for r42:

- Toolset: `modelVisibility=visible`, `proofCompleteness=verified`
- NWN: `modelVisibility=not_visible`, `proofCompleteness=failed`

The durable r42 result is recorded in
`documentation/evidence/h2-r42-owner-toolset-nwn-visual-result-2026-07-24.json`.
Its SHA-256 is
`ee855285cfe83fac4de461ac1fb76af1bc39e327332124f58d79220934abaefd`.

r42 inserted a controllerless identity Aurora Root above the original skeleton.
That layout differs from the project-owned H1 v20 model that NWN previously
drew. r43 removes that extra root and reproduces the relevant H1 v20 layout:

- the mapped 24-joint skeleton root is also the model root;
- that root is renamed to `m2a_h2p43`;
- the root keeps its bind position and orientation controllers;
- the one rigid H2 surface mesh is a direct child of the model root;
- all seven rig-only animation states remain type 0;
- the H2 geometry and turquoise/orange texture are unchanged from r42.

This is the minimal artifact delta admitted by the r42 NWN visual failure.
r43 is still a rigid visibility candidate, not a successful skinned-deformation
claim.

## Frozen lineage

Output directory:
`proof-output/h2-r43-h1-v20-root-rigid-type0-20260724`

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `generated/m2a_h2p43.mdl` | 599784 | `5e169877c2f66d68f7bc8cae58e73087f346ff51b5652b37bc787e59c4c38d2a` |
| `generated/m2a_h2t43.tga` | 12582956 | `03169b1493ed4b2f5269ed6fa611aef787e219cddd08661c9135a7f82191c167` |
| `generated/appearance.2da` | 6901365 | `63111bfbf1af7c1f9f9dfcdca9ad573f3f2051a0842fb171ecd86ea5969bb114` |
| `generated/m2a_h2r43.hak` | 20084361 | `d4381dbdc9aee5f0dab2b8159a414586a913b7ed9bf68fe48515a951cdebbdb1` |
| `generated/m2a_h2r43.mod` | 20280 | `ef2b48b80e4c01b455a04246c8e1da8f839171d9e9b4f9e835b846738f0a0c89` |
| `h2-r43-h1-v20-root-rigid-type0-lineage-contract-v1.json` | 4752 | `931ec8cdd92685886d7ea342c7078b3340c40214d1d56a7bbcc0da5831547028` |

## Exact bindings and scene

- ordered HAK: `m2a_h2r43`
- H2 Appearance row: `15100`
- row label: `M2A_H2_H1_ROOT_SENTINEL`
- `MODELTYPE`: `S`
- `RACE`: `m2a_h2p43`
- H2 fixture tag/id: `h2_fixture`
- H2 display name: `Meshy H2 H1-root clockwork sentinel`
- H2 position: `(10, 14.5, 0)`, orientation `(1, 0)`
- stock control id: `stock_control`
- stock control Appearance row: `102`
- stock control position: `(7, 14.5, 0)`
- player entry: `(10, 10, 0)`, facing positive Y

## Offline verification

`cargo test -p m2a-core` completed with exit code 0 on 2026-07-24.
The focused r43 test verifies the frozen hashes, MOD/HAK/2DA bindings, direct
root-to-rigid-mesh relationship, 24-joint rig, 1543 triangles, and seven type-0
animation states.

No Toolset or NWN proof was run by the agent. Under the project's human-owned
final-proof rule, both r43 live axes remain:

- Toolset: `modelVisibility=not_tested`, `proofCompleteness=missing`
- NWN: `modelVisibility=not_tested`, `proofCompleteness=missing`

## Native installation update — 2026-07-25

The exact frozen artifacts were installed with create-new/no-clobber semantics
and verified byte-for-byte after copying:

- `modules/m2a_h2r43.mod`: 20,280 bytes, SHA-256
  `ef2b48b80e4c01b455a04246c8e1da8f839171d9e9b4f9e835b846738f0a0c89`;
- `hak/m2a_h2r43.hak`: 20,084,361 bytes, SHA-256
  `d4381dbdc9aee5f0dab2b8159a414586a913b7ed9bf68fe48515a951cdebbdb1`.

The owner can now run the exact installed candidate. The agent did not start,
adopt, control or capture Aurora Toolset or NWN, so both live axes remain
`not_tested/missing`.
