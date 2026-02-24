$ErrorActionPreference = 'Stop'

if (-not $IsWindows) {
  throw "[pre-push] Windows-only hook: non-Windows detected"
}

if (-not (Get-Command just -ErrorAction SilentlyContinue)) {
  throw "[pre-push] Required tool 'just' was not found on PATH."
}

Write-Host "[pre-push] Running just ci-deep"
& just ci-deep
if ($LASTEXITCODE -ne 0) {
  exit $LASTEXITCODE
}

Write-Host "[pre-push] OK"
