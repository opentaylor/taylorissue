Check the integrity of the installed OpenClaw package. DO NOT fix anything.

Step 1: Find the OpenClaw install path.
On macOS/Linux: OPENCLAW_DIR=$(node -e "try{console.log(require.resolve('openclaw/package.json').replace('/package.json',''))}catch{}" 2>/dev/null); echo "INSTALL_DIR=$OPENCLAW_DIR"
On Windows: $openclawDir = node -e "try{console.log(require.resolve('openclaw/package.json').replace('/package.json','').replace('\\\\','/'))}catch{}"; echo "INSTALL_DIR=$openclawDir"

Step 2: Get the installed version.
  node -e "try { console.log(require('openclaw/package.json').version) } catch(e) { console.log('unknown') }"

Step 3: Check if dist/control-ui/ directory exists and contains index.html.
On macOS/Linux: ls "$OPENCLAW_DIR/dist/control-ui/index.html" 2>&1 && echo "control_ui_exists=true" || echo "control_ui_exists=false"
On Windows: if (Test-Path "$openclawDir\dist\control-ui\index.html") { "control_ui_exists=true" } else { "control_ui_exists=false" }

Step 4: Count the number of files under dist/control-ui/.
On macOS/Linux: find "$OPENCLAW_DIR/dist/control-ui" -type f 2>/dev/null | wc -l | tr -d ' '
On Windows: (Get-ChildItem -Recurse "$openclawDir\dist\control-ui" -File -EA SilentlyContinue).Count

Step 5: Check if dist/control-ui/assets/ directory exists and has JS/CSS files.
On macOS/Linux: ls "$OPENCLAW_DIR/dist/control-ui/assets/"*.js 2>/dev/null | head -3 && echo "assets_ok=true" || echo "assets_ok=false"
On Windows: if ((Get-ChildItem "$openclawDir\dist\control-ui\assets\*.js" -EA SilentlyContinue).Count -gt 0) { "assets_ok=true" } else { "assets_ok=false" }

Respond with ONLY this JSON:
{"success": true, "version": "<installed version>", "control_ui_present": <true if dist/control-ui/index.html exists>, "asset_count": <number of files in dist/control-ui/>, "assets_ok": <true if assets/ has JS/CSS files>, "details": "<description of what was found>"}
