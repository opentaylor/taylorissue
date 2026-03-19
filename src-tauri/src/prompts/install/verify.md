Verify the OpenClaw installation and start the gateway.

Step 1: Check config file exists.
On Windows: Test-Path "$env:USERPROFILE\.openclaw\openclaw.json"
On macOS/Linux: test -f ~/.openclaw/openclaw.json && echo true
If config file is missing, report failure immediately.

Step 2: Start the gateway in a separate window so it keeps running.
On Windows run this EXACT command:
Start-Process powershell -ArgumentList '-NoExit','-Command','openclaw gateway --port {port}'
On macOS/Linux run:
nohup openclaw gateway --port {port} &

The Start-Process command produces no output — that is normal, NOT an error.

Step 3: Wait 5 seconds, then check gateway status.
On Windows: Start-Sleep -Seconds 5
On macOS/Linux: sleep 5
Then run: openclaw gateway status

Respond with ONLY this JSON:
{"success": true, "status": "<running|stopped>", "port": {port}}
On failure (config file missing): {"success": false, "error": "<reason>"}
