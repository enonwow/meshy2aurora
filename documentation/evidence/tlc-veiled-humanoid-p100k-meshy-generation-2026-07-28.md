# TLC Veiled Humanoid P100K — Meshy generation

Status: `generated-animated-source`

## Exact source

- canonical asset:
  `sample-3d/tlc-veiled-humanoid-h1-p100k-v1`;
- combined GLB:
  `sample-3d/tlc-veiled-humanoid-h1-p100k-v1/source.glb`;
- SHA-256:
  `941affd66d2afe803b2a63ba72c41c30592813d3f1deeae4600f24e0fc566b7d`;
- bytes: `14,592,736`;
- concept:
  `documentation/mockups/the-last-city/creatures/tlc-creature-veiled-humanoid-tpose-front-v1.png`;
- concept SHA-256:
  `2af1ca06237587fc81ef6f9c87d680fd2d479daded31187f5658925df29fb15b`.

## Meshy result

- provider/model: Meshy 6 Image-to-3D;
- target polycount: `100,000`;
- topology: triangles;
- pose: T-pose;
- rig height: `1.85 m`;
- run ID: `1955bf8a-0362-4196-a713-68c15d59e518`;
- credits spent: `65`;
- balance before/after: `842 / 777`;
- skin count: `1`;
- joint count: `24`;
- source vertex count: `101,503`;
- raw triangle count: `102,335`;
- zero-area/degenerate triangles: `2,523`;
- Aurora-safe renderable triangles after the shared sanitation rule: `99,812`.

Meshy `target_polycount` is a target, not an exact result. The generated source
therefore must be described as a 100K-target asset with exactly `99,812`
Aurora-safe renderable triangles, not as an exactly 100,000-triangle payload.

## Animations

Ten same-rig Meshy Animation API outputs were downloaded, hash-verified and
merged into the canonical `source.glb`:

1. `cpause1` — idle;
2. `cwalk` — walk;
3. `crun` — sprint/run;
4. `ca1slashl` — primary attack;
5. `ca1slashr` — secondary attack;
6. `cdamagel` — hit reaction;
7. `ckdbck` — knockdown;
8. `cdead` — death;
9. `cguptokdb` — arise;
10. `ctaunt` — alert.

The merge gate verified identical mesh, node and skin topology across all ten
action GLBs. `source.glb` contains exactly ten clips with these explicit names.

## Moderation retry

The first request used the default Meshy moderation and ended before model
creation with `TASK_REJECTED`, because the fictional horror concept was
classified as potentially harmful. No model payload was created and the
balance proves that this attempt consumed `0` credits.

The owner-approved fictional source was retried unchanged with the explicit
and provenance-recorded `moderation=false` option. The local runner now keeps
moderation enabled by default, accepts only `0` or `1` as an explicit override,
and records the selected value in durable provenance.

## Verification

- Meshy bridge contract: `18 passed`;
- animation merge contract: `2 passed`;
- Studio test suite before the paid run: `201 passed`;
- Studio typecheck: passed;
- P100K segmented pipeline regression: passed;
- canonical Meshy asset-layout gate: passed.

No MOD, HAK or visual Toolset/NWN claim is part of this generation stage.
