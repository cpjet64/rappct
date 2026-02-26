# Autonomous Performance Plan (2026-02-26)

## Scope
- Repository-wide optimization pass focused on measurable CPU hot paths in reusable utility code.
- Branch isolation: `perf-opt-1772071285`.

## Candidate Hot Paths
1. `src/launch/env.rs` `make_wide_block`:
   - Repeated lossy-string lowercase allocations inside sort comparator.
   - Avoidable intermediate allocations while building UTF-16 env block.
2. `src/launch/mod.rs` `merge_parent_env`:
   - Repeated linear scan across custom env for each required key.
   - Duplicate `var_os` lookups for same key.
3. `src/sid.rs` `try_from_sddl` (secondary candidate):
   - Parse path likely less hot but can be benchmarked for cheap wins.

## Method
1. Baseline benchmark harness for selected targets.
2. Apply optimization 1; benchmark + correctness checks.
3. Apply optimization 2; benchmark both standalone and combined.
4. Re-run all accepted optimizations together and compare against baseline.
5. Run verification gates (`just ci-fast`, then `just ci-deep`) and commit.

## Stop Criteria
- No additional candidate provides >=0.5% measured improvement across two passes.
- All quality and security gates remain green.
