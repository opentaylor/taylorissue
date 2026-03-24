Verify the current state of OpenClaw. Re-run checks to confirm everything is working.

Run these health checks:
On macOS/Linux:
  "{openclaw_bin}" gateway status
  "{openclaw_bin}" doctor
  test -f ~/.openclaw/openclaw.json && echo true
On Windows:
  & "{openclaw_bin}" gateway status
  & "{openclaw_bin}" doctor
  Test-Path "$env:USERPROFILE\.openclaw\openclaw.json"

Respond with ONLY this JSON:
{"success": true, "verified": true, "details": "<verification results>"}
If still broken: {"success": true, "verified": false, "details": "<what is still broken>"}
