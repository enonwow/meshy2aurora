# Borzoi c_wolf demo V6 — owner authorization

Date: 2026-08-21

The owner directly requested a corrected V6 after the exact V5 candidate
crashed the NWN client during loading. This instruction authorizes one new
candidate despite the ordinary model-iteration gate.

## Frozen failing predecessor

- candidate: V5
- module: `m2aborzmod5.mod`
- HAK: `m2aborzhak5.hak`
- failure class: NWN client fatal error during module loading
- disposition: immutable forensic predecessor; never overwrite or present as
  a usable fallback

## Authorized minimal delta

V6 must preserve the V5 source model, V5 fitted `c_wolf` carrier rig, weight
transfer, motion contract, geometry, hierarchy and fixture semantics. It may
change only the runtime material emission and candidate identities:

- remove the NWN:EE MTR material extension from the generated MDL;
- remove tangent streams and normal/specular/TXI/MTR resources;
- emit the established classic diffuse TGA binding;
- package exactly one MDL, one diffuse TGA and `appearance.2da` in the HAK;
- use fresh V6 resrefs and filenames.

The generator implementation must select this behavior through a declarative
candidate recipe, not through a version-number conditional.

## Proof boundary

Agents may build, inspect, hash, package and install the exact MOD/HAK for the
owner's test. Aurora Toolset and NWN visual/runtime proof remains human-owned;
agents must not start or control either application for this candidate.
