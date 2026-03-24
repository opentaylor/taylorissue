Check the OpenClaw gateway health. DO NOT fix anything.

Step 1:
On macOS/Linux: "{openclaw_bin}" gateway status 2>&1
On Windows: & "{openclaw_bin}" gateway status 2>&1

How to interpret:
- "RPC probe: ok" → RUNNING.
- "RPC probe: failed" → STOPPED.
- Ignore the "Runtime:" field — it may say "unknown" even when the gateway is running.

Respond with ONLY this JSON:
{"success": true, "status": "<running|stopped|not installed>", "port": <port or null>, "details": "<description>"}
