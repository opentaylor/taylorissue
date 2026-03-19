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
        $prevEAP = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        $nodeVersion = (node -v 2>$null)
        $ErrorActionPreference = $prevEAP
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

function Add-ToProcessPath {
    param([Parameter(Mandatory)][string]$PathEntry)
    if ([string]::IsNullOrWhiteSpace($PathEntry)) { return }
    $entries = @($env:Path -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($entries | Where-Object { $_ -ieq $PathEntry }) { return }
    $env:Path = "$PathEntry;$env:Path"
}

function Get-PortableNodeRoot {
    return (Join-Path $env:LOCALAPPDATA "taylorissue\deps\nodejs")
}

function Use-PortableNodeIfPresent {
    $root = Get-PortableNodeRoot
    $nodeExe = Join-Path $root "node.exe"
    if (-not (Test-Path $nodeExe)) { return $false }
    Add-ToProcessPath $root
    return (Check-Node)
}

function Resolve-LatestNodeLtsVersion {
    $indexUrl = if ($script:USE_CN) {
        "https://npmmirror.com/mirrors/node/index.json"
    } else {
        "https://nodejs.org/dist/index.json"
    }
    $index = Invoke-RestMethod -Uri $indexUrl -UseBasicParsing
    $lts = $index | Where-Object { $_.lts -and $_.lts -ne $false } |
        Where-Object {
            $major = [int]($_.version -replace 'v(\d+)\..*', '$1')
            $major -ge $script:NEED_VERSION
        } | Select-Object -First 1
    if (-not $lts) { throw "Could not find Node.js LTS >= v$($script:NEED_VERSION)" }
    return $lts.version
}

function Install-PortableNode {
    if (Use-PortableNodeIfPresent) {
        Write-OK "User-local Node.js already available"
        return
    }

    Write-Info "Downloading portable Node.js..."
    $version = Resolve-LatestNodeLtsVersion
    $zipName = "node-${version}-win-x64.zip"
    $baseUrl = if ($script:USE_CN) { $script:CN_NODE_MIRROR } else { "https://nodejs.org/dist/" }
    $url = "${baseUrl}${version}/${zipName}"

    $portableRoot = Get-PortableNodeRoot
    $portableParent = Split-Path -Parent $portableRoot
    $tmpZip = Join-Path $env:TEMP $zipName
    $tmpExtract = Join-Path $env:TEMP ("openclaw-node-" + [guid]::NewGuid().ToString("N"))

    New-Item -ItemType Directory -Force -Path $portableParent | Out-Null
    if (Test-Path $portableRoot) { Remove-Item -Recurse -Force $portableRoot }
    if (Test-Path $tmpExtract)   { Remove-Item -Recurse -Force $tmpExtract }
    New-Item -ItemType Directory -Force -Path $tmpExtract | Out-Null

    try {
        Write-Info "Downloading ${version}..."
        Invoke-WebRequest -Uri $url -OutFile $tmpZip -UseBasicParsing
        Write-Info "Extracting..."
        Expand-Archive -Path $tmpZip -DestinationPath $tmpExtract -Force
        $inner = Get-ChildItem -Path $tmpExtract -Directory | Select-Object -First 1
        if ($inner) {
            Move-Item -Path $inner.FullName -Destination $portableRoot -Force
        } else {
            Move-Item -Path (Join-Path $tmpExtract "*") -Destination $portableRoot -Force
        }
    } finally {
        if (Test-Path $tmpZip)     { Remove-Item -Force $tmpZip }
        if (Test-Path $tmpExtract) { Remove-Item -Recurse -Force $tmpExtract }
    }

    Add-ToProcessPath $portableRoot
    $currentUserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not ($currentUserPath -split ";" | Where-Object { $_ -ieq $portableRoot })) {
        [Environment]::SetEnvironmentVariable("Path", "$portableRoot;$currentUserPath", "User")
        Write-Info "Added $portableRoot to user PATH"
    }

    if (-not (Use-PortableNodeIfPresent)) {
        throw "Portable Node.js download completed, but node is still unavailable."
    }
    Write-OK "User-local Node.js ready: $(node -v)"
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

    # Portable fallback — no admin needed
    Write-Info "Package managers unavailable or failed, trying portable Node.js..."
    try {
        Install-PortableNode
        if (Check-Node) { return }
    } catch {
        Write-Warn "Portable Node.js install failed: $($_.Exception.Message)"
    }

    Write-Die "Could not install Node.js. Please install Node.js ${script:NEED_VERSION}+ manually: https://nodejs.org/en/download/"
}

function Check-Npm {
    try {
        $prevEAP = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        $npmVer = (npm -v 2>$null)
        $ErrorActionPreference = $prevEAP
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
        Use-PortableNodeIfPresent | Out-Null
    }

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
