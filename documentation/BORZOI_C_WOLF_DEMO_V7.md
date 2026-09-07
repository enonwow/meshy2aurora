# `m2aborzmod7.mod`

Toolset module name: `Meshy2Aurora Borzoi c_wolf Demo V7`\
Exact Area name: `Meshy2Aurora Borzoi Test Area V7`

Date: 2026-08-21\
Status: `ready_for_owner_proof`

Exact MOD and HAK are installed in the native NWN user directories and their
SHA-256 hashes are byte-identical to the frozen canonical artifacts. The agent
did not start or control Aurora Toolset or NWN; visual proof remains owned by
the project owner.

## Exact correction

V7 preserves the V6 source, V5 fitted `c_wolf` rig, weights, 300,000
triangles, hierarchy, motion contract, motion oracle, diffuse TGA and fixture.
It changes only the material path required by the owner-reported broken-line
artifact:

- source `doubleSided=true` is compiled through the shared NWN:EE material
  compiler;
- the MDL binds `m2aborztex7_m0` in material slot `texture3`;
- the HAK includes one 65-byte canonical MTR:

```text
texture0 m2aborztex7
renderhint normal
transparency 0
twosided 1
```

- all 14 SkinMesh segments have zero tangents;
- no normal map, specular map or TXI is packaged;
- the HAK contains exactly MDL, diffuse TGA, MTR and `appearance.2da`.

The old classic route now fails closed when a bound source material declares
`doubleSided=true`, preventing another silent V6-style semantic loss.

## Exact owner handoff

- test module file: `m2aborzmod7.mod`;
- ordered HAK: `m2aborzhak7`;
- creature blueprint: `m2aborzutc7`;
- Appearance row: `848`;
- model resref: `m2aborzcre7`;
- diffuse texture resref: `m2aborztex7`;
- material resref: `m2aborztex7_m0`;
- creature placement: `(10.0, 14.5, 0.0)`;
- creature orientation: `(0.0, -1.0)`.

## Immutable artifact identity

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `m2aborzmod7.mod` | 15,205 | `7be4c692cafcff70c349b6b4a0310f6450519b38f2a508943b15b50c678d90a2` |
| `m2aborzhak7.hak` | 76,224,077 | `c2e17bd7f7a24ac308148fa5be5255f83bc51dc248eda72d44a0eea4f8b7216c` |
| `m2aborzcre7.mdl` | 25,503,880 | `6659dc523fc5b310a6a12a7359c1234eab57fc262648d370a895da44a71ef359` |
| `m2aborztex7.tga` | 50,331,692 | `170947c6600f95cd55d1c68ea85af03c3cbee6ee2c2c691836cc4b0b77cd8da1` |
| `m2aborztex7_m0.mtr` | 65 | `0d4b71bd14cbb28d7cb5d3832023da7fd4c3f12fca1ec587d6b8b21132f9c834` |

Canonical packet:
`proof-output/borzoi-c-wolf-demo-v7-20260821`.

Native installation:

- `C:\Users\enonw\Documents\Neverwinter Nights\modules\m2aborzmod7.mod`;
- `C:\Users\enonw\Documents\Neverwinter Nights\hak\m2aborzhak7.hak`.

## Offline validation

- `cargo test -p m2a-core`: PASS;
- motion quality: PASS;
- MDL semantic readback: PASS;
- material-extension readback: 14/14 segments match, zero tangents;
- canonical MTR parse/write readback: PASS;
- HAK resource byte readback: PASS;
- MOD scene/fixture readback: PASS;
- source, rig, motion contract and diffuse are byte-identical to V6.

## Owner proof criteria

1. Open `m2aborzmod7.mod`, then Area
   `Meshy2Aurora Borzoi Test Area V7`.
2. Confirm the model is visible and the coat no longer has terrain-visible
   dotted or broken-line gaps.
3. Confirm NWN loads the same module without the V5 fatal crash.
4. Confirm idle, walk, run and attack remain unchanged from V6.

Until the owner reports the result:
`modelVisibility=not_tested`, `proofCompleteness=missing`,
`qualityVerdict=not_tested`.
