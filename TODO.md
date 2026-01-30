# TODO

Recommendations from deep dive code review.

## High Priority

- [x] **Env block footgun**: Make `merge_parent_env()` the default, or require explicit opt-out when providing custom env
- [x] **No capability name discovery**: Expose a public list of valid capability names, or add validation at builder level

## Medium Priority

- [x] **No SDDL validation**: Add a validating `AppContainerSid::try_from_sddl()` that checks format
- [x] **ACL inheritance inflexible**: Make directory inheritance flags configurable
- [x] **Resource existence pre-check**: Validate file/dir/key exists before ACL grant; provide actionable error
- [x] **Negative test coverage**: Add tests for error paths (invalid inputs, permission denied, missing resources)

## Low Priority

- [x] **Magic number constants**: Replace `ACE_FLAGS(0x3u32)` and `0x0000_0004` with named constants
- [x] **Inline documentation**: Add `///` doc comments to public struct fields and methods
- [ ] **Legacy `util.rs`**: Migrate remaining callers to `ffi/` wrappers and deprecate `util.rs`
