# H1 native runtime proof — attempt 13

Status: `FAILED BEFORE NWN / NO RUNTIME PROOF`

## Scope

The only canonical proof module remains `m2a_codex_aproof.mod`; the only HAK
remains `m2a_codex_aproof.hak`.  No second test module or HAK was created.

## What was verified

- The generated v6 module passed the own semantic readback test and was copied
  byte-for-byte to the configured NWN modules folder.
- Toolset opened `m2a_codex_aproof.mod` by its explicit module-list entry.
- A new Toolset viewport gate was observed: its root tree contained only the
  standard top-level categories, with no child under `Areas`.  It therefore
  could not show the proof area or H1, and **NWN was not launched**.
- A local Aurora-first audit found that the former GIT writer emitted wrong
  field types (`Race`, `Phenotype`, wings/tail, `WalkRate`, save bonuses) and
  omitted the six required typed lists.  v8 corrects that independently in the
  own writer and passes its unit test, but Toolset still did not register the
  area.

## Artefacts

- `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-13/toolset-v6-module-open.png`
- `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-13/toolset-v7-after-show.png`
- `proof-output/meshy-h1-native-runtime-2026-07-17-attempt-13/toolset-v8-after-git70.png`
- v8 generated MOD SHA-256:
  `2c99855ba44d935743741409b91a513a8a0d52f70e4b14aa0aa87e7d6baaa442`

The captured Toolset images do not display H1 and are not proof artefacts of
the animation.  No MP4 was produced or claimed.

## Audit conclusion

The remaining fault is not established as an animation or Meshy issue.  It is
an unresolved Toolset acceptance issue in the independently emitted
ARE/GIT/GIC/IFO module contract.  The next fresh iteration must use the local
Toolset-openable module only as a read-only schema/order reference, derive a
specific difference with the own GFF reader, add a regression test, then
rebuild the same canonical module.  It must not iterate through NWN before the
Toolset viewport visibly contains H1.
