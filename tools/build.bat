@echo off
REM Build RustSpore with the MSVC toolchain.
REM
REM The Rust workspace targets x86_64-pc-windows-msvc, which needs a C linker.
REM Without the MSVC environment the build fails because `link.exe` resolves to
REM PortableGit's GNU `link`, and MSVC's CRT import libraries are absent.
REM
REM This script sets LIB, INCLUDE and PATH directly so it never invokes
REM vcvars64.bat (which calls reg.exe, blocked by the sandbox).
REM
REM Usage:
REM   tools\build.bat build --workspace
REM   tools\build.bat test -p dbpf
REM   tools\build.bat run -p spore-bevy -- "C:\path\to\Data\Spore_Game.package"

setlocal

set "MSVC=C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231"
set "SDK=C:\Program Files (x86)\Windows Kits\10"
set "SDKVER=10.0.26100.0"

if not exist "%MSVC%\bin\Hostx64\x64\link.exe" (
    echo ERROR: MSVC link.exe not found at "%MSVC%\bin\Hostx64\x64\link.exe".
    echo Install Visual Studio with the "Desktop development with C++" workload.
    exit /b 1
)

REM Prepend MSVC bin so the real link.exe is found before Git's GNU link.
set "PATH=%MSVC%\bin\Hostx64\x64;%PATH%"

set "LIB=%MSVC%\lib\x64;%SDK%\Lib\%SDKVER%\ucrt\x64;%SDK%\Lib\%SDKVER%\um\x64"
set "INCLUDE=%MSVC%\include;%SDK%\Include\%SDKVER%\shared;%SDK%\Include\%SDKVER%\ucrt;%SDK%\Include\%SDKVER%\um"

cd /d "%~dp0.."
cargo %*
