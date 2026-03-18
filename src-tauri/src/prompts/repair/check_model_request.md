Test whether the model provider responds to a chat completion request. DO NOT fix anything.

First, try the provider directly:

On macOS/Linux:
  curl -s -o /dev/null -w '%{http_code}' -m 30 -X POST \
    '{completions_url}' \
    -H 'Content-Type: application/json' \
    -H 'Authorization: Bearer {api_key}' \
    -d '{"model": "{model}", "max_tokens": 16, "messages": [{"role": "user", "content": "Say OK"}]}'

On Windows (PowerShell):
  try { $r = Invoke-WebRequest -Uri '{completions_url}' -Method POST -ContentType 'application/json' -Headers @{Authorization='Bearer {api_key}'} -Body '{"model": "{model}", "max_tokens": 16, "messages": [{"role": "user", "content": "Say OK"}]}' -TimeoutSec 30 -UseBasicParsing; $r.StatusCode } catch { $_.Exception.Response.StatusCode.value__ }

Then, if the gateway is running, also test the gateway endpoint:

On macOS/Linux:
  curl -s -o /dev/null -w '%{http_code}' -m 30 -X POST \
    'http://localhost:{port}/v1/chat/completions' \
    -H 'Content-Type: application/json' \
    -H 'Authorization: Bearer {gateway_token}' \
    -d '{"model": "{model}", "max_tokens": 16, "messages": [{"role": "user", "content": "Say OK"}]}'

On Windows (PowerShell):
  try { $r = Invoke-WebRequest -Uri 'http://localhost:{port}/v1/chat/completions' -Method POST -ContentType 'application/json' -Headers @{Authorization='Bearer {gateway_token}'} -Body '{"model": "{model}", "max_tokens": 16, "messages": [{"role": "user", "content": "Say OK"}]}' -TimeoutSec 30 -UseBasicParsing; $r.StatusCode } catch { $_.Exception.Response.StatusCode.value__ }

HTTP 200 = working. 404 from gateway = chatCompletions endpoint not enabled (this is fine if the provider test passed). Other codes indicate a problem.

Respond with ONLY this JSON:
{"success": true, "http_status": <provider status code or null>, "gateway_status": <gateway status code or null>, "details": "<description>"}
