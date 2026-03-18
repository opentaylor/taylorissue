$ErrorActionPreference = "Stop"

function Write-OK   { param([string]$Msg) Write-Host "[OK] $Msg" -ForegroundColor Green }
function Write-Info { param([string]$Msg) Write-Host "[.] $Msg" -ForegroundColor Gray }
function Write-Warn { param([string]$Msg) Write-Host "[!] $Msg" -ForegroundColor Yellow }
function Write-Die  { param([string]$Msg) Write-Host "[X] $Msg" -ForegroundColor Red; exit 1 }

$script:USE_CN = $false
$script:CN_NODE_MIRROR = "https://npmmirror.com/mirrors/node/"
$script:NEED_VERSION = 22

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
            Invoke-WebRequest -Uri "https://registry.npmmirror.com" -UseBasicParsing -TimeoutSec 3 -ErrorAction Stop | Out-Null
        }).TotalSeconds
        $intTime = (Measure-Command {
            Invoke-WebRequest -Uri "https://registry.npmjs.org/" -UseBasicParsing -TimeoutSec 3 -ErrorAction Stop | Out-Null
        }).TotalSeconds
        if ($intTime -gt 0.5 -and $cnTime -lt ($intTime * 0.6)) {
            $script:USE_CN = $true
        }
    } catch {}
}

function Refresh-PathFromRegistry {
    $env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [Environment]::GetEnvironmentVariable("Path", "User")
}

function Check-Node {
    try {
        $nodeVersion = (node -v 2>$null)
        if ($nodeVersion) {
            $major = [int]($nodeVersion -replace 'v(\d+)\..*', '$1')
            if ($major -ge $script:NEED_VERSION) {
                Write-OK "Node.js $nodeVersion"
                return $true
            } else {
                Write-Warn "Node.js $nodeVersion found, but v${script:NEED_VERSION}+ required"
                return $false
            }
        }
    } catch {
        Write-Info "Node.js not found"
        return $false
    }
    return $false
}

function Install-Node {
    Write-Info "Installing Node.js..."

    # winget
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        Write-Info "Using winget..."
        winget install OpenJS.NodeJS.LTS --source winget --accept-package-agreements --accept-source-agreements
        Refresh-PathFromRegistry
        if (Check-Node) {
            Write-OK "Node.js installed via winget"
            return
        }
        Write-Warn "winget completed, but Node.js is still unavailable in this shell"
    }

    # Chocolatey
    if (Get-Command choco -ErrorAction SilentlyContinue) {
        Write-Info "Using Chocolatey..."
        choco install nodejs-lts -y
        Refresh-PathFromRegistry
        if (Check-Node) {
            Write-OK "Node.js installed via Chocolatey"
            return
        }
        Write-Warn "Chocolatey completed, but Node.js is still unavailable in this shell"
    }

    # Scoop
    if (Get-Command scoop -ErrorAction SilentlyContinue) {
        Write-Info "Using Scoop..."
        scoop install nodejs-lts
        Refresh-PathFromRegistry
        if (Check-Node) {
            Write-OK "Node.js installed via Scoop"
            return
        }
        Write-Warn "Scoop completed, but Node.js is still unavailable in this shell"
    }

    Write-Die "Could not find a package manager (winget, choco, or scoop). Please install Node.js ${script:NEED_VERSION}+ manually: https://nodejs.org/en/download/"
}

function Check-Npm {
    try {
        $npmVer = (npm -v 2>$null)
        if ($npmVer) {
            Write-OK "npm $npmVer"
            return $true
        }
    } catch {}
    return $false
}

function Main {
    Write-Host ""
    Write-Host "  Install Node.js" -ForegroundColor White
    Write-Host ""
    Detect-CN
    if ($script:USE_CN) { Write-OK "China mainland detected" }

    if (-not (Check-Node)) {
        Install-Node
        Refresh-PathFromRegistry
        if (-not (Check-Node)) {
            Write-Die "Node.js installation may require a terminal restart. Please close this terminal, open a new one, and try again."
        }
    }

    if (-not (Check-Npm)) {
        Write-Die "npm not found after Node.js install"
    }
}

Main
