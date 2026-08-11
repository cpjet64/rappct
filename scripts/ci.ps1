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
$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$minimumFreeBytes = 5GB

function Assert-FreeSpace {
    $driveRoot = [System.IO.Path]::GetPathRoot($repoRoot)
    $driveInfo = [System.IO.DriveInfo]::new($driveRoot)
    if (-not $driveInfo.IsReady) {
        throw "[ci] Repository volume is not ready: $driveRoot"
    }
    if ($driveInfo.AvailableFreeSpace -ge $minimumFreeBytes) {
        return
    }

    throw (
        "[ci] Insufficient free space on {0}: {1:N0} bytes available; " +
        "{2:N0} bytes required. Free space before running CI."
    ) -f $driveRoot, $driveInfo.AvailableFreeSpace, $minimumFreeBytes
}

$startTime = Get-Date
$Steps = 12
$FailedStep = $null

Set-Location -LiteralPath $repoRoot
Assert-FreeSpace

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

# ---- 2. Hygiene ------------------------------------------------------------
Banner 2 "Repository hygiene"
& (Join-Path $PSScriptRoot 'hygiene.ps1')
if ($LASTEXITCODE -ne 0) { Fail "2 - Hygiene" }

# ---- 3. Code size ----------------------------------------------------------
Banner 3 "Code size"
python (Join-Path $PSScriptRoot 'check_code_size.py')
if ($LASTEXITCODE -ne 0) { Fail "3 - Code size" }

# ---- 4. Unused dependencies -----------------------------------------------
Banner 4 "Unused dependencies (cargo machete)"
cargo machete
if ($LASTEXITCODE -ne 0) { Fail "4 - Unused dependencies" }

# ---- 5. Lint ---------------------------------------------------------------
Banner 5 "Lint (cargo clippy)"
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
if ($LASTEXITCODE -ne 0) { Fail "5 - Lint" }

# ---- 6. Tests (nextest) ----------------------------------------------------
Banner 6 "Unit + integration tests (cargo nextest)"
cargo nextest run --workspace --all-features --locked --test-threads 1
if ($LASTEXITCODE -ne 0) { Fail "6 - Tests" }

# ---- 7. Doctests -----------------------------------------------------------
Banner 7 "Doctests (cargo test --doc)"
cargo test --doc --workspace --all-features --locked
if ($LASTEXITCODE -ne 0) { Fail "7 - Doctests" }

# ---- 8. Security audit -----------------------------------------------------
Banner 8 "Security audit (cargo audit)"
cargo audit --db .cache/rustsec-advisory-db
if ($LASTEXITCODE -ne 0) { Fail "8 - Security audit" }

# ---- 9. License & advisory check -------------------------------------------
Banner 9 "License & advisory check (cargo deny)"
cargo deny check
if ($LASTEXITCODE -ne 0) { Fail "9 - License & advisory check" }

# ---- 10. Doc build ---------------------------------------------------------
Banner 10 "Doc build check (cargo doc)"
cargo doc --workspace --all-features --locked --no-deps
if ($LASTEXITCODE -ne 0) { Fail "10 - Doc build" }

# ---- 11. Outdated dependencies ---------------------------------------------
Banner 11 "Outdated dependencies (cargo outdated)"
cargo outdated -R --depth 1 --exit-code 1
if ($LASTEXITCODE -ne 0) { Fail "11 - Outdated dependencies" }

# ---- 12. Release build -----------------------------------------------------
Banner 12 "Release build (cargo build --release)"
cargo build --release --all-features --locked
if ($LASTEXITCODE -ne 0) { Fail "12 - Release build" }

# ---- Summary ---------------------------------------------------------------
$elapsed = (Get-Date) - $startTime
$totalSeconds = [math]::Round($elapsed.TotalSeconds)
Write-Host ""
Write-Host "=== All $Steps steps passed in ${totalSeconds}s ===" -ForegroundColor Green
