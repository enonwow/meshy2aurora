# Codex H1 binary proof module v12 — 2026-07-17

Status: `MODULE/AREA/PLACEMENT PASS`; `MODEL SCALE / ANIMATION RUNTIME OPEN`.

## Canonical installed pair

- module: `m2a_codex_aproof.mod`
- module SHA-256:
  `fbecb9babec38c8698e5f920b0a466bb63fcd5a78ec98a9ca2675e4bde1ab064`
- module bytes: `11116`
- HAK: `m2a_codex_aproof.hak`
- HAK SHA-256:
  `da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756`

There is still exactly one test module and one attached HAK.

## Own readback

- `MOD V1.0`, five resources;
- one `Mod_Area_list` item;
- one `Mod_HakList` item;
- one `tdc01` area, `2 x 2`, four tiles;
- complete Toolset-valid group:
  `113/2`, `113/3`, `113/1`, `0/0`;
- one GIT creature;
- `Appearance_Type=15100`;
- placement `X=10.0`, `Y=10.0`.

The generated module was copied to the user modules directory and source and
destination hashes were equal before Toolset acquired the destination file.

## Toolset gate

Aurora Toolset loaded the exact title:

`BioWare Aurora Neverwinter Nights Toolset v89.8193.37-17 - m2a_codex_aproof.mod`

The module tree exposed:

- `Codex H1 animation proof area`;
- `Creatures`;
- exactly one `Codex Meshy H1 animation proof creature`.

The area opened as `m2a_caproof_area` without the previous
`A bad tile group was found` modal. The physical-display capture shows the
complete floor group and the creature placement selected at the center.

Evidence:

- `proof-output/meshy-h1-codex-animation-proof-v12/evidence/toolset-area-viewport.png`
- `proof-output/meshy-h1-codex-animation-proof-v12/evidence/toolset-module-tree.png`
- `proof-output/meshy-h1-codex-animation-proof-v12/evidence/toolset-display1-area-reopened.png`

## Rejected attempts and correction

- v9 reused `ttr01 / tile 139` as a single tile. Toolset reported a bad tile
  group because that tile was only a fragment of a multi-tile group.
- v10 used the legal `tdc01 / tile 5` filler. It opened without an error but
  produced a black/void proof area.
- v11 centered the creature and restored lighting, but still used only the
  filler tile.
- v12 emits the complete native-confirmed 2x2 group and centers the creature.

The earlier mistake was treating Toolset tree registration as viewport
acceptance. The active gate now requires both.

## Remaining boundary

The creature placement is present and selectable, but the custom H1 model is
visually very small in the Toolset view. This is not an ARE/GIT placement
failure. It remains a separate model scale/rendering and NWN animation-runtime
gate. This evidence does not claim that animation playback is complete.

## Tests

- proof-module unit/readback test: PASS;
- native Toolset module GFF reader regression: PASS;
- `cargo test -p m2a-core --test model_pipeline`: `7 passed`.
