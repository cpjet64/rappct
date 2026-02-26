# Dependency Upgrade Plan (2026-02-26)

## Scope
- Repository: `C:\Dev\repos\active\rappct`
- Branch: `feat/100pct-coverage`
- Strategy: smallest safe increments, lockfile-first, no major jumps unless required.

## Baseline
- Toolchain: Rust `1.93.1` (`rust-toolchain.toml`)
- Key lockfile versions (pre-upgrade): `clap 4.5.56`, `tempfile 3.24.0`, `windows 0.62.2`, `thiserror 2.0.18`, `tracing 0.1.44`, `serde 1.0.228`, `strsim 0.11.1`
- Quality commands: `just ci-fast`, `just ci-deep`

## Waves
1. Wave 1 (low risk): security + patch/minor dependency updates via `cargo update` (lockfile only).
2. Wave 2 (conditional): targeted crate updates if wave 1 misses known safe minor bumps.
3. Wave 3 (deferred by default): majors/toolchain changes only if explicitly justified by security or breakage.

## Risk Controls
- Run vcvars bootstrap before compiler-dependent validation commands.
- Validate after each wave using repo gates + security checks.
- Commit each verified wave locally with rollback-friendly atomic scope.
- No push.
