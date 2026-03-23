Check the OpenClaw gateway health. DO NOT fix anything.

Step 1: Check if openclaw is installed.
On macOS/Linux: which openclaw || echo "not installed"
On Windows: if (Get-Command openclaw -EA SilentlyContinue) { "installed" } else { "not installed" }

Step 2: Check gateway and health status.
  openclaw gateway status 2>&1
  openclaw health 2>&1

The gateway should be running on port {port}.

Respond with ONLY this JSON:
{"success": true, "status": "<running|stopped|not installed>", "port": <port or null>, "details": "<description>"}