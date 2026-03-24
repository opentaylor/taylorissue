Start the OpenClaw gateway as a persistent service.

Step 1: Enable the HTTP Chat Completions endpoint.
  "{openclaw_bin}" config set "gateway.http.endpoints.chatCompletions.enabled" true --strict-json
If this fails, report failure immediately.

Step 2: Install and start the gateway as a persistent service.
  "{openclaw_bin}" gateway install --force
  "{openclaw_bin}" gateway start

Step 3: Wait for the gateway to initialize.
On macOS/Linux: sleep 5
On Windows: Start-Sleep -Seconds 5

Step 4: Verify it started.
  "{openclaw_bin}" gateway status

Respond with ONLY this JSON:
{"success": true, "port": {port}}
On failure: {"success": false, "error": "<reason>"}