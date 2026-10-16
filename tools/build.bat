@echo off
REM Build RustSpore with the MSVC toolchain.
REM
REM The Rust workspace targets x86_64-pc-windows-msvc, which needs a C linker.
REM Without vcvars the build fails because `link.exe` resolves to PortableGit's
REM GNU `link`, and MSVC's CRT import libraries are absent from the environment.
REM This script loads the MSVC environment first, then runs cargo from the repo
REM root with whatever arguments are passed through.
REM
REM Usage:
REM   tools\build.bat build --workspace
REM   tools\build.bat test -p dbpf
REM   tools\build.bat run -p spore-bevy -- "C:\path\to\Data\Spore_Game.package"

setlocal

set "VCVARS=C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat"
if not exist "%VCVARS%" (
    echo ERROR: vcvars64.bat not found at "%VCVARS%".
    echo Install Visual Studio Build Tools with the "Desktop development with C++" workload.
    exit /b 1
)

call "%VCVARS%" >nul
if errorlevel 1 (
    echo ERROR: vcvars64.bat failed.
    exit /b 1
)

cd /d "%~dp0.."
cargo %*
