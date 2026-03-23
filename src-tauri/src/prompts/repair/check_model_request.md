Test whether the model provider responds to a chat completion request via TWO paths. DO NOT fix anything.

Step 1 — Direct provider test:
On macOS/Linux:
  curl -s -o /dev/null -w '%{http_code}' -m 30 -X POST '{completions_url}' -H 'Content-Type: application/json' -H 'Authorization: Bearer {api_key}' -d '{"model": "{model}", "max_tokens": 16, "messages": [{"role": "user", "content": "Say OK"}]}'
On Windows (PowerShell):
  try { $r = Invoke-WebRequest -Uri '{completions_url}' -Method POST -ContentType 'application/json' -Headers @{Authorization='Bearer {api_key}'} -Body '{"model": "{model}", "max_tokens": 16, "messages": [{"role": "user", "content": "Say OK"}]}' -TimeoutSec 30 -UseBasicParsing; $r.StatusCode } catch { $_.Exception.Response.StatusCode.value__ }

Step 2 — Gateway endpoint test (this is the path the app actually uses):
On macOS/Linux:
  curl -s -m 30 -X POST 'http://localhost:{port}/v1/chat/completions' -H 'Content-Type: application/json' -H 'Authorization: Bearer {gateway_token}' -d '{"model": "{model}", "max_tokens": 1, "messages": [{"role": "user", "content": "hi"}]}'
On Windows (PowerShell):
  try { (Invoke-WebRequest -Uri 'http://localhost:{port}/v1/chat/completions' -Method POST -ContentType 'application/json' -Headers @{Authorization='Bearer {gateway_token}'} -Body '{"model": "{model}", "max_tokens": 1, "messages": [{"role": "user", "content": "hi"}]}' -TimeoutSec 30 -UseBasicParsing).Content } catch { $_.Exception.Message }

For Step 2, check the response JSON:
- If usage.total_tokens is 0 and the assistant content starts with a 3-digit number (like "403 ..."), this is an API error surfaced by the gateway. Report it.
- If usage.total_tokens > 0, the gateway path is working.

HTTP 200 on Step 1 does NOT guarantee Step 2 works — the gateway adds system prompts and tools which increase token usage significantly.

Respond with ONLY this JSON:
{"success": true, "http_status": <direct status code or null>, "gateway_status": <"ok"|"error"|null>, "gateway_error": "<error from gateway response if any>", "details": "<description>"}
