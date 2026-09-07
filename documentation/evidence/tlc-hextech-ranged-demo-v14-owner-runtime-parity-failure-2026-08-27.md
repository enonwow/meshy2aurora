# V14 owner runtime verdict — application/demo animation parity failed

Date: 2026-08-27

## Exact candidate and owner capture

- MOD: `m2aimod35d.mod`
- MOD SHA-256: `ce81eb3a2baf8e84bc851ec820f75e0ab0200eb634576c8ff21f6e7a59e1e52e`
- HAK: `m2aihak35d.hak`
- HAK SHA-256: `473a9b5238a290c041192037ec23c3c032a9f18591a512c8eff8cfb8c0098082`
- module: `Meshy2Aurora Hextech Ranged Demo V14`
- Area: `Hextech Ranged Demo V14`
- owner capture:
  `tlc-hextech-ranged-demo-v14-owner-runtime-parity-failure-2026-08-27.png`
- capture SHA-256:
  `4fabcf267a401deffb8f46cd2cfac002a73a95e8f4a154812a56c82da64b02f4`

The owner reported immediately against the installed V14 handoff:

> NIE WIEM CO TY ZROBILES Z ANIMACJAMI ALE CO INNEGO POKAZUJE NASZA
> APLIKACJHA CO INNEGO JEST W DEMIE!

The capture shows the exact weapon and player rendered while the player is in a
locomotion stride. The firearm is carried low and across the body. This does
not match the shoulder-ready pose shown by the application.

Verdict axes for this exact runtime observation:

- `modelVisibility=visible`;
- `proofCompleteness=verified` for the application/runtime animation-parity
  comparison;
- `visualAcceptance=failed`;
- `functionalUsability=failed_animation_parity`;
- ammunition consumption and projectile behavior remain `not_tested` by this
  capture.

## Proven cause

### Application fact

The application preview uses `FIREARM_AUTHORING`: a custom upper-body overlay
on a static `pause1` lower-body key. Its own UI and proof source explicitly say
that `xbowrdy` and `xbowshot` are not pose donors and that the result is not a
runtime-NWN parity claim.

### Generated V14 fact

The generated local `pfh0.mdl` contains only three local states:
`pause1`, `xbowrdy`, and `xbowshot`. It inherits the remaining humanoid state
inventory from supermodel `a_fa`. The V14 screenshot is a locomotion frame, so
the engine requires the `walk`/`run` state family instead of the static app
`pause1` presentation.

There is also a direct implementation split between preview and export. The
preview calls `buildAuroraFirearmUpperBodyOverlayClipsV1`, which retains one
neutral `pause1` key for every root/pelvis/leg track and replaces only the
upper body. The V14 exporter instead called
`buildAuroraFirearmAuthoringClipsV1` directly. That authoring-only builder
emits exactly ten quaternion tracks per clip: torso, neck, both upper arms,
both forearms and both two-node hand chains. It emits no root, pelvis, thigh,
shin or foot tracks. The V14 report confirms `30` total tracks for three clips.

Consequently the app displayed a complete composed pose while the binary V14
clips contained only the upper-body half. Aurora/NWN then had to evaluate the
uncontrolled lower body from bind/inherited state behavior. This explains the
owner-observed collapsed or vehicle-impact-like gait and is independent of the
weapon's local rotation.

The custom BaseItem also sets:

- `WeaponWield=6`;
- `WeaponType=1`;
- `RangedWeapon=27`.

The product's own audited route maps `WeaponWield=6` to `xbowshot` and the
retail crossbow family. The application deliberately excludes that family
from its authored firearm pose. The two surfaces were therefore not driven by
one animation contract.

### Gate failure

The V14 writer report already retained these open runtime deviations:

- `M4A-RUNTIME-ANIM-TREE-PROFILE-OPEN-M6`;
- `M4A-DECOMP-ANIMROOT-CONSUMER-OPEN-M6`;
- `M4A-RUNTIME-STATE-ROUTING-OPEN-M6`.

Despite those deviations, the materializer accepted structural binary
readback as sufficient to create the demo. That was the direct pipeline bug.
The offline test proved file structure, not which animation NWN selects while
the player idles, walks, runs, readies or fires.

The available Toolset decompilation exposes the `WeaponWield` 2DA field but not
the closed NWN client locomotion-state consumer. Local retail/resource evidence
and the owner runtime capture independently establish the required humanoid
state inventory (`pause1`, `walk`, `run`, weapon-ready and weapon-shot states).

## Minimal next iteration boundary

V15 is admitted by this fresh V14 candidate-bound visual failure. The next
iteration must preserve the V14 weapon geometry, item, ammunition, target and
placement unchanged. Its only permitted delta is the animation parity route:

1. one shared state contract for app preview and exported runtime;
2. exact states at minimum: idle, walk, run, ready and shot;
3. native locomotion for root, pelvis and legs, with the firearm upper-body
   overlay baked for each corresponding state;
4. app preview rendered from readback of the exact exported binary MDL rather
   than directly from the authoring-only Three.js clips;
5. materialization blocked while animation-tree, animroot or state-routing
   deviations remain open, or whenever the previewed clip set differs from the
   exported clip set.

No Aurora Toolset or NWN process was started or controlled by the agent while
recording and diagnosing this owner-provided failure.
