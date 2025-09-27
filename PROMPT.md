# PROMPT.md - Agent kickoff prompt (short form)

You are an expert Windows internals + Rust engineer. Implement the Windows portions of `rappct`.
Do not add scope. Keep unsafe in leaf modules. Follow RULES.md and TODO.md.

Priorities:
1) Implement Windows AppContainer profile functions with correct memory ownership:
   - CreateAppContainerProfile (fallback to DeriveAppContainerSidFromAppContainerName on ERROR_ALREADY_EXISTS)
   - DeleteAppContainerProfile
   - GetAppContainerFolderPath (CoTaskMemFree)
   - GetAppContainerNamedObjectPath
2) Capabilities:
   - DeriveCapabilitySidsFromName (LocalFree for returned arrays)
   - Map KnownCapability -> names
3) Launch:
   - STARTUPINFOEX + PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES
   - LPAC via PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY = PROCESS_CREATION_ALL_APPLICATION_PACKAGES_OPT_OUT
   - RAII for attribute list; stdio pipe support
4) Token:
   - GetTokenInformation: TokenIsAppContainer, TokenIsLessPrivilegedAppContainer, TokenAppContainerSid, TokenCapabilities
5) ACL:
   - SetNamedSecurityInfo for files/dirs; SetSecurityInfo for registry
6) Net (feature=net):
   - NetworkIsolationEnumAppContainers, NetworkIsolationSetAppContainerConfig
   - Force explicit acknowledgement for loopback exemption
7) Diagnostics + job objects (later)

Deliver increments that compile. Add tests behind cfg(windows) and feature gates.
