Verify the OpenClaw gateway is running.

Step 1 — Check config exists:
On macOS/Linux: test -f ~/.openclaw/openclaw.json && echo true
On Windows: Test-Path "$env:USERPROFILE\.openclaw\openclaw.json"
If missing, report failure immediately.

Step 2 — Check gateway status:
On macOS/Linux: "{openclaw_bin}" gateway status
On Windows: & "{openclaw_bin}" gateway status

How to interpret:
- "RPC probe: ok" → the gateway is running.
- "RPC probe: failed" → the gateway is stopped.
- Ignore the "Runtime:" field — it may say "unknown" even when the gateway is running.

Respond with ONLY this JSON:
{"success": true, "status": "<running|stopped>", "port": {port}}
On failure: {"success": false, "error": "<reason>"}
