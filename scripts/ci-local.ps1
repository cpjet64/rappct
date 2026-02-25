#requires -Version 7
$ErrorActionPreference = 'Stop'
# Use PowerShell's built-in read-only $IsWindows automatic variable
if (-not $IsWindows) {
  Write-Error "[ci-local] Windows-only checks. Detected non-Windows environment. Aborting."; exit 1
}
$features = @("", "introspection", "net", "introspection,net")
$env:RUST_BACKTRACE = "1"
$env:RUSTFLAGS = "-D warnings"

Write-Host "[ci-local] fmt (stable, workspace)"
# rustfmt and clippy must be pre-installed. Do NOT run 'rustup component add'
# during builds — it mutates the shared RUSTUP_HOME and causes contention
# when multiple repos build concurrently.
# To provision: rustup component add rustfmt clippy
cargo fmt --all -- --check

Write-Host "[ci-local] clippy check"

foreach ($f in $features) {
  if ($f -eq "") {
    Write-Host "[ci-local] test (stable, no features)"; cargo test --all-targets
    Write-Host "[ci-local] clippy (stable, no features)"; cargo clippy --all-targets -- -D warnings
    cargo tree -d | Out-Null
  } else {
    Write-Host "[ci-local] test (stable, features: $f)"; cargo test --all-targets --features "$f"
    Write-Host "[ci-local] clippy (stable, features: $f)"; cargo clippy --all-targets --features "$f" -- -D warnings
    cargo tree -d --features "$f" | Out-Null
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
      Write-Host "[ci-local] test ($msrv, no features)"; cargo +$msrv test --all-targets
      Write-Host "[ci-local] clippy ($msrv, no features)"; cargo +$msrv clippy --all-targets -- -D warnings
    } else {
      Write-Host "[ci-local] test ($msrv, features: $f)"; cargo +$msrv test --all-targets --features "$f"
      Write-Host "[ci-local] clippy ($msrv, features: $f)"; cargo +$msrv clippy --all-targets --features "$f" -- -D warnings
    }
  }
}

Write-Host "[ci-local] beta toolchain"
# Toolchain must be pre-installed. To provision: rustup toolchain install beta && rustup component add clippy --toolchain beta
$hasBeta = rustup toolchain list | Select-String '^beta'

if ($hasBeta) {
  foreach ($f in $features) {
    if ($f -eq "") {
      Write-Host "[ci-local] test (beta, no features)"; cargo +beta test --all-targets; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] beta test failed (no features)" }
      Write-Host "[ci-local] clippy (beta, no features)"; cargo +beta clippy --all-targets -- -D warnings; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] beta clippy failed (no features)" }
    } else {
      Write-Host "[ci-local] test (beta, features: $f)"; cargo +beta test --all-targets --features "$f"; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] beta test failed (features: $f)" }
      Write-Host "[ci-local] clippy (beta, features: $f)"; cargo +beta clippy --all-targets --features "$f" -- -D warnings; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] beta clippy failed (features: $f)" }
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
      Write-Host "[ci-local] test (nightly, no features)"; cargo +nightly test --all-targets; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] nightly test failed (no features)" }
      Write-Host "[ci-local] clippy (nightly, no features)"; cargo +nightly clippy --all-targets -- -D warnings; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] nightly clippy failed (no features)" }
    } else {
      Write-Host "[ci-local] test (nightly, features: $f)"; cargo +nightly test --all-targets --features "$f"; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] nightly test failed (features: $f)" }
      Write-Host "[ci-local] clippy (nightly, features: $f)"; cargo +nightly clippy --all-targets --features "$f" -- -D warnings; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] nightly clippy failed (features: $f)" }
    }
  }
} else {
  Write-Warning "[ci-local] nightly toolchain not installed, skipping. To provision: rustup toolchain install nightly && rustup component add clippy --toolchain nightly"
}

Write-Host "[ci-local] OK"
