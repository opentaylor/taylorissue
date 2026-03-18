Uninstall the OpenClaw npm package.
  1. npm rm -g openclaw 2>/dev/null || true
  2. Verify it is gone:
     On macOS/Linux:
       which openclaw && echo 'still present' || echo 'removed'
     On Windows (PowerShell):
       Get-Command openclaw -ErrorAction SilentlyContinue | Out-Null; if ($?) { echo 'still present' } else { echo 'removed' }

Respond with ONLY this JSON:
{"success": true, "version_removed": "<version or unknown>", "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}
