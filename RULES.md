# RULES.md - Engineering rules for rappct

1. **No magic**. If the OS requires a capability, we expose that to callers and fail otherwise.
2. **Fail closed**. Unknown capability names are errors with suggestions; never silently ignore.
3. **LPAC must be explicit**. No hidden defaults or auto-upgrades from AC to LPAC.
4. **Loopback exemptions are DEV ONLY**. Force an explicit acknowledgement before applying.
5. **RAII for everything**. Attribute lists, handles, and SIDs are owned types with `Drop`.
6. **Unsafe is quarantined** to clearly named modules; keep public APIs safe.
7. **Docs are part of the surface**. Any sharp edge has a doc comment and a README note.
8. **Cross-platform stubs**. On non-Windows, the same APIs exist and return `UnsupportedPlatform`.
9. **No `unwrap`** in lib code. Return `AcError` with context and hints.
10. **Test what matters**. Token flags, network isolation behaviors, capability derivation, ACLs.
