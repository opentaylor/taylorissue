Delete the OpenClaw workspace directory.

On macOS/Linux:
  if [ -d ~/.openclaw/workspace ]; then rm -rf ~/.openclaw/workspace && echo "removed"; else echo "already absent"; fi
On Windows:
  $ws = "$env:USERPROFILE\.openclaw\workspace"; if (Test-Path $ws) { Remove-Item $ws -Recurse -Force; "removed" } else { "already absent" }

Respond with ONLY this JSON:
{"success": true, "existed": true|false, "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}