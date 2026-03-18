Delete OpenClaw configuration files from ~/.openclaw.
Target files: openclaw.json, openclaw.json.bak, packs.json, update-check.json

On macOS/Linux:
  ls ~/.openclaw/{openclaw.json,openclaw.json.bak,packs.json,update-check.json} 2>/dev/null
  Delete any that exist with rm.

On Windows (PowerShell):
  $dir = "$env:USERPROFILE\.openclaw"
  foreach ($f in @('openclaw.json','openclaw.json.bak','packs.json','update-check.json')) { if (Test-Path "$dir\$f") { Remove-Item "$dir\$f" -Force; echo "deleted $f" } }

Respond with ONLY this JSON:
{"success": true, "files_deleted": ["<file1>", ...], "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}
