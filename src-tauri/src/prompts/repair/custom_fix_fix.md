Fix the diagnosed issue. You CAN and SHOULD modify system state.

If no prior diagnosis is available in the conversation, first run quick diagnostic commands (e.g. `openclaw doctor`, `openclaw gateway status`, check config) to understand the current state before applying fixes.

IMPORTANT: If you modify ~/.openclaw/openclaw.json, restart the gateway afterwards:
On Windows: openclaw gateway stop; Start-Process powershell -ArgumentList '-NoExit','-Command','openclaw gateway'
On macOS/Linux: openclaw gateway stop; nohup openclaw gateway &
Do NOT use 'openclaw gateway restart' — it blocks.
Respond with ONLY this JSON:
{"success": true, "actions": ["<action1>", ...], "details": "<summary>"}
On failure: {"success": false, "error": "<why the fix did not work>"}