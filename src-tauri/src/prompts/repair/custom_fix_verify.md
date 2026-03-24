Verify the current state of OpenClaw. Re-run checks to confirm everything is working.

If no prior fix context is available, run standard health checks:
  "{openclaw_bin}" gateway status
  "{openclaw_bin}" doctor
On macOS/Linux: test -f ~/.openclaw/openclaw.json && echo "config exists"
On Windows: if (Test-Path "$env:USERPROFILE\.openclaw\openclaw.json") { "config exists" }

Respond with ONLY this JSON:
{"success": true, "verified": true, "details": "<verification results>"}
If still broken: {"success": true, "verified": false, "details": "<what is still broken>"}