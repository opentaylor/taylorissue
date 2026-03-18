Run the OpenClaw built-in diagnostics. DO NOT fix anything.
  1. openclaw doctor 2>&1
  2. openclaw status 2>&1

Respond with ONLY this JSON:
{"success": true, "warnings": <number of warnings>, "issues": ["<issue1>", ...], "details": "<doctor output summary>"}