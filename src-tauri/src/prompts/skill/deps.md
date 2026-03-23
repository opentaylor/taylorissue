Install the dependencies for OpenClaw skill "{name}".

Run these commands one at a time:
{commands}

If a command is not found (e.g. `python3` on Windows), try the platform equivalent (`python` or `py`). Similarly, `pip3` may be `pip` on Windows.

After all commands succeed, verify each binary is available on PATH.

Respond with ONLY this JSON:
{"success": true, "details": "<packages installed>"}
On failure: {"success": false, "error": "<reason>"}