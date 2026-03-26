# Plan: autonomous-coverage-maximizer

Date: 2026-02-26
Branch: coverage-max session branch

## Steps
- [ ] Detect stack and run baseline coverage (Rust nextest + llvm-cov)
- [ ] Enumerate uncovered files/lines/functions
- [ ] Classify uncovered items (dead / placeholder / uncoverable / testable)
- [ ] Remove proven dead code only
- [ ] Add tests for coverable paths
- [ ] Add detailed inline comments for remaining uncoverable paths
- [ ] Iterate coverage until no additional gains
- [ ] Final verification (`just ci-fast`, `just ci-deep`)
- [ ] Commit verified change sets locally
