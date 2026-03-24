Start the OpenClaw gateway. Run each step one at a time.

Step 1 — Enable HTTP endpoint:
On macOS/Linux: "{openclaw_bin}" config set "gateway.http.endpoints.chatCompletions.enabled" true --strict-json
On Windows: & "{openclaw_bin}" config set "gateway.http.endpoints.chatCompletions.enabled" true --strict-json

Step 2 — Install as persistent service (registers for auto-start on login):
On macOS/Linux: "{openclaw_bin}" gateway install --force
On Windows: & "{openclaw_bin}" gateway install --force

Step 3 — Kill any process occupying port {port} (skip if port is free):
On macOS/Linux: lsof -ti:{port} | xargs kill -9 2>/dev/null || true
On Windows: $p = Get-NetTCPConnection -LocalPort {port} -EA SilentlyContinue | Select-Object -ExpandProperty OwningProcess -Unique; if ($p) { $p | ForEach-Object { Stop-Process -Id $_ -Force -EA SilentlyContinue } }

Step 4 — Start the gateway process:
On macOS/Linux: "{openclaw_bin}" gateway start
On Windows: Start-Process -FilePath cmd.exe -ArgumentList '/c',"$env:USERPROFILE\.openclaw\gateway.cmd" -WindowStyle Hidden
(On Windows, do NOT use "gateway start" — the Scheduled Task kills the process. Launch gateway.cmd directly in a hidden window instead.)

Step 5 — Wait for startup:
On macOS/Linux: sleep 5
On Windows: Start-Sleep -Seconds 8

Step 6 — Verify:
On macOS/Linux: "{openclaw_bin}" gateway status
On Windows: & "{openclaw_bin}" gateway status

How to judge success:
- "RPC probe: ok" → success.
- "RPC probe: failed" → failure.
- Ignore the "Runtime:" field.

Respond with ONLY this JSON:
{"success": true, "port": {port}}
On failure: {"success": false, "error": "<reason>"}
