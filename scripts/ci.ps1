#requires -Version 7
# ---------------------------------------------------------------------------
# ci.ps1 — Local CI pipeline (PowerShell)
#
# Prerequisites (install once):
#   cargo install cargo-nextest
#   cargo install cargo-audit
#   cargo install cargo-deny
#   cargo install cargo-machete
#   cargo install cargo-outdated
# ---------------------------------------------------------------------------
$ErrorActionPreference = 'Stop'

$startTime = Get-Date
$Steps = 10
$FailedStep = $null

function Fail($step) {
    $script:FailedStep = $step
    Write-Host ""
    Write-Host "FAILED at step: $step" -ForegroundColor Red
    exit 1
}

function Banner($num, $name) {
    Write-Host ""
    Write-Host "=== [$num/$Steps] $name ===" -ForegroundColor Cyan
    Write-Host ""
}

# ---- 1. Formatting --------------------------------------------------------
Banner 1 "Formatting (cargo fmt --check)"
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { Fail "1 - Formatting" }

# ---- 2. Unused dependencies -----------------------------------------------
Banner 2 "Unused dependencies (cargo machete)"
cargo machete
if ($LASTEXITCODE -ne 0) { Fail "2 - Unused dependencies" }

# ---- 3. Lint ---------------------------------------------------------------
Banner 3 "Lint (cargo clippy)"
cargo clippy --workspace --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) { Fail "3 - Lint" }

# ---- 4. Tests (nextest) ----------------------------------------------------
Banner 4 "Unit + integration tests (cargo nextest)"
cargo nextest run --workspace --all-features
if ($LASTEXITCODE -ne 0) { Fail "4 - Tests" }

# ---- 5. Doctests -----------------------------------------------------------
Banner 5 "Doctests (cargo test --doc)"
cargo test --doc --workspace --all-features
if ($LASTEXITCODE -ne 0) { Fail "5 - Doctests" }

# ---- 6. Security audit -----------------------------------------------------
Banner 6 "Security audit (cargo audit)"
cargo audit
if ($LASTEXITCODE -ne 0) { Fail "6 - Security audit" }

# ---- 7. License & advisory check -------------------------------------------
Banner 7 "License & advisory check (cargo deny)"
cargo deny check
if ($LASTEXITCODE -ne 0) { Fail "7 - License & advisory check" }

# ---- 8. Doc build ----------------------------------------------------------
Banner 8 "Doc build check (cargo doc)"
cargo doc --workspace --all-features --no-deps
if ($LASTEXITCODE -ne 0) { Fail "8 - Doc build" }

# ---- 9. Outdated dependencies ----------------------------------------------
Banner 9 "Outdated dependencies (cargo outdated)"
cargo outdated --exit-code 1
if ($LASTEXITCODE -ne 0) { Fail "9 - Outdated dependencies" }

# ---- 10. Release build -----------------------------------------------------
Banner 10 "Release build (cargo build --release)"
cargo build --release --all-features
if ($LASTEXITCODE -ne 0) { Fail "10 - Release build" }

# ---- Summary ---------------------------------------------------------------
$elapsed = (Get-Date) - $startTime
$totalSeconds = [math]::Round($elapsed.TotalSeconds)
Write-Host ""
Write-Host "=== All $Steps steps passed in ${totalSeconds}s ===" -ForegroundColor Green
