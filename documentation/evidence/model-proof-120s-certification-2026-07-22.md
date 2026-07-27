# Model proof 120-second SLO certification — 2026-07-22

The shared `aurora-model-proof-120s` workflow is certified for the proof-ready
Toolset-to-NWN interval. Preparation is measured separately.

- Cold preparation: run 5, accepted in 65,743 ms.
- Exact same-PID module switch `m2a_m0r30.mod` -> `m2a_m0r31.mod`, followed by
  exact Area load/readback: run 6, accepted in 76,474 ms.
- Hard limit: 120,000 QPC milliseconds; zero retry in both runs.
- Toolset result in both runs: `visible/verified`.
- NWN result in both runs: `not_visible/verified`.

Run 6 binds the exact r31 MOD SHA
`8575465ef683256a54c101a3f4445a4e22803194c9b899a0e43e3fd069fdeafd`,
singleton HAK `m2a_m0r31` SHA
`ef26ae9a6a9df5cef9b3d8b1d33ab3cbe587af30e2ee0214d0b865e91ea5eb12`,
Area `m2a_m0a31`, fixture row 15100, the exact tree selection/readback, a
validated `TScrollBox`, one request-bound `nwmain`, the fresh log line
`Loading Module: m2a_m0r31`, and the runtime PNG.

The runtime capture implementation uses `CopyFromScreen` only for the exact
PID/start/HWND. If physical ownership probes find an obscurer, it grants NWN a
bounded `HWND_TOP` no-activate z-order lease, performs one capture, and restores
the exact predecessor in `finally`. Restore failure deletes and invalidates the
capture. It never uses `TOPMOST`, global input, activation, or mutates the
obscuring application.

Canonical certification record:
`proof-output/m0-r31-hierarchy-only-20260722/model-proof-120s-slo-certification-v1.json`.

Run 6 acceptance SHA-256:
`4b4e11a996ac6d9fddd86e3fc55ac472e633a3d91d4543a5934b496e091a0fac`.
