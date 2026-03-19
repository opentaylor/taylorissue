$ErrorActionPreference = "Stop"

function Write-OK   { param([string]$Msg) Write-Host "[OK] $Msg" -ForegroundColor Green }
function Write-Info { param([string]$Msg) Write-Host "[.] $Msg" -ForegroundColor Gray }
function Write-Warn { param([string]$Msg) Write-Host "[!] $Msg" -ForegroundColor Yellow }
function Write-Die  { param([string]$Msg) Write-Host "[X] $Msg" -ForegroundColor Red; exit 1 }

$script:USE_CN = $false
$script:CN_GITHUB_PROXY = "https://ghp.ci/"

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

function Check-Git {
    try {
        $null = Get-Command git -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

function Get-PortableGitRoot {
    $base = Join-Path $env:LOCALAPPDATA "taylorissue\deps"
    return (Join-Path $base "portable-git")
}

function Get-PortableGitCommandPath {
    $root = Get-PortableGitRoot
    foreach ($candidate in @(
        (Join-Path $root "mingw64\bin\git.exe"),
        (Join-Path $root "cmd\git.exe"),
        (Join-Path $root "bin\git.exe"),
        (Join-Path $root "git.exe")
    )) {
        if (Test-Path $candidate) { return $candidate }
    }
    return $null
}

function Use-PortableGitIfPresent {
    $gitExe = Get-PortableGitCommandPath
    if (-not $gitExe) { return $false }

    $portableRoot = Get-PortableGitRoot
    foreach ($pathEntry in @(
        (Join-Path $portableRoot "mingw64\bin"),
        (Join-Path $portableRoot "usr\bin"),
        (Split-Path -Parent $gitExe)
    )) {
        if (Test-Path $pathEntry) { Add-ToProcessPath $pathEntry }
    }
    return (Check-Git)
}

function Resolve-PortableGitDownload {
    $releaseApi = "https://api.github.com/repos/git-for-windows/git/releases/latest"
    $headers = @{
        "User-Agent" = "openclaw-installer"
        "Accept"     = "application/vnd.github+json"
    }
    $release = Invoke-RestMethod -Uri $releaseApi -Headers $headers
    if (-not $release -or -not $release.assets) {
        throw "Could not resolve latest git-for-windows release metadata."
    }

    $asset = $release.assets |
        Where-Object { $_.name -match '^MinGit-.*-64-bit\.zip$' -and $_.name -notmatch 'busybox' } |
        Select-Object -First 1
    if (-not $asset) {
        throw "Could not find a MinGit zip asset in the latest git-for-windows release."
    }

    $url = $asset.browser_download_url
    if ($script:USE_CN) {
        $url = $script:CN_GITHUB_PROXY + $url
    }

    return @{
        Tag  = $release.tag_name
        Name = $asset.name
        Url  = $url
    }
}

function Install-PortableGit {
    if (Use-PortableGitIfPresent) {
        $ver = (& git --version 2>$null)
        if ($ver) { Write-OK "User-local Git already available: $ver" }
        return
    }

    Write-Info "Git not found; bootstrapping user-local portable Git..."

    $download = Resolve-PortableGitDownload
    $portableRoot = Get-PortableGitRoot
    $portableParent = Split-Path -Parent $portableRoot
    $tmpZip = Join-Path $env:TEMP $download.Name
    $tmpExtract = Join-Path $env:TEMP ("openclaw-portable-git-" + [guid]::NewGuid().ToString("N"))

    New-Item -ItemType Directory -Force -Path $portableParent | Out-Null
    if (Test-Path $portableRoot) { Remove-Item -Recurse -Force $portableRoot }
    if (Test-Path $tmpExtract)   { Remove-Item -Recurse -Force $tmpExtract }
    New-Item -ItemType Directory -Force -Path $tmpExtract | Out-Null

    try {
        Write-Info "Downloading $($download.Tag)..."
        Invoke-WebRequest -Uri $download.Url -OutFile $tmpZip -UseBasicParsing
        Expand-Archive -Path $tmpZip -DestinationPath $tmpExtract -Force
        Move-Item -Path (Join-Path $tmpExtract "*") -Destination $portableRoot -Force
    } finally {
        if (Test-Path $tmpZip)     { Remove-Item -Force $tmpZip }
        if (Test-Path $tmpExtract) { Remove-Item -Recurse -Force $tmpExtract }
    }

    if (-not (Use-PortableGitIfPresent)) {
        throw "Portable Git bootstrap completed, but git is still unavailable."
    }

    $ver = (& git --version 2>$null)
    Write-OK "User-local Git ready: $ver"
}

function Ensure-Git {
    if (Check-Git) {
        $ver = (& git --version 2>$null)
        Write-OK "Git $ver"
        return
    }

    Refresh-PathFromRegistry
    if (Check-Git) {
        $ver = (& git --version 2>$null)
        Write-OK "Git $ver"
        return
    }

    if (Use-PortableGitIfPresent) {
        $ver = (& git --version 2>$null)
        Write-OK "Git $ver"
        return
    }

    try {
        Install-PortableGit
        if (Check-Git) { return }
    } catch {
        Write-Warn "Portable Git bootstrap failed: $($_.Exception.Message)"
    }

    Write-Die "Git is required but could not be installed. Install Git for Windows manually: https://git-scm.com/download/win"
}

function Main {
    Write-Host ""
    Write-Host "  Install Git" -ForegroundColor White
    Write-Host ""
    Detect-CN
    if ($script:USE_CN) { Write-OK "China mainland detected - GitHub mirror enabled" }
    Ensure-Git
}

Main
