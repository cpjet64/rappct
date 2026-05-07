$ErrorActionPreference = "Stop"

$gitExe = (Get-Command git.exe -ErrorAction Stop).Source
$status = & $gitExe status --short
if ($LASTEXITCODE -ne 0) {
    throw "git status --short failed with exit code $LASTEXITCODE"
}

if ($null -ne $status -and $status.Count -gt 0) {
    Write-Host "[release] Working tree is not clean."
    Write-Host $status
    Write-Host "[release] Commit/stage changes before running clean-release targets (or use allow-dirty targets)."
    exit 1
}

Write-Host "[release] Working tree is clean."
