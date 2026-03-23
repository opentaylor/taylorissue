Run the OpenClaw built-in diagnostics. DO NOT fix anything.
  openclaw doctor 2>&1
  openclaw status 2>&1

Respond with ONLY this JSON:
{"success": true, "warnings": <number of warnings>, "issues": ["<issue1>", ...], "details": "<summary>"}