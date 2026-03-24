Fix strategy: the gateway is not running. Do ALL of these steps in order:
1. Enable the HTTP endpoint:
   "{openclaw_bin}" config set "gateway.http.endpoints.chatCompletions.enabled" true --strict-json
2. Reinstall and start the gateway:
   "{openclaw_bin}" gateway install --force
   "{openclaw_bin}" gateway start
3. Wait 8 seconds for the gateway to fully initialize:
   On macOS/Linux: sleep 8
   On Windows: Start-Sleep -Seconds 8
4. Verify the gateway is running:
   "{openclaw_bin}" gateway status
Do NOT skip the wait step. Do NOT run any other config set commands.