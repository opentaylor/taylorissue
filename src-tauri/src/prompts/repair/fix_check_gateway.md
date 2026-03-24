Fix strategy: the gateway is not running. Enable the HTTP endpoint and start it as a persistent service.
  "{openclaw_bin}" config set "gateway.http.endpoints.chatCompletions.enabled" true --strict-json
  "{openclaw_bin}" gateway install --force
  "{openclaw_bin}" gateway start
Then wait 5 seconds and verify with: "{openclaw_bin}" gateway status