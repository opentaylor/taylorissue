Verify the OpenClaw gateway is running.

Step 1: Check config file exists.
On macOS/Linux: test -f ~/.openclaw/openclaw.json && echo true
On Windows: Test-Path "$env:USERPROFILE\.openclaw\openclaw.json"
If missing, report failure immediately.

Step 2: Check gateway status.
  openclaw gateway status

Respond with ONLY this JSON:
{"success": true, "status": "<running|stopped>", "port": {port}}
On failure: {"success": false, "error": "<reason>"}