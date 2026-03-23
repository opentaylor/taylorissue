Start the OpenClaw gateway.

Step 1: Enable the HTTP Chat Completions endpoint.
  openclaw config set "gateway.http.endpoints.chatCompletions.enabled" true --strict-json
If this fails, report failure immediately.

Step 2: Start the gateway in a background process.
On macOS/Linux:
  nohup openclaw gateway --port {port} &
On Windows:
  Start-Process powershell -ArgumentList '-NoExit','-Command','openclaw gateway --port {port}'
No output from this command is normal.

Step 3: Wait for the gateway to initialize.
On macOS/Linux: sleep 5
On Windows: Start-Sleep -Seconds 5

Respond with ONLY this JSON:
{"success": true, "port": {port}}
On failure: {"success": false, "error": "<reason>"}