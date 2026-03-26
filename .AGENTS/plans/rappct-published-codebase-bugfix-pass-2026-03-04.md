# 2026-03-04 - published crate full codebase bugfix pass

## Plan

- [x] Perform a full codebase review for correctness defects beyond release/publish flow.
- [x] Fix capability metadata sync gaps (back-compat list completeness and lookup behavior).
- [x] Fix launch-related environment merge and env-block construction edge cases.
- [x] Fix network loopback SID ownership/lifetime in `set_loopback`.
- [x] Harden registry root parsing in ACL grant helper.
- [x] Harden SID pointer constructor API to reject null pointers and propagate `Result`.
- [x] Fix compile break introduced during constructor migration (`??` typo in profile SID construction).
- [x] Harden UTF-16 block decoding in test env parsing helpers to avoid conversion panics.
- [x] Add regression tests for each corrected behavior.
- [ ] Re-run `cargo fmt` / lint / compile and targeted tests after edits.
- [ ] Perform final pass for any missed high-impact issues and document follow-up.

## Execution notes

- Focus is on correctness and memory-safety issues that are independent from publish flow.
- Changes are intentionally local and minimal in scope to avoid altering API behavior beyond correctness fixes.

## Findings and completion notes (2026-03-05)

- Correctness regressions identified and fixed:
  - `src/profile.rs`: removed duplicated `??` after `OwnedSid::from_freesid_psid(sid_ptr)`.
  - `src/launch/env.rs`: replaced `String::from_utf16(...).unwrap()` in tests with `from_utf16_lossy`.
- Safety hardening completed:
  - `src/ffi/sid.rs`: `from_localfree_psid` and `from_freesid_psid` now return `Result<Self>` and return `AcError::InvalidSid` for null pointers.
- Targeted regression coverage remains in place from previous pass (`src/ffi/sid.rs`, plus launch/capability/net related tests).
- Next action before merge remains running project quality gates (`fmt`, `clippy`, `tests`) and recording results in this file and `.AGENTS/todo.md`.
