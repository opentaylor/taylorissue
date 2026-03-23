Fix strategy: the gateway is not running. Enable the HTTP endpoint and start it.
  openclaw config set "gateway.http.endpoints.chatCompletions.enabled" true --strict-json
On macOS/Linux: openclaw gateway stop 2>/dev/null; nohup openclaw gateway &
On Windows: openclaw gateway stop 2>$null; Start-Process powershell -ArgumentList '-NoExit','-Command','openclaw gateway'
Then wait 5 seconds and verify with: openclaw gateway status