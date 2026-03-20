{label}

On macOS/Linux, run this exact command:
  bash {script_path}

On Windows (PowerShell), run this exact command:
  & '{script_path}'

Do NOT modify the script or download a different one.
The script handles privilege escalation internally — do NOT prepend sudo or Run-As.
Pass "timeout": 300 in the tool call because this command may take several minutes.

After the script finishes, verify: {verify_cmd}
Respond with ONLY this JSON:
{json_tpl}
On failure: {"success": false, "error": "<reason>"}
