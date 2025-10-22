# Changelog

## [0.13.0] - 2025-10-22

### Bug Fixes

- Edition 2024, remove unused imports, avoid LocalFree import; make set_loopback warnings non-fatal
- 2024 edition updates – mark extern blocks unsafe, wrap unsafe ops, clean imports, correct OwnedHandle::into_file; update CI should pass
- 2024 unsafe rules – annotate unsafe fns, fix EqualSid result handling, silence dead_code in feature stubs, rename unused field; pass -D warnings
- Update to googleapis/release-please-action@v4
- Make release-please wait for CI to pass
- Rollback windows crate to 0.60 for crates.io compatibility
- Add docs.rs metadata for Windows-only crate cross-compilation
- Configure release-please-dev to target dev branch instead of main
- Prevent release-please workflows from running on their own merge commits
- Configure dev branch with separate tag prefix to avoid conflicts
- Checkout correct tag when publishing to crates.io
- Move tag-prefix to root level in release-please-dev config
- Explicitly configure tag-prefix for main branch
- Use release-tag-prefix for proper tag naming
- Create dev config file instead of using inline TOML
- Remove Cargo.lock from .gitignore since it's committed to repo
- Use 'config' instead of 'config_path' in release-plz-action
- Remove tracked files from .gitignore and add release-plz-dev.toml

### Documentation

- Add comprehensive status badges for main and dev branches to README
- Update README to show crates.io installation as primary method
- Clarify that both dev and main publish to crates.io
- Remove dev publishing to crates.io, switch to git-only distribution
- Add cargo add syntax for specific version installation
- Add examples section to README with usage instructions

### Features

- Add automatic crates.io publishing
- Implement dual-branch workflow with automated dev releases
- Migrate from release-please to release-plz

### Miscellaneous Tasks

- Bootstrap community setup
- V0.9.0 – API ergonomics, Windows CI, wide-string helpers, net softening, docs, release-please setup, ignore Cargo.lock
- Ignore rust analyzer cache
- Bump to 0.9.1\n\nContext:\n* Prepare for example improvements and API-guideline polish.\n\nChange:\n* Bump crate version to 0.9.1.\n\nRisk:\n* Low; version metadata only.\n\nTests:\n* Ran cargo test --all-features successfully.\n\nVerification:\n* cargo test --all-features passed locally; clippy clean.
- Normalize line endings and add .gitattributes
- Release rappct 0.10.0
- Release rappct 0.11.0
- Release rappct 0.12.0
- Release 0.13.0-dev.0
- Release 0.13.1-dev.0
- Reset dev branch version to 0.12.2
- Update Cargo.lock for version 0.12.2
- Update bootstrap-sha to prevent analyzing old BREAKING CHANGE commits
- Release rappct 0.12.3
- Clean CHANGELOG-DEV for fresh start after tag-prefix fix
- Reset dev version to 0.12.2 to sync with main
- Update bootstrap-sha to current commit

### Testing

- Mark extern blocks unsafe for Rust 2024; CI green

### Build

- Release-please config for pre-1.0 breaking changes as minor

### Ci

- Add Clippy step; token/util 2024 unsafe fixes for Windows; net EqualSid handling
- Add cargo tree -d visibility; tests: wrap std::env set/remove var in unsafe for Rust 2024 CI

### Examples

- Fix rust-analyzer warnings\n\nContext:\n* rust-analyzer flagged env var calls and iterator usage in examples.\n\nChange:\n* advanced_features: wrap std::env set/remove in unsafe blocks.\n* network_demo/comprehensive_demo: use map_while(Result::ok) for lines.\n\nRisk:\n* None to library behavior; examples only.\n\nTests:\n* cargo check/clippy/test all features pass.\n\nVerification:\n* clippy clean; examples build on Windows.
- Eliminate unreachable expression by returning inside cfg blocks\n\nContext:\n* rust-analyzer flagged unreachable trailing Ok(()) when early-returning in cfg blocks.\n\nChange:\n* advanced_features: return Ok(()) inside cfg(feature) blocks and remove trailing Ok(()).\n* network_demo: return Ok(()) inside net-enabled block to avoid unreachable tail.\n\nRisk:\n* None; examples only.\n\nVerification:\n* cargo check/clippy/test --all-features pass locally.
- Refactor cfg blocks to avoid unreachable/needless_return warnings\n\nContext:\n* rust-analyzer flagged unreachable Ok(()); clippy flagged needless_return in cfg blocks.\n\nChange:\n* advanced_features, network_demo: convert tail returns in cfg blocks to tail expressions; ensure function ends on the block expression.\n\nRisk:\n* None; examples only.\n\nVerification:\n* cargo clippy/test --all-features clean and passing.
- Gate imports behind feature flags to satisfy rust-analyzer
- Split cfg-sensitive demos into dedicated wrappers
- Run curl as the AppContainer process instead of via cmd; remove device/NUL interactions\n\nContext:\n* InternetClient was granted but HTTP failed with 'Access is denied' when launching curl via cmd inside the container.\n\nChange:\n* Launch curl.exe directly as the AppContainer child (host creates the process), avoiding second-level process creation restrictions.\n* Drop redirections to NUL; use -I/-s and rely on exit behavior/headers.\n\nResult:\n* HTTP headers printed successfully under AppContainer with InternetClient.\n\nVerification:\n* Ran cargo run --example rappct_demo and observed 200 OK headers.
- Clarify net feature is for localhost loopback testing; not required for outbound Internet\n\nDocs:\n* Update pro tip text to explain that --features net enables loopback helpers, often needs admin, and is optional for outbound HTTP.
- Demonstrate localhost denial then success with loopback exemption; add tiny local HTTP server\n\nChange:\n* Spawn a minimal TCP listener on 127.0.0.1 and test from AppContainer:\n  - First without loopback (failure)\n  - Then after enabling loopback with net feature (success).\n* Keep outbound HTTP as a separate step to show InternetClient usage.\n\nDocs:\n* Clarify net is for loopback; not needed for outbound.\n\nVerification:\n* Ran demo with and without --features net; behavior matches expectations.
- Clarify narration and add live captured output for loopback + outbound tests\n\nChange:\n* Use launch_in_container_with_io to capture curl stdout; print exit codes and headers.\n* Clearer step texts (baseline, loopback block, loopback allow, outbound).\n* Tiny localhost server demo shows deny/allow progression.\n\nVerification:\n* Ran demo with --features net: outputs clearly reflect each phase.



## Dev Branch Changelog

All notable changes to the dev branch will be documented in this file.

This changelog tracks pre-release versions published from the `dev` branch.
For stable releases, see [CHANGELOG.md](CHANGELOG.md).

The format is based on Keep a Changelog and this file will be managed automatically by release-please.
