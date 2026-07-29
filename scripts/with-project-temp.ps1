param(
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]*$')]
    [string]$Scope = 'task',

    [ValidateRange(1, [long]::MaxValue)]
    [long]$MinimumFreeBytes = 5GB,

    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Executable,

    [Parameter(Position = 1, ValueFromRemainingArguments = $true)]
    [string[]]$CommandArguments
)

$ErrorActionPreference = 'Stop'
$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$driveRoot = [System.IO.Path]::GetPathRoot($repoRoot)
$driveInfo = [System.IO.DriveInfo]::new($driveRoot)

if (-not $driveInfo.IsReady) {
    throw "[project-temp] Repository volume is not ready: $driveRoot"
}
if ($driveInfo.AvailableFreeSpace -lt $MinimumFreeBytes) {
    $message = (
        "[project-temp] Insufficient free space on {0}: {1:N0} bytes available; " +
        "{2:N0} bytes required for scope '{3}'. Free space before retrying."
    ) -f $driveRoot, $driveInfo.AvailableFreeSpace, $MinimumFreeBytes, $Scope
    throw $message
}

$scratchParent = Join-Path $repoRoot (".tmp\tasks\{0}" -f $Scope)
$scratchPath = Join-Path $scratchParent ("run-{0}-{1}" -f $PID, [guid]::NewGuid().ToString('N'))
[void](New-Item -ItemType Directory -Path $scratchPath -Force)

$previousTemp = [Environment]::GetEnvironmentVariable('TEMP', 'Process')
$previousTmp = [Environment]::GetEnvironmentVariable('TMP', 'Process')
$previousTmpDir = [Environment]::GetEnvironmentVariable('TMPDIR', 'Process')
$previousActive = [Environment]::GetEnvironmentVariable('RAPPCT_PROJECT_TEMP_ACTIVE', 'Process')
$commandExitCode = 1
$nativeArguments = @(
    $CommandArguments | ForEach-Object {
        # PowerShell consumes a bare `--` while binding script parameters.
        # Callers use `::` when the native command needs that separator.
        if ($_ -eq '::') { '--' } else { $_ }
    }
)

try {
    [Environment]::SetEnvironmentVariable('TEMP', $scratchPath, 'Process')
    [Environment]::SetEnvironmentVariable('TMP', $scratchPath, 'Process')
    [Environment]::SetEnvironmentVariable('TMPDIR', $scratchPath, 'Process')
    [Environment]::SetEnvironmentVariable('RAPPCT_PROJECT_TEMP_ACTIVE', '1', 'Process')

    & $Executable @nativeArguments
    $commandExitCode = $LASTEXITCODE
    if ($null -eq $commandExitCode) {
        $commandExitCode = if ($?) { 0 } else { 1 }
    }
} finally {
    [Environment]::SetEnvironmentVariable('TEMP', $previousTemp, 'Process')
    [Environment]::SetEnvironmentVariable('TMP', $previousTmp, 'Process')
    [Environment]::SetEnvironmentVariable('TMPDIR', $previousTmpDir, 'Process')
    [Environment]::SetEnvironmentVariable(
        'RAPPCT_PROJECT_TEMP_ACTIVE',
        $previousActive,
        'Process'
    )

    if (Test-Path -LiteralPath $scratchPath) {
        $resolvedScratch = (Get-Item -LiteralPath $scratchPath).FullName
        $resolvedParent = [System.IO.Path]::GetFullPath($scratchParent).TrimEnd(
            [System.IO.Path]::DirectorySeparatorChar,
            [System.IO.Path]::AltDirectorySeparatorChar
        ) + [System.IO.Path]::DirectorySeparatorChar
        if (-not $resolvedScratch.StartsWith(
            $resolvedParent,
            [System.StringComparison]::OrdinalIgnoreCase
        )) {
            throw "[project-temp] Refusing to clean scratch outside $resolvedParent`: $resolvedScratch"
        }
        Remove-Item -LiteralPath $resolvedScratch -Recurse -Force
    }
}

exit $commandExitCode
