Uninstall the OpenClaw npm package.

Step 1: Remove the package.
  npm rm -g openclaw 2>/dev/null || true

Step 2: Verify removal.
On macOS/Linux: which openclaw && echo "still present" || echo "removed"
On Windows: if (Get-Command openclaw -EA SilentlyContinue) { "still present" } else { "removed" }

Respond with ONLY this JSON:
{"success": true, "version_removed": "<version or unknown>", "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}