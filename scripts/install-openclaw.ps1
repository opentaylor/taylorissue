$ErrorActionPreference = "Stop"

function Write-OK   { param([string]$Msg) Write-Host "[OK] $Msg" -ForegroundColor Green }
function Write-Info { param([string]$Msg) Write-Host "[.] $Msg" -ForegroundColor Gray }
function Write-Warn { param([string]$Msg) Write-Host "[!] $Msg" -ForegroundColor Yellow }
function Write-Die  { param([string]$Msg) Write-Host "[X] $Msg" -ForegroundColor Red; exit 1 }

$script:USE_CN = $false
$script:CN_NPM = "https://registry.npmmirror.com"

function Detect-CN {
    try {
        $tz = (Get-TimeZone).Id
        if ($tz -eq "China Standard Time") {
            $script:USE_CN = $true
            return
        }
    } catch {}

    try {
        $cnTime = (Measure-Command {
            Invoke-WebRequest -Uri $script:CN_NPM -UseBasicParsing -TimeoutSec 3 -ErrorAction Stop | Out-Null
        }).TotalSeconds
        $intTime = (Measure-Command {
            Invoke-WebRequest -Uri "https://registry.npmjs.org/" -UseBasicParsing -TimeoutSec 3 -ErrorAction Stop | Out-Null
        }).TotalSeconds
        if ($intTime -gt 0.5 -and $cnTime -lt ($intTime * 0.6)) {
            $script:USE_CN = $true
        }
    } catch {}
}

function Resolve-CommandPath {
    param([Parameter(Mandatory)][string[]]$Candidates)
    foreach ($c in $Candidates) {
        $cmd = Get-Command $c -ErrorAction SilentlyContinue
        if ($cmd -and $cmd.Source) { return $cmd.Source }
    }
    return $null
}

function Get-NpmCommandPath {
    $p = Resolve-CommandPath -Candidates @("npm.cmd", "npm.exe", "npm")
    if (-not $p) { throw "npm not found on PATH." }
    return $p
}

function Add-ToProcessPath {
    param([Parameter(Mandatory)][string]$PathEntry)
    if ([string]::IsNullOrWhiteSpace($PathEntry)) { return }
    $entries = @($env:Path -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($entries | Where-Object { $_ -ieq $PathEntry }) { return }
    $env:Path = "$PathEntry;$env:Path"
}

function Refresh-PathFromRegistry {
    $env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [Environment]::GetEnvironmentVariable("Path", "User")
}

function Install-OpenClaw {
    $spec = "openclaw@latest"

    try {
        $npmCmd = Get-NpmCommandPath
        $regArgs = @()
        if ($script:USE_CN) { $regArgs = @("--registry", $script:CN_NPM) }
        $resolved = & $npmCmd view $spec version @regArgs 2>$null
        if ($resolved) { Write-Info "Target: OpenClaw v$($resolved.Trim())" }
    } catch {}

    Write-Info "Running: npm install -g $spec"

    $npmCmd = Get-NpmCommandPath
    & $npmCmd cache clean --force 2>$null | Out-Null

    $prevLogLevel          = $env:NPM_CONFIG_LOGLEVEL
    $prevUpdateNotifier    = $env:NPM_CONFIG_UPDATE_NOTIFIER
    $prevFund              = $env:NPM_CONFIG_FUND
    $prevAudit             = $env:NPM_CONFIG_AUDIT
    $prevScriptShell       = $env:NPM_CONFIG_SCRIPT_SHELL
    $prevNodeLlamaSkip     = $env:NODE_LLAMA_CPP_SKIP_DOWNLOAD
    $prevSharpIgnore       = $env:SHARP_IGNORE_GLOBAL_LIBVIPS

    $env:NPM_CONFIG_LOGLEVEL          = "error"
    $env:NPM_CONFIG_UPDATE_NOTIFIER   = "false"
    $env:NPM_CONFIG_FUND              = "false"
    $env:NPM_CONFIG_AUDIT             = "false"
    $env:NPM_CONFIG_SCRIPT_SHELL      = "cmd.exe"
    $env:NODE_LLAMA_CPP_SKIP_DOWNLOAD = "1"
    $env:SHARP_IGNORE_GLOBAL_LIBVIPS  = "1"

    $installArgs = @("install", "-g", $spec)
    if ($script:USE_CN) { $installArgs += @("--registry", $script:CN_NPM) }

    try {
        $npmOutput = & $npmCmd @installArgs 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-OK "OpenClaw npm package installed"
            return
        }

        Write-Warn "First attempt failed - cleaning and retrying"
        & $npmCmd cache clean --force 2>$null | Out-Null

        try {
            $npmRoot = (& $npmCmd root -g 2>$null).Trim()
            if ($npmRoot) {
                $ocDir = Join-Path $npmRoot "openclaw"
                if (Test-Path $ocDir) { Remove-Item -Recurse -Force $ocDir 2>$null }
            }
        } catch {}

        $npmOutput = & $npmCmd @installArgs 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-OK "OpenClaw npm package installed (retry succeeded)"
            return
        }

        Write-Warn "npm install failed. Output:"
        $npmOutput | ForEach-Object { Write-Host $_ }
        Write-Die "Could not install OpenClaw. See output above."
    } finally {
        $env:NPM_CONFIG_LOGLEVEL          = $prevLogLevel
        $env:NPM_CONFIG_UPDATE_NOTIFIER   = $prevUpdateNotifier
        $env:NPM_CONFIG_FUND              = $prevFund
        $env:NPM_CONFIG_AUDIT             = $prevAudit
        $env:NPM_CONFIG_SCRIPT_SHELL      = $prevScriptShell
        $env:NODE_LLAMA_CPP_SKIP_DOWNLOAD = $prevNodeLlamaSkip
        $env:SHARP_IGNORE_GLOBAL_LIBVIPS  = $prevSharpIgnore
    }
}

function Get-NpmGlobalBinCandidates {
    param([string]$NpmPrefix)
    $candidates = @()
    if (-not [string]::IsNullOrWhiteSpace($NpmPrefix)) {
        $candidates += $NpmPrefix
        $candidates += (Join-Path $NpmPrefix "bin")
    }
    if (-not [string]::IsNullOrWhiteSpace($env:APPDATA)) {
        $candidates += (Join-Path $env:APPDATA "npm")
    }
    return $candidates | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique
}

function Verify-OpenClaw {
    Refresh-PathFromRegistry

    $npmPrefix = $null
    try {
        $npmPrefix = (& (Get-NpmCommandPath) config get prefix 2>$null).Trim()
    } catch {}

    $npmBins = Get-NpmGlobalBinCandidates -NpmPrefix $npmPrefix
    foreach ($bin in $npmBins) {
        if (Test-Path $bin) { Add-ToProcessPath $bin }
    }

    $ocCmd = Get-Command openclaw.cmd -ErrorAction SilentlyContinue
    if (-not $ocCmd) { $ocCmd = Get-Command openclaw -ErrorAction SilentlyContinue }

    if ($ocCmd -and $ocCmd.Source) {
        try {
            $ver = (& $ocCmd.Source --version 2>$null | Select-Object -First 1).Trim()
            Write-OK "OpenClaw installed successfully ($ver)"
        } catch {
            Write-OK "OpenClaw installed successfully"
        }
    } else {
        Write-Warn "Installed, but 'openclaw' not on PATH"
        Write-Info "Try opening a new terminal."
        if ($npmBins.Count -gt 0) {
            Write-Info "Or add one of these to your PATH:"
            foreach ($b in $npmBins) { Write-Info "  $b" }
        }
    }
}

function Main {
    Write-Host ""
    Write-Host "  Install OpenClaw" -ForegroundColor White
    Write-Host ""
    Detect-CN
    if ($script:USE_CN) { Write-OK "China mainland detected - npm mirror: $($script:CN_NPM)" }
    Install-OpenClaw
    Verify-OpenClaw
}

Main
