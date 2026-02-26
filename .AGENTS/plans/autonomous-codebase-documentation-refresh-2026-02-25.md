# Plan: Autonomous Codebase Documentation Refresh

Date: 2026-02-25
Skill: `autonomous-codebase-documenter`

## Objective
Archive existing documentation and regenerate a fresh, comprehensive documentation suite in `./docs/` (including `SPEC.md`) using parallel exploration/analysis and stack-appropriate tooling choices.

## Tooling Selection (from 2026 matrix)
- Stack detected: Rust crate/library.
- Code reference/API generation: `rustdoc` via `cargo doc`.
- Full project/user docs: `mdBook`.
- Rationale: native Rust API fidelity + reproducible guide portal.

## Steps
- [x] Inventory repo docs and read skill/tooling reference.
- [ ] Archive old docs into `legacy/docs/`.
- [ ] Create fresh `docs/` and live `docs/PROGRESS.md`.
- [ ] Spawn parallel `explorer` agents for major areas.
- [ ] Spawn parallel `analyzer` agents for deep module analysis.
- [ ] Generate `docs/SPEC.md` via `spec-writer`.
- [ ] Generate companion docs via `doc-writer` agents.
- [ ] Generate Rust doc artifacts (`cargo doc`) and mdBook structure/build artifacts.
- [ ] Update root `README.md` to point at `docs/`.
- [ ] Validate (fmt/clippy/test + docs generation checks) and commit locally.

## Constraints
- Keep governance docs (`AGENTS.md`, `CLAUDE.md`) in place.
- Do not push.
