#!/usr/bin/env bash
set -Eeuo pipefail

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

echo "[ci-local] OK"

