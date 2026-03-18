Verify the OpenClaw gateway is healthy.
Run these commands in order:
  openclaw gateway status
  openclaw health

If gateway status shows it is not running, start it:
  On macOS/Linux:
    openclaw gateway --port {port} &
  On Windows (PowerShell):
    Start-Process openclaw -ArgumentList "gateway","--port","{port}" -WindowStyle Hidden
Then re-check: openclaw gateway status

Respond with ONLY this JSON:
{"success": true, "status": "<running|stopped>", "port": <port number>}
On failure: {"success": false, "error": "<reason>"}
