Delete the OpenClaw workspace directory.

On macOS/Linux:
  ls -d ~/.openclaw/workspace 2>/dev/null
  If it exists: rm -rf ~/.openclaw/workspace

On Windows (PowerShell):
  $ws = "$env:USERPROFILE\.openclaw\workspace"
  if (Test-Path $ws) { Remove-Item $ws -Recurse -Force; echo 'removed' } else { echo 'already absent' }

Respond with ONLY this JSON:
{"success": true, "existed": true|false, "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}
