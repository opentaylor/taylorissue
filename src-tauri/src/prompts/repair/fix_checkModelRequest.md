Fix strategy — do these in order, stop once the issue is resolved:
1. Ensure the HTTP endpoint is enabled:
   openclaw config set "gateway.http.endpoints.chatCompletions.enabled" true --strict-json
2. Restart the gateway:
   On macOS/Linux: openclaw gateway stop 2>/dev/null; nohup openclaw gateway &
   On Windows: openclaw gateway stop 2>$null; Start-Process powershell -ArgumentList '-NoExit','-Command','openclaw gateway'
3. Wait 5 seconds, then re-test the gateway endpoint.
4. If the direct provider test also failed (HTTP != 200), verify API key and base URL in the config.