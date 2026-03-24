Run the OpenClaw built-in diagnostics. DO NOT fix anything.
  "{openclaw_bin}" doctor 2>&1
  "{openclaw_bin}" status 2>&1

Classify each finding as either an "error" or a "warning":
- Errors: things that prevent normal operation (service crashed, missing binaries, broken config, gateway cannot start).
- Warnings: informational items that do NOT block operation (missing optional embedding providers, reverse-proxy header trust, unused features). Ignore these.

Respond with ONLY this JSON:
{"success": true, "errors": ["<error1>", ...], "warnings": <number of warnings>, "details": "<summary>"}