Local Git hooks

This repository includes a pre-commit hook to run the same checks as CI:

- cargo fmt --all -- --check
- cargo clippy --all-targets --all-features -D warnings
- cargo llvm-cov nextest --all-features --ignore-filename-regex 'src[\\](acl|capability|diag|error|ffi[\\](attr_list|handles|mem|sec_caps|sid|wstr)|launch[\\]mod|net|profile|token|util)[.]rs$' --fail-under-regions 95 (via just coverage)
- cargo test

Enable hooks locally by pointing Git at the hooks directory:

  git config core.hooksPath .githooks

You can disable or override this per-repo as needed.

Pre-push runs `just ci-deep`. To bypass for an emergency push:

  git push --no-verify
