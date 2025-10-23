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
rustup component add rustfmt | Out-Null
cargo fmt --all -- --check

Write-Host "[ci-local] clippy component"
rustup component add clippy | Out-Null

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

Write-Host "[ci-local] MSRV toolchain"
rustup toolchain install 1.90.0 | Out-Null
rustup component add clippy --toolchain 1.90.0 | Out-Null

foreach ($f in $features) {
  if ($f -eq "") {
    Write-Host "[ci-local] test (msrv 1.90.0, no features)"; cargo +1.90.0 test --all-targets
    Write-Host "[ci-local] clippy (msrv 1.90.0, no features)"; cargo +1.90.0 clippy --all-targets -- -D warnings
  } else {
    Write-Host "[ci-local] test (msrv 1.90.0, features: $f)"; cargo +1.90.0 test --all-targets --features "$f"
    Write-Host "[ci-local] clippy (msrv 1.90.0, features: $f)"; cargo +1.90.0 clippy --all-targets --features "$f" -- -D warnings
  }
}

Write-Host "[ci-local] beta toolchain"
rustup toolchain install beta | Out-Null
rustup component add clippy --toolchain beta | Out-Null

foreach ($f in $features) {
  if ($f -eq "") {
    Write-Host "[ci-local] test (beta, no features)"; cargo +beta test --all-targets; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] beta test failed (no features)" }
    Write-Host "[ci-local] clippy (beta, no features)"; cargo +beta clippy --all-targets -- -D warnings; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] beta clippy failed (no features)" }
  } else {
    Write-Host "[ci-local] test (beta, features: $f)"; cargo +beta test --all-targets --features "$f"; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] beta test failed (features: $f)" }
    Write-Host "[ci-local] clippy (beta, features: $f)"; cargo +beta clippy --all-targets --features "$f" -- -D warnings; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] beta clippy failed (features: $f)" }
  }
}

Write-Host "[ci-local] nightly toolchain"
rustup toolchain install nightly | Out-Null
rustup component add clippy --toolchain nightly | Out-Null

foreach ($f in $features) {
  if ($f -eq "") {
    Write-Host "[ci-local] test (nightly, no features)"; cargo +nightly test --all-targets; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] nightly test failed (no features)" }
    Write-Host "[ci-local] clippy (nightly, no features)"; cargo +nightly clippy --all-targets -- -D warnings; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] nightly clippy failed (no features)" }
  } else {
    Write-Host "[ci-local] test (nightly, features: $f)"; cargo +nightly test --all-targets --features "$f"; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] nightly test failed (features: $f)" }
    Write-Host "[ci-local] clippy (nightly, features: $f)"; cargo +nightly clippy --all-targets --features "$f" -- -D warnings; if ($LASTEXITCODE -ne 0) { Write-Warning "[ci-local] nightly clippy failed (features: $f)" }
  }
}

Write-Host "[ci-local] OK"
