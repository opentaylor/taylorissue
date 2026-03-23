Install the OpenClaw skill "{slug}" from ClawHub.

Step 1: Run the install command.
  clawhub install --force {slug}
If `clawhub` is not found on PATH, try:
  npx -y clawhub install --force {slug}

Step 2: Copy skill to the app's skills directory.
On macOS/Linux:
  mkdir -p ~/.openclaw/skills && cp -R ~/.openclaw/workspace/skills/{slug} ~/.openclaw/skills/{slug}
On Windows:
  New-Item -ItemType Directory -Force "$env:USERPROFILE\.openclaw\skills" | Out-Null; Copy-Item -Recurse -Force "$env:USERPROFILE\.openclaw\workspace\skills\{slug}" "$env:USERPROFILE\.openclaw\skills\{slug}"

Step 3: Verify installation.
On macOS/Linux: test -f ~/.openclaw/skills/{slug}/SKILL.md && echo "ok"
On Windows: if (Test-Path "$env:USERPROFILE\.openclaw\skills\{slug}\SKILL.md") { "ok" }

Respond with ONLY this JSON:
{"success": true, "details": "<brief summary>"}
On failure: {"success": false, "error": "<reason>"}