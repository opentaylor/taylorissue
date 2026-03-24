Test the OpenClaw gateway endpoint. DO NOT fix anything. Do NOT test the upstream provider directly.

On macOS/Linux:
  curl -s -m 30 -X POST 'http://localhost:{port}/v1/chat/completions' -H 'Content-Type: application/json' -H 'Authorization: Bearer {gateway_token}' -d '{"model": "{model}", "max_tokens": 1, "messages": [{"role": "user", "content": "hi"}]}'
On Windows:
  try { (Invoke-WebRequest -Uri 'http://localhost:{port}/v1/chat/completions' -Method POST -ContentType 'application/json' -Headers @{Authorization='Bearer {gateway_token}'} -Body '{"model": "{model}", "max_tokens": 1, "messages": [{"role": "user", "content": "hi"}]}' -TimeoutSec 30 -UseBasicParsing).Content } catch { $_.Exception.Message }

Report gateway_status "ok" if choices[0].message.content has meaningful text. Report "error" only if no content, content starts with a 3-digit HTTP error code, or the request failed. Ignore usage.total_tokens — some proxies return 0.

Respond with ONLY this JSON:
{"success": true, "gateway_status": <"ok"|"error"|null>, "gateway_error": "<error if any>", "details": "<description>"}