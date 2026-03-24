The diagnostic scan for step "{step_id}" found the following issue:

{issue_description}

{guidance}

User configuration — use these values when writing or restoring config:
  API Base URL: {base_url}
  API Key: {api_key}
  Model: {model}
  Gateway Port: {port}

IMPORTANT RULES:
- Execute ONLY the commands listed in the guidance above. Do NOT run any other config set commands.
- Do NOT modify gateway.mode, gateway.bind, or any other config values not mentioned in the guidance.
- After restarting the gateway, you MUST wait at least 5 seconds and then verify with "{openclaw_bin}" gateway status before reporting success.
- NEVER overwrite ~/.openclaw/openclaw.json with a minimal stub.
- If the config is damaged beyond repair, regenerate it with:
  "{openclaw_bin}" onboard --non-interactive --mode local --auth-choice custom-api-key --custom-base-url "{base_url}" --custom-model-id "{model}" --custom-api-key "{api_key}" --custom-provider-id custom --custom-compatibility openai --accept-risk --gateway-port {port} --gateway-bind loopback --skip-skills --skip-channels

Respond with ONLY this JSON:
{"success": true, "details": "<what was done>"}
On failure: {"success": false, "error": "<reason>"}