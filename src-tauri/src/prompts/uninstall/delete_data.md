Delete all remaining data inside ~/.openclaw (credentials, agents, sessions, logs, secrets).

On macOS/Linux:
  if [ -d ~/.openclaw ]; then ls ~/.openclaw/ && rm -rf ~/.openclaw && echo "removed"; else echo "already absent"; fi
On Windows:
  $dir = "$env:USERPROFILE\.openclaw"; if (Test-Path $dir) { Get-ChildItem $dir; Remove-Item $dir -Recurse -Force; "removed" } else { "already absent" }

Respond with ONLY this JSON:
{"success": true, "items_removed": ["<item1>", ...], "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}