# Owner result for `m2bronpal223.mod` and the distinct 224 source decision

Date: 2026-09-04

## Exact owner result

The owner identified the exact test module `m2bronpal223.mod` and reported:

> porażka, wczesniejsza wersja byla duzo lepszy

This is an owner visual-quality rejection of the immutable 223 V2 lineage. It
does not assert that the model was absent.

- module SHA-256: `f3cfa445646faa8e2fdc513775731cb0d5cb3dad0b8eb1a4ac3074b69ad1cb37`
- HAK SHA-256: `a3f02239e5f6e54f61058de5a9ef7bc255e63015e5bb0dae3d1b79e50b902ace`
- MDL SHA-256: `c72bb32aa702e596564c01dfcf8044971a93c4fe0a31a37569bf2f8aac5d9981`
- Toolset model visibility: `visible`
- Toolset proof completeness: `verified` for the owner quality verdict
- NWN model visibility: `not_tested`
- NWN proof completeness: `missing`

## Diagnosis

Candidate 223 changed the earlier donor-top-fit placement into a global
ground-clearance placement. Its Z translation moved from approximately
`-0.12338 m` to `+0.002 m`. That solved the free-standing ground presentation
at the cost of the equipped helmet fit and was therefore rejected by the
owner.

## Distinct 224 lineage decision

The owner then selected a different local source,
`bronze mask 3d model - new.glb`, and explicitly requested a new helmet whose
red/source-cloth material maps to `Cloth 1` and whose yellow/source-metal
material maps to `Metal 1`.

The 224 candidate is admitted as a distinct owner-selected source lineage, not
as a retry or renaming of the rejected 223 payload. Its minimal intended delta
is:

1. use the new two-material source SHA-256
   `8014dfc4a600d8dcd4ad251862e4d20db6900103dd98383e5ea9077d74aa4f0d`;
2. normalize two static mesh nodes to one Aurora-compatible mesh node while
   preserving two material primitives;
3. remove the exact two degenerate triangles required by Profile A;
4. restore the donor-top-fit policy used by the better 222 candidate;
5. bind source material `0` uniformly to PLT layer `4` (`Cloth 1`);
6. bind source material `1` uniformly to PLT layer `2` (`Metal 1`).

No Toolset or NWN process was started or controlled while preparing this
decision.
