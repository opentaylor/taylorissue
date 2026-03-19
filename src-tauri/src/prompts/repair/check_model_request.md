Test whether the model provider responds to a chat completion request. DO NOT fix anything. DO NOT test the gateway — that is checked separately.

On macOS/Linux:
  curl -s -o /dev/null -w '%{http_code}' -m 30 -X POST '{completions_url}' -H 'Content-Type: application/json' -H 'Authorization: Bearer {api_key}' -d '{"model": "{model}", "max_tokens": 16, "messages": [{"role": "user", "content": "Say OK"}]}'

On Windows (PowerShell):
  try { $r = Invoke-WebRequest -Uri '{completions_url}' -Method POST -ContentType 'application/json' -Headers @{Authorization='Bearer {api_key}'} -Body '{"model": "{model}", "max_tokens": 16, "messages": [{"role": "user", "content": "Say OK"}]}' -TimeoutSec 30 -UseBasicParsing; $r.StatusCode } catch { $_.Exception.Response.StatusCode.value__ }

HTTP 200 = working. Other codes indicate a problem.

Respond with ONLY this JSON:
{"success": true, "http_status": <status code or null>, "gateway_status": null, "details": "<description>"}
