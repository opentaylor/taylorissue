Fix strategy: the dist/control-ui/ directory is missing or incomplete in the installed OpenClaw package.
This is a known issue with version 2026.3.22 where the npm package was published without the built UI assets.

Step 1: Find the install path and version.
On macOS/Linux:
  OPENCLAW_DIR=$(node -e "try{console.log(require.resolve('openclaw/package.json').replace('/package.json',''))}catch{}")
  VERSION=$(node -e "console.log(require('openclaw/package.json').version)")
On Windows:
  $openclawDir = node -e "try{console.log(require.resolve('openclaw/package.json').replace('/package.json','').replace('\\\\','/'))}catch{}"
  $VERSION = node -e "console.log(require('openclaw/package.json').version)"

Step 2: Clone the matching tag and build the UI from source.
On macOS/Linux:
  cd /tmp && git clone --depth 1 --branch "v$VERSION" https://github.com/openclaw/openclaw.git openclaw-src
  cd openclaw-src && npm install && npx pnpm ui:build
On Windows:
  cd $env:TEMP; git clone --depth 1 --branch "v$VERSION" https://github.com/openclaw/openclaw.git openclaw-src
  cd openclaw-src; npm install; npx pnpm ui:build

Step 3: Copy the built UI into the installed package.
On macOS/Linux:
  cp -r /tmp/openclaw-src/dist/control-ui "$OPENCLAW_DIR/dist/"
On Windows:
  Copy-Item -Recurse "$env:TEMP\openclaw-src\dist\control-ui" "$openclawDir\dist\" -Force

If copying fails with a permission error, retry the copy command using the `root` tool.

Step 4: Clean up.
On macOS/Linux: rm -rf /tmp/openclaw-src
On Windows: Remove-Item -Recurse -Force "$env:TEMP\openclaw-src"

Step 5: Restart the gateway as a persistent service and verify.
  openclaw gateway install --force
  openclaw gateway start
On macOS/Linux: sleep 5
On Windows: Start-Sleep -Seconds 5
Then verify:
On macOS/Linux: ls "$OPENCLAW_DIR/dist/control-ui/index.html" && echo "Fix verified: control-ui restored"
On Windows: if (Test-Path "$openclawDir\dist\control-ui\index.html") { "Fix verified: control-ui restored" }
