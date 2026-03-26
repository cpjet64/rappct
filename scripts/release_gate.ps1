[CmdletBinding()]
param(
    [string]$Crate = "rappct"
)

$ErrorActionPreference = "Stop"
$GitExe = (Get-Command git.exe -ErrorAction Stop).Source

function Invoke-GitChecked {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    & $script:GitExe @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "git command failed: git $($Arguments -join ' ') (exit $LASTEXITCODE)"
    }
}

$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$logDir = Join-Path (Get-Location) "output\release-gate"
$logPath = Join-Path $logDir ("release-gate-{0}.log" -f $timestamp)
$transcriptStarted = $false

New-Item -ItemType Directory -Force -Path $logDir | Out-Null
Start-Transcript -Path $logPath -NoClobber | Out-Null
$transcriptStarted = $true

try {
    Write-Host "[release-gate] crate: $Crate"
    Write-Host "[release-gate] log: $logPath"
    Write-Host ""
    Write-Host "[release-gate] git branch"
    Invoke-GitChecked @("branch", "-a")
    Write-Host ""
    Write-Host "[release-gate] git worktree list"
    Invoke-GitChecked @("worktree", "list")
    Write-Host ""
    Write-Host "[release-gate] git status --short"
    Invoke-GitChecked @("status", "--short")
    Write-Host ""

    $env:RAPPCT_CRATE = $Crate
    Write-Host "[release-gate] using crate override via RAPPCT_CRATE=$Crate"
    & just release-gate
    if ($LASTEXITCODE -ne 0) {
        throw "just release-gate failed with exit code $LASTEXITCODE"
    }

    Write-Host ""
    Write-Host "Release gate passed. Log: $logPath"
}
catch {
    Write-Error "Release gate failed. See transcript: $logPath"
    throw
}
finally {
    if ($transcriptStarted) {
        Stop-Transcript | Out-Null
    }
}
