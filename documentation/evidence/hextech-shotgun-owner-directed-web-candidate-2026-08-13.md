# Hextech Shotgun owner-directed web candidate

Date: 2026-08-13

Scope: Meshy2Aurora web application and offline geometry gates

Branch: `codex/items-agent-remediation`

## Result

The Studio recognizes one exact candidate from immutable source and reference
identity, applies its directed Bottom/Middle/Top transforms, and submits those
transforms to `ITEM_REFERENCE_MANUAL_FIT_V2`. The candidate passes both
authoritative adjacent-connector gates and preserves the HAND reference frame.

This is a technical web-candidate result. `ownerStatus` remains
`NOT_REVIEWED`; this packet does not claim Aurora Toolset or NWN visual proof.

## Immutable identity

| Input | Identity |
|---|---|
| Concept | `documentation/concepts/firearm-hextech-shotgun-v1/hextech-shotgun-concept.png` · SHA-256 `cfa31ccea74b53b1e0c55182ec3e1ed4a2072041b433009a448b7717509bd8f9` |
| Bottom GLB | `sample-3d/tlc-hextech-shotgun-parts-v1/bottom.glb` · SHA-256 `69c78999590b248bf9c642516ffa595d33774ead3436166963b27dfaa71ad48d` |
| Middle GLB | `sample-3d/tlc-hextech-shotgun-parts-v1/middle.glb` · SHA-256 `8fafe6a55dd77107a67f29c7519f3b6edc390b310f918a89131b003517720147` |
| Top GLB | `sample-3d/tlc-hextech-shotgun-parts-v1/top.glb` · SHA-256 `6ce1281a4ed8a239bf0d6fc9388fe8a977a2811750d40eab4320e13b642c77bf` |
| Reference set | `wbwxh_b_014/wbwxh_m_014/wbwxh_t_014` |
| Output identity | BaseItem `113` · `hextech_shotgun` · ItemClass `WHxSh` · ModelType `2` |

The exact ignored retail references are available read-only under
`C:\Projects\meshy2aurora\local-reference-assets\item\hextech-shotgun-wbwxh-014`:

| File | SHA-256 | Bytes |
|---|---|---:|
| `wbwxh_b_014.mdl` | `66a9c08a9af50181442086cade525a185d9047447f62451ec92952f6f56ca3ec` | `4,812` |
| `wbwxh_m_014.mdl` | `29274c2ccb02e72cd706c6948d0ba0ff0ecc2225dfc4be7126c6b93265c24524` | `16,268` |
| `wbwxh_t_014.mdl` | `996ec5b3878fabfb451c4b6d2e13291beca47144d34bd138833b432d9fe46c0a` | `18,962` |

These files were copied from the earlier temporary extraction only after
before/after SHA-256 equality was confirmed. They remain Git-ignored reference
inputs and are not product, fixture, HAK or MOD payloads.

The tracked concept bytes are hashed by the Vitest regression suite. The
concept comparison card is rendered only after the exact GLB hashes and retail
reference ID resolve this contract.

## Exact directed transforms

| Part | Translation | Quaternion XYZW | Internal uniform scale | Target-space scale |
|---|---|---|---:|---|
| Bottom / ModelPart1 | `[-0.00431, 0.12917034, 0.15773459]` | `[0.5, -0.5, 0.5, 0.5]` | `0.15796308` | `[1, 1, 1]` |
| Middle / ModelPart2 | `[-0.00431, -0.00852164987, -0.18716540565]` | `[-0.5, -0.5, -0.5, 0.5]` | `0.21066014` | `[1, 1, 1]` |
| Top / ModelPart3 | `[-0.00431, 0.008945521, -0.49844033]` | `[-0.5, -0.5, -0.5, 0.5]` | `0.13161969` | `[1, 1, 1]` |

The internal GLB scale is intentionally distinct from the target-space scale.
The latter remains the owner's exact `[1,1,1]` contract. Substituting absolute
`uniformScale=1` fails the authoritative surface/connector gate and is covered
by the real-corpus regression.

## Authoritative geometry result

- validation algorithm: `ITEM_REFERENCE_MANUAL_FIT_V2`;
- tolerance: `0.005`;
- Bottom → Middle axial overlap: approximately `0.0051`;
- Middle → Top axial overlap: approximately `0.0137`;
- adjacent connections: `2/2 OVERLAPPING`;
- non-adjacent Bottom/Top separation: preserved;
- attachment route: `HAND` preserved;
- fit status: `PASSED`.

## Regression coverage

- `itemAuthoringRecipeV2.test.ts` binds the exact tracked concept bytes, exact
  source hashes, reference ID, protected Bottom/Top transforms and Middle-only
  correction scope;
- `ItemWorkflow.test.tsx` proves that the exact candidate performs the baseline
  request followed by manual-fit validation and exposes the concept comparison
  without marking it owner-accepted;
- `crates/m2a-core/tests/item.rs` validates these exact transforms against the
  real canonical GLBs and exact retail MDL frames under the env-gated corpus.

## Remaining owner decision

The final visual decision belongs to the owner. The Studio explicitly shows
`Visual owner acceptance is still pending` beside the exact concept. Agent-run
Toolset/NWN proof remains forbidden by the project decision dated 2026-07-24.
No new MOD, HAK, resref or model iteration was created for this web candidate.

On 2026-08-13 automated control of the local in-app browser was rejected by the
browser URL policy. No screenshot or visual acceptance is inferred from that
lane failure. The application can still be opened for the owner's direct review.
