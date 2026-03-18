The diagnostic scan for step "{step_id}" found the following issue:

{issue_description}

Fix this issue now. You CAN and SHOULD run commands that modify system state.
IMPORTANT: If you modify ~/.openclaw/openclaw.json, you MUST run 'openclaw gateway restart' afterwards so changes take effect.

Respond with ONLY this JSON:
{"success": true, "details": "<what was done to fix it>"}
On failure: {"success": false, "error": "<why the fix did not work>"}