@echo off
if /i "%~1"=="--install-arm64-msvc" goto :install_arm64_msvc

setlocal EnableExtensions EnableDelayedExpansion
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>&1
if errorlevel 1 (echo cargo not found & exit /b 1)

where npm.cmd >nul 2>&1
if not errorlevel 1 set "NPM_CMD=npm.cmd"
if not defined NPM_CMD if exist "%ProgramFiles%\nodejs\npm.cmd" set "NPM_CMD=%ProgramFiles%\nodejs\npm.cmd"
if not defined NPM_CMD (echo npm.cmd not found & exit /b 1)

if /i "!PROCESSOR_ARCHITECTURE!"=="ARM64" (set "VCARCH=arm64" & set "LINKSUB=bin\Hostarm64\arm64") else (set "VCARCH=x64" & set "LINKSUB=bin\Hostx64\x64")

set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "!VSWHERE!" set "VSWHERE=%ProgramFiles%\Microsoft Visual Studio\Installer\vswhere.exe"
set "VSINSTALL="
if exist "!VSWHERE!" for /f "usebackq delims=" %%i in (`"!VSWHERE!" -latest -products Microsoft.VisualStudio.Product.BuildTools -property installationPath`) do set "VSINSTALL=%%i"
if not defined VSINSTALL if exist "!VSWHERE!" for /f "usebackq delims=" %%i in (`"!VSWHERE!" -latest -property installationPath`) do set "VSINSTALL=%%i"
if not exist "!VSINSTALL!\VC\Auxiliary\Build\vcvarsall.bat" (echo MSVC Build Tools not found & exit /b 1)

call "!VSINSTALL!\VC\Auxiliary\Build\vcvarsall.bat" !VCARCH!
where link >nul 2>&1
if errorlevel 1 (
  set "L="
  for /d %%M in ("!VSINSTALL!\VC\Tools\MSVC\*") do if exist "%%M\!LINKSUB!\link.exe" set "L=%%M\!LINKSUB!"
  if defined L (set "PATH=!L!;%PATH%") else (echo link.exe not found & exit /b 1)
)

powershell -NoProfile -ExecutionPolicy Bypass -Command "Get-NetTCPConnection -LocalPort 1420 -State Listen -EA 0|ForEach-Object OwningProcess|Sort-Object -Unique|ForEach-Object{Stop-Process -Id $_ -Force -EA 0}"
cd /d "%~dp0.."
call "!NPM_CMD!" run tauri dev
exit /b !ERRORLEVEL!

:install_arm64_msvc
setlocal EnableExtensions EnableDelayedExpansion
set "S=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\setup.exe"
set "W=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "!S!" exit /b 1
set "BSP="
for /f "usebackq delims=" %%i in (`"!W!" -latest -products Microsoft.VisualStudio.Product.BuildTools -property installationPath`) do set "BSP=%%i"
if not defined BSP exit /b 1
"!S!" modify --installPath "!BSP!" --add Microsoft.VisualStudio.Component.VC.Tools.ARM64 --passive --norestart
exit /b !ERRORLEVEL!
