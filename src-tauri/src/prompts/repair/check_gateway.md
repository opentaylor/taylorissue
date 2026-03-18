Check the OpenClaw gateway health. DO NOT fix anything.
Run these commands in order:
  1. On macOS/Linux:
       which openclaw || echo 'not installed'
     On Windows (PowerShell):
       Get-Command openclaw -ErrorAction SilentlyContinue | Out-Null; if (-not $?) { echo 'not installed' } else { echo 'installed' }
  2. openclaw gateway status 2>&1
  3. openclaw health 2>&1

The gateway should be running on port {port}.

Respond with ONLY this JSON:
{"success": true, "status": "<running|stopped|not installed>", "port": <port number or null>, "details": "<description of gateway state>"}
