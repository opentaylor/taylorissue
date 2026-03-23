Delete OpenClaw configuration files from ~/.openclaw.
Target files: openclaw.json, openclaw.json.bak, packs.json, update-check.json

On macOS/Linux:
  cd ~/.openclaw 2>/dev/null && for f in openclaw.json openclaw.json.bak packs.json update-check.json; do [ -f "$f" ] && rm "$f" && echo "deleted $f"; done
On Windows:
  $dir = "$env:USERPROFILE\.openclaw"; foreach ($f in @('openclaw.json','openclaw.json.bak','packs.json','update-check.json')) { if (Test-Path "$dir\$f") { Remove-Item "$dir\$f" -Force; "deleted $f" } }

Respond with ONLY this JSON:
{"success": true, "files_deleted": ["<file1>", ...], "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}