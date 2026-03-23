Fix the diagnosed issue. You CAN and SHOULD modify system state.

If no prior diagnosis is available, first run quick diagnostics (e.g. `openclaw doctor`, `openclaw gateway status`, check config) before applying fixes.

Respond with ONLY this JSON:
{"success": true, "actions": ["<action1>", ...], "details": "<summary>"}
On failure: {"success": false, "error": "<reason>"}