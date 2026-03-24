Detect the current system environment by running this exact command using the {tool_name} tool:

{detect_command}

You MUST call the {tool_name} tool with this command. Do NOT respond with JSON until you have executed the command and received its output.

Respond with ONLY this JSON:
{"success": true, "os": "<OS name and version>", "arch": "<CPU architecture>", "disk_free": "<free disk space with unit>"}
On failure: {"success": false, "error": "<reason>"}
