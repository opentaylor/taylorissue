Fix strategy: repair or regenerate the configuration.

If contextWindow or maxTokens are missing, set them with these EXACT commands:
  "{openclaw_bin}" config set "models.providers.custom.models[0].contextWindow" 1000000 --strict-json
  "{openclaw_bin}" config set "models.providers.custom.models[0].maxTokens" 32768 --strict-json

Do NOT use any other config key paths (e.g. "model.xxx" is WRONG).
If the JSON is corrupted or no provider exists, regenerate with the onboard command in the fix template.