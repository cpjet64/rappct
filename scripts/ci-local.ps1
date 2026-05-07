#requires -Version 7
$ErrorActionPreference = 'Stop'
# Use PowerShell's built-in read-only $IsWindows automatic variable
if (-not $IsWindows) {
  Write-Error "[ci-local] Windows-only checks. Detected non-Windows environment. Aborting."; exit 1
}

function Invoke-Checked {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Label,
    [Parameter(Mandatory = $true)]
    [scriptblock]$Action
  )

  & $Action
  if ($LASTEXITCODE -ne 0) {
    throw "[ci-local] $Label failed with exit code $LASTEXITCODE"
  }
}

$features = @("", "introspection", "net", "introspection,net")
$env:RUST_BACKTRACE = "1"
$env:RUSTFLAGS = "-D warnings"

Write-Host "[ci-local] fmt (stable, workspace)"
# rustfmt and clippy must be pre-installed. Do NOT run 'rustup component add'
# during builds — it mutates the shared RUSTUP_HOME and causes contention
# when multiple repos build concurrently.
# To provision: rustup component add rustfmt clippy
Invoke-Checked -Label "fmt (stable, workspace)" -Action { cargo fmt --all -- --check }

Write-Host "[ci-local] clippy check"

foreach ($f in $features) {
  if ($f -eq "") {
    Write-Host "[ci-local] test (stable, no features)"; Invoke-Checked -Label "test (stable, no features)" -Action { cargo test --all-targets }
    Write-Host "[ci-local] clippy (stable, no features)"; Invoke-Checked -Label "clippy (stable, no features)" -Action { cargo clippy --all-targets -- -D warnings }
    Invoke-Checked -Label "duplicate dependency check (stable, no features)" -Action { cargo tree -d | Out-Null }
  } else {
    Write-Host "[ci-local] test (stable, features: $f)"; Invoke-Checked -Label "test (stable, features: $f)" -Action { cargo test --all-targets --features "$f" }
    Write-Host "[ci-local] clippy (stable, features: $f)"; Invoke-Checked -Label "clippy (stable, features: $f)" -Action { cargo clippy --all-targets --features "$f" -- -D warnings }
    Invoke-Checked -Label "duplicate dependency check (stable, features: $f)" -Action { cargo tree -d --features "$f" | Out-Null }
  }
}

$msrvList = @("1.88.0", "1.89.0", "1.90.0", "1.91.0", "1.92.0", "1.93.0")

foreach ($msrv in $msrvList) {
  Write-Host "[ci-local] toolchain $msrv"
  # Toolchain must be pre-installed. Do NOT install during builds.
  # To provision: rustup toolchain install $msrv && rustup component add clippy --toolchain $msrv
  if (-not (rustup toolchain list | Select-String "^$msrv")) {
    Write-Warning "[ci-local] toolchain $msrv not installed, skipping"; continue
  }

  foreach ($f in $features) {
    if ($f -eq "") {
      Write-Host "[ci-local] test ($msrv, no features)"; Invoke-Checked -Label "test ($msrv, no features)" -Action { cargo +$msrv test --all-targets }
      Write-Host "[ci-local] clippy ($msrv, no features)"; Invoke-Checked -Label "clippy ($msrv, no features)" -Action { cargo +$msrv clippy --all-targets -- -D warnings }
    } else {
      Write-Host "[ci-local] test ($msrv, features: $f)"; Invoke-Checked -Label "test ($msrv, features: $f)" -Action { cargo +$msrv test --all-targets --features "$f" }
      Write-Host "[ci-local] clippy ($msrv, features: $f)"; Invoke-Checked -Label "clippy ($msrv, features: $f)" -Action { cargo +$msrv clippy --all-targets --features "$f" -- -D warnings }
    }
  }
}

Write-Host "[ci-local] beta toolchain"
# Toolchain must be pre-installed. To provision: rustup toolchain install beta && rustup component add clippy --toolchain beta
$hasBeta = rustup toolchain list | Select-String '^beta'

if ($hasBeta) {
  foreach ($f in $features) {
    if ($f -eq "") {
      Write-Host "[ci-local] test (beta, no features)"; Invoke-Checked -Label "test (beta, no features)" -Action { cargo +beta test --all-targets }
      Write-Host "[ci-local] clippy (beta, no features)"; Invoke-Checked -Label "clippy (beta, no features)" -Action { cargo +beta clippy --all-targets -- -D warnings }
    } else {
      Write-Host "[ci-local] test (beta, features: $f)"; Invoke-Checked -Label "test (beta, features: $f)" -Action { cargo +beta test --all-targets --features "$f" }
      Write-Host "[ci-local] clippy (beta, features: $f)"; Invoke-Checked -Label "clippy (beta, features: $f)" -Action { cargo +beta clippy --all-targets --features "$f" -- -D warnings }
    }
  }
} else {
  Write-Warning "[ci-local] beta toolchain not installed, skipping. To provision: rustup toolchain install beta && rustup component add clippy --toolchain beta"
}

Write-Host "[ci-local] nightly toolchain"
# Toolchain must be pre-installed. To provision: rustup toolchain install nightly && rustup component add clippy --toolchain nightly
$hasNightly = rustup toolchain list | Select-String '^nightly'

if ($hasNightly) {
  foreach ($f in $features) {
    if ($f -eq "") {
      Write-Host "[ci-local] test (nightly, no features)"; Invoke-Checked -Label "test (nightly, no features)" -Action { cargo +nightly test --all-targets }
      Write-Host "[ci-local] clippy (nightly, no features)"; Invoke-Checked -Label "clippy (nightly, no features)" -Action { cargo +nightly clippy --all-targets -- -D warnings }
    } else {
      Write-Host "[ci-local] test (nightly, features: $f)"; Invoke-Checked -Label "test (nightly, features: $f)" -Action { cargo +nightly test --all-targets --features "$f" }
      Write-Host "[ci-local] clippy (nightly, features: $f)"; Invoke-Checked -Label "clippy (nightly, features: $f)" -Action { cargo +nightly clippy --all-targets --features "$f" -- -D warnings }
    }
  }
} else {
  Write-Warning "[ci-local] nightly toolchain not installed, skipping. To provision: rustup toolchain install nightly && rustup component add clippy --toolchain nightly"
}

Write-Host "[ci-local] OK"
