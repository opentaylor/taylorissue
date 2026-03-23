Detect the current system environment.

On macOS/Linux:
  uname -s && uname -r && uname -m && df -h /
On macOS also run: sw_vers
On Windows:
  $o = Get-CimInstance Win32_OperatingSystem; $d = Get-PSDrive C; Write-Output ("OS=" + $o.Caption.Trim() + " " + $o.Version); Write-Output ("ARCH=" + $env:PROCESSOR_ARCHITECTURE); Write-Output ("DISK_GB=" + [math]::Round($d.Free / 1GB, 2))

Respond with ONLY this JSON:
{"success": true, "os": "<OS name and version>", "arch": "<CPU architecture>", "disk_free": "<free disk space with unit>"}
On failure: {"success": false, "error": "<reason>"}