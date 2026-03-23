Configure OpenClaw. Run these 2 commands ONE AT A TIME. Do NOT add extra commands or read any files.

Command 1:
openclaw onboard --non-interactive --mode local --auth-choice custom-api-key --custom-base-url "{base_url}" --custom-model-id "{model}" --custom-api-key "{api_key}" --custom-provider-id custom --custom-compatibility openai --accept-risk --gateway-port {port} --gateway-bind loopback --skip-skills --skip-channels

Ignore ALL warnings (e.g. "gateway not reachable", "WSL2"). These are expected.

Command 2:
openclaw config set "models.providers.custom.models[0].contextWindow" 1000000 --strict-json; openclaw config set "models.providers.custom.models[0].maxTokens" 32768 --strict-json

Respond with ONLY this JSON:
{"success": true, "config_path": "~/.openclaw/openclaw.json", "details": "configured"}
Only report failure if a command exits with a real error (not warnings).