Install the OpenClaw skill "{slug}" from ClawHub.

Run this exact command:
  clawhub install --force {slug}

If `clawhub` is not found on PATH, try:
  npx -y clawhub install --force {slug}

After the command finishes, verify: ls ~/.openclaw/skills/{slug}/SKILL.md

Respond with ONLY this JSON:
{"success": true, "details": "<brief summary>"}
On failure: {"success": false, "error": "<reason>"}