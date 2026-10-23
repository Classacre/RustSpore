[CmdletBinding()]
param(
    [ValidateSet('inspect', 'trace', 'function')]
    [string] $Operation = 'inspect',
    [string] $BinaryPath = 'D:\Games\SPORE Collection\SporeBin\SporeApp.exe',
    [string] $Query,
    [string] $Procedure,
    [string] $GhidraInstallDir,
    [switch] $Focused,
    [ValidateRange(1, 2147483)]
    [int] $StartupTimeoutSeconds = 1200,
    [string] $OutputDirectory,
    [string] $SnapshotPath
)

# Analysis results may contain proprietary game data. Keep them under ignored target/.
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
if (-not $OutputDirectory) {
    $OutputDirectory = Join-Path $repoRoot ('target\rea\' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff'))
}
if (-not $GhidraInstallDir) {
    $GhidraInstallDir = $env:GHIDRA_INSTALL_DIR
    if (-not $GhidraInstallDir) {
        $GhidraInstallDir = [Environment]::GetEnvironmentVariable('GHIDRA_INSTALL_DIR', 'User')
    }
}
if (-not $GhidraInstallDir -or -not (Test-Path -LiteralPath (Join-Path $GhidraInstallDir 'Ghidra\application.properties'))) {
    throw 'Pass -GhidraInstallDir pointing to the installed Ghidra release.'
}
if (-not (Test-Path -LiteralPath $BinaryPath -PathType Leaf)) {
    throw "Binary not found: $BinaryPath"
}
if (-not $SnapshotPath) {
    $targetDigest = (Get-FileHash -LiteralPath $BinaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $profileSuffix = if ($Focused) { '.focused' } else { '' }
    $SnapshotPath = Join-Path $repoRoot ("target\rea\$targetDigest$profileSuffix.snapshot.json")
}
if ($Operation -eq 'trace' -and [string]::IsNullOrWhiteSpace($Query)) {
    throw 'trace requires -Query.'
}
if ($Operation -eq 'function' -and [string]::IsNullOrWhiteSpace($Procedure)) {
    throw 'function requires -Procedure.'
}
if (Test-Path -LiteralPath $OutputDirectory) {
    throw "Choose a new output directory: $OutputDirectory"
}
$null = Get-Command npx.cmd -ErrorAction Stop
$null = New-Item -ItemType Directory -Path $OutputDirectory
$snapshotParent = Split-Path -Parent ([IO.Path]::GetFullPath($SnapshotPath))
$null = New-Item -ItemType Directory -Force -Path $snapshotParent
$outputPath = Join-Path $OutputDirectory 'evidence.json'
$errorPath = Join-Path $OutputDirectory 'stderr.log'
$oldInstall = $env:GHIDRA_INSTALL_DIR
$oldTimeout = $env:REA_GHIDRA_STARTUP_TIMEOUT_MS
$oldDisabled = $env:REA_GHIDRA_DISABLED_ANALYZERS
try {
    $env:GHIDRA_INSTALL_DIR = $GhidraInstallDir
    $env:REA_GHIDRA_STARTUP_TIMEOUT_MS = [string]($StartupTimeoutSeconds * 1000)
    if ($Focused) {
        # Reduced coverage: no automatic stack-variable recovery, propagated
        # constant references, or decompiler switch/jump-table recovery.
        $env:REA_GHIDRA_DISABLED_ANALYZERS = 'Stack,x86 Constant Reference Analyzer,Decompiler Switch Analysis'
    }
    $reaArguments = @('-y', 'rea-agents@6.5.0', $Operation, ([IO.Path]::GetFullPath($BinaryPath)))
    if ($Operation -eq 'trace') { $reaArguments += "--query=$Query" }
    if ($Operation -eq 'function') { $reaArguments += "--procedure=$Procedure" }
    $reaArguments += @('--provider', 'ghidra', '--snapshot', ([IO.Path]::GetFullPath($SnapshotPath)), '--format', 'json')
    Write-Host "REA 6.5.0 / Ghidra: $Operation; startup budget ${StartupTimeoutSeconds}s"
    & npx.cmd @reaArguments 2> $errorPath | Set-Content -LiteralPath $outputPath -Encoding utf8
    $reaExitCode = $LASTEXITCODE
    if ($reaExitCode -ne 0) {
        throw "REA exited $reaExitCode. Inspect $outputPath and $errorPath"
    }
    $null = Get-Content -LiteralPath $outputPath -Raw | ConvertFrom-Json
    Write-Host "Evidence: $outputPath"
    Write-Host "Snapshot: $SnapshotPath"
} finally {
    $env:GHIDRA_INSTALL_DIR = $oldInstall
    $env:REA_GHIDRA_STARTUP_TIMEOUT_MS = $oldTimeout
    $env:REA_GHIDRA_DISABLED_ANALYZERS = $oldDisabled
}
