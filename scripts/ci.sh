#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# ci.sh — Local CI pipeline (Bash)
#
# Prerequisites (install once):
#   cargo install cargo-nextest
#   cargo install cargo-audit
#   cargo install cargo-deny
#   cargo install cargo-machete
#   cargo install cargo-outdated
# ---------------------------------------------------------------------------
set -euo pipefail

start_time=$SECONDS

STEPS=10
FAILED_STEP=""

fail() {
  FAILED_STEP="$1"
  echo ""
  echo "FAILED at step: $FAILED_STEP"
  exit 1
}

banner() {
  local num=$1; shift
  echo ""
  echo "=== [$num/$STEPS] $* ==="
  echo ""
}

# ---- 1. Formatting --------------------------------------------------------
banner 1 "Formatting (cargo fmt --check)"
cargo fmt --all -- --check || fail "1 - Formatting"

# ---- 2. Unused dependencies -----------------------------------------------
banner 2 "Unused dependencies (cargo machete)"
cargo machete || fail "2 - Unused dependencies"

# ---- 3. Lint ---------------------------------------------------------------
banner 3 "Lint (cargo clippy)"
cargo clippy --workspace --all-targets --all-features -- -D warnings || fail "3 - Lint"

# ---- 4. Tests (nextest) ----------------------------------------------------
banner 4 "Unit + integration tests (cargo nextest)"
cargo nextest run --workspace --all-features || fail "4 - Tests"

# ---- 5. Doctests -----------------------------------------------------------
banner 5 "Doctests (cargo test --doc)"
cargo test --doc --workspace --all-features || fail "5 - Doctests"

# ---- 6. Security audit -----------------------------------------------------
banner 6 "Security audit (cargo audit)"
cargo audit || fail "6 - Security audit"

# ---- 7. License & advisory check -------------------------------------------
banner 7 "License & advisory check (cargo deny)"
cargo deny check || fail "7 - License & advisory check"

# ---- 8. Doc build ----------------------------------------------------------
banner 8 "Doc build check (cargo doc)"
cargo doc --workspace --all-features --no-deps || fail "8 - Doc build"

# ---- 9. Outdated dependencies ----------------------------------------------
banner 9 "Outdated dependencies (cargo outdated)"
cargo outdated --exit-code 1 || fail "9 - Outdated dependencies"

# ---- 10. Release build -----------------------------------------------------
banner 10 "Release build (cargo build --release)"
cargo build --release --all-features || fail "10 - Release build"

# ---- Summary ---------------------------------------------------------------
elapsed=$(( SECONDS - start_time ))
echo ""
echo "=== All $STEPS steps passed in ${elapsed}s ==="
