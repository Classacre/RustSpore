@echo off
REM Build RustSpore with the MSVC toolchain.
REM
REM The Rust workspace targets x86_64-pc-windows-msvc, which needs a C linker.
REM Without the MSVC environment the build fails because `link.exe` resolves to
REM PortableGit's GNU `link`, and MSVC's CRT import libraries are absent.
REM
REM This script locates Visual Studio with vswhere, picks the newest MSVC toolset
REM and Windows SDK, and sets PATH, LIB and INCLUDE directly. It deliberately does
REM not invoke vcvars64.bat, which shells out to reg.exe.
REM
REM Usage:
REM   tools\build.bat build --workspace
REM   tools\build.bat test --workspace
REM   tools\build.bat run -p spore-bevy -- "C:\path\to\Data\Spore_Game.package"

setlocal enabledelayedexpansion

set "PF86=%ProgramFiles(x86)%"
set "VSWHERE=%PF86%\Microsoft Visual Studio\Installer\vswhere.exe"

REM --- Locate Visual Studio -------------------------------------------------
set "VSDIR="
if exist "%VSWHERE%" (
    for /f "usebackq delims=" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do (
        if not defined VSDIR set "VSDIR=%%i"
    )
)

REM --- Newest MSVC toolset --------------------------------------------------
set "MSVC="
if defined VSDIR (
    set "MSVCROOT=!VSDIR!\VC\Tools\MSVC"
    if exist "!MSVCROOT!" (
        for /f "delims=" %%d in ('dir /b /ad /o-n "!MSVCROOT!" 2^>nul') do (
            if not defined MSVC if exist "!MSVCROOT!\%%d\bin\Hostx64\x64\link.exe" set "MSVC=!MSVCROOT!\%%d"
        )
    )
)

REM --- Newest Windows SDK with a complete x64 ucrt ---------------------------
set "SDKROOT=%PF86%\Windows Kits\10"
set "SDKVER="
if exist "%SDKROOT%\Lib" (
    for /f "delims=" %%d in ('dir /b /ad /o-n "%SDKROOT%\Lib" 2^>nul') do (
        if not defined SDKVER if exist "%SDKROOT%\Lib\%%d\ucrt\x64\ucrt.lib" set "SDKVER=%%d"
    )
)

REM --- Report what is missing, rather than failing with LNK1181 --------------
if not defined VSDIR goto :no_vs
if not defined MSVC goto :no_tools
if not defined SDKVER goto :no_sdk

echo Using MSVC %MSVC%
echo Using Windows SDK %SDKROOT%\Lib\%SDKVER%

REM Prepend MSVC bin so the real link.exe is found before Git's GNU link.
set "PATH=%MSVC%\bin\Hostx64\x64;%PATH%"
set "LIB=%MSVC%\lib\x64;%SDKROOT%\Lib\%SDKVER%\ucrt\x64;%SDKROOT%\Lib\%SDKVER%\um\x64"
set "INCLUDE=%MSVC%\include;%SDKROOT%\Include\%SDKVER%\shared;%SDKROOT%\Include\%SDKVER%\ucrt;%SDKROOT%\Include\%SDKVER%\um"

cd /d "%~dp0.."
cargo %*
exit /b %ERRORLEVEL%

:no_vs
echo ERROR: Visual Studio with the C++ toolset was not found.
echo Install "Desktop development with C++" from the Visual Studio installer.
exit /b 1

:no_tools
echo ERROR: no MSVC toolset with a Hostx64\x64 link.exe under "%VSDIR%\VC\Tools\MSVC".
echo Repair the "Desktop development with C++" workload.
exit /b 1

:no_sdk
echo ERROR: no Windows SDK with Lib\<version>\ucrt\x64\ucrt.lib under "%SDKROOT%".
echo Install a Windows 10 or 11 SDK from the Visual Studio installer.
exit /b 1
