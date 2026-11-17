# Operators Guide

Running a public proxy invites abuse. This is guidance, not legal advice.

## Before going public

- Add authentication (not implemented yet). Without it, anyone can use your server.
- Keep the SSRF protections on; restrict `allowed_ports`.
- Set rate limits (`per_ip_rate_per_sec`, `per_ip_burst`, `max_streams_per_ip`).
- Put the server behind a reverse proxy and fix client-IP forwarding first, or per-IP limits apply to the proxy itself.

## Logging

Decide what you log and for how long. Avoid logging full URLs by default.

## Abuse handling (template)

1. Publish a contact address for abuse reports.
2. Respond to valid reports within a stated time.
3. Block offending hosts via `denied_hosts` and reload.
4. Keep a short incident log: date, report, action.

## Acceptable-use text (starting point)

"This service may not be used for illegal activity, harassment, or attacking
other systems. We may block traffic and cooperate with lawful requests."

## Incident response

Spike in `varex_rate_limited_total` or `varex_ssrf_blocked_total` usually means scanning:
tighten limits, inspect logs, consider temporary allowlist-only mode (`allowed_hosts`).
