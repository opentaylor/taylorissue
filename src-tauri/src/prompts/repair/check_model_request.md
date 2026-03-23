Test the model provider via TWO paths. DO NOT fix anything.

Step 1 — Direct provider test:
On macOS/Linux:
  curl -s -o /dev/null -w '%{http_code}' -m 30 -X POST '{completions_url}' -H 'Content-Type: application/json' -H 'Authorization: Bearer {api_key}' -d '{"model": "{model}", "max_tokens": 16, "messages": [{"role": "user", "content": "Say OK"}]}'
On Windows:
  try { $r = Invoke-WebRequest -Uri '{completions_url}' -Method POST -ContentType 'application/json' -Headers @{Authorization='Bearer {api_key}'} -Body '{"model": "{model}", "max_tokens": 16, "messages": [{"role": "user", "content": "Say OK"}]}' -TimeoutSec 30 -UseBasicParsing; $r.StatusCode } catch { $_.Exception.Response.StatusCode.value__ }

Step 2 — Gateway endpoint test:
On macOS/Linux:
  curl -s -m 30 -X POST 'http://localhost:{port}/v1/chat/completions' -H 'Content-Type: application/json' -H 'Authorization: Bearer {gateway_token}' -d '{"model": "{model}", "max_tokens": 1, "messages": [{"role": "user", "content": "hi"}]}'
On Windows:
  try { (Invoke-WebRequest -Uri 'http://localhost:{port}/v1/chat/completions' -Method POST -ContentType 'application/json' -Headers @{Authorization='Bearer {gateway_token}'} -Body '{"model": "{model}", "max_tokens": 1, "messages": [{"role": "user", "content": "hi"}]}' -TimeoutSec 30 -UseBasicParsing).Content } catch { $_.Exception.Message }

For Step 2: report gateway_status "ok" if choices[0].message.content has meaningful text. Report "error" only if no content, content starts with a 3-digit HTTP error code, or the request failed. Ignore usage.total_tokens — some proxies return 0.

Respond with ONLY this JSON:
{"success": true, "http_status": <status code or null>, "gateway_status": <"ok"|"error"|null>, "gateway_error": "<error if any>", "details": "<description>"}