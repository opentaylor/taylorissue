Check the OpenClaw configuration. DO NOT fix anything.
Run these commands:
  1. openclaw config file
  2. On macOS/Linux:
       cat ~/.openclaw/openclaw.json 2>/dev/null | head -80
     On Windows (PowerShell):
       Get-Content "$env:USERPROFILE\.openclaw\openclaw.json" -ErrorAction SilentlyContinue | Select-Object -First 80
  3. Check that a model provider is configured with a baseUrl and apiKey
  4. Check contextWindow and maxTokens values (should be large, e.g. 1000000 and 32768)

The expected model should be: {model}
The expected base URL should contain: {base_url}
The gateway port should be: {port}

Respond with ONLY this JSON:
{"success": true, "has_config": true|false, "model_configured": true|false, "details": "<description of config state>"}
