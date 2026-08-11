# Release and Deployment Review

> Historical audit snapshot. Findings may have been remediated; use `RELEASE-CHECKLIST.md`, `CHANGELOG.md`, and fresh validation evidence for current status.

## Current Release Model

Readable current root docs describe a local-only crates.io release flow:

- `Justfile` exposes `release-version-check`, package listing, dry-run, clean release gate, transcript logging, and `release`.
- `scripts/release.ps1` requires branch `main`, clean working tree, explicit typed `PUBLISH`, and cargo registry credentials or typed `FORCE`.
- `RELEASE-CHECKLIST.md` says GitHub-hosted publish workflows have been removed.

Legacy docs conflict with this:

- `legacy/docs/root/WORKFLOW.md` describes GitHub Actions release workflow publishing to GitHub Releases and crates.io.
- Some legacy examples and docs are corrupted and should not be used as operational guidance.

## Release Findings

| ID | Finding | Classification | Evidence | Risk | Required action |
| --- | --- | --- | --- | --- | --- |
| D1 | Release source of truth conflicts with legacy docs | stale | current `RELEASE-CHECKLIST.md` vs `legacy/docs/root/WORKFLOW.md` | Wrong release process can be followed. | Mark legacy release docs archived/stale or remove from operator path. |
| D2 | Release checklist is incomplete | partial | `RELEASE-CHECKLIST.md` has unchecked clean-tree gate, dry-run-clean, release-gate, release-gate-log, release. | Publish not ready. | Complete checklist after source restoration. |
| D3 | Version baseline is stale | stale | latest crates.io baseline recorded as `0.13.3` on 2026-03-04. | Version may not be greater than current published crate. | Re-run live version check. |
| D4 | Changelog and release checklist versions diverge | partial/stale | `CHANGELOG.md` latest release section `0.13.4`; release target `0.13.10`. | Release notes may be incomplete. | Update changelog for actual release candidate. |
| D5 | Publish package include policy is documented but not freshly verified | partial | `RELEASE-CHECKLIST.md` lists include allow-list. | Accidental files may enter crate. | Run `cargo package --list --locked` on clean tree. |
| D6 | Hosted release workflow status unclear | stale | docs say removed, `.github/workflows` listing only showed `ci.yml` and `codeql.yml`. | Fine if local-only, but docs must be consistent. | Confirm no publish workflow exists and document local-only policy. |
| D7 | CodeQL workflow corrupted | broken | `.github/workflows/codeql.yml` content invalid-looking | Deployment/security quality signal missing. | Restore before release. |
| D8 | Release gate will fail until repository is clean and restored | broken | corrupted source plus requested report artifacts make current tree not release-clean. | Cannot release safely. | Restore, commit intended changes, run clean-tree gates. |

## Required Release Gate

After restoration and documentation reconciliation:

```powershell
just release-version-check
just package-list-clean
just publish-dry-run-clean
just release-gate-log
```

Only after the logged gate passes and release notes are current:

```powershell
just release
```

## Deployment Notes

This is a Rust library crate with examples and no long-running service deployment surface. Production deployment means:

- Published crate contents are minimal and intentional.
- Docs.rs/rustdoc build succeeds.
- crates.io version is semver-correct and greater than published baseline.
- Examples and CLI compile and run on supported Windows hosts.
- Security-sensitive optional operations are documented with privilege and host-mutation requirements.
