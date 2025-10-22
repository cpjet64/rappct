#requires -Version 7
$ErrorActionPreference = 'Stop'
$isWindows = $env:OS -eq 'Windows_NT'
if (-not $isWindows) {
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

Write-Host "[ci-local] OK"
