## Summary

<!-- Describe the change and why it is needed. -->

## Validation

<!-- Include exact commands, pipeline links, screenshots, or logs. -->

- [ ] `python scripts/check_code_size.py`
- [ ] `python scripts/hygiene.py`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features --locked -- -D warnings`
- [ ] `cargo test --all-targets --locked`
- [ ] Other applicable checks are documented below

## Breaking changes

<!-- Note API changes, MSRV changes, feature changes, or migration needs. -->

## Checklist

- [ ] Documentation is updated where needed
- [ ] Tests cover the changed behavior
- [ ] Windows-specific behavior is validated where applicable
- [ ] The exact delivered SHA has a terminal successful GitLab pipeline
