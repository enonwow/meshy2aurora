# M0 Last City appearance diag02 — runtime result

Date: 2026-07-22

## Decision

The exact Last City `appearance.2da` prefix is **not sufficient** to make the
current M0 r31 binary model visible in NWN. The same candidate remained visible
in Aurora Toolset and absent in the exact NWN test-module session.

Axes:

- Toolset: `modelVisibility=visible`, `proofCompleteness=verified`.
- NWN: `modelVisibility=not_visible`, `proofCompleteness=failed`.

The NWN visibility verdict is the owner's direct live observation of the exact
candidate-bound session. The central runtime PNG capture failed before producing
an artifact, so this record does not claim a central `verified` runtime packet.

## Exact diagnostic delta

The test preserved the complete Last City table as a byte-identical prefix:

- source bytes: `7,655,336`;
- source SHA-256: `ca0b80b74e068d8ebbd94df6005b5971e50eca5c8662fca10a40688ea2c033a2`;
- source physical rows: `15,219`;
- M0 append: physical row `15219` -> `MODELTYPE=S`, `RACE=m2a_m0p01`;
- H1 append: physical row `15220` -> `MODELTYPE=S`, `RACE=m2a_m6p01`;
- final appearance bytes: `7,655,712`;
- final appearance SHA-256: `f45437470d554d7ecd5e8fef15db0bbd799621308d625fad388950cf5f276c43`.

The M0/H1 MDL and TGA payloads were unchanged. The test changed only the base
appearance-table lineage and the two appended physical row numbers.

## Candidate identity

- MOD: `m2a_diag02.mod`, 25,821 bytes,
  SHA-256 `0334dc33cf51f1489698db5d005b07cd2de3ed0125e31414c7d3a31b05b851bd`.
- HAK: `m2a_diagh02.hak`, 33,530,540 bytes,
  SHA-256 `88e477851a838ba154b16a7ccaa8235a2e6ee81544e59a1ba8c51d8b378bb3e2`.
- Area: `m2a_diaga02`; entry `[10,10,0]`.
- M0 fixture: `candidate_m0`, template `m2a_d_m0`, row `15219`,
  position `[10,14.5,0]`.
- Contract:
  `proof-output/m0-last-city-appearance-diag02-20260722/generated/tri-control-diagnostic-contract-v2.json`,
  SHA-256 `056f1b22e0e058541ee50d08d21cf506b46f5145d6ed0c30a57640e81c173faa`.
- 120-second profile SHA-256:
  `f2c7c7460b2ffbb71ae7968d2f5e87b2fd8e1b40d43f93e64da4672cd2d360a2`.

Generated and installed MOD/HAK copies were byte-identical. Central structural
preflight parsed the exact singleton HAK list, Area, entry point and all three
fixtures successfully.

## Live evidence

Toolset selection and capture:

- selected tree text: `Exact r31 M0 candidate`, occurrence `0`;
- independent caret readback matched the same native handle;
- PNG:
  `proof-output/m0-last-city-appearance-diag02-20260722/live/model-proof-120s-run-1/toolset/capture.png`;
- PNG SHA-256:
  `b68b6834b53a6d209b0f09c599be27e4e72829dc9bd509aad36ef373436a81f7`;
- Toolset packet SHA-256:
  `f58cf365d97947bcb4d07817d8793d9f2687962663d22ee4a19e400b3e8b720f`.

Runtime binding:

- exact NWN PID `18960`, started `2026-07-22T19:37:45.4584259Z`;
- central observation binds the exact MOD hash, Area and entry point;
- fresh client log contains `Loading Module: m2a_diag02`;
- the owner directly observed that the M0 model was absent in NWN.

The central physical-pixel capture failed with
`nwmain_zorder_lift_not_unobscured:18960,18960,20856`. The runner deleted the
unaccepted PNG and wrote blocker SHA-256
`9fe3c4a46bfd0351f67a7e227e0db2e82278932b7d5d0e5bd0574cf4abf03277`.
No automatic retry was performed.

## Consequence

The exact Last City appearance payload did not fix the runtime failure, so a
retail-vs-Last-City base-table mismatch is rejected as the sole cause. Do not
make another MDL/header/animation guess from this result.

The strongest next comparison is the GFF fixture path. In the same historical
diagnostic module the owner created a Creature through Toolset, selected a
stock Appearance and observed it correctly in both Toolset and NWN, while the
programmatically emitted fixtures were absent in NWN. The next task is an exact
semantic/byte comparison of Toolset-authored versus generated UTC/GIT records,
especially `ClassList`, nested structures, struct IDs, field-index ordering and
instance/template fields. A new model iteration is not justified before that
comparison produces a minimal tested generator correction.
