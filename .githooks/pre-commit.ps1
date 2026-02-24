$ErrorActionPreference = 'Stop'

if (-not $IsWindows) {
  throw "[pre-commit] Windows-only hook: non-Windows detected"
}

if (-not (Get-Command just -ErrorAction SilentlyContinue)) {
  throw "[pre-commit] Required tool 'just' was not found on PATH."
}

Write-Host "[pre-commit] Running just ci-fast"
& just ci-fast
if ($LASTEXITCODE -ne 0) {
  exit $LASTEXITCODE
}

Write-Host "[pre-commit] OK"
