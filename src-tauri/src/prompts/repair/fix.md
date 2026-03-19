The diagnostic scan for step "{step_id}" found the following issue:

{issue_description}

Fix this issue now. You CAN and SHOULD run commands that modify system state.
IMPORTANT: If you modify ~/.openclaw/openclaw.json, restart the gateway afterwards:
On Windows: openclaw gateway stop; Start-Process powershell -ArgumentList '-NoExit','-Command','openclaw gateway'
On macOS/Linux: openclaw gateway stop; nohup openclaw gateway &
Do NOT use 'openclaw gateway restart' — it blocks.

Respond with ONLY this JSON:
{"success": true, "details": "<what was done to fix it>"}
On failure: {"success": false, "error": "<why the fix did not work>"}