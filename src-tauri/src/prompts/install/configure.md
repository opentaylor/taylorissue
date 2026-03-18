Configure OpenClaw with the user's model provider.

Run the following onboard command (copy verbatim, do NOT change any values):

openclaw onboard --non-interactive --mode local --auth-choice custom-api-key --custom-base-url "{base_url}" --custom-model-id "{model}" --custom-api-key "{api_key}" --custom-provider-id custom --custom-compatibility openai --accept-risk --gateway-port {port} --gateway-bind loopback --install-daemon --skip-skills --skip-channels --skip-search

If onboard fails, try using openclaw config set to update settings directly, then run openclaw gateway restart.

After onboard succeeds, patch model limits:

openclaw config set "models.providers.custom.models[0].contextWindow" 1000000 --strict-json
openclaw config set "models.providers.custom.models[0].maxTokens" 32768 --strict-json

Restart the gateway:

openclaw gateway restart

Verify config exists (on Windows use Test-Path, on macOS/Linux use ls ~/.openclaw/openclaw.json).

Respond with ONLY this JSON:
{"success": true, "config_path": "<path to config file>", "details": "<brief summary>"}
On failure: {"success": false, "error": "<reason>"}
