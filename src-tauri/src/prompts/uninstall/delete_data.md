Delete all remaining data inside ~/.openclaw.
This includes: credentials, agents, sessions, logs, secrets.

On macOS/Linux:
  ls ~/.openclaw/ 2>/dev/null
  If has contents: rm -rf ~/.openclaw

On Windows (PowerShell):
  $dir = "$env:USERPROFILE\.openclaw"
  if (Test-Path $dir) { Get-ChildItem $dir; Remove-Item $dir -Recurse -Force; echo 'removed' } else { echo 'already absent' }

Respond with ONLY this JSON:
{"success": true, "items_removed": ["<item1>", ...], "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}
