# Local-only publish flow migration plan (rappct)

## Goal
Switch `rappct` from hosted GitHub Actions publish orchestration to a local-only, auditable release path that matches your published-crate process requirements.

## Scope
- Scope is release + publish readiness only (metadata, scripts, workflow wiring, and docs).
- Core crate behavior, API changes, and unrelated CI optimizations are intentionally out of scope.

## Planned changes
- [x] Add local release gate + publish scripts under `scripts/`.
- [x] Add local-release-oriented `Justfile` recipes and a transcript-backed `release-gate-log` target.
- [x] Add explicit version-check logic (local manifest version must be greater than current crates.io version).
- [x] Remove hosted release automation (`.github/workflows/release.yml`, `release-plz.toml`) and stale manifest exclusion entries.
- [x] Remove or correct docs references to release-plz/GitHub release automation.
- [x] Add/update release evidence checklist and task tracking.
- [x] Validate by running local dry-run release gate and capture transcript path.
  - Evidence logged at `output/release-gate/release-gate-2026-03-04_18-24-41.log`.
- [x] **Do not run actual `cargo publish` without explicit user request**.

## Deliverables
- `scripts/release_version_check.ps1`
- `scripts/release_gate.ps1`
- `scripts/release.ps1`
- `Justfile` release targets (`release-version-check`, `package-list`, `publish-dry-run`, `release-gate`, `release-gate-log`, `release`)
- `release-plz.toml` removed
- `.github/workflows/release.yml` removed
- README + tooling/deployment docs updated for local-only flow
- `RELEASE-CHECKLIST.md` (local evidence checklist)
- `.AGENTS/todo.md` entry for execution review notes

## Milestones
1. **Publish readiness baseline**
   - Confirm latest published crates.io version via `cargo search`.
   - Confirm local package metadata and hygiene (`Cargo.toml`).

2. **Release mechanics implementation**
   - Implement preflight check + local dry-run chain.
   - Implement logged gate command writing to `output/release-gate/<timestamp>.log`.

3. **Remove hosted publish path**
   - Delete workflow+config tied to release-plz.
   - Update docs references.

4. **Evidence and verification package**
   - Update checklists/planning artifacts.
   - Run `just release-gate-log`.
   - Record log path and results.

## Notes
- No publish token actions are added.
- Real publish path remains manual and confirmation-gated.
- `just release-gate-log` execution completed locally (transcript above).
