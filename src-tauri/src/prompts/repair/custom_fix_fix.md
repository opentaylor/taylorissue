Fix the diagnosed issue. You CAN and SHOULD modify system state.

If no prior diagnosis is available, first run quick diagnostics:
On macOS/Linux: "{openclaw_bin}" doctor
On Windows: & "{openclaw_bin}" doctor

On macOS/Linux: "{openclaw_bin}" gateway status
On Windows: & "{openclaw_bin}" gateway status

Respond with ONLY this JSON:
{"success": true, "actions": ["<action1>", ...], "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}
