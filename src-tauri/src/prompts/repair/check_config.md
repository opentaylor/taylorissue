Check the OpenClaw configuration. DO NOT fix anything.

Step 1: Validate JSON syntax.
On macOS/Linux: python3 -c "import json; json.load(open('$HOME/.openclaw/openclaw.json'))" 2>&1 && echo "VALID" || echo "INVALID"
On Windows: try { Get-Content "$env:USERPROFILE\.openclaw\openclaw.json" -Raw | ConvertFrom-Json | Out-Null; "VALID" } catch { "INVALID: $_" }
If INVALID, report has_config and valid_json as false.

Step 2: Read config content.
On macOS/Linux: cat ~/.openclaw/openclaw.json 2>/dev/null | head -100
On Windows: Get-Content "$env:USERPROFILE\.openclaw\openclaw.json" -EA SilentlyContinue | Select-Object -First 100

Step 3: Check that a model provider exists under models.providers with a baseUrl and apiKey.
Report model_configured as true if ANY provider has a non-empty baseUrl and apiKey.
Missing contextWindow or maxTokens is NOT a failure — only report model_configured as false if no provider has baseUrl + apiKey.

Expected: base URL contains {base_url}, gateway port={port}.

Respond with ONLY this JSON:
{"success": true, "has_config": true|false, "model_configured": true|false, "valid_json": true|false, "details": "<description>"}