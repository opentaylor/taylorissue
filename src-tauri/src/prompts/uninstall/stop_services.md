Stop and uninstall the OpenClaw gateway service.
Run these commands in order (skip any that fail gracefully):
  1. openclaw gateway stop 2>/dev/null || true
  2. openclaw gateway uninstall 2>/dev/null || true

If the openclaw CLI is not installed, check for leftover service files:

On macOS:
  ls ~/Library/LaunchAgents/ai.openclaw.gateway.plist 2>/dev/null && rm -f ~/Library/LaunchAgents/ai.openclaw.gateway.plist

On Linux:
  systemctl --user stop openclaw-gateway 2>/dev/null; rm -f ~/.config/systemd/user/openclaw-gateway.service && systemctl --user daemon-reload 2>/dev/null

On Windows (PowerShell):
  schtasks /query /tn "openclaw-gateway" 2>$null; if ($?) { schtasks /delete /tn "openclaw-gateway" /f }
  $startup = [System.IO.Path]::Combine($env:APPDATA, 'Microsoft\Windows\Start Menu\Programs\Startup'); Get-ChildItem $startup -Filter '*openclaw*' -ErrorAction SilentlyContinue | Remove-Item -Force

Respond with ONLY this JSON:
{"success": true, "was_running": true|false, "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}
