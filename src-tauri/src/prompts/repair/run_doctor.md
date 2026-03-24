Run the OpenClaw built-in diagnostics. DO NOT fix anything.
  "{openclaw_bin}" doctor 2>&1
  "{openclaw_bin}" status 2>&1

Classify each finding as either an "error" or a "warning":
- Errors: things that prevent normal operation (service crashed, missing binaries, broken config, gateway cannot start).
- Warnings: informational items that do NOT block operation. Ignore these.

The following are ALWAYS warnings, never errors:
- "No embedding provider is ready" or "missing API keys for providers 'openai', 'google', 'voyage', 'mistral'"
- "Memory search enabled, but no embedding provider"
- "Gateway memory probe for the default agent is not ready"
- "Reverse proxy headers are not trusted"
- "Semantic recall needs at least one embedding provider"
- Any message about optional features not being configured

Respond with ONLY this JSON:
{"success": true, "errors": ["<error1>", ...], "warnings": <number of warnings>, "details": "<summary>"}