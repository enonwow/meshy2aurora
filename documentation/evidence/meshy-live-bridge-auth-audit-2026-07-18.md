# Meshy live Bridge authentication audit — 2026-07-18

## Scope and safety boundary

The owner authorized a revocable Meshy test API key for this audit. The key,
Bridge pairing codes, session tokens, balances, signed URLs and raw Meshy task
payloads were neither written to this repository nor reproduced here. The audit
used only read-only Meshy operations: balance and Text-to-3D history. It did
not create, poll, cancel or pay for a Meshy task.

## Evidence

| Check | Result |
| --- | --- |
| Direct Meshy `GET /openapi/v1/balance` | Accepted by Meshy |
| Direct Meshy `GET /openapi/v2/text-to-3d` history | Accepted by Meshy |
| Local Bridge exact-origin health | `READY` |
| Local Bridge one-time pairing | Accepted using the current terminal-emitted code |
| Bridge balance and history proxy | Accepted; ten safe history records returned |

## Finding

The owner-facing failure was not an invalid Meshy API key or a failed Bridge.
The connection screen accepted a *pairing code*, while its wording did not make
clear that this is a separate, single-use value printed by the local Bridge.
Restarting the Docker-supervised Bridge creates a new code; a line copied from
an older restart therefore fails with `PAIRING_REQUIRED`. This made the
owner-facing login path impractical despite a healthy API and Bridge.

## Remediation

- Docker Compose now enables a capability-gated, exact-origin automatic
  pairing action. It mints only a short-lived local session and never returns a
  pairing code or API key to Studio.
- The manual fallback explicitly states that it expects the local terminal
  code, not a Meshy API key. Invalid-code feedback includes a safe local log
  command.
- The Docker-only `Restart local Bridge` control is capability-gated by the
  Bridge contract. It restarts only the local supervisor-managed process and
  never surfaces a code to Studio.

## Remaining boundary

Studio intentionally does not read Docker logs or receive pairing codes. The
direct-process fallback therefore still requires the owner to copy the newest
code locally. Docker Compose uses the narrowly scoped automatic flow only for
the exact configured local Studio origin; an API key never becomes browser
input or browser-visible data.
