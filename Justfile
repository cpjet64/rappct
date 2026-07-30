set windows-shell := ["powershell.exe", "-NoProfile", "-NoLogo", "-Command"]
crate_name := env_var_or_default("RAPPCT_CRATE", "rappct")

# === Modes ===

# Pre-commit: fast checks (~10-30s)
ci-fast: hygiene size fmt lint build test-quick coverage

# Pre-push: exhaustive checks (~5-15min)
ci-deep: ci-fast test-full coverage security docs

ci-pre-commit: ci-fast

# Remote branch/MR gate: excludes local-only fmt, duplicate coverage, docs, and security scans.
ci-remote-fast: hygiene lint-remote build test-full

# === Release flow ===
release-version-check:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -Command "& ./scripts/release_version_check.ps1 -Crate {{crate_name}}"

verify-version:
    node scripts/verify-version-surfaces.cjs

prepare-release version:
    powershell.exe -NoProfile -NoLogo -ExecutionPolicy Bypass -File ./scripts/prepare-release.ps1 -Version {{version}}

prepare-release-dry-run version:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -File ./scripts/prepare-release.ps1 -Version {{version}} -DryRun

create-release-tag version:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -File ./scripts/create-release-tag.ps1 -Version {{version}}

test-release-flow:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -File ./scripts/tests/release-flow.Tests.ps1

api-compat:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -File ./scripts/check-api-compat.ps1

release-surface:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -File ./scripts/check-release-surface.ps1

package-list:
    cargo package --list --allow-dirty --locked

package-list-clean: ensure-clean-tree
    cargo package --list --locked

package-release-evidence:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -File ./scripts/package-release-evidence.ps1

sbom:
    python scripts/generate_sbom.py

publish-dry-run:
    cargo publish --dry-run --allow-dirty --locked

publish-dry-run-clean: ensure-clean-tree
    cargo publish --dry-run --locked

release-gate: verify-version release-version-check api-compat release-surface ci-deep package-list-clean publish-dry-run-clean

release-gate-log:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -Command "& ./scripts/release_gate.ps1 -Crate {{crate_name}}"

release: release-gate-log
    powershell.exe -NoProfile -NoLogo -ExecutionPolicy Bypass -Command "& ./scripts/release.ps1 -Crate {{crate_name}} -SkipGate"

ensure-clean-tree:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -File ./scripts/ensure_clean_tree.ps1

# === Repo Hygiene ===
hygiene:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -Command "& ./scripts/hygiene.ps1"

size:
    python scripts/check_code_size.py

# === Rust Recipes ===
fmt:
    cargo fmt --all -- --check

lint:
    cargo clippy --all-targets --all-features --locked -- -D warnings
    cargo machete

lint-remote:
    cargo clippy --all-targets --all-features --locked -- -D warnings

build:
    cargo build --all-targets --all-features --locked

test-quick:
    cargo nextest run --test-threads 1 --locked

test-full:
    cargo nextest run --test-threads 1 --all-features --locked

coverage:
    cargo llvm-cov nextest --test-threads 1 --all-features --ignore-filename-regex '(^|[\\/])(tests|examples|target|external|legacy)[\\/]' --fail-under-regions 85 --lcov --output-path lcov.info

security: sbom
    cargo deny check
    cargo audit
    python scripts/enforce_advisory_policy.py

docs:
    $env:RUSTFLAGS='-D warnings'; cargo doc --locked --no-deps --all-features
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -Command "& { if (-not (Get-Command mdbook -ErrorAction SilentlyContinue)) { Write-Error 'mdbook is required for the docs gate. Install with: cargo install mdbook --locked'; exit 1 }; mdbook build docs --dest-dir book }"

bench:
    cargo bench --locked

clean:
    cargo clean

# === Frontend (uncomment for mixed projects) ===
# fmt-frontend:
#     pnpm prettier --check .
# lint-frontend:
#     pnpm eslint .
# test-frontend:
#     pnpm vitest run

# === Python (uncomment for Python projects) ===
# fmt-python:
#     uv run ruff format --check .
# lint-python:
#     uv run ruff check .
# test-python:
#     uv run pytest
