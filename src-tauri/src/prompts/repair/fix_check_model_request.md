Fix strategy — do these in order, stop once the issue is resolved:
1. Ensure the HTTP endpoint is enabled:
   "{openclaw_bin}" config set "gateway.http.endpoints.chatCompletions.enabled" true --strict-json
2. Restart the gateway as a persistent service:
   "{openclaw_bin}" gateway install --force
   "{openclaw_bin}" gateway start
3. Wait 5 seconds, then re-test the gateway endpoint.
4. If the direct provider test also failed (HTTP != 200), verify API key and base URL in the config.