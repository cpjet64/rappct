#!/usr/bin/env bash
set -Eeuo pipefail

# Windows-only guard: require Windows_NT environment (Git Bash / PowerShell)
if [[ "${OS:-}" != "Windows_NT" ]]; then
  echo "[ci-local] Windows-only checks. Detected non-Windows OS: $(uname -s). Aborting." >&2
  exit 1
fi

# Mirror CI matrix: rust = [stable, 1.90.0], features = ["", introspection, net, introspection,net]
features_list=("" "introspection" "net" "introspection,net")

export RUST_BACKTRACE=1
export RUSTFLAGS="-D warnings"

echo "[ci-local] fmt (stable, workspace)"
rustup component add rustfmt >/dev/null || true
cargo fmt --all -- --check

echo "[ci-local] clippy component"
rustup component add clippy >/dev/null || true

for FEATS in "${features_list[@]}"; do
  if [[ -z "$FEATS" ]]; then
    echo "[ci-local] test (stable, no features)"; cargo test --all-targets
    echo "[ci-local] clippy (stable, no features)"; cargo clippy --all-targets -- -D warnings
    cargo tree -d || true
  else
    echo "[ci-local] test (stable, features: $FEATS)"; cargo test --all-targets --features "$FEATS"
    echo "[ci-local] clippy (stable, features: $FEATS)"; cargo clippy --all-targets --features "$FEATS" -- -D warnings
    cargo tree -d --features "$FEATS" || true
  fi
done

echo "[ci-local] MSRV toolchain"
rustup toolchain install 1.90.0 -q >/dev/null || true
rustup component add clippy --toolchain 1.90.0 >/dev/null || true

for FEATS in "${features_list[@]}"; do
  if [[ -z "$FEATS" ]]; then
    echo "[ci-local] test (msrv 1.90.0, no features)"; cargo +1.90.0 test --all-targets
    echo "[ci-local] clippy (msrv 1.90.0, no features)"; cargo +1.90.0 clippy --all-targets -- -D warnings
  else
    echo "[ci-local] test (msrv 1.90.0, features: $FEATS)"; cargo +1.90.0 test --all-targets --features "$FEATS"
    echo "[ci-local] clippy (msrv 1.90.0, features: $FEATS)"; cargo +1.90.0 clippy --all-targets --features "$FEATS" -- -D warnings
  fi
done

echo "[ci-local] beta toolchain"
rustup toolchain install beta -q >/dev/null || true
rustup component add clippy --toolchain beta >/dev/null || true

for FEATS in "${features_list[@]}"; do
  if [[ -z "$FEATS" ]]; then
    echo "[ci-local] test (beta, no features)"; cargo +beta test --all-targets || echo "[ci-local][warn] beta test failed (no features)"
    echo "[ci-local] clippy (beta, no features)"; cargo +beta clippy --all-targets -- -D warnings || echo "[ci-local][warn] beta clippy failed (no features)"
  else
    echo "[ci-local] test (beta, features: $FEATS)"; cargo +beta test --all-targets --features "$FEATS" || echo "[ci-local][warn] beta test failed (features: $FEATS)"
    echo "[ci-local] clippy (beta, features: $FEATS)"; cargo +beta clippy --all-targets --features "$FEATS" -- -D warnings || echo "[ci-local][warn] beta clippy failed (features: $FEATS)"
  fi
done

echo "[ci-local] nightly toolchain"
rustup toolchain install nightly -q >/dev/null || true
rustup component add clippy --toolchain nightly >/dev/null || true

for FEATS in "${features_list[@]}"; do
  if [[ -z "$FEATS" ]]; then
    echo "[ci-local] test (nightly, no features)"; cargo +nightly test --all-targets || echo "[ci-local][warn] nightly test failed (no features)"
    echo "[ci-local] clippy (nightly, no features)"; cargo +nightly clippy --all-targets -- -D warnings || echo "[ci-local][warn] nightly clippy failed (no features)"
  else
    echo "[ci-local] test (nightly, features: $FEATS)"; cargo +nightly test --all-targets --features "$FEATS" || echo "[ci-local][warn] nightly test failed (features: $FEATS)"
    echo "[ci-local] clippy (nightly, features: $FEATS)"; cargo +nightly clippy --all-targets --features "$FEATS" -- -D warnings || echo "[ci-local][warn] nightly clippy failed (features: $FEATS)"
  fi
done

echo "[ci-local] OK"
