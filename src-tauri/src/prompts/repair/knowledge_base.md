

## Knowledge Base: Source Code

When you need to understand OpenClaw internals (config format, CLI behavior, error handling), download and explore the source code.

Download (skip if the directory already contains package.json):

On macOS/Linux:
  test -f ~/.taylorissue/openclaw/package.json || (mkdir -p ~/.taylorissue && curl -fSL https://github.com/openclaw/openclaw/archive/refs/heads/main.tar.gz | tar -xz -C ~/.taylorissue && mv ~/.taylorissue/openclaw-main ~/.taylorissue/openclaw)

On Windows:
  if (-not (Test-Path "$env:LOCALAPPDATA\taylorissue\openclaw\package.json")) { New-Item -ItemType Directory -Force "$env:LOCALAPPDATA\taylorissue" | Out-Null; Invoke-WebRequest -Uri "https://github.com/openclaw/openclaw/archive/refs/heads/main.zip" -OutFile "$env:TEMP\openclaw-ref.zip" -UseBasicParsing; Expand-Archive -Path "$env:TEMP\openclaw-ref.zip" -DestinationPath "$env:LOCALAPPDATA\taylorissue" -Force; Rename-Item "$env:LOCALAPPDATA\taylorissue\openclaw-main" "openclaw" -ErrorAction SilentlyContinue; Remove-Item "$env:TEMP\openclaw-ref.zip" -ErrorAction SilentlyContinue }

Source tree location:
  On macOS/Linux: ~/.taylorissue/openclaw
  On Windows: %LOCALAPPDATA%\taylorissue\openclaw

Key directories: tools/, channels/, providers/, agents/, gateway/, cmd/, docs/.

Explore:
  On macOS/Linux: grep -r "KEYWORD" ~/.taylorissue/openclaw/docs/ 2>/dev/null
  On Windows: Select-String -Path "$env:LOCALAPPDATA\taylorissue\openclaw\docs\*" -Pattern "KEYWORD" -Recurse

## Knowledge Base: GitHub Issues

When source code alone doesn't explain a problem, search GitHub Issues for known bugs, workarounds, and fixes.

Search (replace KEYWORDS with terms from the error message):
  On macOS/Linux:
    curl -s "https://api.github.com/search/issues?q=KEYWORDS+repo:openclaw/openclaw&per_page=5" | python3 -c "import sys,json; [print(f'#{i[\"number\"]} [{i[\"state\"]}] {i[\"title\"]}\n  {i[\"html_url\"]}') for i in json.load(sys.stdin).get('items',[])]"
  On Windows:
    (Invoke-RestMethod "https://api.github.com/search/issues?q=KEYWORDS+repo:openclaw/openclaw&per_page=5").items | ForEach-Object { "#{0} [{1}] {2}`n  {3}" -f $_.number,$_.state,$_.title,$_.html_url }

Read a specific issue (replace NUMBER):
  On macOS/Linux:
    curl -s "https://api.github.com/repos/openclaw/openclaw/issues/NUMBER" | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'#{d[\"number\"]} {d[\"title\"]}\nState: {d[\"state\"]}\n\n{d[\"body\"][:2000]}')"
  On Windows:
    $d = Invoke-RestMethod "https://api.github.com/repos/openclaw/openclaw/issues/NUMBER"; "#{0} {1}`nState: {2}`n`n{3}" -f $d.number,$d.title,$d.state,$d.body.Substring(0,[Math]::Min(2000,$d.body.Length))

Read issue comments:
  On macOS/Linux:
    curl -s "https://api.github.com/repos/openclaw/openclaw/issues/NUMBER/comments" | python3 -c "import sys,json; [print(f'@{c[\"user\"][\"login\"]}:\n{c[\"body\"][:500]}\n') for c in json.load(sys.stdin)]"
  On Windows:
    (Invoke-RestMethod "https://api.github.com/repos/openclaw/openclaw/issues/NUMBER/comments") | ForEach-Object { "@{0}:`n{1}`n" -f $_.user.login,$_.body.Substring(0,[Math]::Min(500,$_.body.Length)) }

Tips:
- Use key phrases from the error message as search terms.
- Include closed issues too — they often contain fixes: add +is:closed to KEYWORDS.
- If rate-limited (HTTP 403), wait and retry or narrow the search.
