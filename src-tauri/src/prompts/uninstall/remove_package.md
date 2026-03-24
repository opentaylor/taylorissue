Uninstall the OpenClaw npm package.
Detected install type: {install_type}
Binary path: {openclaw_bin}

If install type is "taylorissue":
  On macOS/Linux:
    rm -rf ~/.taylorissue/app/node_modules/openclaw
    rm -f ~/.taylorissue/app/node_modules/.bin/openclaw
    echo "removed taylorissue-local package"
  On Windows:
    Remove-Item -Recurse -Force "$env:LOCALAPPDATA\taylorissue\app\node_modules\openclaw" -EA SilentlyContinue
    Remove-Item -Force "$env:LOCALAPPDATA\taylorissue\app\node_modules\.bin\openclaw.cmd" -EA SilentlyContinue
    "removed taylorissue-local package"

If install type is "official":
  On macOS/Linux: npm rm -g openclaw 2>/dev/null || true
  On Windows: & npm uninstall -g openclaw 2>$null

Verify removal:
  On macOS/Linux: which openclaw && echo "still present" || echo "removed"
  On Windows: if (Get-Command openclaw -EA SilentlyContinue) { "still present" } else { "removed" }

Respond with ONLY this JSON:
{"success": true, "version_removed": "<version or unknown>", "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}
