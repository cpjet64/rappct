# Lessons Learned

(Capture patterns and corrections here.)

- Replacing broad `Cargo.toml` `exclude` lists with explicit `include` lists is the most reliable way to keep crate publish tarballs deterministic; file-level excludes were still leaking several root meta files even while listed.
- For published-crate releases, enforce release-preconditions in `release.ps1` (branch + clean working tree) even when local `release-gate` already runs, then keep `PUBLISH` confirmation as final human-authentication step.
- For local-only publish flows, avoid double-running the same gate in release command chaining; `just release` should be `release-gate-log` + `scripts/release.ps1 -SkipGate`.
- Favor direct `git.exe` calls in release scripts to avoid shell alias/function interference when collecting auditable logs.
- After bulk constructor signature migrations (`*sid` constructors), run a narrow `rg` sweep for suspicious operators/prefixes (`??`, unwrapped callsites) before finalizing to catch mechanical syntax regressions early.
- For publish-version checks, parse `Cargo.toml` `version` from the `[package]` section only; manifest-wide regexes can accidentally capture dependency versions.
- For crate docs consistency, validate rustdoc file references against real docs paths before release so generated docs don't link to removed files.
- For `CreateProcessW` custom environment blocks, always emit double-NUL termination even when the caller passes an explicit empty env map; single-NUL empty blocks are invalid and can cause launch failures.
- For Windows capability derivation, do not silently continue when `ConvertSidToStringSidW` fails on returned SIDs; fail closed and surface a concrete error after cleanup so partial capability sets are never treated as success.
