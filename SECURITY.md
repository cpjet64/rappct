# Security Policy

Thank you for helping keep rappct and its users safe. This document explains
how to report vulnerabilities and what you can expect during disclosure.

## Supported Versions

- Stable releases on `main`: the latest minor (e.g., 0.13.x) receives fixes.
- Previous minor (e.g., 0.12.x) may receive critical security fixes when feasible.
- Development prereleases on `dev` (tags like `dev-vX.Y.Z`) are best‑effort and
  not supported for security backports.

We do not commit to supporting EOL minor series. If a fix is impractical, we
may recommend upgrading to a patched release.

## Reporting a Vulnerability (Private)

Please use GitHub’s private advisory flow:

- Open the repository’s “Security” tab → “Advisories” → “Report a vulnerability”, or
- Direct link: https://github.com/cpjet64/rappct/security/advisories/new

Include:
- A minimal proof‑of‑concept (Rust code or steps) to reproduce
- Expected vs. actual behavior, and why it’s a security impact
- OS and toolchain details (Windows build, Rust version/MSRV, features used)
- Any logs or diagnostics (sanitized)

If you cannot use the advisory form, open a general issue stating that you have
a security report and we’ll reach out with a private channel. Do not include
vulnerability details in a public issue.

## Disclosure & Response

- Acknowledge receipt within 3 business days.
- Initial triage within 7 business days (impact assessment and next steps).
- Coordinate a fix and release timeline with you; we aim for prompt remediation.
- Publish a GitHub Security Advisory (GHSA) with credits (opt‑out available on request).

If the issue originates in a dependency (e.g., `windows` crate), we will
coordinate upstream as needed and track the resolution here.

## Scope & Guidelines

This crate is Windows‑only and focuses on AppContainer/LPAC helpers, secure
process launch, ACL utilities, and optional network helpers. Reports we actively
triage include (non‑exhaustive):

- Unsoundness or memory safety issues stemming from FFI usage
- Privilege escalation or policy bypass caused by library APIs
- Incorrect ACL or firewall mutations beyond what an API explicitly documents
- Insecure defaults not aligned with the documentation

Out of scope examples:

- Expected AppContainer/LPAC restrictions that break arbitrary programs
- Misuse of the optional `net` feature without explicit confirmation calls
- Broad Windows policy misconfigurations outside the library’s control

## Safe‑Harbor

We support good‑faith research and coordinated disclosure. While testing:

- Do not access or modify user data you do not own.
- Do not perform denial‑of‑service or destructive testing on shared hosts.
- Follow local laws and the GitHub Terms of Service.

## MSRV & Reproducibility

- MSRV is 1.90 (documented in Cargo.toml and README). Repro reports that build
  on MSRV or stable are preferred.
- Tests that require privileged changes (e.g., firewall exemptions) should be
  clearly labeled and minimized in PoCs.

We appreciate your time and care in reporting; thank you for helping improve rappct.

