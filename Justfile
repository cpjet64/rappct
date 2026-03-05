set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
crate_name := env_var_or_default("RAPPCT_CRATE", "rappct")

# === Modes ===

# Pre-commit: fast checks (~10-30s)
ci-fast: hygiene fmt lint build test-quick coverage

# Pre-push: exhaustive checks (~5-15min)
ci-deep: ci-fast test-full coverage security docs

ci-pre-commit: ci-fast

# === Release flow ===
release-version-check:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -Command "& ./scripts/release_version_check.ps1 -Crate {{crate_name}}"

package-list:
    cargo package --list --allow-dirty --locked

package-list-clean: ensure-clean-tree
    cargo package --list --locked

publish-dry-run:
    cargo publish --dry-run --allow-dirty --locked

publish-dry-run-clean: ensure-clean-tree
    cargo publish --dry-run --locked

release-gate: release-version-check ci-fast package-list-clean publish-dry-run-clean

release-gate-log:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -Command "& ./scripts/release_gate.ps1 -Crate {{crate_name}}"

release-publish:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -Command "& ./scripts/release.ps1 -Crate {{crate_name}} -SkipGate"

release: release-gate-log release-publish

ensure-clean-tree:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -Command "& { if (git status --short | Select-Object -First 1) { Write-Host '[release] Working tree is not clean.'; git status --short; Write-Host '[release] Commit/stage changes before running clean-release targets (or use allow-dirty targets).'; exit 1 }; Write-Host '[release] Working tree is clean.' }"

# === Repo Hygiene ===
hygiene:
    powershell.exe -NoProfile -NoLogo -NonInteractive -ExecutionPolicy Bypass -Command "& ./scripts/hygiene.ps1"

# === Rust Recipes ===
fmt:
    cargo fmt --check

lint:
    cargo clippy --all-targets --all-features -- -D warnings
    cargo machete

build:
    cargo build --all-targets --all-features --locked

test-quick:
    cargo nextest run --locked

test-full:
    cargo nextest run --all-features --locked

coverage:
    cargo llvm-cov nextest --all-features --ignore-filename-regex 'src[\\](acl|capability|diag|error|ffi[\\](attr_list|handles|mem|sec_caps|sid|wstr)|launch[\\]mod|net|profile|token|util)[.]rs$' --fail-under-regions 95 --lcov --output-path lcov.info

security:
    cargo deny check
    cargo audit
    python scripts/enforce_advisory_policy.py

docs:
    cmd /c "set RUSTFLAGS=-D warnings && cargo doc --no-deps --all-features"

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
