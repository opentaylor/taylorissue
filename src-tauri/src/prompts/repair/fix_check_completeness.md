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
  cd /tmp && git clone --depth 1 --branch "v$VERSION" https://github.com/openclaw/openclaw.git openclaw-src
  cd openclaw-src && npm install && npx pnpm ui:build

Step 3: Copy the built UI into the installed package.
On macOS/Linux:
  cp -r /tmp/openclaw-src/dist/control-ui "$OPENCLAW_DIR/dist/"
On Windows:
  Copy-Item -Recurse /tmp/openclaw-src/dist/control-ui "$openclawDir\dist\" -Force

If copying fails with a permission error, retry the copy command using the `root` tool.

Step 4: Clean up.
  rm -rf /tmp/openclaw-src

Step 5: Restart the gateway and verify.
On macOS/Linux: openclaw gateway stop; nohup openclaw gateway &
On Windows: openclaw gateway stop; Start-Process powershell -ArgumentList '-NoExit','-Command','openclaw gateway'
Then verify: ls "$OPENCLAW_DIR/dist/control-ui/index.html" && echo "Fix verified: control-ui restored"
