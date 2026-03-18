{label}

On macOS/Linux, run this exact command:
  bash {script_path}

On Windows (PowerShell), run this exact command:
  & '{script_path}'

Do NOT use sudo. Do NOT modify the script or download a different one.
Set timeout to 300 for this command.

After the script finishes, verify: {verify_cmd}
Respond with ONLY this JSON:
{json_tpl}
On failure: {"success": false, "error": "<reason>"}
