# Plan: Autonomous Codebase Documenter Refresh

Date: 2026-02-26
Skill: `autonomous-codebase-documenter`

## Objective
Archive current docs and generate a fresh full documentation set under `./docs` including `SPEC.md`, architecture, API, tooling, and index documents.

## Tooling Selection
- Stack: Rust + PowerShell scripts.
- API/reference docs: `rustdoc` via `cargo doc`.
- Project/user docs: `mdBook`.

## Steps
- [x] Inventory current docs and load toolchain reference.
- [ ] Archive existing docs into `legacy/docs/`.
- [ ] Initialize fresh docs workspace and progress log.
- [ ] Run parallel exploration and deep analysis agents.
- [ ] Generate docs files (`SPEC.md`, `README.md`, `ARCHITECTURE.md`, `API.md`, `TOOLING.md`, `index.md`).
- [ ] Generate docs artifacts and validate commands.
- [ ] Verify and commit locally.
